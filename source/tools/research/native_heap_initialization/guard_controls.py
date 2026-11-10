#!/usr/bin/env python3
"""Provenance refusals only; no failed script is counted as semantic evidence."""
import argparse, hashlib, json, os, shutil, sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, require, replay_receipt

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True,exist_ok=False)
    receipt = dict(commands=[], semantic_negatives=0, classification='artifact_provenance_controls')
    with replay_receipt(a.output,receipt):
        data = a.freeze.read_bytes()
        freeze = json.loads(data)
        guard = ArtifactGuard([a.freeze,*[ROOT/n for n in freeze['files']]])
        require(guard.initial[str(a.freeze.resolve())]['sha256']==hashlib.sha256(data).hexdigest(),'control freeze changed')
        for n, expected in freeze['files'].items():
            require(guard.initial[str((ROOT/n).resolve())]['sha256']==expected,'control input changed '+n)
        base_freeze = HERE/'final-freeze.json'
        baseline = json.loads(base_freeze.read_bytes())
        require(set(baseline['files']) | {str(base_freeze.relative_to(ROOT)),str(Path(__file__).resolve().relative_to(ROOT))}
                == set(freeze['files']), 'control freeze incomplete')
        receipt['inputs'] = guard.initial
        run = LoggedCommands(a.output,ROOT,receipt['commands'],timeout=120,guard=guard)
        results = []
        for mode in ['normal','optimized']:
            for case in ['wrong-source','wrong-controller','omitted-frozen-input']:
                label = mode+'-'+case
                stage = a.output/label
                stage.mkdir()
                for n in baseline['files']:
                    target = stage/n
                    target.parent.mkdir(parents=True,exist_ok=True)
                    shutil.copyfile(ROOT/n,target)
                    require(digest(target)==baseline['files'][n], 'copy identity mismatch')
                trial_freeze = stage/'trial.json'
                trial_freeze.write_bytes(base_freeze.read_bytes())
                lane = stage/HERE.relative_to(ROOT)
                if case == 'omitted-frozen-input':
                    changed = json.loads(trial_freeze.read_bytes())
                    changed['files'].pop(str((HERE/'candidates/initialization.v').relative_to(ROOT)))
                    trial_freeze.write_text(json.dumps(changed))
                    expected = 'freeze inputs differ'
                else:
                    target = lane/('candidates/initialization.v' if case=='wrong-source' else 'check.py')
                    with target.open('a') as f:
                        f.write('\n(* identity mutation *)\n' if case=='wrong-source' else '\n# identity mutation\n')
                    expected = 'frozen snapshot mismatch'
                argv = [sys.executable,'-B']+(['-O'] if mode=='optimized' else [])
                argv += [str(lane/'check.py'),'--freeze',str(trial_freeze),'--output',str(stage/'rejected')]
                require(run(label,argv,dict(os.environ))!=0, 'mutated input unexpectedly accepted')
                rejection = json.loads((stage/'rejected/receipt.json').read_bytes())
                require(rejection['success'] is False and rejection['commands']==[] and
                        expected in rejection['failure']['message'], 'not the intended pre-prover refusal')
                results.append(dict(case=label,classification='provenance_refusal',
                                    failure=rejection['failure'],receipt_sha256=digest(stage/'rejected/receipt.json')))
        receipt['results'] = results
        guard.check()
        print(json.dumps(dict(provenance_refusals=len(results),semantic_negatives=0)))
if __name__=='__main__': main()
