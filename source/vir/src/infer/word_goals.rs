//! Bounded selection of checked u64 word-view laws, never an update summary.
//! Only scoped broadcasts are proposed; unchanged bodies/goals are verified.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk};
use std::collections::HashSet;

const NODES: usize = 4096;
const FUNCTIONS: usize = 32;
const LEMMAS: [&str; 3] = ["lemma_or_member", "lemma_or_view", "lemma_full_view"];

fn identity(f: &Function, name: &str) -> bool {
    models::path_is(&f.x.name.path, CrateId::Vstd, &["word", name])
        && matches!(f.x.kind, FunctionKind::Static)
        && matches!(f.x.item_kind, ItemKind::Function)
        && f.x.proxy.is_none()
        && !f.x.attrs.is_external_body
        && !f.x.attrs.is_unsafe
        && !f.x.attrs.is_async
        && !f.x.attrs.infer
        && !f.x.attrs.exec_assume_termination
        && !f.x.attrs.exec_allows_no_decreases_clause
        && f.x.typ_params.is_empty()
        && f.x.typ_bounds.is_empty()
        && f.x.params.iter().all(|p| p.x.mode == Mode::Spec && !p.x.user_mut)
}

fn catalog(f: &Function, name: &str) -> bool {
    let types: &[IntRange] = match name {
        "lemma_or_member" => &[IntRange::U(64), IntRange::U(64), IntRange::Int],
        "lemma_or_view" => &[IntRange::U(64), IntRange::U(64), IntRange::Nat],
        "lemma_full_view" => &[IntRange::U(64), IntRange::Nat],
        _ => return false,
    };
    identity(f, name)
        && f.x.mode == Mode::Proof
        && f.x.attrs.broadcast_forall
        && enums::unit(&f.x.inner_ret.x.typ)
        && f.x.ensure.1.is_empty()
        && !f.x.ensure.0.is_empty()
        && f.x.params.len() == types.len()
        && f.x.params.iter().zip(types).all(|(p, t)| matches!(&*p.x.typ, TypX::Int(r) if r == t))
}

