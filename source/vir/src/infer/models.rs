//! Closed recognition of trusted standard models, separate from inferred callees.
//! Generic calls are allowed only at these identities and concrete scalar types.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArrayModel {
    AsSlice,
    Fill,
    Len,
}

fn path_is(path: &Path, krate: CrateId, segments: &[&str]) -> bool {
    path.krate == krate
        && path.segments.len() == segments.len()
        && path.segments.iter().zip(segments).all(|(a, b)| a.as_str() == *b)
}
fn reference(t: Typ) -> Typ {
    Arc::new(TypX::Decorate(TypDecoration::Ref, None, t))
}
fn primitive(p: Primitive, args: Vec<Typ>) -> Typ {
    Arc::new(TypX::Primitive(p, Arc::new(args)))
}

pub(super) fn array_model(function: &Function) -> Option<ArrayModel> {
    let f = &function.x;
    if f.mode != Mode::Exec
        || !matches!(f.kind, FunctionKind::Static)
        || !matches!(f.item_kind, ItemKind::Function)
        || !f.require.is_empty()
        || !matches!(f.unwind_spec, Some(UnwindSpec::NoUnwind))
        || f.params.len() != 1
        || f.params[0].x.mode != Mode::Exec
        || f.params[0].x.user_mut
        || f.attrs.is_unsafe
        || f.attrs.is_async
        || f.attrs.infer
        || f.body.is_some()
    {
        return None;
    }
    let kind = if path_is(&f.name.path, CrateId::Vstd, &["array", "array_as_slice"]) {
        ArrayModel::AsSlice
    } else if path_is(&f.name.path, CrateId::Vstd, &["array", "array_fill_for_copy_types"]) {
        ArrayModel::Fill
    } else {
        let p = &f.name.path;
        let proxy = &f.proxy.as_ref()?.x;
        if p.krate != CrateId::Core
            || p.segments.len() != 3
            || p.segments[0].as_str() != "slice"
            || !p.segments[1].starts_with("impl")
            || p.segments[2].as_str() != "len"
            || proxy.krate != CrateId::Vstd
            || proxy.segments.len() != 2
            || proxy.segments[0].as_str() != "slice"
            || !proxy.segments[1].starts_with("_verus_external_fn_specification_")
            || !proxy.segments[1].ends_with("_len")
        {
            return None;
        }
        ArrayModel::Len
    };
    let count = if kind == ArrayModel::Len { 1 } else { 2 };
    if f.typ_params.len() != count {
        return None;
    }
    let t = Arc::new(TypX::TypParam(f.typ_params[0].clone()));
    let n = (count == 2).then(|| Arc::new(TypX::TypParam(f.typ_params[1].clone())));
    let mut sized = 0;
    let mut constant = 0;
    let mut copy = 0;
    for bound in f.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args)
                if args.len() == 1 && crate::ast_util::types_equal(&args[0], &t) =>
            {
                sized += 1
            }
            GenericBoundX::ConstTyp(name, typ)
                if n.as_ref().is_some_and(|n| crate::ast_util::types_equal(name, n))
                    && arrays::index_type(typ) =>
            {
                constant += 1
            }
            GenericBoundX::Trait(TraitId::Path(path), args)
                if kind == ArrayModel::Fill
                    && path_is(path, CrateId::Core, &["marker", "Copy"])
                    && args.len() == 1
                    && crate::ast_util::types_equal(&args[0], &t) =>
            {
                copy += 1
            }
            _ => return None,
        }
    }
    if sized != 1
        || constant != usize::from(count == 2)
        || copy != usize::from(kind == ArrayModel::Fill)
    {
        return None;
    }
    let slice = reference(primitive(Primitive::Slice, vec![t.clone()]));
    let array = n.map(|n| primitive(Primitive::Array, vec![t.clone(), n]));
    let (input, output) = match kind {
        ArrayModel::AsSlice => (reference(array?), slice),
        ArrayModel::Fill => (t, array?),
        ArrayModel::Len => (slice, Arc::new(TypX::Int(IntRange::USize))),
    };
    if !crate::ast_util::types_equal(&f.params[0].x.typ, &input)
        || !crate::ast_util::types_equal(&f.inner_ret.x.typ, &output)
    {
        return None;
    }
    if kind != ArrayModel::Len && (f.proxy.is_some() || !f.attrs.is_external_body) {
        return None;
    }
    Some(kind)
}

