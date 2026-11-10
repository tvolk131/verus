#!/usr/bin/env python3
"""Identity/coverage audit, NEVER a native semantic acceptance checker."""
import argparse
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
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, publication=False, native_bridge_proved=False)
    with replay_receipt(a.output, receipt):
        src = HERE.parent / 'native_heap_lowering/fixtures/drop.rs'
        paths = [a.extraction/n for n in ('drop-native.json', 'drop-borrowck.json', 'receipt.json')]
        guard = ArtifactGuard([*paths, src, Path(__file__), HERE.parent/'native_heap_lowering/lower.py', *support_artifacts()])
        require(json.loads(paths[2].read_text())['success'] is True, 'unhealthy source extraction')
        mapping = build(json.loads(paths[0].read_text()), json.loads(paths[1].read_text()), src)
        out = a.output/'source-obligations.json'
        out.write_text(json.dumps(mapping, indent=2)+'\n')
        receipt.update(artifacts=guard.initial, output_sha256=digest(out),
            native_operations=44, unproved_rejected=44)
        guard.check()
