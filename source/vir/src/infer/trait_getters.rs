//! Bounded composition of ordinary exact trait getter contracts.
//!
//! No name-based model or new axiom: a shared, precondition-free method must
//! ensure result == one same-trait specification observation of its receiver.
//! Normal Verus checks every implementation/default body. Marked declarations
//! are published only after full verification and bound to inference metadata.
use super::*;

pub(super) fn shared(t: &Typ) -> Option<&Typ> {
    match &**t {
        TypX::Decorate(TypDecoration::Ref, None, t) => Some(t),
        _ => None,
    }
}

fn header(f: &Function, mode: Mode) -> Option<Path> {
    let x = &f.x;
    let FunctionKind::TraitMethodDecl { trait_path, .. } = &x.kind else { return None };
    if x.mode != mode
        || x.attrs.infer
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.attrs.prophecy_dependent
        || x.proxy.is_some()
        || x.typ_params.len() != 1
        || x.params.len() != 1
        || x.params[0].x.mode != mode
        || x.inner_ret.x.mode != mode
        || x.outer_ret.x.mode != mode
        || !matches!(x.item_kind, ItemKind::Function)
        || x.attrs.exec_assume_termination
        || x.attrs.exec_allows_no_decreases_clause
        || x.params[0].x.user_mut
        || !x.visibility.is_public()
        || !x.require.is_empty()
        || !x.ensure.1.is_empty()
        || x.returns.is_some()
        || !crate::ast_util::types_equal(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
        || !matches!(shared(&x.params[0].x.typ).map(|t|&**t),Some(TypX::TypParam(n)) if n==&x.typ_params[0])
    {
        return None;
    }
    Some(trait_path.clone())
}

fn local(e: &Expr, name: &VarIdent) -> bool {
    match &e.x {
        ExprX::Var(n) => n == name,
        ExprX::ReadPlace(p, _) => matches!(&p.x,PlaceX::Local(n) if n==name),
        _ => false,
    }
}

pub(super) fn observation(f: &Function) -> bool {
    header(f, Mode::Spec).is_some()
        && f.x.ensure.0.is_empty()
        && !enums::unit(&f.x.inner_ret.x.typ)
        && crate::ast_visitor::typ_visitor_check(&f.x.inner_ret.x.typ, &mut |t| {
            if matches!(&**t, TypX::MutRef(_) | TypX::Decorate(TypDecoration::Ref, ..)) {
                Err(())
            } else {
                Ok(())
            }
        })
        .is_ok()
}

pub(super) fn relation(f: &Function, all: &HashMap<Fun, Function>) -> Option<Function> {
    let path = header(f, Mode::Exec)?;
    if f.x.ensure.0.len() != 1 {
        return None;
    }
    let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &f.x.ensure.0[0].x else { return None };
    if !local(lhs, &f.x.inner_ret.x.name) {
        return None;
    }
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Dynamic, name, types, impls, _),
        args,
        post_args: None,
        body: None,
    } = &rhs.x
    else {
        return None;
    };
    let spec = all.get(name)?;
    if header(spec, Mode::Spec)? != path
        || !observation(spec)
        || args.len() != 1
        || types.len() != 1
        || !impls.is_empty()
        || !local(&args[0], &f.x.params[0].x.name)
        || !matches!(&*types[0],TypX::TypParam(n) if n==&f.x.typ_params[0])
        || !crate::ast_util::types_equal(&f.x.inner_ret.x.typ, &rhs.typ)
    {
        return None;
    }
    Some(spec.clone())
}

pub(super) fn prepare(functions: &[Function], imported: &[Function]) -> Vec<Function> {
    let all = functions.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    let mut getters = std::collections::HashSet::new();
    let mut observations = std::collections::HashSet::new();
    for f in functions {
        if let Some(spec) = relation(f, &all) {
            getters.insert(f.x.name.clone());
            observations.insert(spec.x.name.clone());
        }
    }
    functions
        .iter()
        .map(|f| {
            let mut x = f.x.clone();
            let attrs = Arc::make_mut(&mut x.attrs);
            attrs.infer_trait_getter = getters.contains(&x.name);
            attrs.infer_trait_observation = observations.contains(&x.name);
            f.new_x(x)
        })
        .collect()
}

pub(super) fn validate(functions: &[Function]) -> Result<(), VirErr> {
    let all = functions.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
    for f in functions {
        if f.x.attrs.infer_trait_observation && !observation(f)
            || f.x.attrs.infer_trait_getter
                && !relation(f, &all).is_some_and(|s| s.x.attrs.infer_trait_observation)
        {
            return Err(unsupported(&f.span, "invalid certified trait getter contract"));
        }
    }
    Ok(())
}

