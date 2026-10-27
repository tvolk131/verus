//! Bounded Vec view and single-slot effect candidates using existing vstd models.
//!
//! No executable projection, new axiom, alias guess, or generated proof call.
//! Mutating summaries initially require sound mode: return-only precision checks
//! must not be confused with precision of the final collection state.
use super::*;
use crate::ast_visitor::{VisitorScopeMap, map_expr_visitor_env};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Model {
    Len,
    Index,
    IndexMut,
    SpecLen,
    LenAxiom,
    View,
    SeqLen,
    SeqIndex,
    SeqUpdate,
}

fn path(t: &Typ, krate: CrateId, parts: &[&str]) -> bool {
    matches!(&**t, TypX::Datatype(Dt::Path(p), _, _) if models::path_is(p, krate, parts))
}
fn seq(t: Typ) -> Typ {
    Arc::new(TypX::Datatype(
        Dt::Path(Arc::new(PathX {
            krate: CrateId::Vstd,
            segments: Arc::new(["seq", "Seq"].map(|s| Arc::new(s.into())).to_vec()),
        })),
        Arc::new(vec![t]),
        Arc::new(vec![]),
    ))
}
fn vec_args(t: &Typ) -> Option<&Typs> {
    match &**t {
        TypX::Datatype(Dt::Path(p), args, _)
            if models::path_is(p, CrateId::Alloc, &["vec", "Vec"]) && args.len() == 2 =>
        {
            Some(args)
        }
        _ => None,
    }
}
fn reference(t: &Typ, mutable: bool) -> Option<&Typ> {
    match (&**t, mutable) {
        (TypX::MutRef(t), true) | (TypX::Decorate(TypDecoration::Ref, None, t), false) => Some(t),
        _ => None,
    }
}
fn usize_type(t: &Typ) -> bool {
    matches!(&**t, TypX::Int(IntRange::USize))
}
fn same(a: &Typ, b: &Typ) -> bool {
    crate::ast_util::types_equal(a, b)
}

fn bounds(f: &FunctionX, ts: &[Typ], allocator: bool, copy: bool) -> bool {
    let mut sized = vec![0; ts.len()];
    let (mut allocs, mut copies) = (0, 0);
    for bound in f.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args) if args.len() == 1 => {
                let Some(i) = ts.iter().position(|t| same(t, &args[0])) else { return false };
                sized[i] += 1;
            }
            GenericBoundX::Trait(TraitId::Path(p), args)
                if allocator
                    && ts.len() == 2
                    && models::path_is(p, CrateId::Core, &["alloc", "Allocator"])
                    && args.len() == 1
                    && same(&args[0], &ts[1]) =>
            {
                allocs += 1
            }
            GenericBoundX::Trait(TraitId::Path(p), args)
                if copy
                    && !ts.is_empty()
                    && models::path_is(p, CrateId::Core, &["marker", "Copy"])
                    && args.len() == 1
                    && same(&args[0], &ts[0]) =>
            {
                copies += 1
            }
            _ => return false,
        }
    }
    sized.iter().all(|n| *n == 1)
        && allocs == usize::from(allocator)
        && copies == usize::from(copy && !ts.is_empty())
}

