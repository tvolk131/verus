#!/usr/bin/env python3
"""Inspect checked existing rule boundaries. This never accepts a native theorem."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
BRIDGE = HERE.parent / 'native_semantic_bridge'
sys.path[:0] = [str(ROOT / 'source/tools'), str(BRIDGE)]
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
from proof_checks import parse_axioms, parse_kernel_summary, names
from source_map import authenticated_extraction, build, source_audit_inputs
from acceptance_inputs import recorded_model_sources

FOUNDATION = [
    'lrust.lang.adequacy.lrust_adequacy',
    'lrust.typing.soundness.type_soundness_closed',
    'lrust.typing.soundness.main_type',
    'lrust.lang.heap.heap_ctx',
    'lrust.lang.lifting.lrustGS_irisGS',
    'lrust.util.non_atomic_cell_map.non_atomic_map_alloc_empty',
    'lrust.util.non_atomic_cell_map.non_atomic_map_insert',
    'lrust.lifetime.lifetime_full.llft_alloc',
    'lrust.prophecy.prophecy.proph_init',
    'lrust.typing.uniq_cmra.uniq_init',
    'lrust.typing.uniq_util.alloc_uniq_body',
    'lrust.typing.borrow.type_uniqbor_instr',
    'lrust.typing.borrow.tctx_reborrow_uniq',
    'lrust.typing.type_context.tctx_elt_interp',
    'lrust.typing.programs.typed_instr',
    'lrust.typing.type.ty_own',
    'lrust.typing.uniq_bor.uniq_bor',
]
CONSTANTS = FOUNDATION + ['NativeSemanticBridge.' + n for n in names()]


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, commands=[], publication=False, native_input_proved=False,
                   native_effects_discharged=0, new_theorems=[], new_helpers=[],
                   classification='existing_rule_inspection_not_native_proof')
    with replay_receipt(a.output, receipt):
        frozen_bytes = a.freeze.read_bytes()
        f = json.loads(frozen_bytes)
        require(f['before_first_trial'] is True and f['constants'] == CONSTANTS,
                'unfrozen inspection inventory')
        required = [Path(__file__), HERE/'candidates/entry-v1/EntryInventory.v',
                    BRIDGE/'proof_checks.py', BRIDGE/'acceptance_inputs.py', BRIDGE/'baseline-bindings.json',
                    BRIDGE.parent/'mutable_buffer_goal/foundation-baseline.json',
                    BRIDGE.parent/'mutable_buffer_goal/foundation-pin.json',
                    *source_audit_inputs()]
        require({str(p.relative_to(ROOT)) for p in required} <= set(f['files']),
                'incomplete frozen inspection inputs')
        private = {str(Path(n).resolve()) for n in f['private_inputs']}
        required_private = [Path(f['ready']), Path(f['model_receipt']),
                           *[Path(f['extraction'])/n for n in
                             ('receipt.json', 'drop-native.json', 'drop-borrowck.json')]]
        require({str(p.resolve()) for p in required_private} <= private,
                'incomplete frozen private inspection inputs')
        for name, value in f['files'].items():
            require(digest(ROOT/name) == value, 'freeze mismatch: ' + name)
        for name, value in f['private_inputs'].items():
            require(digest(Path(name)) == value, 'private input mismatch: ' + name)
        ready = json.loads(Path(f['ready']).read_text())
        require(str(Path(ready['receipt']).resolve()) in private,
                'unfrozen baseline receipt')
        trusted = json.loads((BRIDGE.parent/'mutable_buffer_goal/foundation-baseline.json').read_text())
        baseline = Path(ready['build'])
        baseline_result = json.loads(Path(ready['receipt']).read_text())
        require(ready['source_commit'] == '115a618dd35ff8337112cf2081a7f5391d3973d5'
                and ready['baseline_kernel_checked'] is True
                and digest(Path(f['ready'])) == trusted['readiness']['sha256']
                and digest(Path(ready['receipt'])) == trusted['receipt']['sha256']
                and baseline_result['success'] is True, 'untrusted foundation readiness')
        require(len(baseline_result['copied_sources']) == trusted['copied_source_files'],
                'incomplete foundation source inventory')
        for name, value in baseline_result['copied_sources'].items():
            require(digest(baseline/name) == value, 'changed foundation source: ' + name)
        paths = [p for p in (baseline/'src').rglob('*') if p.suffix in ('.v', '.vo')]
        tools = dict(coqc=Path(ready['switch'])/'_opam/bin/coqc',
                     coqchk=Path(ready['switch'])/'_opam/bin/coqchk',
                     runner=Path(ready['runner']), opam=Path(ready['opam']))
        bindings = json.loads((BRIDGE/'baseline-bindings.json').read_text())
        for name, path in tools.items():
            require(digest(path) == bindings['tools'][name], 'changed foundation tool: ' + name)
        object_map = {str(p.relative_to(baseline)): digest(p) for p in paths if p.suffix == '.vo'}
        require(len(object_map) == bindings['compiled_module_count'] and
                hashlib.sha256(json.dumps(object_map, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                == bindings['compiled_module_map_sha256'], 'untrusted baseline objects')
        model = Path(f['model_proofs'])
        model_receipt = json.loads(Path(f['model_receipt']).read_text())
        require(model_receipt['success'] is True and model_receipt['model_proofs_kernel_checked'] is True,
                'model proof setup failure')
        model_sources = recorded_model_sources(model_receipt)
        for module, value in model_receipt['proof_terms'].items():
            require(digest(model/(module+'.vo')) == value, 'changed reviewed model object: '+module)
            require(digest(model/(module+'.v')) == model_sources[module],
                    'changed reviewed model source: '+module)
        paths += list(tools.values()) + [baseline/'_CoqProject', a.freeze, *support_artifacts()]
        paths += [ROOT/n for n in f['files']] + [Path(n) for n in f['private_inputs']]
        paths += [model/(m+s) for m in model_receipt['modules'] for s in ('.v', '.vo')]
        paths += [baseline/name for name in baseline_result['copied_sources']]
        paths += [Path(f['ready']).parent/'gmp-private/manifest.json']
        guard = ArtifactGuard(paths)
        require(guard.initial[str(a.freeze.resolve())]['sha256'] ==
                hashlib.sha256(frozen_bytes).hexdigest(), 'inspection freeze changed while loading')
        for name, value in f['files'].items():
            require(guard.initial[str((ROOT/name).resolve())]['sha256'] == value,
                    'inspection source snapshot mismatch: '+name)
        for name, value in f['private_inputs'].items():
            require(guard.initial[str(Path(name).resolve())]['sha256'] == value,
                    'inspection private snapshot mismatch: '+name)
        for name, value in baseline_result['copied_sources'].items():
            require(guard.initial[str((baseline/name).resolve())]['sha256'] == value,
                    'inspection baseline snapshot mismatch: '+name)
        retained_objects = {str(Path(p).relative_to(baseline)): v['sha256']
                            for p, v in guard.initial.items()
                            if Path(p).suffix == '.vo' and Path(p).is_relative_to(baseline)}
        require(len(retained_objects) == bindings['compiled_module_count'] and
                hashlib.sha256(json.dumps(retained_objects, sort_keys=True,
                    separators=(',', ':')).encode()).hexdigest() == bindings['compiled_module_map_sha256'],
                'untrusted inspection object snapshot')
        for name, path in tools.items():
            require(guard.initial[str(path.resolve())]['sha256'] == bindings['tools'][name],
                    'untrusted inspection tool snapshot: '+name)
        for name, expected in model_receipt['proof_terms'].items():
            require(guard.initial[str((model/(name+'.vo')).resolve())]['sha256'] == expected,
                    'untrusted model object snapshot: '+name)
            require(guard.initial[str((model/(name+'.v')).resolve())]['sha256'] == model_sources[name],
                    'untrusted model source snapshot: '+name)
        source_pin = json.loads((BRIDGE.parent/'mutable_buffer_goal/foundation-pin.json').read_text())
        require(guard.initial[str((baseline/'_CoqProject').resolve())]['sha256'] ==
                source_pin['files']['_CoqProject'], 'untrusted inspection loadpaths')
        receipt['artifacts'] = guard.initial
        extraction = Path(f['extraction'])
        authenticated_extraction(extraction)
        mapping = build(json.loads((extraction/'drop-native.json').read_text()),
                        json.loads((extraction/'drop-borrowck.json').read_text()),
                        BRIDGE.parent/'native_heap_lowering/fixtures/drop.rs')
        require(mapping['operation_count'] == 44 and mapping['native_effects_discharged'] == 0,
                'source audit changed')
        b = json.loads((extraction/'drop-borrowck.json').read_text())
        incoming = next(l for l in b['locals'] if l['local'] == 1)
        require(incoming['ty'] == '&mut [u8; 128]' and b['arg_count'] == 1,
                'wrong incoming argument')
        require(b['loans'][0]['assigned']['local'] == 4 and b['loans'][0]['loan'] == 0,
                'local reborrow changed')
        work = a.output/'inspection'
        work.mkdir()
        target = work/'EntryInventory.v'
        shutil.copyfile(HERE/'candidates/entry-v1/EntryInventory.v', target)
        candidate_guard = ArtifactGuard([target])
        require(candidate_guard.initial[str(target.resolve())]['sha256'] ==
                f['files'][str((HERE/'candidates/entry-v1/EntryInventory.v').relative_to(ROOT))],
                'inspection copy identity mismatch')
        require(not re.search(r'\b(?:Lemma|Theorem|Goal|Definition|Axiom|Admitted|Parameter)\b',
                              target.read_text()), 'inspection introduced a new declaration')
        args = []
        for line in (baseline/'_CoqProject').read_text().splitlines():
            words = shlex.split(line, comments=True)
            if words and words[0] in ('-Q', '-R'):
                require(len(words) == 3, 'unexpected loadpath')
                args += [words[0], str(baseline/words[1]), words[2]]
        args += ['-Q', str(model), 'NativeSemanticBridge', '-Q', str(work), 'NativeSourceEntry']
        env = dict(os.environ)
        run = LoggedCommands(a.output, work, receipt['commands'], timeout=1800, guard=guard)
        require(run('compile-inspection', [str(tools['runner']), str(tools['coqc']), '-q', *args,
                                          str(target)], env) == 0, 'inspection compilation/setup failure')
        assumptions = {}
        statements = {}
        for i, name in enumerate(CONSTANTS):
            assumption_file = work/(f'{i:02d}-assumptions.out')
            statement_file = work/(f'{i:02d}-statement.out')
            assumptions[name] = parse_axioms(assumption_file.read_text())
            statements[name] = dict(path=str(statement_file), sha256=digest(statement_file))
        require(len(assumptions) == 42, 'inspection coverage changed')
        require(run('recursive-kernel', [str(tools['runner']), str(tools['coqchk']), '-silent', '-o',
                                        *args, 'NativeSourceEntry.EntryInventory'], env) == 0,
                'recursive kernel inspection failure')
        kernel = parse_kernel_summary((a.output/'recursive-kernel.stderr').read_text())
        candidate_guard.check()
        guard.check()
        receipt.update(assumptions=assumptions, kernel_axioms=kernel, statements=statements,
                       inspected_constants=len(assumptions), inspection_object_sha256=digest(target.with_suffix('.vo')),
                       outputs={str(p):digest(p) for p in work.iterdir() if p.is_file()},
                       source_entry=dict(input_local=1, reborrow_local=4, unsize_local=3,
                                         local_static_loan=0, incoming_dynamic_loan_recorded=False,
                                         native_simulations_open=44, phase_obligations_open=3),
                       native_effects_remaining=101, semantic_negative_count=0)
        print(json.dumps(dict(output=str(a.output), inspected_constants=len(assumptions), publication=False)))


if __name__ == '__main__':
    main()
