import review as t
from controls import raw,wrap
cases={
 'generic-caller-env':('pub fn call<T>(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {u64::from(b)}',True),
 'aliased-trait':('use core::convert::From as Conversion; pub fn call(b:bool)->(r:u64) ensures r==if b{1u64}else{0u64}, {<u64 as Conversion<bool>>::from(b)}',True),
 'overlapping-borrow':('pub fn call(mut b:bool)->u64 {let r=&mut b;let s=&mut b;let x=*r;let y=*s;u64::from(x&&y)}',False),
}
for n,(s,ok) in cases.items():
 r=raw('semantic-'+n,wrap(s));t.need((r['exit']==0)==ok,'semantic '+n)
 if n=='overlapping-borrow':t.need('cannot borrow' in (t.R/('semantic-'+n)/'stderr').read_text(),'lifetime error required')
d=t.R/'extra-private-unused-export';d.mkdir();p=d/'selected.vir';p.write_bytes((t.R/'ordinary.vir').read_bytes())
s=wrap('fn hidden(b:bool)->u64{u64::from(b)} pub fn call()->u64{0}')
r=raw('extra-private-unused-export',s,export=p);t.need(r['exit']!=0 and not p.exists() and 'local-only' in (d/'stderr').read_text(),'private pruning bypass')
