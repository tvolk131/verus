//! Bounded ordinary-proof assistance for a fixed mutable-byte radix loop.
//!
//! No update summary is created. Every arithmetic, byte and iterator annotation
//! proposed here is checked against the retained body by ordinary verification.
//! Initial scope is intentionally u64/7/[u8;10]; it is not a general effect engine.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor};

const NODES: usize = 8192;
fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(t)) if ss.is_empty() => plain(t),
        ExprX::EvalAndResolve(e, _) | ExprX::NeverToAny(e) => plain(e),
        _ => e,
    }
}
fn local(e: &Expr) -> Option<&VarIdent> {
    match &plain(e).x {
        ExprX::Var(v) => Some(v),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Local(v) => Some(v),
            PlaceX::Temporary(e) => local(e),
            _ => None,
        },
        _ => None,
    }
}
fn constant(e: &Expr, n: usize) -> bool {
    crate::ast_util::const_usize_of_expr(plain(e)) == Some(n)
}
fn integer(t: &Typ, r: IntRange) -> bool {
    matches!(&**t, TypX::Int(a) if *a == r)
}
fn call_name(e: &Expr) -> Option<(&Fun, &Exprs)> {
    match &plain(e).x {
        ExprX::Call { target: CallTarget::Fun(k, n, ..), args, body: None, .. } => Some((
            match k {
                CallTargetKind::DynamicResolved { resolved, .. } => resolved,
                _ => n,
            },
            args,
        )),
        _ => None,
    }
}
fn ordinary(f: &Function) -> bool {
    !f.x.attrs.is_external_body
        && !f.x.attrs.infer
        && !f.x.attrs.is_unsafe
        && !f.x.attrs.is_async
        && !f.x.attrs.exec_assume_termination
        && !f.x.attrs.exec_allows_no_decreases_clause
}
fn library(all: &HashMap<Fun, Function>, path: &[&str], mode: Mode) -> Option<Function> {
    all.values()
        .find(|f| {
            models::path_is(&f.x.name.path, CrateId::Vstd, path)
                && ordinary(f)
                && f.x.mode == mode
                && f.x.proxy.is_none()
                && f.x.typ_params.is_empty()
                && f.x.typ_bounds.is_empty()
        })
        .cloned()
}

// Same frontend-owned scaffold as native_for, accepting an explicit user name
// for the wrapper (`for b in it: ...`) rather than only its generated name.
struct Source {
    loop_expr: Expr,
    iterator: Expr,
    body: Expr,
    wrapper: PatternBinding,
}
fn source(e: &Expr) -> Option<Source> {
    fn decl(s: &Stmt) -> Option<(&Pattern, &Expr)> {
        let StmtX::Decl { pattern, init: Some(p), els: None, .. } = &s.x else { return None };
        let PlaceX::Temporary(e) = &p.x else { return None };
        Some((pattern, e))
    }
    let ExprX::UnaryOpr(UnaryOpr::LoopIsolationBoundary(_), e) = &plain(e).x else { return None };
    let ExprX::Block(ss, Some(_)) = &plain(e).x else { return None };
    if ss.len() != 6 {
        return None;
    }
    let (p, iterator) = decl(&ss[0])?;
    if !matches!(&p.x, PatternX::Var(v) if v.name.0.as_str()=="VERUS_iter_init") {
        return None;
    }
    let (_, m) = decl(&ss[5])?;
    let ExprX::Match(_, arms, _) = &plain(m).x else { return None };
    if arms.len() != 1 {
        return None;
    }
    let PatternX::Var(wrapper) = &arms[0].x.pattern.x else { return None };
    let loop_expr = plain(&arms[0].x.body);
    let ExprX::Loop {
        body, is_for_loop: true, assume_termination: false, atomic_call: false, ..
    } = &loop_expr.x
    else {
        return None;
    };
    let ExprX::Block(ss, None) = &plain(body).x else { return None };
    if ss.len() != 9 {
        return None;
    }
    let (_, body) = decl(&ss[8])?;
    Some(Source {
        loop_expr: loop_expr.clone(),
        iterator: iterator.clone(),
        body: body.clone(),
        wrapper: wrapper.clone(),
    })
}

