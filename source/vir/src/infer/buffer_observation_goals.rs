//! Ordinary checked snapshots and observations, never update/frame summaries.
use super::native_mutable_for::{self as native, NativeMutableFor};
use super::*;
use crate::ast_visitor::{
    VisitorControlFlow, VisitorScopeMap, ast_visitor_dfs, map_expr_visitor, map_stmt_visitor_env,
};
use std::collections::HashSet;

/// One allocator per preparation invocation, seeded with ALL preceding functions.
/// Reserving every serialized integer is deliberately conservative: spans, reads,
/// temporary/borrow identities and bindings all share a collision-free namespace.
struct Fresh {
    used: HashSet<u64>,
    next: u64,
}
impl Fresh {
    fn new(fs: &[Function], imported: &[Function]) -> Self {
        let mut used = HashSet::new();
        fn visit(v: &serde_json::Value, out: &mut HashSet<u64>) {
            match v {
                serde_json::Value::Number(n) => {
                    if let Some(n) = n.as_u64() {
                        out.insert(n);
                    }
                }
                serde_json::Value::Array(v) => {
                    for v in v {
                        visit(v, out);
                    }
                }
                serde_json::Value::Object(v) => {
                    for v in v.values() {
                        visit(v, out);
                    }
                }
                _ => (),
            }
        }
        for f in fs.iter().chain(imported) {
            visit(&serde_json::to_value(f).expect("VIR serialization"), &mut used);
        }
        Self { used, next: u64::MAX - 1 }
    }
    fn id(&mut self) -> u64 {
        while !self.used.insert(self.next) {
            self.next = self.next.checked_sub(1).expect("observation ID exhaustion");
        }
        let n = self.next;
        self.next = self.next.checked_sub(1).expect("observation ID exhaustion");
        n
    }
    fn binding(&mut self) -> VarIdent {
        VarIdent(Arc::new("__buffer_observation".into()), VarIdentDisambiguate::VirTemp(self.id()))
    }
    fn span(&mut self, s: &Span) -> Span {
        let mut s = s.clone();
        s.id = self.id();
        s
    }
    fn stmt(&mut self, s: &Stmt) -> Result<Stmt, VirErr> {
        Ok(map_stmt_visitor_env(
            s,
            &mut VisitorScopeMap::new(),
            self,
            &|a, _, e| {
                let x = match &e.x {
                    ExprX::ReadPlace(p, _) => ExprX::ReadPlace(
                        p.clone(),
                        UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: a.id() },
                    ),
                    _ => e.x.clone(),
                };
                Ok(SpannedTyped::new(&a.span(&e.span), &e.typ, x))
            },
            &|a, _, s| {
                Ok(vec![crate::def::Spanned::new(
                    a.span(&s.span),
                    match &s.x {
                        StmtX::Expr(e) => StmtX::Expr(e.clone()),
                        StmtX::Decl { pattern, mode, init, els, assert_irrefutable } => {
                            StmtX::Decl {
                                pattern: pattern.clone(),
                                mode: *mode,
                                init: init.clone(),
                                els: els.clone(),
                                assert_irrefutable: *assert_irrefutable,
                            }
                        }
                    },
                )])
            },
            &|_, t| Ok(t.clone()),
            &|a, _, p| Ok(SpannedTyped::new(&a.span(&p.span), &p.typ, p.x.clone())),
            &|a, _, p| Ok(SpannedTyped::new(&a.span(&p.span), &p.typ, p.x.clone())),
        )?
        .pop()
        .unwrap())
    }
    fn expr(&mut self, e: &Expr) -> Result<Expr, VirErr> {
        let s = crate::def::Spanned::new(e.span.clone(), StmtX::Expr(e.clone()));
        let s = self.stmt(&s)?;
        let StmtX::Expr(e) = &s.x else { unreachable!() };
        Ok(e.clone())
    }
}
fn expr(s: &Span, t: &Typ, x: ExprX) -> Expr {
    SpannedTyped::new(s, t, x)
}
fn bin(s: &Span, op: BinaryOp, a: Expr, b: Expr) -> Expr {
    expr(s, &bool_typ(), ExprX::Binary(op, a, b))
}
fn eq(s: &Span, a: Expr, b: Expr) -> Expr {
    bin(s, BinaryOp::Eq(Mode::Spec), a, b)
}
fn num(s: &Span, n: usize) -> Expr {
    expr(s, &int_typ(), ExprX::Const(Constant::Int(BigInt::from(n))))
}
fn and(s: &Span, a: Expr, b: Expr) -> Expr {
    expr(s, &bool_typ(), ExprX::Logical(LogicalOp::And, a, b))
}
fn call(s: &Span, f: &Function, ts: Vec<Typ>, args: Vec<Expr>, t: &Typ) -> Expr {
    expr(
        s,
        t,
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                f.x.name.clone(),
                Arc::new(ts),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(args),
            post_args: None,
            body: None,
        },
    )
}
fn seq_type(t: Typ) -> Typ {
    Arc::new(TypX::Datatype(
        Dt::Path(Arc::new(PathX {
            krate: CrateId::Vstd,
            segments: Arc::new(vec![Arc::new("seq".into()), Arc::new("Seq".into())]),
        })),
        Arc::new(vec![t]),
        Arc::new(vec![]),
    ))
}
fn field(s: &Span, tuple: Expr, i: usize, typ: &Typ) -> Expr {
    let p = SpannedTyped::new(s, &tuple.typ, PlaceX::Temporary(tuple.clone()));
    let p = SpannedTyped::new(s, typ, PlaceX::Field(crate::ast_util::mk_tuple_field_opr(2, i), p));
    expr(
        s,
        typ,
        ExprX::ReadPlace(p, UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: 0 }),
    )
}
fn var(s: &Span, n: &VarIdent, t: &Typ) -> Expr {
    expr(s, t, ExprX::Var(n.clone()))
}
fn snapshot(s: &Span, n: VarIdent, value: Expr) -> Stmt {
    crate::def::Spanned::new(
        s.clone(),
        StmtX::Decl {
            pattern: SpannedTyped::new(
                s,
                &value.typ,
                PatternX::Var(PatternBinding {
                    name: n,
                    by_ref: ByRef::No,
                    typ: value.typ.clone(),
                    user_mut: Some(false),
                    copy: false,
                }),
            ),
            mode: Some((Mode::Spec, DeclProph::DelayedInfer)),
            init: Some(SpannedTyped::new(s, &value.typ, PlaceX::Temporary(value.clone()))),
            els: None,
            assert_irrefutable: false,
        },
    )
}

