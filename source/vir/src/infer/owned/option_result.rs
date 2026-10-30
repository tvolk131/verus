//! Compose the existing vstd Option::ok_or contract. Both arguments are eager.
//! This proposes a result relation; ordinary soundness and precision still run.
use super::*;

pub(crate) fn model(f: &Function) -> bool {
    let x = &f.x;
    let p = &x.name.path;
    if x.mode != Mode::Exec
        || x.body.is_some()
        || x.attrs.infer
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.params.len() != 2
        || x.params.iter().any(|p| p.x.mode != Mode::Exec || p.x.user_mut)
        || x.typ_params.len() != 2
        || !x.require.is_empty()
        || x.ensure.0.len() != 1
        || !x.ensure.1.is_empty()
        || !same(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
        || p.krate != CrateId::Core
        || p.segments.len() != 3
        || p.segments[0].as_str() != "option"
        || !p.segments[1].starts_with("impl")
        || p.last_segment().as_str() != "ok_or"
        || !x.proxy.as_ref().is_some_and(|p| {
            super::super::models::spec_proxy(&p.x, &["std_specs", "option"], "ok__or")
        })
    {
        return false;
    }
    let ts: Vec<_> = x.typ_params.iter().map(|n| Arc::new(TypX::TypParam(n.clone()))).collect();
    option(&x.params[0].x.typ).is_some_and(|t| same(t, &ts[0]))
        && same(&x.params[1].x.typ, &ts[1])
        && sums::result(&x.inner_ret.x.typ).is_some_and(|(a, b)| same(a, &ts[0]) && same(b, &ts[1]))
}

pub(crate) fn call(e: &Expr, all: &HashMap<Fun, Function>) -> Option<Exprs> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, n, ts, impls, _),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    (all.get(n).is_some_and(model) && ts.len() == 2 && impls.is_empty() && args.len() == 2)
        .then(|| args.clone())
}

pub(crate) fn uses(f: &Function, all: &HashMap<Fun, Function>) -> bool {
    let mut found = false;
    if let Some(body) = &f.x.body {
        crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
            found |= call(e, all).is_some();
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    found
}

impl Engine<'_> {
    pub(super) fn option_result(
        &mut self,
        s: &mut State,
        e: &Expr,
        args: &Exprs,
    ) -> Result<Val, VirErr> {
        let input = self.expr(s, &args[0])?;
        // ok_or evaluates its error argument even on the Some path.
        let error = self.expr(s, &args[1])?;
        let value = input.scalar(&e.span)?;
        let some = atom(tag_expr(&value.expr))?;
        let payload = option(&input.typ).ok_or_else(|| unsupported(&e.span, "ok_or input type"))?;
        let success =
            self.layout(field_expr(&value.expr, &option_field(&input.typ), payload), 8)?;
        self.result_value(
            &e.span,
            e.typ.clone(),
            Node::Result(some, Some(Box::new(success)), Some(Box::new(error))),
        )
    }
}
