//! Untrusted candidate generation and modular composition for Verus Infer.
//!
//! Executable code is preserved. Candidates are checked by ordinary
//! verification, and `infer_checks` checks their precision against the lowered body.

use crate::ast::*;
use crate::ast_util::{bool_typ, int_typ};
use crate::messages::{Span, error};
use num_bigint::BigInt;
use num_traits::One;
use std::collections::HashMap;
use std::sync::Arc;

mod abstract_infer;
mod arithmetic;
mod arrays;
mod bitvector_hints;
mod bitwise;
mod collections;
mod contract;
mod counting_goals;
mod eligibility;
mod enums;
mod exits;
mod goals;
mod integer_models;
mod loops;
mod models;
mod newtypes;
mod propagation;
mod registry;
mod records;
mod record_models;
mod sequence_views;
mod trait_getters;
mod wrapper_traits;
mod owned;
mod cloning;

pub fn is_owned_summary(f:&Function)->bool{f.x.attrs.infer && f.x.attrs.infer_owned}

/// Concrete exact-mode implementations whose proposed domain must be proved
/// equivalent to the ordinary inherited trait precondition.
pub fn is_wrapper_trait(f: &Function) -> bool {
    wrapper_traits::kind(f).is_some()
}

pub fn is_inference_contract(f: &Function) -> bool {
    f.x.attrs.infer_sequence_view || f.x.attrs.infer_trait_getter || f.x.attrs.infer_trait_observation || f.x.attrs.infer_clone_contract || view_bounds_goals::lemma(f).is_some()
}

pub fn prepare_trait_contracts(functions: &[Function], imported: &[Function]) -> Vec<Function> {
    trait_getters::prepare(functions, imported)
}

/// Internal structural evidence, not a trusted standard-library model.
pub fn is_inferred_sequence_view(f: &Function) -> bool { f.x.attrs.infer_sequence_view }

pub fn validate_inferred_sequence_views(functions: &[Function], datatypes: &[Datatype]) -> Result<(),VirErr> {
    sequence_views::validate_imports(functions,datatypes)?;
    cloning::validate(functions,datatypes)?;
    trait_getters::validate(functions)
}

pub(crate) fn record_sequence_view(typ: &Typ, functions: &HashMap<Fun, Function>) -> Option<Function> {
    sequence_views::observation(typ, functions)
}
mod report;
mod simplify;
mod slices;
mod summary;
mod widening;

pub use report::{GeneratedCandidate, GenerationReport};

pub(crate) use bitvector_hints::decorate as bitvector_hints;
pub use goals::prepare as prepare_proof_goals;
pub use counting_goals::prepare as prepare_counting_goals;
mod view_bounds_goals;
pub use view_bounds_goals::prepare as prepare_view_bound_goals;

pub fn is_state_summary(function: &Function) -> bool {
    function.x.attrs.infer
        && (collections::kind(function) == Some(collections::Kind::Write)
            || records::candidate(function) && records::mutable(function))
}

pub fn is_record_summary(function: &Function) -> bool { function.x.attrs.infer && records::candidate(function) && records::mutable(function) }

pub(crate) fn is_record_callee(f:&Function,typs:&Typs)->bool {(records::candidate(f)||is_owned_summary(f)) && f.x.typ_params.len()==typs.len()}

pub fn is_collection_summary(function: &Function) -> bool {
    function.x.attrs.infer && collections::kind(function).is_some()
}

pub(crate) fn is_vec_maximality_model(function: &Function, typs: &Typs) -> bool {
    matches!(
        collections::model(function),
        Some(collections::Model::Index | collections::Model::IndexMut)
    )
        && typs.len() == 2
        && matches!(&*typs[1], TypX::Datatype(Dt::Path(p), ts, _) if ts.is_empty()
            && models::path_is(p, CrateId::Alloc, &["alloc", "Global"]))
}

/// Resolve the existing, signature-validated Vec view model for state precision.
/// This does not inspect or reuse the candidate's postcondition.
pub(crate) fn record_vec_view(functions: &HashMap<Fun, Function>) -> Option<Function> {
    let mut views = functions.values()
        .filter(|f| collections::model(f) == Some(collections::Model::View));
    let view = views.next()?.clone();
    views.next().is_none().then_some(view)
}

pub(crate) fn record_vec_args(typ: &Typ) -> Option<&Typs> {
    collections::vec_args(typ)
}

use contract::Contract;
use eligibility::{scalar, validate_body, validate_signature};
use summary::instantiate;

/// Proven outcomes, independent of requested precision and candidate caches.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProofStatus {
    pub soundness: bool,
    pub maximality: bool,
    pub exactness: bool,
}

impl ProofStatus {
    /// The publication barrier is identical for same-crate callers, exports,
    /// and imported interfaces. Optional precision probes never replace soundness.
    pub fn satisfies(&self, allow_approx: bool) -> bool {
        crate::infer_proof::policy::accepts(
            allow_approx,
            [self.soundness, self.maximality, self.exactness],
        )
    }
}

// Bound expanded formulas as well as source traversal: repeated substitution of
// locals can otherwise create exponentially large formulas from very small code.
const FORMULA_BUDGET: usize = 4096;

#[derive(Clone)]
struct Term {
    view: Option<Box<slices::View>>,
    expr: Expr,
    size: usize,
}

struct Summary {
    safe: Term,
    value: Option<Term>,
}

fn term(span: &Span, typ: &Typ, x: ExprX, size: usize) -> Result<Term, VirErr> {
    if size > FORMULA_BUDGET {
        return Err(error(span, "Verus Infer incomplete: expanded formula budget exceeded"));
    }
    Ok(Term { view: None, expr: SpannedTyped::new(span, typ, x), size })
}

fn boolean(span: &Span, value: bool) -> Term {
    Term {
        view: None,
        expr: SpannedTyped::new(span, &bool_typ(), ExprX::Const(Constant::Bool(value))),
        size: 1,
    }
}

fn is_bool(t: &Term, value: bool) -> bool {
    matches!(&t.expr.x, ExprX::Const(Constant::Bool(b)) if *b == value)
}