// Follow the frontend's ghost-initialization scaffold: declaration in the
// surrounding scope, then a checked ghost assignment before any mutable borrow.
fn snapshot_pair(s: &Span, n: VarIdent, value: Expr) -> Vec<Stmt> {
    let decl = snapshot(s, n.clone(), value.clone());
    let StmtX::Decl { pattern, mode, els, assert_irrefutable, .. } = &decl.x else {
        unreachable!()
    };
    let decl = crate::def::Spanned::new(
        s.clone(),
        StmtX::Decl {
            pattern: pattern.clone(),
            mode: *mode,
            init: None,
            els: els.clone(),
            assert_irrefutable: *assert_irrefutable,
        },
    );
    let assign = expr(
        s,
        &crate::ast_util::unit_typ(),
        ExprX::Assign {
            place: SpannedTyped::new(s, &value.typ, PlaceX::Local(n)),
            rhs: value.clone(),
            op: None,
            typ: value.typ.clone(),
            resolve: false,
        },
    );
    let ghost = expr(
        s,
        &crate::ast_util::unit_typ(),
        ExprX::Ghost { alloc_wrapper: false, tracked: false, expr: assign },
    );
    vec![decl, crate::def::Spanned::new(s.clone(), StmtX::Expr(ghost))]
}

struct Catalog {
    array: Function,
    slice: Function,
    seq: Function,
    len: Function,
    index: Function,
}
impl Catalog {
    fn new(all: &HashMap<Fun, Function>) -> Option<Self> {
        let find = |p: &[&str]| {
            all.values().find(|f| models::path_is(&f.x.name.path, CrateId::Vstd, p)).cloned()
        };
        let array = find(&["array", "array_view"])?;
        let slice = find(&["slice", "impl&%0", "view"])?;
        let seq = all.values().find(|f| native::wrapper_member(f, "seq"))?.clone();
        let len = all
            .values()
            .find(|f| collections::model(f) == Some(collections::Model::SeqLen))?
            .clone();
        let index = all
            .values()
            .find(|f| collections::model(f) == Some(collections::Model::SeqIndex))?
            .clone();
        for (f, n) in [(&array, 2), (&slice, 1)] {
            if f.x.mode != Mode::Spec
                || f.x.typ_params.len() != n
                || !native::bounds(f, if n == 2 { "array" } else { "slice" })
                || f.x.params.len() != 1
                || f.x.params[0].x.mode != Mode::Spec
                || f.x.params[0].x.user_mut
                || f.x.proxy.is_some()
                || (n == 2 && f.x.attrs.is_external_body)
                || !f.x.require.is_empty()
            {
                return None;
            }
            let g = Arc::new(TypX::TypParam(f.x.typ_params[0].clone()));
            if !native::same(&f.x.inner_ret.x.typ, &seq_type(g.clone())) {
                return None;
            }
            let expected = if n == 2 {
                Arc::new(TypX::Primitive(
                    Primitive::Array,
                    Arc::new(vec![g, Arc::new(TypX::TypParam(f.x.typ_params[1].clone()))]),
                ))
            } else {
                Arc::new(TypX::Decorate(
                    TypDecoration::Ref,
                    None,
                    Arc::new(TypX::Primitive(Primitive::Slice, Arc::new(vec![g]))),
                ))
            };
            if !native::same(&f.x.params[0].x.typ, &expected) {
                return None;
            }
        }
        if !matches!(&slice.x.kind,FunctionKind::TraitMethodImpl {method,trait_path,trait_typ_args,inherit_body_from:None,..}
            if models::path_is(&method.path,CrateId::Vstd,&["view","View","view"])
            && models::path_is(trait_path,CrateId::Vstd,&["view","View"])
            && trait_typ_args.len()==1
            && matches!(&*trait_typ_args[0],TypX::Primitive(Primitive::Slice,ts) if ts.len()==1 && matches!(&*ts[0],TypX::TypParam(n) if n==&slice.x.typ_params[0])))
        {
            return None;
        }
        Some(Self { array, slice, seq, len, index })
    }
    fn key_view(&self, s: &Span, key: Expr) -> Expr {
        let FunctionKind::TraitMethodImpl { method, impl_path, .. } = &self.slice.x.kind else {
            unreachable!()
        };
        let TypX::Decorate(TypDecoration::Ref, None, t) = &*key.typ else { unreachable!() };
        let b = Arc::new(TypX::Int(IntRange::U(8)));
        expr(
            s,
            &seq_type(b.clone()),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::DynamicResolved {
                        resolved: self.slice.x.name.clone(),
                        typs: Arc::new(vec![b]),
                        impl_paths: Arc::new(vec![]),
                        is_trait_default: false,
                    },
                    method.clone(),
                    Arc::new(vec![t.clone()]),
                    Arc::new(vec![ImplPath::TraitImplPath(impl_path.clone())]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(vec![key.clone()]),
                post_args: None,
                body: None,
            },
        )
    }
    fn len(&self, s: &Span, e: Expr) -> Expr {
        let t = iterators::sequence(&e.typ).unwrap().clone();
        call(s, &self.len, vec![t], vec![e], &Arc::new(TypX::Int(IntRange::Nat)))
    }
    fn index(&self, s: &Span, e: Expr, j: Expr) -> Expr {
        let t = iterators::sequence(&e.typ).unwrap().clone();
        call(s, &self.index, vec![t.clone()], vec![e, j], &t)
    }
}
#[derive(Clone)]
struct Decoration {
    loop_id: u64,
    prefix: Vec<Stmt>,
    invariants: Vec<LoopInvariant>,
}