pub(super) fn model(function: &Function) -> Option<Model> {
    let f = &function.x;
    if f.attrs.infer
        || f.attrs.is_unsafe
        || f.attrs.is_async
        || f.body.is_some()
        || f.params.iter().any(|p| p.x.user_mut)
        || !f.ensure.1.is_empty()
    {
        return None;
    }
    let p = &f.name.path;
    let k = if p.krate == CrateId::Alloc
        && p.segments.len() == 3
        && p.segments[0].as_str() == "vec"
        && p.segments[1].starts_with("impl")
        && p.segments[2].as_str() == "len"
        && f.proxy.as_ref().is_some_and(|p| models::spec_proxy(&p.x, &["std_specs", "vec"], "_len"))
    {
        Model::Len
    } else if models::path_is(p, CrateId::Vstd, &["std_specs", "vec", "vec_index"]) {
        Model::Index
    } else if models::path_is(p, CrateId::Vstd, &["std_specs", "vec", "vec_index_mut"]) {
        Model::IndexMut
    } else if models::path_is(p, CrateId::Vstd, &["std_specs", "vec", "spec_vec_len"]) {
        Model::SpecLen
    } else if models::path_is(p, CrateId::Vstd, &["std_specs", "vec", "axiom_spec_len"]) {
        Model::LenAxiom
    } else if p.krate == CrateId::Vstd
        && p.segments.len() == 3
        && p.segments[0].as_str() == "view"
        && p.segments[1].starts_with("impl")
        && p.segments[2].as_str() == "view"
    {
        Model::View
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "len"]) {
        Model::SeqLen
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "index"]) {
        Model::SeqIndex
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "update"]) {
        Model::SeqUpdate
    } else {
        return None;
    };
    let seq_model = matches!(k, Model::SeqLen | Model::SeqIndex | Model::SeqUpdate);
    let ts: Vec<_> = f.typ_params.iter().map(|t| Arc::new(TypX::TypParam(t.clone()))).collect();
    if ts.len() != if seq_model { 1 } else { 2 } || !bounds(f, &ts, !seq_model, false) {
        return None;
    }
    let count = match k {
        Model::Index | Model::IndexMut | Model::SeqIndex => 2,
        Model::SeqUpdate => 3,
        _ => 1,
    };
    if f.params.len() != count || !same(&f.inner_ret.x.typ, &f.outer_ret.x.typ) {
        return None;
    }
    if seq_model {
        if f.mode != Mode::Spec
            || !matches!(f.kind, FunctionKind::Static)
            || f.proxy.is_some()
            || f.params.iter().any(|p| p.x.mode != Mode::Spec)
            || !same(&f.params[0].x.typ, &seq(ts[0].clone()))
            || (count > 1 && !matches!(&*f.params[1].x.typ, TypX::Int(IntRange::Int)))
            || (count == 3 && !same(&f.params[2].x.typ, &ts[0]))
        {
            return None;
        }
        let ret = match k {
            Model::SeqLen => Arc::new(TypX::Int(IntRange::Nat)),
            Model::SeqIndex => ts[0].clone(),
            _ => seq(ts[0].clone()),
        };
        return same(&f.inner_ret.x.typ, &ret).then_some(k);
    }
    let vector = reference(&f.params[0].x.typ, k == Model::IndexMut)?;
    let args = vec_args(vector)?;
    if !args.iter().zip(&ts).all(|(a, b)| same(a, b)) {
        return None;
    }
    let exec = matches!(k, Model::Len | Model::Index | Model::IndexMut);
    if f.mode
        != if exec {
            Mode::Exec
        } else if k == Model::LenAxiom {
            Mode::Proof
        } else {
            Mode::Spec
        }
        || f.params.iter().any(|p| p.x.mode != if exec { Mode::Exec } else { Mode::Spec })
    {
        return None;
    }
    if exec
        && (!matches!(f.kind, FunctionKind::Static)
            || !matches!(f.unwind_spec, Some(UnwindSpec::NoUnwind)))
    {
        return None;
    }
    let ret = match k {
        Model::Len | Model::SpecLen => Arc::new(TypX::Int(IntRange::USize)),
        Model::Index => models::reference(ts[0].clone()),
        Model::IndexMut => Arc::new(TypX::MutRef(ts[0].clone())),
        Model::View => seq(ts[0].clone()),
        Model::LenAxiom => crate::ast_util::unit_typ(),
        _ => unreachable!(),
    };
    if !same(&ret, &f.inner_ret.x.typ) || (count > 1 && !usize_type(&f.params[1].x.typ)) {
        return None;
    }
    match k {
        Model::Len if !f.require.is_empty() || f.ensure.0.len() != 1 => None,
        Model::Index
            if f.proxy.is_some()
                || f.require.len() != 1
                || f.ensure.0.len() != 1
                || !index_domain(f) =>
        {
            None
        }
        Model::IndexMut
            if f.proxy.is_some()
                || f.require.len() != 1
                || f.ensure.0.len() != 3
                || !index_domain(f) =>
        {
            None
        }
        Model::SpecLen | Model::View if !f.require.is_empty() || !f.ensure.0.is_empty() => None,
        Model::LenAxiom
            if !f.require.is_empty() || f.ensure.0.len() != 1 || !f.attrs.broadcast_forall =>
        {
            None
        }
        _ => Some(k),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Len,
    Read,
    Write,
}
pub(super) fn kind(function: &Function) -> Option<Kind> {
    let f = &function.x;
    let first = f.params.first()?;
    let mutable = matches!(&*first.x.typ, TypX::MutRef(_));
    let args = vec_args(reference(&first.x.typ, mutable)?)?;
    if !path(&args[1], CrateId::Alloc, &["alloc", "Global"]) {
        return None;
    }
    let k = if !mutable && f.params.len() == 1 && usize_type(&f.inner_ret.x.typ) {
        Kind::Len
    } else if !mutable && f.params.len() == 2 && same(&f.inner_ret.x.typ, &args[0]) {
        Kind::Read
    } else if mutable
        && f.params.len() == 3
        && same(&f.params[2].x.typ, &args[0])
        && enums::unit(&f.inner_ret.x.typ)
    {
        Kind::Write
    } else {
        return None;
    };
    if k != Kind::Len && !usize_type(&f.params[1].x.typ) {
        return None;
    }
    let ts: Vec<_> = f.typ_params.iter().map(|t| Arc::new(TypX::TypParam(t.clone()))).collect();
    if ts.is_empty() {
        if !scalar(&args[0]) || !f.typ_bounds.is_empty() {
            return None;
        }
    } else if ts.len() != 1
        || !same(&args[0], &ts[0])
        || !(bounds(f, &ts, false, k != Kind::Len) || k == Kind::Len && bounds(f, &ts, false, true))
    {
        return None;
    }
    Some(k)
}