/// Validate both the canonical declaration and its concrete call instantiation.
pub(super) fn array_call(
    expr: &Expr,
    functions: &HashMap<Fun, Function>,
) -> Result<Option<ArrayModel>, VirErr> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, name, typs, impls, _),
        args,
        post_args: None,
        body: None,
    } = &expr.x
    else {
        return Ok(None);
    };
    let Some(kind) = functions.get(name).and_then(array_model) else { return Ok(None) };
    let invalid = || unsupported(&expr.span, "unsupported instantiation of trusted array model");
    if !impls.is_empty() || args.len() != 1 || typs.is_empty() || !scalar(&typs[0]) {
        return Err(invalid());
    }
    let is_slice = |typ: &Typ| {
        matches!(&**typ, TypX::Decorate(TypDecoration::Ref, None, _))
            && matches!(&**arrays::readonly_base(typ), TypX::Primitive(Primitive::Slice, ts)
                if ts.len() == 1 && crate::ast_util::types_equal(&ts[0], &typs[0]))
    };
    match kind {
        ArrayModel::AsSlice | ArrayModel::Fill => {
            if typs.len() != 2 {
                return Err(invalid());
            }
            let typ = primitive(Primitive::Array, typs.as_ref().clone());
            let Some((_, n)) = arrays::shape(&typ) else { return Err(invalid()) };
            match kind {
                ArrayModel::AsSlice
                    if arrays::shape(&args[0].typ).is_some_and(|(t, m)| {
                        n == m && crate::ast_util::types_equal(&t, &typs[0])
                    }) && is_slice(&expr.typ) => {}
                ArrayModel::Fill
                    if crate::ast_util::types_equal(&args[0].typ, &typs[0])
                        && crate::ast_util::types_equal(&expr.typ, &typ) => {}
                _ => return Err(invalid()),
            }
        }
        ArrayModel::Len => {
            if typs.len() != 1 || !arrays::index_type(&expr.typ) || !is_slice(&args[0].typ) {
                return Err(invalid());
            }
        }
    }
    Ok(Some(kind))
}

