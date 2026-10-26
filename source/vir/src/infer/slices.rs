//! Immutable shared slices, expressed with Verus's existing length/index IR.
//! No enumeration or length cap is needed: each read remains a symbolic term.
use super::*;

pub(super) fn element(typ: &Typ) -> Option<Typ> {
    match &**arrays::readonly_base(typ) {
        TypX::Primitive(Primitive::Slice, args) if args.len() == 1 && scalar(&args[0]) => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

pub(super) fn input_type(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Decorate(TypDecoration::Ref, None, inner)
        if matches!(&**inner, TypX::Primitive(Primitive::Slice, _)) && element(inner).is_some())
}

pub(super) fn compatible(a: &Typ, b: &Typ) -> bool {
    match (element(a), element(b)) {
        (Some(a), Some(b)) => crate::ast_util::types_equal(&a, &b),
        _ => false,
    }
}

/// Array-to-slice coercions can keep the array's exact symbolic value during
/// substitution. Length and indexing then specialize to the original array.
/// The reverse direction is never valid.
pub(super) fn substitutes(actual: &Typ, expected: &Typ) -> bool {
    match (arrays::shape(actual), element(expected)) {
        (Some((a, _)), Some(b)) => crate::ast_util::types_equal(&a, &b),
        _ => false,
    }
}

pub(super) fn length(span: &Span, base: Term) -> Result<Term, VirErr> {
    if let Some((_, n)) = arrays::shape(&base.expr.typ) {
        return term(span, &int_typ(), ExprX::Const(Constant::Int(n.into())), 1);
    }
    if element(&base.expr.typ).is_none() {
        return Err(unsupported(span, "length requires an immutable scalar slice or array"));
    }
    term(
        span,
        &int_typ(),
        ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), base.expr),
        1 + base.size,
    )
}

pub(super) fn index(span: &Span, base: Term, index: Term) -> Result<Term, VirErr> {
    if arrays::shape(&base.expr.typ).is_some() {
        return arrays::index(span, base, index);
    }
    let typ = element(&base.expr.typ)
        .ok_or_else(|| unsupported(span, "indexing requires an immutable scalar slice"))?;
    if !eligibility::integer(&index.expr.typ) {
        return Err(unsupported(span, "noninteger slice index"));
    }
    term(
        span,
        &typ,
        ExprX::Binary(BinaryOp::Index(ArrayKind::Slice, BoundsCheck::Allow), base.expr, index.expr),
        1 + base.size + index.size,
    )
}

pub(super) fn bound(span: &Span, base: Term, index: Term) -> Result<Term, VirErr> {
    let zero = term(span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
    let lower = summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), zero, index.clone())?;
    let upper =
        summary::binary(span, BinaryOp::Inequality(InequalityOp::Lt), index, length(span, base)?)?;
    logical(span, LogicalOp::And, lower, upper)
}

/// Canonical spec declarations used implicitly by ordinary slice bounds/read
/// lowering. Their full imported artifact (including axioms) is fingerprinted.
pub(super) fn model(f: &Function) -> bool {
    let f = &f.x;
    let len = f.name == crate::def::fn_slice_len();
    let index = f.name == crate::def::fn_slice_index();
    if !(len || index)
        || f.mode != Mode::Spec
        || !matches!(f.kind, FunctionKind::Static)
        || !matches!(f.item_kind, ItemKind::Function)
        || f.typ_params.len() != 1
        || f.body.is_some()
        || f.proxy.is_some()
        || f.attrs.infer
        || f.params.len() != if len { 1 } else { 2 }
        || f.params.iter().any(|p| p.x.mode != Mode::Spec || p.x.user_mut)
        || !f.require.is_empty()
        || !f.ensure.0.is_empty()
        || !f.ensure.1.is_empty()
    {
        return false;
    }
    let t = Arc::new(TypX::TypParam(f.typ_params[0].clone()));
    let slice = Arc::new(TypX::Decorate(
        TypDecoration::Ref,
        None,
        Arc::new(TypX::Primitive(Primitive::Slice, Arc::new(vec![t.clone()]))),
    ));
    let output = if len { Arc::new(TypX::Int(IntRange::USize)) } else { t.clone() };
    f.typ_bounds.len() == 1
        && matches!(&*f.typ_bounds[0], GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args)
            if args.len()==1 && crate::ast_util::types_equal(&args[0], &t))
        && crate::ast_util::types_equal(&f.params[0].x.typ, &slice)
        && (len || matches!(&*f.params[1].x.typ, TypX::Int(IntRange::Int)))
        && f.inner_ret.x.mode == Mode::Spec
        && f.outer_ret.x.mode == Mode::Spec
        && crate::ast_util::types_equal(&f.inner_ret.x.typ, &output)
        && crate::ast_util::types_equal(&f.outer_ret.x.typ, &output)
}

pub(super) fn dependencies(e: &Expr, functions: &HashMap<Fun, Function>) -> Vec<Fun> {
    fn indexed(p: &Place) -> bool {
        match &p.x {
            PlaceX::Index(_, _, ArrayKind::Slice, _) => true,
            PlaceX::Index(base, _, _, _) | PlaceX::Field(_, base) => indexed(base),
            _ => false,
        }
    }
    let index = matches!(&e.x, ExprX::ReadPlace(p, _) if indexed(p))
        || matches!(e.x, ExprX::Binary(BinaryOp::Index(ArrayKind::Slice, _), ..));
    if index {
        vec![crate::def::fn_slice_len(), crate::def::fn_slice_index()]
    } else if matches!(e.x, ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), _))
        || matches!(&e.x, ExprX::Call { target: CallTarget::Fun(_, name, ..), args, .. }
            if functions.get(name).and_then(models::array_model) == Some(models::ArrayModel::Len)
                && args.len()==1 && models::array_call(&args[0], functions).ok().flatten() != Some(models::ArrayModel::AsSlice))
    {
        vec![crate::def::fn_slice_len()]
    } else {
        vec![]
    }
}

#[cfg(test)]
pub(super) mod tests;