pub(super) fn validate_signature(function: &Function) -> Result<Kind, VirErr> {
    let f = &function.x;
    let k = kind(function)
        .ok_or_else(|| unsupported(&function.span, "collection template signature"))?;
    if f.mode != Mode::Exec
        || !matches!(f.kind, FunctionKind::Static)
        || !matches!(f.item_kind, ItemKind::Function)
        || f.attrs.is_async
        || f.attrs.is_unsafe
        || f.attrs.is_external_body
        || f.proxy.is_some()
        || f.attrs.autospec.is_some()
        || f.attrs.bit_vector
        || f.attrs.integer_ring
        || f.attrs.broadcast_forall
        || f.params.iter().any(|p| p.x.mode != Mode::Exec || p.x.user_mut)
        || f.inner_ret.x.mode != Mode::Exec
        || !same(&f.inner_ret.x.typ, &f.outer_ret.x.typ)
        || !f.require.is_empty()
        || !f.ensure.0.is_empty()
        || !f.ensure.1.is_empty()
        || f.returns.is_some()
        || !f.decrease.is_empty()
        || f.decrease_when.is_some()
        || f.decrease_by.is_some()
    {
        return Err(unsupported(
            &function.span,
            "collection templates require safe executable bodies without handwritten contracts or decreases",
        ));
    }
    if (k == Kind::Write) != f.attrs.infer_allow_approx {
        return Err(unsupported(
            &function.span,
            if k == Kind::Write {
                "mutable collection summaries require precision = \"sound\"; state maximality/exactness are not yet checked"
            } else {
                "read-only collection templates currently require exact mode"
            },
        ));
    }
    Ok(k)
}

