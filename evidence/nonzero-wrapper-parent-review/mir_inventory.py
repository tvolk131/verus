"""Read-only compiler-loaded MIR inventory, diagnostic only; run from own root."""
import sys
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'evidence/nonzero-wrapper-parent-review'))
from resume import *
need(EPOCH=='proofs','matching production tools')
D=R/'loaded-mir';D.mkdir()
p=D/'inventory.rs';p.write_text(r'''#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
use rustc_middle::ty::{self,TyCtxt};
use rustc_span::Symbol;
struct Inventory;
fn dump<'tcx>(tcx:TyCtxt<'tcx>,env:ty::TypingEnv<'tcx>,label:&str,i:ty::Instance<'tcx>) {
 let body=i.instantiate_mir_and_normalize_erasing_regions(tcx,env,
   ty::EarlyBinder::bind(tcx,tcx.instance_mir(i.def).clone()));
 let text=format!("INSTANCE {i:?}\nSIGNATURE {:#?}\n{body:#?}\n",tcx.fn_sig(i.def_id()).instantiate(tcx,i.args));
 let dir=std::env::var("WRAPPER_MIR_OUT").expect("output");
 std::fs::write(std::path::Path::new(&dir).join(format!("{label}.txt")),text).expect("write");
}
impl rustc_driver::Callbacks for Inventory {
 fn after_analysis<'tcx>(&mut self,_:&rustc_interface::interface::Compiler,tcx:TyCtxt<'tcx>)->rustc_driver::Compilation {
  let anchor=tcx.hir_body_owners().find(|id|tcx.item_name(id.to_def_id()).as_str()=="total").expect("original total").to_def_id();
  let env=ty::TypingEnv::non_body_analysis(tcx,anchor);
  for (label,name) in [("total","total"),("partial","partial"),("derived_cmp","cmp"),("derived_partial_cmp","partial_cmp")] {
   let id=tcx.hir_body_owners().find(|id|tcx.item_name(id.to_def_id()).as_str()==name).expect("original body").to_def_id();
   let i=ty::Instance::try_resolve(tcx,env,id,ty::GenericArgs::empty()).expect("resolve").expect("item");dump(tcx,env,label,i);
  }
  let ord=tcx.get_diagnostic_item(Symbol::intern("Ord")).expect("core Ord");
  let cmp=tcx.associated_items(ord).in_definition_order().find(|i|i.name().as_str()=="cmp").expect("Ord cmp").def_id;
  let nz=tcx.get_diagnostic_item(Symbol::intern("NonZero")).expect("core NZ");
  let n=ty::Ty::new_adt(tcx,tcx.adt_def(nz),tcx.mk_args(&[tcx.types.u64.into()]));
  for (label,t) in [("carrier_cmp",n),("primitive_cmp",tcx.types.u64)] {
   let i=ty::Instance::try_resolve(tcx,env,cmp,tcx.mk_args(&[t.into()])).expect("cmp resolve").expect("cmp item");dump(tcx,env,label,i);
  }
  let get=tcx.inherent_impls(nz).iter().flat_map(|id|tcx.associated_items(*id).in_definition_order()).find(|i|i.name().as_str()=="get").expect("get").def_id;
  let i=ty::Instance::try_resolve(tcx,env,get,tcx.mk_args(&[tcx.types.u64.into()])).expect("get resolve").expect("get item");dump(tcx,env,"get",i);
  rustc_driver::Compilation::Stop
 }
}
fn main(){rustc_driver::run_compiler(&std::env::args().collect::<Vec<_>>(),&mut Inventory);}
''')
me=Path(__file__)
j=run('loaded-mir-build',[TC/'bin/rustc',p,'--edition=2021','-o',D/'inventory'],dependencies=[me]);need(j['exit']==0,'diagnostic callback build')
f=json.loads((R/'final-exact-v3-cold/freeze.json').read_text());src=Path(next(iter(f['fixtures'])))
j=run('loaded-mir-run',[D/'inventory',src,'--edition=2021','--crate-type=lib','--sysroot',TC,'--extern','vstd='+str(B/'libvstd.rlib'),'-L','dependency='+str(B)],extra={'WRAPPER_MIR_OUT':str(D)},dependencies=[me,D/'inventory']);need(j['exit']==0,'actual loaded MIR diagnostic')
files={p.name:sha(p) for p in D.glob('*.txt')};need(len(files)==7,'seven actual loaded bodies')
for label,flags in [('expanded',['-Zunpretty=expanded']),('mir',['--emit=mir','-o',str(D/'erased.mir')])]:
 j=run('final-erased-'+label,[TC/'bin/rustc',src,'--edition=2021','--crate-type=lib','--extern','vstd='+str(B/'libvstd.rlib'),'-L','dependency='+str(B),*flags],dependencies=[me]);need(j['exit']==0,'erasure diagnostic '+label)
s=(R/'final-erased-expanded/stdout').read_text();need('a.cmp(b)' in s and 'checked_nonzero' not in s,'original erased call fidelity')
write(R/'loaded-mir-result.json',dict(source=sha(src),controller_sha256=sha(me),seven_loaded_bodies=files,erased_expansion_sha256=sha(R/'final-erased-expanded/stdout'),erased_mir_sha256=sha(D/'erased.mir'),native_correspondences=0,classification='actual pinned compiler MIR and macro-erased runtime diagnostics only; not new source/native laws'))