fn logical(span: &Span, op: LogicalOp, a: Term, b: Term) -> Result<Term, VirErr> {
    if op == LogicalOp::And {
        if is_bool(&a, true) {
            return Ok(b);
        }
        if is_bool(&b, true) {
            return Ok(a);
        }
        if is_bool(&a, false) || is_bool(&b, false) {
            return Ok(boolean(span, false));
        }
    }
    if op == LogicalOp::Or {
        if is_bool(&a, false) {
            return Ok(b);
        }
        if is_bool(&b, false) {
            return Ok(a);
        }
        if is_bool(&a, true) || is_bool(&b, true) {
            return Ok(boolean(span, true));
        }
    }
    term(span, &bool_typ(), ExprX::Logical(op, a.expr, b.expr), 1 + a.size + b.size)
}

fn conditional(span: &Span, c: Term, a: Term, b: Term) -> Result<Term, VirErr> {
    if is_bool(&c, true) {
        return Ok(a);
    }
    if is_bool(&c, false) {
        return Ok(b);
    }
    if is_bool(&a, true) && is_bool(&b, true) {
        return Ok(a);
    }
    let typ = a.expr.typ.clone();
    term(span, &typ, ExprX::If(c.expr, a.expr, Some(b.expr)), 1 + c.size + a.size + b.size)
}

fn unsupported(span: &Span, feature: &str) -> VirErr {
    error(span, format!("Verus Infer unsupported: {feature}"))
}

fn value(summary: &Summary, span: &Span) -> Result<Term, VirErr> {
    summary.value.clone().ok_or_else(|| unsupported(span, "expected a supported return value"))
}

struct Engine<'a> {
    callees: &'a HashMap<Fun, Function>,
    locals: HashMap<VarIdent, Term>,
    remaining: usize,
    exits: Option<exits::Exits>,
}

