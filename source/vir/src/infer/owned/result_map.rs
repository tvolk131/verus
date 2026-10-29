//! Composition of the existing standard Result::map_err specification.
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
        || x.typ_params.len() != 4
        || x.require.len() != 1
        || x.ensure.0.len() != 2
        || !x.ensure.1.is_empty()
        || !same(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
        || p.krate != CrateId::Core
        || p.segments.len() != 3
        || p.segments[0].as_str() != "result"
        || !p.segments[1].starts_with("impl")
        || p.last_segment().as_str() != "map_err"
        || !x.proxy.as_ref().is_some_and(|p| {
            super::super::models::spec_proxy(&p.x, &["std_specs", "result"], "map__err")
        })
    {
        return false;
    }
    let ts: Vec<_> = x.typ_params.iter().map(|n| Arc::new(TypX::TypParam(n.clone()))).collect();
    sums::result(&x.params[0].x.typ).is_some_and(|(a, b)| same(a, &ts[0]) && same(b, &ts[1]))
        && same(&x.params[1].x.typ, &ts[3])
        && sums::result(&x.inner_ret.x.typ).is_some_and(|(a, b)| same(a, &ts[0]) && same(b, &ts[2]))
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
    (all.get(n).is_some_and(model) && ts.len() == 4 && impls.is_empty() && args.len() == 2)
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
    pub(super) fn map_error(
        &mut self,
        s: &mut State,
        e: &Expr,
        args: &Exprs,
    ) -> Result<Val, VirErr> {
        let input = self.expr(s, &args[0])?;
        let (ok, success, error) = self.result_parts(&e.span, &input)?;
        let (param, pre, mapped) = self.callback_value(s, &args[1], false)?;
        if is_bool(&ok, true) {
            return self.result_value(&e.span, e.typ.clone(), Node::Result(ok, success, None));
        }
        let error = error.ok_or_else(|| unsupported(&e.span, "map_err missing error payload"))?;
        let mut local = s.clone();
        local.locals.insert(param.name.clone(), *error);
        let pre = self.formula(&mut local, &pre.expr)?;
        let mut branch = s.clone();
        branch.active =
            logical(&e.span, LogicalOp::And, s.active.clone(), exits::not(&e.span, ok.clone())?)?;
        self.require(&branch, pre)?;
        let error = self.substitute_value(&mut local, &mapped)?;
        self.result_value(&e.span, e.typ.clone(), Node::Result(ok, success, Some(Box::new(error))))
    }
    fn substitute_value(&self, s: &State, value: &Val) -> Result<Val, VirErr> {
        let node = match &value.node {
            Node::Atom(t) => Node::Atom(self.formula(s, &t.expr)?),
            Node::Seq(t) => Node::Seq(self.formula(s, &t.expr)?),
            Node::Fields(fs) => Node::Fields(
                fs.iter()
                    .map(|(op, v)| Ok((op.clone(), self.substitute_value(s, v)?)))
                    .collect::<Result<Vec<_>, VirErr>>()?,
            ),
            Node::Result(tag, a, b) => {
                let tag = self.formula(s, &tag.expr)?;
                let a = if is_bool(&tag, false) {
                    None
                } else {
                    a.as_ref().map(|v| self.substitute_value(s, v).map(Box::new)).transpose()?
                };
                let b = if is_bool(&tag, true) {
                    None
                } else {
                    b.as_ref().map(|v| self.substitute_value(s, v).map(Box::new)).transpose()?
                };
                Node::Result(tag, a, b)
            }
            Node::Option(tag, a) => {
                let tag = self.formula(s, &tag.expr)?;
                let a = if is_bool(&tag, false) {
                    None
                } else {
                    a.as_ref().map(|v| self.substitute_value(s, v).map(Box::new)).transpose()?
                };
                Node::Option(tag, a)
            }
            Node::Variants(vs) => Node::Variants(
                vs.iter()
                    .map(|(v, tag, fs)| {
                        let tag = self.formula(s, &tag.expr)?;
                        let fs = if is_bool(&tag, false) {
                            None
                        } else {
                            fs.as_ref()
                                .map(|fs| {
                                    fs.iter()
                                        .map(|(op, v)| {
                                            Ok((op.clone(), self.substitute_value(s, v)?))
                                        })
                                        .collect::<Result<Vec<_>, VirErr>>()
                                })
                                .transpose()?
                        };
                        Ok((v.clone(), tag, fs))
                    })
                    .collect::<Result<Vec<_>, VirErr>>()?,
            ),
            Node::Iterator(..) => {
                return Err(unsupported(&self.f.span, "mapped iterator callback output"));
            }
        };
        Ok(Val { identity: None, typ: value.typ.clone(), node })
    }
}
