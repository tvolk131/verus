#!/usr/bin/env python3
"""Normal Rocq extra-assumption and inventory refusals, never semantic negatives."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--ready', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    r = dict(schema=1, commands=[], cases=[], publication=False,
             native_bridge_proved=False, semantic_negative_count=0)
    with replay_receipt(out, r):
        freeze = json.loads(a.freeze.read_text())
        for name, value in freeze['files'].items():
            require(digest(ROOT/name) == value, 'changed frozen input '+name)
        guard = ArtifactGuard([a.freeze, a.ready, Path(__file__),
                               *[ROOT/n for n in freeze['files']], *support_artifacts()])
        run = LoggedCommands(out, ROOT, r['commands'], guard=guard, timeout=1800)

        def trial(label, f, marker, stage):
            path = out/(label+'-freeze.json')
            path.write_text(json.dumps(f, indent=2)+'\n')
            result_dir = out/(label+'-trial')
            code = run(label, [sys.executable, '-B', '-O', str(HERE/'check.py'),
                              '--ready', str(a.ready), '--freeze', str(path),
                              '--output', str(result_dir)], os.environ)
            result = json.loads((result_dir/'receipt.json').read_text())
            require(code != 0 and result['success'] is False
                    and marker in result['failure']['message'], 'wrong guard refusal '+label)
            require(result['model_proofs_kernel_checked'] is False
                    and result['publication'] is False and result['native_bridge_proved'] is False,
                    'guard failure acquired a proof status')
            labels = [c['label'] for c in result['commands']]
            require((not labels if stage is None else labels[-1] == stage),
                    'unexpected refusal stage '+label)
            require(all(c.get('exit_code', c.get('returncode')) == 0 for c in result['commands']),
                    'a failed compiler command is not an assumption control')
            r['cases'].append(dict(label=label, classification='proof_acceptance_gate_rejection',
                                   receipt=str(result_dir/'receipt.json'),
                                   receipt_sha256=digest(result_dir/'receipt.json'),
                                   marker=marker, last_stage=stage))

        reduced = dict(freeze, modules=['borrow_handoff'])
        trial('reduced-inventory', reduced, 'exact four-module inventory required', None)
        source = ROOT/freeze['candidate_directory']
        modules = freeze['modules']
        for label, hidden in [('single-line-classic', False), ('kernel-only-classic', True)]:
            candidate = Path(tempfile.mkdtemp(prefix='guard-', dir=HERE/'candidates'))
            try:
                for name in modules:
                    shutil.copyfile(source/(name+'.v'), candidate/(name+'.v'))
                target = candidate/'borrow_handoff.v'
                text = target.read_text()
                text = 'From Coq Require Import Classical_Prop.\n'+text
                if hidden:
                    text += '\nDefinition guard_hidden_classical := classic.\n'
                else:
                    text = text.replace('Proof.\n', 'Proof.\n  destruct (classic True) as [GuardYes | GuardNo].\n  2: { exfalso; exact (GuardNo I). }\n', 1)
                target.write_text(text)
                saved = out/(label+'-source')
                shutil.copytree(candidate, saved)
                modified = dict(freeze, candidate_directory=str(candidate.relative_to(ROOT)),
                                files=dict(freeze['files']),
                                purpose='Acceptance-gate control; no native claim or semantic negative.')
                for name in modules:
                    path = candidate/(name+'.v')
                    modified['files'][str(path.relative_to(ROOT))] = digest(path)
                trial(label, modified,
                      'unexpected kernel axioms' if hidden else 'unexpected logical axiom',
                      'kernel-check' if hidden else 'assumption-inventory')
            finally:
                require({p.name for p in candidate.iterdir()} == {n+'.v' for n in modules},
                        'unexpected files in private control source directory')
                for name in modules:
                    (candidate/(name+'.v')).unlink()
                candidate.rmdir()
        require(len(r['cases']) == 3, 'acceptance control inventory')
        guard.check()
        r['artifacts'] = guard.initial


if __name__ == '__main__':
    main()
