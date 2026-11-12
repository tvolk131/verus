#!/usr/bin/env python3
"""Small F3 controller using the existing experiment/evaluation utilities."""
import argparse, json, os, re, shutil, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
TOOLS=HERE.parents[2]
sys.path.insert(0,str(TOOLS))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_publication
RUST=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
def main():
    p=argparse.ArgumentParser(); p.add_argument('--output',type=Path,required=True)
    p.add_argument('--private',type=Path,default=Path('/tmp/fedimint-f3-3ad8038e-auth'))
    p.add_argument('--scenario',choices=['eligibility','baseline','corrected-baseline','manual','semantic','safety','ablation','client','bad-import'],default='eligibility')
    a=p.parse_args(); out=a.output; out.mkdir(parents=True,exist_ok=False)
    receipt=dict(scenario=a.scenario,commands=[],outcomes=[])
    with replay_receipt(out,receipt):
        inputs=json.loads((HERE/'INPUTS.json').read_text()); bundle=a.private/'bundle'
        require(digest(bundle/'receipt.json')==inputs['runtime_receipt_sha256'],'private receipt')
        for n,h in inputs['runtime']['tools'].items(): require(digest(bundle/n)==h,'private tool '+n)
        require((bundle/'verus-root').is_file(),'marker')
        for n,h in json.loads((HERE/'FREEZE.json').read_text()).items(): require(digest(HERE/n)==h,'frozen '+n)
        env=dict(os.environ,PATH=str(RUST/'bin')+':/home/dev/.cargo/bin:'+os.environ.get('PATH',''),RUSTUP_TOOLCHAIN=str(RUST),RUSTUP_HOME='/home/dev/.rustup',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'),LD_LIBRARY_PATH=str(RUST/'lib'))
        files={}
        provider_inputs=[]
        if a.scenario!='eligibility':
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
        else:
            files[a.scenario]=(HERE/('manual.rs' if a.scenario in ['manual','semantic','safety','ablation'] else a.scenario+'.rs')).read_text()
            if a.scenario=='semantic': files[a.scenario]=files[a.scenario].replace('self.slots() == old(self).slots().update(node_id.0 as int, Some(value)),','self.slots() == old(self).slots(),')
            if a.scenario=='safety': files[a.scenario]=files[a.scenario].replace('requires node_id.0 < old(self).slots().len(),','')
            if a.scenario=='ablation': files[a.scenario]=files[a.scenario].replace('// MECHANICAL_BEGIN', '/*').replace('// MECHANICAL_END', '*/')
        for label,s in files.items(): (out/(label+'.rs')).write_text(s)
        guard=ArtifactGuard([bundle/n for n in inputs['runtime']['tools']]+[HERE/'FREEZE.json',HERE/'INPUTS.json',Path(__file__),TOOLS/'infer_experiment.py',TOOLS/'infer_eval_support.py']+[out/(n+'.rs') for n in files]+provider_inputs)
        run=LoggedCommands(out,out,receipt['commands'],timeout=120,guard=guard)
        if a.scenario!='eligibility':
            cmd=[str(bundle/'verus'),str(out/'f2.rs'),'--crate-name','f2','--crate-type=lib','--edition=2024','--no-cheating','--output-json','--time','--compile','--export',str(out/'f2.vir'),'-o',str(out/'libf2.rlib')]
            code=run('provider',cmd,env)
            data=validate_run(dict(returncode=code,stdout=(out/'provider.stdout').read_text(),stderr=(out/'provider.stderr').read_text()),True)
            receipt['provider']=dict(sha256=digest(out/'f2.rs'),verification=data['verification-results'],scope=data['verus-infer']['verification_scope'])
        for label in files:
            for mode in ['ordinary','infer']:
                name=label+'-'+mode; artifact=out/(name+'.vir')
                cmd=[str(bundle/'verus'),str(out/(label+'.rs')),'--crate-type=lib','--edition=2024','--crate-name','f3_'+label.replace('-','_'),'--no-cheating','--output-json','--time','--compile','--export',str(artifact),'-o',str(out/('lib'+name+'.rlib'))]
                if a.scenario!='eligibility':cmd+=['--extern','f2='+str(out/'libf2.rlib'),'--import','f2='+str(out/'f2.vir')]
                if mode=='infer':cmd+=['--infer-goals']
                code=run(name,cmd,env); stdout=(out/(name+'.stdout')).read_text(); stderr=(out/(name+'.stderr')).read_text()
                expected=a.scenario in ['manual','client'] or (a.scenario=='eligibility' and label=='iterator')
                data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),expected,allow_frontend=a.scenario=='eligibility')
                v=data['verification-results']; inf=data.get('verus-infer',{})
                if a.scenario=='eligibility':
                    require((code==0)==expected,'eligibility outcome changed')
                    kind='dependency_setup' if label in ['keychain','engine'] else 'checked_iterator_body_without_contract'
                else:
                    require(v.get('is-verifying-entire-crate') is True and not v.get('encountered-vir-error'),'full crate checks')
                    require(not v.get('encountered-error') and 'error[E' not in stderr,'setup/frontend failure is not a proof baseline')
                    kind='checked' if expected else ('safety_negative' if a.scenario=='safety' else 'semantic_or_mechanical_failure')
                    require_function_coverage(inf.get('functions',[]),[]); require_exact(inf.get('functions',[]))
                    if mode=='infer':require_publication(data,expected,artifact)
                    else:require(inf.get('interface_written') is False,'ordinary publication')
                receipt['outcomes'].append(dict(label=name,kind=kind,returncode=code,source_sha256=digest(out/(label+'.rs')),verification=v,infer=inf,times=data.get('times-ms'),export_exists=artifact.exists()))
        receipt['runtime']=inputs['runtime']; receipt['freeze_commit']='8fb5fce2'; receipt['runner_sha256']=digest(Path(__file__))
    print(str(out/'receipt.json'))
if __name__=='__main__': main()
