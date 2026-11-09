use super::*;
use num_traits::Zero;

fn value(expression: &Exp, remaining: &mut usize, depth: usize) -> Option<BigInt> {
    *remaining = remaining.checked_sub(1)?;
    let next = depth.checked_sub(1)?;
    let result = match &expression.x {
        ExpX::Const(Constant::Int(number)) => number.clone(),
        ExpX::Unary(UnaryOp::Clip { range, truncate }, operand) => {
            let number = value(operand, remaining, next)?;
            match range {
                IntRange::Int => number,
                IntRange::Nat if number >= 0.into() => number,
                IntRange::U(8 | 16 | 32 | 64 | 128) | IntRange::I(8 | 16 | 32 | 64 | 128) => {
                    let (low, high) = arithmetic::bounds_at(*range, 64)?;
                    if !truncate {
                        if number < low || number > high {
                            return None;
                        }
                        number
                    } else {
                        let modulus = high - &low + 1;
                        ((number - &low) % &modulus + &modulus) % &modulus + low
                    }
                }
                _ => return None,
            }
        }
        ExpX::Binary(operation, left, right) => {
            let left = value(left, remaining, next)?;
            let right = value(right, remaining, next)?;
            match operation {
                sst::BinaryOp::Arith(sst::ArithOp::Add) => left + right,
                sst::BinaryOp::Arith(sst::ArithOp::Sub) => left - right,
                sst::BinaryOp::Arith(sst::ArithOp::Mul) => left * right,
                sst::BinaryOp::Bitwise(BitwiseOp::BitOr) => left | right,
                sst::BinaryOp::Bitwise(BitwiseOp::BitAnd) => left & right,
                sst::BinaryOp::Bitwise(BitwiseOp::BitXor) => left ^ right,
                sst::BinaryOp::Bitwise(BitwiseOp::Shr) => {
                    left >> right.to_usize().filter(|shift| *shift < 128)?
                }
                sst::BinaryOp::Bitwise(BitwiseOp::Shl(
                    IntegerTypeBitwidth::Width(width @ (8 | 16 | 32 | 64 | 128)),
                    false,
                )) => {
                    let shift = right.to_usize().filter(|shift| *shift < *width as usize)?;
                    (left << shift) & ((BigInt::one() << *width) - 1)
                }
                _ => return None,
            }
        }
        _ => return None,
    };
    (result.bits() <= 4096).then_some(result)
}

impl Requests<'_> {
    fn constant_request(
        &mut self,
        expression: &Expr,
        variables: &HashMap<VarIdent, Exp>,
    ) -> Option<Exp> {
        if self.remaining == 0 || self.work == 0 {
            return None;
        }
        let budget = EXPANSION.min(self.work);
        let mut remaining = budget;
        let candidate =
            super::super::bitvector_hints::summary_exp(expression, variables, &mut remaining);
        self.work -= budget - remaining;
        let candidate = candidate?;
        let mut expansion = Expression {
            callees: self.callees,
            definitions: &self.definitions,
            remaining,
            inferred: false,
            bitwise: false,
            premises: Vec::new(),
            resolved: HashMap::new(),
            inputs: HashSet::new(),
        };
        let conclusion = expansion.expand(&candidate, DEPTH);
        self.work -= remaining - expansion.remaining;
        let (conclusion, _) = conclusion?;
        if !expansion.bitwise || !expansion.inputs.is_empty() {
            return None;
        }
        let mut remaining = expansion.remaining;
        let before = remaining;
        let result = value(&conclusion, &mut remaining, DEPTH);
        self.work -= before - remaining;
        let result = result?;
        self.remaining -= 1;
        Some(binary(sst::BinaryOp::Eq, conclusion.clone(), Expression::number(&conclusion, result)))
    }

    pub(super) fn constant_requests(
        &mut self,
        name: &Fun,
        arguments: &sst::Exps,
    ) -> Result<Vec<Stm>, VirErr> {
        let Some(function) = self.callees.get(name).cloned() else { return Ok(Vec::new()) };
        if function.x.attrs.infer
            || function.x.params.len() != arguments.len()
            || function.x.params.iter().any(|parameter| parameter.x.user_mut)
        {
            return Ok(Vec::new());
        }
        let variables = function
            .x
            .params
            .iter()
            .zip(arguments.iter())
            .map(|(parameter, argument)| (parameter.x.name.clone(), argument.clone()))
            .collect();
        let mut facts = Vec::new();
        for formula in function.x.require.iter().chain(function.x.ensure.0.iter()) {
            crate::ast_visitor::expr_visitor_walk(formula, &mut |expression| {
                if self.visits == 0 || self.work == 0 || self.remaining == 0 {
                    return crate::ast_visitor::VisitorControlFlow::<()>::Return;
                }
                self.visits -= 1;
                if matches!(
                    &expression.x,
                    ExprX::Call { .. } | ExprX::Binary(BinaryOp::Bitwise(..), ..)
                ) {
                    if let Some(fact) = self.constant_request(expression, &variables) {
                        facts.push(fact);
                        return crate::ast_visitor::VisitorControlFlow::<()>::Return;
                    }
                }
                crate::ast_visitor::VisitorControlFlow::<()>::Recurse
            });
        }
        facts.into_iter().map(super::super::bitvector_hints::checked).collect()
    }
}

const LOCAL_PAIRS: usize = 8;
const LOCAL_WORK: usize = 8192;
const LOCAL_DEPTH: usize = 64;
const LOCAL_ENTRIES: usize = 32;
const LOCAL_ALIASES: usize = 8;

#[derive(Clone)]
struct LocalStamp {
    name: VarIdent,
    typ: Typ,
    generation: u64,
}

#[derive(Clone)]
struct LocalEquality {
    count: LocalStamp,
    value: BigInt,
    dependencies: Vec<LocalStamp>,
}

#[derive(Clone)]
struct LocalBinding {
    stamp: LocalStamp,
    entry: bool,
    fact: Option<BigInt>,
    alias: Option<LocalEquality>,
}

#[derive(Clone)]
struct LocalShiftKey {
    count: LocalStamp,
    width: u32,
    left: BigInt,
    amount: BigInt,
}

#[derive(Clone, Default)]
struct LocalState {
    bindings: Vec<LocalBinding>,
    seen: Vec<LocalShiftKey>,
}

struct LocalShifts {
    work: usize,
    pairs: usize,
    generation: u64,
    exhausted: bool,
}

