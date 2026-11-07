use super::*;
use crate::infer::tests::{Value, evaluate, function, input, span};

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    let typ = if matches!(op, BinaryOp::Arith(_)) { input("x").typ.clone() } else { bool_typ() };
    SpannedTyped::new(&span(), &typ, ExprX::Binary(op, left, right))
}

fn ret(value: Expr) -> Expr {
    let typ = Arc::new(TypX::Decorate(TypDecoration::Never, None, crate::ast_util::unit_typ()));
    SpannedTyped::new(&span(), &typ, ExprX::Return(Some(value)))
}

fn branch(condition: Expr, yes: Expr, no: Option<Expr>) -> Expr {
    SpannedTyped::new(&span(), &crate::ast_util::unit_typ(), ExprX::If(condition, yes, no))
}

fn block(statement: Expr, tail: Expr) -> Expr {
    let typ = tail.typ.clone();
    SpannedTyped::new(
        &span(),
        &typ,
        ExprX::Block(
            Arc::new(vec![crate::def::Spanned::new(span(), StmtX::Expr(statement))]),
            Some(tail),
        ),
    )
}

#[test]
fn exhaustive_u8_record_return_accumulation_matches_checked_native_semantics() {
    let add = BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8))));
    let sub = BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8))));
    let sum = || binary(add, input("x"), input("y"));
    let difference = || binary(sub, input("x"), input("y"));
    let less = || binary(BinaryOp::Inequality(InequalityOp::Lt), input("x"), input("y"));
    type Reference = fn(u8, u8) -> Option<u8>;
    let cases: [(&str, Expr, Reference); 3] = [
        (
            "partial return and tail",
            block(branch(less(), ret(sum()), None), difference()),
            |left_input, right_input| {
                if left_input < right_input {
                    left_input.checked_add(right_input)
                } else {
                    left_input.checked_sub(right_input)
                }
            },
        ),
        (
            "nested returns bypass later arithmetic",
            block(
                branch(less(), block(branch(less(), ret(input("y")), None), ret(sum())), None),
                difference(),
            ),
            |left_input, right_input| {
                if left_input < right_input {
                    Some(right_input)
                } else {
                    left_input.checked_sub(right_input)
                }
            },
        ),
        (
            "all branches return before unsafe tail",
            block(branch(less(), ret(input("x")), Some(ret(input("y")))), sum()),
            |left_input, right_input| {
                Some(if left_input < right_input { left_input } else { right_input })
            },
        ),
    ];
    for (name, body, reference) in cases {
        let function = function(name, body.clone());
        let all = HashMap::new();
        let mut engine = Interpreter {
            function: &function,
            shape: Shape { typ: crate::ast_util::unit_typ(), fields: vec![] },
            all: &all,
            datatypes: &[],
            requirements: vec![],
            exits: vec![],
            callbacks: vec![],
            remaining: FORMULA_BUDGET,
            plans: vec![],
        };
        let locals = function
            .x
            .params
            .iter()
            .map(|parameter| {
                (parameter.x.name.clone(), atom(input(parameter.x.name.0.as_str())).unwrap())
            })
            .collect();
        let mut state = State { fields: vec![], locals, active: boolean(&span(), true) };
        let normal = engine.expr(&mut state, &body).unwrap();
        let output = engine.finish(&state, normal).unwrap();
        for left_input in 0..=255u8 {
            for right_input in 0..=255u8 {
                let expected = reference(left_input, right_input);
                let safe = engine.requirements.iter().all(|predicate| {
                    evaluate(&predicate.expr, left_input, right_input) == Value::Bool(true)
                });
                assert_eq!(safe, expected.is_some(), "{name}({left_input},{right_input})");
                if let Some(value) = expected {
                    assert_eq!(
                        evaluate(&output.expr, left_input, right_input),
                        Value::Int(value.into()),
                        "{name}({left_input},{right_input})"
                    );
                }
            }
        }
    }
}