fn proposals(d: &NativeMutableFor, c: &Catalog, a: &mut Fresh) -> Result<Decoration, VirErr> {
    let s = &d.loop_expr.span;
    let TypX::MutRef(t) = &*d.buffer.x.typ else { unreachable!() };
    let b = var(s, &d.buffer.x.name, &d.buffer.x.typ);
    let u8t = Arc::new(TypX::Int(IntRange::U(8)));
    let seq8 = seq_type(u8t.clone());
    let before = a.binding();
    let future = a.binding();
    let snapshot_value = |op| {
        call(
            s,
            &c.array,
            vec![u8t.clone(), Arc::new(TypX::ConstInt(BigInt::from(d.capacity)))],
            vec![expr(s, t, ExprX::Unary(op, b.clone()))],
            &seq8,
        )
    };
    let prefix = snapshot_pair(s, before.clone(), snapshot_value(UnaryOp::MutRefCurrent))
        .into_iter()
        .chain(snapshot_pair(
            s,
            future.clone(),
            snapshot_value(UnaryOp::MutRefFuture(MutRefFutureSourceName::Final)),
        ))
        .map(|s| a.stmt(&s))
        .collect::<Result<Vec<_>, _>>()?;
    let before = var(s, &before, &seq8);
    let future = var(s, &future, &seq8);
    let mutref = Arc::new(TypX::MutRef(u8t.clone()));
    let item = if d.key.is_some() {
        Arc::new(TypX::Datatype(
            Dt::Tuple(2),
            Arc::new(vec![
                mutref.clone(),
                Arc::new(TypX::Decorate(TypDecoration::Ref, None, u8t.clone())),
            ]),
            Arc::new(vec![]),
        ))
    } else {
        mutref.clone()
    };
    let seq = call(s, &c.seq, vec![d.iterator.clone()], vec![d.wrapper.clone()], &seq_type(item));
    let key = d.key.as_ref().map(|k| c.key_view(s, k.clone()));
    let length = if let Some(key) = &key {
        let len = c.len(s, key.clone());
        expr(
            s,
            &Arc::new(TypX::Int(IntRange::Nat)),
            ExprX::If(
                bin(s, BinaryOp::Inequality(InequalityOp::Lt), len.clone(), num(s, d.capacity)),
                len,
                Some(num(s, d.capacity)),
            ),
        )
    } else {
        num(s, d.capacity)
    };
    let mut invs = vec![
        eq(s, c.len(s, before.clone()), num(s, d.capacity)),
        eq(s, c.len(s, future.clone()), num(s, d.capacity)),
        eq(s, c.len(s, seq.clone()), length),
    ];
    let jn = a.binding();
    let j = var(s, &jn, &int_typ());
    let at = c.index(s, seq.clone(), j.clone());
    let r = if key.is_some() { field(s, at.clone(), 0, &mutref) } else { at.clone() };
    let cur = expr(s, &u8t, ExprX::Unary(UnaryOp::MutRefCurrent, r.clone()));
    let fin = expr(s, &u8t, ExprX::Unary(UnaryOp::MutRefFuture(MutRefFutureSourceName::Final), r));
    let mut facts = and(
        s,
        eq(s, cur, c.index(s, before, j.clone())),
        eq(s, fin, c.index(s, future, j.clone())),
    );
    if let Some(key) = key {
        let shr = field(s, at, 1, &Arc::new(TypX::Decorate(TypDecoration::Ref, None, u8t.clone())));
        // Shared references are value-transparent in VIR; source deref is erased.
        facts = and(s, facts, eq(s, shr, c.index(s, key, j.clone())));
    }
    let guard = and(
        s,
        bin(s, BinaryOp::Inequality(InequalityOp::Le), num(s, 0), j.clone()),
        bin(s, BinaryOp::Inequality(InequalityOp::Lt), j, c.len(s, seq)),
    );
    invs.push(expr(
        s,
        &bool_typ(),
        ExprX::Quant(
            Quant { quant: air::ast::Quant::Forall },
            Arc::new(vec![Arc::new(VarBinderX { name: jn, a: int_typ() })]),
            expr(s, &bool_typ(), ExprX::Logical(LogicalOp::Implies, guard, facts)),
        ),
    ));
    let invariants = invs
        .iter()
        .map(|e| {
            Ok(LoopInvariant { kind: LoopInvariantKind::InvariantAndEnsures, inv: a.expr(e)? })
        })
        .collect::<Result<Vec<_>, VirErr>>()?;
    Ok(Decoration { loop_id: d.loop_expr.span.id, prefix, invariants })
}
fn bytes<T: serde::Serialize + ?Sized>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).expect("VIR serialization")
}
fn rewrite_loop(e: &Expr, invs: LoopInvariants) -> Expr {
    let ExprX::Loop {
        loop_isolation,
        allow_complex_invariants,
        is_for_loop,
        assume_termination,
        label,
        cond,
        body,
        decrease,
        atomic_call,
        ..
    } = &e.x
    else {
        unreachable!()
    };
    e.new_x(ExprX::Loop {
        loop_isolation: *loop_isolation,
        allow_complex_invariants: *allow_complex_invariants,
        is_for_loop: *is_for_loop,
        assume_termination: *assume_termination,
        label: label.clone(),
        cond: cond.clone(),
        body: body.clone(),
        invs,
        decrease: decrease.clone(),
        atomic_call: *atomic_call,
    })
}
/// Independent whole-Function projection: only our exact prefix and appended
/// invariants may disappear. All executable/resolution nodes and original loop
/// invariants, termination, contracts, signature and source metadata must match.
// Independent ID audit, separate from allocation and from projection.
fn node_ids<T: serde::Serialize + ?Sized>(value: &T) -> Vec<u64> {
    fn walk(v: &serde_json::Value, out: &mut Vec<u64>) {
        match v {
            serde_json::Value::Object(m) => {
                if (m.contains_key("as_string") && m.contains_key("data"))
                    || m.contains_key("preliminary_kind")
                {
                    if let Some(n) = m.get("id").and_then(|n| n.as_u64()) {
                        out.push(n);
                    }
                }
                for v in m.values() {
                    walk(v, out);
                }
            }
            serde_json::Value::Array(v) => {
                for v in v {
                    walk(v, out);
                }
            }
            _ => (),
        }
    }
    let mut out = vec![];
    walk(&serde_json::to_value(value).expect("VIR serialization"), &mut out);
    out
}
fn validate(original: &Function, decorated: &Function, d: &Decoration) -> Result<(), VirErr> {
    let bad = || unsupported(&original.span, "buffer observation whole-function projection");
    if d.prefix.len() != 4 || d.invariants.len() != 4 {
        return Err(bad());
    }
    // Independently constrain the erased scaffold, even if a corrupted producer
    // supplies a matching receipt. A ghost assignment may initialize only its
    // own fresh declaration, never an executable input or existing local.
    fn contains(value: &serde_json::Value, needle: &serde_json::Value) -> bool {
        value == needle
            || match value {
                serde_json::Value::Array(v) => v.iter().any(|v| contains(v, needle)),
                serde_json::Value::Object(m) => m.values().any(|v| contains(v, needle)),
                _ => false,
            }
    }
    let original_json = serde_json::to_value(original).expect("VIR function");
    let mut names = HashSet::new();
    for pair in d.prefix.chunks_exact(2) {
        let StmtX::Decl {
            pattern,
            mode: Some((Mode::Spec, DeclProph::DelayedInfer)),
            init: None,
            els: None,
            ..
        } = &pair[0].x
        else {
            return Err(bad());
        };
        let PatternX::Var(binding) = &pattern.x else { return Err(bad()) };
        if contains(&original_json, &serde_json::to_value(&binding.name).expect("VIR binding"))
            || !names.insert(binding.name.clone())
        {
            return Err(bad());
        }
        let StmtX::Expr(g) = &pair[1].x else { return Err(bad()) };
        let ExprX::Ghost { alloc_wrapper: false, tracked: false, expr: assign } = &g.x else {
            return Err(bad());
        };
        let ExprX::Assign { place, rhs, op: None, typ, resolve: false } = &assign.x else {
            return Err(bad());
        };
        if !matches!(&place.x,PlaceX::Local(n) if n==&binding.name)
            || !native::same(&place.typ, &binding.typ)
            || !native::same(&rhs.typ, &binding.typ)
            || !native::same(typ, &binding.typ)
        {
            return Err(bad());
        }
    }
    fn binders(v: &serde_json::Value, out: &mut Vec<VarIdent>) {
        match v {
            serde_json::Value::Object(m) => {
                if m.contains_key("a") {
                    if let Some(name) = m.get("name") {
                        if let Ok(n) = serde_json::from_value::<VarIdent>(name.clone()) {
                            out.push(n);
                        }
                    }
                }
                for v in m.values() {
                    binders(v, out);
                }
            }
            serde_json::Value::Array(v) => {
                for v in v {
                    binders(v, out);
                }
            }
            _ => (),
        }
    }
    let mut generated_binders = vec![];
    binders(&serde_json::to_value(&d.invariants).expect("VIR invariants"), &mut generated_binders);
    for binding in generated_binders {
        if contains(&original_json, &serde_json::to_value(&binding).expect("VIR binding"))
            || !names.insert(binding)
        {
            return Err(bad());
        }
    }
    let mut used: HashSet<_> = node_ids(original).into_iter().collect();
    for n in node_ids(&d.prefix).into_iter().chain(node_ids(&d.invariants)) {
        if !used.insert(n) {
            return Err(bad());
        }
    }
    let body = decorated.x.body.as_ref().ok_or_else(bad)?;
    let ExprX::Block(stmts, tail) = &body.x else { return Err(bad()) };
    if stmts.len() < d.prefix.len() || bytes(&stmts[..d.prefix.len()]) != bytes(&d.prefix) {
        return Err(bad());
    }
    let stripped =
        body.new_x(ExprX::Block(Arc::new(stmts[d.prefix.len()..].to_vec()), tail.clone()));
    let count = std::cell::Cell::new(0);
    let projected = map_expr_visitor(&stripped, &|e| {
        if !matches!(&e.x, ExprX::Loop { .. }) || e.span.id != d.loop_id {
            return Ok(e.clone());
        }
        let ExprX::Loop { invs, .. } = &e.x else { unreachable!() };
        if invs.len() < d.invariants.len()
            || bytes(&invs[invs.len() - d.invariants.len()..]) != bytes(&d.invariants)
        {
            return Err(bad());
        }
        count.set(count.get() + 1);
        Ok(rewrite_loop(e, Arc::new(invs[..invs.len() - d.invariants.len()].to_vec())))
    })?;
    let mut x = decorated.x.clone();
    x.body = Some(projected);
    if count.get() != 1 || bytes(&original) != bytes(&decorated.new_x(x)) {
        return Err(bad());
    }
    Ok(())
}
fn install(f: &Function, d: &Decoration, all: &HashMap<Fun, Function>) -> Result<Function, VirErr> {
    for i in &d.invariants {
        super::assistance::pure(&i.inv, all)?;
    }
    for s in d.prefix.iter().skip(1).step_by(2) {
        if let StmtX::Expr(g) = &s.x {
            if let ExprX::Ghost { expr, .. } = &g.x {
                if let ExprX::Assign { rhs, .. } = &expr.x {
                    super::assistance::pure(rhs, all)?;
                }
            }
        }
    }
    let original = f.x.body.as_ref().unwrap();
    let body = map_expr_visitor(original, &|e| {
        if !matches!(&e.x, ExprX::Loop { .. }) || e.span.id != d.loop_id {
            return Ok(e.clone());
        }
        let ExprX::Loop { invs, .. } = &e.x else { unreachable!() };
        let mut invs = invs.as_ref().clone();
        invs.extend(d.invariants.clone());
        Ok(rewrite_loop(e, Arc::new(invs)))
    })?;
    let ExprX::Block(stmts, tail) = &body.x else { return Ok(f.clone()) };
    let mut prefix = d.prefix.clone();
    prefix.extend(stmts.iter().cloned());
    let mut x = f.x.clone();
    x.body = Some(body.new_x(ExprX::Block(Arc::new(prefix), tail.clone())));
    let decorated = f.new_x(x);
    validate(f, &decorated, d)?;
    Ok(decorated)
}

