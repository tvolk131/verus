//! Bounded, untrusted proposals of ordinary checked finite-Set lemma calls.
//! Only supplied contracts and visible spec bodies drive discovery. Nothing is
//! assumed, rewritten, exported as an inferred helper, or installed in a contract.
use super::*;
use crate::ast_visitor::{
    VisitorControlFlow, VisitorScopeMap, expr_visitor_walk, map_expr_visitor_env,
};
use std::collections::HashSet;

const NODES: usize = 4096;
const DEFS: usize = 32;
const TERMS: usize = 8;
const PAIRS: usize = 32;
const LEMMA: &[&str] = &["set_lib", "lemma_set_pair_observations"];

fn bytes<T: serde::Serialize + ?Sized>(x: &T) -> Vec<u8> {
    serde_json::to_vec(x).expect("VIR serialization")
}
fn set(t: &Typ) -> Option<&Typ> {
    match &**t {
        TypX::Datatype(Dt::Path(p), ts, ips)
            if models::path_is(p, CrateId::Vstd, &["set", "Set"])
                && ts.len() == 1
                && ips.is_empty() =>
        {
            Some(&ts[0])
        }
        _ => None,
    }
}
fn ordinary(f: &Function) -> bool {
    let x = &f.x;
    !x.attrs.infer
        && !x.attrs.is_external_body
        && !x.attrs.is_unsafe
        && !x.attrs.is_async
        && !x.attrs.prophecy_dependent
        && !x.attrs.exec_assume_termination
        && !x.attrs.exec_allows_no_decreases_clause
        && x.proxy.is_none()
        && matches!(x.kind, FunctionKind::Static)
        && matches!(x.item_kind, ItemKind::Function)
        && x.params.iter().all(|p| p.x.mode == Mode::Spec && !p.x.user_mut)
}
fn catalog(f: &Function) -> bool {
    let x = &f.x;
    models::path_is(&x.name.path, CrateId::Vstd, LEMMA)
        && ordinary(f)
        && x.mode == Mode::Proof
        && !x.attrs.broadcast_forall
        && x.typ_params.len() == 1
        && x.params.len() == 2
        && x.typ_bounds.iter().all(|b| {
            matches!(&**b,
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), ts)
                if ts.len()==1 && matches!(&*ts[0],TypX::TypParam(n) if n==&x.typ_params[0]))
        })
        && x.params.iter().all(|p| {
            matches!(set(&p.x.typ),Some(t)
            if matches!(&**t,TypX::TypParam(n) if n==&x.typ_params[0]))
        })
        && enums::unit(&x.inner_ret.x.typ)
        && enums::unit(&x.outer_ret.x.typ)
        && x.inner_ret.x.mode == Mode::Proof
        && x.outer_ret.x.mode == Mode::Proof
        && x.require.is_empty()
        && x.ensure.0.len() == 3
        && x.ensure.1.is_empty()
        && x.returns.is_none()
        && x.decrease_when.is_none()
}
fn eligible(f: &Function) -> bool {
    ordinary(f)
        && f.x.mode == Mode::Proof
        && f.x.name.path.krate != CrateId::Vstd
        && enums::unit(&f.x.inner_ret.x.typ)
        && enums::unit(&f.x.outer_ret.x.typ)
        && f.x.ensure.1.is_empty()
        && f.x.returns.is_none()
        && f.x.params.iter().all(|p| !mutable_type(&p.x.typ))
        && matches!(f.x.body.as_ref().map(|e| &e.x), Some(ExprX::Block(..)))
}
fn mutable_type(t: &Typ) -> bool {
    let mut found = false;
    let _ = crate::ast_visitor::map_typ_visitor_env(t, &mut found, &|found, t| {
        found_mut_type(t, found);
        Ok(t.clone())
    });
    found
}
fn found_mut_type(t: &Typ, found: &mut bool) {
    *found |= matches!(&**t, TypX::MutRef(_));
}

