use super::*;

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

fn error_type(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Datatype(Dt::Path(path), types, impls)
        if models::path_is(path, CrateId::Core, &["array", "TryFromSliceError"])
            && types.is_empty() && impls.is_empty())
}

pub(super) fn model(function: &Function) -> bool {
    let data = &function.x;
    let path = &data.name.path;
    if path.krate != CrateId::Core
        || path.segments.len() != 3
        || path.segments[0].as_str() != "array"
        || !path.segments[1].starts_with("impl")
        || path.segments[2].as_str() != "try_from"
        || !data.proxy.as_ref().is_some_and(|proxy| {
            models::spec_proxy(&proxy.x, &["std_specs", "chunks_exact"], "try__from")
        })
        || data.mode != Mode::Exec
        || !matches!(data.item_kind, ItemKind::Function)
        || data.attrs.infer
        || data.attrs.is_unsafe
        || data.attrs.is_async
        || data.body.is_some()
        || !data.require.is_empty()
        || data.typ_params.len() != 2
        || data.params.len() != 1
        || data.params[0].x.mode != Mode::Exec
        || data.params[0].x.user_mut
        || data.inner_ret.x.mode != Mode::Exec
        || data.outer_ret.x.mode != Mode::Exec
        || data.ensure.0.len() != 2
        || !data.ensure.1.is_empty()
    {
        return false;
    }
    let same = crate::ast_util::types_equal;
    let element = Arc::new(TypX::TypParam(data.typ_params[0].clone()));
    let length = Arc::new(TypX::TypParam(data.typ_params[1].clone()));
    let array = models::primitive(Primitive::Array, vec![element.clone(), length.clone()]);
    let slice = models::reference(models::primitive(Primitive::Slice, vec![element.clone()]));
    let mut sized = 0;
    let mut copy = 0;
    let mut constant = 0;
    for bound in data.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), types)
                if types.len() == 1 && same(&types[0], &element) =>
            {
                sized += 1
            }
            GenericBoundX::Trait(TraitId::Path(path), types)
                if models::path_is(path, CrateId::Core, &["marker", "Copy"])
                    && types.len() == 1
                    && same(&types[0], &element) =>
            {
                copy += 1
            }
            GenericBoundX::ConstTyp(actual, typ)
                if same(actual, &length) && matches!(&**typ, TypX::Int(IntRange::USize)) =>
            {
                constant += 1
            }
            _ => return false,
        }
    }
    sized == 1
        && copy == 1
        && constant == 1
        && same(&data.params[0].x.typ, &slice)
        && same(&data.inner_ret.x.typ, &data.outer_ret.x.typ)
        && result(&data.inner_ret.x.typ)
            .is_some_and(|(output, error)| same(output, &array) && error_type(error))
        && matches!(&data.kind, FunctionKind::TraitMethodImpl {
            method, impl_path, trait_path, trait_typ_args, inherit_body_from: None,
        } | FunctionKind::ForeignTraitMethodImpl { method, impl_path, trait_path, trait_typ_args }
            if models::path_is(&method.path, CrateId::Core, &["convert", "TryFrom", "try_from"])
                && models::path_is(trait_path, CrateId::Core, &["convert", "TryFrom"])
                && impl_path.krate == CrateId::Core && impl_path.segments.as_slice() == &path.segments[..2]
                && trait_typ_args.len() == 2 && same(&trait_typ_args[0], &array) && same(&trait_typ_args[1], &slice))
}

fn core_impl(implementation: &ImplPath, module: &str) -> bool {
    matches!(implementation, ImplPath::TraitImplPath(path) if path.krate == CrateId::Core
        && path.segments.len() == 2 && path.segments[0].as_str() == module && path.segments[1].starts_with("impl"))
}