struct Shape {
    source: Source,
    number: Expr,
    scalar: Expr,
    index: Expr,
    payload: Expr,
    radix: Function,
}
fn shape(f: &Function, all: &HashMap<Fun, Function>) -> Option<Shape> {
    if f.x.mode != Mode::Exec
        || !ordinary(f)
        || f.x.attrs.bit_vector
        || !f.x.typ_params.is_empty()
        || !f.x.typ_bounds.is_empty()
        || f.x.params.len() != 2
        || !integer(&f.x.params[0].x.typ, IntRange::U(64))
    {
        return None;
    }
    let TypX::MutRef(a) = &*f.x.params[1].x.typ else { return None };
    let TypX::Primitive(Primitive::Array, ts) = &**a else { return None };
    if ts.len() != 2
        || !integer(&ts[0], IntRange::U(8))
        || !matches!(&*ts[1], TypX::ConstInt(n) if *n == BigInt::from(10))
    {
        return None;
    }
    let body = f.x.body.as_ref()?;
    let mut budget = NODES;
    expr_visitor_walk(body, &mut |_| {
        if budget == 0 {
            return VisitorControlFlow::Stop(());
        }
        budget -= 1;
        VisitorControlFlow::Recurse
    });
    if budget == 0 {
        return None;
    }
    let mut sources = Vec::new();
    expr_visitor_walk(body, &mut |e| {
        if let Some(s) = source(e) {
            if !sources.iter().any(|a: &Source| a.loop_expr.span.id == s.loop_expr.span.id) {
                sources.push(s);
            }
        }
        VisitorControlFlow::<()>::Recurse
    });
    if sources.len() != 1 {
        return None;
    }
    let source = sources.pop()?;
    let (iter, args) = call_name(&source.iterator)?;
    let iter = all.get(iter)?;
    // Match the resolved standard slice method, never spelling alone/custom iterators.
    if args.len() != 1
        || !iter
            .x
            .proxy
            .as_ref()
            .is_some_and(|p| models::spec_proxy(&p.x, &["std_specs", "slice"], "iter_mut"))
    {
        return None;
    }
    let (unsize, args) = call_name(&args[0])?;
    if !models::path_is(&unsize.path, CrateId::Vstd, &["array", "ref_mut_array_unsizing_coercion"])
        || args.len() != 1
    {
        return None;
    }
    let (ExprX::TwoPhaseBorrowMut(p) | ExprX::BorrowMut(p)) = &plain(&args[0]).x else {
        return None;
    };
    let PlaceX::DerefMut(p) = &p.x else { return None };
    if !matches!(&p.x, PlaceX::Local(v) if *v == f.x.params[1].x.name) {
        return None;
    }
    let mut writes = Vec::new();
    let mut breaks = 0;
    let mut forbidden = false;
    expr_visitor_walk(&source.body, &mut |e| {
        match &e.x {
            ExprX::Assign { .. } => writes.push(e.clone()),
            ExprX::BreakOrContinue { is_break: true, value: None, .. } => breaks += 1,
            ExprX::Call { .. }
            | ExprX::Loop { .. }
            | ExprX::Return(_)
            | ExprX::BreakOrContinue { .. } => forbidden = true,
            _ => (),
        }
        VisitorControlFlow::<()>::Recurse
    });
    if forbidden || breaks != 1 || writes.len() != 4 {
        return None;
    }
    let ExprX::Assign { place: first, rhs: payload, op: None, .. } = &writes[0].x else {
        return None;
    };
    let PlaceX::DerefMut(first) = &first.x else { return None };
    let PlaceX::Local(byte) = &first.x else { return None };
    let ExprX::Assign { place, rhs: step, op: Some(BinaryOp::Bitwise(BitwiseOp::Shr, _)), .. } =
        &writes[1].x
    else {
        return None;
    };
    let PlaceX::Local(n) = &place.x else { return None };
    if !integer(&place.typ, IntRange::U(64)) || !constant(step, 7) {
        return None;
    }
    let ExprX::Binary(BinaryOp::Bitwise(BitwiseOp::BitOr, _), cast, mask) = &plain(payload).x
    else {
        return None;
    };
    let ExprX::Unary(UnaryOp::Clip { range: IntRange::U(8), .. }, scalar) = &plain(cast).x else {
        return None;
    };
    if local(scalar) != Some(n) || !constant(mask, 128) {
        return None;
    }
    let ExprX::Assign { place, rhs, op: Some(BinaryOp::Bitwise(BitwiseOp::BitAnd, _)), .. } =
        &writes[2].x
    else {
        return None;
    };
    if !matches!(&place.x, PlaceX::DerefMut(p) if matches!(&p.x, PlaceX::Local(v) if v == byte))
        || !constant(rhs, 127)
    {
        return None;
    }
    let ExprX::Assign { place, rhs, op: Some(BinaryOp::Arith(ArithOp::Add(_))), .. } = &writes[3].x
    else {
        return None;
    };
    let PlaceX::Local(i) = &place.x else { return None };
    if !integer(&place.typ, IntRange::USize) || !constant(rhs, 1) || i == n {
        return None;
    }
    // The mask and break must be on the n==0 branch after shifting.
    let mut branch = false;
    expr_visitor_walk(&source.body, &mut |e| {
        if let ExprX::If(cond, yes, None) = &e.x {
            let cond = match &plain(cond).x {
                ExprX::If(c, _, _) => plain(c),
                _ => plain(cond),
            };
            if matches!(&cond.x, ExprX::Binary(BinaryOp::Eq(_), a, b) if local(a) == Some(n) && constant(b,0))
            {
                let mut mask = false;
                let mut exit = false;
                expr_visitor_walk(yes, &mut |e| {
                    mask |= e.span.id == writes[2].span.id;
                    exit |= matches!(&e.x, ExprX::BreakOrContinue { is_break: true, .. });
                    VisitorControlFlow::<()>::Recurse
                });
                branch |= mask && exit;
            }
        }
        VisitorControlFlow::<()>::Recurse
    });
    if !branch {
        return None;
    }
    let mut init_n = false;
    let mut init_i = false;
    expr_visitor_walk(body, &mut |e| {
        if let ExprX::Block(ss, _) = &e.x {
            for s in ss.iter() {
                if let StmtX::Decl { pattern, init: Some(p), .. } = &s.x {
                    if let PatternX::Var(v) = &pattern.x {
                        init_n |= v.name == *n
                            && matches!(&p.x, PlaceX::Local(v) if *v == f.x.params[0].x.name);
                        init_i |=
                            v.name == *i && matches!(&p.x, PlaceX::Temporary(e) if constant(e,0));
                    }
                }
            }
        }
        VisitorControlFlow::<()>::Recurse
    });
    if !init_n || !init_i {
        return None;
    }
    let ExprX::Loop { invs, .. } = &source.loop_expr.x else { return None };
    let mut radices = std::collections::BTreeSet::new();
    for inv in invs.iter() {
        expr_visitor_walk(&inv.inv, &mut |e| {
            if let Some((n, _)) = call_name(e) {
                if all.get(n).is_some_and(|f| {
                    ordinary(f)
                        && f.x.mode == Mode::Spec
                        && f.x.name.path.krate != CrateId::Vstd
                        && f.x.body.is_some()
                        && f.x.params.len() == 1
                        && integer(&f.x.params[0].x.typ, IntRange::Nat)
                        && integer(&f.x.inner_ret.x.typ, IntRange::Nat)
                }) {
                    radices.insert(n.clone());
                }
            }
            VisitorControlFlow::<()>::Recurse
        });
    }
    if radices.len() != 1 {
        return None;
    }
    let span = &source.body.span;
    let index = SpannedTyped::new(span, &place.typ, ExprX::Var(i.clone()));
    let number =
        SpannedTyped::new(span, &f.x.params[0].x.typ, ExprX::Var(f.x.params[0].x.name.clone()));
    let radix = all.get(radices.first()?)?.clone();
    Some(Shape { source, scalar: scalar.clone(), index, number, payload: payload.clone(), radix })
}

