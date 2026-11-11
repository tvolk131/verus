import ast,json,pathlib,sys
import check as c
b=c.b;R=c.R;B=c.B
mode=sys.argv[1]
if mode=='diagnostics':
 d=R/'metadata-rerun';d.mkdir();source=b.ROOT/'source/tools/research/rev_nominal_definition/stage_a/inspect.rs';src=R/'metadata/actual.rs'
 b.need(c.run('inspector-build-rerun',[b.TC/'bin/rustc',source,'--edition=2024','--extern','rust_verify='+str(R/'target/debug/librust_verify.rlib'),'--extern','vir='+str(c.DEPS/'libvir-c3b90db594511844.rlib'),'-L','dependency='+str(c.DEPS),'-L',b.TC/'lib','-C','rpath=yes','-o',d/'inspect'])['exit']==0,'inspector')
 b.need(c.run('actual-metadata-rerun',[d/'inspect',src,'--crate-name','actual_item','--crate-type=lib','--edition=2024','--extern','vstd='+str(B/'libvstd.rlib'),'-L','dependency='+str(B),'--emit=metadata','-o',d/'actual.rmeta'])['exit']==0,'metadata')
 for name,source,needle in [('rust-privacy',"use vstd::prelude::*; verus!{pub fn denied<'a>(r:core::iter::Rev<core::slice::Iter<'a,u64>>) {let _=r.iter;}}",'E0616'),('forged-attribute','use vstd::prelude::*;verus!{#[verifier::internal_representation] pub struct Fake { value:u64 }}','unrecognized verifier attribute')]:
  j,_=c.app(name,source);text=(R/'epochs'/name/'stderr').read_text();b.need(j['exit']!=0 and needle in text,'diagnostic '+name)
if mode=='apps':
 # Extract only pure source literal; do not import historical controller state.
 f=c.module('own_wrapper_fixtures',b.ROOT/'source/tools/research/nonzero_wrapper_ordering/fixtures.py')
 tree=ast.parse((b.ROOT/'source/tools/research/nonzero_wrapper_ordering/controls.py').read_text())
 node=next(n for n in tree.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='CAND' for t in n.targets))
 source=f.source('request')+ast.literal_eval(node.value.right)
 c.app('wrapper28-final',source,28,infer=True)
 # Continue the unchanged apps controller after its completed Item/wrapper prefix.
 text=(b.HERE/'check.py').read_text();start=text.index(" app('packet31'");end=text.index("\nif __name__=='__main__'")
 body='def rest():\n'+text[start:end]+'\nrest()\n'
 exec(compile(body,str(b.HERE/'check.py')+':remaining','exec'),c.__dict__)
