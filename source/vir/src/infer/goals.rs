//! Bounded, demand-driven proof proposals for ordinary callers of exact inference.
//!
//! Substitution is untrusted: we add only a universally CHECKED conditional
//! theorem, never a rewritten caller assertion or an assumed callee result.
//! The original assertion and every executable safety check remain in place.
//! The existing BV backend checks the identical theorem used by the normal
//! query. Failure of an auxiliary query fails verification, just like an
//! explicit `assert(...) by(bit_vector)`.
use super::*;
use crate::sst::{self, Exp, ExpX, FuncCheckSst, Stm, StmX};
use num_traits::ToPrimitive;
use std::collections::HashSet;

const EXPANSION: usize = 8192;
const DEPTH: usize = 64;
const REQUESTS: usize = 8;
const VISITS: usize = 8192;
const WORK: usize = EXPANSION * REQUESTS * 2;

#[derive(Clone)]
enum Definition {
    Value(Exp),
    // Keep only the existing summary and actual arguments until a goal needs
    // this result. No summary expansion is performed merely because of a call.
    Call(Function, sst::Exps),
}

fn binary(op: sst::BinaryOp, a: Exp, b: Exp) -> Exp {
    SpannedTyped::new(&a.span, &bool_typ(), ExpX::Binary(op, a.clone(), b))
}

fn unboxed(mut typ: &Typ) -> &Typ {
    while let TypX::Boxed(inner) = &**typ {
        typ = inner;
    }
    typ
}

fn array_projection(expression: &Exp) -> Option<(&Exp, &Exp)> {
    let ExpX::Call(sst::CallFun::Fun(name, None), types, arguments) = &expression.x else {
        return None;
    };
    if !models::path_is(&name.path, CrateId::Vstd, &["seq", "Seq", "index"])
        || types.len() != 1
        || arguments.len() != 2
    {
        return None;
    }
    let ExpX::Call(
        sst::CallFun::Fun(method, Some((implementation, _))),
        view_types,
        view_arguments,
    ) = &arguments[0].x
    else {
        return None;
    };
    if !models::path_is(&method.path, CrateId::Vstd, &["view", "View", "view"])
        || implementation.path.krate != CrateId::Vstd
        || implementation.path.segments.len() != 3
        || implementation.path.segments[0].as_str() != "array"
        || !implementation.path.segments[1].starts_with("impl")
        || implementation.path.segments[2].as_str() != "view"
        || view_types.len() != 1
        || view_arguments.len() != 1
    {
        return None;
    }
    let array = &view_arguments[0];
    let (element, _) = arrays::shape(unboxed(&array.typ))?;
    if !crate::ast_util::types_equal(&element, &types[0])
        || !crate::ast_util::types_equal(&element, unboxed(&expression.typ))
        || !crate::ast_util::types_equal(unboxed(&array.typ), &view_types[0])
        || !matches!(&**unboxed(&arguments[1].typ), TypX::Int(IntRange::Int))
    {
        return None;
    }
    Some((array, &arguments[1]))
}

struct Expression<'a> {
    definitions: &'a HashMap<VarIdent, Definition>,
    remaining: usize,
    inferred: bool,
    bitwise: bool,
    premises: Vec<Exp>,
    resolved: HashMap<VarIdent, Exp>,
    inputs: HashSet<VarIdent>,
}