fn local(e: &Expr, name: &VarIdent) -> bool {
    fn place(p: &Place, name: &VarIdent) -> bool {
        match &p.x {
            PlaceX::Local(n) => n == name,
            PlaceX::DerefMut(p) => place(p, name),
            _ => false,
        }
    }
    matches!(&e.x, ExprX::Var(n) if n == name)
        || matches!(&e.x, ExprX::ReadPlace(p, _) | ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) if place(p, name))
}

/// Maximality may assume only the exact ordinary Vec-index bounds condition.
/// A strengthened or differently shaped library requirement is not admitted.
fn index_domain(f: &FunctionX) -> bool {
    let Some(req) = f.require.first() else { return false };
    let ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), index, length) = &req.x else {
        return false;
    };
    if !local(index, &f.params[1].x.name) {
        return false;
    }
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, name, ..),
        args,
        post_args: None,
        body: None,
    } = &length.x
    else {
        return false;
    };
    if !models::path_is(&name.path, CrateId::Vstd, &["seq", "Seq", "len"]) || args.len() != 1 {
        return false;
    }
    let ExprX::Call { target: CallTarget::Fun(kind, name, ..), args, post_args: None, body: None } =
        &args[0].x
    else {
        return false;
    };
    let view = match kind {
        CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. } => {
            models::path_is(&name.path, CrateId::Vstd, &["view", "View", "view"])
                && resolved.path.krate == CrateId::Vstd
                && resolved.path.segments.len() == 3
                && resolved.path.segments[0].as_str() == "view"
                && resolved.path.segments[1].starts_with("impl")
                && resolved.path.segments[2].as_str() == "view"
        }
        _ => false,
    };
    view && args.len() == 1 && local(&args[0], &f.params[0].x.name)
}
fn single(mut e: &Expr) -> &Expr {
    loop {
        match &e.x {
            ExprX::Block(ss, Some(tail)) if ss.is_empty() => e = tail,
            ExprX::Block(ss, None) if ss.len() == 1 => {
                if let StmtX::Expr(inner) = &ss[0].x {
                    e = inner;
                } else {
                    return e;
                }
            }
            _ => return e,
        }
    }
}
fn call<'a>(e: &'a Expr, which: Model, functions: &HashMap<Fun, Function>) -> Option<&'a Exprs> {
    match &single(e).x {
        ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, name, ..),
            args,
            post_args: None,
            body: None,
        } if functions.get(name).and_then(model) == Some(which) => Some(args),
        _ => None,
    }
}
fn assignment(e: &Expr, f: &Function, functions: &HashMap<Fun, Function>) -> bool {
    let ExprX::Assign { place, rhs, op: None, resolve: false, .. } = &single(e).x else {
        return false;
    };
    let PlaceX::DerefMut(p) = &place.x else { return false };
    let PlaceX::Temporary(e) = &p.x else { return false };
    let Some(args) = call(e, Model::IndexMut, functions) else { return false };
    args.len() == 2
        && local(&args[0], &f.x.params[0].x.name)
        && local(&args[1], &f.x.params[1].x.name)
        && local(rhs, &f.x.params[2].x.name)
}

fn length_guard(
    e: &Expr,
    f: &Function,
    functions: &HashMap<Fun, Function>,
    op: InequalityOp,
) -> bool {
    matches!(&e.x, ExprX::Binary(BinaryOp::Inequality(actual), i, len)
        if *actual == op && local(i, &f.x.params[1].x.name)
            && call(len, Model::Len, functions).is_some_and(|args| args.len() == 1 && local(&args[0], &f.x.params[0].x.name)))
}

fn empty(e: &Expr) -> bool {
    matches!(&e.x, ExprX::Block(ss, None) if ss.is_empty())
        || matches!(&e.x, ExprX::Ctor(_, _, fs, None) if enums::unit(&e.typ) && fs.is_empty())
}

fn unit_return(e: &Expr) -> bool {
    match &single(e).x {
        ExprX::Return(None) => true,
        ExprX::Return(Some(e)) => empty(single(e)),
        ExprX::NeverToAny(e) => unit_return(e),
        ExprX::ReadPlace(p, _) => matches!(&p.x, PlaceX::Temporary(e) if unit_return(e)),
        _ => false,
    }
}

