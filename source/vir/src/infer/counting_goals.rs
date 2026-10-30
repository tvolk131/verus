//! Select existing checked counting theorems from a caller's supplied contract.
//!
//! This adds scoped `broadcast use` statements, not facts or rewritten goals.
//! Normal Verus uses the checked conditional theorems and checks every original obligation.
//! The fixed catalog and bounded spec-call traversal are deterministic.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk};

const FUNCTIONS: usize = 32;
const NODES: usize = 4096;
const LEMMAS: [&str; 4] = [
    "lemma_count_prefix_bound",
    "lemma_count_prefix_update",
    "lemma_count_prefix_zero",
    "lemma_count_prefix_full",
];

fn identity(f: &Function, name: &str) -> bool {
    let p = &f.x.name.path;
    p.krate == CrateId::Vstd
        && p.segments.len() == 3
        && p.segments[0].as_str() == "seq_lib"
        && p.segments[1].starts_with("impl")
        && p.segments[2].as_str() == name
        && f.x.proxy.is_none()
        && !f.x.attrs.is_external_body
        && !f.x.attrs.is_unsafe
        && !f.x.attrs.is_async
        && !f.x.attrs.infer
        && matches!(f.x.kind, FunctionKind::Static)
        && matches!(f.x.item_kind, ItemKind::Function)
        && f.x.typ_params.len() == 1
        && f.x.params.iter().all(|p| p.x.mode == Mode::Spec && !p.x.user_mut)
}

fn signature(f: &Function, lemma: bool) -> bool {
    let fx = &f.x;
    let Some(t) = fx.typ_params.first() else { return false };
    let elem = Arc::new(TypX::TypParam(t.clone()));
    let seq = |typ: &Typ| {
        matches!(&**typ, TypX::Datatype(Dt::Path(p), ts, _)
        if models::path_is(p, CrateId::Vstd, &["seq", "Seq"])
            && ts.len() == 1 && crate::ast_util::types_equal(&ts[0], &elem))
    };
    let sized = fx.typ_bounds.len() == 1
        && matches!(&*fx.typ_bounds[0],
        GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args)
            if args.len() == 1 && crate::ast_util::types_equal(&args[0], &elem));
    sized
        && fx.params.len() >= 3
        && seq(&fx.params[0].x.typ)
        && matches!(&*fx.params[1].x.typ, TypX::SpecFn(args, ret)
            if args.len() == 1 && crate::ast_util::types_equal(&args[0], &elem)
                && matches!(&**ret, TypX::Bool))
        && matches!(&*fx.params.last().unwrap().x.typ, TypX::Int(IntRange::Nat))
        && if lemma {
            fx.mode == Mode::Proof
                && fx.attrs.broadcast_forall
                && enums::unit(&fx.inner_ret.x.typ)
                && fx.ensure.0.len() == 1
                && fx.ensure.1.is_empty()
                && if identity(f, "lemma_count_prefix_update") {
                    fx.params.len() == 5
                        && matches!(&*fx.params[2].x.typ, TypX::Int(IntRange::Int))
                        && crate::ast_util::types_equal(&fx.params[3].x.typ, &elem)
                } else {
                    fx.params.len() == 3
                }
        } else {
            fx.mode == Mode::Spec
                && fx.params.len() == 3
                && matches!(&*fx.inner_ret.x.typ, TypX::Int(IntRange::Nat))
        }
}

/// Missing/oversized spec bodies merely make the strategy inapplicable. They
/// never authorize a contract or suppress an ordinary proof failure.
fn demands_count(f: &Function, all: &HashMap<Fun, Function>) -> Option<(bool, bool)> {
    let mut pending: Vec<_> = f.x.require.iter().chain(f.x.ensure.0.iter()).cloned().collect();
    let mut visited = std::collections::HashSet::new();
    let mut nodes = NODES;
    let mut found = false;
    let (mut sparse, mut projection) = (false, false);
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
        if nodes == 0 {
            return None;
        }
    }
    while let Some(e) = pending.pop() {
        let mut calls = Vec::new();
        expr_visitor_walk(&e, &mut |e| {
            if nodes == 0 {
                return VisitorControlFlow::Stop(());
            }
            nodes -= 1;
            if let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } = &e.x {
                calls.push(name.clone());
            }
            VisitorControlFlow::Recurse
        });
        if nodes == 0 {
            return None;
        }
        for name in calls {
            let Some(callee) = all.get(&name) else { continue };
            sparse |= models::path_is(&name.path, CrateId::Vstd, &["seq_update", "sparse_entries"]);
            projection |= name.path.krate == CrateId::Vstd
                && name.path.segments.first().is_some_and(|s| s.as_str() == "seq_lib")
                && name.path.last_segment().as_str() == "map_values";
            if identity(callee, "count_prefix") && signature(callee, false) {
                found = true;
            } else if callee.x.mode == Mode::Spec && visited.insert(name) {
                if visited.len() > FUNCTIONS {
                    return None;
                }
                if let Some(body) = &callee.x.body {
                    pending.push(body.clone());
                }
            }
        }
    }
    found.then_some((sparse, projection))
}

