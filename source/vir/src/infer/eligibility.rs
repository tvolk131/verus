//! Shared source eligibility for fresh inference and candidate-cache reuse.
use super::*;

pub(super) fn scalar(typ: &Typ) -> bool {
    matches!(
        &**typ,
        TypX::Bool
            | TypX::Int(IntRange::USize | IntRange::ISize)
            | TypX::Int(IntRange::U(8 | 16 | 32 | 64 | 128) | IntRange::I(8 | 16 | 32 | 64 | 128))
    )
}

pub(super) fn integer(typ: &Typ) -> bool {
    matches!(
        &**typ,
        TypX::Int(
            IntRange::Int
                | IntRange::USize
                | IntRange::ISize
                | IntRange::U(8 | 16 | 32 | 64 | 128)
                | IntRange::I(8 | 16 | 32 | 64 | 128)
        )
    )
}

pub(super) fn value_type(typ: &Typ, allow_approx: bool) -> bool {
    if allow_approx {
        scalar(typ)
    } else {
        enums::supported(typ) || arrays::input_type(typ) || slices::input_type(typ)
    }
}

pub(super) fn immutable_binding(binding: &PatternBinding) -> bool {
    binding.user_mut != Some(true)
        && matches!(binding.by_ref, ByRef::No)
        && value_type(&binding.typ, false)
}

fn readable_place(place: &Place, allow_approx: bool) -> bool {
    match &place.x {
        PlaceX::Temporary(expr)
            if matches!(&*place.typ, TypX::Decorate(TypDecoration::Never, None, t) if enums::unit(t))
                && crate::ast_util::types_equal(&expr.typ, &place.typ) =>
        {
            true
        }
        PlaceX::Local(_) | PlaceX::Temporary(_) => {
            value_type(&place.typ, allow_approx)
                || (!allow_approx && slices::element(&place.typ).is_some())
        }
        PlaceX::Field(op, base) => {
            !allow_approx && enums::valid_field(op, &base.typ) && readable_place(base, false)
        }
        PlaceX::Index(base, index, ArrayKind::Array, BoundsCheck::Error) => {
            !allow_approx
                && arrays::shape(&base.typ)
                    .is_some_and(|(element, _)| crate::ast_util::types_equal(&element, &place.typ))
                && arrays::index_type(&index.typ)
                && readable_place(base, false)
        }
        PlaceX::Index(base, index, ArrayKind::Slice, BoundsCheck::Error) => {
            !allow_approx
                && slices::element(&base.typ)
                    .is_some_and(|element| crate::ast_util::types_equal(&element, &place.typ))
                && arrays::index_type(&index.typ)
                && readable_place(base, false)
        }
        _ => false,
    }
}

/// Executable arithmetic policy and its mathematical summary operation.
/// Both eligibility checking and exact synthesis must use this mapping.
pub(super) fn source_binary(op: BinaryOp) -> Option<(BinaryOp, Option<IntRange>, bool)> {
    let arithmetic = |op| BinaryOp::Arith(op);
    Some(match op {
        BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(
            r @ (IntRange::USize
            | IntRange::ISize
            | IntRange::U(8 | 16 | 32 | 64 | 128)
            | IntRange::I(8 | 16 | 32 | 64 | 128)),
        ))) => (arithmetic(ArithOp::Add(OverflowBehavior::Allow)), Some(r), false),
        BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(
            r @ (IntRange::USize
            | IntRange::ISize
            | IntRange::U(8 | 16 | 32 | 64 | 128)
            | IntRange::I(8 | 16 | 32 | 64 | 128)),
        ))) => (arithmetic(ArithOp::Sub(OverflowBehavior::Allow)), Some(r), false),
        BinaryOp::Arith(ArithOp::Mul(OverflowBehavior::Error(
            r @ (IntRange::USize
            | IntRange::ISize
            | IntRange::U(8 | 16 | 32 | 64 | 128)
            | IntRange::I(8 | 16 | 32 | 64 | 128)),
        ))) => (arithmetic(ArithOp::Mul(OverflowBehavior::Allow)), Some(r), false),
        BinaryOp::Arith(ArithOp::EuclideanDiv(Div0Behavior::Error)) => {
            (arithmetic(ArithOp::EuclideanDiv(Div0Behavior::Allow)), None, true)
        }
        BinaryOp::Arith(ArithOp::EuclideanMod(Div0Behavior::Error)) => {
            (arithmetic(ArithOp::EuclideanMod(Div0Behavior::Allow)), None, true)
        }
        BinaryOp::Arith(ArithOp::TruncatingDiv(
            Div0Behavior::Error,
            SignedDivEdgeCaseBehavior::Error(
                IntegerTypeBitwidth::Width(8 | 16 | 32 | 64 | 128)
                | IntegerTypeBitwidth::ArchWordSize,
            ),
        )) => (
            arithmetic(ArithOp::TruncatingDiv(
                Div0Behavior::Allow,
                SignedDivEdgeCaseBehavior::Allow,
            )),
            None,
            true,
        ),
        BinaryOp::Arith(ArithOp::TruncatingMod(
            Div0Behavior::Error,
            SignedDivEdgeCaseBehavior::Error(
                IntegerTypeBitwidth::Width(8 | 16 | 32 | 64 | 128)
                | IntegerTypeBitwidth::ArchWordSize,
            ),
        )) => (
            arithmetic(ArithOp::TruncatingMod(
                Div0Behavior::Allow,
                SignedDivEdgeCaseBehavior::Allow,
            )),
            None,
            true,
        ),
        BinaryOp::Eq(Mode::Spec | Mode::Exec)
        | BinaryOp::Ne
        | BinaryOp::Inequality(_)
        | BinaryOp::BoolAndNoSC
        | BinaryOp::BoolOrNoSC
        | BinaryOp::BoolXor => (op, None, false),
        _ => return None,
    })
}

