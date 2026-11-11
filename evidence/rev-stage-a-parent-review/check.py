"""Own fresh-tool review controls. No live parent paths or network."""
import importlib.util,json,pathlib,shutil,sys
import build as b
R=pathlib.Path((b.HERE/'PRIVATE_ROOT').read_text().strip()); B=R/'bundle'; DEPS=R/'intermediates/debug/deps'
def run(label,args):
 inputs={str(p):b.sha(p) for p in [b.HERE/'check.py']}
 for a in args:
  p=pathlib.Path(str(a))
  if p.is_file():inputs[str(p)]=b.sha(p)
 j=b.run(R,label,args)
 b.need(all(b.sha(p)==h for p,h in inputs.items()),'extra input guards '+label)
 b.write(R/'epochs'/label/'inputs.json',inputs)
 return j

def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def app(label,source,expected=None,infer=False,extras=()):
 d=R/'apps'/label;d.mkdir(parents=True);p=d/'input.rs';p.write_text(source)
 args=[B/'verus',p,'--crate-name',label.replace('-','_'),'--crate-type=lib','--no-cheating','--compile','--output-json','--log-all','--log-dir',d/'logs','-o',d/('lib'+label.replace('-','_')+'.rlib'),*extras]
 if infer:args.append('--infer-goals')
 j=run(label,args)
 if expected is not None:
  b.need(j['exit']==0,'app exit '+label)
  raw=json.loads((R/'epochs'/label/'stdout').read_text());v=raw['verification-results']
  b.need(v['success'] and v['is-verifying-entire-crate'] and (v['verified'],v['errors'])==(expected,0),'app results '+label)
  b.write(d/'inventory.json',raw['verus-infer'])
 return j,d

