#!/usr/bin/env python3
"""Read-only authentication and replay of two exact, completed MODEL libraries."""
import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path[:0] = [str(ROOT/'source/tools'), str(ROOT/'source/tools/research/native_semantic_bridge')]
from infer_eval_support import ArtifactGuard, digest
from proof_checks import parse_axioms

CASES = {
    'pair': ('30c9c17c', 'model_pair_reader', '/tmp/model-pair-reader-trial-012',
             '534374d1dc414dc7bc98368b4f1acb892796238e6ae03e10286a0421d1e80bde',
             'results.json', 'final-freeze.json', 'ModelPairReader'),
    'loop': ('88a56443', 'model_shared_xor_loop', '/tmp/model-shared-xor-loop-trial011',
             '41b447e1daee6caa6a24e80589f353ca71d07f5293893ff452b40f5ebaf2d881',
             'EVIDENCE.json', 'trial011-freeze.json', 'ModelSharedXorLoop'),
}
READY = Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json')

def require(ok, message):
    if not ok:
        raise ValueError(message)

def load(p):
    return json.loads(Path(p).read_bytes())

def map_hash(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()

def git_bytes(rev, path):
    return subprocess.check_output(['git', 'show', f'{rev}:{path}'], cwd=ROOT)

def local_input(label):
    # Historical worktree paths in receipts are labels ONLY, never opened.
    if '/source/tools/' in label and not label.startswith('/tmp/'):
        return ROOT/('source/tools/'+label.split('/source/tools/', 1)[1])
    p = Path(label)
    require(label.startswith('/tmp/'), 'unexpected external label '+label)
    require('parent-pair-reader-30c9c17c' not in label and
            'parent-shared-xor-loop-88a56443' not in label, 'live replay forbidden')
    return p

def authenticate():
    paths = [Path(__file__), READY]
    summary = {}
    for name,(rev,lib,old,sha,resultfile,freeze,namespace) in CASES.items():
        old = Path(old)
        source = ROOT/'source/tools/research'/lib
        require(digest(old/'receipt.json') == sha, 'worker receipt hash '+name)
        receipt = load(old/'receipt.json')
        require(receipt['success'] is True and receipt['model_proofs_kernel_checked'] is True,
                'worker not successfully checked '+name)
        exact = subprocess.check_output(['git','ls-tree','-r','--name-only',rev,
                                        str(source.relative_to(ROOT))],cwd=ROOT,text=True).splitlines()
        for f in exact:
            require((ROOT/f).read_bytes() == git_bytes(rev,f), 'revision mismatch '+f)
            paths.append(ROOT/f)
        result = load(source/resultfile)
        require(result['receipt']['sha256'] == sha, 'committed receipt pin')
        require(Path(result['freeze']['path']).name == freeze, 'wrong freeze filename')
        require(digest(source/freeze) == result['freeze']['sha256'] == digest(old/'freeze.json'),
                'recorded freeze identity')
        for label,facts in receipt['inputs'].items():
            p = local_input(label)
            require(digest(p) == facts['sha256'], 'worker input identity '+str(p))
            paths.append(p)
        artifacts = {}
        for label,facts in receipt['proof_artifacts'].items():
            p = old/'proofs'/Path(label).name
            require(digest(p) == facts['sha256'], 'worker product identity '+str(p))
            paths.append(p)
            artifacts[p.name] = facts['sha256']
        require({p.name for p in (old/'proofs').iterdir() if p.suffix in ['.v','.vo']} ==
                set(artifacts), 'worker product set')
        for p in old.iterdir():
            if p.is_file():
                paths.append(p)
        summary[name] = dict(revision=subprocess.check_output(['git','rev-parse',rev],cwd=ROOT,text=True).strip(),
            library=lib, namespace=namespace, worker_receipt_sha256=sha, freeze=freeze,
            freeze_sha256=digest(source/freeze), authenticated_inputs=len(receipt['inputs']),
            worker_artifacts=artifacts, assumptions=receipt['assumptions'])
    ready = load(READY)
    baseline = load(ready['receipt'])
    require(ready['source_commit'] == '115a618dd35ff8337112cf2081a7f5391d3973d5' and
            ready['baseline_kernel_checked'] is True and baseline['success'] is True, 'baseline readiness')
    require(len(baseline['copied_sources']) == 523, 'baseline source count')
    build = Path(ready['build'])
    require(len(list((build/'src').rglob('*.vo'))) == 284, 'baseline object count')
    require(digest(READY.parent/'gmp-private/manifest.json') ==
            '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37', 'GMP identity')
    require({'prover-version','build-baseline','kernel-check-baseline'} <=
            {c['label'] for c in baseline['commands'] if c['returncode'] == 0}, 'baseline stages')
    return ArtifactGuard(paths), summary

def replay(name, output):
    _,lib,_,_,_,freeze,_ = CASES[name]
    source = ROOT/'source/tools/research'/lib
    cmd = ['python3','-B',str(source/'check.py'),'--freeze',str(source/freeze),'--output',str(output/name)]
    with (output/(name+'-controller.stdout')).open('w') as out, (output/(name+'-controller.stderr')).open('w') as err:
        r = subprocess.run(cmd,cwd=ROOT,stdout=out,stderr=err)
    require(r.returncode == 0, 'replay failed; retained as setup/development failure: '+name)
    return name

def finish(output, guard, summary):
    for name,(_,lib,old,_,_,_,_) in CASES.items():
        oldr = load(Path(old)/'receipt.json')
        newr = load(output/name/'receipt.json')
        require(newr['success'] is True and newr['model_proofs_kernel_checked'] is True, 'fresh receipt')
        require(newr['assumptions'] == oldr['assumptions'], 'individual assumptions differ')
        require(newr['kernel_axioms'] == oldr['kernel_axioms'], 'recursive assumptions differ')
        oldproducts = {Path(p).name:f['sha256'] for p,f in oldr['proof_artifacts'].items()}
        newproducts = {Path(p).name:f['sha256'] for p,f in newr['proof_artifacts'].items()}
        require(oldproducts == newproducts, 'source/object identities differ '+name)
        for c in newr['commands']:
            require(c['exit_code'] == 0, 'failed fresh command')
            require(not any(a in ['-admit','-norec','-exclude','-vos','-vok','-quick','-vio'] for a in c['argv']),
                    'unsafe/incomplete checking flag')
        # Include generated Goal/idtac marker theorems, omitted from pair's source count.
        inventory = {}
        for glob in sorted((output/name/'proofs').glob('*.glob')):
            names = re.findall(r'^(?:def|prf|ind|constr|rec) \d+:\d+ <> (\S+)',glob.read_text(),re.M)
            inventory[glob.stem] = names
        summary[name].update(fresh_receipt=str(output/name/'receipt.json'),
            fresh_receipt_sha256=digest(output/name/'receipt.json'),
            exact_worker_source_and_object_match=True, individual_assumptions_match=True,
            kernel_axioms=newr['kernel_axioms'], compiled_glob_inventory=inventory,
            commands=[dict(label=c['label'],exit_code=c['exit_code'],seconds=c['seconds']) for c in newr['commands']],
            raw_output_hashes={p.name:digest(p) for p in sorted((output/name).iterdir()) if p.is_file()})
    guard.check()
    summary['scope'] = dict(model_only=True, concrete_combined_writer=False, native=0,
        minimal_open=44, phases_open=3, writer_effects_open=101, publication=False, goal_complete=False,
        canonical='inherited 6/0; not rerun', hmac='inherited 1/1; not rerun')
    (HERE/'results.json').write_text(json.dumps(summary,indent=2)+'\n')

def main():
    p = argparse.ArgumentParser()
    p.add_argument('stage',choices=['freeze','run','finish'])
    p.add_argument('--output',type=Path,required=True)
    a = p.parse_args()
    guard,summary = authenticate()
    freeze = HERE/'review-freeze.json'
    if a.stage == 'freeze':
        require(not freeze.exists(), 'review freeze already exists')
        freeze.write_text(json.dumps(dict(before_trials=True,inputs_sha256=map_hash(guard.initial),
            inputs_count=len(guard.initial),cases=summary,
            output=str(a.output)),indent=2)+'\n')
        print('Authenticated both exact revisions and completed worker artifacts; frozen before trials.')
        return
    f = load(freeze)
    require(f['inputs_sha256'] == map_hash(guard.initial) and f['output'] == str(a.output), 'review freeze changed')
    freeze_guard = ArtifactGuard([freeze])
    if a.stage == 'run':
        a.output.mkdir(exist_ok=False)
        try:
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                list(pool.map(lambda name: replay(name,a.output),CASES))
            finish(a.output,guard,summary)
        except BaseException as e:
            (a.output/'review-failure.json').write_text(json.dumps(dict(type=type(e).__name__,message=str(e),
                classification='setup/development failure; not semantic evidence'),indent=2)+'\n')
            raise
    else:
        finish(a.output,guard,summary)
    freeze_guard.check()
    print('Both fresh normal compilations and unrestricted recursive kernel checks authenticated.')

if __name__ == '__main__':
    main()