fn guarded_assignment(e: &Expr, f: &Function, functions: &HashMap<Fun, Function>) -> bool {
    if let ExprX::If(c, yes, no) = &single(e).x {
        return length_guard(c, f, functions, InequalityOp::Lt)
            && assignment(yes, f, functions)
            && no.as_ref().is_none_or(|e| empty(single(e)));
    }
    // Canonical early-exit guard followed by exactly one assignment.
    if let ExprX::Block(ss, None) = &e.x {
        if ss.len() == 2 {
            if let (StmtX::Expr(first), StmtX::Expr(second)) = (&ss[0].x, &ss[1].x) {
                if let ExprX::If(c, yes, no) = &first.x {
                    return length_guard(c, f, functions, InequalityOp::Ge)
                        && unit_return(yes)
                        && no.as_ref().is_none_or(|e| empty(single(e)))
                        && assignment(second, f, functions);
                }
            }
        }
    }
    false
}

/// The only admitted generic executable dependencies at a collection template.
pub(super) fn source_call(e: &Expr, functions: &HashMap<Fun, Function>) -> bool {
    matches!(&e.x, ExprX::Call { target: CallTarget::Fun(CallTargetKind::Static, name, ..), post_args: None, body: None, .. }
        if matches!(functions.get(name).and_then(model), Some(Model::Len | Model::Index | Model::IndexMut)))
}

fn substitute(e: &Expr, m: &Function, f: &Function, return_value: bool) -> Result<Expr, VirErr> {
    let vector = reference(&f.x.params[0].x.typ, kind(f) == Some(Kind::Write)).unwrap();
    let types: HashMap<_, _> =
        m.x.typ_params.iter().cloned().zip(vec_args(vector).unwrap().iter().cloned()).collect();
    let mut names: HashMap<_, _> =
        m.x.params
            .iter()
            .zip(f.x.params.iter())
            .map(|(a, b)| (a.x.name.clone(), b.x.name.clone()))
            .collect();
    if return_value {
        names.insert(m.x.inner_ret.x.name.clone(), f.x.inner_ret.x.name.clone());
    }
    let rename = |n: &VarIdent| names.get(n).cloned().unwrap_or_else(|| n.clone());
    map_expr_visitor_env(
        e,
        &mut VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            let x = match &e.x {
                ExprX::Var(n) => ExprX::Var(rename(n)),
                ExprX::VarAt(n, at) => ExprX::VarAt(rename(n), *at),
                ExprX::ReadPlace(p, _) => ExprX::ReadPlace(
                    p.clone(),
                    UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
                ),
                _ => e.x.clone(),
            };
            Ok(SpannedTyped::new(&f.span, &e.typ, x))
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| {
            Ok(if let TypX::TypParam(n) = &**t {
                types.get(n).cloned().unwrap_or_else(|| t.clone())
            } else {
                t.clone()
            })
        },
        &|_, _, p| {
            Ok(SpannedTyped::new(
                &f.span,
                &p.typ,
                if let PlaceX::Local(n) = &p.x { PlaceX::Local(rename(n)) } else { p.x.clone() },
            ))
        },
        &|_, _, p| Ok(p.clone()),
    )
}
fn find(functions: &HashMap<Fun, Function>, k: Model, span: &Span) -> Result<Function, VirErr> {
    let mut found = functions.values().filter(|f| model(f) == Some(k));
    let f = found.next().ok_or_else(|| {
        unsupported(span, "collection templates require the canonical vstd Vec/Seq models")
    })?;
    if found.next().is_some() {
        return Err(unsupported(span, "ambiguous collection model"));
    }
    Ok(f.clone())
}