impl LocalShifts {
    fn spend(&mut self, amount: usize) -> bool {
        if let Some(remaining) = self.work.checked_sub(amount) {
            self.work = remaining;
            true
        } else {
            self.exhausted = true;
            false
        }
    }

    fn visit(&mut self, depth: usize) -> bool {
        if depth > LOCAL_DEPTH {
            self.exhausted = true;
            return false;
        }
        self.spend(1)
    }

    fn same_type(left: &Typ, right: &Typ) -> bool {
        match (&**left, &**right) {
            (TypX::Bool, TypX::Bool) => true,
            (TypX::Int(left), TypX::Int(right)) => left == right,
            _ => false,
        }
    }

    fn unsigned(typ: &Typ) -> Option<IntRange> {
        match &**typ {
            TypX::Int(range @ (IntRange::U(8 | 16 | 32 | 64 | 128) | IntRange::USize)) => {
                Some(*range)
            }
            _ => None,
        }
    }

    fn find(&mut self, state: &LocalState, name: &VarIdent) -> Option<usize> {
        for (index, binding) in state.bindings.iter().enumerate() {
            if !self.spend(1) {
                return None;
            }
            if &binding.stamp.name == name {
                return Some(index);
            }
        }
        None
    }

    fn state_cost(&mut self, state: &LocalState) -> Option<usize> {
        let mut entries = state.seen.len();
        for binding in &state.bindings {
            if !self.spend(1) {
                return None;
            }
            entries += 1 + usize::from(binding.fact.is_some());
            if let Some(alias) = &binding.alias {
                entries += 1 + alias.dependencies.len();
            }
        }
        if entries > LOCAL_ENTRIES || !self.spend(entries) {
            self.exhausted = true;
            return None;
        }
        Some(entries)
    }

    fn copy_state(&mut self, state: &LocalState) -> Option<LocalState> {
        self.state_cost(state)?;
        Some(state.clone())
    }

    fn binding(&mut self, state: &mut LocalState, expression: &Exp) -> Option<LocalStamp> {
        let ExpX::Var(name) = &expression.x else { return None };
        if let Some(index) = self.find(state, name) {
            let stamp = &state.bindings[index].stamp;
            return Self::same_type(&stamp.typ, &expression.typ).then(|| stamp.clone());
        }
        self.generation += 1;
        let stamp = LocalStamp {
            name: name.clone(),
            typ: expression.typ.clone(),
            generation: self.generation,
        };
        state.bindings.push(LocalBinding {
            stamp: stamp.clone(),
            entry: false,
            fact: None,
            alias: None,
        });
        self.state_cost(state)?;
        Some(stamp)
    }

    fn current(&mut self, state: &LocalState, stamp: &LocalStamp) -> bool {
        self.find(state, &stamp.name).is_some_and(|index| {
            let current = &state.bindings[index].stamp;
            current.generation == stamp.generation && Self::same_type(&current.typ, &stamp.typ)
        })
    }

    fn equality(
        &mut self,
        state: &mut LocalState,
        expression: &Exp,
        entry: bool,
        depth: usize,
    ) -> Option<LocalEquality> {
        if !self.visit(depth) {
            return None;
        }
        match &expression.x {
            ExpX::UnaryOpr(UnaryOpr::ProofNote(_), operand) => {
                self.equality(state, operand, entry, depth + 1)
            }
            ExpX::Var(name) if matches!(&*expression.typ, TypX::Bool) && !entry => {
                let index = self.find(state, name)?;
                if !self.spend(1) {
                    return None;
                }
                let alias = state.bindings[index].alias.as_ref()?;
                if alias.dependencies.len() >= LOCAL_ALIASES {
                    return None;
                }
                if !self.spend(alias.dependencies.len() + 1) {
                    return None;
                }
                let mut alias = alias.clone();
                if !self.current(state, &alias.count)
                    || !alias.dependencies.iter().all(|stamp| self.current(state, stamp))
                {
                    return None;
                }
                alias.dependencies.push(state.bindings[index].stamp.clone());
                Some(alias)
            }
            ExpX::Binary(sst::BinaryOp::Eq, left, right) => {
                if !self.spend(2) {
                    return None;
                }
                let (count, literal) =
                    if matches!(&left.x, ExpX::Const(_)) { (right, left) } else { (left, right) };
                let range = Self::unsigned(&count.typ)?;
                let ExpX::Const(Constant::Int(value)) = &literal.x else { return None };
                if value < &BigInt::zero() || value.bits() > 128 || value > &127.into() {
                    return None;
                }
                let bits = match range {
                    IntRange::U(bits) => bits,
                    IntRange::USize => 32,
                    _ => return None,
                };
                if value >= &(BigInt::one() << bits) {
                    return None;
                }
                let stamp = if entry {
                    let name = match &count.x {
                        ExpX::Var(name) | ExpX::VarAt(name, VarAt::Pre) => name,
                        _ => return None,
                    };
                    let index = self.find(state, name)?;
                    let binding = &state.bindings[index];
                    if !binding.entry
                        || binding.stamp.generation != 0
                        || !Self::same_type(&binding.stamp.typ, &count.typ)
                    {
                        return None;
                    }
                    binding.stamp.clone()
                } else {
                    self.binding(state, count)?
                };
                Some(LocalEquality { count: stamp, value: value.clone(), dependencies: Vec::new() })
            }
            _ => None,
        }
    }

    fn establish(&mut self, state: &mut LocalState, expression: &Exp, entry: bool, depth: usize) {
        if !self.visit(depth) {
            return;
        }
        if let ExpX::Binary(sst::BinaryOp::And, left, right) = &expression.x {
            self.establish(state, left, entry, depth + 1);
            self.establish(state, right, entry, depth + 1);
        } else if let Some(equality) = self.equality(state, expression, entry, depth + 1) {
            if let Some(index) = self.find(state, &equality.count.name) {
                state.bindings[index].fact = Some(equality.value);
                self.state_cost(state);
            }
        }
    }

