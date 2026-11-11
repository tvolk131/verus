from controls import *
from extra import partial_barrier

def run_extra():
 src=CAND+"""verus!{pub fn alias_parameters(a:&BorTag,b:&BorTag){
 let aa=&*a;let bb=&*b;let c=aa.cmp(bb);let p=aa.partial_cmp(bb);
 assert(c==vstd::std_specs::cmp::OrdSpec::cmp_spec(&value(a),&value(b)));
 assert(p==Some(c));let e=aa.cmp(a);assert(e==Ordering::Equal);
}}"""
 expect('independent-alias-arguments',src)
 # Closed view cannot be revealed to an ordinary external client; the public contract suffices.
 src='use vstd::prelude::*;use std::num::NonZero;\n#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]\n#[derive(Copy,Clone,Hash,PartialEq,Eq,PartialOrd,Ord)] struct BorTag(NonZero<u64>);\nverus!{pub fn unrelated()->u64{7}}'
 prior=(R/'final-bool-prior/wrapper.vir').read_bytes();rows={}
 for goals in [False,True]:
  for route in ['explicit','cargo','both','equal']:
   label='independent-unused-'+str(goals)+'-'+route;d=R/label;d.mkdir();c=d/'libwrapper.vir';a=c if route=='equal' else d/'selected.vir';selected=[a] if route=='explicit' else [c] if route=='cargo' else list(dict.fromkeys([a,c]))
   for p in selected:p.write_bytes(prior)
   expect(label,src,ok=False,goals=goals,cargo=route!='explicit',export=a if route!='cargo' else None)
   need('local-only' in (d/'stderr').read_text() and all(not p.exists() for p in selected),'unused request refuses all selected')
   rows[label]=dict(prior_sha256=hashlib.sha256(prior).hexdigest(),removed=list(map(str,selected)))
 write(R/'independent-unused-invalidation.json',rows)
if __name__=='__main__':run_extra()
