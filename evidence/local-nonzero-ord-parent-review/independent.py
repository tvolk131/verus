"""Independent ordinary source controls. No model or implementation patch."""
from matrix import *

def no_helper(label):
 d=R/label
 need('checked_nonzero_u64_ord%' not in (d/'stdout').read_text(),'unexpected concrete helper')
 for p in (d/'vir').glob('*.vir'):
  need('checked_nonzero_u64_ord%' not in p.read_text(),'unexpected helper in VIR')

def cases():
 relation='r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&a@,&b@)'
 # Concrete calls in unrelated generic caller contexts share only the closed
 # concrete helper; no caller binder may enter its signature or contract.
 src=base.PRE+'''type Alias = NonZero<u64>;
verus! {
pub fn one<T>(a:&Alias,b:&Alias,t:T)->(r:Ordering) ensures REL, { a.cmp(b) }
pub fn two<U:Copy>(a:&Alias,b:&Alias,u:U)->(r:Ordering) ensures REL, { <Alias as Ord>::cmp(a,b) }
pub fn reversed(a:&Alias,b:&Alias)->(r:Ordering)
 ensures r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&b@,&a@), { b.cmp(a) }
pub fn twice(a:&Alias,b:&Alias)->(r:Ordering) ensures REL,
 { let x = a.cmp(a); a.cmp(b) }
pub fn nested<T:Ord>(a:&Alias,b:&Alias,x:&T,y:&T)->(r:Ordering) ensures REL,
 { let z = x.cmp(y); a.cmp(b) }
}'''.replace('REL',relation)
 expect('review-contexts',src,ok=True,verified=6,errors=0)
 # Both argument expressions perform effects, observed in the original caller.
 effects=base.PRE+'''verus! {
pub fn effect_args(a:&NonZero<u64>,b:&NonZero<u64>)->(r:Ordering)
 ensures REL,
 {
   let mut n:u64 = 0;
   let r = <NonZero<u64> as Ord>::cmp({ n = n+1; a }, { n = n+1; b });
   assert(n == 2);
   r
 }
}'''.replace('REL',relation)
 expect('review-effects',effects,ok=True,verified=2,errors=0)
 expect('review-effects-false',effects.replace('assert(n == 2)','assert(n == 1)'),ok=False,verified=1,errors=1)
 # Explicit get/get/cmp mirror is diagnostic ordinary frontend comparison only.
 mirror=DIRECT.replace('{ a.cmp(b) }','{ let na = *a; let x = na.get(); let xr = &x; let nb = *b; let y = nb.get(); let yr = &y; <u64 as Ord>::cmp(xr,yr) }')
 expect('review-mirror',mirror,ok=True,verified=1,errors=0,goals=False)
 no_helper('review-mirror')
 # Local-only auxiliary failure: helper must succeed while the BV query fails.
 aux=DIRECT.replace('{ a.cmp(b) }','{ let r=a.cmp(b); assert((0u64 | (1u64 << 1u64)) == 0u64) by(bit_vector); r }')
 rec=expect('review-ord-auxiliary',aux,ok=False,errors=1)
 data=json.loads((R/'review-ord-auxiliary/stdout').read_text())
 need(data['verus-infer']['failed_functions']==[],'not auxiliary-only')
 need(not data['verification-results']['encountered-vir-error'],'helper health/frontend failure')
 # Genuine wrapper derives excluded by existing profile: they must not activate
 # carrier redirect solely because their generated HIR contains core cmp.
 eq=base.source('eq')
 expect('review-ignored-derive-export',eq,ok=True,verified=6,errors=0,export=True)
 no_helper('review-ignored-derive-export')
 expect('review-wrapper-unproved',base.source('original_calls'),ok=False)
 no_helper('review-wrapper-unproved')

if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use Python -B')
 cases()