impl Expression<'_> {
    fn number(e: &Exp, n: BigInt) -> Exp {
        SpannedTyped::new(&e.span, &int_typ(), ExpX::Const(Constant::Int(n)))
    }

    fn resolve(&mut self, expression: &Exp, depth: usize) -> Option<Exp> {
        self.remaining = self.remaining.checked_sub(1)?;
        let next = depth.checked_sub(1)?;
        if let Some((array, index)) = array_projection(expression) {
            return self.resolve(
                &expression.new_x(ExpX::Binary(
                    sst::BinaryOp::Index(ArrayKind::Array),
                    array.clone(),
                    index.clone(),
                )),
                next,
            );
        }
        match &expression.x {
            ExpX::Var(name) => {
                if let Some(value) = self.resolved.get(name) {
                    return Some(value.clone());
                }
                let value = match self.definitions.get(name) {
                    Some(Definition::Value(value)) => value.clone(),
                    Some(Definition::Call(function, args)) => {
                        let ExprX::Binary(BinaryOp::Eq(Mode::Spec), returned, value) =
                            &function.x.ensure.0.first()?.x
                        else {
                            return None;
                        };
                        if !matches!(&returned.x, ExprX::Var(result) if result == &function.x.inner_ret.x.name)
                            || function.x.params.len() != args.len()
                        {
                            return None;
                        }
                        let variables = function
                            .x
                            .params
                            .iter()
                            .zip(args.iter())
                            .map(|(parameter, argument)| {
                                (parameter.x.name.clone(), argument.clone())
                            })
                            .collect();
                        self.inferred = true;
                        super::bitvector_hints::summary_exp(value, &variables, &mut self.remaining)?
                    }
                    None => return Some(expression.clone()),
                };
                let value = self.resolve(&value, next)?;
                self.resolved.insert(name.clone(), value.clone());
                Some(value)
            }
            ExpX::UnaryOpr(UnaryOpr::Box(_) | UnaryOpr::Unbox(_), value) => {
                self.resolve(value, next)
            }
            ExpX::UnaryOpr(UnaryOpr::Field(field), value) => {
                let value = self.resolve(value, next)?;
                let ExpX::Ctor(datatype, variant, fields) = &value.x else { return None };
                if datatype != &field.datatype || variant != &field.variant || field.get_variant {
                    return None;
                }
                self.resolve(&fields.iter().find(|binding| binding.name == field.field)?.a, next)
            }
            ExpX::Binary(sst::BinaryOp::Index(ArrayKind::Array), array, index) => {
                let array = self.resolve(array, next)?;
                let index = self.resolve(index, next)?;
                let ExpX::ArrayLiteral(elements) = &array.x else { return None };
                let ExpX::Const(Constant::Int(index)) = &index.x else { return None };
                self.resolve(elements.get(index.to_usize()?)?, next)
            }
            _ => Some(expression.clone()),
        }
    }

    // Return a conservative bit-width estimate as well as the proposed term.
    // This bounds bit-blasting before asking the existing converter to run.
    fn expand(&mut self, e: &Exp, depth: usize) -> Option<(Exp, u64)> {
        self.remaining = self.remaining.checked_sub(1)?;
        if depth == 0 {
            return None;
        }
        let next = depth - 1;
        if let ExpX::VarAt(name, VarAt::Pre) = &e.x {
            return self.expand(&e.new_x(ExpX::Var(name.clone())), next);
        }
        let resolved = self.resolve(e, depth)?;
        if !Arc::ptr_eq(e, &resolved) {
            return self.expand(&resolved, next);
        }
        let (x, width) = match &e.x {
            ExpX::Var(v) => {
                let width = match &*e.typ {
                    TypX::Bool => 1,
                    TypX::Int(r @ (IntRange::U(_) | IntRange::I(_))) => {
                        let (lo, hi) = arithmetic::bounds_at(*r, 64)?;
                        if self.inputs.insert(v.clone()) {
                            self.premises.push(binary(
                                sst::BinaryOp::And,
                                binary(
                                    sst::BinaryOp::Inequality(InequalityOp::Le),
                                    Self::number(e, lo),
                                    e.clone(),
                                ),
                                binary(
                                    sst::BinaryOp::Inequality(InequalityOp::Le),
                                    e.clone(),
                                    Self::number(e, hi),
                                ),
                            ));
                        }
                        arithmetic::bounds_at(*r, 64)?.1.bits() + 1
                    }
                    _ => return None,
                };
                (e.x.clone(), width)
            }
            ExpX::Const(Constant::Bool(_)) => (e.x.clone(), 1),
            ExpX::Const(Constant::Int(n)) => (e.x.clone(), n.bits() + 1),
            ExpX::Unary(op @ (UnaryOp::Not | UnaryOp::BitNot(_) | UnaryOp::Clip { .. }), a) => {
                let (a, w) = self.expand(a, next)?;
                (ExpX::Unary(*op, a), w.max(128))
            }
            ExpX::UnaryOpr(UnaryOpr::Box(_) | UnaryOpr::Unbox(_), a) => {
                return self.expand(a, next);
            }
            ExpX::Binary(op, a, b) => {
                if matches!(op, sst::BinaryOp::Eq | sst::BinaryOp::Ne) {
                    let left = self.resolve(a, next)?;
                    let right = self.resolve(b, next)?;
                    if let (ExpX::ArrayLiteral(left), ExpX::ArrayLiteral(right)) =
                        (&left.x, &right.x)
                    {
                        if left.len() != right.len() || left.len() > arrays::MAX_LEN {
                            return None;
                        }
                        let mut equality = e.new_x(ExpX::Const(Constant::Bool(true)));
                        for (left, right) in left.iter().zip(right.iter()) {
                            let (element, _) = self.expand(
                                &binary(sst::BinaryOp::Eq, left.clone(), right.clone()),
                                next,
                            )?;
                            equality = binary(sst::BinaryOp::And, equality, element);
                        }
                        return Some((
                            if *op == sst::BinaryOp::Eq {
                                equality
                            } else {
                                e.new_x(ExpX::Unary(UnaryOp::Not, equality))
                            },
                            1,
                        ));
                    }
                }
                let (a, aw) = self.expand(a, next)?;
                let (b, bw) = self.expand(b, next)?;
                let width = match op {
                    sst::BinaryOp::Arith(sst::ArithOp::Mul) => aw + bw,
                    sst::BinaryOp::Arith(sst::ArithOp::Add | sst::ArithOp::Sub) => aw.max(bw) + 1,
                    sst::BinaryOp::Arith(
                        sst::ArithOp::EuclideanDiv | sst::ArithOp::EuclideanMod,
                    ) => {
                        self.premises.push(binary(
                            sst::BinaryOp::Ne,
                            b.clone(),
                            Self::number(&b, 0.into()),
                        ));
                        aw.max(bw) + 1
                    }
                    sst::BinaryOp::Bitwise(
                        BitwiseOp::BitAnd
                        | BitwiseOp::BitOr
                        | BitwiseOp::BitXor
                        | BitwiseOp::Shl(..)
                        | BitwiseOp::Shr,
                    ) => {
                        self.bitwise = true;
                        aw.max(bw)
                    }
                    sst::BinaryOp::And
                    | sst::BinaryOp::Or
                    | sst::BinaryOp::Xor
                    | sst::BinaryOp::Implies
                    | sst::BinaryOp::Eq
                    | sst::BinaryOp::Ne
                    | sst::BinaryOp::Inequality(_) => 1,
                    _ => return None,
                };
                (ExpX::Binary(*op, a, b), width)
            }
            ExpX::If(c, a, b) => {
                let (c, _) = self.expand(c, next)?;
                let (a, aw) = self.expand(a, next)?;
                let (b, bw) = self.expand(b, next)?;
                (ExpX::If(c, a, b), aw.max(bw))
            }
            _ => return None,
        };
        if width > 4096 {
            return None;
        }
        Some((e.new_x(x), width))
    }
}

