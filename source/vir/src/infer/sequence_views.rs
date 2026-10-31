//! A closed, direct projection language for supplied sequence views.
//!
//! Local definitions are inspected before marking them. Imported marks are
//! covered by the inference certificate, including the view declaration and
//! datatype. No view definition is unfolded in a foreign module, and no axiom
//! or executable operation is introduced.
use super::*;

#[derive(Clone)]
pub(super) struct SequenceView {
    pub function: Function,
    pub types: Typs,
    pub storage: FieldOpr,
    pub element: Typ,
}

fn receiver(f: &Function) -> Option<&Typ> {
    match &*f.x.params.first()?.x.typ {
        TypX::Decorate(TypDecoration::Ref, None, t) => Some(t),
        _ => None,
    }
}

fn signature(f: &Function, datatypes: &[Datatype]) -> Option<(FieldOpr, Typ)> {
    let x = &f.x;
    let FunctionKind::TraitMethodImpl {
        method,
        trait_path,
        trait_typ_args,
        inherit_body_from: None,
        ..
    } = &x.kind
    else {
        return None;
    };
    if !models::path_is(&method.path, CrateId::Vstd, &["view", "View", "view"])
        || !models::path_is(trait_path, CrateId::Vstd, &["view", "View"])
        || x.mode != Mode::Spec
        || x.attrs.infer
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.proxy.is_some()
        || x.params.len() != 1
        || x.params[0].x.mode != Mode::Spec
        || x.params[0].x.user_mut
        || !x.visibility.is_public()
        || !x.require.is_empty()
        || !x.ensure.0.is_empty()
        || !x.ensure.1.is_empty()
        || x.returns.is_some()
        || !x.decrease.is_empty()
        || x.decrease_by.is_some()
        || x.decrease_when.is_some()
        || x.typ_params.len() > 2
    {
        return None;
    }
    let typ = receiver(f)?;
    let TypX::Datatype(_, args, _) = &**typ else { return None };
    if args.len() != x.typ_params.len()
        || !args
            .iter()
            .zip(x.typ_params.iter())
            .all(|(t, p)| matches!(&**t,TypX::TypParam(n) if n==p))
        || trait_typ_args.len() != 1
        || !crate::ast_util::types_equal(&trait_typ_args[0], typ)
    {
        return None;
    }
    // Only Sized (and optionally Copy) bounds, just like bounded records.
    if !records::shape::bounds(&x.typ_bounds, &x.typ_params) {
        return None;
    }
    let fields = records::shape::fields(typ, datatypes)?;
    if fields.len() != 1 {
        return None;
    }
    let (op, vector) = fields.into_iter().next()?;
    let ts = collections::vec_args(&vector)?;
    if !matches!(&*ts[1],TypX::Datatype(Dt::Path(p),xs,_) if xs.is_empty() && models::path_is(p,CrateId::Alloc,&["alloc","Global"]))
        || !records::shape::value_type(&ts[0], &x.typ_params, datatypes, 4)
    {
        return None;
    }
    let result = &x.inner_ret.x.typ;
    if !matches!(&**result,TypX::Datatype(Dt::Path(p),args,_) if models::path_is(p,CrateId::Vstd,&["seq","Seq"]) && args.len()==1 && crate::ast_util::types_equal(&args[0],&ts[0]))
        || !crate::ast_util::types_equal(result, &x.outer_ret.x.typ)
    {
        return None;
    }
    Some((op, vector))
}

fn direct_body(f: &Function, all: &HashMap<Fun, Function>, storage: &FieldOpr) -> bool {
    let Some(mut body) = f.x.body.as_ref() else { return false };
    while let ExprX::Block(stmts, Some(tail)) = &body.x {
        if !stmts.is_empty() {
            return false;
        }
        body = tail;
    }
    let ExprX::Call {
        target:
            CallTarget::Fun(
                CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. },
                name,
                ..,
            ),
        args,
        post_args: None,
        body: None,
    } = &body.x
    else {
        return false;
    };
    if !models::path_is(&name.path, CrateId::Vstd, &["view", "View", "view"])
        || all.get(resolved).and_then(collections::model) != Some(collections::Model::View)
        || args.len() != 1
    {
        return false;
    }
    let ExprX::ReadPlace(p, _) = &args[0].x else { return false };
    let PlaceX::Field(op, base) = &p.x else { return false };
    records::shape::same_path(std::slice::from_ref(op), std::slice::from_ref(storage))
        && matches!(&base.x,PlaceX::Local(n) if n==&f.x.params[0].x.name)
}

pub(super) fn prepare(
    functions: &[Function],
    imported: &[Function],
    datatypes: &[Datatype],
) -> Vec<Function> {
    let all = functions.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    functions
        .iter()
        .map(|f| {
            let mut x = f.x.clone();
            // A source declaration can never supply this evidence itself.
            let direct = signature(f, datatypes).is_some_and(|(op, _)| direct_body(f, &all, &op));
            Arc::make_mut(&mut x.attrs).infer_sequence_view = direct;
            f.new_x(x)
        })
        .collect()
}

