//! Bounded ordinary-proof assistance for a fixed mutable-byte radix loop.
//!
//! No update summary is created. Every arithmetic, byte and iterator annotation
//! proposed here is checked against the retained body by ordinary verification.
//! Initial scope is intentionally u64/7/[u8;10]; it is not a general effect engine.
use super::native_mutable_for::plain;
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor};

const NODES: usize = 8192;
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
fn catalog_signature(f: &Function, path: &[&str]) -> bool {
    let (types, ret): (&[IntRange], Option<IntRange>) = match path {
        ["arithmetic", "power", "pow"] => (&[IntRange::Int, IntRange::Nat], Some(IntRange::Int)),
        ["arithmetic", "power2", "pow2"] => (&[IntRange::Nat], Some(IntRange::Nat)),
        ["bits", "lemma_u64_shr_is_div"] => (&[IntRange::U(64), IntRange::U(64)], None),
        ["arithmetic", "div_mod", "lemma_div_basics"] => (&[IntRange::Int], None),
        ["arithmetic", "div_mod", "lemma_div_denominator"] => {
            (&[IntRange::Int, IntRange::Int, IntRange::Int], None)
        }
        [
            "arithmetic",
            "div_mod",
            "lemma_fundamental_div_mod" | "lemma_small_div_converse" | "lemma_basic_div",
        ] => (&[IntRange::Int, IntRange::Int], None),
        _ => return false,
    };
    matches!(f.x.kind, FunctionKind::Static)
        && matches!(f.x.item_kind, ItemKind::Function)
        && f.x.params.len() == types.len()
        && f.x.ensure.1.is_empty()
        && f.x
            .params
            .iter()
            .zip(types)
            .all(|(p, r)| p.x.mode == Mode::Spec && !p.x.user_mut && integer(&p.x.typ, *r))
        && match ret {
            Some(r) => f.x.mode == Mode::Spec && integer(&f.x.inner_ret.x.typ, r),
            None => {
                f.x.mode == Mode::Proof
                    && enums::unit(&f.x.inner_ret.x.typ)
                    && !f.x.ensure.0.is_empty()
            }
        }
}
fn radix_signature(f: &Function) -> bool {
    ordinary(f)
        && f.x.mode == Mode::Spec
        && f.x.name.path.krate != CrateId::Vstd
        && matches!(f.x.kind, FunctionKind::Static)
        && matches!(f.x.item_kind, ItemKind::Function)
        && f.x.typ_params.is_empty()
        && f.x.typ_bounds.is_empty()
        && f.x.proxy.is_none()
        && !f.x.attrs.prophecy_dependent
        && f.x.body.is_some()
        && f.x.require.is_empty()
        && f.x.ensure.0.is_empty()
        && f.x.ensure.1.is_empty()
        && f.x.params.len() == 1
        && f.x.params[0].x.mode == Mode::Spec
        && !f.x.params[0].x.user_mut
        && integer(&f.x.params[0].x.typ, IntRange::Nat)
        && integer(&f.x.inner_ret.x.typ, IntRange::Nat)
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
                && catalog_signature(f, path)
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
    let source = super::native_mutable_for::for_source(e)?;
    let loop_expr = &source.loop_expr;
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
        iterator: source.iterator,
        body: plain(body).clone(),
        wrapper: source.wrapper,
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
fn shape(
    f: &Function,
    all: &HashMap<Fun, Function>,
    reason: &mut report::AssistanceSkip,
) -> Option<Shape> {
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
    *reason = report::AssistanceSkip::SourceBudget;
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
    *reason = report::AssistanceSkip::ExpectedSingleNativeFor;
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
    *reason = report::AssistanceSkip::IteratorProofShape;
    let (iter, args) = call_name(&source.iterator)?;
    let iter = all.get(iter)?;
    // Match the resolved standard slice method, never spelling alone/custom iterators.
    if args.len() != 1
        || !iter
            .x
            .proxy
            .as_ref()
            .is_some_and(|p| models::spec_proxy(&p.x, &["std_specs", "slice"], "iter__mut"))
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
    *reason = report::AssistanceSkip::UnsupportedIndex;
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
    *reason = report::AssistanceSkip::MissingFinalStateGoal;
    let mut radices = std::collections::BTreeSet::new();
    for inv in invs.iter() {
        expr_visitor_walk(&inv.inv, &mut |e| {
            if let Some((n, _)) = call_name(e) {
                if all.get(n).is_some_and(radix_signature) {
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
    let idx = all.values().find(|f| super::native_mutable_for::wrapper_member(f, "index"))?;
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
    // Native sequence length comes from the shared observation pass. Keep
    // counter alignment and its scalar bound local to the numeric profile.
    Some(vec![
        LoopInvariant {
            kind: LoopInvariantKind::InvariantExceptBreak,
            inv: b.eq(s.index.clone(), index),
        },
        LoopInvariant {
            kind: LoopInvariantKind::InvariantAndEnsures,
            inv: b.bin(BinaryOp::Inequality(InequalityOp::Le), s.index.clone(), b.int(9)),
        },
    ])
}

// Independently erase exactly this pass's recorded annotations. This is separate
// from the installer and compares the entire original function, not its effects.
fn validate_decoration(
    original: &Function,
    candidate: &Function,
    source_loop: &Expr,
    source_body: &Expr,
    initial: &Expr,
    step: &Expr,
    extra: &[LoopInvariant],
) -> Result<(), VirErr> {
    let bad =
        || unsupported(&original.span, "mutable-buffer decoration failed exact source projection");
    let same = |a: &Expr, b: &Expr| match (serde_json::to_vec(a), serde_json::to_vec(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    let original_body = original.x.body.as_ref().ok_or_else(bad)?;
    let candidate_body = candidate.x.body.as_ref().ok_or_else(bad)?;
    let mut metadata = candidate.x.clone();
    metadata.body = original.x.body.clone();
    if serde_json::to_vec(&metadata).map_err(|_| bad())?
        != serde_json::to_vec(&original.x).map_err(|_| bad())?
    {
        return Err(bad());
    }
    let mut initial_count = 0;
    let mut step_count = 0;
    expr_visitor_walk(candidate_body, &mut |e| {
        initial_count += usize::from(e.span.id == initial.span.id);
        step_count += usize::from(e.span.id == step.span.id);
        VisitorControlFlow::<()>::Recurse
    });
    if initial_count != 1 || step_count != 1 {
        return Err(bad());
    }
    let ExprX::Block(ss, tail) = &candidate_body.x else { return Err(bad()) };
    if !matches!(ss.first().map(|s|&s.x),Some(StmtX::Expr(e)) if same(e,initial)) {
        return Err(bad());
    }
    let unprefixed = candidate_body.new_x(ExprX::Block(Arc::new(ss[1..].to_vec()), tail.clone()));
    let ExprX::Loop { invs: original_invs, .. } = &source_loop.x else { return Err(bad()) };
    let loops = std::cell::Cell::new(0);
    let bodies = std::cell::Cell::new(0);
    let projected = map_expr_visitor(&unprefixed, &|e| {
        if e.span.id == source_body.span.id && matches!(&e.x, ExprX::Block(..)) {
            let ExprX::Block(ss, tail) = &e.x else { return Err(bad()) };
            if !matches!(ss.first().map(|s|&s.x),Some(StmtX::Expr(e)) if same(e,step)) {
                return Err(bad());
            }
            let restored = e.new_x(ExprX::Block(Arc::new(ss[1..].to_vec()), tail.clone()));
            if !same(&restored, source_body) {
                return Err(bad());
            }
            bodies.set(bodies.get() + 1);
            return Ok(restored);
        }
        if e.span.id == source_loop.span.id {
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
                if invs.len() != original_invs.len() + extra.len()
                    || serde_json::to_vec(&invs[..original_invs.len()]).map_err(|_| bad())?
                        != serde_json::to_vec(original_invs).map_err(|_| bad())?
                    || serde_json::to_vec(&invs[original_invs.len()..]).map_err(|_| bad())?
                        != serde_json::to_vec(extra).map_err(|_| bad())?
                {
                    return Err(bad());
                }
                loops.set(loops.get() + 1);
                return Ok(e.new_x(ExprX::Loop {
                    loop_isolation: *loop_isolation,
                    allow_complex_invariants: *allow_complex_invariants,
                    is_for_loop: *is_for_loop,
                    assume_termination: *assume_termination,
                    label: label.clone(),
                    cond: cond.clone(),
                    body: body.clone(),
                    atomic_call: *atomic_call,
                    invs: Arc::new(invs[..original_invs.len()].to_vec()),
                    decrease: decrease.clone(),
                }));
            }
        }
        Ok(e.clone())
    })?;
    if loops.get() != 1 || bodies.get() != 1 || !same(&projected, original_body) {
        return Err(bad());
    }
    Ok(())
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = crate::infer::pipeline::function_map(fs, imported);
    fs.iter()
        .map(|f| {
            let decision = |outcome| report::AssistanceDecision {
                function: f.x.name.clone(),
                source: f.span.clone(),
                profile: "mutable_buffer_u64_radix128",
                outcome,
            };
            let mut reason = report::AssistanceSkip::TransportSignature;
            let Some(s) = shape(f, &all, &mut reason) else {
                if reason != report::AssistanceSkip::TransportSignature {
                    report
                        .assistance_decisions
                        .push(decision(report::AssistanceOutcome::Skipped(reason)));
                }
                return Ok(f.clone());
            };
            let Some((initial, step)) = annotations(&s, &all) else {
                report.assistance_decisions.push(decision(report::AssistanceOutcome::Error(
                    "Missing checked arithmetic catalog for mutable-buffer proposal".into(),
                )));
                return Ok(f.clone());
            };
            let Some(extra) = invariants(&s, &all) else {
                report.assistance_decisions.push(decision(report::AssistanceOutcome::Skipped(
                    report::AssistanceSkip::MissingFinalStateGoal,
                )));
                return Ok(f.clone());
            };
            let body = f.x.body.as_ref().unwrap();
            let mut id = u64::MAX - 8192;
            let initial = assistance::fresh(&initial, &mut id)?;
            let step = assistance::fresh(&assistance::current(&step)?, &mut id)?;
            let extra = extra
                .iter()
                .map(|i| {
                    Ok(LoopInvariant { kind: i.kind, inv: assistance::fresh(&i.inv, &mut id)? })
                })
                .collect::<Result<Vec<_>, VirErr>>()?;
            // Fresh annotations may not alias an original or another generated AST id.
            let mut used = std::collections::HashSet::new();
            expr_visitor_walk(body, &mut |e| {
                used.insert(e.span.id);
                VisitorControlFlow::<()>::Recurse
            });
            let mut collision = false;
            for e in std::iter::once(&initial)
                .chain(std::iter::once(&step))
                .chain(extra.iter().map(|i| &i.inv))
            {
                expr_visitor_walk(e, &mut |e| {
                    collision |= !used.insert(e.span.id);
                    VisitorControlFlow::<()>::Recurse
                });
            }
            if collision {
                return Err(unsupported(
                    &f.span,
                    "mutable-buffer generated AST identity collision",
                ));
            }
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
            ss.insert(0, crate::def::Spanned::new(body.span.clone(), StmtX::Expr(initial.clone())));
            let mut x = f.x.clone();
            x.body = Some(changed.new_x(ExprX::Block(Arc::new(ss), tail.clone())));
            let candidate = f.new_x(x);
            validate_decoration(
                f,
                &candidate,
                &s.source.loop_expr,
                &s.source.body,
                &initial,
                &step,
                &extra,
            )?;
            report.body_rechecks.insert(f.x.name.clone());
            report.assistance_decisions.push(report::AssistanceDecision {
                function: f.x.name.clone(),
                source: s.source.loop_expr.span.clone(),
                profile: "mutable_buffer_u64_radix128",
                outcome: report::AssistanceOutcome::Applied,
            });
            Ok(candidate)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input, span};
    fn expr(id: u64, t: Typ, x: ExprX) -> Expr {
        let mut s = span();
        s.id = id;
        SpannedTyped::new(&s, &t, x)
    }
    fn unit(id: u64, x: ExprX) -> Expr {
        expr(id, crate::ast_util::unit_typ(), x)
    }
    fn flag(id: u64, value: bool) -> Expr {
        expr(id, bool_typ(), ExprX::Const(Constant::Bool(value)))
    }
    fn num(id: u64, value: usize) -> Expr {
        expr(id, int_typ(), ExprX::Const(crate::ast_util::const_int_from_usize(value)))
    }
    fn stmt(e: Expr) -> Stmt {
        crate::def::Spanned::new(e.span.clone(), StmtX::Expr(e))
    }
    fn block(id: u64, es: Vec<Expr>) -> Expr {
        unit(id, ExprX::Block(Arc::new(es.into_iter().map(stmt).collect()), None))
    }
    fn ghost(id: u64) -> Expr {
        unit(id, ExprX::Ghost { alloc_wrapper: false, tracked: false, expr: block(id + 1, vec![]) })
    }
    fn replace_body(f: &Function, body: Expr) -> Function {
        let mut x = f.x.clone();
        x.body = Some(body);
        f.new_x(x)
    }
    struct Fixture {
        original: Function,
        candidate: Function,
        lp: Expr,
        body: Expr,
        initial: Expr,
        step: Expr,
        extra: Vec<LoopInvariant>,
    }
    impl Fixture {
        fn validate(&self, c: &Function) -> Result<(), VirErr> {
            validate_decoration(
                &self.original,
                c,
                &self.lp,
                &self.body,
                &self.initial,
                &self.step,
                &self.extra,
            )
        }
    }
    fn fixture() -> Fixture {
        let f = function("buffer", input("x"));
        let body = block(
            30,
            vec![unit(
                31,
                ExprX::AssertAssume { is_assume: false, expr: flag(32, true), msg: None },
            )],
        );
        let lp = unit(
            20,
            ExprX::Loop {
                loop_isolation: true,
                allow_complex_invariants: true,
                is_for_loop: true,
                assume_termination: false,
                label: Label { id: 0, name: None },
                cond: None,
                body: body.clone(),
                atomic_call: false,
                invs: Arc::new(vec![LoopInvariant {
                    kind: LoopInvariantKind::InvariantExceptBreak,
                    inv: flag(21, true),
                }]),
                decrease: Arc::new(vec![num(22, 1)]),
            },
        );
        let original = replace_body(&f, block(10, vec![lp.clone()]));
        let initial = ghost(50);
        let step = ghost(60);
        let extra = vec![LoopInvariant {
            kind: LoopInvariantKind::InvariantAndEnsures,
            inv: flag(70, true),
        }];
        let candidate_body = map_expr_visitor(original.x.body.as_ref().unwrap(), &|e| {
            if e.span.id == 30 {
                return Ok(block(
                    30,
                    vec![
                        step.clone(),
                        unit(
                            31,
                            ExprX::AssertAssume {
                                is_assume: false,
                                expr: flag(32, true),
                                msg: None,
                            },
                        ),
                    ],
                ));
            }
            if let ExprX::Loop { body, .. } = &e.x {
                return Ok(unit(
                    20,
                    ExprX::Loop {
                        loop_isolation: true,
                        allow_complex_invariants: true,
                        is_for_loop: true,
                        assume_termination: false,
                        label: Label { id: 0, name: None },
                        cond: None,
                        body: body.clone(),
                        atomic_call: false,
                        invs: Arc::new(vec![
                            LoopInvariant {
                                kind: LoopInvariantKind::InvariantExceptBreak,
                                inv: flag(21, true),
                            },
                            extra[0].clone(),
                        ]),
                        decrease: Arc::new(vec![num(22, 1)]),
                    },
                ));
            }
            if e.span.id == 10 {
                let ExprX::Block(ss, None) = &e.x else { panic!() };
                let mut ss = (**ss).clone();
                ss.insert(0, stmt(initial.clone()));
                return Ok(unit(10, ExprX::Block(Arc::new(ss), None)));
            }
            Ok(e.clone())
        })
        .unwrap();
        Fixture {
            original,
            candidate: replace_body(&f, candidate_body),
            lp,
            body,
            initial,
            step,
            extra,
        }
    }
    #[test]
    fn independent_projection_accepts_only_exact_annotation_round_trip() {
        let f = fixture();
        assert!(f.validate(&f.candidate).is_ok());
        // A retained executable assertion is changed to a state assignment.
        let mutated = map_expr_visitor(f.candidate.x.body.as_ref().unwrap(), &|e| {
            if e.span.id == 31 {
                let p = SpannedTyped::new(
                    &span(),
                    &input("x").typ,
                    PlaceX::Local(VarIdent(Arc::new("x".into()), VarIdentDisambiguate::AirLocal)),
                );
                return Ok(unit(
                    31,
                    ExprX::Assign {
                        place: p,
                        rhs: input("x"),
                        op: None,
                        typ: input("x").typ.clone(),
                        resolve: false,
                    },
                ));
            }
            Ok(e.clone())
        })
        .unwrap();
        assert!(f.validate(&replace_body(&f.candidate, mutated)).is_err());
        for changed in [21, 22] {
            let mutated = map_expr_visitor(f.candidate.x.body.as_ref().unwrap(), &|e| {
                Ok(if e.span.id == changed {
                    if changed == 21 { flag(21, false) } else { num(22, 2) }
                } else {
                    e.clone()
                })
            })
            .unwrap();
            assert!(f.validate(&replace_body(&f.candidate, mutated)).is_err());
        }
        // Whole-function contracts are part of the independent comparison.
        let mut x = f.candidate.x.clone();
        x.require = Arc::new(vec![flag(80, false)]);
        assert!(f.validate(&f.candidate.new_x(x)).is_err());
    }
    #[test]
    fn independent_projection_rejects_duplicate_and_misplaced_insertions() {
        let f = fixture();
        let ExprX::Block(ss, tail) = &f.candidate.x.body.as_ref().unwrap().x else { panic!() };
        let mut duplicate = (**ss).clone();
        duplicate.insert(0, stmt(f.initial.clone()));
        assert!(
            f.validate(&replace_body(
                &f.candidate,
                unit(10, ExprX::Block(Arc::new(duplicate), tail.clone()))
            ))
            .is_err()
        );
        let moved = map_expr_visitor(f.candidate.x.body.as_ref().unwrap(), &|e| {
            if e.span.id == 30 {
                let ExprX::Block(ss, tail) = &e.x else { panic!() };
                return Ok(unit(30, ExprX::Block(Arc::new(ss[1..].to_vec()), tail.clone())));
            }
            if e.span.id == 10 {
                let ExprX::Block(ss, tail) = &e.x else { panic!() };
                let mut ss = (**ss).clone();
                ss.insert(1, stmt(f.step.clone()));
                return Ok(unit(10, ExprX::Block(Arc::new(ss), tail.clone())));
            }
            Ok(e.clone())
        })
        .unwrap();
        assert!(f.validate(&replace_body(&f.candidate, moved)).is_err());
    }
    #[test]
    fn arithmetic_and_radix_signatures_fail_closed() {
        let f = function("radix", input("x"));
        let mut x = f.x.clone();
        x.attrs = Arc::new(FunctionAttrsX::default());
        x.mode = Mode::Spec;
        let mut p = x.params[0].x.clone();
        p.typ = Arc::new(TypX::Int(IntRange::Nat));
        p.mode = Mode::Spec;
        p.user_mut = false;
        x.params = Arc::new(vec![crate::def::Spanned::new(span(), p)]);
        let mut r = x.inner_ret.x.clone();
        r.typ = Arc::new(TypX::Int(IntRange::Nat));
        x.inner_ret = crate::def::Spanned::new(span(), r);
        x.typ_params = Arc::new(vec![]);
        x.typ_bounds = Arc::new(vec![]);
        x.require = Arc::new(vec![]);
        x.ensure = (Arc::new(vec![]), Arc::new(vec![]));
        let f = f.new_x(x);
        assert!(radix_signature(&f));
        for i in 0..4 {
            let mut x = f.x.clone();
            match i {
                0 => x.typ_params = Arc::new(vec![Arc::new("T".into())]),
                1 => Arc::make_mut(&mut x.attrs).is_external_body = true,
                2 => x.params = Arc::new(vec![]),
                _ => Arc::make_mut(&mut x.attrs).prophecy_dependent = true,
            };
            assert!(!radix_signature(&f.new_x(x)));
        }
        assert!(!catalog_signature(&f, &["bits", "lemma_u64_shr_is_div"]));
        assert!(catalog_signature(&f, &["arithmetic", "power2", "pow2"]));
        let all = HashMap::from([(f.x.name.clone(), f)]);
        assert!(library(&all, &["arithmetic", "power2", "pow2"], Mode::Spec).is_none());
    }
}