    fn invalidate(&mut self, state: &mut LocalState, name: &VarIdent) {
        for binding in &mut state.bindings {
            if !self.spend(1) {
                return;
            }
            if &binding.stamp.name == name {
                self.generation += 1;
                binding.stamp.generation = self.generation;
                binding.entry = false;
                binding.fact = None;
                binding.alias = None;
            } else if let Some(alias) = &binding.alias {
                if !self.spend(alias.dependencies.len() + 1) {
                    return;
                }
                if &alias.count.name == name
                    || alias.dependencies.iter().any(|stamp| &stamp.name == name)
                {
                    binding.alias = None;
                }
            }
        }
        if !self.spend(state.seen.len()) {
            return;
        }
        state.seen.retain(|key| &key.count.name != name);
    }

    fn scan(&mut self, expression: &Exp, candidates: &mut Vec<Exp>, depth: usize) -> bool {
        if !self.visit(depth) {
            return false;
        }
        match &expression.x {
            ExpX::Const(_) | ExpX::Var(_) | ExpX::VarAt(..) => true,
            ExpX::Unary(UnaryOp::Not | UnaryOp::BitNot(_) | UnaryOp::Clip { .. }, operand)
            | ExpX::UnaryOpr(
                UnaryOpr::HasType(_) | UnaryOpr::IntegerTypeBound(_) | UnaryOpr::ProofNote(_),
                operand,
            ) => {
                self.scan(operand, candidates, depth + 1)
            }
            ExpX::Binary(operation, left, right) => {
                if !self.scan(left, candidates, depth + 1)
                    || !self.scan(right, candidates, depth + 1)
                {
                    return false;
                }
                if matches!(operation, sst::BinaryOp::Bitwise(BitwiseOp::Shl(..))) {
                    if candidates.len() == LOCAL_PAIRS {
                        return false;
                    }
                    candidates.push(expression.clone());
                }
                true
            }
            _ => false,
        }
    }

    fn proposal(
        &mut self,
        state: &LocalState,
        expression: &Exp,
    ) -> Option<(LocalShiftKey, Vec<Exp>)> {
        if !self.spend(4) {
            return None;
        }
        let ExpX::Binary(
            sst::BinaryOp::Bitwise(BitwiseOp::Shl(
                IntegerTypeBitwidth::Width(width @ (8 | 16 | 32 | 64 | 128)),
                false,
            )),
            left,
            count,
        ) = &expression.x
        else {
            return None;
        };
        if Self::unsigned(&expression.typ) != Some(IntRange::U(*width))
            || Self::unsigned(&left.typ) != Some(IntRange::U(*width))
        {
            return None;
        }
        let ExpX::Const(Constant::Int(value)) = &left.x else { return None };
        if value < &BigInt::zero() || value.bits() > u64::from(*width) {
            return None;
        }
        let ExpX::Var(name) = &count.x else { return None };
        let range = Self::unsigned(&count.typ)?;
        let index = self.find(state, name)?;
        let binding = &state.bindings[index];
        if !Self::same_type(&binding.stamp.typ, &count.typ) {
            return None;
        }
        let amount = binding.fact.as_ref()?;
        let shift = amount.to_u32().filter(|shift| *shift < *width && *shift <= 127)?;
        for key in &state.seen {
            if !self.spend(1) {
                return None;
            }
            if key.count.name == *name
                && key.count.generation == binding.stamp.generation
                && Self::same_type(&key.count.typ, &count.typ)
                && key.width == *width
                && key.left == *value
                && key.amount == *amount
            {
                return None;
            }
        }
        let architectures: &[u32] = if range == IntRange::USize { &[32, 64] } else { &[0] };
        if architectures.len() > self.pairs || !self.spend(architectures.len() * 24) {
            return None;
        }
        let result = (value << shift) & ((BigInt::one() << *width) - 1);
        let conclusion =
            binary(sst::BinaryOp::Eq, expression.clone(), Expression::number(expression, result));
        let mut facts = Vec::new();
        for architecture in architectures {
            let bits = match range {
                IntRange::U(bits) => bits,
                IntRange::USize => *architecture,
                _ => return None,
            };
            let number = |value: BigInt| {
                SpannedTyped::new(&count.span, &int_typ(), ExpX::Const(Constant::Int(value)))
            };
            let within = binary(
                sst::BinaryOp::And,
                binary(
                    sst::BinaryOp::Inequality(InequalityOp::Le),
                    number(0.into()),
                    count.clone(),
                ),
                binary(
                    sst::BinaryOp::Inequality(InequalityOp::Le),
                    count.clone(),
                    number((BigInt::one() << bits) - 1),
                ),
            );
            let mut premise = binary(
                sst::BinaryOp::And,
                within,
                binary(sst::BinaryOp::Eq, count.clone(), number(amount.clone())),
            );
            if range == IntRange::USize {
                let architecture_bits = SpannedTyped::new(
                    &count.span,
                    &int_typ(),
                    ExpX::UnaryOpr(
                        UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits),
                        number(0.into()),
                    ),
                );
                premise = binary(
                    sst::BinaryOp::And,
                    binary(sst::BinaryOp::Eq, architecture_bits, number((*architecture).into())),
                    premise,
                );
            }
            facts.push(binary(sst::BinaryOp::Implies, premise, conclusion.clone()));
        }
        Some((
            LocalShiftKey {
                count: binding.stamp.clone(),
                width: *width,
                left: value.clone(),
                amount: amount.clone(),
            },
            facts,
        ))
    }

    fn hints(
        &mut self,
        state: &mut LocalState,
        expression: &Exp,
        depth: usize,
    ) -> Result<Vec<Stm>, VirErr> {
        let mut candidates = Vec::new();
        if !self.scan(expression, &mut candidates, depth) {
            return Ok(Vec::new());
        }
        let mut hints = Vec::new();
        for candidate in candidates {
            if let Some((key, facts)) = self.proposal(state, &candidate) {
                self.pairs -= facts.len();
                state.seen.push(key);
                if self.state_cost(state).is_none() {
                    return Ok(Vec::new());
                }
                for fact in facts {
                    hints.push(super::super::bitvector_hints::checked(fact)?);
                }
            }
        }
        Ok(hints)
    }

    fn stm(
        &mut self,
        state: &mut LocalState,
        statement: &Stm,
        depth: usize,
    ) -> Result<Stm, VirErr> {
        if !self.visit(depth) || self.exhausted {
            return Ok(statement.clone());
        }
        let hints = match &statement.x {
            StmX::Block(statements) => {
                let mut result = Vec::new();
                for child in statements.iter() {
                    if self.exhausted {
                        return Ok(statement.clone());
                    }
                    result.push(self.stm(state, child, depth + 1)?);
                }
                let mut retained = Vec::new();
                for binding in &state.bindings {
                    let mut local = false;
                    for child in statements.iter() {
                        if !self.spend(1) {
                            return Ok(statement.clone());
                        }
                        if let StmX::Assign { lhs, .. } = &child.x
                            && lhs.is_init
                            && let ExpX::Var(name) | ExpX::VarLoc(name) = &lhs.dest.x
                            && *name == binding.stamp.name
                        {
                            local = true;
                            break;
                        }
                    }
                    if !local {
                        if !self.spend(
                            1 + binding.alias.as_ref().map_or(0, |alias| alias.dependencies.len()),
                        ) {
                            return Ok(statement.clone());
                        }
                        retained.push(binding.clone());
                    }
                }
                state.bindings = retained;
                for index in 0..state.bindings.len() {
                    if let Some(alias) = &state.bindings[index].alias {
                        if !self.spend(alias.dependencies.len() + 1) {
                            return Ok(statement.clone());
                        }
                        let alias = alias.clone();
                        if !self.current(state, &alias.count)
                            || !alias.dependencies.iter().all(|stamp| self.current(state, stamp))
                        {
                            state.bindings[index].alias = None;
                        }
                    }
                }
                let mut seen = Vec::new();
                for key in &state.seen {
                    if self.current(state, &key.count) {
                        if !self.spend(1) {
                            return Ok(statement.clone());
                        }
                        seen.push(key.clone());
                    }
                }
                state.seen = seen;
                return Ok(statement.new_x(StmX::Block(Arc::new(result))));
            }
            StmX::If(condition, then_arm, else_arm) => {
                let Some(mut then_state) = self.copy_state(state) else {
                    return Ok(statement.clone());
                };
                let Some(mut else_state) = self.copy_state(state) else {
                    return Ok(statement.clone());
                };
                self.establish(&mut then_state, condition, false, depth + 1);
                let then_arm = self.stm(&mut then_state, then_arm, depth + 1)?;
                let else_arm = else_arm
                    .as_ref()
                    .map(|arm| self.stm(&mut else_state, arm, depth + 1))
                    .transpose()?;
                *state = LocalState::default();
                return Ok(statement.new_x(StmX::If(condition.clone(), then_arm, else_arm)));
            }
            StmX::Assert(_, _, expression) => {
                let hints = self.hints(state, expression, depth + 1)?;
                self.establish(state, expression, false, depth + 1);
                hints
            }
            StmX::Assign { lhs, rhs } => {
                let hints = self.hints(state, rhs, depth + 1)?;
                let alias = if lhs.is_init && matches!(&*lhs.dest.typ, TypX::Bool) {
                    self.equality(state, rhs, false, depth + 1)
                } else {
                    None
                };
                let name = match &lhs.dest.x {
                    ExpX::Var(name) | ExpX::VarLoc(name) => Some(name),
                    _ => None,
                };
                if let Some(name) = name {
                    self.invalidate(state, name);
                    if let Some(alias) = alias {
                        let destination = lhs.dest.new_x(ExpX::Var(name.clone()));
                        if let Some(stamp) = self.binding(state, &destination) {
                            if let Some(index) = self.find(state, &stamp.name) {
                                state.bindings[index].alias = Some(alias);
                                self.state_cost(state);
                            }
                        }
                    }
                } else {
                    *state = LocalState::default();
                }
                hints
            }
            StmX::Return { ret_exp: Some(expression), .. } => {
                let hints = self.hints(state, expression, depth + 1)?;
                *state = LocalState::default();
                hints
            }
            StmX::Assume(_)
            | StmX::Fuel(..)
            | StmX::RevealString(_)
            | StmX::RevealByteString(_) => Vec::new(),
            _ => {
                *state = LocalState::default();
                Vec::new()
            }
        };
        if hints.is_empty() || self.exhausted {
            Ok(statement.clone())
        } else {
            let mut statements = hints;
            statements.push(statement.clone());
            Ok(statement.new_x(StmX::Block(Arc::new(statements))))
        }
    }
}