def tests():
 for label,args in [('vir-157-final',[b.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','vir','--lib','--','--test-threads=2']),('rust-verify-lib8',[b.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify','--lib','--','--test-threads=2'])]:
  b.need(run(label,args)['exit']==0,label)
 # Existing integration harness derives ../target-verus/debug relative to its exe.
 dest=R/'target-verus/debug';dest.mkdir(parents=True)
 for n in b.TOOLS:shutil.copy2(B/n,dest/n)
 b.need(run('existing-constructors13',[b.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--test','constructor_function_values','--','--test-threads=2'])['exit']==0,'constructors')

def diagnostics():
 # Build unchanged historical inspector from OWN Git source, against own tools.
 source=b.ROOT/'source/tools/research/rev_nominal_definition/stage_a/inspect.rs'
 d=R/'metadata-final';d.mkdir();src=d/'actual.rs'
 item=pathlib.Path('/tmp/local-nonzero-ord-09t1tshb/source-1adb8b6c2ee64cc1.rs')
 b.need(b.sha(item)=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','Item')
 src.write_bytes(item.read_bytes()+b"\npub fn actual_rev<'a>(e:core::iter::Enumerate<core::slice::Iter<'a,Item>>) -> core::iter::Rev<core::iter::Enumerate<core::slice::Iter<'a,Item>>> {e.rev()}\n")
 virs=list(DEPS.glob('libvir-*.rlib'));b.need(len(virs)==1,'one VIR')
 b.need(run('inspector-build-final',[b.TC/'bin/rustc',source,'--edition=2024','--extern','rust_verify='+str(R/'target/debug/librust_verify.rlib'),'--extern','vir='+str(virs[0]),'-L','dependency='+str(DEPS),'-L',b.TC/'lib','-C','rpath=yes','-o',d/'inspect'])['exit']==0,'inspector')
 b.need(run('actual-metadata-final',[d/'inspect',src,'--crate-name','actual_item','--crate-type=lib','--edition=2024','--extern','vstd='+str(B/'libvstd.rlib'),'-L','dependency='+str(B),'--emit=metadata','-o',d/'actual.rmeta'])['exit']==0,'metadata')
 for name,source,needle in [('rust-privacy',"use vstd::prelude::*; verus!{pub fn denied<'a>(r:core::iter::Rev<core::slice::Iter<'a,u64>>) {let _=r.iter;}}",'E0616'),('forged-attribute','use vstd::prelude::*;verus!{#[verifier::internal_representation] pub struct Fake { value:u64 }}','unrecognized verifier attribute')]:
  j,_=app(name,source);text=(R/'epochs'/name/'stderr').read_text();b.need(j['exit']!=0 and needle in text,'diagnostic '+name)

def apps():
 item=pathlib.Path('/tmp/local-nonzero-ord-09t1tshb/source-1adb8b6c2ee64cc1.rs')
 b.need(b.sha(item)=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','exact Item')
 app('item132',item.read_text(),132,infer=True)
 f=module('own_wrapper_fixtures',b.ROOT/'source/tools/research/nonzero_wrapper_ordering/fixtures.py')
 app('wrapper28',f.source('request'),28,infer=True)
 app('packet31',(b.ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text(),31,infer=True)
 source='use vstd::prelude::*;verus!{pub fn all_bool(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}'
 p=R/'bool.vir';_,provider=app('bool_provider',source,2,infer=True,extras=['--export',p])
 extra=['--extern','bool_provider='+str(provider/'libbool_provider.rlib'),'--import','bool_provider='+str(p),'-L',provider]
 client='use vstd::prelude::*;verus!{pub fn client(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {bool_provider::all_bool(b)}}'
 app('bool_import',client,1,extras=extra)
 # Normal typed assertion failure, separate from metadata diagnostics.
 j,_=app('bool_false',client.replace('r==if b {1u64}else{0u64}','r==0u64'),extras=extra)
 raw=json.loads((R/'epochs/bool_false/stdout').read_text())
 b.need(j['exit']==1 and raw['verification-results']['errors']==1 and not raw['verification-results']['encountered-vir-error'],'false bool is ordinary proof failure')
 # Old-format trailer, typed policy and truncated base controls only.
 data=p.read_bytes();i=data.rfind(b'{"schema":');b.need(i>=0 and data.endswith(b'VRINFER1'),'trailer')
 trailer=json.loads(data[i:-8])
 for label,value in [('old_schema',38),('old_policy','candidate-bound-evidence-v74'),('truncated_base',None)]:
  bad=R/(label+'.vir');meta=dict(trailer)
  if label=='truncated_base':bad.write_bytes(data[:20])
  else:
   meta['schema' if label=='old_schema' else 'policy']=value;bad.write_bytes(data[:i]+json.dumps(meta,separators=(',',':')).encode()+b'VRINFER1')
  e=list(extra);e[e.index('bool_provider='+str(p))]='bool_provider='+str(bad)
  j,_=app(label,client,extras=e)
  text=(R/'epochs'/label/'stderr').read_text()
  b.need(j['exit']!=0 and ('incompatible inference' in text or 'failed to deserialize imported library' in text),'format refusal '+label)
 ordinary='''use vstd::prelude::*;
verus!{mod store {pub struct Secret {value:u64} impl Secret {
pub closed spec fn view(&self)->u64 {self.value}
pub fn new(value:u64)->(r:Self) ensures r.view()==value, {Self{value}}
pub fn replace(&mut self,value:u64) ensures final(self).view()==value, {let slot=&mut self.value;*slot=value;}
}} pub fn client(x:u64)->(r:u64) ensures r==x, {let mut s=store::Secret::new(0);s.replace(x);assert(s.view()==x);x}}'''
 app('ordinary_borrow3',ordinary,3)
 # Baseline gate negatives use ordinary visible source structures, not Internal.
 app('generic_nested','''use vstd::prelude::*;verus!{
struct Wrap<T>{value:T} enum Tree { Leaf(u8), Node(Box<Tree>) }
pub fn round(x:u8)->(r:u8) ensures r==x, {let mut a=Wrap{value:Wrap{value:x}};let p=&mut a.value.value;*p=x; a.value.value}
pub fn recursive(t:Tree)->(r:Tree) {t}
}''',2)
 for label,source in [('private_pattern','use vstd::prelude::*;verus!{mod inner{pub struct S {x:u64}} fn f(s:inner::S){let inner::S{x}=s;}}'),('private_contract','use vstd::prelude::*;verus!{mod inner{pub struct S{x:u64} pub open spec fn leak(s:S)->u64{s.x}}}'),('private_constructor','use vstd::prelude::*;verus!{mod inner{pub struct S {x:u64}} fn f(){let _=inner::S{x:0};}}')]:
  j,_=app(label,source);text=(R/'epochs'/label/'stderr').read_text();b.need(j['exit']!=0 and ('private' in text or 'non-visible' in text),'privacy '+label)

if __name__=='__main__':{'tests':tests,'diagnostics':diagnostics,'apps':apps}[sys.argv[1]]()
