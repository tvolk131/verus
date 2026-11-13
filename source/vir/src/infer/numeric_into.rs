use super::*;

fn names(fallible: bool) -> (&'static str, &'static str, &'static str, &'static str) {
    if fallible {
        ("TryInto", "try_into", "ExTryInto", "TryFrom")
    } else {
        ("Into", "into", "ExInto", "From")
    }
}

fn model_kind(function: &Function, fallible: bool) -> bool {
    let data = &function.x;
    let (boundary, method, extension, conversion) = names(fallible);
    if data.mode != Mode::Exec
        || data.attrs.infer
        || data.attrs.is_unsafe
        || data.attrs.is_async
        || data.body.is_some()
        || data.typ_params.len() != 2
        || data.params.len() != 1
        || data.params[0].x.mode != Mode::Exec
        || data.params[0].x.user_mut
        || data.inner_ret.x.mode != Mode::Exec
        || data.outer_ret.x.mode != Mode::Exec
        || !data.require.is_empty()
        || !data.ensure.1.is_empty()
        || !crate::ast_util::types_equal(&data.inner_ret.x.typ, &data.outer_ret.x.typ)
    {
        return false;
    }
    if models::path_is(&data.name.path, CrateId::Core, &["convert", boundary, method]) {
        return matches!(data.kind, FunctionKind::TraitMethodDecl { .. })
            && data.ensure.0.len() == 1
            && data.proxy.as_ref().is_some_and(|proxy| {
                models::path_is(
                    &proxy.x,
                    CrateId::Vstd,
                    &["std_specs", "convert", extension, method],
                )
            });
    }
    let path = &data.name.path;
    let source = Arc::new(TypX::TypParam(data.typ_params[0].clone()));
    let destination = Arc::new(TypX::TypParam(data.typ_params[1].clone()));
    let same = crate::ast_util::types_equal;
    if path.krate != CrateId::Core
        || path.segments.len() != 3
        || path.segments[0].as_str() != "convert"
        || !path.segments[1].starts_with("impl")
        || path.segments[2].as_str() != method
        || !data.proxy.as_ref().is_some_and(|proxy| {
            models::spec_proxy(
                &proxy.x,
                &["std_specs", "convert"],
                if fallible { "try__into" } else { "into" },
            )
        })
        || !same(&data.params[0].x.typ, &source)
        || !matches!(&data.kind, FunctionKind::TraitMethodImpl {
            method: declaration, impl_path, trait_path, trait_typ_args, inherit_body_from: None,
        } | FunctionKind::ForeignTraitMethodImpl {
            method: declaration, impl_path, trait_path, trait_typ_args,
        } if models::path_is(&declaration.path, CrateId::Core, &["convert", boundary, method])
            && models::path_is(trait_path, CrateId::Core, &["convert", boundary])
            && impl_path.krate == CrateId::Core && impl_path.segments.as_slice() == &path.segments[..2]
            && trait_typ_args.len() == 2 && same(&trait_typ_args[0], &source) && same(&trait_typ_args[1], &destination))
    {
        return false;
    }
    let mut sized = 0;
    let mut conversions = 0;
    for bound in data.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), types)
                if types.len() == 1
                    && (same(&types[0], &source) || same(&types[0], &destination)) =>
            {
                sized += 1
            }
            GenericBoundX::Trait(TraitId::Path(path), types)
                if models::path_is(path, CrateId::Core, &["convert", conversion])
                    && types.len() == 2
                    && same(&types[0], &destination)
                    && same(&types[1], &source) =>
            {
                conversions += 1
            }
            _ => return false,
        }
    }
    let output = if fallible {
        matches!(&*data.inner_ret.x.typ, TypX::Datatype(Dt::Path(path), types, implementations)
            if models::path_is(path, CrateId::Core, &["result", "Result"])
                && implementations.is_empty() && types.len() == 2 && same(&types[0], &destination)
                && matches!(&*types[1], TypX::Projection { trait_typ_args, trait_path, name }
                    if models::path_is(trait_path, CrateId::Core, &["convert", "TryFrom"])
                        && name.as_str() == "Error" && trait_typ_args.len() == 2
                        && same(&trait_typ_args[0], &destination) && same(&trait_typ_args[1], &source)))
    } else {
        same(&data.inner_ret.x.typ, &destination)
    };
    sized == 2 && conversions == 1 && output
}

