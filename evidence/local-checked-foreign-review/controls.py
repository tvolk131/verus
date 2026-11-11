"""Bounded independent publication, call/argument and arithmetic controls."""
import hashlib,json,sys
from pathlib import Path
import review as t
from proofs import source
R=t.R;B=R/'bundle'
def raw(label,text,options=(),goals=True,export=None,cargo=False):
 d=R/label;d.mkdir(exist_ok=True);src=d/'component.rs';src.write_text(text)
 args=[B/'verus',src,'--crate-name=component','--crate-type=lib','--no-cheating','--output-json','--compile','--multiple-errors','20','-o',d/'libcomponent.rlib']
 extra={}
 if cargo:
  project=R/'cargo-project';project.mkdir(exist_ok=True)
  (project/'Cargo.toml').write_text('[package]\nname="review_bool"\nversion="0.0.0"\nedition="2021"\n')
  package='review_bool-0.0.0-'+hashlib.sha256(str(project/'Cargo.toml').encode()).hexdigest()[:12]
  extra={'__VERUS_DRIVER_VIA_CARGO__':'1','__VERUS_DRIVER_VERIFY_'+package:'1','CARGO_PKG_NAME':'review_bool','CARGO_PKG_VERSION':'0.0.0','CARGO_MANIFEST_DIR':str(project),'CARGO_PRIMARY_PACKAGE':'1'}
  args=[B/'rust_verify',t.TC/'bin/rustc',src,'--crate-name','component','--crate-type=lib','--no-cheating','--output-json','--emit=metadata','--out-dir',d,'--VIA-CARGO','compile-when-primary-package','--extern','verus_builtin='+str(B/'libverus_builtin.rlib'),'--extern','verus_builtin_macros='+str(B/'libverus_builtin_macros.so'),'--extern','vstd='+str(B/'libvstd.rlib'),'--import','vstd='+str(B/'vstd.vir'),'-L','dependency='+str(B)]
 if '--no-erasure-check' in options and '--compile' in args:args.remove('--compile')
 if goals:args+=['--infer-goals']
 args+=list(options)
 if export:args+=['--export',export]
 return t.run(label,args,extra)
def wrap(s):return 'use vstd::prelude::*;\nverus! {\n'+s+'\n}\n'
def exports():
 seed=wrap('pub fn seed()->u64 {0}')
 old=R/'ordinary.vir';r=raw('ordinary-export',seed,goals=False,export=old);t.need(r['exit']==0 and old.exists(),'ordinary export');saved=old.read_bytes()
 p=R/'ordinary-infer.vir';r=raw('ordinary-infer-export',seed,export=p);t.need(r['exit']==0 and p.exists(),'infer-aware ordinary export')
 cases=[('complete',(),source('actual')),('false',(),source('false')),*[(n,opts,source('actual')) for n,opts in [('no-verify',('--no-verify',)),('no-lifetime',('--no-lifetime',)),('no-erasure',('--no-erasure-check',)),('no-traits',('--no-trait-conflicts',)),('filter-function',('--verify-root','--verify-function','all_bool')),('filter-module',('--verify-module','root')),('filter-only-module',('--verify-only-module','root')),('filter-root',('--verify-root',))]],('type-error',(),source('actual')+'fn bad()->u64{()}'),('parse-error',(),source('actual')+'fn broken( {')]
 rows=[]
 for cargo in [False,True]:
  for n,opts,text in cases:
   label=('cargo-' if cargo else 'explicit-')+n;d=R/label;d.mkdir(exist_ok=True);p=d/('libcomponent.vir' if cargo else 'selected.vir');p.write_bytes(saved)
   r=raw(label,text,opts,export=None if cargo else p,cargo=cargo)
   t.need(r['exit']!=0 and not p.exists(),'export/refusal '+label)
   if n not in ['type-error','parse-error']:t.need('local-only' in (d/'stderr').read_text(),'wrong refusal '+label)
   rows.append(dict(label=label,removed=True))
 for n,opts,text in cases[2:10]:
  label='local-'+n;r=raw(label,text,opts);t.need(r['exit']!=0 and 'complete source/body' in (R/label/'stderr').read_text(),'partial check accepted '+n)
 t.write(R/'exports.json',dict(legacy_ordinary_sha256=hashlib.sha256(saved).hexdigest(),controls=rows))