pub fn prepare(
    functions: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Vec<Function> {
    let all: HashMap<_, _> =
        functions.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    let lemmas: Vec<_> = LEMMAS
        .iter()
        .filter_map(|name| {
            all.values().find(|f| identity(f, name) && signature(f, true)).map(|f| f.x.name.clone())
        })
        .collect();
    if lemmas.len() != LEMMAS.len() {
        return functions.to_vec();
    }
    let optional: Vec<_> = [
        ("lemma_sparse_entries_count", 2, 2),
        ("lemma_sparse_filter_count", 3, 5),
        ("lemma_sparse_map_filter_count", 4, 6),
        ("lemma_count_prefix_congruence", 2, 5),
    ]
    .iter()
    .filter_map(|(name, types, params)| {
        all.values()
            .find(|f| fold_goals::catalog(f, "seq_update", name, *types, *params))
            .map(|f| (*name == "lemma_count_prefix_congruence", f.x.name.clone()))
    })
    .collect();
    functions
        .iter()
        .map(|f| {
            // Never enable the library's own theorems while proving the library.
            if f.x.name.path.krate == CrateId::Vstd
                || f.x.mode == Mode::Spec
                || f.x.attrs.infer
                || f.x.attrs.bit_vector
                || f.x.attrs.is_external_body
                || f.x.body.is_none()
            {
                return f.clone();
            }
            let Some((sparse, projection)) = demands_count(f, &all) else { return f.clone() };
            // Optional conditional theorems are selected only for demanded
            // sparse/projection vocabulary. The library and inferred precision
            // obligations never receive their own catalog as proof assistance.
            let lemmas: Vec<_> = lemmas
                .iter()
                .cloned()
                .chain(
                    optional
                        .iter()
                        .filter(|(congruence, _)| if *congruence { projection } else { sparse })
                        .map(|(_, name)| name.clone()),
                )
                .collect();
            let body = f.x.body.as_ref().unwrap();
            let ExprX::Block(original, tail) = &body.x else { return f.clone() };
            let mut statements: Vec<_> = lemmas
                .iter()
                .map(|name| {
                    let e = SpannedTyped::new(
                        &f.span,
                        &crate::ast_util::unit_typ(),
                        ExprX::Fuel(name.clone(), 1, true),
                    );
                    crate::def::Spanned::new(f.span.clone(), StmtX::Expr(e))
                })
                .collect();
            statements.extend(original.iter().cloned());
            let mut fx = f.x.clone();
            // Preserve the original block's identity, children and tail. Wrapping
            // it in another block with the same span duplicates resolution AstIds.
            fx.body = Some(body.new_x(ExprX::Block(Arc::new(statements), tail.clone())));
            report.goal_lemmas.push((fx.name.clone(), lemmas.clone()));
            f.new_x(fx)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input, span};

    fn catalog_function(name: &str, lemma: bool) -> Function {
        let base = function(name, input("x"));
        let mut f = base.x.clone();
        f.name = Arc::new(FunX {
            path: Arc::new(PathX {
                krate: CrateId::Vstd,
                segments: Arc::new(
                    ["seq_lib", "impl&%0", name].map(|s| Arc::new(s.into())).to_vec(),
                ),
            }),
        });
        f.mode = if lemma { Mode::Proof } else { Mode::Spec };
        f.typ_params = Arc::new(vec![Arc::new("T".into())]);
        let elem = Arc::new(TypX::TypParam(f.typ_params[0].clone()));
        f.typ_bounds = Arc::new(vec![Arc::new(GenericBoundX::Trait(
            TraitId::Sizedness(Sizedness::Sized),
            Arc::new(vec![elem.clone()]),
        ))]);
        let seq = Arc::new(TypX::Datatype(
            Dt::Path(Arc::new(PathX {
                krate: CrateId::Vstd,
                segments: Arc::new(["seq", "Seq"].map(|s| Arc::new(s.into())).to_vec()),
            })),
            Arc::new(vec![elem.clone()]),
            Arc::new(vec![]),
        ));
        let mut types = vec![seq, Arc::new(TypX::SpecFn(Arc::new(vec![elem.clone()]), bool_typ()))];
        if name == "lemma_count_prefix_update" {
            types.extend([crate::ast_util::int_typ(), elem]);
        }
        types.push(Arc::new(TypX::Int(IntRange::Nat)));
        f.params = Arc::new(
            types
                .into_iter()
                .enumerate()
                .map(|(i, typ)| {
                    let mut p = f.params[0].x.clone();
                    p.name = VarIdent(Arc::new(format!("p{i}")), VarIdentDisambiguate::AirLocal);
                    p.typ = typ;
                    p.mode = Mode::Spec;
                    crate::def::Spanned::new(span(), p)
                })
                .collect(),
        );
        let mut ret = f.inner_ret.x.clone();
        ret.typ =
            if lemma { crate::ast_util::unit_typ() } else { Arc::new(TypX::Int(IntRange::Nat)) };
        f.inner_ret = crate::def::Spanned::new(span(), ret);
        f.outer_ret = f.inner_ret.clone();
        let mut attrs = FunctionAttrsX::default();
        attrs.broadcast_forall = lemma;
        f.attrs = Arc::new(attrs);
        f.body = None;
        f.ensure = (
            Arc::new(if lemma { vec![boolean(&span(), true).expr] } else { vec![] }),
            Arc::new(vec![]),
        );
        base.new_x(f)
    }

    fn fixture() -> (Function, Vec<Function>) {
        let counter = catalog_function("count_prefix", false);
        let mut catalog = vec![counter.clone()];
        catalog.extend(LEMMAS.map(|n| catalog_function(n, true)));
        let base = function("caller", input("x"));
        let mut f = base.x.clone();
        f.attrs = Arc::new(FunctionAttrsX::default());
        f.body = Some(SpannedTyped::new(
            &span(),
            &input("x").typ,
            ExprX::Block(Arc::new(vec![]), f.body.clone()),
        ));
        f.require = Arc::new(vec![SpannedTyped::new(
            &span(),
            &crate::ast_util::int_typ(),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    counter.x.name.clone(),
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
        )]);
        (base.new_x(f), catalog)
    }

    #[test]
    fn selection_adds_only_scoped_fuel_and_preserves_original_body() {
        let (caller, catalog) = fixture();
        let mut report = GenerationReport::default();
        let result = prepare(&[caller.clone()], &catalog, &mut report);
        assert_eq!(report.goal_lemmas.len(), 1);
        let original = caller.x.body.as_ref().unwrap();
        let ExprX::Block(_, Some(tail)) = &original.x else { panic!() };
        let ExprX::Block(ss, Some(body)) = &result[0].x.body.as_ref().unwrap().x else { panic!() };
        assert!(Arc::ptr_eq(body, tail));
        assert_eq!(original.span.id, result[0].x.body.as_ref().unwrap().span.id);
        assert_eq!(ss.len(), 4);
        assert!(
            ss.iter()
                .all(|s| matches!(&s.x, StmtX::Expr(e) if matches!(&e.x, ExprX::Fuel(_, 1, true))))
        );
        let mut restored = result[0].x.clone();
        restored.body = caller.x.body.clone();
        assert_eq!(format!("{restored:?}"), format!("{:?}", caller.x));
    }

    #[test]
    fn catalog_and_demand_gates_skip_unrelated_inferred_and_library_functions() {
        let (caller, mut catalog) = fixture();
        let run = |f: Function, available: &[Function]| {
            let mut report = GenerationReport::default();
            prepare(&[f], available, &mut report);
            report.goal_lemmas.len()
        };
        assert_eq!(run(caller.clone(), &catalog), 1);
        let mut f = caller.x.clone();
        f.require = Arc::new(vec![]);
        assert_eq!(run(caller.new_x(f), &catalog), 0);
        let mut f = caller.x.clone();
        Arc::make_mut(&mut f.attrs).infer = true;
        assert_eq!(run(caller.new_x(f), &catalog), 0);
        let mut f = caller.x.clone();
        let mut path = (*f.name.path).clone();
        path.krate = CrateId::Vstd;
        f.name = Arc::new(FunX { path: Arc::new(path) });
        assert_eq!(run(caller.new_x(f), &catalog), 0);
        let mut bad = catalog[1].x.clone();
        Arc::make_mut(&mut bad.attrs).broadcast_forall = false;
        catalog[1] = catalog[1].new_x(bad);
        assert_eq!(run(caller, &catalog), 0);
    }

    #[test]
    fn exhausted_node_budget_leaves_the_function_unchanged() {
        let (caller, catalog) = fixture();
        let mut f = caller.x.clone();
        Arc::make_mut(&mut f.require).push(SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::ArrayLiteral(Arc::new(vec![boolean(&span(), true).expr; NODES + 1])),
        ));
        let caller = caller.new_x(f);
        let mut report = GenerationReport::default();
        let result = prepare(&[caller.clone()], &catalog, &mut report);
        assert!(report.goal_lemmas.is_empty());
        assert!(Arc::ptr_eq(&result[0], &caller));
    }
}