struct Builder<'a> {
    span: &'a Span,
    all: &'a HashMap<Fun, Function>,
}
impl Builder<'_> {
    fn expr(&self, t: &Typ, x: ExprX) -> Expr {
        SpannedTyped::new(self.span, t, x)
    }
    fn int(&self, n: usize) -> Expr {
        crate::ast_util::mk_int_lit_from_usize(self.span, n)
    }
    fn bin(&self, op: BinaryOp, a: Expr, b: Expr) -> Expr {
        let t = match op {
            BinaryOp::Eq(_) | BinaryOp::Inequality(_) => bool_typ(),
            _ => int_typ(),
        };
        self.expr(&t, ExprX::Binary(op, a, b))
    }
    fn eq(&self, a: Expr, b: Expr) -> Expr {
        self.bin(BinaryOp::Eq(Mode::Spec), a, b)
    }
    fn arith(&self, op: ArithOp, a: Expr, b: Expr) -> Expr {
        self.bin(BinaryOp::Arith(op), a, b)
    }
    fn call(&self, f: &Function, args: Vec<Expr>) -> Expr {
        self.expr(
            &f.x.inner_ret.x.typ,
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
                args: Arc::new(args),
                post_args: None,
                body: None,
            },
        )
    }
    fn law(&self, name: &str, args: Vec<Expr>) -> Option<Expr> {
        Some(self.call(&library(self.all, &["arithmetic", "div_mod", name], Mode::Proof)?, args))
    }
    fn unit(&self, x: ExprX) -> Expr {
        self.expr(&crate::ast_util::unit_typ(), x)
    }
    fn assertion(&self, expr: Expr) -> Expr {
        self.unit(ExprX::AssertAssume { is_assume: false, expr, msg: None })
    }
    fn block(&self, es: Vec<Expr>) -> Expr {
        self.unit(ExprX::Block(
            Arc::new(
                es.into_iter()
                    .map(|e| crate::def::Spanned::new(self.span.clone(), StmtX::Expr(e)))
                    .collect(),
            ),
            None,
        ))
    }
    fn ghost(&self, es: Vec<Expr>) -> Expr {
        self.unit(ExprX::Ghost { alloc_wrapper: false, tracked: false, expr: self.block(es) })
    }
}
fn annotations(s: &Shape, all: &HashMap<Fun, Function>) -> Option<(Expr, Expr)> {
    let b = Builder { span: &s.source.body.span, all };
    let r = |i| b.call(&s.radix, vec![i]);
    let i = s.index.clone();
    let next = b.arith(ArithOp::Add(OverflowBehavior::Allow), i.clone(), b.int(1));
    let current = r(i.clone());
    let after = r(next);
    let num = s.number.clone();
    let pow = library(all, &["arithmetic", "power", "pow"], Mode::Spec)?;
    let pow2 = library(all, &["arithmetic", "power2", "pow2"], Mode::Spec)?;
    let shr = library(all, &["bits", "lemma_u64_shr_is_div"], Mode::Proof)?;
    let seven = b.expr(
        &Arc::new(TypX::Int(IntRange::U(64))),
        ExprX::Const(crate::ast_util::const_int_from_usize(7)),
    );
    let mut es = vec![
        b.unit(ExprX::Fuel(s.radix.x.name.clone(), 11, false)),
        b.assertion(b.bin(BinaryOp::Inequality(InequalityOp::Gt), current.clone(), b.int(0))),
        b.call(&shr, vec![s.scalar.clone(), seven]),
        b.unit(ExprX::Fuel(pow.x.name.clone(), 1, false)),
        b.unit(ExprX::AssertCompute(
            b.eq(b.call(&pow2, vec![b.int(7)]), b.int(128)),
            ComputeMode::Z3,
        )),
        b.law("lemma_div_denominator", vec![num.clone(), current.clone(), b.int(128)])?,
        b.law("lemma_fundamental_div_mod", vec![num.clone(), current.clone()])?,
        b.law("lemma_fundamental_div_mod", vec![num.clone(), after.clone()])?,
        b.assertion(b.bin(BinaryOp::Inequality(InequalityOp::Gt), after.clone(), b.int(0))),
        b.law("lemma_small_div_converse", vec![num.clone(), after])?,
    ];
    es.push(b.unit(ExprX::If(
        b.bin(BinaryOp::Inequality(InequalityOp::Lt), num.clone(), current.clone()),
        b.block(vec![b.law("lemma_basic_div", vec![num.clone(), current])?]),
        None,
    )));
    let r10 = r(b.int(10));
    let huge = b.expr(
        &int_typ(),
        ExprX::Const(crate::ast_util::const_int_from_u128(1180591620717411303424)),
    );
    es.push(b.unit(ExprX::If(
        b.eq(i, b.int(9)),
        b.block(vec![
            b.unit(ExprX::AssertCompute(b.eq(r10.clone(), huge), ComputeMode::Z3)),
            b.law("lemma_basic_div", vec![num.clone(), r10])?,
        ]),
        None,
    )));
    let rem = b.arith(ArithOp::EuclideanMod(Div0Behavior::Allow), s.scalar.clone(), b.int(128));
    let mask = b.expr(
        &s.payload.typ,
        ExprX::Binary(
            BinaryOp::Bitwise(BitwiseOp::BitAnd, BitshiftBehavior::Allow),
            s.payload.clone(),
            b.expr(&s.payload.typ, ExprX::Const(crate::ast_util::const_int_from_usize(127))),
        ),
    );
    for fact in [
        b.eq(
            s.payload.clone(),
            b.arith(ArithOp::Add(OverflowBehavior::Allow), rem.clone(), b.int(128)),
        ),
        b.eq(mask, rem),
    ] {
        es.push(b.unit(ExprX::AssertQuery {
            requires: Arc::new(vec![]),
            ensures: Arc::new(vec![fact]),
            proof: b.block(vec![]),
            mode: AssertQueryMode::BitVector,
        }));
    }
    Some((b.ghost(vec![b.law("lemma_div_basics", vec![num])?]), b.ghost(es)))
}