impl Engine<'_> {
    fn spend(&mut self, span: &Span) -> Result<(), VirErr> {
        if self.remaining == 0 {
            return Err(error(span, "Verus Infer incomplete: analysis budget exceeded"));
        }
        self.remaining -= 1;
        Ok(())
    }

    fn place(&mut self, place: &Place) -> Result<Summary, VirErr> {
        self.spend(&place.span)?;
        match &place.x {
            PlaceX::Local(name) => self.local(&place.span, name),
            PlaceX::Temporary(expr) => self.expr(expr),
            PlaceX::Index(base, index, ArrayKind::Array, BoundsCheck::Error)
                if arrays::shape(&base.typ).is_some() =>
            {
                self.array_index(&place.span, base, index)
            }
            PlaceX::Index(base, index, ArrayKind::Slice, BoundsCheck::Error)
                if slices::element(&base.typ).is_some() =>
            {
                self.array_index(&place.span, base, index)
            }
            PlaceX::Field(op, base)
                if op.check == VariantCheck::None
                    && op.field == crate::def::positional_field_ident(0)
                    && enums::supported(&base.typ) =>
            {
                let base = self.place(base)?;
                let selected = enums::field(&place.span, value(&base, &place.span)?, &op.variant)?;
                Ok(Summary { safe: base.safe, value: Some(selected) })
            }
            _ => Err(unsupported(&place.span, "references, projections, and mutable state")),
        }
    }

    fn local(&self, span: &Span, name: &VarIdent) -> Result<Summary, VirErr> {
        let value = self
            .locals
            .get(name)
            .ok_or_else(|| unsupported(span, "uninitialized or nonlocal value"))?
            .clone();
        Ok(Summary { safe: boolean(span, true), value: Some(value) })
    }

    fn expr(&mut self, expr: &Expr) -> Result<Summary, VirErr> {
        self.spend(&expr.span)?;
        let span = &expr.span;
        let pure = |value| Ok(Summary { safe: boolean(span, true), value: Some(value) });
        if let Some(arg) = widening::call(expr, self.callees) {
            let arg = self.expr(arg)?;
            let TypX::Int(range) = &*expr.typ else { unreachable!() };
            return Ok(Summary { safe: arg.safe.clone(), value: Some(summary::unary(span,
                UnaryOp::Clip { range: *range, truncate: true }, value(&arg, span)?)?) });
        }
        if let Some(n) = arithmetic::negative_literal(expr) {
            return pure(term(span, &int_typ(), ExprX::Const(Constant::Int(n)), 1)?);
        }
        if let Some(range) = slices::range_call(expr, self.callees)? {
            return self.subslice(expr, range);
        }
        match &expr.x {
            _ if arithmetic::arch_constant(expr).is_some() => {
                Ok(Summary { safe: boolean(span, true), value: arithmetic::arch_constant(expr) })
            }
            ExprX::Const(c @ (Constant::Bool(_) | Constant::Int(_))) => {
                let typ = if matches!(c, Constant::Int(_)) { int_typ() } else { bool_typ() };
                pure(term(span, &typ, ExprX::Const(c.clone()), 1)?)
            }
            ExprX::Ctor(dt, variant, fields, None)
                if matches!(&*expr.typ, TypX::Datatype(actual, ..) if actual == dt)
                    && enums::supported(&expr.typ) =>
            {
                if fields.len() > 1
                    || fields
                        .first()
                        .is_some_and(|f| f.name != crate::def::positional_field_ident(0))
                {
                    return Err(unsupported(span, "unsupported enum fields"));
                }
                let field = fields.first().map(|f| self.expr(&f.a)).transpose()?;
                let safe =
                    field.as_ref().map(|f| f.safe.clone()).unwrap_or_else(|| boolean(span, true));
                let output = enums::ctor(
                    span,
                    &expr.typ,
                    variant,
                    field.as_ref().map(|f| value(f, span)).transpose()?,
                )?;
                Ok(Summary { safe, value: Some(output) })
            }
            ExprX::Match(place, arms, irrefutable) => {
                if let Some(exits) = &self.exits {
                    if let Some(p) = propagation::recognize(expr, self.callees, &exits.result_type)?
                    {
                        return self.propagate(expr, p);
                    }
                }
                self.match_expr(span, &expr.typ, place, arms, *irrefutable)
            }
            ExprX::Var(name) => self.local(span, name),
            ExprX::ReadPlace(place, _) => self.place(place),
            ExprX::ArrayLiteral(elements) => {
                let mut safe = boolean(span, true);
                let mut values = Vec::new();
                for element in elements.iter() {
                    let summary = self.expr(element)?;
                    safe = logical(span, LogicalOp::And, safe, summary.safe.clone())?;
                    values.push(value(&summary, span)?);
                }
                Ok(Summary { safe, value: Some(arrays::literal(span, &expr.typ, values)?) })
            }
            ExprX::Unary(op @ UnaryOp::BitNot(_), arg) if bitwise::unary_valid(*op, &arg.typ) => {
                let arg_summary = self.expr(arg)?;
                let operand = bitwise::coerce(span, &arg.typ, value(&arg_summary, span)?)?;
                Ok(Summary {
                    safe: arg_summary.safe,
                    value: Some(summary::unary(span, *op, operand)?),
                })
            }
            ExprX::Unary(UnaryOp::Not, arg) if matches!(&*arg.typ, TypX::Bool) => {
                let arg = self.expr(arg)?;
                let v = value(&arg, span)?;
                Ok(Summary {
                    safe: arg.safe,
                    value: Some(term(
                        span,
                        &bool_typ(),
                        ExprX::Unary(UnaryOp::Not, v.expr),
                        1 + v.size,
                    )?),
                })
            }
            ExprX::Unary(UnaryOp::Clip { range, .. }, arg) if arithmetic::exact_range(*range) => {
                // rust_to_vir also inserts these coercions around integer
                // division/remainder. Preserve their exact truncation semantics.
                let arg = self.expr(arg)?;
                let v = value(&arg, span)?;
                Ok(Summary {
                    safe: arg.safe,
                    value: Some(term(
                        span,
                        &expr.typ,
                        ExprX::Unary(UnaryOp::Clip { range: *range, truncate: true }, v.expr),
                        1 + v.size,
                    )?),
                })
            }
            ExprX::Logical(op, lhs, rhs) => {
                let lhs = self.expr(lhs)?;
                let a = value(&lhs, span)?;
                let entry = self.exits.as_ref().map(|e| (e.active.clone(), e.values.len()));
                if let Some((entry, _)) = &entry {
                    let guard = match op {
                        LogicalOp::And | LogicalOp::Implies => a.clone(),
                        LogicalOp::Or => exits::not(span, a.clone())?,
                    };
                    self.exits.as_mut().unwrap().active =
                        logical(span, LogicalOp::And, entry.clone(), guard)?;
                }
                let rhs = self.expr(rhs)?;
                if let Some((entry, count)) = entry {
                    let exits = self.exits.as_mut().unwrap();
                    exits.active = if exits.values.len() == count {
                        entry
                    } else {
                        let skip = match op {
                            LogicalOp::And | LogicalOp::Implies => exits::not(span, a.clone())?,
                            LogicalOp::Or => a.clone(),
                        };
                        logical(
                            span,
                            LogicalOp::Or,
                            logical(span, LogicalOp::And, entry, skip)?,
                            exits.active.clone(),
                        )?
                    };
                }
                let b = value(&rhs, span)?;
                let rhs_safe = match op {
                    LogicalOp::And | LogicalOp::Implies => {
                        conditional(span, a.clone(), rhs.safe, boolean(span, true))?
                    }
                    LogicalOp::Or => conditional(span, a.clone(), boolean(span, true), rhs.safe)?,
                };
                Ok(Summary {
                    safe: logical(span, LogicalOp::And, lhs.safe, rhs_safe)?,
                    value: Some(logical(span, *op, a, b)?),
                })
            }
            ExprX::Binary(
                BinaryOp::Index(ArrayKind::Array | ArrayKind::Slice, BoundsCheck::Error),
                lhs,
                rhs,
            ) => {
                let array = self.expr(lhs)?;
                let index = self.expr(rhs)?;
                self.index_summary(span, array, index)
            }
            ExprX::Binary(op @ BinaryOp::Bitwise(..), lhs, rhs) => {
                let (pure, shift) = bitwise::source(*op, &lhs.typ, &rhs.typ, &expr.typ)
                    .ok_or_else(|| unsupported(span, "invalid executable bitwise operation"))?;
                let l = self.expr(lhs)?;
                let r = self.expr(rhs)?;
                let a = bitwise::coerce(span, &lhs.typ, value(&l, span)?)?;
                let mut b = value(&r, span)?;
                if shift.is_none() {
                    b = bitwise::coerce(span, &rhs.typ, b)?;
                }
                let mut safe = logical(span, LogicalOp::And, l.safe, r.safe)?;
                if let Some(width) = shift {
                    let valid = bitwise::shift_safe(span, b.clone(), width)?;
                    safe = logical(span, LogicalOp::And, safe, self.require_active(span, valid)?)?;
                }
                Ok(Summary { safe, value: Some(summary::binary(span, pure, a, b)?) })
            }
            ExprX::Binary(op, lhs, rhs) => {
                let lhs = self.expr(lhs)?;
                let rhs = self.expr(rhs)?;
                let a = value(&lhs, span)?;
                let b = value(&rhs, span)?;
                let mut safe = logical(span, LogicalOp::And, lhs.safe, rhs.safe)?;
                let (pure_op, range, div_check) =
                    eligibility::source_binary(*op).ok_or_else(|| {
                        unsupported(span, "this binary operation or arithmetic policy")
                    })?;
                let out = summary::binary(span, pure_op, a.clone(), b.clone())?;
                if let Some(range) = range {
                    let bounds = arithmetic::in_range(span, out.clone(), range)?;
                    safe = logical(span, LogicalOp::And, safe, self.require_active(span, bounds)?)?;
                }
                if div_check {
                    let signed = match op {
                        BinaryOp::Arith(
                            ArithOp::TruncatingDiv(_, SignedDivEdgeCaseBehavior::Error(width))
                            | ArithOp::TruncatingMod(_, SignedDivEdgeCaseBehavior::Error(width)),
                        ) => Some(arithmetic::signed_range(*width)),
                        _ => None,
                    };
                    let valid = arithmetic::division_valid(span, a, b, signed)?;
                    safe = logical(span, LogicalOp::And, safe, self.require_active(span, valid)?)?;
                }
                Ok(Summary { safe, value: Some(out) })
            }
            ExprX::If(cond, then_expr, else_expr) => {
                let cond = self.expr(cond)?;
                let c = value(&cond, span)?;
                let entry = self.exits.as_ref().map(|e| (e.active.clone(), e.values.len()));
                if let Some((entry, _)) = &entry {
                    self.exits.as_mut().unwrap().active =
                        logical(span, LogicalOp::And, entry.clone(), c.clone())?;
                }
                let a = self.expr(then_expr)?;
                let a_live = self.exits.as_ref().map(|e| e.active.clone());
                if let Some((entry, _)) = &entry {
                    self.exits.as_mut().unwrap().active =
                        logical(span, LogicalOp::And, entry.clone(), exits::not(span, c.clone())?)?;
                }
                let b = match else_expr {
                    Some(expr) => self.expr(expr)?,
                    None => Summary { safe: boolean(span, true), value: None },
                };
                if let Some((entry, count)) = entry {
                    let exits = self.exits.as_mut().unwrap();
                    exits.active = if exits.values.len() == count {
                        entry
                    } else {
                        logical(span, LogicalOp::Or, a_live.unwrap(), exits.active.clone())?
                    };
                }
                let safe_branches = conditional(span, c.clone(), a.safe, b.safe)?;
                let safe = logical(span, LogicalOp::And, cond.safe, safe_branches)?;
                let out = if self.exits.is_some() && enums::unit(&expr.typ) {
                    Some(exits::unit_value(span)?)
                } else {
                    match (a.value, b.value) {
                        (Some(a), Some(b)) => Some(summary::conditional(span, c, a, b)?),
                        (None, None) => None,
                        _ => {
                            return Err(unsupported(span, "branches with different result shapes"));
                        }
                    }
                };
                Ok(Summary { safe, value: out })
            }
            ExprX::Block(stmts, tail) => {
                let saved = self.locals.clone();
                let mut safe = boolean(span, true);
                for stmt in stmts.iter() {
                    self.spend(&stmt.span)?;
                    let summary = match &stmt.x {
                        StmtX::Expr(expr) => self.expr(expr)?,
                        StmtX::Decl { pattern, mode, init: Some(init), els, .. } => {
                            if !matches!(mode, None | Some((Mode::Exec, _))) {
                                return Err(unsupported(&stmt.span, "ghost or tracked locals"));
                            }
                            self.declaration(&stmt.span, pattern, init, els.as_ref())?
                        }
                        _ => {
                            return Err(unsupported(
                                &stmt.span,
                                "uninitialized bindings or let-else",
                            ));
                        }
                    };
                    safe = logical(span, LogicalOp::And, safe, summary.safe)?;
                }
                let tail = match tail {
                    Some(expr) => self.expr(expr)?,
                    None => Summary { safe: boolean(span, true), value: None },
                };
                self.locals = saved;
                Ok(Summary {
                    safe: logical(span, LogicalOp::And, safe, tail.safe)?,
                    value: tail.value,
                })
            }
            ExprX::Loop { .. } => Err(unsupported(span, "loops")),
            ExprX::NeverToAny(inner) => self.never_value(expr, inner),
            ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Static, name, typs, impls, _),
                args,
                post_args: None,
                body: None,
            } => {
                if let Some(model) = models::array_call(expr, self.callees)? {
                    return self.array_model_call(expr, model);
                }
                if self.callees.get(name).is_some_and(|f| {
                    arithmetic::division_model(f) || slices::model(f) || arrays::equality_model(f)
                }) {
                    return Err(unsupported(
                        span,
                        "signed division models are implicit dependencies, not executable callees",
                    ));
                }
                if self.callees.get(name).and_then(propagation::model).is_some() {
                    return Err(unsupported(
                        span,
                        "propagation helpers require a complete Option/Result ? expression",
                    ));
                }
                if !typs.is_empty() || !impls.is_empty() {
                    return Err(unsupported(
                        span,
                        "only static, nongeneric inferred calls are supported",
                    ));
                }
                let callee = self
                    .callees
                    .get(name)
                    .ok_or_else(|| {
                        unsupported(span, "calls require an acyclic same-crate inferred callee or certified imported summary")
                    })?
                    .clone();
                if integer_models::recognize(&callee).is_some() {
                    if args.len() != callee.x.params.len() {
                        return Err(unsupported(span, "integer arithmetic argument count"));
                    }
                    let mut safe = boolean(span, true);
                    let mut values = Vec::new();
                    for arg in args.iter() {
                        let arg = self.expr(arg)?;
                        safe = logical(span, LogicalOp::And, safe, arg.safe.clone())?;
                        values.push(value(&arg, span)?);
                    }
                    return Ok(Summary {
                        safe,
                        value: Some(integer_models::value(span, &callee, values)?),
                    });
                }
                if args.len() != callee.x.params.len() {
                    return Err(unsupported(span, "callee argument count mismatch"));
                }
                let mut safe = boolean(span, true);
                let mut substitutions = HashMap::new();
                for (arg, param) in args.iter().zip(callee.x.params.iter()) {
                    let arg = self.expr(arg)?;
                    safe = logical(span, LogicalOp::And, safe, arg.safe.clone())?;
                    substitutions.insert(param.x.name.clone(), value(&arg, span)?);
                }
                let pre = instantiate(&callee.x.require[0], &substitutions)?;
                let ExprX::Binary(BinaryOp::Eq(_), _, output) = &callee.x.ensure.0[0].x else {
                    return Err(unsupported(span, "invalid candidate summary"));
                };
                Ok(Summary {
                    safe: logical(span, LogicalOp::And, safe, self.require_active(span, pre)?)?,
                    value: Some(instantiate(output, &substitutions)?),
                })
            }
            ExprX::Call { .. } => {
                Err(unsupported(span, "only static, nongeneric inferred calls are supported"))
            }
            ExprX::Return(value) => self.return_expr(span, value.as_ref()),
            ExprX::Assign { .. } | ExprX::BorrowMut(_) => Err(unsupported(span, "mutable state")),
            ExprX::Unary(UnaryOp::Clip { .. }, _) => Err(unsupported(span, "integer casts")),
            ExprX::AssertAssume { .. } | ExprX::Ghost { .. } | ExprX::AssertBy { .. } => {
                Err(unsupported(span, "proof statements and assumptions in inferred bodies"))
            }
            _ => Err(unsupported(
                span,
                "this expression; the supported subset includes fixed-width integer arithmetic, booleans, immutable locals, and branches",
            )),
        }
    }
}

