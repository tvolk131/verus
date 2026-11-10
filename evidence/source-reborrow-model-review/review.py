#!/usr/bin/env python3
"""Independent bounded evidence audit. No implementation edits or native acceptance."""
import argparse
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
LANE = ROOT/'source/tools/research/native_source_reborrow'
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require
MODULES = ['Model', 'Laws', 'Span', 'Controls']
GOAL_PIN = '11014c3ac73ddf13c464769f5d0d1cbd24598ab52777718d31872c70b3b8f7b0'

def independent_inventory(text):
    """Parse compiler module blocks, recording constructor-to-inductive coverage."""
    blocks = re.split(r'(?=^     (?:Definition|Parameter|Record|Inductive) )', text, flags=re.M)[1:]
    result = {}
    for block in blocks:
        m = re.match(r'     (Definition|Parameter|Record|Inductive) (\w+)', block)
        require(m is not None, 'unrecognized module block')
        kind, name = m.groups()
        require(name not in result, 'duplicate compiler declaration')
        result[name] = {'kind':kind, 'kernel_name':name}
        if kind == 'Inductive':
            constructors = re.findall(r'^\s*(?:\|\s*)?(\w+)\s*:', block.split(':=',1)[1], re.M)
        elif kind == 'Record':
            constructors = [re.search(r':=\s*(\w+)',block)[1]]
        else:
            constructors = []
        for constructor in constructors:
            require(constructor not in result, 'duplicate constructor')
            result[constructor] = {'kind':'constructor', 'kernel_name':name}
    return result