pub(super) fn validate_imports(
    functions: &[Function],
    datatypes: &[Datatype],
) -> Result<(), VirErr> {
    let all = functions.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
    for f in functions.iter().filter(|f| f.x.attrs.infer_sequence_view) {
        let invalid = || unsupported(&f.span, "invalid certified direct sequence view");
        let (storage, _) = signature(f, datatypes).ok_or_else(invalid)?;
        if f.x.body.is_some() && !direct_body(f, &all, &storage) {
            return Err(invalid());
        }
    }
    Ok(())
}

pub(super) fn find(
    typ: &Typ,
    all: &HashMap<Fun, Function>,
    datatypes: &[Datatype],
) -> Option<SequenceView> {
    let TypX::Datatype(dt, args, _) = &**typ else { return None };
    let mut found = None;
    for f in all.values().filter(|f| f.x.attrs.infer_sequence_view) {
        let Some((storage, vector)) = signature(f, datatypes) else { continue };
        let Some(TypX::Datatype(actual, _, _)) = receiver(f).map(|t| &**t) else { continue };
        if actual != dt || f.x.typ_params.len() != args.len() {
            continue;
        }
        // The marker is installed before inference. Later normal VIR passes
        // may rewrite its body; imported closed views have no exported body.
        let substitutions = f.x.typ_params.iter().cloned().zip(args.iter().cloned()).collect();
        let vector = crate::sst_util::subst_typ(&substitutions, &vector);
        let element = collections::vec_args(&vector)?[0].clone();
        if found.is_some() {
            return None;
        }
        found = Some(SequenceView { function: f.clone(), types: args.clone(), storage, element });
    }
    found
}

/// Module pruning intentionally erases private datatype fields. The checked,
/// certificate-bound marker retains the direct-projection observation selected
/// before pruning; the candidate relation never supplies this evidence.
pub(super) fn observation(typ: &Typ, all: &HashMap<Fun, Function>) -> Option<Function> {
    let TypX::Datatype(dt, args, _) = &**typ else { return None };
    // Normalize shared-reference decorations when matching the logical wrapper.
    // Only a previously certified marker can supply this observation.
    let observed_receiver = |f: &Function| {
        let mut t = f.x.params.first()?.x.typ.clone();
        while let TypX::Decorate(TypDecoration::Ref, None, inner) = &*t {
            t = inner.clone();
        }
        match &*t {
            TypX::Datatype(actual, _, _) => Some(actual.clone()),
            _ => None,
        }
    };
    let mut views=all.values().filter(|f|f.x.attrs.infer_sequence_view && f.x.mode==Mode::Spec
        && f.x.typ_params.len()==args.len()
        && observed_receiver(f).as_ref() == Some(dt)
        && matches!(&f.x.kind,FunctionKind::TraitMethodImpl{method,..} if models::path_is(&method.path,CrateId::Vstd,&["view","View","view"])));
    let view = views.next()?.clone();
    views.next().is_none().then_some(view)
}

pub(super) fn call(v: &SequenceView, value: Expr) -> Result<Term, VirErr> {
    let span = value.span.clone();
    let FunctionKind::TraitMethodImpl { method, impl_path, trait_typ_args, .. } =
        &v.function.x.kind
    else {
        unreachable!()
    };
    let ts = v.function.x.typ_params.iter().cloned().zip(v.types.iter().cloned()).collect();
    let ret = crate::sst_util::subst_typ(&ts, &v.function.x.inner_ret.x.typ);
    let typ = crate::sst_util::subst_typ(&ts, &v.function.x.params[0].x.typ);
    let value = SpannedTyped::new(
        &span,
        &typ,
        ExprX::ReadPlace(
            SpannedTyped::new(&span, &value.typ, PlaceX::Temporary(value.clone())),
            UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
        ),
    );
    let target = CallTarget::Fun(
        CallTargetKind::DynamicResolved {
            resolved: v.function.x.name.clone(),
            typs: v.types.clone(),
            impl_paths: Arc::new(vec![]),
            is_trait_default: false,
        },
        method.clone(),
        Arc::new(trait_typ_args.iter().map(|t| crate::sst_util::subst_typ(&ts, t)).collect()),
        Arc::new(vec![ImplPath::TraitImplPath(impl_path.clone())]),
        CallTargetAttrs {
            autospec: AutospecUsage::Final,
            const_var: false,
            assume_external_allowed: false,
        },
    );
    records::atom(SpannedTyped::new(
        &span,
        &ret,
        ExprX::Call { target, args: Arc::new(vec![value]), post_args: None, body: None },
    ))
}
