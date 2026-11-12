#!/usr/bin/env python3
"""Private accepted78 tools; bounded exact-source probes; full ordinary checks."""
import argparse, json, os, shutil, sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_publication
from probes import generate

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    p.add_argument('--cases',nargs='+',default=['publication','selection','serialization'])
    p.add_argument('--validate',action='store_true');p.add_argument('--imports',action='store_true')
    a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    r=dict(kind='BOUNDED_ORIGINAL_BODY_ELIGIBILITY_NOT_APPLICATION_THEOREM',commands=[],outcomes=[],source_coupling_proved=False)
    with replay_receipt(out,r):
        frozen=json.loads((HERE/'FROZEN.json').read_text()); rt=frozen['runtime'];src=Path(rt['bundle'])
        require(digest(src/'receipt.json')==rt['receipt_sha256'],'receipt')
        frozen['source_sha256'].update(json.loads((HERE/'DEPENDENCIES02.json').read_text())['additional_source_sha256'])
        for n,h in frozen['source_sha256'].items(): require(digest(HERE/'original'/n)==h,n)
        for n,h in rt['tool_hashes'].items(): require(digest(src/n)==h,n)
        bundle=out/'bundle';bundle.mkdir()
        for n in rt['tool_hashes']: shutil.copy2(src/n,bundle/n)
        shutil.copy2(src/'verus-root',bundle/'verus-root')
        r['fixtures']=generate(out)
        toolchain=frozen['toolchain']
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+toolchain+'/bin:/usr/bin:/bin',
            RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',
            VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',LD_LIBRARY_PATH=toolchain+'/lib',VERUS_Z3_PATH=str(bundle/'z3'))
        guard=ArtifactGuard([HERE/'FROZEN.json',HERE/'probes.py',HERE/'replay.py',
            *[HERE/'original'/n for n in frozen['source_sha256']],*[bundle/n for n in rt['tool_hashes']],
            *[out/(n+'.rs') for n in r['fixtures']]])
        run=LoggedCommands(out,out,r['commands'],timeout=120,guard=guard)
        for case in a.cases:
            for goals in [False,True]:
                label=case+('_infer' if goals else '_ordinary');artifact=out/(label+'.vir')
                artifact.write_text('seeded stale interface\n')
                cmd=[str(bundle/'verus'),str(out/(case+'.rs')),'--crate-type=lib','--edition=2024',
                    '--crate-name',case,'--no-cheating','--output-json','--time','--multiple-errors','8',
                    '--export',str(artifact)]+(['--infer-goals'] if goals else [])
                if case == 'supported': cmd += ['--compile', '-o', str(out/('lib_'+label+'.rlib'))]
                code=run(label,cmd,dict(env,VERUS_INFER_CACHE=str(out/('cache_'+label))))
                data=json.loads((out/(label+'.stdout')).read_text())
                err=(out/(label+'.stderr')).read_text()
                v=data['verification-results']
                if v.get('success') is True: classification='checked'
                elif 'error[E' in err: classification='context'
                elif v.get('encountered-vir-error') or ('not yet supported' in err or 'not supported' in err): classification='frontend'
                elif 'postcondition not satisfied' in err or 'precondition not satisfied' in err or 'assertion failed' in err: classification='proof_or_contract'
                elif 'precondition not met: index in bounds' in err: classification='safety'
                else: classification='other_frontend_or_contract'
                require(not any(x in err.lower() for x in ['panicked at','resource limit','timed out']),'resource failure, not semantic evidence')
                if a.validate:
                    validate(case,goals,code,data,err,artifact)
                r['outcomes'].append(dict(label=label,classification=classification,returncode=code,
                    verification=v,infer=data.get('verus-infer'),times=data.get('times-ms'),
                    export_exists=artifact.exists(),stderr_sha256=digest(out/(label+'.stderr'))))
                print(label,classification,v,flush=True)
        if a.imports: imports(out,env,run,r)
        guard.check();r['guard_passed']=True


EXEC = {'DatabaseHeader::primary_slot','DatabaseHeader::secondary_slot',
        'DatabaseHeader::published_slot','DatabaseHeader::swap_primary_slot','DatabaseHeader::write_secondary_slot'}