def semantic():
 cases={
 'qualified-typed':('pub fn call(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {<u64 as core::convert::From<bool>>::from(b)}',True),
 'trait-path':('pub fn call(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {core::convert::From::from(b)}',True),
 'argument-once':('pub fn call(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {let mut n=0u64; let v=u64::from({n=n+1;b}); assert(n==1u64); v}',True),
 'argument-bad-assert':('pub fn call(b:bool)->u64 {let mut n=0u64; let v=u64::from({n=n+1;b}); assert(n==0u64); v}',False),
 'unused-private':('fn unused(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {u64::from(b)} pub fn call()->u64 {0}',True),
 'nested-scope':('pub fn call(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {if b {u64::from(b)} else {let c=b;u64::from(c)}}',True),
 'false-body':('pub fn call(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {let v=u64::from(b);1u64-v}',False),
 'caller-precondition':('fn needs(b:bool)->(r:bool) requires b, ensures r==b, {b} pub fn call(b:bool)->u64 {u64::from(needs(b))}',False),
 'other-output':('pub fn call(b:bool)->u32 {u32::from(b)}',None),
 'other-input':('pub fn call(b:u32)->(r:u64) ensures r==b as u64, {u64::from(b)}',None),
 'generic':('pub fn call<T:From<bool>>(b:bool)->T {T::from(b)}',None),
 'custom-trait':('pub trait From<T>:Sized {fn from(b:T)->Self;} impl From<bool> for u64 {fn from(b:bool)->Self {2}} pub fn call(b:bool)->u64 {<u64 as From<bool>>::from(b)}',None),
 'fndef-value':('pub fn call(b:bool)->u64 {let f=u64::from;f(b)}',None),
 'fn-coercion':('pub fn call(b:bool)->u64 {let f:fn(bool)->u64=u64::from;f(b)}',None),
 }
 for name,(body,success) in cases.items():
  a=raw('semantic-'+name,wrap(body));j=json.loads((R/('semantic-'+name)/'stdout').read_text())
  if success is not None:t.need((a['exit']==0)==success,'unexpected semantic '+name)
  else:
   t.need(not any('checked_foreign' in f for f in j.get('func-details',{})),'new helper '+name)
   b=raw('semantic-'+name+'-off',wrap(body),goals=False)
   t.need(a.get('verification')==b.get('verification'),'inherited behavior changed '+name)
 for w in [8,16,32,64]:
  k=w-1;limit=1<<k
  base=f'pub fn pack(low:u{w},b:bool)->(r:bool) REQUIRES ensures r==b, {{let marker=u64::from(b);let high=if b {{1u{w}}}else{{0u{w}}};let word=low | (high<<{k}u64); (word & {limit}u{w})>0}}'
  for bounded in [True,False]:
   n=f'packing-{w}-'+('bounded' if bounded else 'unbounded');body=base.replace('REQUIRES',f'requires low < {limit}u{w},' if bounded else '')
   r=raw(n,wrap(body));t.need((r['exit']==0)==bounded,'packing domain '+n)
 # Shift boundary 0 and >=width and signed operations: no new arithmetic law.
 for n,body in {
  'shift-zero':'pub fn call(b:bool,x:u64)->(r:u64) ensures r==x, {let z=u64::from(b); x<<0u64}',
  'shift-overflow':'pub fn call(b:bool,x:u64)->u64 {let z=u64::from(b);x<<64u64}',
  'signed-false':'pub fn call(b:bool,x:i64)->(r:i64) ensures r>=0, {let z=u64::from(b);x<<1u64}',
  'constant-nested':'const MASK:u64=((1u64<<63u64)^0xffffu64)>>1u64; pub fn call(b:bool,x:u64)->(r:bool) ensures r, {let z=u64::from(b);let y=x<<2u64;MASK==0x4000000000007fffu64}',
 }.items():
  r=raw('semantic-'+n,wrap(body));t.need((r['exit']==0)==(n in ['constant-nested']),'shift/constant control '+n)
if __name__=='__main__':
 if sys.argv[1]=='exports':exports()
 if sys.argv[1]=='semantic':semantic()
