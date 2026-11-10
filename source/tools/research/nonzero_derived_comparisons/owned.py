"""Guarded normal vstd build and application trials of an owned matching build."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
from trial import ROOT, BUNDLE, RUSTC, authenticate, source
sys.path.insert(0,str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require,require_exact

def application(case):
    if case=='standard_order':
        return '''use vstd::prelude::*;
use std::num::NonZero;
verus! {
fn standard_cmp(a:&NonZero<u64>,b:&NonZero<u64>)->(r:core::cmp::Ordering)
    ensures r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&a@,&b@),
{ a.cmp(b) }
}
'''
    s=source('typed_eq')
    observer='''impl BorTag {pub closed spec fn __verus_infer_nonzero_u64_view(&self)->u64 {self.0@}}\n'''
    s=s.replace('verus! {\n','verus! {\n'+observer,1)
    if case=='typed':return s
    # Exercise the actual macro path on the genuine, unchanged declaration.
    a=s.index('impl BorTag {pub open spec fn __verus_infer_derived_eq')
    b=s.index('fn compare_equal',a)
    s=s[:a]+s[b:]
    ia=s.index('proof fn identity');ib=s.index('fn compare_equal',ia);s=s[:ia]+s[ib:]
    s=s.replace(observer,'')
    s=s.replace('#[verus_verify]','#[verus_verify(infer_derive(PartialEq))]')
    if case=='macro':return s
    if case=='false':return s.replace('ensures r == (tag_value(a) == tag_value(b)),','ensures r != (tag_value(a) == tag_value(b)),')
    if case=='width':return s.replace('{ a == b }','''{
    let r = a == b;
    proof {
        if tag_value(a) == u64::MAX && tag_value(b) == u64::MAX { assert(r); }
        if tag_value(a) == 9223372036854775808u64 && tag_value(b) == 1 { assert(!r); }
    }
    r
}''')
    if case=='unsupported':return s.replace('NonZero<u64>','NonZero<u32>')
    if case=='order':return s.replace('infer_derive(PartialEq)','infer_derive(PartialEq,PartialOrd,Ord)')
    if case=='wrong_model':return application('typed').replace('self.0@ == rhs.0@','self.0@ != rhs.0@')
    if case=='weak':return application('typed').replace('self.0@}}','0}}')
    raise ValueError(case)

def main():
    p=argparse.ArgumentParser();p.add_argument('build',type=Path)
    p.add_argument('case',choices=['vstd','typed','macro','false','width','unsupported','order','weak','wrong_model','standard_order'])
    p.add_argument('--expect',choices=['probe','positive','semantic','frontend'],default='probe')
    p.add_argument('--off',action='store_true')
    p.add_argument('--bundle',type=Path)
    p.add_argument('--warm',action='store_true')
    p.add_argument('--export',action='store_true')
    a=p.parse_args();build_receipt=json.loads((a.build/'receipt.json').read_bytes());profile=Path(build_receipt.get('target',str(a.build/'target')))/'release'
    if a.bundle:
        br=json.loads((a.bundle/'receipt.json').read_bytes());require(br['success'] is True,'frozen bundle failure')
        for n,h in br['tools'].items():require(digest(a.bundle/n)==h,'frozen tool drift')
        profile=a.bundle
    out=Path(tempfile.mkdtemp(prefix='nonzero-owned-'+a.case+'-',dir='/tmp'));out.chmod(0o700)
    print(out,flush=True)
    rec=dict(schema=1,commands=[],case=a.case,publication=False,goal_complete=False,source_acceptance=False)
    with replay_receipt(out,rec):
        authenticate()
        require(json.loads((a.build/'receipt.json').read_bytes())['success'] is True,'compiler build not successful')
        if a.case!='vstd':require((profile/'vstd.vir').is_file(),'matching vstd absent')
        tools=[p for p in profile.iterdir() if p.is_file() and p.name in {
            'verus','rust_verify','libverus_builtin.rlib','libverus_builtin_macros.so',
            'libverus_state_machines_macros.so','vstd_build','vstd.vir','libvstd.rlib'}]
        inputs=tools+[a.build/'receipt.json',RUSTC,BUNDLE/'z3',HERE/'trial.py',Path(__file__)]
        names=['source/builtin_macros/src/infer_derive_ops.rs','source/vir/src/infer.rs',
               'source/vir/src/infer/wrapper_compare.rs','source/vir/src/infer/wrapper_binary.rs',
               'source/vir/src/infer/newtypes.rs','source/vir/src/infer/registry.rs',
               'source/rust_verify/src/automatic_derive.rs','source/rust_verify/src/external.rs']
        inputs += [ROOT/n for n in names]
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',
                 LD_LIBRARY_PATH=str(RUSTC.parent.parent/'lib'),VERUS_IN_VARGO='1',
                 VERUS_Z3_PATH=str(BUNDLE/'z3'),VERUS_INFER_CACHE=str(out/'cache'),TMPDIR=str(out),
                 XDG_CACHE_HOME=str(out/'cache'),CARGO_BUILD_JOBS='2')
        if a.case=='vstd':
            # Normal verifying build: inherited vstd admissions are not banned.
            argv=[str(profile/'vstd_build'),str(profile),'--release','--verbose']
            inputs=[x for x in inputs if x.name not in ['vstd.vir','libvstd.rlib']]
            inputs += [p for p in (ROOT/'source/vstd').rglob('*') if p.is_file()]
            cwd=ROOT/'source'
        else:
            src=out/'component.rs';src.write_text(application(a.case));inputs.append(src)
            argv=[str(profile/'verus'),str(src),'--crate-type=lib','--no-cheating','--output-json',
                  '--multiple-errors','20','--log-all','--log-dir',str(out/'vir')]
            if not a.off:argv+=['--infer-goals']
            cwd=out
        if a.export:argv+=['--export',str(out/'component.vir'),'--compile','-o',str(out/'libcomponent.rlib')]
        if a.bundle:inputs.append(a.bundle/'receipt.json')
        guard=ArtifactGuard(inputs)
        (out/'input-freeze.json').write_text(json.dumps(dict(inputs=guard.initial,argv=argv,
              environment={k:env[k] for k in ['RUSTUP_TOOLCHAIN','LD_LIBRARY_PATH','VERUS_IN_VARGO','VERUS_Z3_PATH','VERUS_INFER_CACHE','TMPDIR','XDG_CACHE_HOME','CARGO_BUILD_JOBS']},before_first_trial=True,expected=a.expect),indent=2)+'\n')
        fg=ArtifactGuard([out/'input-freeze.json']);rec['freeze_sha256']=digest(out/'input-freeze.json')
        run=LoggedCommands(out,cwd,rec['commands'],timeout=1800 if a.case=='vstd' else 120,guard=guard)
        try:
            rc=run('check',argv,env);rec['terminal_exit']=rc
            if a.warm:
                require(rc==0,'cold run failed before warm')
                rec['cold_cache']={str(p.relative_to(out)):digest(p) for p in (out/'cache').rglob('*') if p.is_file()}
                rc=run('warm',argv,env);rec['warm_terminal_exit']=rc
            label='warm' if a.warm else 'check'
            err=(out/(label+'.stderr')).read_text()
            require(not any(t in err.lower() for t in ['panicked at','timed out','resource limit','internal compiler error']),'resource/compiler failure')
            if a.case=='vstd':
                require(rc==0,'matching normal vstd failed')
                rec['matching_vstd_products']={str(profile/n):digest(profile/n) for n in ['vstd.vir','libvstd.rlib']}
            else:
                data=json.loads((out/(label+'.stdout')).read_text());vr=data['verification-results'];inf=data['verus-infer']
                rec['verification']=vr;rec['inference']=inf
                require(inf['interface_written'] is a.export,'interface/export status differs')
                if a.expect in ['positive','semantic']:
                    require(vr.get('is-verifying-entire-crate') is True,'partial checking')
                    require(inf['functions'],'empty helper list cannot certify derive')
                    require_exact(inf['functions'])
                    require([f['function'] for f in inf['functions']]==['component::BorTag::eq'],'helper identity differs')
                    native=inf['verification_scope']['native_functions']
                    bodies=[b for b in native if b['selected'] and b['body_status'] not in ['specification_definition_or_declaration','selected_item_without_translated_body']]
                    expected={'<BorTag as std::clone::Clone>::clone','<BorTag as std::cmp::PartialEq>::eq','compare_equal'}
                    if a.case=='typed':expected.add('identity')
                    require({b['rust_function'] for b in bodies}==expected and len(bodies)==len(expected),'native body set differs')
                    if a.warm:require(all(f['metrics']['cache']=='hit' for f in inf['functions']),'not warm candidate reuse')
                    if a.expect=='positive':
                        require(rc==0 and vr['success'] is True and vr['errors']==0,'ordinary proof failed')
                        require(inf['verification_scope']['complete_selected_scope_verified'] is True,'unchecked native body')
                        rec['source_acceptance']=True
                    else:
                        require(rc!=0 and vr['errors']==1 and 'postcondition not satisfied' in err,'not one semantic failure')
                        require(not any(t in err for t in ['precondition not satisfied','error[E','possible arithmetic']),'unhealthy control')
                        require(inf['failed_functions']==['component::compare_equal'],'wrong failed control')
                        require(all(b['body_status']=='selected_body_not_certified_by_successful_full_run' for b in bodies),'unexpected failed-crate scope state')
                elif a.expect=='frontend':require(rc!=0 and vr.get('encountered-vir-error') is True,'not frontend')
                print(json.dumps(dict(exit=rc,verification=vr,functions=len(inf['functions']))),flush=True)
        finally:
            guard.check();fg.check();rec['final_guards_passed']=True
            rec['products']={str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name!='receipt.json'}
if __name__=='__main__':main()
