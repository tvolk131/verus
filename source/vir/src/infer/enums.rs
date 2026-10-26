//! Closed support for core Option/Result over supported scalar payloads, plus unit.
//! Checked arithmetic candidates are verified using the existing vstd models.
use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) enum ValueType {
    Bool,
    Unsigned(u32),
    USize,
    ISize,
    Signed(u32),
    Unit,
    Option(Box<ValueType>),
    Result(Box<ValueType>, Box<ValueType>),
}
fn path(module: &str, name: &str) -> Path {
    Arc::new(PathX {
        krate: CrateId::Core,
        segments: Arc::new(vec![Arc::new(module.into()), Arc::new(name.into())]),
    })
}
fn is_path(dt: &Dt, module: &str, name: &str) -> bool {
    matches!(dt, Dt::Path(p) if p.krate == CrateId::Core && p.segments.len() == 2 && p.segments[0].as_str() == module && p.segments[1].as_str() == name)
}
pub(super) fn unit(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Datatype(Dt::Tuple(0), ts, _) if ts.is_empty())
}
fn payload(typ: &Typ) -> bool {
    scalar(typ) || unit(typ)
}
pub(super) fn variants(typ: &Typ) -> Option<Vec<(Ident, Option<Typ>)>> {
    let TypX::Datatype(dt, ts, _) = &**typ else {
        return None;
    };
    let ident = |s: &str| Arc::new(s.to_owned());
    if is_path(dt, "option", "Option") && ts.len() == 1 && payload(&ts[0]) {
        Some(vec![(ident("None"), None), (ident("Some"), Some(ts[0].clone()))])
    } else if is_path(dt, "result", "Result") && ts.len() == 2 && ts.iter().all(payload) {
        Some(vec![(ident("Ok"), Some(ts[0].clone())), (ident("Err"), Some(ts[1].clone()))])
    } else if unit(typ) {
        Some(vec![(crate::def::prefix_tuple_variant(0), None)])
    } else {
        None
    }
}
pub(super) fn supported(typ: &Typ) -> bool {
    scalar(typ) || variants(typ).is_some()
}
pub(super) fn compatible(a: &Typ, b: &Typ) -> bool {
    (eligibility::integer(a) && eligibility::integer(b))
        || arrays::compatible(a, b)
        || slices::compatible(a, b)
        || crate::ast_util::types_equal(a, b)
}
impl ValueType {
    pub(super) fn from_typ(typ: &Typ) -> Option<Self> {
        if !supported(typ) {
            return None;
        }
        match &**typ {
            TypX::Bool => Some(Self::Bool),
            TypX::Int(IntRange::USize) => Some(Self::USize),
            TypX::Int(IntRange::ISize) => Some(Self::ISize),
            TypX::Int(IntRange::U(bits)) => Some(Self::Unsigned(*bits)),
            TypX::Int(IntRange::I(bits)) => Some(Self::Signed(*bits)),
            TypX::Datatype(Dt::Tuple(0), ..) => Some(Self::Unit),
            TypX::Datatype(dt, ts, _) if is_path(dt, "option", "Option") => {
                Some(Self::Option(Box::new(Self::from_typ(&ts[0])?)))
            }
            TypX::Datatype(dt, ts, _) if is_path(dt, "result", "Result") => Some(Self::Result(
                Box::new(Self::from_typ(&ts[0])?),
                Box::new(Self::from_typ(&ts[1])?),
            )),
            _ => None,
        }
    }
    pub(super) fn typ(&self) -> Option<Typ> {
        let typ = match self {
            Self::Bool => bool_typ(),
            Self::USize => Arc::new(TypX::Int(IntRange::USize)),
            Self::ISize => Arc::new(TypX::Int(IntRange::ISize)),
            Self::Unsigned(bits @ (8 | 16 | 32 | 64 | 128)) => {
                Arc::new(TypX::Int(IntRange::U(*bits)))
            }
            Self::Signed(bits @ (8 | 16 | 32 | 64 | 128)) => {
                Arc::new(TypX::Int(IntRange::I(*bits)))
            }
            Self::Unit => crate::ast_util::unit_typ(),
            Self::Option(t) => Arc::new(TypX::Datatype(
                Dt::Path(path("option", "Option")),
                Arc::new(vec![t.typ()?]),
                Arc::new(vec![]),
            )),
            Self::Result(t, e) => Arc::new(TypX::Datatype(
                Dt::Path(path("result", "Result")),
                Arc::new(vec![t.typ()?, e.typ()?]),
                Arc::new(vec![]),
            )),
            _ => return None,
        };
        supported(&typ).then_some(typ)
    }
}
fn variant_payload(typ: &Typ, variant: &Ident) -> Option<Option<Typ>> {
    variants(typ)?.into_iter().find(|(v, _)| v == variant).map(|(_, t)| t)
}
pub(super) fn ctor(
    span: &Span,
    typ: &Typ,
    variant: &Ident,
    field: Option<Term>,
) -> Result<Term, VirErr> {
    let expected = variant_payload(typ, variant)
        .ok_or_else(|| unsupported(span, "unsupported enum constructor"))?;
    let mut fields = Vec::new();
    let mut size = 1;
    match (expected, field) {
        (None, None) => {}
        (Some(expected), Some(mut field)) if compatible(&field.expr.typ, &expected) => {
            if !crate::ast_util::types_equal(&field.expr.typ, &expected) {
                let TypX::Int(
                    range @ (IntRange::USize | IntRange::ISize | IntRange::U(_) | IntRange::I(_)),
                ) = &*expected
                else {
                    return Err(unsupported(span, "enum payload type mismatch"));
                };
                field = term(
                    span,
                    &expected,
                    ExprX::Unary(UnaryOp::Clip { range: *range, truncate: true }, field.expr),
                    1 + field.size,
                )?;
            }
            size += field.size;
            fields.push(Arc::new(air::ast::BinderX {
                name: crate::def::positional_field_ident(0),
                a: field.expr,
            }));
        }
        _ => return Err(unsupported(span, "enum payload type or arity mismatch")),
    }
    let TypX::Datatype(dt, ..) = &**typ else { unreachable!() };
    term(span, typ, ExprX::Ctor(dt.clone(), variant.clone(), Arc::new(fields), None), size)
}
pub(super) fn is_variant(span: &Span, value: Term, variant: &Ident) -> Result<Term, VirErr> {
    variant_payload(&value.expr.typ, variant)
        .ok_or_else(|| unsupported(span, "unsupported variant test"))?;
    if let ExprX::Ctor(_, actual, _, None) = &value.expr.x {
        return Ok(boolean(span, actual == variant));
    }
    let TypX::Datatype(dt, ..) = &*value.expr.typ else { unreachable!() };
    term(
        span,
        &bool_typ(),
        ExprX::UnaryOpr(
            UnaryOpr::IsVariant { datatype: dt.clone(), variant: variant.clone() },
            value.expr,
        ),
        1 + value.size,
    )
}
pub(super) fn field(span: &Span, value: Term, variant: &Ident) -> Result<Term, VirErr> {
    let typ = variant_payload(&value.expr.typ, variant)
        .flatten()
        .ok_or_else(|| unsupported(span, "variant has no scalar payload"))?;
    if let ExprX::Ctor(_, actual, fields, None) = &value.expr.x {
        if actual == variant {
            return Ok(Term { expr: fields[0].a.clone(), size: value.size - 1 });
        }
    }
    let TypX::Datatype(dt, ..) = &*value.expr.typ else { unreachable!() };
    let op = FieldOpr {
        datatype: dt.clone(),
        variant: variant.clone(),
        field: crate::def::positional_field_ident(0),
        get_variant: false,
        check: VariantCheck::None,
    };
    term(span, &typ, ExprX::UnaryOpr(UnaryOpr::Field(op), value.expr), 1 + value.size)
}