/// Dependencies are collected from executable bodies, never from candidate contracts.
/// A DFS provides deterministic callee-first ordering and rejects every cycle.
pub fn dependency_order(functions: &[Function]) -> Result<Vec<Fun>, VirErr> {
    order_functions(functions, true)
}

/// Order the verifier's later, resolution-expanded bodies. Source eligibility
/// and helper recognition have already run before candidate installation.
/// Inspect every call here so compiler-generated scaffolding cannot hide an
/// inferred dependency from the publication barrier.
pub fn verification_order(functions: &[Function]) -> Result<Vec<Fun>, VirErr> {
    order_functions(functions, false)
}

fn order_functions(functions: &[Function], source: bool) -> Result<Vec<Fun>, VirErr> {
    fn visit(
        f: &Function,
        all: &HashMap<Fun, Function>,
        source: bool,
        active: &mut std::collections::HashSet<Fun>,
        done: &mut std::collections::HashSet<Fun>,
        order: &mut Vec<Fun>,
    ) -> Result<(), VirErr> {
        if done.contains(&f.x.name) {
            return Ok(());
        }
        if active.len() >= 128 {
            return Err(error(&f.span, "Verus Infer incomplete: call graph depth budget exceeded"));
        }
        if !active.insert(f.x.name.clone()) {
            return Err(unsupported(&f.span, "recursive inferred call graph"));
        }
        if source && is_owned_summary(f) { owned::validate_source(f)?; }
        if let Some(body) = &f.x.body {
            let mut call = |e: &Expr, helper: bool| {
                // Resolution inserts borrow-expiration scaffolding. Traverse it
                // normally, but ignore it when resolving this dependency edge.
                let plain = if !source {
                    if let ExprX::Call { target, args, .. } = &e.x {
                        Some(e.new_x(ExprX::Call {
                            target: target.clone(), args: args.clone(), post_args: None, body: None,
                        }))
                    } else { None }
                } else { None };
                let e = plain.as_ref().unwrap_or(e);
                let rewritten = wrapper_traits::source_call(e, all);
                let e = rewritten.as_ref().unwrap_or(e);
                if is_owned_summary(f) && cloning::call(e,all){return Ok(());}
                if is_owned_summary(f) && owned::model_call(e,all).is_some(){return Ok(());}
                if !source && is_owned_summary(f) && matches!(&e.x,ExprX::Call{target:CallTarget::Fun(CallTargetKind::DynamicResolved{resolved,..},..),..}
                    if all.get(resolved).is_some_and(|f|f.x.attrs.infer_sequence_view || collections::model(f)==Some(collections::Model::View))){return Ok(());}
                if records::candidate(f) && trait_getters::source_call(e,all).is_some() {return Ok(());}
                if widening::call(e, all).is_some() { return Ok(()); }
                let range = slices::range_call(e, all)?;
                let e = range.as_ref().map(|r| &r.helper).unwrap_or(e);
                if let ExprX::Call {
                    target: CallTarget::Fun(CallTargetKind::Static, name, typs, impls, _),
                    post_args,
                    body,
                    ..
                } = &e.x
                {
                    if source && (post_args.is_some() || body.is_some()) {
                        return Err(unsupported(&e.span,"source calls cannot contain embedded verification code"));
                    }
                    let callee = all.get(name).filter(|f| f.x.attrs.infer || is_approved_model(f)).ok_or_else(|| {
                        unsupported(&e.span, "calls require an acyclic same-crate inferred callee or certified imported summary")
                    })?;
                    if arithmetic::division_model(callee)
                        || slices::model(callee)
                        || arrays::equality_model(callee)
                    {
                        return Err(unsupported(
                            &e.span,
                            "signed division models are implicit dependencies, not executable callees",
                        ));
                    }
                    if source && propagation::model(callee).is_some() && !helper {
                        return Err(unsupported(
                            &e.span,
                            "propagation helpers require a complete Option/Result ? expression",
                        ));
                    }
                    if source
                        && !helper
                        && models::array_call(e, all)?.is_none()
                        && !collections::source_call(e, all)
                        && !record_models::as_ref(callee)
                        && !(records::candidate(f) && records::candidate(callee))
                        && !(is_owned_summary(f) && (is_owned_summary(callee)||callee.x.typ_params.is_empty()))
                        && (!typs.is_empty() || !impls.is_empty())
                    {
                        return Err(unsupported(&e.span, "generic calls"));
                    }
                    if callee.x.attrs.infer {
                        visit(callee, all, source, active, done, order)?;
                    }
                } else if matches!(e.x, ExprX::Call { .. }) {
                    if matches!(&e.x,ExprX::Call{target:CallTarget::Fun(CallTargetKind::Dynamic,..),..}) {
                        return Err(unsupported(&e.span,"trait inference requires a certified precondition-free getter with one exact observation equation"));
                    }
                    return Err(unsupported(
                        &e.span,
                        "only static, nongeneric inferred calls are supported",
                    ));
                }
                Ok(())
            };
            if source {
                propagation::visit_calls(body, &f.x.inner_ret.x.typ, all, &mut call)?;
            } else {
                crate::ast_visitor::expr_visitor_check(body, &mut |_, e| call(e, false))?;
            }
        }
        active.remove(&f.x.name);
        done.insert(f.x.name.clone());
        order.push(f.x.name.clone());
        Ok(())
    }
    let all = functions.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
    let mut order = Vec::new();
    let mut active = std::collections::HashSet::new();
    let mut done = std::collections::HashSet::new();
    for f in functions.iter().filter(|f| f.x.attrs.infer) {
        visit(f, &all, source, &mut active, &mut done, &mut order)?;
    }
    Ok(order)
}