/// Entry/final snapshots describe the iterator borrow only when no other
/// mutable use of the buffer can change its state outside the selected loop.
/// Inspect places as well as expressions: declarations and match patterns can
/// acquire mutable aliases without an expression-level read or reborrow node.
fn entry_observations_compatible(f: &Function, d: &NativeMutableFor) -> bool {
    fn data_value(t: &Typ) -> bool {
        matches!(&**t, TypX::Bool | TypX::Int(_)) || native::array(t).is_some()
    }
    // A readonly place may still execute side effects in an index, temporary or
    // WithExpr. Check those expressions normally instead of skipping the tree.
    fn readonly_place(p: &Place, d: &NativeMutableFor) -> bool {
        match &p.x {
            PlaceX::Local(_) => true,
            PlaceX::Field(_, p)
            | PlaceX::DerefMut(p)
            | PlaceX::ModeUnwrap(p, _)
            | PlaceX::UserDefinedTypInvariantObligation(p, _) => readonly_place(p, d),
            PlaceX::Index(p, i, ..) => readonly_place(p, d) && expression(i, d),
            PlaceX::WithExpr(e, p) => expression(e, d) && readonly_place(p, d),
            PlaceX::Temporary(e) => expression(e, d),
        }
    }
    fn expression(e: &Expr, d: &NativeMutableFor) -> bool {
        !matches!(
            ast_visitor_dfs(
                e,
                &mut VisitorScopeMap::new(),
                &mut (),
                &mut |_, _, e| {
                    if e.span.id == d.loop_expr.span.id
                        || e.span.id == d.iterator_init.span.id
                        || matches!(&e.x, ExprX::Ghost { .. })
                    {
                        return VisitorControlFlow::Return;
                    }
                    if let ExprX::ReadPlace(p, kind) = &e.x {
                        let readonly = matches!(
                            kind.preliminary_kind,
                            ReadKind::Spec | ReadKind::SpecAfterBorrow | ReadKind::ImmutBor
                        ) || data_value(&p.typ);
                        if readonly {
                            return if readonly_place(p, d) {
                                VisitorControlFlow::Return
                            } else {
                                VisitorControlFlow::Stop(())
                            };
                        }
                    }
                    if matches!(&e.x, ExprX::Var(v) | ExprX::VarAt(v, _) if v == &d.buffer.x.name) {
                        return VisitorControlFlow::Stop(());
                    }
                    VisitorControlFlow::Recurse
                },
                &mut |_, _, s| {
                    if matches!(&s.x, StmtX::Decl { mode: Some((Mode::Spec | Mode::Proof, _)), .. })
                    {
                        return VisitorControlFlow::Return;
                    }
                    if let StmtX::Decl { pattern, init: Some(p), els: None, .. } = &s.x {
                        let by_value = match &pattern.x {
                            PatternX::Wildcard => true,
                            PatternX::Var(binding) => matches!(binding.by_ref, ByRef::No),
                            _ => false,
                        };
                        if by_value && data_value(&p.typ) {
                            return if readonly_place(p, d) {
                                VisitorControlFlow::Return
                            } else {
                                VisitorControlFlow::Stop(())
                            };
                        }
                    }
                    VisitorControlFlow::Recurse
                },
                &mut |_, _, _| VisitorControlFlow::Recurse,
                &mut |_, _, _, _| VisitorControlFlow::Recurse,
                &mut |_, _, p| {
                    if matches!(&p.x, PlaceX::Local(v) if v == &d.buffer.x.name) {
                        VisitorControlFlow::Stop(())
                    } else {
                        VisitorControlFlow::Recurse
                    }
                },
            ),
            VisitorControlFlow::Stop(())
        )
    }
    f.x.body.as_ref().is_some_and(|body| expression(body, d))
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = super::pipeline::function_map(fs, imported);
    let Some(c) = Catalog::new(&all) else { return Ok(fs.to_vec()) };
    let mut fresh = Fresh::new(fs, imported);
    fs.iter()
        .map(|f| {
            let Some(d) = native::describe(f, &all) else { return Ok(f.clone()) };
            if !entry_observations_compatible(f, &d) {
                return Ok(f.clone());
            }
            let p = proposals(&d, &c, &mut fresh)?;
            let f = install(f, &p, &all)?;
            report.body_rechecks.insert(f.x.name.clone());
            report.buffer_observations.push((f.x.name.clone(), d.loop_expr.span.clone()));
            Ok(f)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input, span};
    fn fixture() -> (Function, Decoration) {
        let s = span();
        let t = expr(&s, &bool_typ(), ExprX::Const(Constant::Bool(true)));
        let lp = expr(
            &s,
            &crate::ast_util::unit_typ(),
            ExprX::Loop {
                loop_isolation: true,
                allow_complex_invariants: true,
                is_for_loop: true,
                assume_termination: false,
                label: Label { id: 0, name: None },
                cond: None,
                body: input("x"),
                invs: Arc::new(vec![LoopInvariant {
                    kind: LoopInvariantKind::InvariantExceptBreak,
                    inv: t.clone(),
                }]),
                decrease: Arc::new(vec![num(&s, 7)]),
                atomic_call: false,
            },
        );
        let body = expr(
            &s,
            &crate::ast_util::unit_typ(),
            ExprX::Block(
                Arc::new(vec![crate::def::Spanned::new(s.clone(), StmtX::Expr(lp))]),
                None,
            ),
        );
        let f = function("client", body);
        let mut a = Fresh::new(&[f.clone()], &[]);
        let n1 = a.binding();
        let n2 = a.binding();
        let prefix = snapshot_pair(&s, n1, t.clone())
            .into_iter()
            .chain(snapshot_pair(&s, n2, t.clone()))
            .map(|s| a.stmt(&s).unwrap())
            .collect();
        let invariants = (0..4)
            .map(|_| LoopInvariant {
                kind: LoopInvariantKind::InvariantAndEnsures,
                inv: a.expr(&t).unwrap(),
            })
            .collect();
        (f, Decoration { loop_id: s.id, prefix, invariants })
    }
    fn body_map(f: &Function, fe: impl Fn(&Expr) -> Result<Expr, VirErr>) -> Function {
        let mut x = f.x.clone();
        x.body = Some(map_expr_visitor(x.body.as_ref().unwrap(), &fe).unwrap());
        f.new_x(x)
    }
    #[test]
    fn projection_preserves_entire_function_and_rejects_mutations() {
        let (f, d) = fixture();
        let g = install(&f, &d, &HashMap::new()).unwrap();
        validate(&f, &g, &d).unwrap();
        let mut x = g.x.clone();
        x.require = Arc::new(vec![input("x")]);
        assert!(validate(&f, &g.new_x(x), &d).is_err());
        let altered = body_map(&g, |e| {
            Ok(match &e.x {
                ExprX::Var(n) if n.0.as_str() == "x" => num(&e.span, 3),
                _ => e.clone(),
            })
        });
        assert!(validate(&f, &altered, &d).is_err());
        for which in 0..4 {
            let altered = body_map(&g, |e| {
                let ExprX::Loop { invs, decrease, .. } = &e.x else { return Ok(e.clone()) };
                let mut e = if which == 0 {
                    let mut invs = invs.as_ref().clone();
                    invs[0].inv = num(&e.span, 99);
                    rewrite_loop(e, Arc::new(invs))
                } else if which == 1 {
                    let mut invs = invs.as_ref().clone();
                    invs.extend(d.invariants.clone());
                    rewrite_loop(e, Arc::new(invs))
                } else if which == 2 {
                    let mut invs = invs.as_ref().clone();
                    invs.swap(0, 1);
                    rewrite_loop(e, Arc::new(invs))
                } else {
                    e.clone()
                };
                if which == 3 {
                    let mut v = serde_json::to_value(&e).unwrap();
                    v["x"]["Loop"]["decrease"] =
                        serde_json::to_value(Arc::new(vec![num(&e.span, 8)])).unwrap();
                    e = serde_json::from_value(v).unwrap();
                }
                let _ = decrease;
                Ok(e)
            });
            assert!(validate(&f, &altered, &d).is_err());
        }
        let ExprX::Block(ss, tail) = &g.x.body.as_ref().unwrap().x else { unreachable!() };
        let mut reordered = ss.as_ref().clone();
        reordered.swap(0, 1);
        let mut x = g.x.clone();
        x.body =
            Some(g.x.body.as_ref().unwrap().new_x(ExprX::Block(Arc::new(reordered), tail.clone())));
        assert!(validate(&f, &g.new_x(x), &d).is_err());
    }
    #[test]
    fn snapshot_schema_rejects_mode_existing_binding_and_executable_writes() {
        let (f, d) = fixture();
        for control in 0..3 {
            let mut d = d.clone();
            let StmtX::Decl { pattern, mode, init, els, assert_irrefutable } = &d.prefix[0].x
            else {
                unreachable!()
            };
            if control == 0 {
                d.prefix[0] = crate::def::Spanned::new(
                    d.prefix[0].span.clone(),
                    StmtX::Decl {
                        pattern: pattern.clone(),
                        mode: Some((Mode::Exec, DeclProph::Default)),
                        init: init.clone(),
                        els: els.clone(),
                        assert_irrefutable: *assert_irrefutable,
                    },
                );
            } else if control == 1 {
                let PatternX::Var(b) = &pattern.x else { unreachable!() };
                let mut b = b.clone();
                b.name = f.x.params[0].x.name.clone();
                d.prefix[0] = crate::def::Spanned::new(
                    d.prefix[0].span.clone(),
                    StmtX::Decl {
                        pattern: pattern.new_x(PatternX::Var(b)),
                        mode: *mode,
                        init: init.clone(),
                        els: els.clone(),
                        assert_irrefutable: *assert_irrefutable,
                    },
                );
            } else {
                let StmtX::Expr(g) = &d.prefix[1].x else { unreachable!() };
                let ExprX::Ghost { expr, .. } = &g.x else { unreachable!() };
                let ExprX::Assign { place: _, rhs, op, typ, resolve } = &expr.x else {
                    unreachable!()
                };
                let place = SpannedTyped::new(
                    &g.span,
                    &rhs.typ,
                    PlaceX::Local(f.x.params[0].x.name.clone()),
                );
                let assign = expr.new_x(ExprX::Assign {
                    place,
                    rhs: rhs.clone(),
                    op: *op,
                    typ: typ.clone(),
                    resolve: *resolve,
                });
                d.prefix[1] = crate::def::Spanned::new(
                    d.prefix[1].span.clone(),
                    StmtX::Expr(g.new_x(ExprX::Ghost {
                        alloc_wrapper: false,
                        tracked: false,
                        expr: assign,
                    })),
                );
            }
            assert!(install(&f, &d, &HashMap::new()).is_err());
        }
    }
    #[test]
    fn consistent_binding_corruption_keeps_nodes_fresh_but_is_rejected() {
        let (f, d) = fixture();
        let local = VarIdent(Arc::new("existing_local".into()), VarIdentDisambiguate::AirLocal);
        let quantified =
            VarIdent(Arc::new("existing_quantifier".into()), VarIdentDisambiguate::AirLocal);
        let truth = expr(&span(), &bool_typ(), ExprX::Const(Constant::Bool(true)));
        let mut fx = f.x.clone();
        fx.require = Arc::new(vec![expr(
            &span(),
            &bool_typ(),
            ExprX::Quant(
                Quant { quant: air::ast::Quant::Forall },
                Arc::new(vec![Arc::new(VarBinderX { name: quantified.clone(), a: int_typ() })]),
                truth.clone(),
            ),
        )]);
        let body = fx.body.as_ref().unwrap();
        let ExprX::Block(stmts, tail) = &body.x else { unreachable!() };
        let mut ss = snapshot_pair(&span(), local.clone(), truth);
        ss.extend(stmts.iter().cloned());
        fx.body = Some(body.new_x(ExprX::Block(Arc::new(ss), tail.clone())));
        let f = f.new_x(fx);
        let g = install(&f, &d, &HashMap::new()).unwrap();
        let StmtX::Decl { pattern, .. } = &d.prefix[0].x else { unreachable!() };
        let PatternX::Var(b) = &pattern.x else { unreachable!() };
        // Rename declaration, assignment and every read together, without
        // changing any expression/read/span IDs, in both body and receipt.
        fn rename<T: serde::Serialize + serde::de::DeserializeOwned>(
            v: &T,
            old: &VarIdent,
            new: &VarIdent,
        ) -> T {
            fn walk(v: &mut serde_json::Value, old: &serde_json::Value, new: &serde_json::Value) {
                if v == old {
                    *v = new.clone();
                    return;
                }
                match v {
                    serde_json::Value::Object(m) => {
                        for v in m.values_mut() {
                            walk(v, old, new)
                        }
                    }
                    serde_json::Value::Array(v) => {
                        for v in v {
                            walk(v, old, new)
                        }
                    }
                    _ => (),
                }
            }
            let mut v = serde_json::to_value(v).unwrap();
            walk(&mut v, &serde_json::to_value(old).unwrap(), &serde_json::to_value(new).unwrap());
            serde_json::from_value(v).unwrap()
        }
        let old = b.name.clone();
        for new in [f.x.params[0].x.name.clone(), local, quantified] {
            let mut corrupt = d.clone();
            corrupt.prefix = rename(&d.prefix, &old, &new);
            corrupt.invariants = rename(&d.invariants, &old, &new);
            let changed: Function = rename(&g, &old, &new);
            assert_eq!(node_ids(&g), node_ids(&changed));
            assert!(validate(&f, &changed, &corrupt).is_err());
        }
        // Compose on the already decorated function, then consistently rename
        // only the second pass's new binder to the first pass's existing binder.
        let mut a = Fresh::new(&[g.clone()], &[]);
        let mut second = d.clone();
        let n1 = a.binding();
        let n2 = a.binding();
        let value = expr(&span(), &bool_typ(), ExprX::Const(Constant::Bool(true)));
        second.prefix = snapshot_pair(&span(), n1.clone(), value.clone())
            .into_iter()
            .chain(snapshot_pair(&span(), n2, value))
            .map(|s| a.stmt(&s).unwrap())
            .collect();
        second.invariants = second
            .invariants
            .iter()
            .map(|i| LoopInvariant { kind: i.kind, inv: a.expr(&i.inv).unwrap() })
            .collect();
        let g2 = install(&g, &second, &HashMap::new()).unwrap();
        let mut reuse = second.clone();
        reuse.prefix = rename(&second.prefix, &n1, &old);
        reuse.invariants = rename(&second.invariants, &n1, &old);
        let reused: Function = rename(&g2, &n1, &old);
        assert_eq!(node_ids(&g2), node_ids(&reused));
        assert!(validate(&g, &reused, &reuse).is_err());
        // Duplicate binder identities across independent quantifiers are invalid
        // even when every node ID has been freshly allocated.
        let mut a = Fresh::new(&[f.clone()], &[]);
        let n = a.binding();
        let q = expr(
            &span(),
            &bool_typ(),
            ExprX::Quant(
                Quant { quant: air::ast::Quant::Forall },
                Arc::new(vec![Arc::new(VarBinderX { name: n, a: int_typ() })]),
                expr(&span(), &bool_typ(), ExprX::Const(Constant::Bool(true))),
            ),
        );
        let mut duplicate = d.clone();
        for i in 0..2 {
            duplicate.invariants[i].inv = a.expr(&q).unwrap();
        }
        assert!(install(&f, &duplicate, &HashMap::new()).is_err());
    }
    #[test]
    fn audit_rejects_collisions_even_if_receipt_is_corrupted() {
        let (f, mut d) = fixture();
        let g = install(&f, &d, &HashMap::new()).unwrap();
        d.invariants[1].inv = d.invariants[0].inv.clone();
        let altered = body_map(&g, |e| {
            let ExprX::Loop { invs, .. } = &e.x else { return Ok(e.clone()) };
            let mut invs = invs[..1].to_vec();
            invs.extend(d.invariants.clone());
            Ok(rewrite_loop(e, Arc::new(invs)))
        });
        assert!(validate(&f, &altered, &d).is_err());
    }
    #[test]
    fn fresh_ids_cover_reads_places_patterns_two_functions_and_composition() {
        let (f, d) = fixture();
        let g = install(&f, &d, &HashMap::new()).unwrap();
        let mut a = Fresh::new(&[f.clone(), g.clone()], &[]);
        let existing: HashSet<_> = node_ids(&[f.clone(), g.clone()]).into_iter().collect();
        let p = SpannedTyped::new(
            &span(),
            &int_typ(),
            PlaceX::Local(VarIdent(Arc::new("x".into()), VarIdentDisambiguate::AirLocal)),
        );
        let r = expr(
            &span(),
            &int_typ(),
            ExprX::ReadPlace(p, UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: 0 }),
        );
        let mut seen = existing;
        for _ in 0..2 {
            let n = a.binding();
            let s = a.stmt(&snapshot(&span(), n, r.clone())).unwrap();
            for id in node_ids(&s) {
                assert!(seen.insert(id));
            }
        }
        // Compose the decorator with itself; projection restores the prior pass.
        let mut d2 = d.clone();
        d2.prefix = d2.prefix.iter().map(|s| a.stmt(s).unwrap()).collect();
        // Fresh new snapshot bindings, rather than re-declaring earlier ones.
        d2.prefix = (0..2)
            .flat_map(|_| {
                let n = a.binding();
                snapshot_pair(&span(), n, r.clone())
            })
            .collect::<Vec<_>>()
            .iter()
            .map(|s| a.stmt(s).unwrap())
            .collect();
        d2.invariants = d2
            .invariants
            .iter()
            .map(|i| LoopInvariant { kind: i.kind, inv: a.expr(&i.inv).unwrap() })
            .collect();
        let g2 = install(&g, &d2, &HashMap::new()).unwrap();
        validate(&g, &g2, &d2).unwrap();
        assert!(validate(&f, &g2, &d).is_err());
    }
}
