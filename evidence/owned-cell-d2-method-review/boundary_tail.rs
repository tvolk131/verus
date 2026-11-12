// Synthetic compiler-API challenge only. No loaded-native/source proof credit.
fn local() -> Path { Arc::new(PathX { krate: CrateId::Id(Arc::new("review".into()), 1), segments: Arc::new(vec![]) }) }
#[derive(Clone, Copy)] enum Operation { New, IntoInner, GetMut }
struct Spans(std::cell::Cell<u64>);
impl Spans { fn get_next_span_id(&self)->u64 { let n=self.0.get()+1;self.0.set(n);n } }
fn global(k:&Krate)->vir::context::GlobalCtx {
    vir::context::GlobalCtx::new(k,local().krate.clone(),span(),10.0,Arc::new(std::sync::Mutex::new(None)),Arc::new(std::sync::Mutex::new(None)),Arc::new(vir::context::WarningCtx{fun_warn_configs:k.functions.iter().map(|f|(f.x.name.clone(),None)).collect()}),air::context::SmtSolver::Z3,true,true,false,false,false).unwrap()
}
fn used()->vir::prune::UsedBuiltins { vir::prune::UsedBuiltins{uses_array:false,uses_bytestr:false,uses_pointee_metadata:false,uses_ieee_float:false} }
fn context(k:&Krate,a:Option<&vir::internal_repr::OwnedNativeMethodAuthority<'_>>)->Result<vir::context::Ctx,VirErr> {
    let m=k.modules[0].clone();
    match a {
        Some(a)=>vir::context::Ctx::new_owned_native_methods(k,global(k),m,vec![],vec![],Default::default(),used(),vec![],vec![],a),
        None=>vir::context::Ctx::new(k,global(k),m,vec![],vec![],Default::default(),used(),vec![],vec![]),
    }
}
#[test]
fn exact_objects_diagnostics_and_inert_context() {
    use vir::internal_repr::{OwnedNativeMethodAuthority as A, OWNED_NATIVE_METHOD_IDENTITIES as IDS};
    let d=definition();let spans=Spans(std::cell::Cell::new(0));
    let functions=[Operation::New,Operation::IntoInner,Operation::GetMut].into_iter().enumerate().map(|(i,op)|build(IDS[i],op,&d,&spans)).collect();
    let k=Arc::new(KrateX{functions,datatypes:vec![d.clone()],modules:vec![Spanned::new(span(),ModuleX{path:local(),reveals:None})],reveal_groups:vec![],traits:vec![],trait_impls:vec![],assoc_type_impls:vec![],opaque_types:vec![],external_fns:vec![],external_types:vec![],path_as_rust_names:vec![],arch:Arch{word_bits:ArchWordBits::Exactly(64)}});
    // This public VIR factory trusts its caller for loaded-native acquisition.
    // No rustc context is present in this synthetic compiler-API fixture.
    let a=A::from_checked_native_unit(&k,IDS).unwrap();
    assert!(context(&k,None).is_err());
    let mut ctx=context(&k,Some(&a)).unwrap();
    let alias=k.clone();a.validate(&alias).unwrap();ctx.revalidate_owned_native_unit(&alias).unwrap();
    let copied=Arc::new((*k).clone());assert!(a.validate(&copied).is_err());assert!(ctx.revalidate_owned_native_unit(&copied).is_err());
    for f in &k.functions {
        ctx.revalidate_owned_native_function(f).unwrap();
        let copied=f.new_x(f.x.clone());assert!(ctx.revalidate_owned_native_function(&copied).is_err());
        ctx.func_map.insert(f.x.name.clone(),copied);assert!(ctx.revalidate_owned_native_function(f).is_err());
        ctx.func_map.insert(f.x.name.clone(),f.clone());
    }
    let mut negative=(*k).clone();
    for f in &mut negative.functions {
        let mut x=f.x.clone();let first=x.ensure.0[0].clone();
        Arc::make_mut(&mut x.ensure.0)[0]=first.new_x(ExprX::Unary(UnaryOp::Not,first.clone()));*f=f.new_x(x);
    }
    let negative=Arc::new(negative);assert!(a.validate(&negative).is_err());assert!(A::from_checked_native_unit(&negative,IDS).is_err());
    let diagnostic=A::from_checked_native_negative_unit(&negative,IDS).unwrap();assert!(diagnostic.validate(&k).is_err());
    let mut metadata=(*k).clone();metadata.functions.clear();let metadata=Arc::new(metadata);let inert=context(&metadata,None).unwrap();
    assert!(inert.revalidate_owned_native_function(&k.functions[0]).is_err());
    eprintln!("independent exact object/diagnostic/default-entry/metadata-inert controls passed; VIR factory acquisition remains compiler TCB");
    let mut empty=(*k).clone();empty.functions.clear();empty.datatypes.clear();let empty=Arc::new(empty);
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||ctx.revalidate_owned_native_unit(&empty)));
    assert!(result.is_err(),"record source-identified empty-unit rejection panic");
    eprintln!("D2-R1 reproduced: authorized Ctx + reconstructed zero-datatype unit panics at context.rs:805 instead of returning Err; no proof accepted");
}
