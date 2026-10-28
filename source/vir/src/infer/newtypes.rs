//! Candidate-only scalar projection for transparent, one-field Rust wrappers.
//!
//! The projected functions are never installed or verified in place of source.
//! Lift their formulas back to the source types, then check all three obligations
//! against the original bodies. No operator is interpreted by its trait name.
use super::*;
use crate::ast_visitor::{expr_visitor_check, map_expr_visitor_env};

#[derive(Default)]
pub(super) struct Newtypes {
    shapes: std::collections::BTreeMap<Dt, Datatype>,
}

impl Newtypes {
    pub(super) fn new(datatypes: &[Datatype]) -> Self {
        let shapes = datatypes
            .iter()
            .filter(|d| {
                d.x.mode == Mode::Exec
                    && d.x.proxy.is_none()
                    && matches!(d.x.transparency, DatatypeTransparency::WhenVisible(_))
                    && d.x.typ_params.is_empty()
                    && d.x.typ_bounds.is_empty()
                    && d.x.user_defined_invariant_fn.is_none()
                    && !d.x.destructor
                    && d.x.variants.len() == 1
                    && d.x.variants[0].fields.len() == 1
                    && d.x.variants[0].fields[0].a.1 == Mode::Exec
                    && scalar(&d.x.variants[0].fields[0].a.0)
            })
            .map(|d| (d.x.name.clone(), d.clone()))
            .collect();
        Self { shapes }
    }

    pub(super) fn fingerprint(&self) -> String {
        format!("{:?}", self.shapes)
    }

    pub(super) fn wrapper(&self, typ: &Typ) -> bool {
        match &**typ {
            TypX::MutRef(t) => self.shape(t).is_some(),
            _ => self.shape(typ).is_some(),
        }
    }

    fn shape(&self, typ: &Typ) -> Option<&Datatype> {
        match &**typ {
            TypX::Datatype(dt, ts, _) if ts.is_empty() => self.shapes.get(dt),
            TypX::Decorate(TypDecoration::Ref, None, inner) => self.shape(inner),
            _ => None,
        }
    }

    fn typ(&self, typ: &Typ) -> Typ {
        self.shape(typ).map_or_else(|| typ.clone(), |d| d.x.variants[0].fields[0].a.0.clone())
    }

    fn field(&self, op: &FieldOpr) -> bool {
        self.shapes.get(&op.datatype).is_some_and(|d| {
            op.variant == d.x.variants[0].name
                && op.field == d.x.variants[0].fields[0].name
                && !op.get_variant
                && op.check == VariantCheck::None
        })
    }