CLONES = {'<'+n+' as std::clone::Clone>::clone' for n in ['PageNumber','BtreeHeader','TransactionId','TransactionHeader','DatabaseHeader']}
def scope(data, positive, expected_exec, expected_proof):
    inf=data['verus-infer']; require_exact(inf['functions']);require_function_coverage(inf['functions'],[])
    sc=inf['verification_scope'];require(sc['native_collection_completed'],'native inventory')
    if positive: require(sc['complete_selected_scope_verified'],'complete selected scope')
    actual={b['rust_function'] for b in sc['native_functions'] if b['body_status']=='checked_exec_body'}
    proofs={b['rust_function'] for b in sc['native_functions'] if b['body_status']=='checked_proof_body'}
    if positive:
        require(actual==expected_exec, 'exec inventory '+str(actual))
        require(proofs==expected_proof, 'proof inventory '+str(proofs))
        require(sc['checked_exec_bodies']==len(actual) and sc['checked_proof_bodies']==len(proofs),'scope counts')
    require(not any(b['selected'] and b['body_status'] in ['external_specification','unverified_body_with_contract','interface_without_checked_body'] for b in sc['native_functions']),'unchecked selected contract')
def validate(case,goals,code,data,err,artifact):
    if case in ['publication','supported','publication_wrong','publication_unsafe']:
        positive=case in ['publication','supported']
        validate_run(dict(returncode=code,stdout=json.dumps(data),stderr=err),positive)
        scope(data,positive,EXEC|CLONES|({'publication_witness'} if case=='supported' else set()), {'slot_domain'} if case=='supported' else set())
        require(data['verification-results']['is-verifying-entire-crate'],'whole selected crate')
        if case=='publication_wrong':
            require('postcondition not satisfied' in err and data['verification-results']['errors']==1,'semantic control')
            require('precondition not' not in err and 'overflow' not in err,'mixed semantic failure')
        if case=='publication_unsafe':
            require(err.count('error: precondition not met: index in bounds')==2,'safety control')
            require('postcondition not satisfied' not in err,'mixed safety failure')
        if goals: require_publication(data,positive,artifact)
        elif positive: require(artifact.exists(),'positive ordinary export')
    else:
        validate_run(dict(returncode=code,stdout=json.dumps(data),stderr=err),False,allow_frontend=True)
        require(not artifact.exists(),'frontend stale export')
        require(data['verification-results'].get('encountered-vir-error'),'expected actual frontend blocker')
        required=['function pointer types'] if case=='serialization' else ['core::io::error::Error','core::panic::location::Location']
        require(all(x in err for x in required),'unexpected blocker')
def imports(out,env,run,r):
    base=r"""extern crate supported;
use supported::*;
use vstd::prelude::*;
verus! {
pub fn imported_publication(h: &DatabaseHeader) -> (r: &TransactionHeader)
    requires h.primary_slot < 2, (h.primary_slot ^ 1) < 2, h.primary_slot != (h.primary_slot ^ 1),
    ensures *r == h.transaction_slots[(if h.primary_unflushed { h.primary_slot ^ 1 } else { h.primary_slot }) as int],
{ h.published_slot() }
}
"""
    for provider in ['supported_ordinary','supported_infer']:
        for case in ['healthy','wrong','unsafe','failed_import']:
            text=base
            if case=='wrong':text=text.replace('if h.primary_unflushed { h.primary_slot ^ 1 } else { h.primary_slot }','if h.primary_unflushed { h.primary_slot } else { h.primary_slot ^ 1 }')
            if case=='unsafe':text=text.replace('requires h.primary_slot < 2, (h.primary_slot ^ 1) < 2, h.primary_slot != (h.primary_slot ^ 1),','requires true,')
            label=provider+'_client_'+case;src=out/(label+'.rs');src.write_text(text)
            target=out/(label+'.vir');shutil.copy2(out/(provider+'.vir'),target)
            imported=out/(provider+'.vir') if case!='failed_import' else out/(provider+'_client_wrong.vir')
            cmd=[str(out/'bundle/verus'),str(src),'--crate-type=lib','--edition=2024','--crate-name','client',
                '--no-cheating','--output-json','--time','--infer-goals','--export',str(target),
                '--extern','supported='+str(out/('lib_'+provider+'.rlib')),'--import','supported='+str(imported)]
            code=run(label,cmd,dict(env,VERUS_INFER_CACHE=str(out/('cache_'+label))))
            stdout=(out/(label+'.stdout')).read_text();err=(out/(label+'.stderr')).read_text()
            if case=='failed_import':
                require(code!=0 and ('No such file' in err or 'failed to read' in err),'failed import must reject')
                data=json.loads(stdout) if stdout.strip() else None
            else:
                data=validate_run(dict(returncode=code,stdout=stdout,stderr=err),case=='healthy')
                scope(data,case=='healthy',{'imported_publication'},set())
                require_publication(data,case=='healthy',target)
                if case=='wrong':require('postcondition not satisfied' in err,'import semantic control')
                if case=='unsafe':require('precondition not satisfied' in err,'import safety control')
            r['outcomes'].append(dict(label=label,classification=case,returncode=code,data=data,
                export_exists=target.exists(),stderr_sha256=digest(out/(label+'.stderr'))))
            print(label,code,flush=True)

if __name__=='__main__':main()
