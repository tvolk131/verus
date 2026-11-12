#!/usr/bin/env python3
"""Small F3 controller using the existing experiment/evaluation utilities."""
import argparse, json, os, re, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
TOOLS=HERE.parents[2]
sys.path.insert(0,str(TOOLS))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_publication
RUST=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
def main():
    p=argparse.ArgumentParser(); p.add_argument('--output',type=Path,required=True)
    p.add_argument('--provider',type=Path)
    p.add_argument('--rejected',type=Path)
    p.add_argument('--private',type=Path,default=Path('/tmp/fedimint-f3-3ad8038e-auth'))
    p.add_argument('--scenario',choices=['eligibility','count','baseline','corrected-baseline','aggregation','manual','semantic','safety','duplicate','client','client-negative','failed-import','omit-array','omit-set'],default='eligibility')
    a=p.parse_args(); out=a.output; out.mkdir(parents=True,exist_ok=False)
    receipt=dict(scenario=a.scenario,commands=[],outcomes=[])
    with replay_receipt(out,receipt):
        inputs=json.loads((HERE/'INPUTS.json').read_text()); bundle=a.private/'bundle'
        require(digest(bundle/'receipt.json')==inputs['runtime_receipt_sha256'],'private receipt')
        for n,h in inputs['runtime']['tools'].items(): require(digest(bundle/n)==h,'private tool '+n)
        require((bundle/'verus-root').is_file(),'marker')
        for n,h in json.loads((HERE/'FREEZE.json').read_text()).items(): require(digest(HERE/n)==h,'frozen '+n)
        for n,h in json.loads((HERE/'AGGREGATION_FREEZE.json').read_text()).items(): require(digest(HERE/n)==h,'aggregation freeze '+n)
        def body(text,name,manual=False):
            pattern=r'    (?:pub )?fn '+name+(r'\(mut self' if name=='add_signature' else r'\(')
            m=re.search(pattern,text); require(m is not None,'missing method '+name)
            start=text.index('\n    {',m.start())+5 if manual else text.index('{',m.start())
            j=start+1; depth=1
            while depth: depth+=(text[j]=='{')-(text[j]=='}'); j+=1
            return text[start:j]
        fidelity={}
        for n in ['get','insert','iter','item_count']:
            fixture='iterator-eligibility.rs' if n=='iter' else 'count-eligibility.rs' if n=='item_count' else 'manual.rs'
            original=body((HERE/'original-node.rs').read_text(),n)
            require(original==body((HERE/fixture).read_text(),n),'original executable body '+n)
            fidelity[n]=dict(identical=True,original_file='original-node.rs',fixture=fixture)
        actual=body((HERE/'manual.rs').read_text(),'add_signature',True)
        actual=actual.replace('        assert(*signature =~= self.slots()[index.0 as int].unwrap());\n','')
        require(actual==body((HERE/'original-signature.rs').read_text(),'add_signature'),'original add_signature executable body')
        fidelity['add_signature']=dict(identical_after_erasure=True,concrete_type='[u8;64]',external_trait_integration_checked=False)
        receipt['fidelity']=fidelity
        env=dict(os.environ,PATH=str(RUST/'bin')+':/home/dev/.cargo/bin:'+os.environ.get('PATH',''),RUSTUP_TOOLCHAIN=str(RUST),RUSTUP_HOME='/home/dev/.rustup',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'),LD_LIBRARY_PATH=str(RUST/'lib'))
        files={}
        provider_inputs=[]
        if a.scenario not in ['eligibility','count']:
            for n,h in json.loads((HERE/'BOOKKEEPING_FREEZE.json').read_text()).items(): require(digest(HERE/n)==h,'bookkeeping freeze '+n)
            old=HERE.parents[1]/'practical_redb_fedimint/fedimint_quorum'
            provider=(old/'component.rs').read_text()
            for k,v in json.loads((old/'manual.json').read_text()).items():
                marker='// MANUAL_'+k+': unsolved baseline slot, never claimed as a proof.'
                require(provider.count(marker)==1,'historical slot'); provider=provider.replace(marker,v)
            provider+='\n'+(old/'adapter.rs').read_text()
            (out/'f2.rs').write_text(provider)
            provider_inputs=[old/n for n in ['component.rs','manual.json','adapter.rs']]+[out/'f2.rs']
        if a.scenario=='eligibility':
            for n in ['keychain','engine']:
                s=(HERE/('original-'+n+'.rs')).read_text()
                s='use vstd::prelude::*;\n'+re.sub(r'(?m)^(impl|pub struct|pub fn)',r'#[verus_verify]\n\1',s)
                files[n]=s
            files['iterator']=(HERE/'iterator-eligibility.rs').read_text()
        elif a.scenario=='count':
            files['count']=(HERE/'count-eligibility.rs').read_text()
        else:
            files[a.scenario]=(HERE/('manual.rs' if a.scenario in ['manual','semantic','safety','duplicate','omit-array','omit-set'] else 'client.rs' if a.scenario in ['client','client-negative','failed-import'] else a.scenario+'.rs')).read_text()
            if a.scenario=='semantic': files[a.scenario]=files[a.scenario].replace('final(self).slots() == old(self).slots().update(node_id.0 as int, Some(value)),','final(self).slots() == old(self).slots(),')
            if a.scenario=='safety': files[a.scenario]=files[a.scenario].replace('requires node_id.0 < old(self).slots().len(),','')
            if a.scenario=='omit-array': files[a.scenario]=files[a.scenario].replace('        assert(*signature =~= self.slots()[index.0 as int].unwrap());\n','')
            if a.scenario=='omit-set': files[a.scenario]=files[a.scenario].replace('assert(occupied(s.update(i as int, Some(value))) =~= occupied(s).insert(i));','')
            if a.scenario=='client-negative': files[a.scenario]=files[a.scenario].replace('assert(map.signers().len() == after_first.len());','assert(map.signers().len() == after_first.len() + 1);')
            if a.scenario=='duplicate': files[a.scenario]=files[a.scenario].replace('if self.signers().contains(index.0) { 0nat } else { 1nat }','1nat')
        if a.scenario in ['semantic','safety']:
            files[a.scenario]=files[a.scenario].split('verus! {\n// Finite occupied-index set',1)[0]
        for label,s in files.items(): (out/(label+'.rs')).write_text(s)
        guard=ArtifactGuard([bundle/n for n in inputs['runtime']['tools']]+[HERE/'FREEZE.json',HERE/'INPUTS.json',Path(__file__),TOOLS/'infer_experiment.py',TOOLS/'infer_eval_support.py']+[out/(n+'.rs') for n in files]+provider_inputs)
        run=LoggedCommands(out,out,receipt['commands'],timeout=120,guard=guard)
        if a.scenario not in ['eligibility','count']:
            cmd=[str(bundle/'verus'),str(out/'f2.rs'),'--crate-name','f2','--crate-type=lib','--edition=2024','--no-cheating','--output-json','--time','--compile','--export',str(out/'f2.vir'),'-o',str(out/'libf2.rlib')]
            code=run('provider',cmd,env)
            data=validate_run(dict(returncode=code,stdout=(out/'provider.stdout').read_text(),stderr=(out/'provider.stderr').read_text()),True)
            receipt['provider']=dict(sha256=digest(out/'f2.rs'),verification=data['verification-results'],scope=data['verus-infer']['verification_scope'])
        if a.scenario in ['client','client-negative','failed-import']:
            require(a.provider is not None,'checked provider directory required')
            pr=json.loads((a.provider/'receipt.json').read_text())
            require(pr['success'] is True and pr['scenario']=='manual','only successful checked provider')
            for x in ['manual-ordinary.vir','libmanual-ordinary.rlib']:
                require((a.provider/x).is_file(),'missing checked provider artifact')
        guarded_imports=([out/'libf2.rlib',out/'f2.vir'] if a.scenario not in ['eligibility','count'] else [])
        if a.scenario in ['client','client-negative','failed-import']:
            guarded_imports += [a.provider/n for n in ['receipt.json','manual.rs','manual-ordinary.vir','libmanual-ordinary.rlib','f2.vir','libf2.rlib']]
        import_guard=ArtifactGuard(guarded_imports)
        receipt['imports']={str(p):digest(p) for p in guarded_imports}
        for label in files:
            for mode in ['ordinary','infer']:
                name=label+'-'+mode; artifact=out/(name+'.vir')
                cmd=[str(bundle/'verus'),str(out/(label+'.rs')),'--crate-type=lib','--edition=2024','--crate-name','f3_'+label.replace('-','_'),'--no-cheating','--output-json','--time','--compile','--export',str(artifact),'-o',str(out/('lib'+name+'.rlib'))]
                if a.scenario not in ['eligibility','count']:cmd+=['--extern','f2='+str(out/'libf2.rlib'),'--import','f2='+str(out/'f2.vir')]
                if a.scenario in ['client','client-negative','failed-import']:
                    cmd[cmd.index('f2='+str(out/'libf2.rlib'))]='f2='+str(a.provider/'libf2.rlib')
                    cmd[cmd.index('f2='+str(out/'f2.vir'))]='f2='+str(a.provider/'f2.vir')
                    imp=a.provider/'manual-ordinary.vir' if a.scenario!='failed-import' else a.rejected
                    if a.scenario=='failed-import': require(imp is not None and not imp.exists(),'failed export must be absent')
                    cmd+=['-L','dependency='+str(a.provider),'--extern','f3_manual='+str(a.provider/'libmanual-ordinary.rlib'),'--import','f3_manual='+str(imp)]
                if mode=='infer':cmd+=['--infer-goals']
                import_guard.check()
                code=run(name,cmd,env); import_guard.check(); stdout=(out/(name+'.stdout')).read_text(); stderr=(out/(name+'.stderr')).read_text()
                if a.scenario=='failed-import':
                    require(code!=0 and not artifact.exists(),'failed import cannot publish')
                    require(str(a.rejected) in stderr and ('No such file' in stderr or 'could not' in stderr or 'cannot' in stderr),'specific import failure')
                    require('panicked at' not in stderr,'panic is not an import control')
                    receipt['outcomes'].append(dict(label=name,kind='failed_import',returncode=code,missing_export=str(a.rejected),stderr_sha256=digest(out/(name+'.stderr'))))
                    continue
                expected=a.scenario in ['manual','client'] or (a.scenario=='corrected-baseline' and mode=='ordinary') or (a.scenario=='omit-set' and mode=='ordinary') or (a.scenario=='eligibility' and label=='iterator')
                data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),expected,allow_frontend=a.scenario in ['eligibility','baseline'])
                v=data['verification-results']; inf=data.get('verus-infer',{})
                if a.scenario=='eligibility':
                    require((code==0)==expected,'eligibility outcome changed')
                    kind='dependency_setup' if label in ['keychain','engine'] else 'checked_iterator_body_without_contract'
                else:
                    require(v.get('is-verifying-entire-crate') is True and not v.get('encountered-vir-error'),'full crate checks')
                    if a.scenario!='baseline': require('error[E' not in stderr and not v.get('encountered-vir-error'),'setup/frontend failure is not a proof baseline')
                    kind='checked' if expected else 'type_setup' if a.scenario=='baseline' else 'count_eligibility_precondition' if a.scenario=='count' else 'safety_negative' if a.scenario=='safety' else 'semantic_or_mechanical_failure'
                    require_function_coverage(inf.get('functions',[]),[]); require_exact(inf.get('functions',[]))
                    if mode=='infer':require_publication(data,expected,artifact)
                    else:require(inf.get('interface_written') is False,'ordinary publication')
                if a.scenario!='eligibility' and a.scenario!='baseline':
                    exe=['clone','get','insert']; proofs=[]
                    if a.scenario=='count': exe=['clone','iter','item_count']
                    elif a.scenario in ['client','client-negative']: exe=['retransmission']
                    elif a.scenario not in ['semantic','safety']:
                        exe+=['record_signature']; proofs=['occupied_finite','occupied_update']
                        if a.scenario!='corrected-baseline': exe+=['add_signature']
                    scope=inf['verification_scope']
                    require(scope['native_collection_completed'] is True,'native collection')
                    for k,names in [('exec',exe),('proof',proofs)]:
                        if expected:
                            bodies=[b for b in scope['native_functions'] if b['body_status']=='checked_'+k+'_body']
                            require_function_coverage(bodies,names)
                            require(scope['checked_'+k+'_bodies']==len(names),'checked count')
                    if not expected:
                        require(scope['checked_exec_bodies']==scope['checked_proof_bodies']==0,'failed crate certifies nothing')
                        require_function_coverage([b for b in scope['native_functions'] if b['body_status']=='selected_body_not_certified_by_successful_full_run'],exe+proofs)
                    require(not any(b.get('selected') and b['body_status'] in ['unverified_body_with_contract','external_specification','interface_without_checked_body'] for b in scope['native_functions']),'unchecked local summary')
                    failures={f.rsplit('::',1)[-1] for f in inf['failed_functions']}
                    intended=set()
                    if a.scenario=='count': intended={'item_count'}
                    elif a.scenario in ['semantic','safety']: intended={'insert'}
                    elif a.scenario in ['duplicate','omit-array']: intended={'add_signature'}
                    elif a.scenario=='client-negative': intended={'retransmission'}
                    elif a.scenario in ['corrected-baseline','omit-set'] and mode=='infer': intended={'occupied_update'}
                    elif a.scenario=='aggregation': intended={'add_signature'} | ({'occupied_update'} if mode=='infer' else set())
                    require(failures==intended,'exact failing functions: '+str(failures))
                    if not expected:
                        diagnostic='precondition not satisfied' if a.scenario in ['count','safety'] else 'assertion failed' if a.scenario=='client-negative' else 'postcondition not satisfied'
                        require(diagnostic in stderr,'intended failure missing')
                        if a.scenario not in ['safety','omit-array']:
                            require('recommendation not met' not in stderr and 'possible arithmetic underflow/overflow' not in stderr,'unhealthy semantic negative')
                receipt['outcomes'].append(dict(label=name,kind=kind,returncode=code,source_sha256=digest(out/(label+'.rs')),verification=v,infer=inf,times=data.get('times-ms'),export_exists=artifact.exists()))
        receipt['runtime']=inputs['runtime']; receipt['freeze_commit']='8fb5fce2'; receipt['runner_sha256']=digest(Path(__file__))
    print(str(out/'receipt.json'))
if __name__=='__main__': main()
