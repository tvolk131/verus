import check as c
for label,source in [('private_pattern','use vstd::prelude::*;verus!{mod inner{pub struct S {x:u64}} fn f(s:inner::S){let inner::S{x}=s;}}'),('private_contract','use vstd::prelude::*;verus!{mod inner{pub struct S{x:u64} pub open spec fn leak(s:S)->u64{s.x}}}'),('private_constructor','use vstd::prelude::*;verus!{mod inner{pub struct S {x:u64}} fn f(){let _=inner::S{x:0};}}')]:
 j,_=c.app(label,source);text=(c.R/'epochs'/label/'stderr').read_text();c.b.need(j['exit']!=0 and ('private' in text or 'non-visible' in text),'privacy '+label)
for label in ['item132','ordinary_borrow3']:
 d=c.R/'apps'/label
 args=[c.b.TC/'bin/rustc',d/'input.rs','--crate-type=lib','--crate-name','runtime_'+label,'--edition=2024','--extern','vstd='+str(c.B/'libvstd.rlib'),'-L','dependency='+str(c.B),'--emit=mir','-o',d/'runtime.mir']
 c.b.need(c.run('runtime-'+label,args)['exit']==0,'erased runtime MIR '+label)
