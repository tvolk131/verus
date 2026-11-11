from controls import *
import controls as c
import resume as x
N=R
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
    if not (N/'partial-only-local/receipt.json').exists():c.expect('partial-only-local',src,n=6)
    prior=(N/'final-bool-prior/wrapper.vir').read_bytes();rows={}
    for goals in [False,True]:
        for route in ['explicit','cargo','both','equal']:
            label='partial-request-'+('on' if goals else 'off')+'-'+route;d=N/label;d.mkdir(exist_ok=True);cargo=d/'libwrapper.vir';explicit=cargo if route=='equal' else d/'selected.vir';selected=[explicit] if route=='explicit' else [cargo] if route=='cargo' else list(dict.fromkeys([explicit,cargo]))
            if not (d/'receipt.json').exists():
                for p in selected:p.write_bytes(prior)
                c.expect(label,src,ok=False,goals=goals,cargo=route!='explicit',export=explicit if route!='cargo' else None)
            else:need(json.loads((d/'receipt.json').read_text())['exit']!=0,'retained partial refusal')
            need('local-only' in (d/'stderr').read_text(),'request-level barrier');need(all(not p.exists() for p in selected),'all partial-only selected paths removed')
            rows[label]=dict(prior_sha256=x.hashlib.sha256(prior).hexdigest(),removed=list(map(str,selected)))
    write(N/'partial-only-invalidation.json',rows)

if __name__=="__main__":
 {"proposals":proposals,"partial-barrier":partial_barrier}[sys.argv[1]]()