    fn uses_wrapper(&self, f: &Function) -> bool {
        if f.x.params.iter().any(|p| self.shape(&p.x.typ).is_some())
            || self.shape(&f.x.inner_ret.x.typ).is_some()
        {
            return true;
        }
        let mut found = false;
        if let Some(body) = &f.x.body {
            crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
                found |= self.shape(&e.typ).is_some();
                if found {
                    crate::ast_visitor::VisitorControlFlow::Return
                } else {
                    crate::ast_visitor::VisitorControlFlow::Recurse
                }
            });
        }
        found
    }

    pub(super) fn project(&self, f: &Function) -> Result<Function, VirErr> {
        if !f.x.attrs.infer || f.x.attrs.infer_allow_approx || records::candidate(f) && !f.x.params.first().is_some_and(|p|self.shape(&p.x.typ).is_some()) || !self.uses_wrapper(f) {
            return Ok(f.clone());
        }
        if !matches!(f.x.kind, FunctionKind::Static)
            && wrapper_traits::kind(f) != Some(wrapper_traits::Kind::From)
        {
            return Err(unsupported(
                &f.span,
                "inference on trait implementations: their preconditions are inherited from the trait; use an inherent method or explicit operator specifications",
            ));
        }
        if self.shape(&f.x.inner_ret.x.typ).is_some()
            && !matches!(&*f.x.inner_ret.x.typ, TypX::Datatype(..))
        {
            return Err(unsupported(&f.span, "returning wrapper references"));
        }
        let mut projected = f.x.clone();
        projected.kind = FunctionKind::Static;
        let param = |p: &Param| {
            let mut x = p.x.clone();
            x.typ = self.typ(&x.typ);
            p.new_x(x)
        };
        projected.params = Arc::new(f.x.params.iter().map(param).collect());
        projected.inner_ret = param(&f.x.inner_ret);
        projected.body = f.x.body.as_ref().map(|b| self.expr(b)).transpose()?;
        projected.require =
            Arc::new(f.x.require.iter().map(|e| self.expr(e)).collect::<Result<_, _>>()?);
        projected.ensure.0 =
            Arc::new(f.x.ensure.0.iter().map(|e| self.expr(e)).collect::<Result<_, _>>()?);
        Ok(f.new_x(projected))
    }

    fn expr(&self, expr: &Expr) -> Result<Expr, VirErr> {
        let mut remaining = FORMULA_BUDGET;
        expr_visitor_check(expr, &mut |_, e| {
            remaining = remaining
                .checked_sub(1)
                .ok_or_else(|| unsupported(&e.span, "wrapper projection budget exceeded"))?;
            if let ExprX::Ctor(dt, variant, fields, update) = &e.x {
                if let Some(d) = self.shapes.get(dt) {
                    if update.is_some()
                        || variant != &d.x.variants[0].name
                        || fields.len() != 1
                        || fields[0].name != d.x.variants[0].fields[0].name
                    {
                        return Err(unsupported(&e.span, "wrapper constructor shape"));
                    }
                }
            }
            Ok(())
        })?;
        map_expr_visitor_env(
            expr,
            &mut crate::ast_visitor::VisitorScopeMap::new(),
            &mut (),
            &|_, _, e| {
                match &e.x {
                    ExprX::Ctor(dt, _, fs, None) if self.shapes.contains_key(dt) => {
                        return Ok(fs[0].a.clone());
                    }
                    ExprX::UnaryOpr(UnaryOpr::Field(op), base) if self.field(op) => {
                        return Ok(base.clone());
                    }
                    _ => {}
                }
                Ok(SpannedTyped::new(&e.span, &self.typ(&e.typ), e.x.clone()))
            },
            &|_, _, s| Ok(vec![s.clone()]),
            &|_, t| Ok(t.clone()),
            &|_, _, p| {
                if let PlaceX::Field(op, base) = &p.x {
                    if self.field(op) {
                        return Ok(base.clone());
                    }
                }
                Ok(SpannedTyped::new(&p.span, &self.typ(&p.typ), p.x.clone()))
            },
            &|_, _, p| {
                let x = if let PatternX::Var(b) = &p.x {
                    let mut b = b.clone();
                    b.typ = self.typ(&b.typ);
                    PatternX::Var(b)
                } else {
                    p.x.clone()
                };
                Ok(SpannedTyped::new(&p.span, &self.typ(&p.typ), x))
            },
        )
    }

    pub(super) fn lift(
        &self,
        original: &Function,
        projected: &Function,
    ) -> Result<Contract, VirErr> {
        let span = &original.span;
        let vars = original
            .x
            .params
            .iter()
            .map(|p| {
                let expr = SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone()));
                let expr = if let Some(d) = self.shape(&p.x.typ) {
                    SpannedTyped::new(
                        &p.span,
                        &self.typ(&p.x.typ),
                        ExprX::UnaryOpr(
                            UnaryOpr::Field(FieldOpr {
                                datatype: d.x.name.clone(),
                                variant: d.x.variants[0].name.clone(),
                                field: d.x.variants[0].fields[0].name.clone(),
                                get_variant: false,
                                check: VariantCheck::None,
                            }),
                            expr,
                        ),
                    )
                } else {
                    expr
                };
                (p.x.name.clone(), Term { view: None, expr, size: 2 })
            })
            .collect();
        let pre = instantiate(&projected.x.require[0], &vars)?;
        let ExprX::Binary(BinaryOp::Eq(Mode::Spec), _, output) = &projected.x.ensure.0[0].x else {
            return Err(unsupported(span, "invalid projected wrapper summary"));
        };
        let mut output = instantiate(output, &vars)?;
        if let Some(d) = self.shape(&original.x.inner_ret.x.typ) {
            // Returning references is not supported, even though shared wrapper
            // receivers can be read as scalar inputs.
            if !matches!(&*original.x.inner_ret.x.typ, TypX::Datatype(..)) {
                return Err(unsupported(span, "returning wrapper references"));
            }
            let field_typ = &d.x.variants[0].fields[0].a.0;
            if !crate::ast_util::types_equal(&output.expr.typ, field_typ) {
                let TypX::Int(range) = &**field_typ else {
                    return Err(unsupported(span, "wrapper payload type"));
                };
                output =
                    summary::unary(span, UnaryOp::Clip { range: *range, truncate: true }, output)?;
            }
            output = term(
                span,
                &original.x.inner_ret.x.typ,
                ExprX::Ctor(
                    d.x.name.clone(),
                    d.x.variants[0].name.clone(),
                    Arc::new(vec![Arc::new(air::ast::BinderX {
                        name: d.x.variants[0].fields[0].name.clone(),
                        a: output.expr,
                    })]),
                    None,
                ),
                output.size + 1,
            )?;
        }
        let result = SpannedTyped::new(
            span,
            &original.x.inner_ret.x.typ,
            ExprX::Var(original.x.inner_ret.x.name.clone()),
        );
        Contract::new(
            pre.expr,
            SpannedTyped::new(
                span,
                &bool_typ(),
                ExprX::Binary(BinaryOp::Eq(Mode::Spec), result, output.expr),
            ),
        )
    }
}