fn contract(
    f: &Function,
    functions: &HashMap<Fun, Function>,
    k: Kind,
    guarded: bool,
) -> Result<Contract, VirErr> {
    let m = find(
        functions,
        match k {
            Kind::Len => Model::Len,
            Kind::Read => Model::Index,
            Kind::Write => Model::IndexMut,
        },
        &f.span,
    )?;
    if k == Kind::Len {
        return Contract::new(
            boolean(&f.span, true).expr,
            substitute(&m.x.ensure.0[0], &m, f, true)?,
        );
    }
    let pre = substitute(&m.x.require[0], &m, f, false)?;
    if k == Kind::Read {
        let ExprX::Binary(BinaryOp::Eq(Mode::Spec), _, value) = &m.x.ensure.0[0].x else {
            return Err(unsupported(&f.span, "Vec index model relation"));
        };
        let value = substitute(value, &m, f, false)?;
        let result = SpannedTyped::new(
            &f.span,
            &f.x.inner_ret.x.typ,
            ExprX::Var(f.x.inner_ret.x.name.clone()),
        );
        return Contract::new(
            pre,
            SpannedTyped::new(
                &f.span,
                &bool_typ(),
                ExprX::Binary(BinaryOp::Eq(Mode::Spec), result, value),
            ),
        );
    }
    let ExprX::Binary(BinaryOp::Eq(Mode::Spec), after, update) = &m.x.ensure.0[1].x else {
        return Err(unsupported(&f.span, "Vec mutation model relation"));
    };
    let ExprX::Call { target, args, post_args: None, body: None } = &update.x else {
        return Err(unsupported(&f.span, "Vec mutation update model"));
    };
    if args.len() != 3 {
        return Err(unsupported(&f.span, "Vec mutation update arity"));
    }
    let mut args = (**args).clone();
    args[2] =
        SpannedTyped::new(&f.span, &f.x.params[2].x.typ, ExprX::Var(f.x.params[2].x.name.clone()));
    let update = substitute(
        &SpannedTyped::new(
            &f.span,
            &update.typ,
            ExprX::Call {
                target: target.clone(),
                args: Arc::new(args),
                post_args: None,
                body: None,
            },
        ),
        &m,
        f,
        false,
    )?;
    let after = substitute(after, &m, f, false)?;
    let output = if guarded {
        let ExprX::Call { args, .. } = &update.x else { unreachable!() };
        let before = args[0].clone();
        let ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), index, length) = &pre.x else {
            return Err(unsupported(&f.span, "Vec bounds model"));
        };
        let ExprX::Call { target, args: length_args, .. } = &length.x else {
            return Err(unsupported(&f.span, "Vec length model"));
        };
        if length_args.len() != 1 {
            return Err(unsupported(&f.span, "Vec length arity"));
        }
        let length = SpannedTyped::new(
            &f.span,
            &length.typ,
            ExprX::Call {
                target: target.clone(),
                args: Arc::new(vec![before.clone()]),
                post_args: None,
                body: None,
            },
        );
        let guard = SpannedTyped::new(
            &f.span,
            &bool_typ(),
            ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), index.clone(), length),
        );
        SpannedTyped::new(&f.span, &update.typ, ExprX::If(guard, update.clone(), Some(before)))
    } else {
        update
    };
    Contract::new(
        if guarded { boolean(&f.span, true).expr } else { pre },
        SpannedTyped::new(
            &f.span,
            &bool_typ(),
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), after, output),
        ),
    )
}