pub(crate) fn is_subslice_maximality_model(
    function: &Function,
    resolved: &Function,
    typs: &Typs,
) -> bool {
    slices::maximality_model(function, resolved, typs)
}

pub fn is_checked_arithmetic(function: &Function) -> bool {
    enums::checked(function).is_some()
}

pub fn is_approved_model(function: &Function) -> bool {
    cloning::model(function) || widening::model(function) || owned::model(function).is_some() || wrapper_traits::into_model(function) || record_models::as_ref(function) || collections::model(function).is_some()
        || integer_models::recognize(function).is_some()
        || models::array_model(function).is_some()
        || propagation::model(function).is_some()
        || arithmetic::division_model(function)
        || slices::model(function)
        || slices::range_model(function)
        || arrays::equality_model(function)
}

pub fn trusted_model_dependencies(functions: &[Function], available: &[Function]) -> Vec<Function> {
    let available: HashMap<_, _> =
        available.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
    let mut used = std::collections::BTreeMap::new();
    for f in functions.iter().filter(|f| f.x.attrs.infer) {
        if collections::kind(f).is_some() || records::candidate(f) || is_owned_summary(f) {
            for model in available.values().filter(|f| collections::model(f).is_some()) {
                used.insert(model.x.name.clone(), model.clone());
            }
        }
        if arrays::shape(&f.x.inner_ret.x.typ).is_some() {
            for model in available.values().filter(|f| arrays::equality_model(f)) {
                used.insert(model.x.name.clone(), model.clone());
            }
        }
        if let Some(body) = &f.x.body {
            crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
                if let Ok(Some(range)) = slices::range_call(e, &available) {
                    if let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } =
                        &range.helper.x
                    {
                        if let Some(model) = available.get(name) {
                            used.insert(name.clone(), model.clone());
                        }
                    }
                }
                if let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } = &e.x {
                    if let Some(callee) = available.get(name).filter(|f| is_approved_model(f)) {
                        used.insert(name.clone(), callee.clone());
                    }
                }
                if let ExprX::Call { target: CallTarget::Fun(CallTargetKind::DynamicResolved { resolved, .. }, ..), .. } = &e.x {
                    if let Some(callee) = available.get(resolved).filter(|f| widening::model(f) || wrapper_traits::into_model(f) || owned::model(f).is_some()) {
                        used.insert(resolved.clone(), callee.clone());
                    }
                }
                if let Some(name) = arithmetic::division_dependency(e) {
                    if let Some(model) =
                        available.get(&name).filter(|f| arithmetic::division_model(f))
                    {
                        used.insert(name, model.clone());
                    }
                }
                for name in slices::dependencies(e, &available) {
                    if let Some(model) = available.get(&name).filter(|f| slices::model(f)) {
                        used.insert(name, model.clone());
                    }
                }
                crate::ast_visitor::VisitorControlFlow::<()>::Recurse
            });
        }
    }
    used.into_values().collect()
}