def goal_pin(snapshot):
    start=b'## Required property and domain\n'; end=b'## Coordination\n'
    require(snapshot.count(start)==snapshot.count(end)==1, 'goal boundaries')
    i,j=snapshot.index(start),snapshot.index(end)
    require(i<j and hashlib.sha256(snapshot[i:j]).hexdigest()==GOAL_PIN,'goal requirements')

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--output', type=Path, required=True)
    a=ap.parse_args(); out=a.output.resolve(); out.mkdir(parents=True,exist_ok=False)
    receipt={'schema':1,'commands':[],'model_review_checked':False,
             'native_correspondences':0,'publication':False,'goal_complete':False}
    with replay_receipt(out,receipt):
        loaded={}
        def load(path):
            path=Path(path); b=path.read_bytes(); loaded[path]=b; return json.loads(b)
        pre=load(HERE/'pretrial.json'); trial=load(HERE/'controller-freeze.json')
        require(pre['before_first_trial'] is True and trial['before_first_trial'] is True,'pretrial freeze missing')
        old={Path(p):load(p) for p in pre['completed_receipts']}
        parent_path=next(p for p in old if 'parent-integration001' in str(p))
        parent=old[parent_path]
        paths=[HERE/'pretrial.json',HERE/'controller-freeze.json',
               *[ROOT/p for p in pre['files']],*[ROOT/p for p in trial['files']],*old]
        # Never follow the historical checkout labels in a receipt's /home paths.
        external={p:x for p,x in parent['inputs'].items() if p.startswith('/tmp/')}
        paths += [Path(p) for p in external]
        paths += [Path(p) for r in old.values() for p in r['proof_artifacts']]
        guard=ArtifactGuard(paths)
        def retained(p):return guard.initial[str(Path(p).resolve())]['sha256']
        for p,b in loaded.items():require(retained(p)==hashlib.sha256(b).hexdigest(),'loaded bytes mismatch '+str(p))
        for f in [pre,trial]:
            for p,h in f['files'].items():require(retained(ROOT/p)==h,'freeze mismatch '+p)
        for p,x in external.items():require(retained(p)==x['sha256'],'external identity '+p)
        for p,x in pre['completed_receipts'].items():
            require(retained(p)==x['sha256'],'receipt identity '+p)
            for path,h in x['proof_artifacts'].items():require(retained(path)==h,'completed product '+path)
        target=ROOT/'source/tools/research/mutable_buffer_goal/TARGET.md'
        snapshot=target.read_bytes()
        require(hashlib.sha256(snapshot).hexdigest()==retained(target),'loaded TARGET mismatch')
        goal_pin(snapshot)
        integration=load(LANE/'integration-results.json')
        for field in ['worker_receipt','integration_receipt']:
            x=integration[field]
            require(pre['completed_receipts'][x['path']]['sha256']==x['sha256'],'integration identity')
        freeze=load(LANE/'integration001-freeze.json')
        names={m+'.'+n for m in MODULES for n in freeze['constants'][m]}
        require(len(names)==134,'expected 134 declarations')
        for r in old.values():
            require(r['success'] is True and r['model_proofs_kernel_checked'] is True,'completed check unsuccessful')
            require(set(r['assumptions'])==names and all(v==[] for v in r['assumptions'].values()),'completed assumption coverage')
            require(r['kernel_axioms']==[],'completed axioms')
        receipt['inputs']=guard.initial
        env=dict(os.environ)
        run=LoggedCommands(out,ROOT,receipt['commands'],timeout=1800,guard=guard)
        for opt,label in [([], 'normal-controls'),(['-O'],'optimized-controls')]:
            require(run(label,[sys.executable,'-B',*opt,str(LANE/'test_check.py')],env)==0,'pure control failure')
        replay=out/'replay'
        require(run('fresh-replay',[sys.executable,'-B',str(LANE/'check.py'),'--freeze',
                    str(LANE/'integration001-freeze.json'),'--output',str(replay)],env)==0,'replay setup/proof failure, not semantic evidence')
        fresh=load(replay/'receipt.json')
        require(fresh['success'] is True and fresh['model_proofs_kernel_checked'] is True,'fresh replay failed')
        require(set(fresh['assumptions'])==names and all(v==[] for v in fresh['assumptions'].values()),'fresh assumption coverage')
        require(fresh['kernel_axioms']==[] and fresh['constant_count']==134,'fresh kernel result')
        for flag in ['publication','goal_complete','source_certificate_proved','actual_source_instance',
                     'all_lawful_rust_inputs_covered','source_validity_interpretation_proved','source_cf_bindings_proved']:
            require(fresh[flag] is False,'native claim '+flag)
        product_guard=ArtifactGuard([replay/'receipt.json',*[Path(p) for p in fresh['inputs']],
                                    *[Path(p) for p in fresh['proof_artifacts']]])
        for p,b in loaded.items():
            known=guard.initial.get(str(p.resolve()),product_guard.initial.get(str(p.resolve())))
            require(known is not None and known['sha256']==hashlib.sha256(b).hexdigest(),'loaded final mismatch')
        def objects(r,root):
            return {str(Path(p).relative_to(root)):x['sha256'] for p,x in r['proof_artifacts'].items()
                    if Path(p).suffix in ['.v','.vo']}
        expected=objects(parent,parent_path.parent)
        for p,r in old.items():require(objects(r,p.parent)==expected,'worker/parent V/VO mismatch')
        require(objects(fresh,replay)==expected,'fresh V/VO mismatch')
        receipt['exact_v_vo']=expected
        inventory={}
        for m in MODULES:
            text=(replay/'proofs/inventory'/(m+'.module.out')).read_text()
            discovered=independent_inventory(text)
            require(set(discovered)==set(freeze['constants'][m]),'independent compiler inventory '+m)
            inventory[m]=discovered
            for n in discovered:
                require((replay/'proofs/inventory'/(m+'.'+n+'.assumptions.out')).read_text().strip()==
                        'Closed under the global context','individual assumptions '+m+'.'+n)
        inspect=(replay/'proofs/Assumptions.v').read_text()
        allowed=[r'From NativeSourceReborrow Require Import Model Laws Span Controls\.',
                 r'Redirect "inventory/[\w.]+\.assumptions" Print Assumptions NativeSourceReborrow\.[\w.]+\.',
                 r'Redirect "inventory/[\w.]+\.statement" Check NativeSourceReborrow\.[\w.]+\.',
                 r'Redirect "inventory/\w+\.module" Print Module NativeSourceReborrow\.\w+\.']
        require(all(any(re.fullmatch(p,line) for p in allowed) for line in inspect.splitlines()),'hidden inspection command')
        require(len(inspect.splitlines())==1+134*2+4,'inspection command count')
        # Verbose kernel run enables an independent declaration coverage audit.
        argv=next(c['argv'] for c in fresh['commands'] if c['label']=='kernel')
        require('-silent' in argv and not any(x in argv for x in ['-admit','-norec']), 'kernel invocation')
        argv=[x for x in argv if x!='-silent']
        kernel_run=LoggedCommands(out,replay/'proofs',receipt['commands'],timeout=1800,guard=product_guard)
        require(kernel_run('verbose-kernel',argv,env)==0,'verbose recursive kernel failure')
        log=(out/'verbose-kernel.stderr').read_text()+(out/'verbose-kernel.stdout').read_text()
        checked=set(re.findall(r'checking (?:cst|ind):([^\s]+)',log))
        for m,ds in inventory.items():
            for n,x in ds.items():
                key='NativeSourceReborrow.'+m+'.'+x['kernel_name']
                require(key in checked,'kernel declaration absent '+key)
                x['kernel_covered_by']=key
        summary=log.split('CONTEXT SUMMARY')
        require(len(summary)==2,'kernel summary count')
        require({s.strip() for s in summary[1].splitlines() if s.strip()}=={
            '===============','* Theory: Set is predicative','* Theory: Rewrite rules are not allowed',
            '* Axioms: <none>','* Constants/Inductives relying on type-in-type: <none>',
            '* Constants/Inductives relying on unsafe (co)fixpoints: <none>',
            '* Inductives whose positivity is assumed: <none>'},'recursive trust summary')
        (out/'inventory.json').write_text(json.dumps(inventory,indent=2)+'\n')
        product_guard.check();guard.check()
        for p,b in loaded.items():require(p.read_bytes()==b,'loaded input changed at success '+str(p))
        require(target.read_bytes()==snapshot,'TARGET changed');goal_pin(snapshot)
        receipt.update(model_review_checked=True,compiler_inventory_count=sum(map(len,inventory.values())),
                       constructor_coverage='constructors checked within their inductive/record blocks',
                       recursive_kernel_axioms=[],individual_assumptions_empty=True,
                       all_final_guards_before_success=True,goal_requirements_sha256=GOAL_PIN,
                       fresh_receipt={'path':str(replay/'receipt.json'),'sha256':digest(replay/'receipt.json')},
                       inventory_sha256=digest(out/'inventory.json'))
    print(json.dumps({'output':str(out),'success':receipt['success']}))

if __name__=='__main__':main()