/// Follow only supplied spec definitions from requested contracts/assertions.
/// Reaching either budget discards the entire selection (no partial discovery).
fn demanded(f: &Function, all: &HashMap<Fun, Function>) -> bool {
    let mut pending: Vec<_> = f.x.require.iter().chain(f.x.ensure.0.iter()).cloned().collect();
    let mut nodes = NODES;
    if let Some(body) = &f.x.body {
        expr_visitor_walk(body, &mut |e| {
            if nodes == 0 {
                return VisitorControlFlow::Stop(());
            }
            nodes -= 1;
            if let ExprX::AssertAssume { is_assume: false, expr, .. } = &e.x {
                pending.push(expr.clone());
            }
            VisitorControlFlow::Recurse
        });
    }
    let mut visited = HashSet::new();
    let mut found = false;
    while let Some(e) = pending.pop() {
        let mut calls = Vec::new();
        expr_visitor_walk(&e, &mut |e| {
            if nodes == 0 {
                return VisitorControlFlow::Stop(());
            }
            nodes -= 1;
            if let ExprX::Call { target: CallTarget::Fun(k, n, ..), .. } = &e.x {
                calls.push(match k {
                    CallTargetKind::DynamicResolved { resolved, .. } => resolved.clone(),
                    _ => n.clone(),
                });
            }
            VisitorControlFlow::Recurse
        });
        if nodes == 0 {
            return false;
        }
        for n in calls {
            let Some(c) = all.get(&n) else { continue };
            found |= c.x.mode == Mode::Spec && (identity(c, "view") || identity(c, "member"));
            if c.x.mode == Mode::Spec && n.path.krate != CrateId::Vstd && visited.insert(n) {
                if visited.len() > FUNCTIONS {
                    return false;
                }
                if let Some(b) = &c.x.body {
                    pending.push(b.clone());
                }
            }
        }
    }
    nodes > 0 && found
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Vec<Function> {
    let all = crate::infer::pipeline::function_map(fs, imported);
    let selected: Vec<_> = LEMMAS
        .iter()
        .filter_map(|n| all.values().find(|f| catalog(f, n)).map(|f| f.x.name.clone()))
        .collect();
    if selected.len() != LEMMAS.len() {
        return fs.to_vec();
    }
    fs.iter()
        .map(|f| {
            if f.x.name.path.krate == CrateId::Vstd
                || f.x.mode != Mode::Exec
                || f.x.attrs.infer
                || f.x.attrs.bit_vector
                || f.x.attrs.is_external_body
                || f.x.attrs.is_unsafe
                || f.x.attrs.is_async
                || f.x.attrs.exec_assume_termination
                || f.x.attrs.exec_allows_no_decreases_clause
            {
                return f.clone();
            }
            let Some(body) = &f.x.body else { return f.clone() };
            let ExprX::Block(stmts, tail) = &body.x else { return f.clone() };
            if !demanded(f, &all) {
                return f.clone();
            }
            let mut prefix: Vec<_> = selected
                .iter()
                .map(|n| {
                    crate::def::Spanned::new(
                        f.span.clone(),
                        StmtX::Expr(SpannedTyped::new(
                            &f.span,
                            &crate::ast_util::unit_typ(),
                            ExprX::Fuel(n.clone(), 1, true),
                        )),
                    )
                })
                .collect();
            prefix.extend(stmts.iter().cloned());
            let mut x = f.x.clone();
            x.body = Some(body.new_x(ExprX::Block(Arc::new(prefix), tail.clone())));
            report.word_lemmas.push((x.name.clone(), selected.clone()));
            f.new_x(x)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input, span};

    fn library(name: &str) -> Function {
        let base = function(name, input("x"));
        let mut x = base.x.clone();
        x.name = Arc::new(FunX {
            path: Arc::new(PathX {
                krate: CrateId::Vstd,
                segments: Arc::new(["word", name].map(|s| Arc::new(s.into())).to_vec()),
            }),
        });
        let mut attrs = FunctionAttrsX::default();
        attrs.broadcast_forall = name.starts_with("lemma_");
        x.attrs = Arc::new(attrs);
        x.mode = if name == "view" { Mode::Spec } else { Mode::Proof };
        x.typ_params = Arc::new(vec![]);
        x.typ_bounds = Arc::new(vec![]);
        let types = match name {
            "lemma_or_member" => vec![IntRange::U(64), IntRange::U(64), IntRange::Int],
            "lemma_or_view" => vec![IntRange::U(64), IntRange::U(64), IntRange::Nat],
            _ => vec![IntRange::U(64), IntRange::Nat],
        };
        x.params = Arc::new(
            types
                .into_iter()
                .enumerate()
                .map(|(i, t)| {
                    let mut p = x.params[0].x.clone();
                    p.name = VarIdent(Arc::new(format!("p{i}")), VarIdentDisambiguate::AirLocal);
                    p.mode = Mode::Spec;
                    p.typ = Arc::new(TypX::Int(t));
                    crate::def::Spanned::new(span(), p)
                })
                .collect(),
        );
        let mut ret = x.inner_ret.x.clone();
        ret.typ = crate::ast_util::unit_typ();
        x.inner_ret = crate::def::Spanned::new(span(), ret);
        x.outer_ret = x.inner_ret.clone();
        x.ensure = (Arc::new(vec![boolean(&span(), true).expr]), Arc::new(vec![]));
        x.body = None;
        base.new_x(x)
    }
    fn call(f: &Function) -> Expr {
        SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    f.x.name.clone(),
                    Arc::new(vec![]),
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(vec![]),
                post_args: None,
                body: None,
            },
        )
    }
    fn fixture() -> (Function, Vec<Function>) {
        let mut catalog: Vec<_> = LEMMAS.map(library).into();
        catalog.push(library("view"));
        let base = function("caller", input("x"));
        let mut x = base.x.clone();
        x.attrs = Arc::new(FunctionAttrsX::default());
        x.require = Arc::new(vec![call(catalog.last().unwrap())]);
        x.body = Some(SpannedTyped::new(
            &span(),
            &input("x").typ,
            ExprX::Block(Arc::new(vec![]), x.body.clone()),
        ));
        (base.new_x(x), catalog)
    }
    fn selected(f: &Function, lib: &[Function]) -> bool {
        let mut r = GenerationReport::default();
        let result = prepare(&[f.clone()], lib, &mut r);
        if r.word_lemmas.is_empty() {
            assert!(Arc::ptr_eq(f, &result[0]));
            false
        } else {
            let original = f.x.body.as_ref().unwrap();
            let ExprX::Block(_, original_tail) = &original.x else { panic!() };
            let b = result[0].x.body.as_ref().unwrap();
            assert_eq!(b.span.id, original.span.id);
            let ExprX::Block(stmts, tail) = &b.x else { panic!() };
            assert!(Arc::ptr_eq(tail.as_ref().unwrap(), original_tail.as_ref().unwrap()));
            assert_eq!(stmts.len(), 3);
            assert!(stmts.iter().all(
                |s| matches!(&s.x, StmtX::Expr(e) if matches!(&e.x, ExprX::Fuel(_, 1, true)))
            ));
            let mut restored = result[0].x.clone();
            restored.body = f.x.body.clone();
            assert_eq!(format!("{restored:?}"), format!("{:?}", f.x));
            true
        }
    }
    #[test]
    fn only_scoped_fuel_and_original_body_and_contracts() {
        let (f, lib) = fixture();
        assert!(selected(&f, &lib));
    }
    #[test]
    fn eligibility_and_catalog_identity_fail_closed() {
        let (f, lib) = fixture();
        for i in 0..11 {
            let mut x = f.x.clone();
            match i {
                0 => Arc::make_mut(&mut x.attrs).infer = true,
                1 => Arc::make_mut(&mut x.attrs).is_external_body = true,
                2 => Arc::make_mut(&mut x.attrs).is_unsafe = true,
                3 => Arc::make_mut(&mut x.attrs).is_async = true,
                4 => Arc::make_mut(&mut x.attrs).exec_assume_termination = true,
                5 => x.mode = Mode::Proof,
                6 => x.body = None,
                7 => Arc::make_mut(&mut x.attrs).bit_vector = true,
                8 => Arc::make_mut(&mut x.attrs).exec_allows_no_decreases_clause = true,
                9 => {
                    let mut path = (*x.name.path).clone();
                    path.krate = CrateId::Vstd;
                    x.name = Arc::new(FunX { path: Arc::new(path) });
                }
                _ => x.require = Arc::new(vec![]),
            }
            assert!(!selected(&f.new_x(x), &lib));
        }
        for i in 0..5 {
            let mut bad = lib.clone();
            let mut x = bad[0].x.clone();
            match i {
                0 => Arc::make_mut(&mut x.attrs).broadcast_forall = false,
                1 => Arc::make_mut(&mut x.attrs).is_external_body = true,
                2 => Arc::make_mut(&mut x.attrs).exec_assume_termination = true,
                3 => x.params = Arc::new(vec![]),
                _ => {
                    // Same spelling in a user crate cannot be a catalog law.
                    let mut path = (*x.name.path).clone();
                    path.krate = f.x.name.path.krate.clone();
                    x.name = Arc::new(FunX { path: Arc::new(path) });
                }
            }
            bad[0] = bad[0].new_x(x);
            assert!(!selected(&f, &bad));
        }
        assert!(!selected(&f, &lib[1..]));
    }
    #[test]
    fn both_budgets_discard_partial_discovery() {
        let (f, lib) = fixture();
        let mut x = f.x.clone();
        Arc::make_mut(&mut x.require).insert(
            0,
            SpannedTyped::new(
                &span(),
                &bool_typ(),
                ExprX::ArrayLiteral(Arc::new(vec![boolean(&span(), true).expr; NODES + 1])),
            ),
        );
        assert!(!selected(&f.new_x(x), &lib));
        let mut lib = lib;
        let mut next = lib.last().unwrap().clone();
        for i in 0..=FUNCTIONS {
            let base = function(&format!("alias{i}"), input("x"));
            let mut x = base.x.clone();
            x.mode = Mode::Spec;
            x.body = Some(call(&next));
            next = base.new_x(x);
            lib.push(next.clone());
        }
        let mut x = f.x.clone();
        // Discover the real view as well, then exceed spec traversal budget.
        Arc::make_mut(&mut x.require).insert(0, call(&next));
        assert!(!selected(&f.new_x(x), &lib));
    }
}
