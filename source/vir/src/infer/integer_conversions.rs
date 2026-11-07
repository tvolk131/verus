use super::*;

fn clean(function: &Function) -> bool {
    let data = &function.x;
    data.mode == Mode::Exec
        && !data.attrs.infer
        && !data.attrs.is_unsafe
        && !data.attrs.is_async
        && data.body.is_none()
        && data.params.len() == 1
        && data.params[0].x.mode == Mode::Exec
        && !data.params[0].x.user_mut
        && data.ensure.1.is_empty()
        && crate::ast_util::types_equal(&data.inner_ret.x.typ, &data.outer_ret.x.typ)
}

fn integer(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Int(range) if arithmetic::bounds(*range).is_some())
}

fn error_type(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Datatype(Dt::Path(path), types, impls)
        if models::path_is(path, CrateId::Core, &["num", "error", "TryFromIntError"])
            && types.is_empty() && impls.is_empty())
}

fn result(typ: &Typ) -> Option<(&Typ, &Typ)> {
    match &**typ {
        TypX::Datatype(Dt::Path(path), types, impls)
            if models::path_is(path, CrateId::Core, &["result", "Result"])
                && types.len() == 2
                && impls.is_empty() =>
        {
            Some((&types[0], &types[1]))
        }
        _ => None,
    }
}

pub(super) fn unwrap_model(function: &Function) -> bool {
    let data = &function.x;
    let path = &data.name.path;
    if !clean(function)
        || path.krate != CrateId::Core
        || path.segments.len() != 3
        || path.segments[0].as_str() != "result"
        || !path.segments[1].starts_with("impl")
        || path.segments[2].as_str() != "unwrap"
        || !matches!(data.kind, FunctionKind::Static)
        || data.typ_params.len() != 2
        || data.require.len() != 1
        || data.ensure.0.len() != 1
        || !data
            .proxy
            .as_ref()
            .is_some_and(|proxy| models::spec_proxy(&proxy.x, &["std_specs", "result"], "unwrap"))
    {
        return false;
    }
    let success = Arc::new(TypX::TypParam(data.typ_params[0].clone()));
    let failure = Arc::new(TypX::TypParam(data.typ_params[1].clone()));
    result(&data.params[0].x.typ).is_some_and(|(output, error)| {
        crate::ast_util::types_equal(output, &success)
            && crate::ast_util::types_equal(error, &failure)
    }) && crate::ast_util::types_equal(&data.inner_ret.x.typ, &success)
}

pub(super) fn conversion_model(function: &Function) -> bool {
    let data = &function.x;
    let path = &data.name.path;
    if !clean(function) || !data.require.is_empty() {
        return false;
    }
    if models::path_is(path, CrateId::Core, &["convert", "TryFrom", "try_from"]) {
        return matches!(data.kind, FunctionKind::TraitMethodDecl { .. })
            && data.typ_params.len() == 2
            && data.ensure.0.len() == 1
            && data.proxy.as_ref().is_some_and(|proxy| {
                models::path_is(
                    &proxy.x,
                    CrateId::Vstd,
                    &["std_specs", "convert", "ExTryFrom", "try_from"],
                )
            });
    }
    path.krate == CrateId::Core
        && path.segments.len() == 4
        && path.segments[0].as_str() == "convert"
        && path.segments[1].as_str() == "num"
        && path.segments[2].starts_with("impl")
        && path.segments[3].as_str() == "try_from"
        && data.typ_params.is_empty()
        && data.typ_bounds.is_empty()
        && integer(&data.params[0].x.typ)
        && result(&data.inner_ret.x.typ)
            .is_some_and(|(output, error)| integer(output) && error_type(error))
        && matches!(&data.kind, FunctionKind::TraitMethodImpl {
            method, inherit_body_from: None, ..
        } | FunctionKind::ForeignTraitMethodImpl { method, .. }
            if models::path_is(&method.path, CrateId::Core, &["convert", "TryFrom", "try_from"]))
        && data.proxy.as_ref().is_some_and(|proxy| {
            models::spec_proxy(&proxy.x, &["std_specs", "convert"], "try__from")
        })
}