pub(super) fn validate_signature(function: &Function) -> Result<(), VirErr> {
    let f = &function.x;
    let span = &function.span;
    if f.mode != Mode::Exec
        || !matches!(f.kind, FunctionKind::Static)
        || !matches!(f.item_kind, ItemKind::Function)
        || !f.typ_params.is_empty()
        || !f.typ_bounds.is_empty()
        || f.attrs.is_async
        || f.attrs.is_unsafe
        || f.attrs.is_external_body
        || f.proxy.is_some()
        || f.attrs.autospec.is_some()
        || f.attrs.bit_vector
        || f.attrs.integer_ring
        || f.attrs.broadcast_forall
    {
        return Err(unsupported(
            span,
            "only safe, nongeneric executable functions with visible bodies are eligible",
        ));
    }
    if !f.require.is_empty()
        || !f.ensure.0.is_empty()
        || !f.ensure.1.is_empty()
        || f.returns.is_some()
        || !f.decrease.is_empty()
        || f.decrease_when.is_some()
        || f.decrease_by.is_some()
    {
        return Err(unsupported(
            span,
            "combining inference with handwritten contracts or decreases clauses",
        ));
    }
    if !(enums::supported(&f.inner_ret.x.typ)
        || matches!(&*f.inner_ret.x.typ, TypX::Primitive(Primitive::Array, _))
            && arrays::shape(&f.inner_ret.x.typ).is_some())
        || f.params
            .iter()
            .any(|p| !value_type(&p.x.typ, false) || p.x.mode != Mode::Exec || p.x.user_mut)
    {
        return Err(unsupported(
            span,
            "parameters and return values must be bool, usize/isize, fixed-width unsigned/signed integers (8/16/32/64/128 bits), or core Option/Result with scalar/unit payloads; immutable parameters may also be scalar arrays of length 0..=64 or shared references to them, or shared scalar slices (owned fixed scalar arrays may also be returned; slice returns are also unsupported)",
        ));
    }
    if !value_type(&f.inner_ret.x.typ, f.attrs.infer_allow_approx)
        || f.params.iter().any(|p| !value_type(&p.x.typ, f.attrs.infer_allow_approx))
    {
        return Err(unsupported(
            span,
            "sound interval inference requires scalar signatures; Option/Result inference currently requires exact mode, as do arrays and shared slices",
        ));
    }
    Ok(())
}

// Source eligibility is checked independently of cached formulas, so cache data
// cannot permit proof assumptions, effects, or unsupported control flow.
pub(super) fn validate_body(f: &Function, callees: &HashMap<Fun, Function>) -> Result<(), VirErr> {
    if loops::recognize(f)?.is_some() {
        return Ok(());
    }
    let body = f.x.body.as_ref().ok_or_else(|| unsupported(&f.span, "missing executable body"))?;
    let mut remaining = FORMULA_BUDGET;
    validate_root(body, f, callees, &mut remaining)
}