pub(super) use super::integer_models::Operation as Checked;

pub(super) fn checked(function: &Function) -> Option<Checked> {
    let m = integer_models::recognize(function)?;
    (m.policy == integer_models::Policy::Checked).then_some(m.operation)
}
#[cfg(test)]
pub(super) fn checked_value(
    span: &Span,
    function: &Function,
    a: Term,
    b: Term,
) -> Result<Term, VirErr> {
    if checked(function).is_none() {
        return Err(unsupported(span, "unapproved checked arithmetic model"));
    }
    integer_models::value(span, function, vec![a, b])
}

pub(super) fn validate_match(place: &Place, arms: &Arms, irrefutable: bool) -> Result<(), VirErr> {
    if irrefutable || !supported(&place.typ) || arms.is_empty() {
        return Err(unsupported(&place.span, "unsupported or nonexhaustive match"));
    }
    let name = VarIdent(Arc::new("__infer_match_check".into()), VarIdentDisambiguate::AirLocal);
    let selected =
        Term { expr: SpannedTyped::new(&place.span, &place.typ, ExprX::Var(name)), size: 1 };
    let empty = HashMap::new();
    let mut engine =
        Engine { callees: &empty, locals: HashMap::new(), remaining: FORMULA_BUDGET, exits: None };
    for arm in arms.iter() {
        if !matches!(arm.x.guard.x, ExprX::Const(Constant::Bool(true))) {
            return Err(unsupported(&arm.x.guard.span, "match guards"));
        }
        engine.bind_pattern(&arm.x.pattern, selected.clone())?;
    }
    Ok(())
}
pub(super) fn valid_field(op: &FieldOpr, base: &Typ) -> bool {
    op.check == VariantCheck::None
        && !op.get_variant
        && op.field == crate::def::positional_field_ident(0)
        && matches!(&**base, TypX::Datatype(dt, ..) if dt == &op.datatype)
        && variant_payload(base, &op.variant).flatten().is_some()
}

