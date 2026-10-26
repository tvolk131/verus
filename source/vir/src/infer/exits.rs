//! Function exits are captured as guarded values. This only builds formulas;
//! the executable body and its verifier-generated return checks stay intact.
use super::*;

pub(super) struct Exits {
    pub active: Term,
    pub values: Vec<(Term, Term)>,
    pub result_type: Typ,
}

impl Exits {
    pub fn for_body(body: &Expr, result_type: &Typ) -> Option<Self> {
        let mut found = false;
        crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
            found |= matches!(e.x, ExprX::Return(_));
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
        found.then(|| Self {
            active: boolean(&body.span, true),
            values: Vec::new(),
            result_type: result_type.clone(),
        })
    }
}

pub(super) fn not(span: &Span, value: Term) -> Result<Term, VirErr> {
    if let ExprX::Const(Constant::Bool(b)) = value.expr.x {
        Ok(boolean(span, !b))
    } else {
        summary::unary(span, UnaryOp::Not, value)
    }
}

pub(super) fn unit_value(span: &Span) -> Result<Term, VirErr> {
    enums::ctor(span, &crate::ast_util::unit_typ(), &crate::def::prefix_tuple_variant(0), None)
}

/// An unreachable normal value for a never-to-any coercion. These values are
/// only used outside the active path; all executed values are proved normally.
fn unused_value(span: &Span, typ: &Typ) -> Result<Term, VirErr> {
    if matches!(&**typ, TypX::Bool) {
        return Ok(boolean(span, false));
    }
    if eligibility::integer(typ) {
        return term(span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1);
    }
    if let Some((element, length)) = arrays::shape(typ) {
        let typ = arrays::typ(element.clone(), length).unwrap();
        return arrays::literal(span, &typ, vec![unused_value(span, &element)?; length]);
    }
    let (variant, payload) = enums::variants(typ)
        .and_then(|v| v.into_iter().next())
        .ok_or_else(|| unsupported(span, "unreachable value outside the supported subset"))?;
    enums::ctor(span, typ, &variant, payload.map(|t| unused_value(span, &t)).transpose()?)
}