pub(super) fn source_call(
    e: &Expr,
    all: &HashMap<Fun, Function>,
) -> Option<(Function, Function, Typs, Expr)> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Dynamic, name, types, impls, _),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    let f = all.get(name)?;
    if !f.x.attrs.infer_trait_getter || types.len() != 1 || args.len() != 1 || !impls.is_empty() {
        return None;
    }
    let spec = relation(f, all)?;
    if !spec.x.attrs.infer_trait_observation {
        return None;
    }
    Some((f.clone(), spec, types.clone(), args[0].clone()))
}

/// Preserve associated types as opaque values. Only explicit associated-type
/// equality bounds are rewritten, with a bounded recursive normalization.
pub(super) fn normalize(t: &Typ, bounds: &GenericBounds) -> Result<Typ, VirErr> {
    fn go(t: &Typ, bounds: &GenericBounds, depth: usize) -> Option<Typ> {
        if depth == 0 {
            return None;
        }
        let t = crate::ast_visitor::map_typ_visitor(t, &|t| {
            if let TypX::Projection { trait_path, trait_typ_args, name } = &**t {
                for b in bounds.iter() {
                    if let GenericBoundX::TypEquality(p, args, n, value) = &**b {
                        if p == trait_path
                            && n == name
                            && args.len() == trait_typ_args.len()
                            && args
                                .iter()
                                .zip(trait_typ_args.iter())
                                .all(|(a, b)| crate::ast_util::types_equal(a, b))
                        {
                            return go(value, bounds, depth - 1).ok_or_else(|| {
                                crate::messages::error_bare(
                                    "Verus Infer incomplete: cyclic associated-type equality",
                                )
                            });
                        }
                    }
                }
            }
            Ok(t.clone())
        })
        .ok()?;
        Some(t)
    }
    go(t, bounds, 8).ok_or_else(|| {
        crate::messages::error_bare(
            "Verus Infer incomplete: associated-type normalization budget exceeded",
        )
    })
}

pub(super) fn instantiate(
    spec: &Function,
    types: &Typs,
    receiver: Term,
    bounds: &GenericBounds,
) -> Result<Term, VirErr> {
    let span = &receiver.expr.span;
    let ts = spec.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
    let ret = normalize(&crate::sst_util::subst_typ(&ts, &spec.x.inner_ret.x.typ), bounds)?;
    let param = normalize(&crate::sst_util::subst_typ(&ts, &spec.x.params[0].x.typ), bounds)?;
    let receiver = records::coerce(span, receiver.clone(), &param)?;
    records::atom(SpannedTyped::new(
        span,
        &ret,
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Dynamic,
                spec.x.name.clone(),
                types.clone(),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![receiver.expr]),
            post_args: None,
            body: None,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path(name: &str) -> Path {
        Arc::new(PathX {
            krate: CrateId::Id(Arc::new("test".into()), 1),
            segments: Arc::new(vec![Arc::new(name.into())]),
        })
    }
    fn parameter(name: &str) -> Typ {
        Arc::new(TypX::TypParam(Arc::new(name.into())))
    }
    fn projection(trait_name: &str, self_type: Typ, name: &str) -> Typ {
        Arc::new(TypX::Projection {
            trait_path: path(trait_name),
            trait_typ_args: Arc::new(vec![self_type]),
            name: Arc::new(name.into()),
        })
    }
    fn equality(trait_name: &str, self_type: Typ, name: &str, value: Typ) -> GenericBound {
        Arc::new(GenericBoundX::TypEquality(
            path(trait_name),
            Arc::new(vec![self_type]),
            Arc::new(name.into()),
            value,
        ))
    }
    #[test]
    fn nested_associated_equalities_preserve_the_opaque_payload_type() {
        let u = parameter("U");
        let family = parameter("Family");
        let nested = projection("FamilyTrait", projection("Event", u.clone(), "Family"), "Payload");
        let bounds = Arc::new(vec![equality("Event", u, "Family", family.clone())]);
        let result = normalize(&nested, &bounds).unwrap();
        assert!(crate::ast_util::types_equal(
            &result,
            &projection("FamilyTrait", family, "Payload")
        ));
    }
    #[test]
    fn equal_associated_names_from_different_traits_are_not_interchanged() {
        let u = parameter("U");
        let t = projection("A", u.clone(), "Item");
        let bounds = Arc::new(vec![equality("B", u, "Item", parameter("Other"))]);
        assert!(crate::ast_util::types_equal(&normalize(&t, &bounds).unwrap(), &t));
    }
    #[test]
    fn cyclic_associated_equalities_fail_with_a_bounded_diagnostic() {
        let u = parameter("U");
        let t = projection("Event", u.clone(), "Item");
        let bounds = Arc::new(vec![equality("Event", u, "Item", t.clone())]);
        assert!(normalize(&t, &bounds).is_err());
    }
}