struct Requests<'a> {
    callees: &'a HashMap<Fun, Function>,
    post_condition: &'a sst::PostConditionSst,
    definitions: HashMap<VarIdent, Definition>,
    assumptions: Vec<Exp>,
    remaining: usize,
    visits: usize,
    work: usize,
}

impl Requests<'_> {
    fn array_projection_call(&mut self, statement: &Stm) -> bool {
        let StmX::Call {
            fun: sst::CallTarget::Fun(name),
            resolved_method: Some((implementation, _)),
            mode: Mode::Spec,
            args,
            dest: Some(destination),
            body: None,
            ..
        } = &statement.x
        else {
            return false;
        };
        if !models::path_is(
            &name.path,
            CrateId::Vstd,
            &["array", "ArrayAdditionalSpecFns", "spec_index"],
        ) || args.len() != 2
            || !destination.is_init
        {
            return false;
        }
        let Some((element, _)) = arrays::shape(unboxed(&args[0].typ)) else { return false };
        if !crate::ast_util::types_equal(&element, unboxed(&destination.dest.typ))
            || !matches!(&**unboxed(&args[1].typ), TypX::Int(IntRange::Int))
        {
            return false;
        }
        let Some(function) = self.callees.get(implementation) else { return false };
        let FunctionKind::TraitMethodImpl { method, trait_path, .. } = &function.x.kind else {
            return false;
        };
        if method != name
            || !models::path_is(trait_path, CrateId::Vstd, &["array", "ArrayAdditionalSpecFns"])
            || function.x.mode != Mode::Spec
            || function.x.proxy.is_some()
            || function.x.attrs.infer
            || function.x.attrs.is_unsafe
            || function.x.params.len() != 2
            || function.x.params.iter().any(|parameter| parameter.x.user_mut)
        {
            return false;
        }
        let (ExpX::Var(name) | ExpX::VarLoc(name)) = &destination.dest.x else { return false };
        self.definitions.remove(name);
        true
    }

    fn array_equality(&mut self, statement: &Stm) -> bool {
        let StmX::Call {
            fun: sst::CallTarget::Fun(name),
            resolved_method: Some((implementation, _)),
            mode: Mode::Exec,
            args,
            dest: Some(destination),
            body: None,
            ..
        } = &statement.x
        else {
            return false;
        };
        if !models::path_is(&name.path, CrateId::Core, &["cmp", "PartialEq", "eq"])
            || args.len() != 2
        {
            return false;
        }
        let unboxed = |typ: &Typ| {
            let mut typ = typ.clone();
            while let TypX::Boxed(inner) | TypX::Decorate(TypDecoration::Ref, None, inner) = &*typ {
                typ = inner.clone();
            }
            typ
        };
        let left = unboxed(&args[0].typ);
        let right = unboxed(&args[1].typ);
        if !arrays::compatible(&left, &right) {
            return false;
        }
        let Some((element, _)) = arrays::shape(&left) else { return false };
        if !matches!(&*element, TypX::Bool | TypX::Int(IntRange::U(_) | IntRange::I(_))) {
            return false;
        }
        let Some(function) = self.callees.get(implementation) else { return false };
        if !function
            .x
            .proxy
            .as_ref()
            .is_some_and(|proxy| models::spec_proxy(&proxy.x, &["std_specs", "array"], "_eq"))
            || function.x.mode != Mode::Exec
            || function.x.body.is_some()
            || function.x.params.len() != 2
            || !function.x.require.is_empty()
            || function.x.params.iter().any(|parameter| parameter.x.user_mut)
        {
            return false;
        }
        let (ExpX::Var(name) | ExpX::VarLoc(name)) = &destination.dest.x else { return false };
        self.definitions.insert(
            name.clone(),
            Definition::Value(binary(sst::BinaryOp::Eq, args[0].clone(), args[1].clone())),
        );
        true
    }

    fn request(&mut self, goal: &Exp, inferred_requirement: bool) -> Option<Exp> {
        if self.remaining == 0 || self.work == 0 {
            return None;
        }
        let budget = EXPANSION.min(self.work);
        let mut e = Expression {
            definitions: &self.definitions,
            remaining: budget,
            inferred: inferred_requirement,
            bitwise: false,
            premises: vec![],
            resolved: HashMap::new(),
            inputs: HashSet::new(),
        };
        let conclusion = e.expand(goal, DEPTH);
        self.work -= budget - e.remaining;
        let (conclusion, _) = conclusion?;
        if !e.inferred || !e.bitwise {
            return None;
        }
        let before = e.remaining;
        for assumption in &self.assumptions {
            // Unsupported assumptions are omitted, never guessed. Losing one
            // can make this tactic incomplete, but cannot justify a false fact.
            if let Some((assumption, _)) = e.expand(assumption, DEPTH) {
                e.premises.push(assumption);
            }
        }
        self.work -= before - e.remaining;
        let mut fact = conclusion;
        for premise in e.premises.into_iter().rev() {
            fact = binary(sst::BinaryOp::Implies, premise, fact);
        }
        self.remaining -= 1;
        // A failed specialized proof should identify the requested assertion,
        // not a parameter inside an expanded callee summary.
        Some(SpannedTyped::new(&goal.span, &bool_typ(), fact.x.clone()))
    }

    fn stm(&mut self, stm: &Stm) -> Result<Stm, VirErr> {
        if self.visits == 0 {
            return Ok(stm.clone());
        }
        self.visits -= 1;
        match &stm.x {
            StmX::Block(stms) => Ok(stm.new_x(StmX::Block(Arc::new(
                stms.iter().map(|s| self.stm(s)).collect::<Result<_, _>>()?,
            )))),
            StmX::Assert(_, _, goal) => {
                if let Some(fact) = self.request(goal, false) {
                    return Ok(stm.new_x(StmX::Block(Arc::new(vec![
                        super::bitvector_hints::checked(fact)?,
                        stm.clone(),
                    ]))));
                }
                Ok(stm.clone())
            }
            StmX::Assume(e) => {
                if self.assumptions.len() < 32 {
                    self.assumptions.push(e.clone());
                }
                Ok(stm.clone())
            }
            StmX::Assign { lhs, rhs } => {
                if let ExpX::Var(v) | ExpX::VarLoc(v) = &lhs.dest.x {
                    self.definitions.insert(v.clone(), Definition::Value(rhs.clone()));
                } else {
                    self.definitions.clear();
                }
                Ok(stm.clone())
            }
            StmX::Call { fun: sst::CallTarget::Fun(name), args, dest: Some(dest), .. } => {
                if self.array_equality(stm) || self.array_projection_call(stm) {
                    return Ok(stm.clone());
                }
                let callee = self
                    .callees
                    .get(name)
                    .filter(|f| f.x.attrs.infer && !f.x.attrs.infer_allow_approx);
                if let (Some(callee), ExpX::Var(v) | ExpX::VarLoc(v)) = (callee, &dest.dest.x) {
                    let variables = callee
                        .x
                        .params
                        .iter()
                        .zip(args.iter())
                        .map(|(parameter, argument)| (parameter.x.name.clone(), argument.clone()))
                        .collect();
                    let mut hints = Vec::new();
                    for requirement in callee.x.require.iter().take(REQUESTS) {
                        if self.remaining == 0 || self.work == 0 {
                            break;
                        }
                        let mut budget = EXPANSION.min(self.work);
                        let before = budget;
                        let requirement = super::bitvector_hints::summary_exp(
                            requirement,
                            &variables,
                            &mut budget,
                        );
                        self.work -= before - budget;
                        if let Some(requirement) = requirement
                            && let Some(fact) = self.request(&requirement, true)
                        {
                            hints.push(super::bitvector_hints::checked(fact)?);
                        }
                    }
                    self.definitions
                        .insert(v.clone(), Definition::Call(callee.clone(), args.clone()));
                    if !hints.is_empty() {
                        hints.push(stm.clone());
                        return Ok(stm.new_x(StmX::Block(Arc::new(hints))));
                    }
                } else {
                    self.definitions.clear();
                }
                Ok(stm.clone())
            }
            StmX::Return { ret_exp, .. } => {
                if let (Some(destination), Some(value)) = (&self.post_condition.dest, ret_exp) {
                    self.definitions.insert(destination.clone(), Definition::Value(value.clone()));
                }
                let mut hints = Vec::new();
                for goal in self.post_condition.ens_exps.iter() {
                    if let Some(fact) = self.request(goal, false) {
                        hints.push(super::bitvector_hints::checked(fact)?);
                    }
                }
                hints.push(stm.clone());
                Ok(stm.new_x(StmX::Block(Arc::new(hints))))
            }
            StmX::If(c, a, b) => {
                let definitions = self.definitions.clone();
                let assumptions = self.assumptions.clone();
                self.assumptions.push(c.clone());
                let a = self.stm(a)?;
                self.definitions = definitions;
                self.assumptions = assumptions.clone();
                self.assumptions.push(c.new_x(ExpX::Unary(UnaryOp::Not, c.clone())));
                let b = b.as_ref().map(|s| self.stm(s)).transpose()?;
                self.definitions.clear();
                self.assumptions = assumptions;
                Ok(stm.new_x(StmX::If(c.clone(), a, b)))
            }
            // Do not propagate guessed state through loops, proof scopes,
            // mutation-capable calls, or other unsupported control flow.
            _ => {
                self.definitions.clear();
                Ok(stm.clone())
            }
        }
    }
}