impl Engine<'_> {
    pub(super) fn array_model_call(
        &mut self,
        expr: &Expr,
        kind: ArrayModel,
    ) -> Result<Summary, VirErr> {
        let ExprX::Call { args, .. } = &expr.x else { unreachable!() };
        let span = &expr.span;
        if kind == ArrayModel::Len {
            let array = self.expr(&args[0])?;
            let output = slices::length(span, value(&array, span)?)?;
            return Ok(Summary { safe: array.safe, value: Some(output) });
        }
        let arg = self.expr(&args[0])?;
        let output = if kind == ArrayModel::Fill {
            let (_, n) = arrays::shape(&expr.typ)
                .ok_or_else(|| unsupported(span, "unsupported repeated array"))?;
            Some(arrays::literal(span, &expr.typ, vec![value(&arg, span)?; n])?)
        } else {
            // Preserve the exact array value through a slice coercion. Summary
            // substitution specializes subsequent length/index operations.
            Some(value(&arg, span)?)
        };
        Ok(Summary { safe: arg.safe, value: output })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input, span};

    fn path(krate: CrateId, parts: &[&str]) -> Path {
        Arc::new(PathX {
            krate,
            segments: Arc::new(parts.iter().map(|s| Arc::new((*s).into())).collect()),
        })
    }
    fn model(kind: ArrayModel) -> Function {
        let base = function("model", input("x"));
        let mut f = base.x.clone();
        let t = Arc::new(TypX::TypParam(Arc::new("T".into())));
        let n = Arc::new(TypX::TypParam(Arc::new("N".into())));
        f.typ_params = Arc::new(vec![Arc::new("T".into())]);
        let mut bounds = vec![Arc::new(GenericBoundX::Trait(
            TraitId::Sizedness(Sizedness::Sized),
            Arc::new(vec![t.clone()]),
        ))];
        if kind != ArrayModel::Len {
            Arc::make_mut(&mut f.typ_params).push(Arc::new("N".into()));
            bounds.push(Arc::new(GenericBoundX::ConstTyp(
                n.clone(),
                Arc::new(TypX::Int(IntRange::USize)),
            )));
        }
        if kind == ArrayModel::Fill {
            bounds.push(Arc::new(GenericBoundX::Trait(
                TraitId::Path(path(CrateId::Core, &["marker", "Copy"])),
                Arc::new(vec![t.clone()]),
            )));
        }
        f.typ_bounds = Arc::new(bounds);
        let array = primitive(Primitive::Array, vec![t.clone(), n]);
        let slice = reference(primitive(Primitive::Slice, vec![t.clone()]));
        let (name, input, output) = match kind {
            ArrayModel::AsSlice => {
                (path(CrateId::Vstd, &["array", "array_as_slice"]), reference(array), slice)
            }
            ArrayModel::Fill => {
                (path(CrateId::Vstd, &["array", "array_fill_for_copy_types"]), t, array)
            }
            ArrayModel::Len => (
                path(CrateId::Core, &["slice", "impl_test", "len"]),
                slice,
                Arc::new(TypX::Int(IntRange::USize)),
            ),
        };
        f.name = Arc::new(FunX { path: name });
        if kind == ArrayModel::Len {
            f.proxy = Some(crate::def::Spanned {
                span: span(),
                x: path(CrateId::Vstd, &["slice", "_verus_external_fn_specification_test_len"]),
            });
        }
        let mut param = f.params[0].x.clone();
        param.typ = input;
        f.params = Arc::new(vec![f.params[0].new_x(param)]);
        let mut ret = f.inner_ret.x.clone();
        ret.typ = output;
        f.inner_ret = f.inner_ret.new_x(ret);
        f.outer_ret = f.inner_ret.clone();
        f.body = None;
        f.attrs = Arc::new(FunctionAttrsX { is_external_body: true, ..FunctionAttrsX::default() });
        f.unwind_spec = Some(UnwindSpec::NoUnwind);
        base.new_x(f)
    }
    fn call(f: &Function, types: Vec<Typ>, arg: Expr, output: Typ) -> Expr {
        SpannedTyped::new(
            &span(),
            &output,
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    f.x.name.clone(),
                    Arc::new(types),
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(vec![arg]),
                post_args: None,
                body: None,
            },
        )
    }
    #[test]
    fn array_model_identity_signature_and_instantiation_are_closed() {
        for kind in [ArrayModel::AsSlice, ArrayModel::Fill, ArrayModel::Len] {
            let model = model(kind);
            assert_eq!(array_model(&model), Some(kind));
            let mut f = model.x.clone();
            f.unwind_spec = None;
            assert!(array_model(&model.new_x(f)).is_none());
            let mut f = model.x.clone();
            f.name = Arc::new(FunX {
                path: path(
                    CrateId::Id(Arc::new("user".into()), 1),
                    &["array", "array_fill_for_copy_types"],
                ),
            });
            assert!(array_model(&model.new_x(f)).is_none());
        }
        let fill = model(ArrayModel::Fill);
        let functions = HashMap::from([(fill.x.name.clone(), fill.clone())]);
        let element = input("x").typ.clone();
        for length in [0, 4, 64] {
            let array = arrays::typ(element.clone(), length).unwrap();
            let call = call(
                &fill,
                vec![element.clone(), Arc::new(TypX::ConstInt(length.into()))],
                input("x"),
                array,
            );
            assert_eq!(array_call(&call, &functions).unwrap(), Some(ArrayModel::Fill));
        }
        let oversized =
            primitive(Primitive::Array, vec![element.clone(), Arc::new(TypX::ConstInt(65.into()))]);
        let call =
            call(&fill, vec![element, Arc::new(TypX::ConstInt(65.into()))], input("x"), oversized);
        assert!(array_call(&call, &functions).is_err());
    }

    #[test]
    fn zero_length_repeat_still_evaluates_its_operand_and_roundtrips_cache() {
        let fill = model(ArrayModel::Fill);
        let slice = model(ArrayModel::AsSlice);
        let len = model(ArrayModel::Len);
        let element = input("x").typ.clone();
        let types = vec![element.clone(), Arc::new(TypX::ConstInt(0.into()))];
        let one = SpannedTyped::new(&span(), &element, ExprX::Const(Constant::Int(1.into())));
        let operand = SpannedTyped::new(
            &span(),
            &element,
            ExprX::Binary(
                BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
                input("x"),
                one,
            ),
        );
        let array = call(&fill, types.clone(), operand, arrays::typ(element.clone(), 0).unwrap());
        let converted = call(
            &slice,
            types,
            array,
            reference(primitive(Primitive::Slice, vec![element.clone()])),
        );
        let length =
            call(&len, vec![element.clone()], converted, Arc::new(TypX::Int(IntRange::USize)));
        let body = SpannedTyped::new(
            &span(),
            &element,
            ExprX::Unary(UnaryOp::Clip { range: IntRange::U(8), truncate: true }, length),
        );
        let f = function("zero_repeat", body);
        let mut registry = registry::Registry::new(None);
        let imports = [fill, slice, len];
        let fresh = registry.infer_with_imports(&[f.clone()], &imports).unwrap();
        let cached = registry.infer_with_imports(&[f], &imports).unwrap();
        assert_eq!((registry.hits, registry.misses), (1, 1));
        assert_eq!(format_contract(&fresh[0]), format_contract(&cached[0]));
        for x in 0..=255u8 {
            let safe = crate::infer::tests::evaluate(&fresh[0].x.require[0], x, 0);
            assert_eq!(safe, crate::infer::tests::Value::Bool(x.checked_add(1).is_some()));
        }
    }
}
