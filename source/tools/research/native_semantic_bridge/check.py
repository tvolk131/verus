#!/usr/bin/env python3
"""Full normal Rocq compilation + transitive kernel check. Never publishes facts."""
import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, require, replay_receipt
from proof_checks import check_inventory, inventory_source, parse_inventory, parse_kernel_summary
PIN = '115a618dd35ff8337112cf2081a7f5391d3973d5'

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--ready', type=Path, required=True)
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, commands=[], publication=False, native_bridge_proved=False,
        model_proofs_kernel_checked=False, native_effects_discharged=0,
        inferred_verus_helpers=[], verus_checks_invoked=False)
    with replay_receipt(a.output, receipt):
        frozen_bytes = a.freeze.read_bytes()
        freeze = json.loads(frozen_bytes)
        require(freeze['before_first_trial'] is True, 'missing pretrial freeze')
        candidate = (ROOT/freeze['candidate_directory']).resolve()
        require(candidate.is_relative_to(HERE/'candidates'), 'proof source outside candidate directory')
        modules = freeze['modules']
        check_inventory(candidate, modules)
        required = [Path(__file__), HERE/'proof_checks.py', HERE/'source-bindings.json',
                    HERE/'baseline-bindings.json', HERE.parent/'mutable_buffer_goal/foundation-baseline.json',
                    HERE.parent/'mutable_buffer_goal/foundation-pin.json',
                    *[candidate/(name+'.v') for name in modules]]
        require({str(p.relative_to(ROOT)) for p in required} <= set(freeze['files']),
                'incomplete frozen trust inputs')
        for name, value in freeze['files'].items():
            require(digest(ROOT/name) == value, 'freeze mismatch '+name)
        ready = json.loads(a.ready.read_text())
        require(ready['source_commit'] == PIN and ready['baseline_kernel_checked'] is True,
                'unready/wrong foundation')
        baseline_receipt = Path(ready['receipt'])
        baseline_result = json.loads(baseline_receipt.read_text())
        require(baseline_result['success'] is True,
                'baseline final receipt did not succeed')
        trusted = json.loads((HERE.parent/'mutable_buffer_goal/foundation-baseline.json').read_text())
        require(digest(a.ready) == trusted['readiness']['sha256'], 'untrusted readiness marker')
        require(digest(baseline_receipt) == trusted['receipt']['sha256'],
                'untrusted baseline receipt')
        successful_stages = {c['label'] for c in baseline_result.get('commands', []) if c['returncode'] == 0}
        require({'prover-version', 'build-baseline', 'kernel-check-baseline'} <= successful_stages,
                'incomplete baseline commands')
        baseline = Path(ready['build']).resolve()
        original = Path('/tmp/verus-infer-buffer-cycle-20261109/verus-belt-115a618d')
        copied = json.loads(baseline_receipt.read_text())['copied_sources']
        originals = subprocess.check_output(['git', 'ls-files', 'src'], cwd=original, text=True).splitlines()
        require(len(copied) == trusted['copied_source_files'] == len(originals) > 0
                and set(copied) == set(originals), 'incomplete baseline source inventory')
        require(subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=original, text=True).strip() == PIN,
                'baseline source commit changed')
        for name, value in copied.items():
            require(digest(baseline/name) == value == digest(original/name),
                    'unmodified baseline source mismatch '+name)
        switchbin = Path(ready['switch'])/'_opam/bin'
        coqc, coqchk = switchbin/'coqc', switchbin/'coqchk'
        wrapper = Path(ready['runner'])
        gmp_manifest = Path(trusted['readiness']['path']).parent/'gmp-private/manifest.json'
        paths = [ROOT/n for n in freeze['files']]
        # Freeze the full compiled foundation and sources for this trial; no
        # installation or baseline writes. Loadpaths point read-only at baseline.
        paths += [p for p in (baseline/'src').rglob('*') if p.suffix in ('.v', '.vo')]
        paths += [baseline/name for name in copied]
        paths += [coqc, coqchk, wrapper, gmp_manifest, Path(ready['opam']), baseline/'_CoqProject', a.ready, baseline_receipt,
                  a.freeze, *support_artifacts()]
        guard = ArtifactGuard(paths)
        # Authenticate the retained snapshot, rather than earlier live reads.
        # A persistent substitution between those reads and this snapshot must
        # not become the new trusted starting state.
        require(guard.initial[str(a.freeze.resolve())]['sha256'] ==
                hashlib.sha256(frozen_bytes).hexdigest(), 'freeze changed while loading')
        for name, value in freeze['files'].items():
            require(guard.initial[str((ROOT/name).resolve())]['sha256'] == value,
                    'freeze mismatch '+name)
        require(guard.initial[str(a.ready.resolve())]['sha256'] == trusted['readiness']['sha256'],
                'untrusted readiness snapshot')
        require(guard.initial[str(baseline_receipt.resolve())]['sha256'] == trusted['receipt']['sha256'],
                'untrusted baseline receipt snapshot')
        for name, value in copied.items():
            require(guard.initial[str((baseline/name).resolve())]['sha256'] == value,
                    'unmodified baseline source snapshot mismatch '+name)
        source_pin = json.loads((HERE.parent/'mutable_buffer_goal/foundation-pin.json').read_text())
        require(guard.initial[str((baseline/'_CoqProject').resolve())]['sha256'] ==
                source_pin['files']['_CoqProject'], 'untrusted foundation loadpaths')
        bindings = json.loads((HERE/'baseline-bindings.json').read_text())
        object_map = {str(Path(p).relative_to(baseline)): v['sha256']
                      for p, v in guard.initial.items()
                      if Path(p).suffix == '.vo' and Path(p).is_relative_to(baseline)}
        encoded = json.dumps(object_map, sort_keys=True, separators=(',', ':')).encode()
        require(len(object_map) == bindings['compiled_module_count']
                and hashlib.sha256(encoded).hexdigest() == bindings['compiled_module_map_sha256'],
                'untrusted baseline proof objects')
        for name, path in [('coqc', coqc), ('coqchk', coqchk), ('runner', wrapper), ('opam', Path(ready['opam']))]:
            require(guard.initial[str(path.resolve())]['sha256'] == bindings['tools'][name],
                    'untrusted baseline tool: '+name)
        receipt['artifacts'] = guard.initial
        run = LoggedCommands(a.output, ROOT, receipt['commands'], timeout=1800, guard=guard)
        env = dict(os.environ, PATH=str(switchbin)+':'+os.environ['PATH'])
        args = []
        for line in (baseline/'_CoqProject').read_text().splitlines():
            words = shlex.split(line, comments=True)
            if words and words[0] in ('-Q', '-R'):
                require(len(words) == 3, 'unexpected foundation loadpath')
                args.extend([words[0], str(baseline/words[1]), words[2]])
        work = a.output/'proofs'
        work.mkdir()
        args += ['-Q', str(work), 'NativeSemanticBridge']
        require(run('version', [str(wrapper), str(coqc), '--version'], env) == 0, 'prover version setup')
        require('version 8.20.0' in (a.output/'version.stdout').read_text(), 'wrong prover')
        source_text = '\n'.join((candidate/(name+'.v')).read_text() for name in modules)
        require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|native_compute|vm_cast_no_check)\b', source_text),
                'unchecked proof construct in selected candidate')
        source_guards = []
        def hold_source(path, expected):
            held = ArtifactGuard([path])
            require(held.initial[str(path.resolve())]['sha256'] == expected,
                    'generated proof source identity mismatch')
            source_guards.append(held)
        for name in modules:
            target = work/(name+'.v')
            shutil.copyfile(candidate/(name+'.v'), target)
            hold_source(target, freeze['files'][str((candidate/(name+'.v')).relative_to(ROOT))])
            require(run('compile-'+name, [str(wrapper), str(coqc), '-q', *args, str(target)], env) == 0,
                    'Rocq script/setup failure (not semantic evidence): '+name)
            require(target.with_suffix('.vo').is_file(), 'missing compiled module '+name)
        inventory = work/'AssumptionInventory.v'
        inventory_text = inventory_source()
        inventory.write_text(inventory_text)
        hold_source(inventory, hashlib.sha256(inventory_text.encode()).hexdigest())
        require(run('assumption-inventory', [str(wrapper), str(coqc), '-q', *args, str(inventory)], env) == 0,
                'complete assumption inventory failed')
        assumptions = parse_inventory((a.output/'assumption-inventory.stdout').read_text())
        outputs = ArtifactGuard([p for p in work.iterdir() if p.suffix in ('.v', '.vo')])
        # Recheck proof terms recursively, with normal kernel checking enabled.
        require(run('kernel-check', [str(wrapper), str(coqchk), '-silent', '-o', *args,
                *['NativeSemanticBridge.'+n for n in modules],
                'NativeSemanticBridge.AssumptionInventory'], env) == 0,
                'transitive kernel check failed')
        kernel_axioms = parse_kernel_summary((a.output/'kernel-check.stderr').read_text())
        outputs.check()
        for source_guard in source_guards:
            source_guard.check()
        receipt.update(model_proofs_kernel_checked=True, modules=modules,
            named_theorem_count=18, byte_type_obligation_count=6,
            assumption_inventory=assumptions, kernel_axioms=kernel_axioms,
            logical_axioms=sorted({a for values in assumptions.values() for a in values}),
            native_claim_axioms=[], proof_terms={n:digest(work/(n+'.vo')) for n in modules},
            source_connection_proved=False, original_array_future_import_proved=False)
        receipt['proof_artifacts'] = outputs.initial
        guard.check()
        print(json.dumps(dict(output=str(a.output), model_proofs_kernel_checked=True,
                              publication=False)))

if __name__ == '__main__':
    main()
