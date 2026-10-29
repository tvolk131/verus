//! Complete tags and guarded payload views for collection-bearing value enums.
use super::*;

pub(super) type Branch = (Ident, Term, Option<Vec<(FieldOpr, Val)>>);

pub(super) fn has_heap_enum(
    t: &Typ,
    all: &HashMap<Fun, Function>,
    ds: &[Datatype],
    depth: usize,
) -> bool {
    if depth == 0 {
        return false;
    }
    let t = base(t);
    records::shape::variants(t, ds).is_some_and(|vs| {
        vs.iter().any(|(_, fs)| fs.iter().any(|(_, t)| heap_known(t, all, ds, depth - 1)))
    }) || sums::result(t).is_some_and(|(a, b)| {
        has_heap_enum(a, all, ds, depth - 1) || has_heap_enum(b, all, ds, depth - 1)
    }) || option(t).is_some_and(|t| has_heap_enum(t, all, ds, depth - 1))
}

impl Engine<'_> {
    pub(super) fn variants_layout(&self, e: &Expr, depth: usize) -> Result<Option<Node>, VirErr> {
        if !has_heap_enum(&e.typ, self.all, self.ds, 8) {
            return Ok(None);
        }
        let Some(vs) = records::shape::variants(&e.typ, self.ds) else {
            return Ok(None);
        };
        let TypX::Datatype(dt, ..) = &*e.typ else { unreachable!() };
        Ok(Some(Node::Variants(
            vs.into_iter()
                .map(|(variant, fs)| {
                    let tag = variant_test(&e.span, atom(e.clone())?, dt, &variant)?;
                    let fields = fs
                        .into_iter()
                        .map(|(field, t)| {
                            let op = FieldOpr {
                                datatype: dt.clone(),
                                variant: variant.clone(),
                                field,
                                get_variant: false,
                                check: VariantCheck::None,
                            };
                            Ok((op.clone(), self.layout(field_expr(e, &op, &t), depth - 1)?))
                        })
                        .collect::<Result<Vec<_>, VirErr>>()?;
                    Ok((variant, tag, Some(fields)))
                })
                .collect::<Result<Vec<_>, VirErr>>()?,
        )))
    }
    pub(super) fn variants_ctor(
        &mut self,
        s: &mut State,
        e: &Expr,
        variant: &Ident,
        fields: &air::ast::Binders<Expr>,
    ) -> Result<Val, VirErr> {
        let Some(Node::Variants(branches)) = self.variants_layout(
            &SpannedTyped::new(&e.span, &e.typ, ExprX::Var(self.f.x.inner_ret.x.name.clone())),
            8,
        )?
        else {
            return Err(unsupported(&e.span, "structured enum constructor layout"));
        };
        let branches = branches
            .into_iter()
            .map(|(v, _, layout)| {
                let payload = if &v == variant {
                    let layout = layout.unwrap();
                    if fields.len() != layout.len() {
                        return Err(unsupported(&e.span, "structured enum constructor fields"));
                    }
                    Some(
                        layout
                            .into_iter()
                            .map(|(op, _)| {
                                let field = fields.iter().find(|f| f.name == op.field).ok_or_else(
                                    || unsupported(&e.span, "missing structured enum field"),
                                )?;
                                Ok((op, self.expr(s, &field.a)?))
                            })
                            .collect::<Result<Vec<_>, VirErr>>()?,
                    )
                } else {
                    None
                };
                Ok((v.clone(), boolean(&e.span, &v == variant), payload))
            })
            .collect::<Result<Vec<_>, VirErr>>()?;
        Ok(Val { typ: e.typ.clone(), node: Node::Variants(branches) })
    }
    pub(super) fn variants_post(&self, branches: &[Branch], out: &Expr) -> Result<Term, VirErr> {
        let TypX::Datatype(dt, ..) = &*out.typ else { unreachable!() };
        let mut post = boolean(&out.span, true);
        for (variant, tag, fields) in branches {
            let test = variant_test(&out.span, atom(out.clone())?, dt, variant)?;
            post = logical(
                &out.span,
                LogicalOp::And,
                post,
                records::equal(&out.span, test.clone(), tag.clone())?,
            )?;
            if let Some(fields) = fields {
                let mut payload = boolean(&out.span, true);
                for (op, v) in fields {
                    payload = logical(
                        &out.span,
                        LogicalOp::And,
                        payload,
                        self.post(v, &field_expr(out, op, &v.typ))?,
                    )?;
                }
                post = logical(
                    &out.span,
                    LogicalOp::And,
                    post,
                    logical(&out.span, LogicalOp::Implies, test, payload)?,
                )?;
            }
        }
        Ok(post)
    }
    pub(super) fn variants_decode(
        &mut self,
        s: &mut State,
        out: &Expr,
        layout: &[Branch],
        clauses: &[Expr],
    ) -> Result<Node, VirErr> {
        let TypX::Datatype(dt, ..) = &*out.typ else { unreachable!() };
        Ok(Node::Variants(
            layout
                .iter()
                .map(|(variant, _, fs)| {
                    let test = variant_test(&out.span, atom(out.clone())?, dt, variant)?;
                    let rhs = find_rhs(&test.expr, clauses)?;
                    let tag = self.formula(s, &rhs)?;
                    let q = clauses.iter().find_map(|c| match &c.x {
                        ExprX::Logical(LogicalOp::Implies, g, q)
                            if skeleton(g) == skeleton(&test.expr) =>
                        {
                            Some(q)
                        }
                        _ => None,
                    });
                    let fields = if let Some(q) = q {
                        let mut cs = vec![];
                        split(q, &mut cs);
                        Some(
                            fs.as_ref()
                                .unwrap()
                                .iter()
                                .map(|(op, v)| {
                                    Ok((
                                        op.clone(),
                                        self.decode_parts(s, &field_expr(out, op, &v.typ), v, &cs)?,
                                    ))
                                })
                                .collect::<Result<Vec<_>, VirErr>>()?,
                        )
                    } else if is_bool(&tag, false) {
                        None
                    } else {
                        return Err(unsupported(&out.span, "missing structured enum payload"));
                    };
                    Ok((variant.clone(), tag, fields))
                })
                .collect::<Result<Vec<_>, VirErr>>()?,
        ))
    }
}
