"""Focused Stage A checks, after matching tools/full NORMAL vstd are built."""
import hashlib,json,os,pathlib,shutil,sys
import build
HERE=pathlib.Path(__file__).resolve().parent

def main():
 build.need(sys.flags.dont_write_bytecode,'use -B')
 r=pathlib.Path(os.environ.get('REV_STAGE_A_ROOT') or (HERE/'PRIVATE_ROOT').read_text().strip());b=r/'bundle';label=sys.argv[1]
 d=r/label;d.mkdir()
 # Every invocation retains its exact sources/driver and controller digests.
 inputs={str(p):build.sha(p) for p in [HERE/'inspect.rs',HERE/'check.py',HERE/'build.py']}
 old=HERE.parent/'FREEZE.json';f=json.loads(old.read_text());item=pathlib.Path(f['allowed_item']['path'])
 build.need(build.sha(item)==f['allowed_item']['sha256'],'exact genuine Item')
 src=d/'item.rs';src.write_bytes(item.read_bytes()+b'''\npub fn actual_rev<'a>(e:core::iter::Enumerate<core::slice::Iter<'a,Item>>) -> core::iter::Rev<core::iter::Enumerate<core::slice::Iter<'a,Item>>> {e.rev()}\n''')
 inputs[str(src)]=build.sha(src)
 build.write(d/'inputs.json',inputs)
 tc=build.TC;deps=r/'src/source/target-verus/debug/deps'
 # Use only the matching own compiler's library and dependencies.
 inspector=d/'inspect'
 virs=list(deps.glob('libvir-*.rlib'));build.need(len(virs)==1,'one matching own VIR library')
 result=build.run(r,label+'-driver-build',[tc/'bin/rustc',HERE/'inspect.rs','--edition=2024','--extern','rust_verify='+str(r/'target/debug/librust_verify.rlib'),'--extern','vir='+str(virs[0]),'-L','dependency='+str(deps),'-L',tc/'lib','-C','rpath=yes','-o',inspector])
 build.need(result['exit']==0,'loaded validator diagnostic build')
 result=build.run(r,label+'-metadata',[inspector,src,'--crate-name','rev_stage_a_item','--crate-type=lib','--edition=2024','--extern','vstd='+str(b/'libvstd.rlib'),'-L','dependency='+str(b),'--emit=metadata','-o',d/'item.rmeta'])
 build.need(result['exit']==0,'actual loaded metadata validator')
 text=(r/'epochs'/(label+'-metadata')/'stdout').read_text()
 build.need('METADATA_PASS true WRONG_ADT_REFUSED true MUTATED_DESCRIPTOR_REFUSED true INSTALLED false' in text,'actual metadata outcomes')
 sources={
 'ordinary':'''use vstd::prelude::*;
verus! {
mod store {
    pub struct Secret { value:u64 }
    impl Secret {
        pub closed spec fn view(&self)->u64 {self.value}
        pub fn new(value:u64)->(r:Self) ensures r.view()==value, {Self{value}}
        pub fn replace(&mut self,value:u64) ensures final(self).view()==value, {
            let slot=&mut self.value;
            *slot=value;
        }
    }
}
pub fn client(x:u64)->(r:u64) ensures r==x, {
    let mut s=store::Secret::new(0);
    s.replace(x);
    assert(s.view()==x);
    x
}
}
''',
 'privacy':'''use vstd::prelude::*;
verus! { pub fn denied<'a>(r:core::iter::Rev<core::slice::Iter<'a,u64>>) {let _=r.iter;} }
''',
 'attribute':'''use vstd::prelude::*;
verus! { #[verifier::internal_representation] pub struct Forged { value:u64 } }
''',
 }
 trials={}
 for name,source in sources.items():
  p=d/(name+'.rs');p.write_text(source);inputs[str(p)]=build.sha(p)
  j=build.run(r,label+'-'+name,[b/'verus',p,'--crate-name','stage_a_'+name,'--crate-type=lib','--no-cheating','--compile','--output-json','-o',d/(name+'.rlib')])
  stderr=(r/'epochs'/(label+'-'+name)/'stderr').read_text()
  if name=='ordinary':
   raw=json.loads((r/'epochs'/(label+'-'+name)/'stdout').read_text());v=raw['verification-results'];build.need(j['exit']==0 and v['success'] and v['is-verifying-entire-crate'],'ordinary field borrow smoke');trials[name]=v
  elif name=='privacy':build.need(j['exit']!=0 and 'E0616' in stderr,'source privacy control')
  else:build.need(j['exit']!=0 and 'internal_representation' in stderr and ('unrecognized' in stderr or 'unknown' in stderr),'source attribute cannot forge descriptor')
 build.need(all(build.sha(p)==h for p,h in inputs.items()),'input/controller drift')
 build.write(d/'inputs.json',inputs)
 build.write(d/'receipt.json',dict(inputs_sha256=build.sha(d/'inputs.json'),inspector_sha256=build.sha(inspector),trials=trials,actual_core_metadata=True,production_installer=False,publication=False,goal_complete=False))
 print(json.dumps(dict(actual_core_metadata=True,trials=trials,production_installer=False)))
if __name__=='__main__':main()
