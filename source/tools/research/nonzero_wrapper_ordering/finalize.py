"""Final guarded runtime, separate append-only receipt namespace in the OWN build root."""
import sys
from pathlib import Path
import resume as x
from resume import need,sha,write,ROOT,TC,b
BUILD_ROOT=x.R
N=BUILD_ROOT/'sealed-validation'
N.mkdir(exist_ok=True)
x.R=N
import controls as c
import checks as checks
E=x.EPOCH;B=x.B
need(E=='sealed','use WRAPPER_EPOCH=sealed')

def freeze():
    need((BUILD_ROOT/'vstd-sealed/receipt.json').exists(),'normal matching vstd first')
    x.freeze();write(N/'runtime-origin.json',dict(build_root=str(BUILD_ROOT),tool_freeze_sha256=sha(BUILD_ROOT/'sealed-eight-tools.json'),vstd_receipt_sha256=sha(BUILD_ROOT/'vstd-sealed/receipt.json')))
def proposals():
    for label,expr in [('correct','self.0@'),('wrong_view','0u64'),('weak_view','self.0@ % 2'),('swapped_view','((u64::MAX as int-self.0@ as int) as u64)')]:
        src=x.fixtures.source('authored')+'verus!{impl BorTag{pub closed spec fn __verus_infer_nonzero_u64_view(&self)->u64{'+expr+'}}}\n'+c.CAND[len(c.BASE):]
        j=c.expect('proposal-'+label,src,ok=label=='correct')
        if label!='correct':need(any(f['proofs']['soundness']['status']=='not_proved' and f['proofs']['exactness']['status']=='not_proved' for f in j['inference']['functions']),'actual failed proposal obligations; dependent helpers are not credited')
def partial_barrier():
    src='''use vstd::prelude::*;use std::num::NonZero;
#[verus_verify(infer_derive(PartialEq,PartialOrd))]
#[derive(Copy,Clone,Hash,PartialEq,Eq,PartialOrd)] pub struct BorTag(NonZero<u64>);
'''
    c.expect('partial-only-local',src,n=6)
    prior=(N/'final-bool-prior/wrapper.vir').read_bytes();rows={}
    for goals in [False,True]:
        for route in ['explicit','cargo','both','equal']:
            label='partial-request-'+('on' if goals else 'off')+'-'+route;d=N/label;d.mkdir(exist_ok=True);cargo=d/'libwrapper.vir';explicit=cargo if route=='equal' else d/'selected.vir';selected=[explicit] if route=='explicit' else [cargo] if route=='cargo' else list(dict.fromkeys([explicit,cargo]))
            for p in selected:p.write_bytes(prior)
            c.expect(label,src,ok=False,goals=goals,cargo=route!='explicit',export=explicit if route!='cargo' else None)
            need('local-only' in (d/'stderr').read_text(),'request-level barrier');need(all(not p.exists() for p in selected),'all partial-only selected paths removed')
            rows[label]=dict(prior_sha256=x.hashlib.sha256(prior).hexdigest(),removed=list(map(str,selected)))
    write(N/'partial-only-invalidation.json',rows)
def test_build():checks.compile_tests()
def test_run():
    artifacts={}
    for label in ['final-test-build-v4','final-unit-build-v2']:
        for line in (N/label/'stdout').read_text().splitlines():
            try:j=x.json.loads(line)
            except ValueError:continue
            if j.get('reason')=='compiler-artifact' and j.get('executable') and j['profile']['test']:
                name=j['target']['name']+('-unit' if j['target']['name']=='rust_verify' and 'lib' in j['target']['kind'] else '');artifacts[name]=j['executable']
    for p in [BUILD_ROOT/'target-verus/debug',ROOT/'source/target-verus/release']:
        p.parent.mkdir(parents=True,exist_ok=True)
        if p.is_symlink():need(str(p.resolve()).startswith(str(BUILD_ROOT)+'/'),'own alias only');p.unlink()
        need(not p.exists(),'unexpected harness directory');p.symlink_to(B,target_is_directory=True)
    z=ROOT/'source/z3'
    if z.is_symlink():need(sha(z)==sha(B/'z3'),'pinned own solver alias');z.unlink()
    need(not z.exists(),'own solver path');z.symlink_to(B/'z3')
    for p in [BUILD_ROOT/'target/debug',BUILD_ROOT/'build/debug']:
        p.mkdir(parents=True,exist_ok=True)
        for n in b.TOOLS:
            q=p/n
            if q.exists() and sha(q)==sha(B/n):continue
            if q.exists():q.unlink()
            x.shutil.copy2(B/n,q)
    for name in ['vir','rust_verify-unit',*checks.SUITES]:
        j=x.run('final-test-'+name,[artifacts[name],'--test-threads=1'],dependencies=[Path(artifacts[name])]);need(j['exit']==0,'focused suite '+name+': '+(N/('final-test-'+name)/'stdout').read_text()[-5000:])
if __name__=='__main__':
    mode=sys.argv[1]
    if mode=='freeze':freeze()
    else:
        need((N/'sealed-eight-tools.json').exists(),'eight tools frozen before proofs')
        {'semantics':c.semantics,'exports':c.exports,'compatibility':c.compatibility,'shapes':c.shapes,'proposals':proposals,'partial-barrier':partial_barrier,'test-build':test_build,'test-run':test_run}[mode]()
