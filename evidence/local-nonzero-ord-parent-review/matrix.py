"""New local-source controls; no source change or test is a proof substitute."""
from trials import *

def expect(label,source,*,ok,verified=None,errors=None,diagnostic=None,**kw):
 r=app(label,source,**kw)
 need((r['exit']==0)==ok,'unexpected exit '+label+': '+(R/label/'stderr').read_text()[-3000:])
 if verified is not None:need(r.get('verification',{}).get('verified')==verified,'verified inventory '+label)
 if errors is not None:need(r.get('verification',{}).get('errors')==errors,'error inventory '+label)
 if diagnostic:need(diagnostic in (R/label/'stderr').read_text(),'missing diagnostic '+label)
 return r

if __name__=='__main__':
 mode=sys.argv[1];epoch=os.environ.get('LOCAL_NONZERO_ORD_BUNDLE','bundle')
 if mode=='local':
  expect(epoch+'-cold',DIRECT,ok=True,verified=2,errors=0)
  expect(epoch+'-warm',DIRECT,ok=True,verified=2,errors=0)
  expect(epoch+'-off',DIRECT,ok=False,verified=0,errors=1,goals=False)
  expect(epoch+'-partial',base.source('carrier_partial'),ok=True,verified=5,errors=0)
  expect(epoch+'-swapped',DIRECT.replace('(&a@,&b@)','(&b@,&a@)'),ok=False,verified=1,errors=1)
  expect(epoch+'-false',DIRECT.replace('r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&a@,&b@)','r == Ordering::Less'),ok=False,verified=1,errors=1)
  expect(epoch+'-weak',DIRECT.replace('r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&a@,&b@)','true'),ok=True,verified=2,errors=0)
  alias=DIRECT.replace('pub fn total(a:&NonZero<u64>,b:&NonZero<u64>)','pub fn total(a:&NonZero<u64>)').replace('(&a@,&b@)','(&a@,&a@)').replace('a.cmp(b)','a.cmp(a)')
  expect(epoch+'-alias',alias,ok=True,verified=2,errors=0)
 elif mode=='exports':
  boolfn='verus! {pub fn bool_word(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}'
  for suffix,src,options in [('direct',DIRECT,()),('mixed',DIRECT+boolfn,()),('false',DIRECT.replace('(&a@,&b@)','(&b@,&a@)'),()),('private',DIRECT.replace('pub fn total','fn total')+'verus! {pub fn unrelated()->(r:u64) ensures r==9u64, {9}}',()),*[(flag,DIRECT,(flag,)) for flag in ['--no-verify','--no-lifetime','--no-trait-conflicts','--no-erasure-check']]]:
   for cargo in [False,True]:
    label=epoch+'-export-'+suffix+('-cargo' if cargo else '-explicit');d=R/label;d.mkdir(exist_ok=True)
    selected=d/('liblocal_order.vir' if cargo else 'local_order.vir');selected.write_bytes(b'selected stale output')
    expect(label,src,ok=False,options=options,export=True,cargo=cargo)
    need(not selected.exists(),'selected stale output survived '+label)

 elif mode=='bool':
  text='use vstd::prelude::*; verus! {pub fn all_bool(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}'
  expect(epoch+'-bool-provider',text,ok=True,verified=2,errors=0,export=True,name='provider')
  deps=[('provider',R/(epoch+'-bool-provider'))]
  for label,condition,n,err,ok in [('ordinary','r==if b {1u64}else{0u64}',1,0,True),('false','r==0u64',0,1,False)]:
   src='use vstd::prelude::*; verus! {pub fn observe(b:bool)->(r:u64) ensures '+condition+', {provider::all_bool(b)}}'
   expect(epoch+'-bool-'+label,src,ok=ok,verified=n,errors=err,goals=False,name='bool_'+label,deps=deps)
 elif mode=='boundaries':
  # Observations support regression only. Each ordinary symbolic property uses
  # the SAME source std Ord call. The full-domain theorem is the direct case.
  src=base.PRE+'verus! {\n'
  for index,(a,b) in enumerate([(1,1),(1,2**61),(2**61,2**63),(2**63,2**64-2),(2**64-2,2**64-1)]):
   for order,(x,y) in enumerate([(a,b),(b,a)]):
    src+=f'pub fn observation_{index}_{order}(a:&NonZero<u64>,b:&NonZero<u64>)->(r:Ordering) requires a@ == {x}u64, b@ == {y}u64, ensures r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&a@,&b@), {{a.cmp(b)}}\n'
  src+='}\n'
  expect(epoch+'-observations',src,ok=True,verified=11,errors=0)

 elif mode=='exact':
  for label in ['exact-cold','exact-warm']:
   rec=expect(epoch+'-'+label,DIRECT,ok=True,verified=2,errors=0,paths_label=epoch+'-same-invocation')
   shutil.copytree(R/(epoch+'-same-invocation')/'vir',R/(epoch+'-'+label)/'retained-vir')
  a=json.loads((R/(epoch+'-exact-cold')/'freeze.json').read_text())
  b=json.loads((R/(epoch+'-exact-warm')/'freeze.json').read_text())
  need(a==b,'cold/warm invocation/source/tool/environment mismatch')
 elif mode=='global-gates':
  text='use vstd::prelude::*; verus! {pub fn all_bool(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}'
  aux=text.replace('{u64::from(b)}','{let x=u64::from(b); assert((0u64 | (1u64 << 1u64)) == 0u64) by(bit_vector); x}')
  erased=text+'\n#[cfg(not(verus_keep_ghost_body))] fn erased_type_error()->u64 { () }\n'
  for label,src in [('auxiliary',aux),('erased',erased)]:
   for cargo in [False,True]:
    name=epoch+'-global-'+label+('-cargo' if cargo else '-explicit');d=R/name;d.mkdir(exist_ok=True)
    selected=d/('liblocal_order.vir' if cargo else 'local_order.vir');selected.write_bytes(b'stale selected output')
    r=expect(name,src,ok=False,export=True,cargo=cargo)
    need(not selected.exists(),'global failure left selected output')
    j=json.loads((d/'stdout').read_text());v=j['verification-results']
    if label=='erased':need(v.get('verified')==2 and 'E0308' in (d/'stderr').read_text(),'late erased failure classification')
    else:need(j['verus-infer'].get('failed_functions')==[] and v['errors']==1,'isolated auxiliary classification')
 elif mode=='relations-and-filters':
  src=base.PRE+'''verus! {
pub fn relation(a:&NonZero<u64>,b:&NonZero<u64>)->(r:Ordering)
 ensures (r==Ordering::Less) == (a@ < b@),
         (r==Ordering::Equal) == (a@ == b@),
         (r==Ordering::Greater) == (a@ > b@),
 {a.cmp(b)}
}'''
  expect(epoch+'-three-relations',src,ok=True,verified=2,errors=0)
  for flag,args in [('function',['--verify-function','total']),('module',['--verify-module','root'])]:
   for cargo in [False,True]:
    label=epoch+'-filter-'+flag+('-cargo' if cargo else '-explicit');d=R/label;d.mkdir(exist_ok=True)
    selected=d/('liblocal_order.vir' if cargo else 'local_order.vir');selected.write_bytes(b'stale output')
    expect(label,DIRECT,ok=False,options=args,export=True,cargo=cargo)
    need(not selected.exists(),'partial selected interface survived')
 elif mode=='filters-v2':
  for flag,args in [('function',['--verify-root','--verify-function','total']),('module',['--verify-module','sub'])]:
   src=DIRECT if flag=='function' else DIRECT.replace('verus! {','mod sub { use super::*;verus! {')+'}\n'
   for cargo in [False,True]:
    label=epoch+'-filter-v2-'+flag+('-cargo' if cargo else '-explicit');d=R/label;d.mkdir(exist_ok=True)
    selected=d/('liblocal_order.vir' if cargo else 'local_order.vir');selected.write_bytes(b'stale output')
    expect(label,src,ok=False,options=args,export=True,cargo=cargo)
    need(not selected.exists(),'partial selected interface survived')
 elif mode=='erasure-v2':
  for cargo in [False,True]:
   label=epoch+'-export-erasure-v2'+('-cargo' if cargo else '-explicit');d=R/label;d.mkdir(exist_ok=True)
   selected=d/('liblocal_order.vir' if cargo else 'local_order.vir');selected.write_bytes(b'stale output')
   expect(label,DIRECT,ok=False,options=['--no-erasure-check'],export=True,cargo=cargo,diagnostic='complete ordinary')
   need(not selected.exists(),'no-erasure selected interface survived')
 elif mode=='unsupported':
  value=DIRECT.replace('{ a.cmp(b) }','{ let f = <NonZero<u64> as Ord>::cmp; f(a,b) }')
  expect(epoch+'-function-value',value,ok=False,verified=0,errors=1)
  generic='''#![feature(nonzero_internals)]
use vstd::prelude::*; use std::num::NonZero;
verus! {pub fn generic<T:core::num::ZeroablePrimitive + Ord>(a:&NonZero<T>,b:&NonZero<T>)->core::cmp::Ordering {a.cmp(b)}}
'''
  r=app(epoch+'-generic',generic)
  # Generic frontend support is not required; it must never create this concrete
  # helper or earn the concrete numeric result, even if a weak property checks.
  d=R/(epoch+'-generic');text=(d/'stderr').read_text()
  need('checked_nonzero_u64_ord' not in (d/'stdout').read_text(),'generic concrete helper registered')
  for p in (d/'vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'generic helper in VIR')
 elif mode=='unsupported-v2':
  cases={
   'generic':'''#![feature(nonzero_internals)]
use vstd::prelude::*; use std::num::NonZero;
verus! {pub fn generic<T:core::num::ZeroablePrimitive + Ord>(a:&NonZero<T>,b:&NonZero<T>)->core::cmp::Ordering {a.cmp(b)}}
''',
   'lookalike':'''use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Ord)] pub struct NonZero<T>(T);
verus! {pub fn lookalike(a:&NonZero<u64>,b:&NonZero<u64>)->core::cmp::Ordering {a.cmp(b)}}
''',
   'overloaded-deref':'''use vstd::prelude::*; use std::num::NonZero;
#[verus_verify] pub struct Holder(NonZero<u64>);
#[verus_verify] impl core::ops::Deref for Holder { type Target=NonZero<u64>; fn deref(&self)->&NonZero<u64>{&self.0} }
verus! {pub fn indirect(a:&Holder,b:&NonZero<u64>)->core::cmp::Ordering {a.cmp(b)}}
'''}
  for name,src in cases.items():
   label=epoch+'-unsupported-'+name;app(label,src);d=R/label
   need('checked_nonzero_u64_ord%' not in (d/'stdout').read_text(),'unsupported concrete helper registered')
   for p in (d/'vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'unsupported helper in VIR')
