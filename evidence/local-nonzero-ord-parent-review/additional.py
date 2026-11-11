from matrix import *
mode=sys.argv[1]
if mode=='relation':
 src=base.PRE+'''verus! {
pub fn relation(a:&NonZero<u64>,b:&NonZero<u64>)->(r:Ordering)
 ensures (r==Ordering::Less) == (a@ < b@),
         (r==Ordering::Equal) == (a@ == b@),
         (r==Ordering::Greater) == (a@ > b@),
 {a.cmp(b)}
}'''
 expect('review-three-relations',src,ok=True,verified=2,errors=0)
elif mode=='old-policy':
 srcdir=R/'bundle-bool-provider';dest=R/'old-policy-input';dest.mkdir()
 shutil.copy2(srcdir/'libprovider.rlib',dest/'libprovider.rlib')
 raw=(srcdir/'provider.vir').read_bytes()
 need(raw.count(b'candidate-bound-evidence-v73')==1,'one current policy')
 (dest/'provider.vir').write_bytes(raw.replace(b'candidate-bound-evidence-v73',b'candidate-bound-evidence-v72'))
 src='use vstd::prelude::*; verus! {pub fn observe(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {provider::all_bool(b)}}'
 expect('review-old-policy-import',src,ok=False,goals=False,name='stale_consumer',deps=[('provider',dest)])
elif mode=='dual-output':
 label='review-dual-selected-outputs';d=R/label;d.mkdir()
 cargo=d/'liblocal_order.vir';explicit=d/'explicit.vir'
 for p in [cargo,explicit]:p.write_bytes(b'selected stale sentinel')
 rec=app(label,DIRECT,cargo=True,options=['--export',str(explicit)])
 need(rec['exit']!=0,'local ordering export was accepted')
 err=(d/'stderr').read_text();need('local-only' in err,'not selected-output barrier')
 write(d/'output-state.json',dict(cargo_exists=cargo.exists(),explicit_exists=explicit.exists(),diagnostic='local-only',classification='ordinary dual selected output path control; sentinel existence only'))
 print((d/'output-state.json').read_text())
elif mode=='function-value':
 src=DIRECT.replace('{ a.cmp(b) }','{ let f = <NonZero<u64> as Ord>::cmp; f(a,b) }')
 rec=app('review-function-value',src)
 need(rec['exit']!=0 and rec['verification']['encountered-vir-error'],'frontend unsupported classification')
 for p in (R/'review-function-value/vir').glob('*.vir'):
  need('checked_nonzero_u64_ord%' not in p.read_text(),'function value concrete helper')
