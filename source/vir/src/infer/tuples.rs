//! Flat scalar pair values. No new input types, axioms or observations.
use super::*;

pub(super) fn shape(typ: &Typ) -> Option<&Typs> {
    match &**typ {
        TypX::Datatype(Dt::Tuple(2), ts, _) if ts.len() == 2 && ts.iter().all(scalar) => Some(ts),
        _ => None,
    }
}

pub(super) fn valid_ctor(typ: &Typ, dt: &Dt, variant: &Ident, fields: &Binders<Expr>) -> bool {
    shape(typ).is_some_and(|ts| {
        *dt == Dt::Tuple(2)
            && *variant == crate::def::prefix_tuple_variant(2)
            && fields.len() == 2
            && fields.iter().enumerate().all(|(i, f)| {
                f.name == crate::def::positional_field_ident(i)
                    && crate::ast_util::types_equal(&f.a.typ, &ts[i])
            })
    })
}

pub(super) fn ctor(span: &Span, typ: &Typ, values: Vec<Term>) -> Result<Term, VirErr> {
    let ts = shape(typ).ok_or_else(|| unsupported(span, "expected flat scalar pair"))?;
    if values.len() != 2 {
        return Err(unsupported(span, "pair arity mismatch"));
    }
    let mut size = 1;
    let mut fields = Vec::new();
    for (i, mut value) in values.into_iter().enumerate() {
        if value.view.is_some() || !enums::compatible(&value.expr.typ, &ts[i]) {
            return Err(unsupported(span, "pair component type mismatch"));
        }
        // Mathematical arithmetic is lifted back to the exact source scalar type,
        // just as for existing scalar array/enum payloads. Safety is proved separately.
        if !crate::ast_util::types_equal(&value.expr.typ, &ts[i]) {
            let TypX::Int(range) = &*ts[i] else {
                return Err(unsupported(span, "pair scalar type"));
            };
            value = summary::unary(span, UnaryOp::Clip { range: *range, truncate: true }, value)?;
        }
        size += value.size;
        fields.push(Arc::new(air::ast::BinderX {
            name: crate::def::positional_field_ident(i),
            a: value.expr,
        }));
    }
    term(
        span,
        typ,
        ExprX::Ctor(Dt::Tuple(2), crate::def::prefix_tuple_variant(2), Arc::new(fields), None),
        size,
    )
}

pub(super) fn index(op: &FieldOpr, typ: &Typ) -> Option<usize> {
    shape(typ)?;
    if op.datatype != Dt::Tuple(2)
        || op.variant != crate::def::prefix_tuple_variant(2)
        || op.get_variant
        || op.check != VariantCheck::None
    {
        return None;
    }
    (0..2).find(|i| op.field == crate::def::positional_field_ident(*i))
}

pub(super) fn field(span: &Span, value: Term, i: usize) -> Result<Term, VirErr> {
    let ts = shape(&value.expr.typ).ok_or_else(|| unsupported(span, "expected pair projection"))?;
    let typ = ts.get(i).ok_or_else(|| unsupported(span, "pair projection index"))?.clone();
    if let ExprX::Ctor(dt, variant, fields, None) = &value.expr.x {
        if valid_ctor(&value.expr.typ, dt, variant, fields) {
            return term(span, &typ, fields[i].a.x.clone(), value.size);
        }
    }
    let op = FieldOpr {
        datatype: Dt::Tuple(2),
        variant: crate::def::prefix_tuple_variant(2),
        field: crate::def::positional_field_ident(i),
        get_variant: false,
        check: VariantCheck::None,
    };
    term(span, &typ, ExprX::UnaryOpr(UnaryOpr::Field(op), value.expr), 1 + value.size)
}