pub(super) fn conversion<'a>(
    expression: &'a Expr,
    functions: &HashMap<Fun, Function>,
) -> Option<(&'a Expr, Fun)> {
    let mut direct = None;
    let (operand, types, implementations) =
        if let Some(parts) = numeric_into::forwarding(expression, functions, true) {
            parts
        } else {
            let ExprX::Call {
                target:
                    CallTarget::Fun(
                        CallTargetKind::DynamicResolved {
                            resolved,
                            typs,
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
            if !functions.get(boundary).is_some_and(integer_conversions::conversion_model)
                || !functions.get(resolved).is_some_and(model)
                || args.len() != 1
                || types.len() != 2
                || typs.len() != 2
                || impl_paths.len() != 1
                || implementations.len() != 2
                || implementations[1] != impl_paths[0]
            {
                return None;
            }
            let (element, length) = arrays::shape(&types[0])?;
            if !crate::ast_util::types_equal(&typs[0], &element)
                || !matches!(&*typs[1], TypX::ConstInt(actual) if *actual == BigInt::from(length))
            {
                return None;
            }
            direct = Some(resolved);
            (&args[0], types, implementations)
        };
    if types.len() != 2 || implementations.len() != 2 || !core_impl(&implementations[1], "marker") {
        return None;
    }
    let (output, error) = result(&expression.typ)?;
    let (input, destination) =
        if slices::input_type(&types[0]) { (&types[0], &types[1]) } else { (&types[1], &types[0]) };
    let (element, _) = arrays::shape(destination)?;
    if !error_type(error)
        || !crate::ast_util::types_equal(output, destination)
        || !slices::input_type(input)
        || !slices::element(input)
            .is_some_and(|actual| crate::ast_util::types_equal(&actual, &element))
        || !crate::ast_util::types_equal(
            arrays::readonly_base(&operand.typ),
            arrays::readonly_base(input),
        )
    {
        return None;
    }
    let mut candidates = functions.values().filter(|function| model(function)
        && matches!(&function.x.kind, FunctionKind::TraitMethodImpl { impl_path, .. } | FunctionKind::ForeignTraitMethodImpl { impl_path, .. }
            if matches!(&implementations[0], ImplPath::TraitImplPath(actual) if actual == impl_path)));
    let matched = candidates.next()?;
    if candidates.next().is_some() || direct.is_some_and(|name| name != &matched.x.name) {
        return None;
    }
    Some((operand, matched.x.name.clone()))
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
    if !functions.get(name).is_some_and(integer_conversions::unwrap_model)
        || args.len() != 1
        || !maximality_model(functions.get(name)?, types)
        || implementations.len() != 1
        || !core_impl(&implementations[0], "array")
    {
        return None;
    }
    let (output, error) = result(&args[0].typ)?;
    if !crate::ast_util::types_equal(output, &expression.typ)
        || !crate::ast_util::types_equal(output, &types[0])
        || !crate::ast_util::types_equal(error, &types[1])
    {
        return None;
    }
    conversion(&args[0], functions).map(|(operand, _)| operand)
}

pub(super) fn value(span: &Span, input: Term, output: &Typ) -> Result<(Term, Term), VirErr> {
    let (_, length) =
        arrays::shape(output).ok_or_else(|| unsupported(span, "slice conversion array shape"))?;
    let count = term(span, &int_typ(), ExprX::Const(Constant::Int(length.into())), 1)?;
    let safe = summary::binary(
        span,
        BinaryOp::Eq(Mode::Spec),
        slices::length(span, input.clone())?,
        count,
    )?;
    let elements = (0..length)
        .map(|index| {
            slices::index(
                span,
                input.clone(),
                term(span, &int_typ(), ExprX::Const(Constant::Int(index.into())), 1)?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((safe, arrays::literal(span, output, elements)?))
}

pub(super) fn maximality_model(function: &Function, types: &Typs) -> bool {
    integer_conversions::unwrap_model(function)
        && types.len() == 2
        && arrays::shape(&types[0]).is_some()
        && error_type(&types[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate_with, function, input, span};

    fn path(krate: CrateId, segments: &[&str]) -> Path {
        Arc::new(PathX {
            krate,
            segments: Arc::new(
                segments.iter().map(|segment| Arc::new((*segment).into())).collect(),
            ),
        })
    }

    fn canonical() -> Function {
        let original = function("candidate", input("x"));
        let mut data = original.x.clone();
        data.name =
            Arc::new(FunX { path: path(CrateId::Core, &["array", "impl_test", "try_from"]) });
        data.proxy = Some(crate::def::Spanned {
            span: span(),
            x: path(
                CrateId::Vstd,
                &["std_specs", "chunks_exact", "_verus_external_fn_specification_test_try__from"],
            ),
        });
        data.typ_params = Arc::new(vec![Arc::new("T".into()), Arc::new("N".into())]);
        let element = Arc::new(TypX::TypParam(data.typ_params[0].clone()));
        let length = Arc::new(TypX::TypParam(data.typ_params[1].clone()));
        let array = models::primitive(Primitive::Array, vec![element.clone(), length.clone()]);
        let slice = models::reference(models::primitive(Primitive::Slice, vec![element.clone()]));
        data.typ_bounds = Arc::new(vec![
            Arc::new(GenericBoundX::Trait(
                TraitId::Sizedness(Sizedness::Sized),
                Arc::new(vec![element.clone()]),
            )),
            Arc::new(GenericBoundX::Trait(
                TraitId::Path(path(CrateId::Core, &["marker", "Copy"])),
                Arc::new(vec![element]),
            )),
            Arc::new(GenericBoundX::ConstTyp(length, Arc::new(TypX::Int(IntRange::USize)))),
        ]);
        data.kind = FunctionKind::ForeignTraitMethodImpl {
            method: Arc::new(FunX {
                path: path(CrateId::Core, &["convert", "TryFrom", "try_from"]),
            }),
            impl_path: path(CrateId::Core, &["array", "impl_test"]),
            trait_path: path(CrateId::Core, &["convert", "TryFrom"]),
            trait_typ_args: Arc::new(vec![array.clone(), slice.clone()]),
        };
        let mut parameter = data.params[0].x.clone();
        parameter.typ = slice;
        data.params = Arc::new(vec![data.params[0].new_x(parameter)]);
        let mut output = data.inner_ret.x.clone();
        output.typ = Arc::new(TypX::Datatype(
            Dt::Path(path(CrateId::Core, &["result", "Result"])),
            Arc::new(vec![
                array,
                Arc::new(TypX::Datatype(
                    Dt::Path(path(CrateId::Core, &["array", "TryFromSliceError"])),
                    Arc::new(vec![]),
                    Arc::new(vec![]),
                )),
            ]),
            Arc::new(vec![]),
        ));
        data.inner_ret = data.inner_ret.new_x(output);
        data.outer_ret = data.inner_ret.clone();
        data.body = None;
        data.ensure = (Arc::new(vec![boolean(&span(), true).expr; 2]), Arc::new(vec![]));
        data.attrs = Arc::new(FunctionAttrsX::default());
        original.new_x(data)
    }

    #[test]
    fn native_identity_proxy_signature_bounds_and_policy_are_closed() {
        let original = canonical();
        assert!(model(&original));
        for mutation in 0..16 {
            let mut data = original.x.clone();
            match mutation {
                0 => data.proxy = None,
                1 => {
                    data.name = Arc::new(FunX {
                        path: path(CrateId::Core, &["slice", "impl_test", "try_from"]),
                    })
                }
                2 => {
                    data.name = Arc::new(FunX {
                        path: path(
                            CrateId::Id(Arc::new("fake".into()), 1),
                            &["array", "impl_test", "try_from"],
                        ),
                    })
                }
                3 => data.mode = Mode::Spec,
                4 => data.body = Some(input("x")),
                5 => data.require = Arc::new(vec![boolean(&span(), false).expr]),
                6 => data.ensure.0 = Arc::new(vec![]),
                7 => data.ensure.1 = Arc::new(vec![boolean(&span(), true).expr]),
                8 => data.typ_bounds = Arc::new(data.typ_bounds[..2].to_vec()),
                9 => data.typ_params = Arc::new(vec![]),
                10 => data.kind = FunctionKind::Static,
                11 => Arc::make_mut(&mut data.attrs).infer = true,
                12 => Arc::make_mut(&mut data.attrs).is_unsafe = true,
                13 => Arc::make_mut(&mut data.attrs).is_async = true,
                14 => data.params = Arc::new(vec![]),
                _ => data.outer_ret = data.params[0].clone(),
            }
            assert!(!model(&original.new_x(data)), "guard {mutation}");
        }
    }

    #[test]
    fn symbolic_lengths_and_all_scalar_elements_match_native_conversion() {
        for length in [0, 1, 4, 8, 32, 64] {
            let typ = arrays::typ(Arc::new(TypX::Int(IntRange::U(8))), length).unwrap();
            let slice = models::reference(models::primitive(
                Primitive::Slice,
                vec![Arc::new(TypX::Int(IntRange::U(8)))],
            ));
            let operand = Term {
                expr: SpannedTyped::new(&span(), &slice, input("x").x.clone()),
                size: 1,
                view: None,
            };
            let (safe, output) = value(&span(), operand, &typ).unwrap();
            for actual in [0, 1, 7, 8, 9, 31, 32, 63, 64, 65, 128] {
                for byte in 0..=255 {
                    let expected = Value::Array(
                        (0..actual).map(|index| Value::Int((byte + index as i128) % 256)).collect(),
                    );
                    assert_eq!(
                        evaluate_with(&safe.expr, &|_| expected.clone()),
                        Value::Bool(actual == length)
                    );
                    if actual == length {
                        assert_eq!(evaluate_with(&output.expr, &|_| expected.clone()), expected);
                    }
                }
            }
        }
    }
}