pub(super) fn infer(f: &Function, functions: &HashMap<Fun, Function>) -> Result<Contract, VirErr> {
    let k = validate_signature(f)?;
    let mut remaining = FORMULA_BUDGET;
    if let Some(body) = &f.x.body {
        crate::ast_visitor::expr_visitor_check(body, &mut |_, e| {
            remaining = remaining.checked_sub(1).ok_or_else(|| {
                unsupported(&e.span, "collection template analysis budget exceeded")
            })?;
            Ok::<_, VirErr>(())
        })?;
    }
    let body =
        single(f.x.body.as_ref().ok_or_else(|| unsupported(&f.span, "missing collection body"))?);
    let matched = if k == Kind::Len {
        call(body, Model::Len, functions)
            .is_some_and(|args| args.len() == 1 && local(&args[0], &f.x.params[0].x.name))
    } else if k == Kind::Read {
        matches!(&body.x, ExprX::ReadPlace(p, _) if matches!(&p.x, PlaceX::Temporary(e)
            if call(e, Model::Index, functions).is_some_and(|args| args.len()==2 && local(&args[0], &f.x.params[0].x.name) && local(&args[1], &f.x.params[1].x.name))))
    } else {
        assignment(body, f, functions) || guarded_assignment(body, f, functions)
    };
    if !matched {
        return Err(unsupported(
            &body.span,
            "collection templates support a direct Vec length/read or a single slot assignment",
        ));
    }
    contract(f, functions, k, k == Kind::Write && !assignment(body, f, functions))
}