pub fn infer_functions(
    functions: &[Function],
    imported: &[Function],
    configuration: &str,
) -> Result<Vec<Function>, VirErr> {
    infer_functions_with_report(
        functions,
        imported,
        configuration,
        &mut GenerationReport::default(),
    )
}

pub fn infer_functions_with_report(
    functions: &[Function],
    imported: &[Function],
    configuration: &str,
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    infer_functions_with_datatypes(functions, imported, &[], configuration, report)
}

pub fn infer_functions_with_datatypes(
    functions: &[Function],
    imported: &[Function],
    datatypes: &[Datatype],
    configuration: &str,
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    *report = GenerationReport::default();
    report.requested = functions.iter().filter(|f| f.x.attrs.infer).count();
    if !functions.iter().any(|f| f.x.attrs.infer) {
        // A Clone provider may contain only an ordinary derived implementation.
        // Its direct-view evidence still needs certification and export gating.
        if functions.iter().any(|f| f.x.name.path.last_segment().as_str() == "__verus_clone_relation") {
            let prepared = sequence_views::prepare(functions, imported, datatypes);
            return Ok(cloning::prepare(&prepared, imported, datatypes));
        }
        return Ok(functions.to_vec());
    }
    let start = std::time::Instant::now();
    let mut registry = registry::Registry::from_environment(configuration);
    let result = registry.infer_with_datatypes(functions, imported, datatypes);
    report.elapsed_micros = start.elapsed().as_micros();
    report.candidates = registry.generated;
    report.failure = result.as_ref().err().map(|e| e.note.clone());
    result
}

pub fn validate_imported_summary_with_context(
    function: &Function,
    datatypes: &[Datatype],
    functions: &[Function],
) -> Result<(), VirErr> {
    let projected = newtypes::Newtypes::new(datatypes).project(function)?;
    if is_owned_summary(&projected){return owned::validate_summary(&projected,&functions.iter().map(|f|(f.x.name.clone(),f.clone())).collect(),datatypes);}
    if records::candidate(&projected) {
        records::validate_summary(&projected, &functions.iter().map(|f|(f.x.name.clone(),f.clone())).collect(), datatypes)
    } else if collections::kind(&projected).is_some() {
        collections::validate_summary(&projected,
            &functions.iter().map(|f| (f.x.name.clone(), f.clone())).collect())
    } else { validate_imported_summary(&projected) }
}

/// Validate the restricted summary shape on the import boundary. Proof
/// guarantees and schema/dependency fingerprints are checked by the driver.
pub fn validate_imported_summary(function: &Function) -> Result<(), VirErr> {
    crate::infer_proof::CandidateId::from_function(function)?;
    let f = &function.x;
    if f.attrs.infer_allow_approx {
        return abstract_infer::validate_summary(function);
    }
    let invalid = || unsupported(&function.span, "invalid imported inferred summary");
    if !f.attrs.infer
        || f.body.is_some()
        || f.require.len() != 1
        || f.ensure.0.len() != 1
        || !f.ensure.1.is_empty()
    {
        return Err(invalid());
    }
    let mut signature = f.clone();
    signature.require = Arc::new(vec![]);
    signature.ensure = (Arc::new(vec![]), Arc::new(vec![]));
    validate_signature(&function.new_x(signature))?;
    let ExprX::Binary(BinaryOp::Eq(Mode::Spec), result, output) = &f.ensure.0[0].x else {
        return Err(invalid());
    };
    if !matches!(&result.x, ExprX::Var(v) if v == &f.inner_ret.x.name) {
        return Err(invalid());
    }
    let params = f
        .params
        .iter()
        .map(|p| {
            (
                p.x.name.clone(),
                Term {
                    view: None,
                    expr: SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())),
                    size: 1,
                },
            )
        })
        .collect();
    let pre = instantiate(&f.require[0], &params)?;
    let output = instantiate(output, &params)?;
    if !matches!(&*pre.expr.typ, TypX::Bool)
        || !enums::compatible(&output.expr.typ, &f.inner_ret.x.typ)
    {
        return Err(invalid());
    }
    Ok(())
}

