//! Independent synthetic VIR boundary controls; no source/native proof credit.
use std::sync::Arc;
use vir::ast::*;
use vir::def::Spanned;
use vir::internal_repr::{InternalRepresentation as Repr, CORE_OWNED_CELL_IDENTITY};
fn span()->vir::messages::Span {vir::messages::Span{raw_span:Arc::new(()),id:0,data:vec![],as_string:"D1 independent boundary".into()}}
fn path(parts:&[&str])->Path {Arc::new(PathX{krate:CrateId::Core,segments:Arc::new(parts.iter().map(|x|Arc::new(x.to_string())).collect())})}
fn definition()->Datatype {
 let p=path(&["cell","UnsafeCell"]);let owner=p.pop_segment();let param:Ident=Arc::new("T".into());let t=Arc::new(TypX::TypParam(param.clone()));
 let mut d=DatatypeX{name:Dt::Path(p),proxy:None,owning_module:Some(owner.clone()),visibility:Visibility{restricted_to:None},transparency:DatatypeTransparency::Never,
 typ_params:Arc::new(vec![(param,AcceptRecursiveType::Reject)]),typ_bounds:Arc::new(vec![]),
 variants:Arc::new(vec![Variant{name:Arc::new("UnsafeCell".into()),ctor_style:CtorPrintStyle::Braces,fields:Arc::new(vec![vir::ast_util::ident_binder(&vir::def::field_ident_from_rust("value"),&(t.clone(),Mode::Exec,Visibility{restricted_to:Some(owner)}))])}]),
 mode:Mode::Exec,ext_equal:false,user_defined_invariant_fn:None,sized_constraint:Some(t),destructor:false};
 d.transparency=DatatypeTransparency::Internal(Repr::owned_unsafe_cell_u64(&d,CORE_OWNED_CELL_IDENTITY).unwrap());Spanned::new(span(),d)
}
fn typ(d:&Datatype,t:Typ)->Typ {Arc::new(TypX::Datatype(d.x.name.clone(),Arc::new(vec![t]),Arc::new(vec![])))}
fn backend_accepts(k:&Krate, base:&Krate)->bool {
 use vir::context::{Ctx,GlobalCtx,WarningCtx};
 let root=path(&["consumer"]);let module=Spanned::new(span(),ModuleX{path:root,reveals:None});
 let global=GlobalCtx::new(base,CrateId::Core,span(),10.0,Arc::new(std::sync::Mutex::new(None)),Arc::new(std::sync::Mutex::new(None)),Arc::new(WarningCtx{fun_warn_configs:std::collections::HashMap::new()}),air::context::SmtSolver::Z3,true,false,false,false,false).unwrap();
 Ctx::new(k,global,module,vec![],vec![],std::collections::HashSet::new(),vir::prune::UsedBuiltins{uses_array:false,uses_bytestr:false,uses_pointee_metadata:false,uses_ieee_float:false},vec![],vec![]).is_ok()
}
fn krate(d:Datatype)->Krate {Arc::new(KrateX{functions:vec![],datatypes:vec![d],modules:vec![],reveal_groups:vec![],traits:vec![],trait_impls:vec![],assoc_type_impls:vec![],opaque_types:vec![],external_fns:vec![],external_types:vec![],path_as_rust_names:vec![],arch:Arch{word_bits:ArchWordBits::Exactly(64)}})}
#[test]
fn independent_descriptor_wire_privacy_and_recursive_type_controls() {
 let d=definition();let DatatypeTransparency::Internal(r)=&d.x.transparency else{panic!()};
 assert!(Repr::owned_unsafe_cell_u64(&d.x,(0,0,0)).is_err());
 let root=path(&["consumer"]);assert!(!r.source_visible_to(&root));assert!(r.source_visible_to(&path(&["cell","child"])));
 assert!(serde_json::to_string(&d.x).is_err());assert!(serde_json::from_str::<DatatypeTransparency>(r#"{"Internal":{}}"#).is_err());
 for i in 0..12 {
  let mut x=d.x.clone();match i {
   0=>x.name=Dt::Path(path(&["cell","Lookalike"])),1=>x.proxy=Some(Spanned{span:span(),x:root.clone()}),
   2=>x.visibility=Visibility{restricted_to:Some(root.clone())},3=>x.variants=Arc::new(vec![]),
   4=>Arc::make_mut(&mut x.typ_params)[0].0=Arc::new("U".into()),5=>x.sized_constraint=Some(Arc::new(TypX::Bool)),
   6=>Arc::make_mut(&mut x.variants)[0].ctor_style=CtorPrintStyle::Tuple,
   7=>{let b=&mut Arc::make_mut(&mut Arc::make_mut(&mut x.variants)[0].fields)[0];Arc::make_mut(b).a.2=Visibility{restricted_to:None}},
   8=>{let b=&mut Arc::make_mut(&mut Arc::make_mut(&mut x.variants)[0].fields)[0];Arc::make_mut(b).a.0=Arc::new(TypX::Int(IntRange::U(64)))},
   9=>{let b=&mut Arc::make_mut(&mut Arc::make_mut(&mut x.variants)[0].fields)[0];Arc::make_mut(b).a.1=Mode::Spec},
   10=>x.owning_module=None,11=>x.destructor=true,_=>unreachable!()};assert!(r.validate(&x).is_err(),"shape {i}");
 }
 let k=krate(d.clone());let good=typ(&d,Arc::new(TypX::Int(IntRange::U(64))));let bad=typ(&d,Arc::new(TypX::Int(IntRange::U(32))));
 assert!(vir::internal_repr::validate_type_use(&k,&good,&span()).is_ok());
 for t in [bad.clone(),Arc::new(TypX::MutRef(bad.clone())),Arc::new(TypX::Datatype(Dt::Tuple(1),Arc::new(vec![bad.clone()]),Arc::new(vec![]))),typ(&d,bad.clone())] {
  assert!(vir::internal_repr::validate_type_use(&k,&t,&span()).is_err());
 }
 // Actual datatype field container traversed at backend entry.
 let mut outer=d.x.clone();outer.name=Dt::Path(path(&["consumer","Wrap"]));outer.transparency=DatatypeTransparency::Never;
 outer.typ_params=Arc::new(vec![]);outer.sized_constraint=None;
 Arc::make_mut(&mut Arc::make_mut(&mut Arc::make_mut(&mut outer.variants)[0].fields)[0]).a.0=bad;
 let mut x=(*k).clone();x.datatypes.push(Spanned::new(span(),outer));assert!(vir::internal_repr::validate_krate_uses(&Arc::new(x)).is_err());
}
#[test]
fn document_backend_container_scope_before_any_activation() {
 let d=definition();let bad=typ(&d,Arc::new(TypX::Int(IntRange::U(32))));let base=krate(d);let p=path(&["test_trait"]);
 let mut k=(*base).clone();
 k.assoc_type_impls.push(Spanned::new(span(),AssocTypeImplX{name:Arc::new("A".into()),impl_path:p.clone(),typ_params:Arc::new(vec![]),typ_bounds:Arc::new(vec![]),trait_path:p.clone(),trait_typ_args:Arc::new(vec![]),typ:bad.clone(),impl_paths:Arc::new(vec![])}));
 let k=Arc::new(k);assert!(vir::internal_repr::validate_krate_uses(&k).is_ok());assert!(backend_accepts(&k,&base),"Ctx entry accepts associated impl wrong-width use");
 let mut k=(*base).clone();
 k.trait_impls.push(Spanned::new(span(),TraitImplX{impl_path:p.clone(),typ_params:Arc::new(vec![]),typ_bounds:Arc::new(vec![]),trait_path:p.clone(),trait_typ_args:Arc::new(vec![bad.clone()]),trait_typ_arg_impls:Spanned::new(span(),Arc::new(vec![])),owning_module:None,auto_imported:false,external_trait_blanket:false}));
 let k=Arc::new(k);assert!(vir::internal_repr::validate_krate_uses(&k).is_ok());assert!(backend_accepts(&k,&base),"Ctx entry accepts trait impl wrong-width use");
 let mut k=(*base).clone();
 k.opaque_types.push(Spanned::new(span(),OpaqueTypeX{def_fun:Arc::new(FunX{path:p.clone()}),name:p,typ_params:Arc::new(vec![bad]),typ_bounds:Arc::new(vec![])}));
 let k=Arc::new(k);assert!(vir::internal_repr::validate_krate_uses(&k).is_ok());assert!(backend_accepts(&k,&base),"Ctx entry accepts opaque wrong-width use");
}
