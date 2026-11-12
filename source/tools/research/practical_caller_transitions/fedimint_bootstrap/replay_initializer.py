#!/usr/bin/env python3
"""Focused initializer checking on registered private inputs; existing replay utilities."""
import argparse,json,os,re,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands,replay_receipt,require,validate_run,require_exact,require_function_coverage,require_publication
from infer_eval_support import ArtifactGuard,digest
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
EXEC={'<aleph_bft::NodeCount as std::convert::From<usize>>::from',
      'aleph_bft::<impl std::convert::From<aleph_bft::NodeCount> for usize>::from',
      '<aleph_bft::NodeIndex as std::clone::Clone>::clone',
      'aleph_bft::NodeMap::<T>::with_size','aleph_bft::NodeMap::<T>::insert','Keychain::bootstrap_multi'}

def body(text,name):
    m=re.search(r'    (?:pub )?fn '+name+r'\(',text);require(m is not None,'missing '+name)
    start=text.index('{',m.start());j=start+1;depth=1
    while depth:depth+=(text[j]=='{')-(text[j]=='}');j+=1
    return text[start:j]

def fidelity(text):
    old=(HERE/'original-node.rs').read_text();original=(HERE/'original-keychain.rs').read_text()
    for n in ['with_size','insert']:require(body(text,n)==body(old,n),'original body drift '+n)
    require(body(text,'bootstrap_multi')==body(original,'bootstrap_multi'),'caller executable drift')
    require(re.search(r'pub struct Keychain\s*\{[^}]+\}',text).group()==re.search(r'pub struct Keychain\s*\{[^}]+\}',original).group(),'all genuine fields must match exactly')
    expanded=(HERE/'expanded-nodecount.rs.txt').read_text()
    require(body(text.split('impl derive_more::From<(usize)> for NodeCount {',1)[1],'from')==body(expanded.split('impl derive_more::From<(usize)> for NodeCount {',1)[1],'from'),'actual generated From<usize>')
    require(body(text.split('impl derive_more::core::convert::From<NodeCount> for (usize) {',1)[1],'from')==body(expanded.split('impl derive_more::core::convert::From<NodeCount> for (usize) {',1)[1],'from'),'actual generated From<NodeCount>')
    require(text.count('#[verifier::external_type_specification]')==3,'exactly three nominal domains')

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    p.add_argument('--scenario',choices=['setup','positive','written-none','length','others','count','safety','client','failed-import'],default='positive')
    p.add_argument('--provider',type=Path);p.add_argument('--failed-export',type=Path);a=p.parse_args()
    a.output.mkdir(parents=True,exist_ok=False);r=dict(kind='F3C_ORIGINAL_BOOTSTRAP',scenario=a.scenario,commands=[],outcomes=[])
    with replay_receipt(a.output,r):
        f=json.loads((HERE/'BOOTSTRAP_INPUTS.json').read_text());base=Path(f['private_root']);bundle=base/'bundle'
        for n,h in f['files'].items():require(digest(HERE/n)==h,'frozen source '+n)
        for manifest in ['ADAPTER_CORRECTION.json','VALIDATION_FREEZE.json']:
            for n,h in json.loads((HERE/manifest).read_text()).items():require(digest(HERE/n)==h,'frozen adapter/validation '+n)
        require(digest(Path(f['context_receipt']))==f['original_context_receipt_sha256'],'exact registered context')
        require(digest(Path(f['expansion_receipt']))==f['expansion_receipt_sha256'],'actual expansion receipt')
        require(json.loads(Path(f['expansion_receipt']).read_text())['success'],'failed expansion')
        for n,h in f['artifacts'].items():require(digest(Path(n))==h,'registered private native lib '+n)
        require(digest(bundle/'receipt.json')==f['runtime_receipt_sha256'],'runtime receipt')
        require(f['runtime']['success'] and f['runtime']['source_commit']=='5ecea5fe81c2bc2ef0c9050cdb29cea0c7ef21c4' and len(f['runtime']['tools'])==8,'accepted runtime')
        for n,h in f['runtime']['tools'].items():require(digest(bundle/n)==h,'private accepted tool '+n)
        require((bundle/'verus-root').is_file(),'complete runtime marker')
        original=HERE/('baseline.rs' if a.scenario=='setup' else 'client.rs' if a.scenario in ['client','failed-import'] else 'corrected-baseline.rs')
        text=original.read_text()
        mutations={'written-none':('r.slots()[index.0 as int] == Some(*signature)','r.slots()[index.0 as int] == None'),
            'length':('r.slots().len() == self.configured_len()','r.slots().len() == self.configured_len() + 1'),
            'others':('==> #[trigger] r.slots()[j] == None','==> #[trigger] r.slots()[j] == Some(*signature)'),
            'count':('r.signers().len() == 1','r.signers().len() == 2'),
            'safety':('requires index.0 < self.configured_len(),','')}
        if a.scenario in mutations:
            old,new=mutations[a.scenario];require(text.count(old)==1,'single mutation only');text=text.replace(old,new)
        if a.scenario not in ['client','failed-import']:fidelity(text)
        source=a.output/'initializer.rs';source.write_text(text)
        guarded=[original,source,Path(__file__),HERE/'BOOTSTRAP_INPUTS.json',HERE/'VALIDATION_FREEZE.json',Path(f['context_receipt']),Path(f['expansion_receipt'])]+[Path(n) for n in f['artifacts']]+[bundle/n for n in f['runtime']['tools']]
        if a.scenario in ['client','failed-import']:
            require(a.provider is not None,'checked provider required');pr=json.loads((a.provider/'receipt.json').read_text())
            require(pr['success'] and pr['scenario']=='positive' and pr['frozen_source_sha256']==digest(HERE/'corrected-baseline.rs'),'only checked exact frozen provider')
            for n,h in pr['artifacts'].items():require(digest(a.provider/n)==h,'checked import bytes '+n);guarded.append(a.provider/n)
            guarded.append(a.provider/'receipt.json')
        guard=ArtifactGuard(guarded);run=LoggedCommands(a.output,a.output,r['commands'],timeout=180,guard=guard)
        r['inputs']={str(p):digest(p) for p in guarded};r['manual_nominal_domains']=3;r['frozen_source_sha256']=digest(original)
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+str(TC/'bin')+':/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN=str(TC),LD_LIBRARY_PATH=str(TC/'lib'),VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(a.output/'cache'))
        for mode in ['ordinary','infer']:
            artifact=a.output/(mode+'.vir')
            if mode=='infer' and a.scenario not in ['positive','client','failed-import']:artifact.write_text('STALE-EXPORT-MUST-BE-REMOVED')
            cmd=[str(bundle/'verus'),str(source),'--edition=2024','--crate-type=lib','--crate-name','f3c_client' if a.scenario in ['client','failed-import'] else 'f3c_bootstrap','--no-cheating','--output-json','--time','--compile','--export',str(artifact),'-o',str(a.output/('lib'+mode+'.rlib')),'--extern','secp256k1='+str(base/'context/libs/libsecp256k1.rlib'),'-L','dependency='+str(base/'context/libs')]
            if a.scenario in ['client','failed-import']:
                imp=a.provider/'ordinary.vir' if a.scenario=='client' else a.failed_export
                if a.scenario=='failed-import':require(imp is not None and not imp.exists(),'failed export must actually be absent')
                cmd+=['--extern','f3c_bootstrap='+str(a.provider/'libordinary.rlib'),'--import','f3c_bootstrap='+str(imp),'-L','dependency='+str(a.provider)]
            if mode=='infer':cmd+=['--infer-goals']
            code=run(mode,cmd,env);stdout=(a.output/(mode+'.stdout')).read_text();stderr=(a.output/(mode+'.stderr')).read_text()
            if a.scenario=='failed-import':
                require(code!=0 and not artifact.exists() and str(imp) in stderr and 'could not open imported library file' in stderr and 'panicked at' not in stderr,'specific failed import rejection')
                r['outcomes'].append(dict(mode=mode,kind='failed_import_not_proof',returncode=code,stderr_sha256=digest(a.output/(mode+'.stderr'))));continue
            expected=a.scenario in ['positive','client']
            data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),expected,allow_frontend=a.scenario=='setup')
            v=data['verification-results'];inf=data['verus-infer'];scope=inf['verification_scope']
            require_function_coverage(inf.get('functions',[]),[]);require_exact(inf.get('functions',[]))
            if a.scenario!='setup':
                require(v['is-verifying-entire-crate'] and not v['encountered-vir-error'] and 'error[E' not in stderr,'full ordinary checks, not setup')
                require(scope['native_collection_completed'],'complete body collection')
                names=EXEC if a.scenario!='client' else {'initializer_observation','zero_constructor_and_conversion_boundaries'}
                status='checked_exec_body' if expected else 'selected_body_not_certified_by_successful_full_run'
                bodies=[b for b in scope['native_functions'] if b['body_status']==status]
                require({b['rust_function'] for b in bodies}==names and len(bodies)==len(names),'exact local exec inventory')
                require(all(b['selected'] and b['source'].startswith(str(source)+':') for b in bodies),'checked/selected source coverage')
                require(scope['checked_exec_bodies']==(len(names) if expected else 0) and scope['checked_proof_bodies']==0,'exact exec/proof counts')
                require(not any(b['body_status']=='checked_proof_body' or b['body_status'] in ['unverified_body_with_contract','external_specification','interface_without_checked_body'] and b['selected'] for b in scope['native_functions']),'unchecked local summary or extra proof')
                failed=set(inf['failed_functions']);require(failed==(set() if expected else {'f3c_bootstrap::Keychain::bootstrap_multi'}),'exact failed caller')
                if expected:require(scope['complete_selected_scope_verified'],'entire proof')
                else:
                    diagnostic='precondition not satisfied' if a.scenario=='safety' else 'postcondition not satisfied'
                    require(diagnostic in stderr,'intended diagnostic')
                    if a.scenario!='safety':require('recommendation not met' not in stderr and 'precondition not satisfied' not in stderr and 'possible arithmetic underflow/overflow' not in stderr,'unhealthy semantic negative')
                breakdown=[b for m in data['times-ms']['smt']['smt-run-module-times'] for b in m.get('function-breakdown',[])]
                require(all(b['success'] for b in breakdown if b['function'] not in failed),'healthy dependencies')
            if mode=='infer':require_publication(data,expected,artifact)
            else:require(inf['interface_written'] is False,'ordinary Infer publication')
            r['outcomes'].append(dict(mode=mode,kind='setup' if a.scenario=='setup' else 'safety_negative' if a.scenario=='safety' else 'checked' if expected else 'semantic_negative',returncode=code,verification=v,scope=scope,failed_functions=inf['failed_functions'],inferred_helpers=inf['functions'],times=data['times-ms'],interface_written=inf['interface_written'],export_exists=artifact.exists(),stderr_sha256=digest(a.output/(mode+'.stderr'))))
            print(mode,v,flush=True)
        if a.scenario=='positive':r['artifacts']={n:digest(a.output/n) for n in ['ordinary.vir','libordinary.rlib','infer.vir','libinfer.rlib','initializer.rs']}
if __name__=='__main__':main()