/// This checker intentionally accepts no binder or snapshot, even if nested
/// within a purportedly pure spec call. All free names must be source parameters.
fn rooted(e: &Expr, f: &Function, all: &HashMap<Fun, Function>) -> bool {
    fn place(p: &Place, names: &HashSet<VarIdent>) -> bool {
        match &p.x {
            PlaceX::Local(n) => names.contains(n),
            PlaceX::Field(_, p) => place(p, names),
            PlaceX::Temporary(_) => true, // expressions are visited below
            _ => false,
        }
    }
    let names: HashSet<_> = f.x.params.iter().map(|p| p.x.name.clone()).collect();
    let mut n = NODES;
    map_expr_visitor_env(
        e,
        &mut VisitorScopeMap::new(),
        &mut n,
        &|n, _, e| {
            *n = n.checked_sub(1).ok_or_else(|| unsupported(&e.span, "set term budget"))?;
            let yes = !mutable_type(&e.typ)
                && match &e.x {
                    ExprX::Const(_) => true,
                    ExprX::Var(v) => names.contains(v),
                    ExprX::ReadPlace(_, k) => matches!(k.preliminary_kind, ReadKind::Spec),
                    ExprX::Unary(UnaryOp::Clip { .. } | UnaryOp::Not, _) => true,
                    ExprX::UnaryOpr(UnaryOpr::Field(_), _)
                    | ExprX::Binary(..)
                    | ExprX::BinaryOpr(..)
                    | ExprX::Logical(..)
                    | ExprX::If(_, _, Some(_)) => true,
                    ExprX::Block(ss, Some(_)) => ss.is_empty(),
                    ExprX::Call {
                        target: CallTarget::Fun(CallTargetKind::Static, n, ts, _, a),
                        post_args: None,
                        body: None,
                        args,
                    } => all.get(n).is_some_and(|g| {
                        g.x.mode == Mode::Spec
                            && crate::ast_util::is_visible_to_opt(
                                &g.x.visibility,
                                &f.x.owning_module,
                            )
                            && !g.x.attrs.prophecy_dependent
                            && !g.x.attrs.is_unsafe
                            && !g.x.attrs.is_async
                            && g.x.proxy.is_none()
                            && !a.assume_external_allowed
                            && !a.const_var
                            && ts.len() == g.x.typ_params.len()
                            && args.len() == g.x.params.len()
                            && g.x.params.iter().all(|p| p.x.mode == Mode::Spec && !p.x.user_mut)
                    }),
                    _ => false,
                };
            if yes {
                Ok(e.clone())
            } else {
                Err(unsupported(&e.span, "non parameter-rooted set term"))
            }
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, _, p| {
            if !mutable_type(&p.typ) && place(p, &names) {
                Ok(p.clone())
            } else {
                Err(unsupported(&p.span, "mutable or captured set place"))
            }
        },
        &|_, _, p| Ok(p.clone()),
    )
    .is_ok()
}

/// Normalize only ephemeral IDs. Unlike the generic skeleton utility, this
/// retains complete types, namespaces, datatype and call implementation paths.
fn normalized(e: &Expr) -> Expr {
    map_expr_visitor_env(
        e,
        &mut VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            let mut span = e.span.clone();
            span.id = 0;
            let x = match &e.x {
                ExprX::ReadPlace(p, k) => ExprX::ReadPlace(
                    p.clone(),
                    UnfinalizedReadKind { preliminary_kind: k.preliminary_kind, id: 0 },
                ),
                _ => e.x.clone(),
            };
            Ok(SpannedTyped::new(&span, &e.typ, x))
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, _, p| {
            let mut s = p.span.clone();
            s.id = 0;
            Ok(SpannedTyped::new(&s, &p.typ, p.x.clone()))
        },
        &|_, _, p| Ok(p.clone()),
    )
    .expect("normalization")
}
fn compact(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(t)) if ss.is_empty() => compact(t),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Temporary(t) => compact(t),
            _ => e,
        },
        _ => e,
    }
}
fn term_key(e: &Expr) -> Vec<u8> {
    // Source locations are not semantic term identity. Types/impl paths ARE.
    let mut v = serde_json::to_value(normalized(compact(e))).expect("VIR serialization");
    fn strip(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(o) => {
                o.remove("span");
                for v in o.values_mut() {
                    strip(v);
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    strip(v);
                }
            }
            _ => (),
        }
    }
    strip(&mut v);
    bytes(&v)
}
fn instantiate(e: &Expr, g: &Function, ts: &Typs, args: &Exprs) -> Result<Expr, VirErr> {
    let types: HashMap<_, _> = g.x.typ_params.iter().cloned().zip(ts.iter().cloned()).collect();
    let names: HashMap<_, _> =
        g.x.params.iter().map(|p| p.x.name.clone()).zip(args.iter().cloned()).collect();
    // Fail closed rather than risk capture when inserting actual arguments
    // beneath a binder from a different source function.
    let actual_names: HashSet<_> = args
        .iter()
        .flat_map(|a| {
            let mut names = vec![];
            expr_visitor_walk(a, &mut |e| {
                if let ExprX::Var(n) = &e.x {
                    names.push(n.clone());
                }
                if let ExprX::ReadPlace(p, _) = &e.x {
                    if let PlaceX::Local(n) = &p.x {
                        names.push(n.clone());
                    }
                }
                VisitorControlFlow::<()>::Recurse
            });
            names
        })
        .collect();
    let mut capture = false;
    expr_visitor_walk(e, &mut |e| {
        if let ExprX::Quant(_, bs, _) | ExprX::Closure(bs, _) = &e.x {
            capture |= bs.iter().any(|b| actual_names.contains(&b.name));
        }
        VisitorControlFlow::<()>::Recurse
    });
    if capture {
        return Err(unsupported(&e.span, "set observation binder capture"));
    }
    map_expr_visitor_env(
        e,
        &mut VisitorScopeMap::new(),
        &mut (),
        &|_, scope, e| {
            Ok(match &e.x {
                ExprX::Var(n) if scope.get(n).is_none() && names.contains_key(n) => {
                    names[n].clone()
                }
                _ => e.clone(),
            })
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| {
            Ok(match &**t {
                TypX::TypParam(n) => types.get(n).cloned().unwrap_or_else(|| t.clone()),
                _ => t.clone(),
            })
        },
        &|_, scope, p| {
            Ok(match &p.x {
                PlaceX::Local(n) if scope.get(n).is_none() && names.contains_key(n) => {
                    p.new_x(PlaceX::Temporary(names[n].clone()))
                }
                _ => p.clone(),
            })
        },
        &|_, _, p| Ok(p.clone()),
    )
}

fn demanded(f: &Function, all: &HashMap<Fun, Function>) -> Option<Vec<Expr>> {
    let mut pending: Vec<_> =
        f.x.require
            .iter()
            .chain(f.x.ensure.0.iter())
            .map(|e| (e.clone(), Vec::<Fun>::new()))
            .collect();
    let (mut nodes, mut defs) = (NODES, DEFS);
    let mut terms = vec![];
    let mut keys = HashSet::new();
    while let Some((e, stack)) = pending.pop() {
        let mut exhausted = false;
        crate::ast_visitor::expr_visitor_dfs(&e, &mut VisitorScopeMap::new(), &mut |scope, e| {
            if f.x.params.iter().any(|p| scope.get(&p.x.name).is_some()) {
                return VisitorControlFlow::Return;
            }
            if nodes == 0 {
                exhausted = true;
                return VisitorControlFlow::Stop(());
            }
            nodes -= 1;
            if set(&e.typ).is_some() && rooted(e, f, all) {
                // Keep actual compact calls such as allocated(t). Never expand
                // their Set-returning definitions into filter/closure internals.
                if keys.insert(term_key(e)) {
                    terms.push(compact(e).clone());
                }
            }
            if let ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Static, n, ts, _, _),
                args,
                post_args: None,
                body: None,
            } = &e.x
            {
                if let Some(g) = all.get(n).filter(|g| {
                    g.x.name.path.krate != CrateId::Vstd
                        && ordinary(g)
                        && g.x.mode == Mode::Spec
                        && set(&g.x.inner_ret.x.typ).is_none()
                        && g.x.body.is_some()
                        && ts.len() == g.x.typ_params.len()
                        && args.len() == g.x.params.len()
                        && f.x.owning_module.as_ref().is_some_and(|m| {
                            crate::ast_util::is_body_visible_to(&g.x.body_visibility, m)
                        })
                        && args.iter().all(|a| rooted(a, f, all))
                }) {
                    if defs == 0 || stack.contains(n) {
                        exhausted = true;
                        return VisitorControlFlow::Stop(());
                    }
                    defs -= 1;
                    match instantiate(g.x.body.as_ref().unwrap(), g, ts, args) {
                        Ok(body) => {
                            let mut stack = stack.clone();
                            stack.push(n.clone());
                            pending.push((body, stack));
                        }
                        Err(_) => {
                            exhausted = true;
                            return VisitorControlFlow::Stop(());
                        }
                    }
                }
            }
            VisitorControlFlow::Recurse
        });
        if exhausted || terms.len() > TERMS {
            return None;
        }
    }
    // Exactly one bounded closure: intersection operands expose their union.
    let observed = terms.clone();
    for e in observed {
        if let ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, n, ts, ips, attrs),
            args,
            post_args: None,
            body: None,
        } = &e.x
        {
            if models::path_is(&n.path, CrateId::Vstd, &["set", "Set", "intersect"])
                && args.len() == 2
            {
                let unions: Vec<_> = all
                    .values()
                    .filter(|g| {
                        models::path_is(&g.x.name.path, CrateId::Vstd, &["set", "Set", "union"])
                            && g.x.mode == Mode::Spec
                            && g.x.params.len() == 2
                            && g.x.typ_params.len() == ts.len()
                    })
                    .collect();
                if unions.len() != 1 {
                    return None;
                }
                let u = e.new_x(ExprX::Call {
                    target: CallTarget::Fun(
                        CallTargetKind::Static,
                        unions[0].x.name.clone(),
                        ts.clone(),
                        ips.clone(),
                        attrs.clone(),
                    ),
                    args: args.clone(),
                    post_args: None,
                    body: None,
                });
                if !rooted(&u, f, all) {
                    return None;
                }
                if keys.insert(term_key(&u)) {
                    terms.push(u);
                }
            }
        }
    }
    if terms.len() > TERMS { None } else { Some(terms) }
}
fn calls(
    f: &Function,
    all: &HashMap<Fun, Function>,
    lemma: &Function,
) -> Option<(usize, Vec<Expr>)> {
    if !eligible(f) {
        return None;
    }
    let terms = demanded(f, all)?;
    let mut out = vec![];
    for a in &terms {
        for b in &terms {
            if bytes(&a.typ) != bytes(&b.typ) {
                continue;
            }
            out.push(SpannedTyped::new(
                &f.span,
                &crate::ast_util::unit_typ(),
                ExprX::Call {
                    target: CallTarget::Fun(
                        CallTargetKind::Static,
                        lemma.x.name.clone(),
                        Arc::new(vec![set(&a.typ)?.clone()]),
                        Arc::new(vec![]),
                        CallTargetAttrs {
                            autospec: AutospecUsage::Final,
                            const_var: false,
                            assume_external_allowed: false,
                        },
                    ),
                    args: Arc::new(vec![a.clone(), b.clone()]),
                    post_args: None,
                    body: None,
                },
            ));
            if out.len() > PAIRS {
                return None;
            }
        }
    }
    Some((terms.len(), out))
}