pub(super) fn conversion<'a>(
    expression: &'a Expr,
    functions: &HashMap<Fun, Function>,
) -> Option<&'a Expr> {
    let ExprX::Call {
        target:
            CallTarget::Fun(
                CallTargetKind::DynamicResolved {
                    resolved,
                    typs: resolved_types,
                    impl_paths,
                    is_trait_default: false,
                },
                boundary,
                types,
                implementations,
                _,
            ),
        args,
        post_args: None,
        body: None,
    } = &expression.x
    else {
        return None;
    };
    let declaration = functions.get(boundary).filter(|function| conversion_model(function))?;
    let implementation = functions.get(resolved).filter(|function| conversion_model(function))?;
    let (method, impl_path, trait_typ_args) = match &implementation.x.kind {
        FunctionKind::TraitMethodImpl { method, impl_path, trait_typ_args, .. }
        | FunctionKind::ForeignTraitMethodImpl { method, impl_path, trait_typ_args, .. } => {
            (method, impl_path, trait_typ_args)
        }
        _ => return None,
    };
    let (output, error) = result(&expression.typ)?;
    (matches!(declaration.x.kind, FunctionKind::TraitMethodDecl { .. })
        && method == boundary
        && resolved_types.is_empty()
        && impl_paths.is_empty()
        && implementations.len() == 1
        && matches!(&implementations[0], ImplPath::TraitImplPath(path) if path == impl_path)
        && args.len() == 1
        && types.len() == 2
        && trait_typ_args.len() == 2
        && crate::ast_util::types_equal(&types[0], output)
        && crate::ast_util::types_equal(&types[1], &args[0].typ)
        && types
            .iter()
            .zip(trait_typ_args.iter())
            .all(|(actual, declared)| crate::ast_util::types_equal(actual, declared))
        && error_type(error)
        && crate::ast_util::types_equal(&expression.typ, &implementation.x.inner_ret.x.typ)
        && crate::ast_util::types_equal(&args[0].typ, &implementation.x.params[0].x.typ))
    .then_some(&args[0])
}

pub(super) fn call<'a>(
    expression: &'a Expr,
    functions: &HashMap<Fun, Function>,
) -> Option<&'a Expr> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, name, types, implementations, _),
        args,
        post_args: None,
        body: None,
    } = &expression.x
    else {
        return None;
    };
    if args.len() != 1 || types.len() != 2 || !functions.get(name).is_some_and(unwrap_model) {
        return None;
    }
    let (output, error) = result(&args[0].typ)?;
    if !integer(output)
        || !error_type(error)
        || !crate::ast_util::types_equal(output, &expression.typ)
        || !crate::ast_util::types_equal(output, &types[0])
        || !crate::ast_util::types_equal(error, &types[1])
        || implementations.len() != 1
        || !matches!(&implementations[0], ImplPath::TraitImplPath(path)
            if path.krate == CrateId::Core && path.segments.len() == 3
                && path.segments[0].as_str() == "num"
                && path.segments[1].as_str() == "error"
                && path.segments[2].starts_with("impl"))
    {
        return None;
    }
    conversion(&args[0], functions)
}

pub(super) fn value(span: &Span, input: Term, output: &Typ) -> Result<(Term, Term), VirErr> {
    let TypX::Int(range) = &**output else {
        return Err(unsupported(span, "integer conversion output type"));
    };
    let safe = arithmetic::in_range(span, input.clone(), *range)?;
    Ok((safe, summary::unary(span, UnaryOp::Clip { range: *range, truncate: true }, input)?))
}

