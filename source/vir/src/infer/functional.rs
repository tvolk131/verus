//! Callees with a functional specification: preconditions over their inputs
//! plus either a `returns` clause or exactly one `ensures ret == E`.
//!
//! This covers vstd's `assume_specification`s for std functions (for example
//! `u64::is_multiple_of` and `Option::unwrap`) and hand-written contracts of
//! the same shape. No model is added to the trusted base: the caller's
//! candidate is still discharged by ordinary verification against the
//! callee's own specification, whose preconditions the maximality check
//! already treats as the callee's safe domain. A callee that requires
//! `false` and returns `!` (for example `core::panicking::assert_failed`,
//! reached through `assert_eq!`) is a diverging call.
use super::*;

pub(crate) enum Shape<'a> {
    Value { requires: &'a [Expr], output: &'a Expr },
    Diverges,
}

fn never(t: &Typ) -> bool {
    matches!(&**t, TypX::Decorate(TypDecoration::Never, ..))
}

fn is_false(e: &Expr) -> bool {
    matches!(&e.x, ExprX::Const(Constant::Bool(false)))
}

fn is_local(e: &Expr, name: &VarIdent) -> bool {
    match &e.x {
        ExprX::Var(n) => n == name,
        ExprX::ReadPlace(p, _) => matches!(&p.x, PlaceX::Local(n) if n == name),
        _ => false,
    }
}

fn mentions(e: &Expr, name: &VarIdent) -> bool {
    let mut found = false;
    crate::ast_visitor::expr_visitor_walk(e, &mut |e| {
        match &e.x {
            ExprX::Var(n) | ExprX::VarAt(n, _) if n == name => found = true,
            ExprX::ReadPlace(p, _) if matches!(&p.x, PlaceX::Local(n) if n == name) => found = true,
            _ => {}
        }
        crate::ast_visitor::VisitorControlFlow::<()>::Recurse
    });
    found
}

/// The functional shape of a callee's specification, if it has one.
/// Inferred functions use their own candidate path and are excluded.
pub(crate) fn shape(f: &Function) -> Option<Shape<'_>> {
    let x = &f.x;
    // Existing approved models and checked interfaces keep their own,
    // more specific treatment; this rule only covers the remaining callees.
    if is_approved_model(f) || is_inference_contract(f) || is_wrapper_operator_boundary(f) {
        return None;
    }
    if x.mode != Mode::Exec
        || x.attrs.infer
        || !matches!(x.kind, FunctionKind::Static)
        || !x.ensure.1.is_empty()
        || x.params
            .iter()
            .any(|p| p.x.mode != Mode::Exec || p.x.user_mut || matches!(&*p.x.typ, TypX::MutRef(_)))
    {
        return None;
    }
    if never(&x.inner_ret.x.typ) {
        return (x.ensure.0.is_empty() && x.returns.is_none() && x.require.iter().any(is_false))
            .then_some(Shape::Diverges);
    }
    let ret = &x.inner_ret.x.name;
    let output = match (&x.returns, &x.ensure.0[..]) {
        (Some(output), []) => output,
        (None, [ensure]) => match &ensure.x {
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) if is_local(lhs, ret) => rhs,
            _ => return None,
        },
        _ => return None,
    };
    if mentions(output, ret) || x.require.iter().any(|r| mentions(r, ret)) {
        return None;
    }
    Some(Shape::Value { requires: &x.require, output })
}

pub(crate) fn is_functional(f: &Function) -> bool {
    shape(f).is_some()
}

/// A callee's requirement and output, instantiated at one call.
pub(super) struct Instance {
    pub requires: Term,
    /// None for a diverging callee.
    pub output: Option<Term>,
}

/// Substitute call arguments for parameters, and call type arguments for
/// type parameters, in the callee's specification expressions.
pub(super) fn instantiate(
    span: &Span,
    callee: &Function,
    types: &Typs,
    args: Vec<Term>,
) -> Result<Instance, VirErr> {
    let shape = shape(callee).ok_or_else(|| unsupported(span, "non-functional callee"))?;
    let x = &callee.x;
    if args.len() != x.params.len() || types.len() != x.typ_params.len() {
        return Err(unsupported(span, "functional callee arity"));
    }
    let Shape::Value { requires, output } = shape else {
        return Ok(Instance { requires: boolean(span, false), output: None });
    };
    let ts: HashMap<Ident, Typ> = x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
    let mut vars: HashMap<VarIdent, Expr> = HashMap::new();
    for (param, arg) in x.params.iter().zip(args) {
        let typ = crate::sst_util::subst_typ(&ts, &param.x.typ);
        let arg = records::coerce(span, arg.clone(), &typ).unwrap_or(arg);
        vars.insert(param.x.name.clone(), arg.expr);
    }
    let subst = |e: &Expr| -> Result<Term, VirErr> {
        let mapped = crate::ast_visitor::map_expr_visitor_env(
            e,
            &mut crate::ast_visitor::VisitorScopeMap::new(),
            &mut (),
            &|_, _, e| {
                Ok(match &e.x {
                    ExprX::Var(n) => vars.get(n).cloned().unwrap_or_else(|| e.clone()),
                    // Parameters were substituted into places as temporaries; a
                    // spec read of a temporary is just its value.
                    ExprX::ReadPlace(p, _) => match &p.x {
                        PlaceX::Local(n) if vars.contains_key(n) => vars[n].clone(),
                        PlaceX::Temporary(inner)
                            if crate::ast_util::types_equal(&inner.typ, &e.typ) =>
                        {
                            inner.clone()
                        }
                        _ => e.clone(),
                    },
                    _ => e.clone(),
                })
            },
            &|_, _, st| Ok(vec![st.clone()]),
            &|_, t| {
                Ok(match &**t {
                    TypX::TypParam(n) => ts.get(n).cloned().unwrap_or_else(|| t.clone()),
                    _ => t.clone(),
                })
            },
            &|_, _, p| {
                Ok(match &p.x {
                    PlaceX::Local(n) if vars.contains_key(n) => {
                        p.new_x(PlaceX::Temporary(vars[n].clone()))
                    }
                    _ => p.clone(),
                })
            },
            &|_, _, p| Ok(p.clone()),
        )?;
        // Keep the call site's span so diagnostics point at the caller.
        let mapped = SpannedTyped::new(span, &mapped.typ, mapped.x.clone());
        records::atom(mapped)
    };
    let mut pre = boolean(span, true);
    for r in requires.iter() {
        pre = logical(span, LogicalOp::And, pre, subst(r)?)?;
    }
    Ok(Instance { requires: pre, output: Some(subst(output)?) })
}