pub(super) fn model(function: &Function) -> bool {
    model_kind(function, false) || model_kind(function, true)
}

pub(super) fn forwarding<'a>(
    expression: &'a Expr,
    functions: &HashMap<Fun, Function>,
    fallible: bool,
) -> Option<(&'a Expr, &'a Typs, &'a ImplPaths)> {
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
    let declaration = functions.get(boundary).filter(|function| model_kind(function, fallible))?;
    let blanket = functions.get(resolved).filter(|function| model_kind(function, fallible))?;
    let (method, impl_path) = match &blanket.x.kind {
        FunctionKind::TraitMethodImpl { method, impl_path, .. }
        | FunctionKind::ForeignTraitMethodImpl { method, impl_path, .. } => (method, impl_path),
        _ => return None,
    };
    let same = crate::ast_util::types_equal;
    if !matches!(declaration.x.kind, FunctionKind::TraitMethodDecl { .. })
        || method != boundary
        || types.len() != 2
        || args.len() != 1
        || resolved_types.len() != 2
        || !resolved_types.iter().zip(types.iter()).all(|(left, right)| same(left, right))
        || implementations.len() != impl_paths.len() + 1
        || impl_paths.is_empty()
        || !matches!(&implementations[0], ImplPath::TraitImplPath(path) if path == impl_path)
        || implementations[1..] != impl_paths[..]
    {
        return None;
    }
    Some((&args[0], types, impl_paths))
}

fn dispatch<'a>(
    expression: &'a Expr,
    functions: &HashMap<Fun, Function>,
    fallible: bool,
) -> Option<(&'a Expr, Fun)> {
    let (operand, types, impl_paths) = forwarding(expression, functions, fallible)?;
    let same = crate::ast_util::types_equal;
    if impl_paths.len() != 1
        || !same(&operand.typ, &types[0])
        || !matches!(&*types[0], TypX::Int(range) if arithmetic::exact_range(*range))
        || !matches!(&*types[1], TypX::Int(range) if arithmetic::exact_range(*range))
    {
        return None;
    }
    let ImplPath::TraitImplPath(conversion_path) = &impl_paths[0] else { return None };
    if !fallible
        && same(&types[0], &types[1])
        && same(&expression.typ, &types[1])
        && conversion_path.krate == CrateId::Core
        && conversion_path.segments.len() == 2
        && conversion_path.segments[0].as_str() == "convert"
        && conversion_path.segments[1].starts_with("impl")
    {
        let declaration = functions.values().find(|function| {
            widening::model(function)
                && models::path_is(
                    &function.x.name.path,
                    CrateId::Core,
                    &["convert", "From", "from"],
                )
        })?;
        return Some((operand, declaration.x.name.clone()));
    }
    let mut conversions = functions.values().filter(|function| {
        (if fallible { integer_conversions::conversion_model(function) } else { widening::model(function) })
            && matches!(&function.x.kind, FunctionKind::TraitMethodImpl { impl_path, .. } | FunctionKind::ForeignTraitMethodImpl { impl_path, .. } if impl_path == conversion_path)
    });
    let conversion = conversions.next()?;
    if conversions.next().is_some() {
        return None;
    }
    let (output, input) = match &conversion.x.kind {
        FunctionKind::TraitMethodImpl { trait_typ_args, .. }
        | FunctionKind::ForeignTraitMethodImpl { trait_typ_args, .. }
            if trait_typ_args.len() == 2 =>
        {
            (&trait_typ_args[0], &trait_typ_args[1])
        }
        _ => return None,
    };
    (same(&types[0], input)
        && same(&types[1], output)
        && same(&operand.typ, &conversion.x.params[0].x.typ)
        && same(&expression.typ, &conversion.x.inner_ret.x.typ))
    .then(|| (operand, conversion.x.name.clone()))
}

pub(super) fn operand<'a>(
    expression: &'a Expr,
    functions: &HashMap<Fun, Function>,
    fallible: bool,
) -> Option<&'a Expr> {
    dispatch(expression, functions, fallible).map(|(operand, _)| operand)
}

pub(super) fn dependencies(expression: &Expr, functions: &HashMap<Fun, Function>) -> Vec<Fun> {
    [false, true]
        .into_iter()
        .filter_map(|fallible| dispatch(expression, functions, fallible).map(|(_, name)| name))
        .collect()
}