impl Engine<'_> {
    fn bind_pattern(&mut self, pattern: &Pattern, value: Term) -> Result<Term, VirErr> {
        self.spend(&pattern.span)?;
        let span = &pattern.span;
        match &pattern.x {
            PatternX::Wildcard => Ok(boolean(span, true)),
            PatternX::Var(binding) if eligibility::immutable_binding(binding) => {
                self.locals.insert(binding.name.clone(), value);
                Ok(boolean(span, true))
            }
            PatternX::Constructor(dt, variant, fields) if supported(&pattern.typ) => {
                if !matches!(&*value.expr.typ, TypX::Datatype(actual, ..) if actual == dt) {
                    return Err(unsupported(span, "match datatype mismatch"));
                }
                let mut test = is_variant(span, value.clone(), variant)?;
                if let Some(payload) = variant_payload(&pattern.typ, variant)
                    .ok_or_else(|| unsupported(span, "unsupported match variant"))?
                {
                    if fields.len() != 1 || fields[0].name != crate::def::positional_field_ident(0)
                    {
                        return Err(unsupported(
                            span,
                            "only single-payload variants are supported",
                        ));
                    }
                    if !crate::ast_util::types_equal(&fields[0].a.typ, &payload) {
                        return Err(unsupported(span, "match payload type mismatch"));
                    }
                    let selected = field(span, value, variant)?;
                    let sub = self.bind_pattern(&fields[0].a, selected)?;
                    test = logical(span, LogicalOp::And, test, sub)?;
                } else if !fields.is_empty() {
                    return Err(unsupported(span, "unexpected empty-variant fields"));
                }
                Ok(test)
            }
            PatternX::Expr(e)
                if (matches!(e.x, ExprX::Const(Constant::Bool(_) | Constant::Int(_)))
                    || arithmetic::negative_literal(e).is_some())
                    && scalar(&pattern.typ) =>
            {
                let c = self.expr(e)?;
                let c = super::value(&c, span)?;
                term(
                    span,
                    &bool_typ(),
                    ExprX::Binary(BinaryOp::Eq(Mode::Spec), value.expr, c.expr),
                    1 + value.size + c.size,
                )
            }
            _ => Err(unsupported(
                span,
                "only by-value Option/Result patterns, scalar literals, and wildcards are supported",
            )),
        }
    }
    pub(super) fn match_expr(
        &mut self,
        span: &Span,
        result_type: &Typ,
        place: &Place,
        arms: &Arms,
        assert_irrefutable: bool,
    ) -> Result<Summary, VirErr> {
        if assert_irrefutable || !supported(&place.typ) || arms.is_empty() {
            return Err(unsupported(span, "unsupported or nonexhaustive match"));
        }
        let scrutinee = self.place(place)?;
        let selected = super::value(&scrutinee, span)?;
        let saved = self.locals.clone();
        let unit_result = self.exits.is_some() && unit(result_type);
        let entry = self.exits.as_ref().map(|e| (e.active.clone(), e.values.len()));
        let mut remaining = boolean(span, true);
        let mut alive = boolean(span, false);
        let mut branches = Vec::new();
        for (index, arm) in arms.iter().enumerate() {
            self.locals = saved.clone();
            if !matches!(arm.x.guard.x, ExprX::Const(Constant::Bool(true))) {
                return Err(unsupported(&arm.x.guard.span, "match guards"));
            }
            let test = self.bind_pattern(&arm.x.pattern, selected.clone())?;
            if let Some((entry, _)) = &entry {
                let selected = if index + 1 == arms.len() {
                    remaining.clone()
                } else {
                    logical(span, LogicalOp::And, remaining.clone(), test.clone())?
                };
                self.exits.as_mut().unwrap().active =
                    logical(span, LogicalOp::And, entry.clone(), selected)?;
            }
            let mut body = self.expr(&arm.x.body)?;
            if unit_result {
                body.value = None;
            }
            if let Some(exits) = &self.exits {
                alive = logical(span, LogicalOp::Or, alive, exits.active.clone())?;
                remaining =
                    logical(span, LogicalOp::And, remaining, exits::not(span, test.clone())?)?;
            }
            branches.push((test, body));
        }
        self.locals = saved;
        if let Some((entry, count)) = entry {
            let exits = self.exits.as_mut().unwrap();
            exits.active = if exits.values.len() == count { entry } else { alive };
        }
        // rustc has checked exhaustiveness. Match the verifier's lowering:
        // earlier arms take priority and the final arm is unconditional.
        let (_, mut summary) = branches.pop().unwrap();
        while let Some((test, branch)) = branches.pop() {
            summary = Summary {
                safe: conditional(span, test.clone(), branch.safe, summary.safe)?,
                value: match (branch.value, summary.value) {
                    (Some(a), Some(b)) => Some(conditional(span, test, a, b)?),
                    (None, None) => None,
                    _ => return Err(unsupported(span, "match result shapes differ")),
                },
            };
        }
        summary.safe = logical(span, LogicalOp::And, scrutinee.safe, summary.safe)?;
        if unit_result {
            summary.value = Some(exits::unit_value(span)?);
        }
        Ok(summary)
    }
}
