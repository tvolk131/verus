"""Independent synthetic VIR role/assertion oracle using this review's fresh library.

The ORIGINAL native/client executable is never modified. Extract frozen builder
and ordinary pipeline into a NEW synthetic compiler-API fixture, replace rustc
acquisition with explicit public VIR factories, and inspect Invalid's message
and exact ensures span. This grants no loaded-native or original-client proof.
"""
import json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path(__file__).resolve().parents[2];sys.path.insert(0,str(R/'source/tools'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
O=Path(sys.argv[1]);O.mkdir(exist_ok=False);deps=Path(sys.argv[2])/'build/debug/deps'
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu');S=Path('/tmp/owned-cell-client-C1-20261112-freeze')
B=json.loads(Path(__file__).with_name('BINDING.json').read_text());lower=R/'source/rust_verify/src/checked_owned_cell_lowering.rs';old=R/'evidence/owned-cell-d1-review/boundary.rs'
require(digest(lower)==B['source'][str(lower.relative_to(R))],'frozen lowering')
s=lower.read_text();descriptor=old.read_text().split('fn typ(')[0]
helpers=s[s.index('fn u64t()'):s.index('fn datatype(')]
builder=s[s.index('fn build('):s.index('fn matches_type(')]
builder='fn build(native:u64,op:Operation,d:&Datatype,spans:&Spans)->Function {\n'+builder.split(') -> Function {\n',1)[1]
builder=builder.replace('span(tcx, native)','span()').replace('crate::rust_to_vir_base::mk_crate_id(tcx, rustc_hir::def_id::LOCAL_CRATE)','local().krate.clone()').replace('tcx.def_path_hash(native).local_hash().as_u64()','native')
base='''
use std::collections::HashMap;
use vir::ast_util::{bool_typ,str_unique_var};
use vir::internal_repr::{OwnedNativeMethodAuthority as A,OWNED_NATIVE_METHOD_IDENTITIES as IDS};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] enum Operation {New,IntoInner,GetMut}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] enum NativeQuery {Positive,FalseFirst,FalseFuture}
struct Spans(std::cell::Cell<u64>);
impl Spans {fn get_next_span_id(&self)->u64 {let n=self.0.get()+1;self.0.set(n);n}}
fn local()->Path {Arc::new(PathX{krate:CrateId::Id(Arc::new("c1_review".into()),1),segments:Arc::new(vec![])})}
fn authority(k:&Krate,q:NativeQuery)->Result<A<'_>,&'static str> {match q {
NativeQuery::Positive=>A::from_checked_native_unit(k,IDS),
NativeQuery::FalseFirst=>A::from_checked_native_negative_unit(k,IDS),
NativeQuery::FalseFuture=>A::from_checked_native_false_future_unit(k,IDS)}}
#[test] fn role_and_exact_failed_postcondition() {
let d=definition();let root=local();let spans=Spans(std::cell::Cell::new(100));
let functions=[Operation::New,Operation::IntoInner,Operation::GetMut].into_iter().enumerate().map(|(i,o)|build(IDS[i],o,&d,&spans)).collect();
let modules=vec![Spanned::new(span(),ModuleX{path:root.clone(),reveals:None}),Spanned::new(span(),ModuleX{path:d.x.owning_module.clone().unwrap(),reveals:None})];
let k=Arc::new(KrateX{functions,datatypes:vec![d.clone()],modules,reveal_groups:vec![],traits:vec![],trait_impls:vec![],assoc_type_impls:vec![],opaque_types:vec![],external_fns:vec![],external_types:vec![],path_as_rust_names:vec![],arch:Arch{word_bits:ArchWordBits::Exactly(64)}});
'''
setup=s[s.index('    vir::internal_repr::validate_krate_uses(&k).unwrap();'):s.index('    let first = &k.functions[0];')]
global_code=s[s.index('    let global = |unit: &Krate'):s.index('    let module = k.modules[0].clone();')]
pipeline=s[s.index('    let run_queries = |unit: &Krate'):s.index('    run_queries(&k, NativeQuery::Positive);')]
pipeline=pipeline.replace('method_authority_query(tcx, ','authority(').replace('method_authority(tcx, &bad)','authority(&bad, NativeQuery::Positive)')
anchor='''                    let is_query = matches!(&**command, air::ast::CommandX::CheckValid(_));'''
extra='''
                    if let air::context::ValidityResult::Invalid(_,Some(message),_) = &result {
                        let message=message.downcast_ref::<vir::messages::MessageX>().unwrap();
                        assert_eq!(message.note,vir::def::POSTCONDITION_FAILURE);
                        let original=simple.functions.iter().find(|g|g.x.name==f.x.name).unwrap();
                        let index=if query==NativeQuery::FalseFuture {1}else{0};
                        let wanted=original.x.ensure.0[index].span.id;
                        assert!(message.labels.iter().any(|l|l.span.id==wanted && l.note==vir::def::THIS_POST_FAILED),"wrong failed postcondition {message:?}; expected span {wanted}");
                        eprintln!("C1 REVIEW EXACT INVALID role={query:?} function={:?} ensure_index={index} span={wanted} note={}",f.x.name,message.note);
                    } else if matches!(&result,air::context::ValidityResult::Invalid(..)) {panic!("Invalid without bound message");}
'''
require(anchor in pipeline,'pipeline anchor');pipeline=pipeline.replace(anchor,anchor+extra)
end='''
let a=authority(&k,NativeQuery::Positive).unwrap();
let mut first=(*k).clone();
for f in &mut first.functions {let mut x=f.x.clone();let e=x.ensure.0[0].clone();Arc::make_mut(&mut x.ensure.0)[0]=e.new_x(ExprX::Unary(UnaryOp::Not,e.clone()));*f=f.new_x(x);}
let first=Arc::new(first);
let mut future=(*k).clone();let f=&future.functions[2];let mut x=f.x.clone();let e=x.ensure.0[1].clone();Arc::make_mut(&mut x.ensure.0)[1]=e.new_x(ExprX::Unary(UnaryOp::Not,e.clone()));future.functions[2]=f.new_x(x);let future=Arc::new(future);
assert!(Arc::ptr_eq(&k.functions[0],&future.functions[0]));assert!(Arc::ptr_eq(&k.functions[1],&future.functions[1]));assert!(Arc::ptr_eq(k.functions[2].x.body.as_ref().unwrap(),future.functions[2].x.body.as_ref().unwrap()));assert!(Arc::ptr_eq(&k.functions[2].x.ensure.0[0],&future.functions[2].x.ensure.0[0]));
for (i,unit) in [&k,&first,&future].iter().enumerate() {for (j,query) in [NativeQuery::Positive,NativeQuery::FalseFirst,NativeQuery::FalseFuture].into_iter().enumerate() {assert_eq!(authority(unit,query).is_ok(),i==j,"exact 3x3 role matrix");}}
assert!(a.validate(&future).is_err());
run_queries(&k,NativeQuery::Positive);run_queries(&first,NativeQuery::FalseFirst);run_queries(&future,NativeQuery::FalseFuture);
eprintln!("C1 REVIEW: nine synthetic role queries;5 Valid4 Invalid, all four failures bound to intended postconditions; cross-role matrix9 and healthy Arc identities passed; NOT loaded-native or client proof");
}
'''
import re
combined=descriptor+helpers+builder+base+setup+global_code+pipeline+end
combined=re.sub(r'\bauthority\(', 'scope_authority(', combined).replace('authority.authorizes(&k, &copied, d)', 'authority.authorizes(&k, &copied, &d)')
fixture=O/'fixture.rs';fixture.write_text(combined)
libs=[];artifacts=[]
for n in ['vir','air']:
 matches=list(deps.glob('lib'+n+'-*.rlib'));require(len(matches)==1,'one fresh library '+n);artifacts+=matches;libs+=['--extern',n+'='+str(matches[0])]
inputs=[Path(__file__),lower,old,fixture,S/'z3',*artifacts];g=ArtifactGuard(inputs)
r=dict(commands=[],provenance='NEW synthetic VIR compiler-API/assertion oracle; exact frozen builder/pipeline extraction with explicit substitutions, this review fresh VIR library. Not original harness, native acquisition or client proof.',inputs={str(p):digest(p) for p in inputs})
env=dict(os.environ,PATH='/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),TMPDIR=str(O),VERUS_Z3_PATH=str(S/'z3'),RUST_MIN_STACK='20971520')
with replay_receipt(O,r):
 run=LoggedCommands(O,R/'source',r['commands'],timeout=120,guard=g)
 require(run('build',[str(TC/'bin/rustc'),'--edition=2024','--test',str(fixture),'-L','dependency='+str(deps),*libs,'-o',str(O/'role-tests')],env)==0,'synthetic compile')
 require(run('run',[str(O/'role-tests'),'--nocapture','--test-threads=1'],env)==0,'role/assertion oracle')
 err=(O/'run.stderr').read_text();require(err.count('C1 REVIEW EXACT INVALID')==4,'four exact Invalid labels')
 require(err.count('role=FalseFuture')==1 and 'ensure_index=1' in err,'intended future postcondition')
 g.check();r.update(guard_passed=True,distinct_application_proofs=0,synthetic_method_queries=9,exact_invalid_messages=[x for x in err.splitlines() if x.startswith('C1 REVIEW')],logs={p.name:digest(p) for p in O.glob('*.std*')})
print(json.dumps(dict(output=str(O),success=r['success'],receipt_sha256=digest(O/'receipt.json'))))
