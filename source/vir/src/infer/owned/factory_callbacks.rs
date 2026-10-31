//! Untrusted total callback equations for ordinary finite-view factory goals.
//! The frontend scaffold and callback bodies undergo ordinary Verus checking;
//! these functions are not published as exact inferred iterator values.
use super::*;

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    ds: &[Datatype],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = crate::infer::pipeline::function_map(fs, imported);
    fs.iter()
        .map(|f| {
            if !f.x.attrs.infer_iterator_factory || f.x.attrs.infer || f.x.body.is_none() {
                return Ok(f.clone());
            }
            let body = f.x.body.as_ref().unwrap();
            let mut closures = vec![];
            crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
                if let ExprX::NonSpecClosure { requires, ensures, .. } = &e.x {
                    if requires.is_empty() && ensures.is_empty() {
                        closures.push(e.clone());
                    }
                    return crate::ast_visitor::VisitorControlFlow::<()>::Return;
                }
                crate::ast_visitor::VisitorControlFlow::Recurse
            });
            if closures.is_empty() {
                return Ok(f.clone());
            }
            if closures.len() > 4 {
                return Err(unsupported(&f.span, "factory callback budget exceeded"));
            }
            let mut eng = Engine {
                f,
                all: &all,
                ds,
                requirements: vec![],
                exits: vec![],
                plans: vec![],
                callbacks: vec![],
                remaining: FORMULA_BUDGET,
            };
            let mut state = State {
                locals: HashMap::new(),
                objects: HashMap::new(),
                active: boolean(&f.span, true),
            };
            for p in f.x.params.iter() {
                let value = SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone()));
                state.objects.insert(p.x.name.clone(), atom(value.clone())?);
                state.locals.insert(p.x.name.clone(), eng.layout(value, 8)?);
            }
            for closure in &closures {
                let (_, pre, _) = eng.callback_value(&state, closure, false)?;
                if !is_bool(&pre, true) {
                    return Err(unsupported(
                        &closure.span,
                        "factory callbacks require unrestricted safety",
                    ));
                }
            }
            let mut x = f.x.clone();
            x.body = Some(callbacks::annotate(f, body, &eng.callbacks)?);
            report.factory_callbacks.push((f.x.name.clone(), closures.len()));
            Ok(f.new_x(x))
        })
        .collect()
}