struct Fresh {
    used: HashSet<u64>,
    next: u64,
}
impl Fresh {
    fn new(fs: &[Function], imported: &[Function]) -> Self {
        fn reserve(v: &serde_json::Value, used: &mut HashSet<u64>) {
            match v {
                serde_json::Value::Number(n) => {
                    if let Some(n) = n.as_u64() {
                        used.insert(n);
                    }
                }
                serde_json::Value::Object(o) => {
                    for v in o.values() {
                        reserve(v, used)
                    }
                }
                serde_json::Value::Array(a) => {
                    for v in a {
                        reserve(v, used)
                    }
                }
                _ => (),
            }
        }
        let mut used = HashSet::new();
        for f in fs.iter().chain(imported) {
            reserve(&serde_json::to_value(f).unwrap(), &mut used);
        }
        Self { used, next: u64::MAX - 1 }
    }
    fn id(&mut self) -> u64 {
        while !self.used.insert(self.next) {
            self.next = self.next.checked_sub(1).expect("ID exhaustion");
        }
        let n = self.next;
        self.next -= 1;
        n
    }
    fn expr(&mut self, e: &Expr) -> Result<Expr, VirErr> {
        map_expr_visitor_env(
            e,
            &mut VisitorScopeMap::new(),
            self,
            &|a, _, e| {
                let mut s = e.span.clone();
                s.id = a.id();
                let x = match &e.x {
                    ExprX::ReadPlace(p, k) => ExprX::ReadPlace(
                        p.clone(),
                        UnfinalizedReadKind { preliminary_kind: k.preliminary_kind, id: a.id() },
                    ),
                    _ => e.x.clone(),
                };
                Ok(SpannedTyped::new(&s, &e.typ, x))
            },
            &|_, _, s| Ok(vec![s.clone()]),
            &|_, t| Ok(t.clone()),
            &|a, _, p| {
                let mut s = p.span.clone();
                s.id = a.id();
                Ok(SpannedTyped::new(&s, &p.typ, p.x.clone()))
            },
            &|_, _, p| Ok(p.clone()),
        )
    }
}
/// Independent projection plus exact typed/placement validation, followed by an
/// ID audit against all original and previously decorated functions.
fn validate(
    original: &Function,
    candidate: &Function,
    all: &HashMap<Fun, Function>,
    lemma: &Function,
    reserved: &HashSet<u64>,
) -> Result<(), VirErr> {
    let bad = || unsupported(&original.span, "invalid set-pair decoration");
    if !catalog(lemma) {
        return Err(bad());
    }
    let (_, expected) = calls(original, all, lemma).ok_or_else(bad)?;
    let Some(before) = &original.x.body else { return Err(bad()) };
    let Some(after) = &candidate.x.body else { return Err(bad()) };
    let ExprX::Block(ss, tail) = &after.x else { return Err(bad()) };
    if expected.is_empty() || ss.len() < expected.len() {
        return Err(bad());
    }
    let mut ids = reserved.clone();
    for (s, e) in ss.iter().zip(&expected) {
        let StmtX::Expr(actual) = &s.x else { return Err(bad()) };
        if bytes(&normalized(actual)) != bytes(&normalized(e)) || !ids.insert(s.span.id) {
            return Err(bad());
        }
        map_expr_visitor_env(
            actual,
            &mut VisitorScopeMap::new(),
            &mut ids,
            &|ids, _, e| {
                if !ids.insert(e.span.id) {
                    return Err(bad());
                }
                if let ExprX::ReadPlace(_, k) = &e.x {
                    if !ids.insert(k.id) {
                        return Err(bad());
                    }
                }
                Ok(e.clone())
            },
            &|_, _, _| Err(bad()),
            &|_, t| Ok(t.clone()),
            &|ids, _, p| if ids.insert(p.span.id) { Ok(p.clone()) } else { Err(bad()) },
            &|_, _, _| Err(bad()),
        )?;
    }
    let mut x = candidate.x.clone();
    x.body = Some(after.new_x(ExprX::Block(Arc::new(ss[expected.len()..].to_vec()), tail.clone())));
    if bytes(&candidate.new_x(x)) != bytes(original) {
        return Err(bad());
    }
    let _ = before;
    Ok(())
}
pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    // Ambiguous function identity is never resolved by map insertion order.
    let mut names = HashSet::new();
    if fs.iter().chain(imported).any(|f| !names.insert(f.x.name.clone())) {
        return Ok(fs.to_vec());
    }
    let all = pipeline::function_map(fs, imported);
    let catalog: Vec<_> = all.values().filter(|f| catalog(f)).collect();
    if catalog.len() != 1 {
        return Ok(fs.to_vec());
    }
    let lemma = catalog[0];
    let mut fresh = Fresh::new(fs, imported);
    let mut out = vec![];
    for f in fs {
        let Some((terms, calls)) = calls(f, &all, lemma).filter(|(_, c)| !c.is_empty()) else {
            out.push(f.clone());
            continue;
        };
        let reserved = fresh.used.clone();
        let mut prefix = vec![];
        for e in calls {
            let e = fresh.expr(&e)?;
            let mut s = e.span.clone();
            s.id = fresh.id();
            prefix.push(crate::def::Spanned::new(s, StmtX::Expr(e)));
        }
        let count = prefix.len();
        let body = f.x.body.as_ref().unwrap();
        let ExprX::Block(ss, tail) = &body.x else { unreachable!() };
        prefix.extend(ss.iter().cloned());
        let mut x = f.x.clone();
        x.body = Some(body.new_x(ExprX::Block(Arc::new(prefix), tail.clone())));
        let candidate = f.new_x(x);
        validate(f, &candidate, &all, lemma, &reserved)?;
        report.body_rechecks.insert(f.x.name.clone());
        report.set_pairs.push((f.x.name.clone(), terms, count));
        out.push(candidate);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::def::Spanned;
    use crate::infer::tests::span;
    fn path(parts: &[&str]) -> Path {
        Arc::new(PathX {
            krate: CrateId::Vstd,
            segments: Arc::new(parts.iter().map(|s| Arc::new((*s).into())).collect()),
        })
    }
    fn set_typ(t: Typ) -> Typ {
        Arc::new(TypX::Datatype(
            Dt::Path(path(&["set", "Set"])),
            Arc::new(vec![t]),
            Arc::new(vec![]),
        ))
    }
    fn fixture() -> (Function, Function) {
        let base = crate::infer::loops::tests::counted_function();
        let mut x = base.x.clone();
        x.mode = Mode::Proof;
        Arc::make_mut(&mut x.attrs).infer = false;
        x.typ_params = Arc::new(vec![]);
        x.typ_bounds = Arc::new(vec![]);
        x.params = Arc::new(
            (0..2)
                .map(|i| {
                    let mut p = x.params[0].x.clone();
                    p.name = VarIdent(Arc::new(format!("s{i}")), VarIdentDisambiguate::AirLocal);
                    p.mode = Mode::Spec;
                    p.user_mut = false;
                    p.typ = set_typ(int_typ());
                    Spanned::new(span(), p)
                })
                .collect(),
        );
        let mut r = x.inner_ret.x.clone();
        r.typ = crate::ast_util::unit_typ();
        r.mode = Mode::Proof;
        x.inner_ret = Spanned::new(span(), r);
        x.outer_ret = x.inner_ret.clone();
        x.require = Arc::new(
            x.params
                .iter()
                .map(|p| SpannedTyped::new(&span(), &p.x.typ, ExprX::Var(p.x.name.clone())))
                .collect(),
        );
        x.ensure = (Arc::new(vec![]), Arc::new(vec![]));
        x.body = Some(SpannedTyped::new(
            &span(),
            &crate::ast_util::unit_typ(),
            ExprX::Block(Arc::new(vec![]), None),
        ));
        let f = base.new_x(x.clone());
        x.name = Arc::new(FunX { path: path(LEMMA) });
        x.typ_params = Arc::new(vec![Arc::new("A".into())]);
        x.params = Arc::new(
            x.params
                .iter()
                .map(|p| {
                    let mut q = p.x.clone();
                    q.typ = set_typ(Arc::new(TypX::TypParam(x.typ_params[0].clone())));
                    p.new_x(q)
                })
                .collect(),
        );
        x.require = Arc::new(vec![]);
        x.ensure.0 = Arc::new(vec![boolean(&span(), true).expr; 3]);
        x.body = None;
        (f, base.new_x(x))
    }
    fn installed(f: &Function, l: &Function) -> (Function, HashMap<Fun, Function>, HashSet<u64>) {
        let all = pipeline::function_map(&[f.clone()], &[l.clone()]);
        let used = Fresh::new(&[f.clone()], &[l.clone()]).used;
        let out = prepare(&[f.clone()], &[l.clone()], &mut GenerationReport::default()).unwrap();
        (out[0].clone(), all, used)
    }
    #[test]
    fn catalog_and_signature_fail_closed() {
        let (f, l) = fixture();
        assert!(eligible(&f));
        assert!(catalog(&l));
        for i in 0..13 {
            let mut x = l.x.clone();
            match i {
                0 => {
                    let mut p = (*x.name.path).clone();
                    p.krate = CrateId::Alloc;
                    x.name = Arc::new(FunX { path: Arc::new(p) });
                }
                1 => x.mode = Mode::Spec,
                2 => Arc::make_mut(&mut x.attrs).is_external_body = true,
                3 => Arc::make_mut(&mut x.attrs).infer = true,
                4 => Arc::make_mut(&mut x.attrs).prophecy_dependent = true,
                5 => x.require = Arc::new(vec![boolean(&span(), true).expr]),
                6 => x.typ_params = Arc::new(vec![]),
                7 => {
                    let mut p = x.params[0].x.clone();
                    p.mode = Mode::Proof;
                    Arc::make_mut(&mut x.params)[0] = x.params[0].new_x(p);
                }
                8 => {
                    let mut p = x.params[0].x.clone();
                    p.user_mut = true;
                    Arc::make_mut(&mut x.params)[0] = x.params[0].new_x(p);
                }
                9 => {
                    let mut p = x.params[0].x.clone();
                    p.typ = int_typ();
                    Arc::make_mut(&mut x.params)[0] = x.params[0].new_x(p);
                }
                10 => Arc::make_mut(&mut x.attrs).exec_assume_termination = true,
                11 => Arc::make_mut(&mut x.attrs).is_async = true,
                _ => x.ensure.0 = Arc::new(vec![]),
            }
            assert!(!catalog(&l.new_x(x)), "catalog {i}");
        }
        for i in 0..8 {
            let mut x = f.x.clone();
            match i {
                0 => x.mode = Mode::Exec,
                1 => Arc::make_mut(&mut x.attrs).infer = true,
                2 => Arc::make_mut(&mut x.attrs).prophecy_dependent = true,
                3 => Arc::make_mut(&mut x.attrs).is_unsafe = true,
                4 => Arc::make_mut(&mut x.attrs).exec_allows_no_decreases_clause = true,
                5 => {
                    let mut p = x.params[0].x.clone();
                    p.typ = Arc::new(TypX::MutRef(p.typ));
                    Arc::make_mut(&mut x.params)[0] = x.params[0].new_x(p);
                }
                6 => x.body = None,
                _ => {
                    let mut p = x.params[0].x.clone();
                    p.mode = Mode::Exec;
                    Arc::make_mut(&mut x.params)[0] = x.params[0].new_x(p);
                }
            }
            assert!(!eligible(&f.new_x(x)), "signature {i}");
        }
    }
    #[test]
    fn pure_roots_typed_dedup_and_budget() {
        let (f, l) = fixture();
        let all = pipeline::function_map(&[f.clone()], &[l]);
        let e = &f.x.require[0];
        assert!(rooted(e, &f, &all));
        let captured = e.new_x(ExprX::Var(f.x.inner_ret.x.name.clone()));
        assert!(!rooted(&captured, &f, &all));
        let old = e.new_x(ExprX::Old(e.clone()));
        assert!(!rooted(&old, &f, &all));
        let assumed = e.new_x(ExprX::AssertAssume {
            is_assume: true,
            expr: boolean(&span(), true).expr,
            msg: None,
        });
        assert!(!rooted(&assumed, &f, &all));
        let mut x = f.x.clone();
        x.require = Arc::new(vec![e.clone(); 100]);
        assert_eq!(demanded(&f.new_x(x), &all).unwrap().len(), 1);
        let mut x = f.x.clone();
        x.require = Arc::new(vec![e.clone(); NODES + 1]);
        assert!(demanded(&f.new_x(x), &all).is_none());
        let changed = SpannedTyped::new(&e.span, &set_typ(bool_typ()), e.x.clone());
        assert_ne!(term_key(e), term_key(&changed));
    }
    #[test]
    fn whole_function_projection_and_id_controls() {
        let (f, l) = fixture();
        let (candidate, all, reserved) = installed(&f, &l);
        assert_ne!(bytes(&candidate), bytes(&f));
        assert!(validate(&f, &candidate, &all, &l, &reserved).is_ok());
        for i in 0..10 {
            let mut x = candidate.x.clone();
            let body = x.body.clone().unwrap();
            let ExprX::Block(ss, tail) = &body.x else { unreachable!() };
            let mut ss = (**ss).clone();
            match i {
                0 => x.require = Arc::new(vec![]),
                1 => x.decrease = Arc::new(vec![boolean(&span(), false).expr]),
                2 => Arc::make_mut(&mut x.attrs).prophecy_dependent = true,
                3 => {
                    ss.swap(0, 1);
                }
                4 => {
                    ss.push(ss[0].clone());
                }
                5 => {
                    ss.remove(0);
                }
                6 => {
                    let StmtX::Expr(e) = &ss[0].x else { unreachable!() };
                    let mut s = ss[0].span.clone();
                    s.id = *reserved.iter().next().unwrap();
                    ss[0] = Spanned::new(s, StmtX::Expr(e.clone()));
                }
                7 => {
                    let StmtX::Expr(e) = &ss[0].x else { unreachable!() };
                    let ExprX::Call { target, args, .. } = &e.x else { unreachable!() };
                    let mut args = (**args).clone();
                    let TypX::Datatype(dt, ts, _) = &*args[0].typ else { unreachable!() };
                    let t = Arc::new(TypX::Datatype(
                        dt.clone(),
                        ts.clone(),
                        Arc::new(vec![ImplPath::TraitImplPath(path(&["evil"]))]),
                    ));
                    args[0] = SpannedTyped::new(&args[0].span, &t, args[0].x.clone());
                    ss[0] = ss[0].new_x(StmtX::Expr(e.new_x(ExprX::Call {
                        target: target.clone(),
                        args: Arc::new(args),
                        post_args: None,
                        body: None,
                    })));
                }
                8 => {
                    let StmtX::Expr(e) = &ss[0].x else { unreachable!() };
                    ss[0] = ss[0].new_x(StmtX::Expr(e.new_x(ExprX::AssertAssume {
                        is_assume: true,
                        expr: boolean(&span(), false).expr,
                        msg: None,
                    })));
                }
                _ => {
                    x.body = Some(body.new_x(ExprX::Block(
                        Arc::new(ss.clone()),
                        Some(boolean(&span(), false).expr),
                    )))
                }
            }
            if i != 9 {
                x.body = Some(body.new_x(ExprX::Block(Arc::new(ss), tail.clone())));
            }
            assert!(
                validate(&f, &candidate.new_x(x), &all, &l, &reserved).is_err(),
                "mutation {i}"
            );
        }
        // A second invocation sees every ID allocated by the first pass.
        let second =
            prepare(&[candidate.clone()], &[l.clone()], &mut GenerationReport::default()).unwrap();
        let reserved2 = Fresh::new(&[candidate.clone()], &[l.clone()]).used;
        assert!(validate(&candidate, &second[0], &all, &l, &reserved2).is_ok());
    }
    #[test]
    fn visible_substitution_hidden_cycles_and_capture() {
        let (f, l) = fixture();
        let mut gx = l.x.clone();
        gx.name = Arc::new(FunX {
            path: Arc::new(PathX {
                krate: CrateId::Id(Arc::new("local".into()), 99),
                segments: Arc::new(vec![Arc::new("observe".into())]),
            }),
        });
        gx.mode = Mode::Spec;
        gx.require = Arc::new(vec![]);
        gx.ensure = (Arc::new(vec![]), Arc::new(vec![]));
        let mut r = gx.inner_ret.x.clone();
        r.typ = bool_typ();
        r.mode = Mode::Spec;
        gx.inner_ret = gx.inner_ret.new_x(r);
        gx.outer_ret = gx.inner_ret.clone();
        gx.body_visibility = BodyVisibility::Visibility(Visibility { restricted_to: None });
        gx.body = Some(SpannedTyped::new(
            &span(),
            &gx.params[0].x.typ,
            ExprX::Var(gx.params[0].x.name.clone()),
        ));
        let g = l.new_x(gx);
        let mut fx = f.x.clone();
        fx.owning_module = Some(path(&["caller"]));
        let actual = f.x.require.clone();
        let invocation = SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    g.x.name.clone(),
                    Arc::new(vec![int_typ()]),
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: actual.clone(),
                post_args: None,
                body: None,
            },
        );
        fx.require = Arc::new(vec![invocation.clone()]);
        let f = f.new_x(fx);
        let all = pipeline::function_map(&[f.clone(), g.clone()], &[l.clone()]);
        let expanded =
            instantiate(g.x.body.as_ref().unwrap(), &g, &Arc::new(vec![int_typ()]), &actual)
                .unwrap();
        assert_eq!(term_key(&expanded), term_key(&actual[0]));
        let mut hidden = g.x.clone();
        hidden.body_visibility = BodyVisibility::Uninterpreted;
        let hidden = pipeline::function_map(&[f.clone(), g.new_x(hidden)], &[l.clone()]);
        // The actual arguments are visible observations in both cases; hidden
        // definition bodies must never create additional terms.
        assert_eq!(demanded(&f, &hidden).unwrap().len(), 2);
        let mut cycle = g.x.clone();
        cycle.body = Some(invocation);
        let cyclic = pipeline::function_map(&[f.clone(), g.new_x(cycle)], &[l.clone()]);
        assert!(demanded(&f, &cyclic).is_none());
        let bound = actual[0].clone();
        let q = SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::Quant(
                Quant { quant: air::ast::Quant::Forall },
                Arc::new(vec![Arc::new(VarBinderX {
                    name: f.x.params[0].x.name.clone(),
                    a: int_typ(),
                })]),
                bound,
            ),
        );
        assert!(instantiate(&q, &g, &Arc::new(vec![int_typ()]), &actual).is_err());
        assert!(demanded(&f, &all).is_some());
    }
}