/// Bounded scalar BV obligations across exact calls, transparent records and
/// literal arrays, requested by assertions, callee requirements and returns.
/// This decorates ordinary callers only; precision queries are never inputs.
pub fn prepare(
    check: &FuncCheckSst,
    callees: &HashMap<Fun, Function>,
) -> Result<FuncCheckSst, VirErr> {
    if !matches!(check.post_condition.kind, sst::PostConditionKind::Ensures) {
        return Err(error(&check.body.span, "Verus Infer goals: expected an ordinary proof body"));
    }
    let mut requests = Requests {
        callees,
        post_condition: &check.post_condition,
        definitions: HashMap::new(),
        assumptions: check.reqs.iter().take(32).cloned().collect(),
        remaining: REQUESTS,
        visits: VISITS,
        work: WORK,
    };
    let mut result = check.clone();
    result.body = requests.stm(&check.body)?;
    super::packing_goals::decorate(&result, callees)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variable(name: &str) -> Exp {
        let input = crate::infer::tests::input(name);
        let ExprX::Var(name) = &input.x else { panic!() };
        SpannedTyped::new(&input.span, &input.typ, ExpX::Var(name.clone()))
    }

    fn expand(value: &Exp, definitions: &HashMap<VarIdent, Definition>, budget: usize) -> bool {
        Expression {
            definitions,
            remaining: budget,
            inferred: false,
            bitwise: false,
            premises: Vec::new(),
            resolved: HashMap::new(),
            inputs: HashSet::new(),
        }
        .expand(value, DEPTH)
        .is_some()
    }

    #[test]
    fn expansion_and_alias_cycles_stop_at_the_budget() {
        let value = variable("value");
        assert!(!expand(&value, &HashMap::new(), 1));
        let ExpX::Var(name) = &value.x else { panic!() };
        let cycle = HashMap::from([(name.clone(), Definition::Value(value.clone()))]);
        assert!(!expand(&value, &cycle, EXPANSION));
        let mut nested = value.clone();
        for _ in 0..DEPTH + 1 {
            nested = nested.new_x(ExpX::Unary(
                UnaryOp::Clip { range: IntRange::U(8), truncate: true },
                nested.clone(),
            ));
        }
        assert!(!expand(&nested, &HashMap::new(), EXPANSION));
    }

    #[test]
    fn only_literal_in_bounds_array_projections_are_eliminated() {
        let value = variable("value");
        let array = SpannedTyped::new(
            &value.span,
            &arrays::typ(value.typ.clone(), 1).unwrap(),
            ExpX::ArrayLiteral(Arc::new(vec![value.clone()])),
        );
        let index = |position: usize| {
            value.new_x(ExpX::Binary(
                sst::BinaryOp::Index(ArrayKind::Array),
                array.clone(),
                Expression::number(&value, position.into()),
            ))
        };
        assert!(expand(&index(0), &HashMap::new(), EXPANSION));
        assert!(!expand(&index(1), &HashMap::new(), EXPANSION));
        assert!(!expand(
            &value.new_x(ExpX::Binary(
                sst::BinaryOp::Index(ArrayKind::Array),
                array,
                value.clone()
            )),
            &HashMap::new(),
            EXPANSION,
        ));
    }

    #[test]
    fn canonical_spec_array_projection_checks_identity_types_and_bounds() {
        let value = variable("value");
        let array_type = arrays::typ(value.typ.clone(), 1).unwrap();
        let array = SpannedTyped::new(
            &value.span,
            &array_type,
            ExpX::ArrayLiteral(Arc::new(vec![value.clone()])),
        );
        let function = |parts: &[&str]| {
            Arc::new(FunX {
                path: Arc::new(PathX {
                    krate: CrateId::Vstd,
                    segments: Arc::new(parts.iter().map(|part| Arc::new((*part).into())).collect()),
                }),
            })
        };
        let projection = |module: &str, index: usize, result_type: Typ| {
            let view = SpannedTyped::new(
                &value.span,
                &Arc::new(TypX::Datatype(
                    Dt::Path(function(&["seq", "Seq"]).path.clone()),
                    Arc::new(vec![value.typ.clone()]),
                    Arc::new(vec![]),
                )),
                ExpX::Call(
                    sst::CallFun::Fun(
                        function(&["view", "View", "view"]),
                        Some((function(&[module, "impl&%0", "view"]), Arc::new(vec![]))),
                    ),
                    Arc::new(vec![array_type.clone()]),
                    Arc::new(vec![array.clone()]),
                ),
            );
            SpannedTyped::new(
                &value.span,
                &result_type,
                ExpX::Call(
                    sst::CallFun::Fun(function(&["seq", "Seq", "index"]), None),
                    Arc::new(vec![value.typ.clone()]),
                    Arc::new(vec![view, Expression::number(&value, index.into())]),
                ),
            )
        };
        let actual = projection("array", 0, value.typ.clone());
        assert!(array_projection(&actual).is_some());
        assert!(expand(&actual, &HashMap::new(), EXPANSION));
        assert!(array_projection(&projection("local", 0, value.typ.clone())).is_none());
        assert!(array_projection(&projection("array", 0, bool_typ())).is_none());
        assert!(!expand(&projection("array", 1, value.typ.clone()), &HashMap::new(), EXPANSION));
    }

    #[test]
    fn rejected_requests_consume_the_shared_work_budget() {
        let callees = HashMap::new();
        let post_condition = sst::PostConditionSst {
            dest: None,
            ens_exps: Arc::new(Vec::new()),
            ens_spec_precondition_stms: Arc::new(Vec::new()),
            kind: sst::PostConditionKind::Ensures,
        };
        let mut requests = Requests {
            callees: &callees,
            post_condition: &post_condition,
            definitions: HashMap::new(),
            assumptions: Vec::new(),
            remaining: REQUESTS,
            visits: VISITS,
            work: 1,
        };
        assert!(requests.request(&variable("value"), true).is_none());
        assert_eq!(requests.work, 0);
        assert_eq!(requests.remaining, REQUESTS);
        assert!(requests.request(&variable("value"), true).is_none());
    }
}