pub(super) fn maximality_model(function: &Function, types: &Typs) -> bool {
    unwrap_model(function) && types.len() == 2 && integer(&types[0]) && error_type(&types[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate_integer, function, input, span};

    #[test]
    fn independent_native_oracle_checks_success_domain_and_value() {
        let unsigned = Arc::new(TypX::Int(IntRange::U(8)));
        let signed = Arc::new(TypX::Int(IntRange::I(8)));
        for (target, signed_target) in [(&unsigned, false), (&signed, true)] {
            let input = Term { expr: input("x"), size: 1, view: None };
            let (safe, output) = value(&span(), input, target).unwrap();
            for number in -32768i128..=65535 {
                let expected = if signed_target {
                    i8::try_from(number).map(i128::from).ok()
                } else {
                    u8::try_from(number).map(i128::from).ok()
                };
                assert_eq!(
                    evaluate_integer(&safe.expr, number, 0),
                    Value::Bool(expected.is_some())
                );
                if let Some(expected) = expected {
                    assert_eq!(evaluate_integer(&output.expr, number, 0), Value::Int(expected));
                }
            }
        }
    }

    fn path(krate: CrateId, segments: &[&str]) -> Path {
        Arc::new(PathX {
            krate,
            segments: Arc::new(
                segments.iter().map(|segment| Arc::new((*segment).into())).collect(),
            ),
        })
    }

    #[test]
    fn unwrap_identity_signature_and_success_requirement_are_closed() {
        let original = function("candidate", input("x"));
        let mut data = original.x.clone();
        data.name =
            Arc::new(FunX { path: path(CrateId::Core, &["result", "impl_test", "unwrap"]) });
        data.proxy = Some(crate::def::Spanned {
            span: span(),
            x: path(
                CrateId::Vstd,
                &["std_specs", "result", "_verus_external_fn_specification_test_unwrap"],
            ),
        });
        data.typ_params = Arc::new(vec![Arc::new("Success".into()), Arc::new("Failure".into())]);
        let success = Arc::new(TypX::TypParam(data.typ_params[0].clone()));
        let failure = Arc::new(TypX::TypParam(data.typ_params[1].clone()));
        let mut parameter = data.params[0].x.clone();
        parameter.typ = Arc::new(TypX::Datatype(
            Dt::Path(path(CrateId::Core, &["result", "Result"])),
            Arc::new(vec![success.clone(), failure]),
            Arc::new(vec![]),
        ));
        data.params = Arc::new(vec![data.params[0].new_x(parameter)]);
        let mut output = data.inner_ret.x.clone();
        output.typ = success;
        data.inner_ret = data.inner_ret.new_x(output);
        data.outer_ret = data.inner_ret.clone();
        data.body = None;
        data.require = Arc::new(vec![boolean(&span(), true).expr]);
        data.ensure = (Arc::new(vec![boolean(&span(), true).expr]), Arc::new(vec![]));
        data.attrs =
            Arc::new(FunctionAttrsX { is_external_body: true, ..FunctionAttrsX::default() });
        let model = original.new_x(data);
        assert!(unwrap_model(&model));
        for mutation in 0..8 {
            let mut changed = model.x.clone();
            match mutation {
                0 => changed.proxy = None,
                1 => changed.require = Arc::new(vec![]),
                2 => changed.ensure.0 = Arc::new(vec![]),
                3 => changed.typ_params = Arc::new(vec![]),
                4 => changed.body = Some(input("x")),
                5 => Arc::make_mut(&mut changed.attrs).is_unsafe = true,
                6 => {
                    changed.name = Arc::new(FunX {
                        path: path(CrateId::Core, &["option", "impl_test", "unwrap"]),
                    })
                }
                _ => {
                    changed.name = Arc::new(FunX {
                        path: path(
                            CrateId::Id(Arc::new("application".into()), 1),
                            &["result", "impl_test", "unwrap"],
                        ),
                    })
                }
            }
            assert!(!unwrap_model(&model.new_x(changed)), "{mutation}");
        }
    }
}