fn invariants(s: &Shape, all: &HashMap<Fun, Function>) -> Option<Vec<LoopInvariant>> {
    let b = Builder { span: &s.source.loop_expr.span, all };
    let idx = all.values().find(|f| {
        f.x.mode == Mode::Spec
            && f.x.name.path.krate == CrateId::Vstd
            && f.x.name.path.segments.len() == 4
            && f.x.name.path.segments[0].as_str() == "std_specs"
            && f.x.name.path.segments[1].as_str() == "iter"
            && f.x.name.path.segments[2].starts_with("impl")
            && f.x.name.path.last_segment().as_str() == "index"
    })?;
    let TypX::Datatype(_, ts, _) = &*s.source.wrapper.typ else { return None };
    let wrapper = b.expr(&s.source.wrapper.typ, ExprX::Var(s.source.wrapper.name.clone()));
    let index = b.expr(
        &int_typ(),
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                idx.x.name.clone(),
                ts.clone(),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![wrapper]),
            post_args: None,
            body: None,
        },
    );
    // Reuse the supplied written-prefix observation, including its resolved type.
    let ExprX::Loop { invs, .. } = &s.source.loop_expr.x else { return None };
    let mut seq = None;
    for inv in invs.iter() {
        expr_visitor_walk(&inv.inv, &mut |e| {
            if let Some((n, args)) = call_name(e) {
                if n.path.krate == CrateId::Vstd
                    && n.path.segments.len() == 4
                    && n.path.segments[0].as_str() == "std_specs"
                    && n.path.segments[1].as_str() == "iter"
                    && n.path.segments[2].starts_with("impl")
                    && n.path.last_segment().as_str() == "seq"
                    && args.len() == 1
                    && local(&args[0]) == Some(&s.source.wrapper.name)
                {
                    seq = Some(e.clone());
                }
            }
            VisitorControlFlow::<()>::Recurse
        });
    }
    let seq = seq?;
    let len = all.values().find(|f| collections::model(f) == Some(collections::Model::SeqLen))?;
    let TypX::Datatype(_, ts, _) = &*seq.typ else { return None };
    let length = b.expr(
        &int_typ(),
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                len.x.name.clone(),
                ts.clone(),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![seq]),
            post_args: None,
            body: None,
        },
    );
    Some(vec![
        LoopInvariant {
            kind: LoopInvariantKind::InvariantExceptBreak,
            inv: b.eq(s.index.clone(), index),
        },
        LoopInvariant {
            kind: LoopInvariantKind::InvariantAndEnsures,
            inv: b.bin(BinaryOp::Inequality(InequalityOp::Le), s.index.clone(), b.int(9)),
        },
        LoopInvariant {
            kind: LoopInvariantKind::InvariantAndEnsures,
            inv: b.eq(length, b.int(10)),
        },
    ])
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = crate::infer::pipeline::function_map(fs, imported);
    fs.iter()
        .map(|f| {
            let Some(s) = shape(f, &all) else {
                return Ok(f.clone());
            };
            let Some((initial, step)) = annotations(&s, &all) else {
                return Ok(f.clone());
            };
            let Some(extra) = invariants(&s, &all) else {
                return Ok(f.clone());
            };
            let body = f.x.body.as_ref().unwrap();
            let mut id = u64::MAX - 8192;
            let initial = assistance::fresh(&initial, &mut id)?;
            let step = assistance::fresh(&step, &mut id)?;
            let changed = map_expr_visitor(body, &|e| {
                if e.span.id == s.source.loop_expr.span.id {
                    if let ExprX::Loop {
                        loop_isolation,
                        allow_complex_invariants,
                        is_for_loop,
                        assume_termination,
                        label,
                        cond,
                        body,
                        atomic_call,
                        invs,
                        decrease,
                    } = &e.x
                    {
                        let mut invs = (**invs).clone();
                        invs.extend(extra.iter().cloned());
                        return Ok(e.new_x(ExprX::Loop {
                            loop_isolation: *loop_isolation,
                            allow_complex_invariants: *allow_complex_invariants,
                            is_for_loop: *is_for_loop,
                            assume_termination: *assume_termination,
                            label: label.clone(),
                            cond: cond.clone(),
                            body: body.clone(),
                            atomic_call: *atomic_call,
                            invs: Arc::new(invs),
                            decrease: decrease.clone(),
                        }));
                    }
                }
                if e.span.id == s.source.body.span.id {
                    let ExprX::Block(ss, tail) = &e.x else {
                        return Ok(e.clone());
                    };
                    let mut ss = (**ss).clone();
                    ss.insert(
                        0,
                        crate::def::Spanned::new(e.span.clone(), StmtX::Expr(step.clone())),
                    );
                    return Ok(e.new_x(ExprX::Block(Arc::new(ss), tail.clone())));
                }
                Ok(e.clone())
            })?;
            let ExprX::Block(ss, tail) = &changed.x else {
                return Ok(f.clone());
            };
            let mut ss = (**ss).clone();
            ss.insert(0, crate::def::Spanned::new(body.span.clone(), StmtX::Expr(initial)));
            let mut x = f.x.clone();
            x.body = Some(changed.new_x(ExprX::Block(Arc::new(ss), tail.clone())));
            report.assistance_decisions.push(report::AssistanceDecision {
                function: f.x.name.clone(),
                source: s.source.loop_expr.span.clone(),
                profile: "mutable_buffer_u64_radix128",
                outcome: report::AssistanceOutcome::Applied,
            });
            Ok(f.new_x(x))
        })
        .collect()
}
