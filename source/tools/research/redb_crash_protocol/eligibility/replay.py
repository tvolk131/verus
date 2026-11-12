#!/usr/bin/env python3
"""Fresh private accepted78 runs with immutable inputs and complete receipts."""
import argparse, json, os, shutil, sys
sys.dont_write_bytecode=True
from pathlib import Path
from fixtures import HERE, baseline, contracted, closeout_cases
sys.path.insert(0,str(HERE.parents[2]))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require, require_exact, require_function_coverage, validate_run, require_publication

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    p.add_argument('--contracts',action='store_true')
    p.add_argument('--closeout',action='store_true')
    a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    r=dict(kind='COMPLETE_SELECTION_NOMINAL_ONLY_ELIGIBILITY',commands=[],outcomes=[],source_coupling_proved=False)
    with replay_receipt(out,r):
        f=json.loads((HERE/'FROZEN.json').read_text());rt=f['runtime'];src=Path(rt['bundle'])
        for n,h in f['inputs'].items():require(digest(HERE.parent/n)==h,n)
        for n,h in f['shared_helpers'].items():require(digest(HERE.parents[4]/n)==h,n)
        require(digest(src/'receipt.json')==rt['receipt_sha256'],'accepted receipt')
        for n,h in rt['tool_hashes'].items():require(digest(src/n)==h,n)
        bundle=out/'bundle';bundle.mkdir()
        for n in rt['tool_hashes']:shutil.copy2(src/n,bundle/n)
        shutil.copy2(src/'verus-root',bundle/'verus-root')
        r['authentication']=dict(receipt_sha256=digest(src/'receipt.json'),tools={n:digest(bundle/n) for n in rt['tool_hashes']},helpers=f['shared_helpers'])
        texts=closeout_cases() if a.closeout else {'selection':contracted()} if a.contracts else {'without_nominal':baseline(False),'with_nominal':baseline()}
        for n,t in texts.items():(out/(n+'.rs')).write_text(t)
        if not (a.contracts or a.closeout):
            require(digest(out/'with_nominal.rs')==f['baseline_sha256'],'baseline freeze')
            require(digest(out/'without_nominal.rs')==f['control_sha256'],'control freeze')
        r['fixtures']={n:digest(out/(n+'.rs')) for n in texts}
        toolchain=f['toolchain']
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+toolchain+'/bin:/usr/bin:/bin',
            RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',
            VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',LD_LIBRARY_PATH=toolchain+'/lib',VERUS_Z3_PATH=str(bundle/'z3'))
        guard=ArtifactGuard([HERE/'FROZEN.json',HERE/'fixtures.py',HERE/'replay.py',
            *[HERE.parent/n for n in f['inputs']],*[bundle/n for n in rt['tool_hashes']],
            *[out/(n+'.rs') for n in texts]])
        run=LoggedCommands(out,out,r['commands'],timeout=120,guard=guard)
        for case in texts:
            for infer in [False,True]:
                label=case+('_infer' if infer else '_ordinary');artifact=out/(label+'.vir')
                artifact.write_text('seeded stale interface\n')
                cmd=[str(bundle/'verus'),str(out/(case+'.rs')),'--crate-type=lib','--edition=2024',
                    '--crate-name',case,'--no-cheating','--output-json','--time','--multiple-errors','8',
                    '--export',str(artifact)]+(['--infer-goals'] if infer else [])
                if case == 'eligible':cmd += ['--compile','-o',str(out/('lib_'+label+'.rlib'))]
                code=run(label,cmd,dict(env,VERUS_INFER_CACHE=str(out/('cache_'+label))))
                data=json.loads((out/(label+'.stdout')).read_text());err=(out/(label+'.stderr')).read_text()
                require(not any(x in err.lower() for x in ['panicked at','resource limit','timed out']),'resource failure')
                inf=data.get('verus-infer',{});require_exact(inf.get('functions',[]));require_function_coverage(inf.get('functions',[]),[])
                if a.closeout:validate_closeout(case,infer,code,data,err,artifact)
                v=data['verification-results']
                cls='checked' if v.get('success') else ('frontend' if v.get('encountered-vir-error') else 'contract_or_safety')
                r['outcomes'].append(dict(label=label,classification=cls,returncode=code,data=data,
                    export_exists=artifact.exists(),stderr_sha256=digest(out/(label+'.stderr'))))
                print(label,cls,v,flush=True)
        if a.closeout:imports(out,env,run,r)
        guard.check();r['guard_passed']=True


