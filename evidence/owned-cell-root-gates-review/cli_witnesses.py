#!/usr/bin/env python3
"""Independent valid source coverage matrix; no synthesized VIR compilation."""
import json, sys
from pathlib import Path
sys.dont_write_bytecode=True
import recheck as r
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require,validate_run,require_failure_kind

def main():
    r.authenticate_snapshot_only()
    b=json.loads((r.OUT/'binding.json').read_text())
    out=r.OUT/'cli';out.mkdir(exist_ok=True)
    for d in ['logs','cases']: (out/d).mkdir(exist_ok=True)
    guard=ArtifactGuard([r.ROOT/n for n in b['source_sha256']]+[r.OUT/'snapshot'/n for n in [*b['authenticated_files'],'verus-root','receipt.json']])
    receipt=dict(kind='INDEPENDENT_CLI_SOURCE_WITNESSES_ON_AUTHENTICATED_PARENT_ARTIFACT', commands=[], cases=[])
    if (out/'receipt.json').exists():
        receipt=json.loads((out/'receipt.json').read_text())
        receipt.pop('failure',None)
    runner=LoggedCommands(out/'logs',r.ROOT,receipt['commands'],timeout=90,guard=guard)
    matrices=[]
    roots={'none':'','verify':'#![verifier::verify]','macro':'#![verus::internal(verus_macro)]','exec':'#![verifier::exec]'}
    contexts={'plain':('pub fn unit() {}',[]), 'item':('#[verifier::verify] pub fn unit() {}',[]),
              'module':('#[verifier::verify] mod checked {pub fn unit() {}}',[]),
              'default':('pub fn unit() {}',['--no-external-by-default'])}
    for root,attr in roots.items():
        for context,(body,extra) in contexts.items():
            matrices.append((f'coverage-{root}-{context}',attr+'\nuse vstd::prelude::*; '+body,extra,'unmarked' if context=='plain' else 'checked'))
    exclusions={
        'external':'verifier::external','external_body':'verifier::external_body',
        'external_fn_specification':'verifier::external_fn_specification',
        'external_type_specification':'verifier::external_type_specification',
        'external_trait_specification':'verifier::external_trait_specification','trusted':'verus::trusted',
        'const_body':'verus::internal(const_body)','reveal_fn':'verus::internal(reveal_fn)',
        'reveal_group':'verus::internal(reveal_group)','item_broadcast_use':'verus::internal(item_broadcast_use)',
        'size_of_global':'verus::internal(size_of)','structural_const_wrapper':'verus::internal(structural_const_wrapper)'}
    for name,attr in exclusions.items():
        kind=('frontend-skipped const_body/reveal_fn' if name in ['const_body','reveal_fn'] else
              'frontend directive/special-const' if name in ['reveal_group','item_broadcast_use','size_of_global','structural_const_wrapper'] else
              'external/abstract/skipped/trusted')
        matrices.append(('exclude-'+name,f'#![{attr}]\nuse vstd::prelude::*; #[verifier::verify] pub fn unit() {{}}',[],kind))
    for context,body,extra in [
        ('item','#[verifier::verify] pub fn lie()->u64 {verus_builtin::ensures(|r:u64| [verus_builtin::constrain_type(r,lie()),false]); 0}',[]),
        ('module','#[verifier::verify] mod checked {use vstd::prelude::*; pub fn lie()->u64 {verus_builtin::ensures(|r:u64| [verus_builtin::constrain_type(r,lie()),false]); 0}}',[]),
        ('default','pub fn lie()->u64 {verus_builtin::ensures(|r:u64| [verus_builtin::constrain_type(r,lie()),false]); 0}',['--no-external-by-default'])]:
        matrices.append(('false-contract-'+context,'#![verifier::verify]\nuse vstd::prelude::*; '+body,extra,'false-contract'))
    with replay_receipt(out,receipt):
        require(len(matrices)==31,'31 source fixtures')
        for name,source,extra,kind in matrices:
            path=out/'cases'/(name+'.rs');path.write_text(source)
            for infer in [False,True]:
                for feature in [False,True]:
                    label=f'{name}-infer{int(infer)}-feature{int(feature)}'
                    if any(row['label']==label for row in receipt['cases']):
                        row=next(row for row in receipt['cases'] if row['label']==label)
                        require(row['source_sha256']==digest(path),'resumed exact source')
                        continue
                    cached=next((c for c in receipt['commands'] if c['label']==label),None)
                    d=Path(cached['argv'][cached['argv'].index('-o')+1]).parent if cached else out/label
                    if d.exists() and not cached: d=out/(label+'-corrected')
                    d.mkdir(exist_ok=bool(cached));(d/'tmp').mkdir(exist_ok=bool(cached))
                    env=r.env();env.update(TMPDIR=str(d/'tmp'),VERUS_INFER_CACHE=str(d/'cache'))
                    argv=[str(r.OUT/'snapshot/verus'),str(path),'--crate-type=lib','--crate-name=witness','--edition=2024','--no-cheating','--compile','--output-json','-o',str(d/'witness.rlib'),*extra]
                    if infer: argv+=['--infer-goals']
                    if feature: argv+=['--experimental-owned-unsafe-cell']
                    if cached:
                        require(cached['argv']==argv and cached['exit_code'] is not None,'revalidate exact prior command')
                        code=cached['exit_code']
                        receipt.setdefault('revalidated_original_outputs',[]).append(label)
                    else:
                        code=runner(label,argv,env)
                    stdout=(out/'logs'/(label+'.stdout')).read_text();stderr=(out/'logs'/(label+'.stderr')).read_text()
                    data=json.loads(stdout);v=data['verification-results'];i=data['verus-infer'];scope=i['verification_scope']
                    require(not any(s in stderr.lower() for s in ['panicked at','resource limit','timed out']),'no infrastructure failure')
                    require(i['functions']==[] and i['requested_functions']==0 and i['generated_functions']==0 and not i['interface_written'],'zero helper/export credit')
                    expected=''
                    if feature:
                        expected=('native/unverified/skipped local body is forbidden' if kind=='unmarked' else
                                  ('inactive: F1 admission passed; functions=1, bodies=2, headers=1' if kind=='false-contract' else 'inactive: F1 admission passed; functions=1, bodies=1') if kind in ['checked','false-contract'] else kind)
                        require(code!=0 and expected in stderr and not v.get('success',False),'specific inactive refusal')
                        require(data['func-details']=={} and scope['checked_exec_bodies']==0 and scope['checked_proof_bodies']==0,'refusal carries zero proof credit')
                        if kind not in ['checked','false-contract']: require('inactive: F1 admission passed;' not in stderr,'no later admission')
                        require(not (d/'witness.rlib').exists(),'no compiled feature output')
                    else:
                        validate_run(dict(stdout=stdout,stderr=stderr,returncode=code),kind!='false-contract')
                        require(not v.get('encountered-vir-error') and (not v.get('encountered-error') or kind=='false-contract'),'valid ordinary frontend')
                        if kind=='false-contract':
                            require_failure_kind(stderr,'postcondition not satisfied');expected='postcondition not satisfied'
                            require(v['errors']==1 and any(k.startswith('witness::') and k.endswith('::lie') for k in data['func-details']),'actual failed ordinary contract')
                            require(len(scope['native_functions'])==1 and scope['native_functions'][0]['selected'] and scope['native_functions'][0]['ordinary_contract'],'actual selected authored contract')
                            require(scope['checked_exec_bodies']==0 and not scope['complete_selected_scope_verified'],'failed proof earns no success credit')
                        elif kind=='unmarked':
                            require(v['verified']==0 and scope['checked_exec_bodies']==0 and data['func-details']=={},'root-only is ordinary unverified')
                            require(len(scope['native_functions'])==1 and scope['native_functions'][0]['body_status']=='outside_selected_scope','explicit unselected inventory')
                        else:
                            require(v['verified']==1 and scope['checked_exec_bodies']==1 and len(data['func-details'])==1,'actual ordinary checking')
                    row=dict(label=label,source_sha256=digest(path),kind=kind,infer=infer,feature=feature,exit_code=code,
                             verified=v.get('verified'),errors=v.get('errors'),checked_exec_bodies=scope['checked_exec_bodies'],
                             checked_proof_bodies=scope['checked_proof_bodies'],expected_diagnostic=expected,
                             stdout_sha256=digest(out/'logs'/(label+'.stdout')),stderr_sha256=digest(out/'logs'/(label+'.stderr')))
                    receipt['cases'].append(row)
                    (out/'partial-results.json').write_text(json.dumps(receipt,indent=2)+'\n')
        require(len(receipt['cases'])==124,'exact 124 cases')
        guard.check();receipt.update(guards_passed=True, native_body_proofs=0, state_proofs=0)
    print(json.dumps(dict(success=receipt['success'],cases=len(receipt['cases']))))
if __name__=='__main__':main()
