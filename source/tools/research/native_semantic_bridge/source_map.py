#!/usr/bin/env python3
"""Identity/coverage audit, NEVER a native semantic acceptance checker."""
import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
sys.path.insert(0, str(HERE.parent / 'native_heap_lowering'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import require, replay_receipt
from lower import Lower, sha


def source_audit_inputs():
    """Required pretrial inputs, independently of a caller's freeze entries."""
    return [Path(__file__), HERE/'audit.py', HERE/'source-bindings.json',
            HERE.parent/'native_heap_lowering/lower.py',
            HERE.parent/'native_heap_lowering/fixtures/drop.rs',
            *support_artifacts()]


def validate_source_freeze(freeze, initial):
    require(freeze['before_first_trial'] is True, 'missing pretrial freeze')
    required = {str(p.relative_to(ROOT)) for p in source_audit_inputs()}
    require(required <= set(freeze['files']), 'incomplete frozen source-audit inputs')
    for name, value in freeze['files'].items():
        require(initial[str((ROOT/name).resolve())]['sha256'] == value,
                'frozen source changed '+name)


def guard_source_freeze(freeze_path, extraction):
    """Validate exactly the snapshot retained for the final stability check."""
    frozen_bytes = freeze_path.read_bytes()
    freeze = json.loads(frozen_bytes)
    paths = [extraction/n for n in ('drop-native.json', 'drop-borrowck.json', 'receipt.json')]
    guard = ArtifactGuard([*paths, *source_audit_inputs(), freeze_path,
                           *[ROOT/n for n in freeze['files']]])
    require(guard.initial[str(freeze_path.resolve())]['sha256'] ==
            hashlib.sha256(frozen_bytes).hexdigest(), 'freeze changed while loading')
    validate_source_freeze(freeze, guard.initial)
    return guard


def authenticated_extraction(directory):
    """Bind this audit to the independently reviewed, frozen extraction outputs."""
    pin = json.loads((HERE / 'source-bindings.json').read_text())
    require(set(pin['outputs']) == {'drop-native.json', 'drop-borrowck.json'},
            'incomplete extraction output bindings')
    receipt = directory / 'receipt.json'
    require(digest(receipt) == pin['receipt_sha256'], 'unbound extraction receipt')
    require(json.loads(receipt.read_text())['success'] is True, 'unhealthy source extraction')
    for name, expected in pin['outputs'].items():
        require(digest(directory / name) == expected, 'unbound extraction output: ' + name)


def borrowck_chain(native, borrowck):
    """Check statements and operands, independently of the separate loan metadata."""
    loan = borrowck['loans'][0]
    blocks = {b['bb']: b for b in borrowck['blocks']}
    require(len(blocks) == len(borrowck['blocks']) and set(blocks) == {0, 1, 2},
            'borrowck block inventory')
    start = blocks[0]
    require(start['cleanup'] is False, 'borrowck entry cleanup')
    statements = start['statements']
    reserve = loan['reserve']
    require(reserve == dict(bb=0, statement=3) and len(statements) == 6,
            'borrowck loan issue position')
    issue = statements[3]
    require(issue['location'] == reserve, 'borrowck issue location')
    reborrow = issue['ir']
    require(reborrow['op'] == 'assign' and reborrow['destination'] == loan['assigned'],
            'borrowck reborrow destination')
    value = reborrow['value']
    require(value['op'] == 'ref' and value['place'] == loan['borrowed']
            and value['mutable'] is True and value['fake'] is False
            and value['kind'] == 'Mut { kind: TwoPhaseBorrow }', 'borrowck reborrow operand')
    unsize = statements[4]['ir']
    require(unsize['op'] == 'assign' and unsize['value']['op'] == 'unsize'
            and unsize['value']['operand'] == dict(op='move', place=loan['assigned']),
            'borrowck unsize source')
    dest = unsize['destination']
    require(dest == dict(local=3, projection=[], ty='&mut [u8]')
            and unsize['value']['to'] == dest['ty'], 'borrowck unsize destination')
    call = start['terminator']
    require(call['op'] == 'call' and call['arguments'] == [dict(op='move', place=dest)],
            'borrowck constructor operand')
    root = next(i for i in native['instances'] if i['id'] == native['root'])
    first = next(b for b in root['blocks'] if b['bb'] == 0)['term_ir']
    require(call['arguments'] == first['arguments'] and call['destination'] == first['destination']
            and call['function']['ty'] == first['function']['ty'], 'borrowck/native constructor chain')
    require(start['call']['definition'] == 'core::slice::<impl [T]>::iter_mut',
            'borrowck constructor identity')

# Candidates are pointers into actual defined semantic rules, NOT simulations.
RULES = {
    'storage_live': ('local lifetime', 'MIR StorageLive/dead stack-frame simulation'),
    'storage_dead': ('local lifetime', 'MIR storage end is not loan/prophecy resolution'),
    'ref': ('borrow.tctx_reborrow_uniq', 'two-phase source loan/region interpretation and C/F import'),
    'Unsize': ('none', 'slice fat pointer/array coercion representation and exact C/F'),
    'PtrMetadata': ('none', 'Rust fat pointer metadata extraction'),
    'raw_ptr': ('raw_view.typed_source_raw_view', 'native RawPtr provenance and retag simulation'),
    'Transmute': ('none', 'transparent NonNull nonzero-valid-range and pointer provenance'),
    'PtrToPtr': ('none', 'fat-to-thin pointer representation and provenance'),
    'Offset': ('lang.BinOpOffset', 'finite byte-address in-allocation isize checked Rust offset; model is unbounded'),
    'aggregate': ('product.prod_ty', 'MIR ADT layout/PhantomData/move and retained loan representation'),
    'use': ('programs.type_path_instr', 'MIR copy/move, Use::retag and zero-sized unit interpretation'),
    'call': ('function.type_call', 'resolved native callee simulation, stack/return/unwind composition'),
    'return': ('cont.type_jump', 'actual return/drop-glue and same-original-loan final observation'),
    'goto': ('programs.type_seq', 'MIR control-flow transition simulation'),
    'switch': ('programs.type_if', 'CTFE evaluation and branch exclusion correspondence'),
}

def build(native, borrowck, source):
    # Reuse the existing source-binding eligibility checker, not its heap semantics.
    lower = Lower(native, borrowck, digest(source))
    borrowck_chain(native, borrowck)
    plan = lower.build()
    rows = []
    for instance in native['instances']:
        for block in instance['blocks']:
            require(block['cleanup'] is False, 'unexpected native cleanup')
            operations = [(str(n), s['ir']) for n, s in enumerate(block['statements'])]
            operations.append(('term', block['term_ir']))
            for location, ir in operations:
                v = ir.get('value', ir)
                kind = v.get('kind') if isinstance(v.get('kind'), str) else None
                tag = kind if kind in RULES else v['op']
                require(tag in RULES, 'unknown primitive: ' + tag)
                candidate, obligation = RULES[tag]
                rows.append(dict(instance=instance['id'], bb=block['bb'], location=location,
                    payload_sha256=sha(ir), primitive=tag, model_rule_candidate=candidate,
                    simulation_status='unproved_rejected', obligation=obligation))
    require(len(rows) == 44 and len(native['instances']) == 6, 'frozen source graph size')
    require(len(plan['steps']) == 49, 'inherited model trace size')
    phase = []
    for block in borrowck['blocks']:
        for statement in block['statements']:
            if statement['ir']['op'] == 'unsupported_statement':
                phase.append(dict(location=statement['location'], ir=statement['ir'],
                    status='rejected_no_phase_erasure_proof'))
        if block['cleanup']:
            phase.append(dict(bb=block['bb'], ir=block['terminator'],
                status='rejected_no_exception_simulation'))
        if block['terminator']['op'] == 'call':
            phase.append(dict(bb=block['bb'], unwind=block['terminator']['unwind'],
                status='rejected_no_no_unwind_proof'))
    require(len(phase) == 3, 'pre-borrowck phase differences changed')
    return dict(schema=1, native_operations=rows, primitive_counts=dict(Counter(r['primitive'] for r in rows)),
        instance_count=6, operation_count=44, model_steps=49,
        source_loan=borrowck['loans'][0], borrowck_phase_obligations=phase,
        native_effects_discharged=0, publication=False, native_bridge_proved=False,
        note='Model-rule candidates do not certify MIR meanings; all native primitives reject.')

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--extraction', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--freeze', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, publication=False, native_bridge_proved=False)
    with replay_receipt(a.output, receipt):
        guard = guard_source_freeze(a.freeze, a.extraction)
        src = HERE.parent / 'native_heap_lowering/fixtures/drop.rs'
        paths = [a.extraction/n for n in ('drop-native.json', 'drop-borrowck.json', 'receipt.json')]
        authenticated_extraction(a.extraction)
        mapping = build(json.loads(paths[0].read_text()), json.loads(paths[1].read_text()), src)
        out = a.output/'source-obligations.json'
        out.write_text(json.dumps(mapping, indent=2)+'\n')
        receipt.update(artifacts=guard.initial, output_sha256=digest(out),
            native_operations=44, unproved_rejected=44)
        guard.check()