pub(super) fn decorate_local_shifts(
    check: &FuncCheckSst,
    parameters: &sst::Pars,
) -> Result<FuncCheckSst, VirErr> {
    let mut decorator =
        LocalShifts { work: LOCAL_WORK, pairs: LOCAL_PAIRS, generation: 0, exhausted: false };
    let mut state = LocalState::default();
    for parameter in parameters.iter() {
        if !decorator.spend(1) {
            return Ok(check.clone());
        }
        if LocalShifts::unsigned(&parameter.x.typ).is_some() {
            state.bindings.push(LocalBinding {
                stamp: LocalStamp {
                    name: parameter.x.name.clone(),
                    typ: parameter.x.typ.clone(),
                    generation: 0,
                },
                entry: true,
                fact: None,
                alias: None,
            });
            decorator.state_cost(&state);
        }
    }
    for requirement in check.reqs.iter() {
        decorator.establish(&mut state, requirement, true, 0);
        if decorator.exhausted {
            return Ok(check.clone());
        }
    }
    let body = decorator.stm(&mut state, &check.body, 0)?;
    if decorator.exhausted {
        return Ok(check.clone());
    }
    let mut result = check.clone();
    result.body = body;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::def::Spanned;

    fn local_variable(name: &str, typ: Typ) -> Exp {
        let input = crate::infer::tests::input(name);
        let ExprX::Var(name) = &input.x else { panic!() };
        SpannedTyped::new(&input.span, &typ, ExpX::Var(name.clone()))
    }

    fn local_count(name: &str, range: IntRange) -> Exp {
        local_variable(name, Arc::new(TypX::Int(range)))
    }

    fn local_shift(count: &Exp, width: u32, left: u128) -> Exp {
        let operand = SpannedTyped::new(
            &count.span,
            &Arc::new(TypX::Int(IntRange::U(width))),
            ExpX::Const(Constant::Int(left.into())),
        );
        operand.new_x(ExpX::Binary(
            sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(width), false)),
            operand.clone(),
            count.clone(),
        ))
    }

    fn local_equality(count: &Exp, amount: u32) -> Exp {
        binary(sst::BinaryOp::Eq, count.clone(), Expression::number(count, amount.into()))
    }

    fn local_assert(expression: &Exp) -> Stm {
        Spanned::new(expression.span.clone(), StmX::Assert(None, None, expression.clone()))
    }

    fn local_assign(destination: &Exp, rhs: &Exp, is_init: bool) -> Stm {
        let ExpX::Var(name) = &destination.x else { panic!() };
        Spanned::new(
            destination.span.clone(),
            StmX::Assign {
                lhs: sst::Dest { dest: destination.new_x(ExpX::VarLoc(name.clone())), is_init },
                rhs: rhs.clone(),
            },
        )
    }

    fn local_block(statements: Vec<Stm>) -> Stm {
        Spanned::new(literal(0).span.clone(), StmX::Block(Arc::new(statements)))
    }

    fn local_decorator() -> LocalShifts {
        LocalShifts { work: LOCAL_WORK, pairs: LOCAL_PAIRS, generation: 0, exhausted: false }
    }

    fn local_use(count: &Exp) -> Stm {
        local_assert(&binary(sst::BinaryOp::Eq, local_shift(count, 64, 1), literal(1)))
    }

    fn local_pair_count(statement: &Stm) -> usize {
        match &statement.x {
            StmX::AssertBitVector { .. } => 1,
            StmX::Block(statements) => statements.iter().map(local_pair_count).sum(),
            StmX::If(_, then_arm, else_arm) => {
                local_pair_count(then_arm) + else_arm.as_ref().map_or(0, local_pair_count)
            }
            _ => 0,
        }
    }

    fn local_check(body: Stm, reqs: Vec<Exp>) -> FuncCheckSst {
        FuncCheckSst {
            reqs: Arc::new(reqs),
            return_opaque_type_eqs: Arc::new(Vec::new()),
            post_condition: Arc::new(sst::PostConditionSst {
                dest: None,
                ens_exps: Arc::new(Vec::new()),
                ens_spec_precondition_stms: Arc::new(Vec::new()),
                kind: sst::PostConditionKind::Ensures,
            }),
            unwind: sst::UnwindSst::MayUnwind,
            body,
            local_decls: Arc::new(Vec::new()),
            local_decls_decreases_init: Arc::new(Vec::new()),
            statics: Arc::new(Vec::new()),
        }
    }

    #[test]
    fn local_shift_keeps_complete_guards_original_expression_and_pair_identity() {
        for range in [IntRange::U(8), IntRange::U(128), IntRange::USize] {
            let count = local_count("count", range);
            let mut decorator = local_decorator();
            let mut state = LocalState::default();
            decorator.establish(&mut state, &local_equality(&count, 7), false, 0);
            let expression = local_shift(&count, 8, 3);
            let (_, facts) = decorator.proposal(&state, &expression).unwrap();
            assert_eq!(facts.len(), if range == IntRange::USize { 2 } else { 1 });
            for (index, fact) in facts.iter().enumerate() {
                let pair = super::super::super::bitvector_hints::checked(fact.clone()).unwrap();
                let StmX::Block(statements) = &pair.x else { panic!() };
                let StmX::AssertBitVector { requires, ensures } = &statements[0].x else {
                    panic!()
                };
                let StmX::Assume(used) = &statements[1].x else { panic!() };
                assert!(requires.is_empty());
                assert!(Arc::ptr_eq(&ensures[0], used) && Arc::ptr_eq(used, fact));
                let ExpX::Binary(sst::BinaryOp::Implies, premise, conclusion) = &fact.x else {
                    panic!()
                };
                let ExpX::Binary(sst::BinaryOp::Eq, original, result) = &conclusion.x else {
                    panic!()
                };
                assert!(Arc::ptr_eq(original, &expression));
                assert!(
                    matches!(&result.x, ExpX::Const(Constant::Int(value)) if value == &128.into())
                );
                let premise = if range == IntRange::USize {
                    let ExpX::Binary(sst::BinaryOp::And, guard, premise) = &premise.x else {
                        panic!()
                    };
                    let ExpX::Binary(sst::BinaryOp::Eq, architecture, bits) = &guard.x else {
                        panic!()
                    };
                    assert!(matches!(
                        &architecture.x,
                        ExpX::UnaryOpr(
                            UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits),
                            _
                        )
                    ));
                    assert!(
                        matches!(&bits.x, ExpX::Const(Constant::Int(value)) if value == &(if index == 0 { 32 } else { 64 }).into())
                    );
                    premise
                } else {
                    premise
                };
                let ExpX::Binary(sst::BinaryOp::And, within, equality) = &premise.x else {
                    panic!()
                };
                let ExpX::Binary(sst::BinaryOp::Eq, original_count, _) = &equality.x else {
                    panic!()
                };
                assert!(Arc::ptr_eq(original_count, &count));
                assert!(LocalShifts::same_type(&original_count.typ, &count.typ));
                let ExpX::Binary(sst::BinaryOp::And, lower, upper) = &within.x else { panic!() };
                let ExpX::Binary(_, zero, lower_count) = &lower.x else { panic!() };
                let ExpX::Binary(_, upper_count, maximum) = &upper.x else { panic!() };
                assert!(Arc::ptr_eq(lower_count, &count) && Arc::ptr_eq(upper_count, &count));
                assert!(matches!(&zero.x, ExpX::Const(Constant::Int(value)) if value.is_zero()));
                let bits = match range {
                    IntRange::U(bits) => bits,
                    _ => {
                        if index == 0 {
                            32
                        } else {
                            64
                        }
                    }
                };
                assert!(
                    matches!(&maximum.x, ExpX::Const(Constant::Int(value)) if value == &((BigInt::one() << bits) - 1))
                );
            }
        }
    }

    #[test]
    fn local_assertion_provenance_is_sequential_not_an_initializer_or_assume() {
        let count = local_count("count", IntRange::USize);
        let boolean = local_variable("assertion_temp", bool_typ());
        let use_statement = local_use(&count);
        let equality = local_equality(&count, 0);
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        assert_eq!(local_pair_count(&decorator.stm(&mut state, &use_statement, 0).unwrap()), 0);
        decorator
            .stm(&mut state, &Spanned::new(count.span.clone(), StmX::Assume(equality.clone())), 0)
            .unwrap();
        decorator.stm(&mut state, &local_assign(&boolean, &equality, true), 0).unwrap();
        assert_eq!(local_pair_count(&decorator.stm(&mut state, &use_statement, 0).unwrap()), 0);
        let assertion = local_assert(&boolean);
        assert!(Arc::ptr_eq(&decorator.stm(&mut state, &assertion, 0).unwrap(), &assertion));
        let result = decorator.stm(&mut state, &use_statement, 0).unwrap();
        assert_eq!(local_pair_count(&result), 2);
        let StmX::Block(statements) = &result.x else { panic!() };
        assert!(Arc::ptr_eq(statements.last().unwrap(), &use_statement));
    }

    #[test]
    fn local_aliases_and_facts_expire_on_dependency_writes_and_shadowing() {
        let count = local_count("count", IntRange::U(64));
        let boolean = local_variable("first", bool_typ());
        let second = local_variable("second", bool_typ());
        for destination in [&count, &boolean] {
            let mut decorator = local_decorator();
            let mut state = LocalState::default();
            decorator
                .stm(&mut state, &local_assign(&boolean, &local_equality(&count, 0), true), 0)
                .unwrap();
            decorator.stm(&mut state, &local_assign(&second, &boolean, true), 0).unwrap();
            decorator.stm(&mut state, &local_assign(destination, &literal(1), false), 0).unwrap();
            decorator.stm(&mut state, &local_assert(&second), 0).unwrap();
            assert!(decorator.proposal(&state, &local_shift(&count, 64, 1)).is_none());
        }
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        decorator.establish(&mut state, &local_equality(&count, 0), false, 0);
        let ExpX::Var(name) = &count.x else { panic!() };
        let shadow =
            count.new_x(ExpX::Var(VarIdent(name.0.clone(), VarIdentDisambiguate::VirParam)));
        assert!(decorator.proposal(&state, &local_shift(&shadow, 64, 1)).is_none());
        decorator.stm(&mut state, &local_assign(&count, &literal(1), false), 0).unwrap();
        assert!(decorator.proposal(&state, &local_shift(&count, 64, 1)).is_none());
    }

    #[test]
    fn local_rhs_is_processed_before_destination_invalidation() {
        let count = local_count("count", IntRange::U(64));
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        decorator.establish(&mut state, &local_equality(&count, 0), false, 0);
        let statement = local_assign(&count, &local_shift(&count, 64, 1), false);
        assert_eq!(local_pair_count(&decorator.stm(&mut state, &statement, 0).unwrap()), 1);
        assert!(decorator.proposal(&state, &local_shift(&count, 64, 1)).is_none());
    }

    #[test]
    fn local_branch_and_block_aliases_do_not_leak() {
        let count = local_count("count", IntRange::U(64));
        let boolean = local_variable("temporary", bool_typ());
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        let statement = Spanned::new(
            count.span.clone(),
            StmX::If(local_equality(&count, 0), local_use(&count), Some(local_use(&count))),
        );
        let result = decorator.stm(&mut state, &statement, 0).unwrap();
        let StmX::If(_, then_arm, Some(else_arm)) = &result.x else { panic!() };
        assert_eq!(local_pair_count(then_arm), 1);
        assert_eq!(local_pair_count(else_arm), 0);
        assert!(state.bindings.is_empty());
        let block = local_block(vec![local_assign(&boolean, &local_equality(&count, 0), true)]);
        decorator.stm(&mut state, &block, 0).unwrap();
        decorator.stm(&mut state, &local_assert(&boolean), 0).unwrap();
        assert!(decorator.proposal(&state, &local_shift(&count, 64, 1)).is_none());
    }

    #[test]
    fn local_entry_current_prestate_stops_at_the_first_write() {
        let count = local_count("count", IntRange::USize);
        let ExpX::Var(name) = &count.x else { panic!() };
        let parameters = Arc::new(vec![Spanned::new(
            count.span.clone(),
            sst::ParX { name: name.clone(), typ: count.typ.clone(), mode: Mode::Exec },
        )]);
        let pre = count.new_x(ExpX::VarAt(name.clone(), VarAt::Pre));
        let check = local_check(
            local_block(vec![
                local_use(&count),
                local_assign(&count, &literal(1), false),
                local_use(&count),
            ]),
            vec![local_equality(&pre, 0)],
        );
        let result = decorate_local_shifts(&check, &parameters).unwrap();
        assert_eq!(local_pair_count(&result.body), 2);
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        decorator.establish(&mut state, &local_equality(&pre, 0), false, 0);
        assert!(state.bindings.is_empty());
    }

    #[test]
    fn local_unsupported_scopes_calls_and_unknown_writes_clear_provenance() {
        let count = local_count("count", IntRange::U(64));
        let inner = local_block(vec![local_assert(&local_equality(&count, 0)), local_use(&count)]);
        let scopes = vec![
            StmX::DeadEnd(inner.clone()),
            StmX::OpenInvariant(inner.clone()),
            StmX::ClosureInner {
                body: inner.clone(),
                typ_inv_vars: Arc::new(Vec::new()),
                precision_body: None,
            },
            StmX::AssertQuery {
                mode: AssertQueryMode::NonLinear,
                typ_inv_exps: Arc::new(Vec::new()),
                typ_inv_vars: Arc::new(Vec::new()),
                body: inner.clone(),
            },
            StmX::AssertBitVector {
                requires: Arc::new(Vec::new()),
                ensures: Arc::new(vec![local_equality(&count, 0)]),
            },
            StmX::Call {
                fun: sst::CallTarget::AssumeExternal,
                resolved_method: None,
                mode: Mode::Exec,
                is_trait_default: None,
                typ_args: Arc::new(Vec::new()),
                args: Arc::new(Vec::new()),
                split: None,
                dest: None,
                assert_id: None,
                body: None,
            },
            StmX::Assign {
                lhs: sst::Dest { dest: count.new_x(ExpX::Loc(count.clone())), is_init: false },
                rhs: literal(1),
            },
            StmX::Loop {
                loop_isolation: true,
                pre_stms: Arc::new(Vec::new()),
                is_for_loop: false,
                id: 0,
                label: Label { id: 0, name: None },
                cond: None,
                body: inner,
                invs: Arc::new(Vec::new()),
                decrease: Arc::new(Vec::new()),
                typ_inv_vars: Arc::new(Vec::new()),
                modified_vars: None,
                au_branch_bool: None,
                pre_modified_params_incl: None,
                pre_modified_params_excl: None,
            },
        ];
        for scope in scopes {
            let mut decorator = local_decorator();
            let mut state = LocalState::default();
            decorator.establish(&mut state, &local_equality(&count, 0), false, 0);
            let original = Spanned::new(count.span.clone(), scope);
            assert!(Arc::ptr_eq(&original, &decorator.stm(&mut state, &original, 0).unwrap()));
            assert!(state.bindings.is_empty());
        }
    }

    #[test]
    fn local_ineligible_shapes_ranges_and_invalid_counts_have_no_proposal() {
        for range in
            [IntRange::U(8), IntRange::USize, IntRange::I(64), IntRange::ISize, IntRange::Int]
        {
            let count = local_count("count", range);
            let mut decorator = local_decorator();
            let mut state = LocalState::default();
            assert!(decorator.proposal(&state, &local_shift(&count, 8, 1)).is_none());
            decorator.establish(&mut state, &local_equality(&count, 8), false, 0);
            assert!(decorator.proposal(&state, &local_shift(&count, 8, 1)).is_none());
            decorator.establish(&mut state, &local_equality(&count, 0), false, 0);
            let expression = local_shift(&count, 8, 1);
            let ExpX::Binary(_, left, _) = &expression.x else { panic!() };
            let cast =
                count.new_x(ExpX::Unary(UnaryOp::Clip { range, truncate: true }, count.clone()));
            assert!(decorator.proposal(&state, &local_shift(&cast, 8, 1)).is_none());
            let variable_left = expression.new_x(ExpX::Binary(
                sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(8), false)),
                count.clone(),
                count.clone(),
            ));
            assert!(decorator.proposal(&state, &variable_left).is_none());
            let signed = expression.new_x(ExpX::Binary(
                sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(8), true)),
                left.clone(),
                count.clone(),
            ));
            assert!(decorator.proposal(&state, &signed).is_none());
            let architecture = expression.new_x(ExpX::Binary(
                sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::ArchWordSize, false)),
                left.clone(),
                count.clone(),
            ));
            assert!(decorator.proposal(&state, &architecture).is_none());
            let conditional = expression.new_x(ExpX::If(
                local_equality(&count, 0),
                expression.clone(),
                expression.clone(),
            ));
            assert!(decorator.hints(&mut state, &conditional, 0).unwrap().is_empty());
        }
    }

    #[test]
    fn local_budgets_deduplication_alias_chains_and_no_half_pairs() {
        let count = local_count("count", IntRange::USize);
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        decorator.establish(&mut state, &local_equality(&count, 0), false, 0);
        decorator.pairs = 1;
        assert!(decorator.hints(&mut state, &local_shift(&count, 64, 1), 0).unwrap().is_empty());
        assert_eq!(decorator.pairs, 1);
        decorator.pairs = LOCAL_PAIRS;
        assert_eq!(decorator.hints(&mut state, &local_shift(&count, 64, 1), 0).unwrap().len(), 2);
        assert!(decorator.hints(&mut state, &local_shift(&count, 64, 1), 0).unwrap().is_empty());
        decorator.work = 1;
        assert!(decorator.hints(&mut state, &local_shift(&count, 64, 2), 0).unwrap().is_empty());
        assert!(decorator.exhausted);
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        let mut boolean = local_variable("alias0", bool_typ());
        decorator
            .stm(&mut state, &local_assign(&boolean, &local_equality(&count, 0), true), 0)
            .unwrap();
        for index in 1..=LOCAL_ALIASES {
            let next = local_variable(&format!("alias{index}"), bool_typ());
            decorator.stm(&mut state, &local_assign(&next, &boolean, true), 0).unwrap();
            boolean = next;
        }
        assert!(
            decorator.exhausted || decorator.equality(&mut state, &boolean, false, 0).is_none()
        );
        let original = local_block(
            (0..LOCAL_WORK).map(|_| local_assert(&local_equality(&count, 0))).collect(),
        );
        let check = local_check(original.clone(), Vec::new());
        assert!(Arc::ptr_eq(
            &decorate_local_shifts(&check, &Arc::new(Vec::new())).unwrap().body,
            &original
        ));
        let mut decorator = local_decorator();
        assert!(!decorator.visit(LOCAL_DEPTH + 1));
        assert!(decorator.exhausted);
    }

    #[test]
    fn local_safety_wrapper_gets_the_pair_before_the_original_assertion_and_rhs() {
        let count = local_count("count", IntRange::USize);
        let boolean = local_variable("assertion_temp", bool_typ());
        let destination = local_count("result", IntRange::U(64));
        let shift = local_shift(&count, 64, 1);
        let subtraction = shift.new_x(ExpX::Binary(
            sst::BinaryOp::Arith(sst::ArithOp::Sub),
            shift.clone(),
            Expression::number(&shift, 1.into()),
        ));
        let safety = SpannedTyped::new(
            &subtraction.span,
            &bool_typ(),
            ExpX::UnaryOpr(UnaryOpr::HasType(subtraction.typ.clone()), subtraction.clone()),
        );
        let initializer = local_assign(&boolean, &local_equality(&count, 0), true);
        let assertion = local_assert(&boolean);
        let safety_assertion = local_assert(&safety);
        let assignment = local_assign(&destination, &subtraction, true);
        let original = local_block(vec![
            initializer.clone(),
            assertion.clone(),
            safety_assertion.clone(),
            assignment.clone(),
        ]);
        let decorated =
            decorate_local_shifts(&local_check(original, Vec::new()), &Arc::new(Vec::new()))
                .unwrap();
        let StmX::Block(statements) = &decorated.body.x else { panic!() };
        assert!(Arc::ptr_eq(&statements[0], &initializer));
        assert!(Arc::ptr_eq(&statements[1], &assertion));
        let StmX::Block(before_safety) = &statements[2].x else { panic!() };
        assert_eq!(before_safety.len(), 3);
        assert_eq!(local_pair_count(&statements[2]), 2);
        assert!(Arc::ptr_eq(before_safety.last().unwrap(), &safety_assertion));
        assert!(Arc::ptr_eq(&statements[3], &assignment));
        for pair in before_safety.iter().take(2) {
            let StmX::Block(checked) = &pair.x else { panic!() };
            let StmX::AssertBitVector { ensures, .. } = &checked[0].x else { panic!() };
            let StmX::Assume(used) = &checked[1].x else { panic!() };
            assert!(Arc::ptr_eq(&ensures[0], used));
            let ExpX::Binary(sst::BinaryOp::Implies, _, conclusion) = &used.x else { panic!() };
            let ExpX::Binary(sst::BinaryOp::Eq, original_shift, _) = &conclusion.x else {
                panic!()
            };
            assert!(Arc::ptr_eq(original_shift, &shift));
        }
    }

    #[test]
    fn local_safety_wrapper_does_not_seed_equality_or_scan_guarded_subtrees() {
        let count = local_count("count", IntRange::USize);
        let shift = local_shift(&count, 64, 1);
        let wrapped_equality = SpannedTyped::new(
            &count.span,
            &bool_typ(),
            ExpX::UnaryOpr(UnaryOpr::HasType(bool_typ()), local_equality(&count, 0)),
        );
        let mut decorator = local_decorator();
        let mut state = LocalState::default();
        decorator.stm(&mut state, &local_assert(&wrapped_equality), 0).unwrap();
        assert_eq!(local_pair_count(&decorator.stm(&mut state, &local_use(&count), 0).unwrap()), 0);
        decorator.establish(&mut state, &local_equality(&count, 0), false, 0);
        let conditional = shift.new_x(ExpX::If(
            local_equality(&count, 0),
            shift.clone(),
            Expression::number(&shift, 0.into()),
        ));
        let guarded = SpannedTyped::new(
            &shift.span,
            &bool_typ(),
            ExpX::UnaryOpr(UnaryOpr::HasType(shift.typ.clone()), conditional),
        );
        let statement = local_assert(&guarded);
        assert!(Arc::ptr_eq(&decorator.stm(&mut state, &statement, 0).unwrap(), &statement));
        let safety = SpannedTyped::new(
            &shift.span,
            &bool_typ(),
            ExpX::UnaryOpr(UnaryOpr::HasType(shift.typ.clone()), shift.clone()),
        );
        let statement = local_assert(&safety);
        decorator.work = 2;
        assert!(Arc::ptr_eq(&decorator.stm(&mut state, &statement, 0).unwrap(), &statement));
        assert!(decorator.exhausted);
        assert_eq!(decorator.pairs, LOCAL_PAIRS);
    }
    fn literal(number: u64) -> Exp {
        let input = crate::infer::tests::input("constant");
        SpannedTyped::new(
            &input.span,
            &Arc::new(TypX::Int(IntRange::U(64))),
            ExpX::Const(Constant::Int(number.into())),
        )
    }

    fn shift(amount: u64) -> Exp {
        let operand = literal(128);
        operand.new_x(ExpX::Binary(
            sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(64), false)),
            operand.clone(),
            literal(amount),
        ))
    }

    #[test]
    fn closed_guesses_remain_separate_from_identical_backend_checked_facts() {
        let expression = shift(8);
        let guess = value(&expression, &mut 8192, DEPTH).unwrap();
        assert_eq!(guess, 32768.into());
        let fact =
            binary(sst::BinaryOp::Eq, expression.clone(), Expression::number(&expression, guess));
        let pair = super::super::super::bitvector_hints::checked(fact.clone()).unwrap();
        let StmX::Block(statements) = &pair.x else { panic!() };
        let StmX::AssertBitVector { requires, ensures } = &statements[0].x else { panic!() };
        let StmX::Assume(assumed) = &statements[1].x else { panic!() };
        assert!(requires.is_empty());
        assert!(Arc::ptr_eq(&ensures[0], &fact));
        assert!(Arc::ptr_eq(assumed, &fact));
        let ExpX::Binary(sst::BinaryOp::Eq, checked_expression, _) = &fact.x else { panic!() };
        assert!(Arc::ptr_eq(checked_expression, &expression));
    }

    #[test]
    fn closed_evaluation_declines_variables_large_shifts_and_exhausted_budgets() {
        let input = crate::infer::tests::input("unknown");
        let ExprX::Var(name) = &input.x else { panic!() };
        let variable = literal(0).new_x(ExpX::Var(name.clone()));
        assert!(value(&variable, &mut 8192, DEPTH).is_none());
        assert!(value(&shift(64), &mut 8192, DEPTH).is_none());
        assert!(value(&shift(1), &mut 1, DEPTH).is_none());
        assert!(value(&shift(1), &mut 8192, 1).is_none());
        let architecture = literal(1).new_x(ExpX::Unary(
            UnaryOp::Clip { range: IntRange::USize, truncate: true },
            literal(1),
        ));
        assert!(value(&architecture, &mut 8192, DEPTH).is_none());
    }

    #[test]
    fn inferred_summaries_reserve_the_existing_request_budget() {
        let input = crate::infer::tests::input("unused");
        let operand = input.new_x(ExprX::Const(Constant::Int(128.into())));
        let expression = input.new_x(ExprX::Binary(
            BinaryOp::Bitwise(BitwiseOp::BitOr, BitshiftBehavior::Allow),
            operand,
            input.new_x(ExprX::Const(Constant::Int(1.into()))),
        ));
        let equality = SpannedTyped::new(
            &input.span,
            &bool_typ(),
            ExprX::Binary(
                BinaryOp::Eq(Mode::Spec),
                expression,
                input.new_x(ExprX::Const(Constant::Int(129.into()))),
            ),
        );
        let function = crate::infer::tests::function("bounded", input);
        let post_condition = sst::PostConditionSst {
            dest: None,
            ens_exps: Arc::new(Vec::new()),
            ens_spec_precondition_stms: Arc::new(Vec::new()),
            kind: sst::PostConditionKind::Ensures,
        };
        for inferred in [true, false] {
            let mut data = function.x.clone();
            let mut attributes = (*data.attrs).clone();
            attributes.infer = inferred;
            data.attrs = Arc::new(attributes);
            data.params = Arc::new(Vec::new());
            data.ensure.0 = Arc::new(vec![equality.clone()]);
            let function = function.new_x(data);
            let callees = HashMap::from([(function.x.name.clone(), function.clone())]);
            let mut requests = Requests {
                callees: &callees,
                post_condition: &post_condition,
                definitions: HashMap::new(),
                assumptions: Vec::new(),
                remaining: REQUESTS,
                visits: VISITS,
                work: WORK,
            };
            let facts =
                requests.constant_requests(&function.x.name, &Arc::new(Vec::new())).unwrap();
            assert_eq!(facts.len(), usize::from(!inferred));
            assert_eq!(requests.remaining, REQUESTS - usize::from(!inferred));
            if inferred {
                assert_eq!(requests.work, WORK);
                assert_eq!(requests.visits, VISITS);
            }
        }
    }
}
