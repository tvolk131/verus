use super::*;
use crate::def::Spanned;
use crate::sst::{self, Exp, ExpX, FuncCheckSst, StmX};
use num_traits::ToPrimitive;
use std::collections::{BTreeSet, HashSet};

fn shapes(
    function: &Function,
    shapes: &mut BTreeSet<(u32, u64)>,
    calls: &mut Vec<Fun>,
    remaining: &mut usize,
) {
    for requirement in function
        .x
        .require
        .iter()
        .chain(function.x.ensure.0.iter())
        .chain(function.x.body.iter().filter(|_| function.x.mode == Mode::Spec))
    {
        crate::ast_visitor::expr_visitor_walk(requirement, &mut |expression| {
            if *remaining == 0 {
                return crate::ast_visitor::VisitorControlFlow::<()>::Return;
            }
            *remaining -= 1;
            if let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } = &expression.x {
                calls.push(name.clone());
            }
            if let ExprX::Binary(
                BinaryOp::Bitwise(BitwiseOp::BitXor, BitshiftBehavior::Allow),
                left,
                right,
            ) = &expression.x
            {
                let width = match &*left.typ {
                    TypX::Int(IntRange::U(width @ (8 | 16 | 32 | 64))) => Some(*width),
                    TypX::Int(IntRange::USize) => Some(0),
                    _ => None,
                };
                if let (Some(width), ExprX::Const(Constant::Int(mask))) = (width, &right.x) {
                    if let Some(mask) = mask.to_u64().filter(|mask| {
                        *mask > 0 && mask.ilog2() < if width == 0 { 31 } else { width - 1 }
                    }) {
                        shapes.insert((width, mask));
                    }
                }
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
}

fn fact(span: &Span, width: u32, mask: u64) -> Exp {
    let typ = Arc::new(TypX::Int(if width == 0 { IntRange::USize } else { IntRange::U(width) }));
    let name =
        VarIdent(Arc::new("__verus_infer_xor_operand".into()), VarIdentDisambiguate::AirLocal);
    let value = SpannedTyped::new(span, &typ, ExpX::Var(name.clone()));
    let number = |number: u64| {
        SpannedTyped::new(span, &int_typ(), ExpX::Const(Constant::Int(number.into())))
    };
    let binary = |operation, left: Exp, right: Exp| {
        SpannedTyped::new(span, &bool_typ(), ExpX::Binary(operation, left, right))
    };
    let xor = SpannedTyped::new(
        span,
        &typ,
        ExpX::Binary(sst::BinaryOp::Bitwise(BitwiseOp::BitXor), value.clone(), number(mask)),
    );
    let bound = 1u64 << (mask.ilog2() + 1);
    let within = |expression: Exp| {
        binary(
            sst::BinaryOp::And,
            binary(sst::BinaryOp::Inequality(InequalityOp::Le), number(0), expression.clone()),
            binary(sst::BinaryOp::Inequality(InequalityOp::Lt), expression, number(bound)),
        )
    };
    let consequence = if mask == bound - 1 {
        let difference = SpannedTyped::new(
            span,
            &int_typ(),
            ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Sub), number(mask), value.clone()),
        );
        binary(
            sst::BinaryOp::And,
            within(xor.clone()),
            binary(sst::BinaryOp::Eq, xor.clone(), difference),
        )
    } else {
        within(xor.clone())
    };
    let relation = binary(sst::BinaryOp::Implies, within(value), consequence);
    let binders = Arc::new(vec![Arc::new(VarBinderX { name, a: typ })]);
    let triggers = Arc::new(vec![Arc::new(vec![xor])]);
    let binder = Spanned::new(
        span.clone(),
        sst::BndX::Quant(crate::ast_util::QUANT_FORALL, binders, triggers, None),
    );
    SpannedTyped::new(span, &bool_typ(), ExpX::Bind(binder, relation))
}

pub(super) fn decorate(
    check: &FuncCheckSst,
    callees: &HashMap<Fun, Function>,
) -> Result<FuncCheckSst, VirErr> {
    let mut selected = BTreeSet::new();
    let mut calls = Vec::new();
    crate::sst_visitor::map_stm_visitor(&check.body, &mut |statement| {
        if let StmX::Call { fun: sst::CallTarget::Fun(name), .. } = &statement.x {
            calls.push(name.clone());
        }
        Ok(statement.clone())
    })?;
    for expression in check.reqs.iter().chain(check.post_condition.ens_exps.iter()) {
        crate::sst_visitor::map_exp_visitor(expression, &mut |expression| {
            if let ExpX::Call(sst::CallFun::Fun(name, _) | sst::CallFun::Recursive(name), ..) =
                &expression.x
            {
                calls.push(name.clone());
            }
            expression.clone()
        });
    }
    let mut seen = HashSet::new();
    let mut remaining = FORMULA_BUDGET;
    while let Some(name) = calls.pop() {
        if seen.len() >= 64 || remaining == 0 {
            break;
        }
        if seen.insert(name.clone()) {
            if let Some(function) = callees.get(&name) {
                shapes(function, &mut selected, &mut calls, &mut remaining);
            }
        }
    }
    if selected.is_empty() {
        return Ok(check.clone());
    }
    let mut statements = selected
        .into_iter()
        .take(4)
        .map(|(width, mask)| bitvector_hints::checked(fact(&check.body.span, width, mask)))
        .collect::<Result<Vec<_>, _>>()?;
    statements.push(check.body.clone());
    let mut result = check.clone();
    result.body = Spanned::new(check.body.span.clone(), StmX::Block(Arc::new(statements)));
    Ok(result)
}
