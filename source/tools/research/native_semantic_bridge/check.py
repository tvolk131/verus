#!/usr/bin/env python3
"""Full normal Rocq compilation + transitive kernel check. Never publishes facts."""
import argparse
import json
import os
import re
import shlex
import shutil
import sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, require, replay_receipt
PIN = '115a618dd35ff8337112cf2081a7f5391d3973d5'
ALLOWED = {'functional_extensionality_dep', 'Eqdep.Eq_rect_eq.eq_rect_eq'}

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
        ready = json.loads(a.ready.read_text())
        require(ready['source_commit'] == PIN and ready['baseline_kernel_checked'] is True,
                'unready/wrong foundation')
        baseline_receipt = Path(ready['receipt'])
        require(json.loads(baseline_receipt.read_text())['success'] is True,
                'baseline final receipt did not succeed')
        baseline = Path(ready['build']).resolve()
        switchbin = Path(ready['switch'])/'_opam/bin'
        coqc, coqchk = switchbin/'coqc', switchbin/'coqchk'
        freeze = json.loads(a.freeze.read_text())
        require(freeze['before_first_trial'] is True, 'missing pretrial freeze')
        for name, value in freeze['files'].items():
            require(digest(ROOT/name) == value, 'freeze mismatch '+name)
        paths = [ROOT/n for n in freeze['files']]
        # Freeze the full compiled foundation and sources for this trial; no
        # installation or baseline writes. Loadpaths point read-only at baseline.
        paths += [p for p in (baseline/'src').rglob('*') if p.suffix in ('.v', '.vo')]
        paths += [coqc, coqchk, baseline/'_CoqProject', a.ready, baseline_receipt,
                  a.freeze, *support_artifacts()]
        guard = ArtifactGuard(paths)
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
        require(run('version', [str(coqc), '--version'], env) == 0, 'prover version setup')
        require('version 8.20.0' in (a.output/'version.stdout').read_text(), 'wrong prover')
        candidate = ROOT/freeze['candidate_directory']
        modules = freeze['modules']
        require(modules and len(modules) == len(set(modules)), 'module inventory')
        source_text = '\n'.join((candidate/(name+'.v')).read_text() for name in modules)
        require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|native_compute|vm_cast_no_check)\b', source_text),
                'unchecked proof construct in selected candidate')
        for name in modules:
            target = work/(name+'.v')
            shutil.copyfile(candidate/(name+'.v'), target)
            require(run('compile-'+name, [str(coqc), '-q', *args, str(target)], env) == 0,
                    'Rocq script/setup failure (not semantic evidence): '+name)
            require(target.with_suffix('.vo').is_file(), 'missing compiled module '+name)
        # Recheck proof terms recursively, with normal kernel checking enabled.
        require(run('kernel-check', [str(coqchk), '-silent', '-o', *args,
                *['NativeSemanticBridge.'+n for n in modules]], env) == 0,
                'transitive kernel check failed')
        assumptions = set()
        for name in modules:
            output = (a.output/('compile-'+name+'.stdout')).read_text()
            assumptions.update(re.findall(r'^(\S+)\s*:\s*$', output, re.M))
        assumptions.discard('Axioms')
        require(assumptions <= ALLOWED, 'unexpected axioms: '+str(assumptions-ALLOWED))
        receipt.update(model_proofs_kernel_checked=True, modules=modules,
            logical_axioms=sorted(assumptions),
            native_claim_axioms=[], proof_terms={n:digest(work/(n+'.vo')) for n in modules},
            source_connection_proved=False, original_array_future_import_proved=False)
        guard.check()
        print(json.dumps(dict(output=str(a.output), model_proofs_kernel_checked=True,
                              publication=False)))

if __name__ == '__main__':
    main()
