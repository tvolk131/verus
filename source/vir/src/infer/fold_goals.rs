//! Bounded selection of ordinary checked fold-observation lemmas.
//! No goal is rewritten and no assertion is assumed. Selection enables only
//! scoped broadcasts after a caller asks for sequence length or element facts.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk};
use std::collections::{BTreeSet, HashSet};

const NODES: usize = 4096;
const FUNCTIONS: usize = 32;

fn path(f: &Function, module: &str, name: &str) -> bool {
    let mut segments: Vec<_> = module.split("::").collect();
    segments.push(name);
    models::path_is(&f.x.name.path, CrateId::Vstd, &segments)
        && f.x.proxy.is_none()
        && !f.x.attrs.is_external_body
        && !f.x.attrs.is_unsafe
        && !f.x.attrs.is_async
        && !f.x.attrs.infer
        && matches!(f.x.kind, FunctionKind::Static)
}

pub(super) fn catalog(f: &Function, module: &str, name: &str, types: usize, params: usize) -> bool {
    path(f, module, name)
        && f.x.mode == Mode::Proof
        && f.x.attrs.broadcast_forall
        && f.x.typ_params.len() == types
        && f.x.params.len() == params
        && f.x.params.iter().all(|p| p.x.mode == Mode::Spec && !p.x.user_mut)
        && enums::unit(&f.x.inner_ret.x.typ)
        && f.x.ensure.1.is_empty()
}

fn demands(f: &Function, all: &HashMap<Fun, Function>) -> Option<(bool, bool, bool)> {
    let mut goals: Vec<_> = f.x.require.iter().chain(f.x.ensure.0.iter()).cloned().collect();
    let mut contracts = Vec::new();
    let mut nodes = NODES;
    if let Some(body) = &f.x.body {
        expr_visitor_walk(body, &mut |e| {
            if nodes == 0 {
                return VisitorControlFlow::Stop(());
            }
            nodes -= 1;
            match &e.x {
                ExprX::AssertAssume { is_assume: false, expr, .. } => goals.push(expr.clone()),
                ExprX::Call { target: CallTarget::Fun(_, n, ..), .. } => contracts.push(n.clone()),
                _ => (),
            }
            VisitorControlFlow::Recurse
        });
    }
    let mut visited = HashSet::new();
    let (mut length, mut index, mut fold, mut scatter) = (false, false, false, false);
    // Only requested expressions establish demand. Callee contracts can expose
    // a fold equation but cannot manufacture an observation request.
    for observe in [true, false] {
        if !observe {
            for n in &contracts {
                if let Some(c) = all.get(n) {
                    goals.extend(c.x.ensure.0.iter().cloned());
                }
            }
            visited.clear();
        }
        while let Some(e) = goals.pop() {
            let mut calls = Vec::new();
            expr_visitor_walk(&e, &mut |e| {
                if nodes == 0 {
                    return VisitorControlFlow::Stop(());
                }
                nodes -= 1;
                if observe
                    && (matches!(&e.x, ExprX::Binary(BinaryOp::Index(..), ..))
                        || matches!(&e.x, ExprX::ReadPlace(p, _) if matches!(&p.x, PlaceX::Index(..))))
                {
                    index = true;
                }
                if let ExprX::Call { target: CallTarget::Fun(kind, n, ..), .. } = &e.x {
                    let n = match kind {
                        CallTargetKind::DynamicResolved { resolved, .. } => resolved,
                        _ => n,
                    };
                    if observe
                        && n.path.krate == CrateId::Vstd
                        && n.path.segments[0].as_str() == "seq"
                    {
                        length |= n.path.last_segment().as_str() == "len";
                        index |= matches!(n.path.last_segment().as_str(), "index" | "spec_index");
                    }
                    if observe && n.path.krate == CrateId::Vstd {
                        fold |= matches!(n.path.last_segment().as_str(), "fold_left" | "scatter");
                        // A requested elementwise mapping asks for the complete
                        // sequence projection, even without an explicit len or
                        // index expression in the caller's contract.
                        if n.path.segments.first().is_some_and(|s| s.as_str() == "seq_lib")
                            && n.path.last_segment().as_str() == "map_values"
                        {
                            length = true;
                            index = true;
                        }
                    }
                    calls.push(n.clone());
                }
                VisitorControlFlow::Recurse
            });
            if nodes == 0 {
                return None;
            }
            for n in calls {
                let Some(c) = all.get(&n) else { continue };
                scatter |= path(c, "seq_update", "scatter");
                if c.x.mode == Mode::Spec
                    && c.x.name.path.krate != CrateId::Vstd
                    && visited.insert(n)
                {
                    if visited.len() > FUNCTIONS {
                        return None;
                    }
                    if let Some(b) = &c.x.body {
                        goals.push(b.clone());
                    }
                }
            }
        }
    }
    (scatter && (length || index || fold)).then_some((length, index, fold))
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Vec<Function> {
    let all: HashMap<_, _> =
        fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    let find = |module, name, types, params| {
        all.values().find(|f| catalog(f, module, name, types, params)).map(|f| f.x.name.clone())
    };
    fs.iter()
        .map(|f| {
            if f.x.name.path.krate == CrateId::Vstd
                || f.x.mode == Mode::Spec
                || f.x.attrs.infer
                || f.x.attrs.bit_vector
                || f.x.attrs.is_external_body
            {
                return f.clone();
            }
            let Some(body) = &f.x.body else { return f.clone() };
            let ExprX::Block(stmts, tail) = &body.x else { return f.clone() };
            let Some((length, index, fold)) = demands(f, &all) else { return f.clone() };
            let mut selected = BTreeSet::new();
            if fold {
                if let Some(n) = find("seq_update", "lemma_fold_left_congruence", 2, 4) {
                    selected.insert(n);
                }
            }
            if length || index {
                if let Some(n) = find("seq_update", "lemma_scatter_len", 2, 4) {
                    selected.insert(n);
                }
                if let Some(n) = find("seq_update", "lemma_sparse_entries_empty", 2, 2) {
                    selected.insert(n);
                }
            }
            if index {
                for (module, name, types, params) in [
                    ("seq_update", "lemma_scatter_index", 2, 5),
                    ("seq_update", "lemma_scatter_sparse", 3, 5),
                    ("seq_update", "lemma_last_write", 1, 3),
                    ("std_specs::iter_adapters", "lemma_flatten_options_contains", 1, 2),
                ] {
                    if let Some(n) = find(module, name, types, params) {
                        selected.insert(n);
                    }
                }
            }
            if selected.is_empty() {
                return f.clone();
            }
            let selected: Vec<_> = selected.into_iter().collect();
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
            report.fold_lemmas.push((x.name.clone(), selected));
            f.new_x(x)
        })
        .collect()
}