/// Generate a provisional contract. Only the normal verification pipeline may
/// establish its soundness; this function does not grant any proof status.
pub fn infer_function(function: &Function) -> Result<Function, VirErr> {
    if !function.x.attrs.infer {
        return Ok(function.clone());
    }
    validate_signature(function)?;
    validate_body(function, &HashMap::new())?;
    infer_with_callees(function, &HashMap::new())?.install(function, "standalone")
}

fn infer_with_callees(
    function: &Function,
    callees: &HashMap<Fun, Function>,
) -> Result<Contract, VirErr> {
    if let Some(plan) = loops::recognize(function)? {
        return plan.contract(function);
    }
    if function.x.attrs.infer_allow_approx {
        return abstract_infer::infer(function, callees);
    }
    if uses_approximate_callee(function, callees) {
        return Err(unsupported(
            &function.span,
            "exact inference requires exact-mode callees; use precision = \"sound\" or an ordinary verified caller for sound approximations",
        ));
    }
    infer_exact_with_callees(function, callees)
}

pub fn uses_approximate_callee(function: &Function, functions: &HashMap<Fun, Function>) -> bool {
    let mut found = false;
    if let Some(body) = &function.x.body {
        crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
            if let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } = &e.x {
                found |= functions.get(name).is_some_and(|f| f.x.attrs.infer_allow_approx);
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    found
}

fn infer_exact_with_callees(
    function: &Function,
    callees: &HashMap<Fun, Function>,
) -> Result<Contract, VirErr> {
    validate_signature(function)?;
    let f = &function.x;
    let span = &function.span;
    let body = f.body.as_ref().ok_or_else(|| unsupported(span, "missing executable body"))?;
    let locals = f
        .params
        .iter()
        .map(|p| {
            (
                p.x.name.clone(),
                Term {
                    view: None,
                    expr: SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())),
                    size: 1,
                },
            )
        })
        .collect();
    let mut engine = Engine {
        callees,
        locals,
        remaining: FORMULA_BUDGET,
        exits: exits::Exits::for_body(body, &f.inner_ret.x.typ),
    };
    let summary = engine.expr(body)?;
    let summary = engine.finish_exits(span, summary)?;
    let out = value(&summary, span)?;
    if out.view.is_some() {
        return Err(unsupported(span, "returning borrowed slice views"));
    }
    let result =
        SpannedTyped::new(span, &f.inner_ret.x.typ, ExprX::Var(f.inner_ret.x.name.clone()));
    let ensures = SpannedTyped::new(
        span,
        &bool_typ(),
        ExprX::Binary(BinaryOp::Eq(Mode::Spec), result, out.expr),
    );
    Contract::new(summary.safe.expr, ensures)
}

/// A deliberately small presentation layer. It prints the very same formulas
/// that are verified, never a separately approximated display contract.
pub fn format_contract(function: &Function) -> String {
    let (pre, post) = format_contract_parts(function);
    let suffix =
        if function.x.attrs.infer_allow_approx { " [sound approximation candidate]" } else { "" };
    format!("requires {pre}; ensures {post}{suffix}")
}

/// Separate the checked predicates for structured presentation. Arithmetic is
/// mathematical, with explicit machine casts and Rust signed division/remainder.
pub fn format_contract_parts(function: &Function) -> (String, String) {
    if collections::kind(function).is_some() || records::candidate(function) || is_owned_summary(function) {
        let result_name = VarIdent(Arc::new("result".into()), VarIdentDisambiguate::AirLocal);
        let result = &function.x.inner_ret.x.name;
        let post = crate::ast_visitor::map_expr_place_visitor(&function.x.ensure.0[0],
            &|e| Ok(if matches!(&e.x, ExprX::Var(v) if v == result) {
                e.new_x(ExprX::Var(result_name.clone()))
            } else { e.clone() }),
            &|p| Ok(if matches!(&p.x, PlaceX::Local(v) if v == result) {
                p.new_x(PlaceX::Local(result_name.clone()))
            } else { p.clone() })).expect("collection summary presentation");
        return (format_expr(&function.x.require[0]), format_expr(&post));
    }
    if function.x.attrs.infer_allow_approx {
        let result = Term {
            view: None,
            expr: SpannedTyped::new(
                &function.span,
                &function.x.inner_ret.x.typ,
                ExprX::Var(VarIdent(Arc::new("result".into()), VarIdentDisambiguate::AirLocal)),
            ),
            size: 1,
        };
        let names = HashMap::from([(function.x.inner_ret.x.name.clone(), result)]);
        let post =
            instantiate(&function.x.ensure.0[0], &names).expect("generated interval postcondition");
        return (format_expr(&function.x.require[0]), format_expr(&post.expr));
    }
    let ExprX::Binary(BinaryOp::Eq(_), _, output) = &function.x.ensure.0[0].x else {
        unreachable!()
    };
    (format_expr(&function.x.require[0]), format!("result == {}", format_expr(output)))
}

