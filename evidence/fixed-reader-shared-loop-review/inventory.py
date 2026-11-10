#!/usr/bin/env python3
"""Query all already-checked constants, including generated inventory helpers.

No declarations are introduced: coqtop runs only Require, Check and Print
Assumptions. The queried objects have already undergone recursive coqchk.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
sys.dont_write_bytecode = True
from review import HERE, ROOT, CASES, READY, authenticate, load, require, map_hash
from infer_eval_support import ArtifactGuard, digest

def main():
    p = argparse.ArgumentParser()
    p.add_argument('stage',choices=['freeze','run'])
    p.add_argument('--output',type=Path,required=True)
    a = p.parse_args()
    guard,_ = authenticate()
    review_result = load(HERE/'results.json')
    ready = load(READY)
    coqtop = Path(ready['switch'])/'_opam/bin/coqtop'
    paths = [*guard.paths, Path(__file__), HERE/'results.json', coqtop]
    # Hash-only preservation checks, not canonical/HMAC proof replays.
    research = ROOT/'source/tools/research'
    source_freeze = research/'mutable_buffer_goal/freeze.json'
    transfer = research/'mutable_buffer_goal/transfer-freeze.json'
    paths.extend([source_freeze,transfer])
    preserved = {}
    for label,sha in load(source_freeze)['files'].items():
        x = ROOT/label
        require(digest(x) == sha, 'canonical frozen source changed')
        paths.append(x)
        preserved[label] = sha
    tf = load(transfer)
    manifest = ROOT/tf['pretrial_manifest']
    require(digest(manifest) == tf['pretrial_manifest_sha256'], 'transfer manifest changed')
    paths.append(manifest)
    for label,sha in load(manifest)['files'].items():
        x = manifest.parent/label
        require(digest(x) == sha, 'transfer frozen source changed')
        paths.append(x)
        preserved[str(x.relative_to(ROOT))] = sha
    jobs = {}
    for name,(_,_,_,_,_,_,ns) in CASES.items():
        d = Path(review_result[name]['fresh_receipt']).parent
        r = load(d/'receipt.json')
        require(digest(d/'receipt.json') == review_result[name]['fresh_receipt_sha256'], 'fresh receipt changed')
        paths.append(d/'receipt.json')
        for label,info in r['proof_artifacts'].items():
            x = d/'proofs'/Path(label).name
            require(digest(x) == info['sha256'], 'fresh product changed')
            paths.extend([x, x.with_suffix('.glob')])
        expected = r.get('declarations',r.get('declaration_inventory'))
        main_names = [m+'.'+n for m,values in expected.items() for n in values]
        if name == 'pair':
            helpers = ['Assumptions.Unnamed_thm']+['Assumptions.Unnamed_thm'+str(i) for i in range(31)]
        else:
            helpers = [n for names in r['inventory_helper_theorems'].values() for n in names]
        glob_names = []
        for x in sorted((d/'proofs').glob('*.glob')):
            glob_names += [x.stem+'.'+n for n in re.findall(
                r'^(?:def|prf|ind|constr|rec) \d+:\d+ <> (\S+)', x.read_text(),re.M)]
        require(len(glob_names) == len(main_names)+len(helpers) and
                set(glob_names) == set(main_names+helpers), 'compiled declaration inventory differs')
        imports = list(expected)+(['Assumptions'] if name == 'pair' else ['Assumptions','ProofTerms'])
        prefix = 'From '+ns+' Require Import '+' '.join(imports)+'.\n'
        queries = {
            'helpers': prefix+''.join('Print Assumptions '+ns+'.'+n+'.\n' for n in helpers),
            'types': prefix+'Set Printing All.\n'+''.join('Check '+ns+'.'+n+'.\n' for n in main_names),
        }
        for kind,text in queries.items():
            q = HERE/(name+'-'+kind+'.v')
            if a.stage == 'freeze':
                require(not q.exists(), 'query already exists')
                q.write_text(text)
            require(q.read_text() == text, 'query differs')
            paths.append(q)
        compile_command = r['commands'][0]['argv']
        require(compile_command[2] == '-q', 'unexpected compiler command')
        jobs[name] = dict(args=compile_command[3:-1], helpers=helpers, main=main_names,
                          glob=glob_names, namespace=ns)
    frozen = ArtifactGuard(paths)
    freeze = HERE/'inventory-freeze.json'
    data = dict(before_queries=True, inputs_sha256=map_hash(frozen.initial),inputs_count=len(frozen.initial),
                coqtop_sha256=digest(coqtop),output=str(a.output),jobs=jobs)
    if a.stage == 'freeze':
        require(not freeze.exists(), 'inventory freeze already exists')
        freeze.write_text(json.dumps(data,indent=2)+'\n')
        return
    require(load(freeze) == data, 'query freeze changed')
    freeze_guard = ArtifactGuard([freeze])
    a.output.mkdir(exist_ok=False)
    results = dict(new_declarations=[],new_instances=[],constructors=[],obligations=[],cases={},
                   preserved_frozen_source_hashes=preserved, canonical_and_hmac_rerun=False)
    env = dict(os.environ)
    for key in ['COQPATH','COQLIB','COQCORELIB','OCAMLPATH','CAML_LD_LIBRARY_PATH','LD_PRELOAD','LD_LIBRARY_PATH']:
        env.pop(key,None)
    for name,job in jobs.items():
        results['cases'][name] = dict(all_compiled_declarations=job['glob'],
            source_declarations=job['main'],helper_assumptions={n:[] for n in job['helpers']},queries={})
        for kind in ['helpers','types']:
            q = HERE/(name+'-'+kind+'.v')
            cmd = [ready['runner'],str(coqtop),'-quiet','-batch',*job['args'],'-l',str(q)]
            frozen.check()
            run = subprocess.run(cmd,env=env,cwd=a.output,capture_output=True)
            for stream in ['stdout','stderr']:
                (a.output/(name+'-'+kind+'.'+stream)).write_bytes(getattr(run,stream))
            require(run.returncode == 0, 'query failed; setup/development failure')
            if kind == 'helpers':
                require(run.stdout.decode().strip().splitlines() ==
                        ['Closed under the global context']*len(job['helpers']), 'helper assumptions differ')
            results['cases'][name]['queries'][kind] = dict(argv=cmd,exit_code=run.returncode,
                stdout_sha256=digest(a.output/(name+'-'+kind+'.stdout')),
                stderr_sha256=digest(a.output/(name+'-'+kind+'.stderr')))
            frozen.check()
    guard.check()
    freeze_guard.check()
    (HERE/'inventory-results.json').write_text(json.dumps(results,indent=2)+'\n')
    print('All 108 compiled declarations inventoried: 28 source constants, 80 closed marker helpers.')

if __name__ == '__main__':
    main()