fn validate_root(
    body: &Expr,
    f: &Function,
    callees: &HashMap<Fun, Function>,
    remaining: &mut usize,
) -> Result<(), VirErr> {
    use crate::ast_visitor::{VisitorControlFlow, expr_visitor_dfs};
    let walked = expr_visitor_dfs(
        body,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut |_, e| {
            let checked = (|| {
                if !f.x.attrs.infer_allow_approx {
                    if let Some(range) = slices::range_call(e, callees)? {
                        crate::ast_visitor::expr_visitor_check(e, &mut |_, _| {
                            *remaining = remaining
                                .checked_sub(1)
                                .ok_or_else(|| unsupported(&e.span, "analysis budget exceeded"))?;
                            Ok::<_, VirErr>(())
                        })?;
                        validate_root(&range.base, f, callees, remaining)?;
                        for (_, field) in range.fields {
                            validate_root(&field, f, callees, remaining)?;
                        }
                        return Ok(true);
                    }
                    if let Some(p) = propagation::recognize(e, callees, &f.x.inner_ret.x.typ)? {
                        // Charge all lowered scaffolding, but validate the operand through
                        // the ordinary source gate. No generated internal type is generally eligible.
                        crate::ast_visitor::expr_visitor_check(e, &mut |_, _| {
                            *remaining = remaining
                                .checked_sub(1)
                                .ok_or_else(|| unsupported(&e.span, "analysis budget exceeded"))?;
                            Ok::<_, VirErr>(())
                        })?;
                        validate_root(&p.operand, f, callees, remaining)?;
                        return Ok(true);
                    }
                }
                if *remaining == 0 {
                    return Err(unsupported(&e.span, "analysis budget exceeded"));
                }
                *remaining -= 1;
                match &e.x {
                    ExprX::Const(Constant::Bool(_) | Constant::Int(_))
                    | ExprX::Var(_)
                    | ExprX::NeverToAny(_)
                    | ExprX::If(..) => {}
                    ExprX::Ctor(dt, variant, fields, None)
                        if !f.x.attrs.infer_allow_approx
                            && enums::supported(&e.typ)
                            && matches!(&*e.typ, TypX::Datatype(actual, ..) if actual == dt) =>
                    {
                        if fields.len() > 1
                            || fields
                                .first()
                                .is_some_and(|f| f.name != crate::def::positional_field_ident(0))
                        {
                            return Err(unsupported(&e.span, "unsupported enum fields"));
                        }
                        let field =
                            fields.first().map(|f| Term { view: None, expr: f.a.clone(), size: 1 });
                        enums::ctor(&e.span, &e.typ, variant, field)?;
                    }
                    ExprX::Match(place, arms, irrefutable) if !f.x.attrs.infer_allow_approx => {
                        enums::validate_match(place, arms, *irrefutable)?
                    }
                    ExprX::Ctor(..) | ExprX::Match(..) => {
                        return Err(unsupported(
                            &e.span,
                            "enum expressions currently require exact inference",
                        ));
                    }
                    ExprX::ReadPlace(p, _) if readable_place(p, f.x.attrs.infer_allow_approx) => {}
                    ExprX::ArrayLiteral(elements)
                        if !f.x.attrs.infer_allow_approx
                            && arrays::shape(&e.typ).is_some_and(|(_, n)| n == elements.len()) => {}
                    ExprX::ArrayLiteral(_) => {
                        return Err(unsupported(
                            &e.span,
                            "array literals require exact mode and at most 64 scalar elements",
                        ));
                    }
                    ExprX::Binary(BinaryOp::Index(ArrayKind::Array, BoundsCheck::Error), a, i)
                        if !f.x.attrs.infer_allow_approx
                            && arrays::shape(&a.typ).is_some()
                            && arrays::index_type(&i.typ) => {}
                    ExprX::Binary(BinaryOp::Index(ArrayKind::Slice, BoundsCheck::Error), a, i)
                        if !f.x.attrs.infer_allow_approx
                            && slices::element(&a.typ).is_some()
                            && arrays::index_type(&i.typ) => {}
                    _ if arithmetic::arch_constant(e).is_some() => {}
                    ExprX::Unary(op @ UnaryOp::BitNot(_), a)
                        if bitwise::unary_valid(*op, &a.typ)
                            && crate::ast_util::types_equal(&e.typ, &a.typ) => {}
                    ExprX::Binary(op @ BinaryOp::Bitwise(..), a, b)
                        if bitwise::source(*op, &a.typ, &b.typ, &e.typ).is_some() => {}
                    ExprX::Unary(UnaryOp::Not, a) if matches!(&*a.typ, TypX::Bool) => {}
                    ExprX::Unary(UnaryOp::Clip { range, .. }, _)
                        if arithmetic::exact_range(*range) => {}
                    ExprX::Logical(..) => {}
                    ExprX::Binary(op, ..) if source_binary(*op).is_some() => {}
                    ExprX::Binary(..) if arithmetic::negative_literal(e).is_some() => {}
                    ExprX::Call { .. } if widening::call(e, callees).is_some() => {}
                    ExprX::Call { .. }
                        if !f.x.attrs.infer_allow_approx
                            && models::array_call(e, callees)?.is_some() => {}
                    ExprX::Call { .. }
                        if f.x.attrs.infer_allow_approx && !value_type(&e.typ, true) =>
                    {
                        return Err(unsupported(
                            &e.span,
                            "enum expressions currently require exact inference",
                        ));
                    }
                    ExprX::Call {
                        target: CallTarget::Fun(CallTargetKind::Static, _, typs, impls, _),
                        post_args: None,
                        body: None,
                        ..
                    } if typs.is_empty() && impls.is_empty() => {}
                    ExprX::Block(stmts, _) => {
                        for s in stmts.iter() {
                            if *remaining == 0 {
                                return Err(error(
                                    &s.span,
                                    "Verus Infer incomplete: analysis budget exceeded",
                                ));
                            }
                            *remaining -= 1;
                            match &s.x {
                                StmtX::Expr(_) => {}
                                StmtX::Decl {
                                    pattern,
                                    mode: None | Some((Mode::Exec, DeclProph::Default)),
                                    init: Some(_),
                                    els,
                                    assert_irrefutable: false,
                                } => {
                                    if f.x.attrs.infer_allow_approx {
                                        if els.is_some()
                                            || !matches!(&pattern.x, PatternX::Var(b) if immutable_binding(b))
                                        {
                                            return Err(unsupported(
                                                &s.span,
                                                "rich patterns and let-else require exact mode",
                                            ));
                                        }
                                    } else {
                                        if matches!(&pattern.x,PatternX::Var(b) if !immutable_binding(b))
                                        {
                                            return Err(unsupported(
                                                &s.span,
                                                "mutable or nonscalar locals",
                                            ));
                                        }
                                        if els.is_none()
                                            && !matches!(
                                                &pattern.x,
                                                PatternX::Var(_) | PatternX::Wildcard
                                            )
                                        {
                                            return Err(unsupported(
                                                &s.span,
                                                "refutable binding requires let-else",
                                            ));
                                        }
                                        enums::validate_pattern(pattern)?;
                                    }
                                }
                                _ => {
                                    return Err(unsupported(
                                        &s.span,
                                        "uninitialized bindings or let-else",
                                    ));
                                }
                            }
                        }
                    }
                    ExprX::Loop { .. } => return Err(unsupported(&e.span, "loops")),
                    ExprX::Return(_) if !f.x.attrs.infer_allow_approx => {}
                    ExprX::Return(_) => {
                        return Err(unsupported(
                            &e.span,
                            "early returns currently require exact mode",
                        ));
                    }
                    ExprX::AssertAssume { .. } | ExprX::Ghost { .. } | ExprX::AssertBy { .. } => {
                        return Err(unsupported(
                            &e.span,
                            "proof statements and assumptions in inferred bodies",
                        ));
                    }
                    ExprX::Assign { .. } | ExprX::BorrowMut(_) => {
                        return Err(unsupported(&e.span, "mutable state"));
                    }
                    _ => {
                        return Err(unsupported(
                            &e.span,
                            "expression outside the scalar loop-free subset",
                        ));
                    }
                }
                Ok::<_, VirErr>(false)
            })();
            match checked {
                Ok(true) => VisitorControlFlow::Return,
                Ok(false) => VisitorControlFlow::Recurse,
                Err(err) => VisitorControlFlow::Stop(err),
            }
        },
    );
    match walked {
        VisitorControlFlow::Stop(err) => Err(err),
        _ => Ok(()),
    }
}

/// These conditions are the existing, shared entry restrictions of the two loop
/// profiles. Reporting a restriction never relaxes it or changes a proof query.
pub(super) fn loop_assistance(f: &Function) -> Option<super::report::AssistanceSkip> {
    use super::report::AssistanceSkip::*;
    if f.x.attrs.infer {
        Some(InferredFunction)
    } else if f.x.mode != Mode::Exec {
        Some(NonExecutable)
    } else if f.x.attrs.is_external_body {
        Some(ExternalBody)
    } else if f.x.attrs.is_unsafe || f.x.attrs.is_async {
        Some(UnsafeOrAsync)
    } else if f.x.attrs.exec_assume_termination || f.x.attrs.exec_allows_no_decreases_clause {
        Some(TerminationSuppressed)
    } else if !enums::unit(&f.x.inner_ret.x.typ) {
        Some(NonUnitReturn)
    } else if f.x.body.is_none() {
        Some(MissingBody)
    } else if f.x.params.iter().any(|p| p.x.user_mut) {
        Some(MutableParameterBinding)
    } else if !f.x.params.first().is_some_and(|p| matches!(&*p.x.typ, TypX::MutRef(_))) {
        Some(MissingMutableReceiver)
    } else {
        None
    }
}
