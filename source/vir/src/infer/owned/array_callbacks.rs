//! Proposed domain/content contracts for native slice.try_into().expect(...)
//! callbacks. All calls and the unchanged closure body verify ordinarily.
use super::*;

fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(t)) if ss.is_empty() => plain(t),
        ExprX::ReadPlace(p, _) => {
            if let PlaceX::Temporary(e) = &p.x {
                plain(e)
            } else {
                e
            }
        }
        _ => e,
    }
}
fn call<'a>(
    e: &'a Expr,
    all: &HashMap<Fun, Function>,
    module: &str,
    suffix: &str,
) -> Option<&'a Exprs> {
    let ExprX::Call { target: CallTarget::Fun(kind, n, ..), args, post_args: None, body: None } =
        &plain(e).x
    else {
        return None;
    };
    let resolved =
        if let CallTargetKind::DynamicResolved { resolved, .. } = kind { resolved } else { n };
    let f = all.get(resolved).or_else(|| all.get(n))?;
    f.x.proxy
        .as_ref()
        .filter(|p| super::super::models::spec_proxy(&p.x, &["std_specs", module], suffix))?;
    Some(args)
}
pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    fs.iter()
        .map(|f| {
            if f.x.mode != Mode::Exec
                || f.x.attrs.infer
                || f.x.attrs.is_external_body
                || f.x.attrs.is_unsafe
            {
                return Ok(f.clone());
            }
            let Some(body) = &f.x.body else { return Ok(f.clone()) };
            let mut plans = vec![];
            crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
                if let ExprX::NonSpecClosure {
                    params,
                    proof_fn_modes: None,
                    body,
                    requires,
                    ensures,
                    ret,
                    external_spec: None,
                } = &e.x
                {
                    let candidate = (|| {
                        if params.len() != 1 || !requires.is_empty() || !ensures.is_empty() {
                            return None;
                        }
                        let (element, n) = arrays::shape(&ret.a)?;
                        if !slices::input_type(&params[0].a)
                            || !same(&slices::element(&params[0].a)?, &element)
                        {
                            return None;
                        }
                        let expect = call(body, &all, "result", "_expect")?;
                        if expect.len() != 2 {
                            return None;
                        }
                        let convert = call(&expect[0], &all, "convert", "try__into")?;
                        if convert.len() != 1 {
                            return None;
                        }
                        let arg = observations::logical_value(plain(&convert[0]));
                        if !matches!(&arg.x,ExprX::Var(name) if name==&params[0].name) {
                            return None;
                        }
                        let input = atom(SpannedTyped::new(
                            &e.span,
                            &params[0].a,
                            ExprX::Var(params[0].name.clone()),
                        ))
                        .ok()?;
                        let output =
                            atom(SpannedTyped::new(&e.span, &ret.a, ExprX::Var(ret.name.clone())))
                                .ok()?;
                        let length =
                            term(&e.span, &int_typ(), ExprX::Const(Constant::Int(n.into())), 1)
                                .ok()?;
                        let pre = records::equal(
                            &e.span,
                            slices::length(&e.span, input.clone()).ok()?,
                            length,
                        )
                        .ok()?;
                        let mut post = boolean(&e.span, true);
                        for i in 0..n {
                            let index =
                                term(&e.span, &int_typ(), ExprX::Const(Constant::Int(i.into())), 1)
                                    .ok()?;
                            let eq = records::equal(
                                &e.span,
                                arrays::index(&e.span, output.clone(), index.clone()).ok()?,
                                slices::index(&e.span, input.clone(), index).ok()?,
                            )
                            .ok()?;
                            post = logical(&e.span, LogicalOp::And, post, eq).ok()?;
                        }
                        Some(callbacks::Plan {
                            span: e.span.id,
                            requires: Arc::new(vec![pre.expr]),
                            ensures: Arc::new(vec![post.expr]),
                        })
                    })();
                    if let Some(plan) = candidate {
                        plans.push(plan);
                    }
                    return crate::ast_visitor::VisitorControlFlow::<()>::Return;
                }
                crate::ast_visitor::VisitorControlFlow::Recurse
            });
            if plans.is_empty() || plans.len() > 4 {
                return Ok(f.clone());
            }
            report.array_callbacks.push((f.x.name.clone(), plans.len()));
            let mut x = f.x.clone();
            x.body = Some(error_callbacks::decorate(f, body, &plans)?);
            Ok(f.new_x(x))
        })
        .collect()
}