impl Engine<'_> {
    pub(super) fn capture_exit(
        &mut self,
        span: &Span,
        guard: Term,
        output: Term,
    ) -> Result<(), VirErr> {
        let exits = self.exits.as_mut().ok_or_else(|| unsupported(span, "early returns"))?;
        if !enums::compatible(&output.expr.typ, &exits.result_type) {
            return Err(unsupported(span, "return value type mismatch"));
        }
        if !is_bool(&guard, false) {
            exits.values.push((guard, output));
        }
        Ok(())
    }

    pub(super) fn require_active(&self, span: &Span, requirement: Term) -> Result<Term, VirErr> {
        match &self.exits {
            Some(exits) => {
                conditional(span, exits.active.clone(), requirement, boolean(span, true))
            }
            None => Ok(requirement),
        }
    }

    pub(super) fn return_expr(
        &mut self,
        span: &Span,
        expr: Option<&Expr>,
    ) -> Result<Summary, VirErr> {
        let arg = match expr {
            Some(expr) => self.expr(expr)?,
            None => Summary { safe: boolean(span, true), value: None },
        };
        let exits = self.exits.as_mut().ok_or_else(|| unsupported(span, "early returns"))?;
        let output = match arg.value {
            Some(output) => output,
            None if enums::unit(&exits.result_type) => unused_value(span, &exits.result_type)?,
            None => return Err(unsupported(span, "return requires a supported value")),
        };
        let active = exits.active.clone();
        self.capture_exit(span, active, output)?;
        self.exits.as_mut().unwrap().active = boolean(span, false);
        Ok(Summary { safe: arg.safe, value: None })
    }

    pub(super) fn never_value(&mut self, expr: &Expr, inner: &Expr) -> Result<Summary, VirErr> {
        let mut summary = self.expr(inner)?;
        if self.exits.is_some() && summary.value.is_none() && !enums::unit(&expr.typ) {
            summary.value = Some(unused_value(&expr.span, &expr.typ)?);
        }
        Ok(summary)
    }

    pub(super) fn finish_exits(&self, span: &Span, summary: Summary) -> Result<Summary, VirErr> {
        let Some(exits) = &self.exits else { return Ok(summary) };
        let mut output = match summary.value {
            Some(value) => value,
            None if is_bool(&exits.active, false) || enums::unit(&exits.result_type) => {
                unused_value(span, &exits.result_type)?
            }
            None => return Err(unsupported(span, "missing normal return value")),
        };
        for (guard, value) in exits.values.iter().rev() {
            output = summary::conditional(span, guard.clone(), value.clone(), output)?;
        }
        Ok(Summary { safe: summary.safe, value: Some(output) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate, function, input, span};

    fn binary(op: BinaryOp, a: Expr, b: Expr) -> Expr {
        let typ =
            if matches!(op, BinaryOp::Arith(_)) { input("x").typ.clone() } else { bool_typ() };
        SpannedTyped::new(&span(), &typ, ExprX::Binary(op, a, b))
    }

    fn ret(value: Expr) -> Expr {
        let typ = Arc::new(TypX::Decorate(TypDecoration::Never, None, crate::ast_util::unit_typ()));
        SpannedTyped::new(&span(), &typ, ExprX::Return(Some(value)))
    }

    fn coerce(value: Expr, typ: &Typ) -> Expr {
        SpannedTyped::new(&span(), typ, ExprX::NeverToAny(value))
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
    fn exhaustive_u8_returns_preserve_reachability_and_operand_order() {
        let add = BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8))));
        let sub = BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8))));
        let div = BinaryOp::Arith(ArithOp::EuclideanDiv(Div0Behavior::Error));
        let sum = || binary(add, input("x"), input("y"));
        let difference = || binary(sub, input("x"), input("y"));
        let less = || binary(BinaryOp::Inequality(InequalityOp::Lt), input("x"), input("y"));
        let zero =
            SpannedTyped::new(&span(), &input("x").typ, ExprX::Const(Constant::Int(0.into())));
        let y_zero = binary(BinaryOp::Eq(Mode::Exec), input("y"), zero.clone());
        let guard_return = |guard: Expr, value: Expr| {
            SpannedTyped::new(
                &span(),
                &crate::ast_util::unit_typ(),
                ExprX::If(guard, ret(value), None),
            )
        };
        type Reference = fn(u8, u8) -> Option<u8>;
        let cases: [(&str, Expr, Reference); 5] = [
            (
                "guarded division",
                block(guard_return(y_zero, zero), binary(div, input("x"), input("y"))),
                |x, y| Some(if y == 0 { 0 } else { x / y }),
            ),
            ("branch return", block(guard_return(less(), sum()), difference()), |x, y| {
                if x < y { x.checked_add(y) } else { x.checked_sub(y) }
            }),
            (
                "prior operand",
                binary(div, sum(), coerce(ret(input("x")), &input("x").typ)),
                |x, y| x.checked_add(y).map(|_| x),
            ),
            (
                "later operand",
                binary(div, coerce(ret(input("y")), &input("x").typ), sum()),
                |_, y| Some(y),
            ),
            (
                "short circuit return",
                block(
                    SpannedTyped::new(
                        &span(),
                        &bool_typ(),
                        ExprX::Logical(LogicalOp::And, less(), coerce(ret(sum()), &bool_typ())),
                    ),
                    difference(),
                ),
                |x, y| if x < y { x.checked_add(y) } else { x.checked_sub(y) },
            ),
        ];
        for (name, body, reference) in cases {
            let original = function(name, body);
            let inferred = infer_function(&original).unwrap();
            assert!(Arc::ptr_eq(
                original.x.body.as_ref().unwrap(),
                inferred.x.body.as_ref().unwrap()
            ));
            let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else {
                panic!("contract")
            };
            for x in 0..=255u8 {
                for y in 0..=255u8 {
                    let expected = reference(x, y);
                    assert_eq!(
                        evaluate(&inferred.x.require[0], x, y),
                        Value::Bool(expected.is_some()),
                        "{name}({x},{y})"
                    );
                    if let Some(expected) = expected {
                        assert_eq!(
                            evaluate(output, x, y),
                            Value::Int(expected.into()),
                            "{name}({x},{y})"
                        );
                    }
                }
            }
        }
    }
}