pub(super) fn validate_summary(
    f: &Function,
    functions: &HashMap<Fun, Function>,
) -> Result<(), VirErr> {
    crate::infer_proof::CandidateId::from_function(f)?;
    if f.x.body.is_some()
        || f.x.require.len() != 1
        || f.x.ensure.0.len() != 1
        || !f.x.ensure.1.is_empty()
    {
        return Err(unsupported(&f.span, "invalid collection summary"));
    }
    let mut signature = f.x.clone();
    signature.require = Arc::new(vec![]);
    signature.ensure = (Arc::new(vec![]), Arc::new(vec![]));
    let k = validate_signature(&f.new_x(signature))?;
    let expected = contract(
        f,
        functions,
        k,
        k == Kind::Write && matches!(f.x.require[0].x, ExprX::Const(Constant::Bool(true))),
    )?;
    if format!("{:?}", expected.pre) != format!("{:?}", f.x.require[0])
        || format!("{:?}", expected.post) != format!("{:?}", f.x.ensure.0[0])
    {
        return Err(unsupported(
            &f.span,
            "collection summary does not match the checked view/effect schema",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input, span};

    fn fun(krate: CrateId, parts: &[&str]) -> Fun {
        Arc::new(FunX {
            path: Arc::new(PathX {
                krate,
                segments: Arc::new(parts.iter().map(|s| Arc::new((*s).into())).collect()),
            }),
        })
    }

    fn index_model() -> Function {
        let base = function("index", input("x"));
        let mut f = base.x.clone();
        f.name = fun(CrateId::Vstd, &["std_specs", "vec", "vec_index"]);
        f.attrs = Arc::new(FunctionAttrsX::default());
        f.body = None;
        f.unwind_spec = Some(UnwindSpec::NoUnwind);
        f.typ_params = Arc::new(vec![Arc::new("T".into()), Arc::new("A".into())]);
        let ts: Vec<_> = f.typ_params.iter().map(|n| Arc::new(TypX::TypParam(n.clone()))).collect();
        f.typ_bounds = Arc::new(vec![
            Arc::new(GenericBoundX::Trait(
                TraitId::Sizedness(Sizedness::Sized),
                Arc::new(vec![ts[0].clone()]),
            )),
            Arc::new(GenericBoundX::Trait(
                TraitId::Sizedness(Sizedness::Sized),
                Arc::new(vec![ts[1].clone()]),
            )),
            Arc::new(GenericBoundX::Trait(
                TraitId::Path(fun(CrateId::Core, &["alloc", "Allocator"]).path.clone()),
                Arc::new(vec![ts[1].clone()]),
            )),
        ]);
        let vector = Arc::new(TypX::Datatype(
            Dt::Path(fun(CrateId::Alloc, &["vec", "Vec"]).path.clone()),
            Arc::new(ts.clone()),
            Arc::new(vec![]),
        ));
        let parameter_types =
            [models::reference(vector.clone()), Arc::new(TypX::Int(IntRange::USize))];
        f.params = Arc::new(
            f.params
                .iter()
                .zip(parameter_types)
                .map(|(p, typ)| {
                    let mut x = p.x.clone();
                    x.typ = typ;
                    p.new_x(x)
                })
                .collect(),
        );
        let mut ret = f.inner_ret.x.clone();
        ret.typ = models::reference(ts[0].clone());
        f.inner_ret = f.inner_ret.new_x(ret);
        f.outer_ret = f.inner_ret.clone();
        let attrs = CallTargetAttrs {
            autospec: AutospecUsage::Final,
            const_var: false,
            assume_external_allowed: false,
        };
        let view = SpannedTyped::new(
            &span(),
            &seq(ts[0].clone()),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::DynamicResolved {
                        resolved: fun(CrateId::Vstd, &["view", "impl&%8", "view"]),
                        typs: Arc::new(ts.clone()),
                        impl_paths: Arc::new(vec![]),
                        is_trait_default: false,
                    },
                    fun(CrateId::Vstd, &["view", "View", "view"]),
                    Arc::new(vec![vector]),
                    Arc::new(vec![]),
                    attrs.clone(),
                ),
                args: Arc::new(vec![SpannedTyped::new(
                    &span(),
                    &f.params[0].x.typ,
                    ExprX::Var(f.params[0].x.name.clone()),
                )]),
                post_args: None,
                body: None,
            },
        );
        let length = SpannedTyped::new(
            &span(),
            &Arc::new(TypX::Int(IntRange::Nat)),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    fun(CrateId::Vstd, &["seq", "Seq", "len"]),
                    Arc::new(vec![ts[0].clone()]),
                    Arc::new(vec![]),
                    attrs,
                ),
                args: Arc::new(vec![view]),
                post_args: None,
                body: None,
            },
        );
        let index =
            SpannedTyped::new(&span(), &f.params[1].x.typ, ExprX::Var(f.params[1].x.name.clone()));
        f.require = Arc::new(vec![SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), index, length),
        )]);
        f.ensure = (Arc::new(vec![boolean(&span(), true).expr]), Arc::new(vec![]));
        base.new_x(f)
    }

    #[test]
    fn maximality_gate_requires_the_canonical_exact_index_domain() {
        let f = index_model();
        assert_eq!(model(&f), Some(Model::Index));
        let mut stronger = f.x.clone();
        stronger.require = Arc::new(vec![SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::Logical(LogicalOp::And, f.x.require[0].clone(), boolean(&span(), false).expr),
        )]);
        assert_eq!(model(&f.new_x(stronger)), None);
        let ExprX::Binary(_, index, length) = &f.x.require[0].x else { unreachable!() };
        let mut off_by_one = f.x.clone();
        off_by_one.require = Arc::new(vec![SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), index.clone(), length.clone()),
        )]);
        assert_eq!(model(&f.new_x(off_by_one)), None);
        let mut unconstrained = f.x.clone();
        unconstrained.require = Arc::new(vec![boolean(&span(), true).expr]);
        assert_eq!(model(&f.new_x(unconstrained)), None);
    }

    #[test]
    fn collection_model_identity_signature_and_body_are_closed() {
        let f = index_model();
        let mut user = f.x.clone();
        user.name =
            fun(CrateId::Id(Arc::new("user".into()), 1), &["std_specs", "vec", "vec_index"]);
        assert_eq!(model(&f.new_x(user)), None);
        let mut body = f.x.clone();
        body.body = Some(boolean(&span(), true).expr);
        assert_eq!(model(&f.new_x(body)), None);
        let mut unsafe_model = f.x.clone();
        Arc::make_mut(&mut unsafe_model.attrs).is_unsafe = true;
        assert_eq!(model(&f.new_x(unsafe_model)), None);
        let mut wrong_index = f.x.clone();
        let mut ps = (*wrong_index.params).clone();
        let mut p = ps[1].x.clone();
        p.typ = Arc::new(TypX::Int(IntRange::I(8)));
        ps[1] = ps[1].new_x(p);
        wrong_index.params = Arc::new(ps);
        assert_eq!(model(&f.new_x(wrong_index)), None);
    }
}