EXEC={'DatabaseHeader::'+n for n in ['primary_slot','secondary_slot','published_slot','swap_primary_slot','write_secondary_slot']}
EXEC|={'UnrepairedDatabaseHeader::select_primary_slot','UnrepairedDatabaseHeader::finalize_transaction_slots'}
EXEC|={'<'+n+' as std::clone::Clone>::clone' for n in ['PageNumber','BtreeHeader','TransactionId','TransactionHeader','DatabaseHeader']}
def scope(data,positive,names):
    inf=data['verus-infer'];require_exact(inf['functions']);require_function_coverage(inf['functions'],[])
    sc=inf['verification_scope'];require(sc['native_collection_completed'],'inventory')
    require(data['verification-results']['is-verifying-entire-crate'],'whole crate')
    status='checked_exec_body' if positive else 'selected_body_not_certified_by_successful_full_run'
    actual={b['rust_function'] for b in sc['native_functions'] if b['body_status']==status}
    require(actual==names,'body inventory '+str(actual))
    require(not any(b['selected'] and b['body_status'] in ['unverified_body_with_contract','external_specification','interface_without_checked_body'] for b in sc['native_functions']),'unchecked selected body')
    if positive:
        require(sc['complete_selected_scope_verified'] and sc['checked_exec_bodies']==len(names) and sc['checked_proof_bodies']==0,'complete scope counts')
def validate_closeout(case,infer,code,data,err,artifact):
    positive=case=='eligible'
    validate_run(dict(returncode=code,stdout=json.dumps(data),stderr=err),positive)
    scope(data,positive,EXEC)
    if infer:require_publication(data,positive,artifact)
    elif positive:require(artifact.exists(),'ordinary checked export')
    if case=='finalize_wrong':
        require('postcondition not satisfied' in err and 'precondition not satisfied' not in err,'semantic control')
        require(data['verification-results']['errors']==1,'semantic error count')
    if case=='selection_unsafe':
        require('precondition not satisfied' in err and 'postcondition not satisfied' not in err,'safety domain control')
        require(data['verification-results']['errors']==1,'safety error count')
def imports(out,env,run,r):
    base=r"""extern crate eligible;
use eligible::*;
use vstd::prelude::*;
verus! {
pub fn imported_finalize(h: UnrepairedDatabaseHeader) -> (r: Result<DatabaseHeader>)
    ensures r.is_Err() == h.primary_corrupted,
        match r {
            Ok(header) => header == h.inner,
            Err(StorageError::Corrupted(msg)) => msg@ == "Primary commit slot is corrupted"@,
            _ => false,
        },
{ h.finalize_transaction_slots() }
pub fn imported_selection(h: &mut UnrepairedDatabaseHeader) -> Result<bool>
    requires old(h).inner.primary_slot < 2, (old(h).inner.primary_slot ^ 1) < 2,
        old(h).inner.primary_slot != (old(h).inner.primary_slot ^ 1),
{ h.select_primary_slot() }
}
"""
    for provider in ['eligible_ordinary','eligible_infer']:
        for case in ['healthy','wrong','unsafe','failed_import']:
            text=base
            if case=='wrong':text=text.replace('r.is_Err() == h.primary_corrupted','r.is_Err() == !h.primary_corrupted')
            if case=='unsafe':text=text.replace('requires old(h).inner.primary_slot < 2, (old(h).inner.primary_slot ^ 1) < 2,\n        old(h).inner.primary_slot != (old(h).inner.primary_slot ^ 1),','requires true,')
            label=provider+'_client_'+case;source=out/(label+'.rs');source.write_text(text)
            target=out/(label+'.vir');shutil.copy2(out/(provider+'.vir'),target)
            imported=out/(provider+'.vir') if case!='failed_import' else out/(provider+'_client_wrong.vir')
            cmd=[str(out/'bundle/verus'),str(source),'--crate-type=lib','--edition=2024','--crate-name','client',
                '--no-cheating','--output-json','--time','--infer-goals','--export',str(target),
                '--extern','eligible='+str(out/('lib_'+provider+'.rlib')),'--import','eligible='+str(imported)]
            code=run(label,cmd,dict(env,VERUS_INFER_CACHE=str(out/('cache_'+label))))
            stdout=(out/(label+'.stdout')).read_text();err=(out/(label+'.stderr')).read_text()
            if case=='failed_import':
                require(code!=0 and ('No such file' in err or 'failed to read' in err),'missing failed export rejects')
                data=json.loads(stdout) if stdout.strip() else None
            else:
                data=validate_run(dict(returncode=code,stdout=stdout,stderr=err),case=='healthy')
                scope(data,case=='healthy',{'imported_finalize','imported_selection'})
                require_publication(data,case=='healthy',target)
                require('postcondition not satisfied' in err if case=='wrong' else True,'import semantic control')
                require('precondition not satisfied' in err if case=='unsafe' else True,'import safety control')
            r['outcomes'].append(dict(label=label,returncode=code,data=data,export_exists=target.exists(),stderr_sha256=digest(out/(label+'.stderr'))))
            print(label,code,flush=True)
if __name__=='__main__':main()
