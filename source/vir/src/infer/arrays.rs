//! Closed immutable array support. Summaries use Verus's existing array index
//! operation; the original executable supplies the independent bounds checks.
use super::*;
use num_traits::ToPrimitive;

pub(super) const MAX_LEN: usize = 64;

pub(super) fn shape(typ: &Typ) -> Option<(Typ, usize)> {
    let typ = readonly_base(typ);
    let TypX::Primitive(Primitive::Array, args) = &**typ else { return None };
    if args.len() != 2 || !scalar(&args[0]) {
        return None;
    }
    let TypX::ConstInt(length) = &*args[1] else { return None };
    let length = length.to_usize()?;
    (length <= MAX_LEN).then(|| (args[0].clone(), length))
}

/// Shared references have the same immutable value encoding as their referent.
/// Never erase mutable references, boxes, or other decorations here.
pub(super) fn readonly_base(mut typ: &Typ) -> &Typ {
    while let TypX::Decorate(TypDecoration::Ref, None, inner) = &**typ {
        typ = inner;
    }
    typ
}

pub(super) fn input_type(typ: &Typ) -> bool {
    match &**typ {
        TypX::Primitive(Primitive::Array, _) => shape(typ).is_some(),
        TypX::Decorate(TypDecoration::Ref, None, inner)
            if matches!(&**inner, TypX::Primitive(Primitive::Array, _)) =>
        {
            shape(inner).is_some()
        }
        _ => false,
    }
}

pub(super) fn compatible(a: &Typ, b: &Typ) -> bool {
    match (shape(a), shape(b)) {
        (Some((a, n)), Some((b, m))) => n == m && crate::ast_util::types_equal(&a, &b),
        _ => false,
    }
}

pub(super) fn typ(element: Typ, length: usize) -> Option<Typ> {
    let typ = Arc::new(TypX::Primitive(
        Primitive::Array,
        Arc::new(vec![element, Arc::new(TypX::ConstInt(length.into()))]),
    ));
    shape(&typ).map(|_| typ)
}

pub(super) fn index_type(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Int(IntRange::USize))
}

pub(super) fn literal(span: &Span, typ: &Typ, elements: Vec<Term>) -> Result<Term, VirErr> {
    let (element_type, length) = shape(typ)
        .ok_or_else(|| unsupported(span, "arrays require at most 64 fixed scalar elements"))?;
    if elements.len() != length {
        return Err(unsupported(span, "array literal length mismatch"));
    }
    let mut values = Vec::new();
    let mut size = 1;
    for mut element in elements {
        if !enums::compatible(&element.expr.typ, &element_type) {
            return Err(unsupported(span, "array element type mismatch"));
        }
        if !crate::ast_util::types_equal(&element.expr.typ, &element_type) {
            let TypX::Int(range @ (IntRange::U(_) | IntRange::I(_))) = &*element_type else {
                return Err(unsupported(span, "array element type mismatch"));
            };
            element =
                summary::unary(span, UnaryOp::Clip { range: *range, truncate: true }, element)?;
        }
        size += element.size;
        values.push(element.expr);
    }
    term(span, typ, ExprX::ArrayLiteral(Arc::new(values)), size)
}

pub(super) fn index(span: &Span, array: Term, index: Term) -> Result<Term, VirErr> {
    let (element_type, _) = shape(&array.expr.typ)
        .ok_or_else(|| unsupported(span, "indexing requires a supported fixed-size array"))?;
    if !eligibility::integer(&index.expr.typ) {
        return Err(unsupported(span, "noninteger array index"));
    }
    term(
        span,
        &element_type,
        ExprX::Binary(
            BinaryOp::Index(ArrayKind::Array, BoundsCheck::Allow),
            array.expr,
            index.expr,
        ),
        1 + array.size + index.size,
    )
}

impl Engine<'_> {
    pub(super) fn array_index(
        &mut self,
        span: &Span,
        base: &Place,
        index: &Expr,
    ) -> Result<Summary, VirErr> {
        let array = self.place(base)?;
        let index = self.expr(index)?;
        self.index_summary(span, array, index)
    }

    pub(super) fn index_summary(
        &self,
        span: &Span,
        array: Summary,
        index: Summary,
    ) -> Result<Summary, VirErr> {
        let a = value(&array, span)?;
        let i = value(&index, span)?;
        let safe = logical(span, LogicalOp::And, array.safe, index.safe)?;
        let bounds = self.require_active(span, slices::bound(span, a.clone(), i.clone())?)?;
        let safe = logical(span, LogicalOp::And, safe, bounds)?;
        Ok(Summary { safe, value: Some(slices::index(span, a, i)?) })
    }
}