/// Read-only presentation of installed loop proof annotations, never evidence.
pub fn format_loop_proofs(function: &Function) -> Vec<serde_json::Value> {
    let mut proofs = Vec::new();
    if let Some(body) = &function.x.body {
        crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
            if let ExprX::Loop { invs, decrease, loop_isolation, .. } = &e.x {
                proofs.push(serde_json::json!({
                    "source": e.span.as_string,
                    "invariants": invs.iter().map(|i| format_expr(&i.inv)).collect::<Vec<_>>(),
                    "decreases": decrease.iter().map(format_expr).collect::<Vec<_>>(),
                    "loop_isolation": loop_isolation,
                    "arithmetic_prover": if function.x.attrs.nonlinear { "nonlinear" } else { "default" },
                    "checked_by": "ordinary soundness, including termination; independent precision queries",
                }));
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    proofs
}

fn format_expr(e: &Expr) -> String {
    match &e.x {
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::UnsignedMax), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            "usize::MAX".into()
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::SignedMin), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            "isize::MIN".into()
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::SignedMax), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            "isize::MAX".into()
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::ISize, .. }, a) => {
            format!("({} as isize)", format_expr(a))
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            "usize::BITS".into()
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::USize, .. }, a) => {
            format!("({} as usize)", format_expr(a))
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, .. }, a) => {
            format!("({} as int)", format_expr(a))
        }
        ExprX::ArrayLiteral(elements) => {
            format!("[{}]", elements.iter().map(format_expr).collect::<Vec<_>>().join(", "))
        }
        ExprX::Closure(params,body)=>format!("|{}| {}",params.iter().map(|p|p.name.0.to_string()).collect::<Vec<_>>().join(", "),format_expr(body)),
        ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), a) => format!("({}).len()", format_expr(a)),
        ExprX::Binary(
            BinaryOp::Index(ArrayKind::Array | ArrayKind::Slice, BoundsCheck::Allow),
            a,
            i,
        ) => {
            format!("({})[{}]", format_expr(a), format_expr(i))
        }
        ExprX::Ctor(_, variant, fields, None) => {
            if fields.is_empty() {
                if variant.as_str() == "tuple%0" { "()".into() } else { variant.to_string() }
            } else {
                format!("{}({})", variant, fields.iter().map(|f| format_expr(&f.a)).collect::<Vec<_>>().join(", "))
            }
        }
        ExprX::UnaryOpr(UnaryOpr::IsVariant { variant, .. }, a) => {
            format!("is_{}({})", variant, format_expr(a))
        }
        ExprX::UnaryOpr(UnaryOpr::Field(op), a) => {
            format!("({})->{}_{}", format_expr(a), op.variant, op.field)
        }
        ExprX::Const(Constant::Bool(b)) => b.to_string(),
        ExprX::Const(Constant::Int(i)) => i.to_string(),
        ExprX::Var(v) => v.0.to_string(),
        ExprX::VarAt(v, VarAt::Pre) => format!("old({})", v.0),
        ExprX::ReadPlace(p, _) => format_place(p),
        ExprX::Old(e) => format!("old({})", format_expr(e)),
        ExprX::Unary(UnaryOp::MutRefCurrent, e) => format_expr(e),
        ExprX::Unary(UnaryOp::MutRefFuture(_) | UnaryOp::MutRefFinal(_), e) => format!("final({})", format_expr(e)),
        ExprX::Call { target: CallTarget::Fun(_, name, ..), args, .. } => {
            let p = &name.path;
            if p.krate == CrateId::Vstd && p.segments.first().is_some_and(|s| s.as_str() == "view") && args.len() == 1 {
                format!("({})@", format_expr(&args[0]))
            } else if models::path_is(p, CrateId::Vstd, &["std_specs", "vec", "spec_vec_len"]) && args.len() == 1 {
                format!("({}).len()", format_expr(&args[0]))
            } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "len"]) && args.len() == 1 {
                format!("({}).len()", format_expr(&args[0]))
            } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "index"]) && args.len() == 2 {
                format!("({})[{}]", format_expr(&args[0]), format_expr(&args[1]))
            } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "update"]) && args.len() == 3 {
                format!("({}).update({}, {})", format_expr(&args[0]), format_expr(&args[1]), format_expr(&args[2]))
            } else { format!("{}({})", crate::ast_util::fun_as_friendly_rust_name(name), args.iter().map(format_expr).collect::<Vec<_>>().join(", ")) }
        }
        ExprX::Logical(op, a, b) => format!(
            "({} {} {})",
            format_expr(a),
            match op {
                LogicalOp::And => "&&",
                LogicalOp::Or => "||",
                LogicalOp::Implies => "==>",
            },
            format_expr(b)
        ),
        ExprX::Unary(UnaryOp::BitNot(_), a) => format!("!({})", format_expr(a)),
        ExprX::Binary(BinaryOp::Bitwise(op, _), a, b) => {
            let symbol = match op {
                BitwiseOp::BitAnd => "&",
                BitwiseOp::BitOr => "|",
                BitwiseOp::BitXor => "^",
                BitwiseOp::Shl(..) => "<<",
                BitwiseOp::Shr => ">>",
            };
            format!("({} {symbol} {})", format_expr(a), format_expr(b))
        }
        ExprX::Unary(UnaryOp::Not, a) => format!("!({})", format_expr(a)),
        ExprX::Unary(UnaryOp::Clip { range: IntRange::U(bits), .. }, a) => {
            format!("({} as u{bits})", format_expr(a))
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::I(bits), .. }, a) => {
            format!("({} as i{bits})", format_expr(a))
        }
        ExprX::Binary(BinaryOp::Arith(ArithOp::TruncatingDiv(..)), a, b) => {
            format!("rust_div({}, {})", format_expr(a), format_expr(b))
        }
        ExprX::Binary(BinaryOp::Arith(ArithOp::TruncatingMod(..)), a, b) => {
            format!("rust_rem({}, {})", format_expr(a), format_expr(b))
        }
        ExprX::Binary(op, a, b) => {
            let op = match op {
                BinaryOp::Arith(ArithOp::Add(_)) => "+",
                BinaryOp::Arith(ArithOp::Sub(_)) => "-",
                BinaryOp::Arith(ArithOp::Mul(_)) => "*",
                BinaryOp::Arith(ArithOp::EuclideanDiv(_)) => "/",
                BinaryOp::Arith(ArithOp::EuclideanMod(_)) => "%",
                BinaryOp::Eq(_) => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Inequality(InequalityOp::Le) => "<=",
                BinaryOp::Inequality(InequalityOp::Lt) => "<",
                BinaryOp::Inequality(InequalityOp::Ge) => ">=",
                BinaryOp::Inequality(InequalityOp::Gt) => ">",
                BinaryOp::BoolAndNoSC => "&",
                BinaryOp::BoolOrNoSC => "|",
                BinaryOp::BoolXor => "^",
                _ => unreachable!("only synthesized operators are printed"),
            };
            format!("({} {op} {})", format_expr(a), format_expr(b))
        }
        ExprX::If(c, a, Some(b)) => {
            format!("if {} {{ {} }} else {{ {} }}", format_expr(c), format_expr(a), format_expr(b))
        }
        _ => unreachable!("only synthesized expressions are printed"),
    }
}

fn format_place(p: &Place) -> String {
    match &p.x {
        PlaceX::Local(v) => v.0.to_string(),
        PlaceX::Temporary(e) => format_expr(e),
        PlaceX::DerefMut(p) => format_place(p),
        _ => format!("{:?}", p.x),
    }
}

#[cfg(test)]
mod integer_tests;
#[cfg(test)]
mod platform_tests;
#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod coverage_tests;
