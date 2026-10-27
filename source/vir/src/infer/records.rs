//! Bounded symbolic effects and queries for one borrowed transparent record.
//! Candidates describe every field, and are checked against the original body.
//! No invariant is assumed or discovered by this untrusted synthesizer.
use super::*;
use collections::Model;

mod calls;
pub(super) mod shape;
use shape::{Field, Shape, shape, value_type};

fn same(a: &Typ, b: &Typ) -> bool {
    crate::ast_util::types_equal(a, b)
}

pub(super) fn mutable(f: &Function) -> bool {
    f.x.params.first().is_some_and(|p| matches!(&*p.x.typ, TypX::MutRef(_)))
}
pub(super) fn candidate(f: &Function) -> bool {
    shape::candidate(f)
}

fn validate_signature(f: &Function, datatypes: &[Datatype], all: &HashMap<Fun,Function>) -> Result<Shape, VirErr> {
    let shape = shape(f, datatypes, all)?;
    let x = &f.x;
    if x.mode != Mode::Exec
        || !matches!(x.kind, FunctionKind::Static)
        || !matches!(x.item_kind, ItemKind::Function)
        || x.inner_ret.x.mode != Mode::Exec
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.attrs.bit_vector
        || x.attrs.integer_ring
        || x.attrs.broadcast_forall
        || x.proxy.is_some()
        || x.attrs.autospec.is_some()
        || x.params.iter().any(|p| p.x.mode != Mode::Exec || p.x.user_mut)
        || !same(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
        || !x.require.is_empty()
        || !x.ensure.0.is_empty()
        || !x.ensure.1.is_empty()
        || x.returns.is_some()
        || !x.decrease.is_empty()
        || x.decrease_by.is_some()
        || x.decrease_when.is_some()
    {
        return Err(unsupported(
            &f.span,
            "record state inference requires no handwritten contracts, proof settings or decreases",
        ));
    }
    Ok(shape)
}

pub(super) fn atom(e: Expr) -> Result<Term, VirErr> {
    let mut size = 0;
    crate::ast_visitor::expr_visitor_walk(&e, &mut |_| {
        size += 1;
        if size > FORMULA_BUDGET {
            crate::ast_visitor::VisitorControlFlow::Stop(())
        } else {
            crate::ast_visitor::VisitorControlFlow::Recurse
        }
    });
    term(&e.span, &e.typ, e.x.clone(), size)
}
fn coerce(span: &Span, value: Term, typ: &Typ) -> Result<Term, VirErr> {
    if same(&value.expr.typ, typ) {
        return Ok(value);
    }
    let unwrap = |t: &Typ| {
        let mut t = t.clone();
        while let TypX::Decorate(TypDecoration::Ref, None, inner) = &*t {
            t = inner.clone();
        }
        t
    };
    if same(&unwrap(&value.expr.typ), &unwrap(typ)) {
        let p = SpannedTyped::new(span, &value.expr.typ, PlaceX::Temporary(value.expr.clone()));
        return atom(SpannedTyped::new(
            span,
            typ,
            ExprX::ReadPlace(
                p,
                UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
            ),
        ));
    }
    if let TypX::Int(r) = &**typ {
        if scalar(typ) && eligibility::integer(&value.expr.typ) {
            return summary::unary(span, UnaryOp::Clip { range: *r, truncate: true }, value);
        }
    }
    Err(unsupported(span, "record value type or reference alias"))
}
fn equal(span: &Span, a: Term, b: Term) -> Result<Term, VirErr> {
    term(
        span,
        &bool_typ(),
        ExprX::Binary(BinaryOp::Eq(Mode::Spec), a.expr, b.expr),
        1 + a.size + b.size,
    )
}
fn field_value(f: &Function, shape: &Shape, field: &Field, future: bool) -> Result<Term, VirErr> {
    let span = &f.span;
    let mut value =
        SpannedTyped::new(span, &f.x.params[0].x.typ, ExprX::Var(f.x.params[0].x.name.clone()));
    if mutable(f) {
        if !future {
            value = SpannedTyped::new(span, &value.typ, ExprX::Old(value.clone()));
        }
        value = SpannedTyped::new(
            span,
            &shape.typ,
            ExprX::Unary(
                if future {
                    UnaryOp::MutRefFuture(MutRefFutureSourceName::Final)
                } else {
                    UnaryOp::MutRefCurrent
                },
                value,
            ),
        );
    }
    for step in &field.path {
        value = SpannedTyped::new(
            span,
            &step.typ,
            ExprX::UnaryOpr(UnaryOpr::Field(step.op.clone()), value),
        );
    }
    atom(value)
}

#[derive(Clone)]
struct State {
    fields: Vec<Term>,
    locals: HashMap<VarIdent, Term>,
    active: Term,
}
struct Interpreter<'a> {
    function: &'a Function,
    shape: Shape,
    all: &'a HashMap<Fun, Function>,
    datatypes: &'a [Datatype],
    requirements: Vec<Term>,
    remaining: usize,
}

impl Interpreter<'_> {
    fn spend(&mut self, span: &Span) -> Result<(), VirErr> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| error(span, "Verus Infer incomplete: record source budget exceeded"))?;
        Ok(())
    }
    fn require(&mut self, state: &State, span: &Span, condition: Term) -> Result<(), VirErr> {
        self.requirements.push(conditional(
            span,
            state.active.clone(),
            condition,
            boolean(span, true),
        )?);
        Ok(())
    }
    fn path(&self, p: &Place) -> Option<Vec<FieldOpr>> {
        match &p.x {
            PlaceX::Local(n)
                if !mutable(self.function) && n == &self.function.x.params[0].x.name =>
            {
                Some(vec![])
            }
            PlaceX::DerefMut(base)
                if mutable(self.function)
                    && matches!(&base.x,PlaceX::Local(n) if n==&self.function.x.params[0].x.name) =>
            {
                Some(vec![])
            }
            PlaceX::Field(op, base) if op.check == VariantCheck::None && !op.get_variant => {
                let mut path = self.path(base)?;
                path.push(op.clone());
                Some(path)
            }
            _ => None,
        }
    }
    fn field(&self, p: &Place) -> Option<usize> {
        let path = self.path(p)?;
        self.shape.fields.iter().position(|f| {
            let mut physical=f.path.iter().map(|p|p.op.clone()).collect::<Vec<_>>();
            if let Some(v)=&f.view { physical.push(v.storage.clone()); }
            shape::same_path(&path, &physical)
        })
    }
    fn vector(&self, e: &Expr) -> Option<usize> {
        let p = match &e.x {
            ExprX::ReadPlace(p, _) | ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) => p,
            _ => return None,
        };
        let i = self.field(p)?;
        self.shape.fields[i].element.as_ref()?;
        Some(i)
    }
    fn model_call(
        &self,
        span: &Span,
        model: Model,
        types: Vec<Typ>,
        values: Vec<Term>,
    ) -> Result<Term, VirErr> {
        let mut found = self.all.values().filter(|f| collections::model(f) == Some(model));
        let f = found.next().ok_or_else(|| {
            unsupported(span, "record effects require canonical vstd Vec/Seq models")
        })?;
        if found.next().is_some() {
            return Err(unsupported(span, "ambiguous record collection model"));
        }
        let substitutions = f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
        let ret = crate::sst_util::subst_typ(&substitutions, &f.x.inner_ret.x.typ);
        let args = values
            .iter()
            .zip(f.x.params.iter())
            .map(|(v, p)| {
                let typ = crate::sst_util::subst_typ(&substitutions, &p.x.typ);
                if matches!(&*typ, TypX::Decorate(TypDecoration::Ref, None, _)) {
                    let place =
                        SpannedTyped::new(span, &v.expr.typ, PlaceX::Temporary(v.expr.clone()));
                    SpannedTyped::new(
                        span,
                        &typ,
                        ExprX::ReadPlace(
                            place,
                            UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
                        ),
                    )
                } else {
                    v.expr.clone()
                }
            })
            .collect();
        let size = 1 + values.iter().map(|v| v.size).sum::<usize>();
        let target =
            if let FunctionKind::TraitMethodImpl { method, impl_path, trait_typ_args, .. } =
                &f.x.kind
            {
                let impls = values
                    .first()
                    .and_then(|v| match &*v.expr.typ {
                        TypX::Datatype(_, _, impls) => Some(impls.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| Arc::new(vec![]));
                CallTarget::Fun(
                    CallTargetKind::DynamicResolved {
                        resolved: f.x.name.clone(),
                        typs: Arc::new(types.clone()),
                        impl_paths: impls.clone(),
                        is_trait_default: false,
                    },
                    method.clone(),
                    Arc::new(
                        trait_typ_args
                            .iter()
                            .map(|t| crate::sst_util::subst_typ(&substitutions, t))
                            .collect(),
                    ),
                    Arc::new(
                        std::iter::once(ImplPath::TraitImplPath(impl_path.clone()))
                            .chain(impls.iter().cloned())
                            .collect(),
                    ),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                )
            } else {
                CallTarget::Fun(
                    CallTargetKind::Static,
                    f.x.name.clone(),
                    Arc::new(types),
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                )
            };
        term(
            span,
            &ret,
            ExprX::Call { target, args: Arc::new(args), post_args: None, body: None },
            size,
        )
    }
    fn view(&self, field: usize, future: bool) -> Result<Term, VirErr> {
        let f = &self.shape.fields[field];
        let value = field_value(self.function, &self.shape, f, future)?;
        if let Some(v)=&f.view { return sequence_views::call(v,value.expr); }
        self.model_call(
            &self.function.span,
            Model::View,
            (**collections::vec_args(&f.typ).unwrap()).clone(),
            vec![value],
        )
    }
    fn len(&self, field: usize) -> Result<Term, VirErr> {
        let f = &self.shape.fields[field];
        if f.view.is_some() {
            let len=self.model_call(&self.function.span,Model::SeqLen,vec![f.element.clone().unwrap()],vec![self.view(field,false)?])?;
            return term(&self.function.span,&int_typ(),ExprX::Unary(UnaryOp::Clip{range:IntRange::Int,truncate:false},len.expr),1+len.size);
        }
        self.model_call(
            &self.function.span,
            Model::SpecLen,
            (**collections::vec_args(&f.typ).unwrap()).clone(),
            vec![field_value(self.function, &self.shape, f, false)?],
        )
    }
    fn readonly(&mut self, s: &mut State, e: &Expr) -> Result<Term, VirErr> {
        let before: Vec<_> = s.fields.iter().map(|v| v.expr.clone()).collect();
        let active = s.active.expr.clone();
        let result = self.expr(s, e)?;
        if !Arc::ptr_eq(&active, &s.active.expr)
            || !before.iter().zip(&s.fields).all(|(a, b)| Arc::ptr_eq(a, &b.expr))
        {
            return Err(unsupported(
                &e.span,
                "side effects inside record values or assignment indices",
            ));
        }
        Ok(result)
    }
    fn place(&mut self, s: &mut State, p: &Place) -> Result<Term, VirErr> {
        if let Some(i) = self.field(p) {
            if self.shape.fields[i].element.is_some() {
                return Err(unsupported(&p.span, "vector aliases outside direct model calls"));
            }
            return Ok(s.fields[i].clone());
        }
        match &p.x {
            PlaceX::Local(n) => s
                .locals
                .get(n)
                .cloned()
                .ok_or_else(|| unsupported(&p.span, "record aliases and uninitialized locals")),
            PlaceX::Temporary(e) => self.expr(s, e),
            PlaceX::Field(op, base) if op.check == VariantCheck::None && !op.get_variant => {
                let base = self.place(s, base)?;
                self.project(&p.span, base, op)
            }
            _ => Err(unsupported(
                &p.span,
                "record place outside scalar fields or direct vector indexing",
            )),
        }
    }
    fn project(&self, span: &Span, value: Term, op: &FieldOpr) -> Result<Term, VirErr> {
        let fields =
            constructor_fields(span, &value.expr.typ, &op.datatype, &op.variant, self.datatypes)?;
        let typ = fields
            .into_iter()
            .find(|(n, _)| n == &op.field)
            .map(|(_, t)| t)
            .ok_or_else(|| unsupported(span, "record tuple/Option field"))?;
        term(span, &typ, ExprX::UnaryOpr(UnaryOpr::Field(op.clone()), value.expr), 1 + value.size)
    }
    fn pattern(&mut self, s: &mut State, p: &Pattern, value: Term) -> Result<Term, VirErr> {
        self.spend(&p.span)?;
        match &p.x {
            PatternX::Wildcard => Ok(boolean(&p.span, true)),
            PatternX::ImmutRef(inner) => self.pattern(s, inner, value),
            PatternX::Var(b)
                if matches!(b.by_ref, ByRef::No | ByRef::ImmutRef)
                    && b.user_mut != Some(true)
                    && value_type(&b.typ, &self.function.x.typ_params, self.datatypes, 4) =>
            {
                s.locals.insert(b.name.clone(), coerce(&p.span, value, &b.typ)?);
                Ok(boolean(&p.span, true))
            }
            PatternX::Constructor(dt, variant, fields) => {
                let expected =
                    constructor_fields(&p.span, &value.expr.typ, dt, variant, self.datatypes)?;
                let mut condition = if matches!(dt, Dt::Tuple(_)) {
                    boolean(&p.span, true)
                } else {
                    term(
                        &p.span,
                        &bool_typ(),
                        ExprX::UnaryOpr(
                            UnaryOpr::IsVariant { datatype: dt.clone(), variant: variant.clone() },
                            value.expr.clone(),
                        ),
                        value.size + 1,
                    )?
                };
                if fields.len() != expected.len() {
                    return Err(unsupported(&p.span, "record match field coverage"));
                }
                for field in fields.iter() {
                    let op = FieldOpr {
                        datatype: dt.clone(),
                        variant: variant.clone(),
                        field: field.name.clone(),
                        get_variant: false,
                        check: VariantCheck::None,
                    };
                    let projected = self.project(&p.span, value.clone(), &op)?;
                    let child = self.pattern(s, &field.a, projected)?;
                    condition = logical(&p.span, LogicalOp::And, condition, child)?;
                }
                Ok(condition)
            }
            _ => Err(unsupported(
                &p.span,
                "record patterns support immutable Copy bindings, tuples and Option constructors",
            )),
        }
    }
    fn expr(&mut self, s: &mut State, e: &Expr) -> Result<Term, VirErr> {
        self.spend(&e.span)?;
        let span = &e.span;
        if let Some(arg) = widening::call(e, self.all) {
            let arg = self.readonly(s, arg)?;
            return coerce(span, arg, &e.typ);
        }
        if let ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, name, types, ..),
            args,
            post_args: None,
            body: None,
        } = &e.x
        {
            if let Some(callee) =
                self.all.get(name).filter(|f| candidate(f) && f.x.attrs.infer).cloned()
            {
                return self.helper_call(s, e, &callee, types, args);
            }
            if self.all.get(name).is_some_and(record_models::as_ref) {
                if args.len() != 1 || types.len() != 1 {
                    return Err(unsupported(span, "Option as_ref shape"));
                }
                let value = self.readonly(s, &args[0])?;
                let variant: Ident = Arc::new("Some".into());
                let op = FieldOpr {
                    datatype: match &*value.expr.typ {
                        TypX::Datatype(dt, ..) => dt.clone(),
                        _ => return Err(unsupported(span, "Option as_ref input")),
                    },
                    variant: variant.clone(),
                    field: crate::def::positional_field_ident(0),
                    get_variant: false,
                    check: VariantCheck::None,
                };
                let payload = self.project(span, value.clone(), &op)?;
                let borrow = SpannedTyped::new(
                    span,
                    &models::reference(payload.expr.typ.clone()),
                    ExprX::ReadPlace(
                        SpannedTyped::new(
                            span,
                            &payload.expr.typ,
                            PlaceX::Temporary(payload.expr.clone()),
                        ),
                        UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
                    ),
                );
                let some = atom(SpannedTyped::new(
                    span,
                    &e.typ,
                    ExprX::Ctor(
                        op.datatype.clone(),
                        variant.clone(),
                        Arc::new(vec![Arc::new(air::ast::BinderX {
                            name: op.field.clone(),
                            a: borrow,
                        })]),
                        None,
                    ),
                ))?;
                let none = atom(SpannedTyped::new(
                    span,
                    &e.typ,
                    ExprX::Ctor(
                        op.datatype.clone(),
                        Arc::new("None".into()),
                        Arc::new(vec![]),
                        None,
                    ),
                ))?;
                let test = atom(SpannedTyped::new(
                    span,
                    &bool_typ(),
                    ExprX::UnaryOpr(
                        UnaryOpr::IsVariant { datatype: op.datatype.clone(), variant },
                        value.expr,
                    ),
                ))?;
                return conditional(span, test, some, none);
            }
        }
        match &e.x {
            ExprX::Const(Constant::Bool(_) | Constant::Int(_)) => atom(e.clone()),
            ExprX::Var(n) => {
                s.locals.get(n).cloned().ok_or_else(|| {
                    unsupported(span, "record value is not a Copy parameter or local")
                })
            }
            ExprX::ReadPlace(p, _) => self.place(s, p),
            ExprX::NeverToAny(inner) if enums::unit(&e.typ) => self.expr(s, inner),
            ExprX::Return(e) => {
                if let Some(e) = e {
                    let v = self.expr(s, e)?;
                    if !enums::unit(&v.expr.typ) {
                        return Err(unsupported(span, "non-unit early returns in record queries"));
                    }
                }
                s.active = boolean(span, false);
                exits::unit_value(span)
            }
            ExprX::Ctor(dt, variant, fields, None)
                if value_type(&e.typ, &self.function.x.typ_params, self.datatypes, 4) =>
            {
                let expected = constructor_fields(span, &e.typ, dt, variant, self.datatypes)?;
                if expected.len() != fields.len() {
                    return Err(unsupported(span, "record value constructor"));
                }
                let mut values = Vec::new();
                let mut size = 1;
                for field in fields.iter() {
                    let typ = expected
                        .iter()
                        .find(|(n, _)| n == &field.name)
                        .map(|(_, t)| t)
                        .ok_or_else(|| unsupported(span, "record constructor field"))?;
                    let v = self.readonly(s, &field.a)?;
                    // Keep mathematical integer equations through value
                    // wrappers. The source body's checked arithmetic/return
                    // types establish the range; a modular cast would erase
                    // useful bounds when a collection view is closed.
                    let v = if eligibility::integer(typ) && eligibility::integer(&v.expr.typ) {
                        v
                    } else { coerce(span, v, typ)? };
                    size += v.size;
                    values
                        .push(Arc::new(air::ast::BinderX { name: field.name.clone(), a: v.expr }));
                }
                term(
                    span,
                    &e.typ,
                    ExprX::Ctor(dt.clone(), variant.clone(), Arc::new(values), None),
                    size,
                )
            }
            ExprX::Unary(UnaryOp::Not, a) => {
                let a = self.expr(s, a)?;
                summary::unary(span, UnaryOp::Not, a)
            }
            ExprX::Unary(op @ UnaryOp::Clip { .. }, a) => {
                let a = self.expr(s, a)?;
                summary::unary(span, *op, a)
            }
            ExprX::UnaryOpr(UnaryOpr::Field(op), a) => {
                let a = self.expr(s, a)?;
                self.project(span, a, op)
            }
            ExprX::Binary(op, a, b) => {
                let a = self.readonly(s, a)?;
                let b = self.readonly(s, b)?;
                self.binary(s, span, *op, a, b)
            }
            ExprX::Logical(op, a, b) => {
                let yes = if *op == LogicalOp::Or {
                    SpannedTyped::new(span, &bool_typ(), ExprX::Const(Constant::Bool(true)))
                } else {
                    b.clone()
                };
                let no = if *op == LogicalOp::Or {
                    b.clone()
                } else {
                    SpannedTyped::new(
                        span,
                        &bool_typ(),
                        ExprX::Const(Constant::Bool(*op == LogicalOp::Implies)),
                    )
                };
                self.expr(s, &e.new_x(ExprX::If(a.clone(), yes, Some(no))))
            }
            ExprX::If(c, yes, no) => {
                let c = self.expr(s, c)?;
                let mut a = s.clone();
                let mut b = s.clone();
                a.active = logical(span, LogicalOp::And, s.active.clone(), c.clone())?;
                b.active =
                    logical(span, LogicalOp::And, s.active.clone(), exits::not(span, c.clone())?)?;
                let active_a = a.active.expr.clone();
                let active_b = b.active.expr.clone();
                let va = self.expr(&mut a, yes)?;
                let vb = if let Some(no) = no {
                    self.expr(&mut b, no)?
                } else {
                    exits::unit_value(span)?
                };
                for i in 0..s.fields.len() {
                    s.fields[i] = if Arc::ptr_eq(&a.fields[i].expr, &b.fields[i].expr) {
                        a.fields[i].clone()
                    } else {
                        conditional(span, c.clone(), a.fields[i].clone(), b.fields[i].clone())?
                    };
                }
                if !Arc::ptr_eq(&active_a, &a.active.expr)
                    || !Arc::ptr_eq(&active_b, &b.active.expr)
                {
                    s.active = logical(span, LogicalOp::Or, a.active, b.active)?;
                }
                conditional(span, c, va, vb)
            }
            ExprX::Block(statements, tail) => {
                let locals = s.locals.clone();
                for stmt in statements.iter() {
                    self.spend(&stmt.span)?;
                    match &stmt.x {
                        StmtX::Expr(e) => {
                            self.expr(s, e)?;
                        }
                        StmtX::Decl {
                            pattern,
                            mode: None | Some((Mode::Exec, DeclProph::Default)),
                            init: Some(p),
                            els: None,
                            ..
                        } => {
                            let PatternX::Var(binding) = &pattern.x else {
                                return Err(unsupported(
                                    &stmt.span,
                                    "record locals require simple immutable Copy bindings",
                                ));
                            };
                            if binding.by_ref != ByRef::No
                                || binding.user_mut == Some(true)
                                || !value_type(
                                    &binding.typ,
                                    &self.function.x.typ_params,
                                    self.datatypes,
                                    4,
                                )
                            {
                                return Err(unsupported(
                                    &stmt.span,
                                    "record local aliases or mutable locals",
                                ));
                            }
                            let v = self.place(s, p)?;
                            s.locals.insert(binding.name.clone(), coerce(span, v, &binding.typ)?);
                        }
                        _ => {
                            return Err(unsupported(
                                &stmt.span,
                                "record statements, proof code or uninitialized locals",
                            ));
                        }
                    }
                }
                let value = if let Some(tail) = tail {
                    self.expr(s, tail)?
                } else {
                    exits::unit_value(span)?
                };
                s.locals = locals;
                Ok(value)
            }
            ExprX::Match(p, arms, _) if arms.len() <= 4 => {
                let value = self.place(s, p)?;
                let mut seen = boolean(span, false);
                let mut results = Vec::new();
                for arm in arms.iter() {
                    let mut branch = s.clone();
                    let p = self.pattern(&mut branch, &arm.x.pattern, value.clone())?;
                    let available =
                        logical(span, LogicalOp::And, exits::not(span, seen.clone())?, p.clone())?;
                    branch.active = logical(span, LogicalOp::And, s.active.clone(), available)?;
                    let guard = self.readonly(&mut branch, &arm.x.guard)?;
                    let condition = logical(span, LogicalOp::And, p, guard)?;
                    branch.active =
                        logical(span, LogicalOp::And, branch.active, condition.clone())?;
                    let result = self.readonly(&mut branch, &arm.x.body)?;
                    seen = logical(span, LogicalOp::Or, seen, condition.clone())?;
                    results.push((condition, result));
                }
                self.require(s, span, seen)?;
                let mut out = results
                    .last()
                    .ok_or_else(|| unsupported(span, "empty record match"))?
                    .1
                    .clone();
                for (c, v) in results.into_iter().rev().skip(1) {
                    out = conditional(span, c, v, out)?;
                }
                Ok(out)
            }
            ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Static, name, ..),
                args,
                post_args: None,
                body: None,
            } => {
                let model = self.all.get(name).and_then(collections::model);
                let i = args.first().and_then(|e| self.vector(e)).ok_or_else(|| {
                    unsupported(span, "record vector calls must directly borrow a field")
                })?;
                match model {
                    Some(Model::Len) if args.len() == 1 => self.len(i),
                    Some(Model::Index) if args.len() == 2 => {
                        let index = self.readonly(s, &args[1])?;
                        let bound = summary::binary(
                            span,
                            BinaryOp::Inequality(InequalityOp::Lt),
                            index.clone(),
                            self.len(i)?,
                        )?;
                        self.require(s, span, bound)?;
                        self.model_call(
                            span,
                            Model::SeqIndex,
                            vec![self.shape.fields[i].element.clone().unwrap()],
                            vec![s.fields[i].clone(), index],
                        )
                    }
                    _ => Err(unsupported(
                        span,
                        "record calls support direct Vec length/index and closed primitive From conversions",
                    )),
                }
            }
            ExprX::Assign { place, rhs, op, resolve: false, .. } => {
                let value = self.readonly(s, rhs)?;
                if let Some(i) = self.field(place) {
                    if self.shape.fields[i].element.is_some() {
                        return Err(unsupported(span, "whole-vector replacement"));
                    }
                    let value = if let Some(op) = op {
                        self.binary(s, span, *op, s.fields[i].clone(), value)?
                    } else {
                        value
                    };
                    let value = coerce(span, value, &self.shape.fields[i].typ)?;
                    s.fields[i] = conditional(span, s.active.clone(), value, s.fields[i].clone())?;
                } else if let Some(prefix) = self.path(place) {
                    if op.is_some() {
                        return Err(unsupported(span, "compound nested-record assignment"));
                    }
                    let indices: Vec<_> = self
                        .shape
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| {
                            f.path.len() > prefix.len()
                                && shape::same_path(
                                    &prefix,
                                    &f.path[..prefix.len()]
                                        .iter()
                                        .map(|p| p.op.clone())
                                        .collect::<Vec<_>>(),
                                )
                        })
                        .map(|(i, _)| i)
                        .collect();
                    if indices.is_empty() {
                        return Err(unsupported(span, "nested-record assignment target"));
                    }
                    for i in indices {
                        if self.shape.fields[i].element.is_some() {
                            return Err(unsupported(span, "whole-vector replacement"));
                        }
                        let mut leaf = value.clone();
                        for step in &self.shape.fields[i].path[prefix.len()..] {
                            leaf = self.project(span, leaf, &step.op)?;
                        }
                        s.fields[i] =
                            conditional(span, s.active.clone(), leaf, s.fields[i].clone())?;
                    }
                } else {
                    if op.is_some() {
                        return Err(unsupported(span, "compound vector-slot assignment"));
                    }
                    let PlaceX::DerefMut(p) = &place.x else {
                        return Err(unsupported(span, "record assignment target"));
                    };
                    let PlaceX::Temporary(e) = &p.x else {
                        return Err(unsupported(span, "record vector assignment place"));
                    };
                    let ExprX::Call {
                        target: CallTarget::Fun(CallTargetKind::Static, name, ..),
                        args,
                        post_args: None,
                        body: None,
                    } = &e.x
                    else {
                        return Err(unsupported(span, "record vector assignment call"));
                    };
                    if self.all.get(name).and_then(collections::model) != Some(Model::IndexMut)
                        || args.len() != 2
                    {
                        return Err(unsupported(span, "record vector assignment model"));
                    }
                    let i = self
                        .vector(&args[0])
                        .ok_or_else(|| unsupported(span, "record vector assignment alias"))?;
                    let index = self.readonly(s, &args[1])?;
                    self.require(
                        s,
                        span,
                        summary::binary(
                            span,
                            BinaryOp::Inequality(InequalityOp::Lt),
                            index.clone(),
                            self.len(i)?,
                        )?,
                    )?;
                    let element = self.shape.fields[i].element.clone().unwrap();
                    let value = coerce(span, value, &element)?;
                    let updated = self.model_call(
                        span,
                        Model::SeqUpdate,
                        vec![element],
                        vec![s.fields[i].clone(), index, value],
                    )?;
                    s.fields[i] =
                        conditional(span, s.active.clone(), updated, s.fields[i].clone())?;
                }
                exits::unit_value(span)
            }
            _ => Err(unsupported(
                span,
                "record expression outside the bounded, loop-free state subset",
            )),
        }
    }
    fn binary(
        &mut self,
        s: &State,
        span: &Span,
        op: BinaryOp,
        a: Term,
        b: Term,
    ) -> Result<Term, VirErr> {
        let normalize = |v: Term| -> Result<Term, VirErr> {
            let mut typ = v.expr.typ.clone();
            while let TypX::Decorate(TypDecoration::Ref, None, t) = &*typ {
                typ = t.clone();
            }
            coerce(span, v, &typ)
        };
        let a = normalize(a)?;
        let b = normalize(b)?;
        if matches!(op, BinaryOp::Eq(_) | BinaryOp::Ne)
            && (same(&a.expr.typ, &b.expr.typ)
                || eligibility::integer(&a.expr.typ) && eligibility::integer(&b.expr.typ))
        {
            return term(span, &bool_typ(), ExprX::Binary(op, a.expr, b.expr), 1 + a.size + b.size);
        }
        let (pure, range, div) = eligibility::source_binary(op)
            .ok_or_else(|| unsupported(span, "record binary operation"))?;
        if div {
            return Err(unsupported(span, "record division/remainder"));
        }
        let value = summary::binary(span, pure, a, b)?;
        if let Some(range) = range {
            self.require(s, span, arithmetic::in_range(span, value.clone(), range)?)?;
        }
        Ok(value)
    }
}

fn constructor_fields(
    span: &Span,
    typ: &Typ,
    dt: &Dt,
    variant: &Ident,
    datatypes: &[Datatype],
) -> Result<Vec<(Ident, Typ)>, VirErr> {
    let invalid = || unsupported(span, "record value constructor outside tuples and Option");
    let mut typ = typ;
    while let TypX::Decorate(TypDecoration::Ref, None, inner) = &**typ {
        typ = inner;
    }
    let TypX::Datatype(actual, args, _) = &**typ else { return Err(invalid()) };
    if actual != dt {
        return Err(invalid());
    }
    match dt {
        Dt::Tuple(n)
            if *n <= 4 && args.len() == *n && variant == &crate::def::prefix_tuple_variant(*n) =>
        {
            Ok(args
                .iter()
                .enumerate()
                .map(|(i, t)| (crate::def::positional_field_ident(i), t.clone()))
                .collect())
        }
        Dt::Path(p)
            if models::path_is(p, CrateId::Core, &["option", "Option"]) && args.len() == 1 =>
        {
            match variant.as_str() {
                "None" => Ok(vec![]),
                "Some" => Ok(vec![(crate::def::positional_field_ident(0), args[0].clone())]),
                _ => Err(invalid()),
            }
        }
        _ => shape::fields(typ, datatypes)
            .filter(|fs| fs.iter().all(|(op, _)| &op.datatype == dt && &op.variant == variant))
            .map(|fs| fs.into_iter().map(|(op, t)| (op.field, t)).collect())
            .ok_or_else(invalid),
    }
}

pub(super) fn infer(
    f: &Function,
    all: &HashMap<Fun, Function>,
    datatypes: &[Datatype],
) -> Result<Contract, VirErr> {
    let shape = validate_signature(f, datatypes, all)?;
    let mut engine = Interpreter {
        function: f,
        shape,
        all,
        datatypes,
        requirements: vec![],
        remaining: FORMULA_BUDGET,
    };
    let mut fields = Vec::new();
    for (i, field) in engine.shape.fields.iter().enumerate() {
        fields.push(if field.element.is_some() {
            engine.view(i, false)?
        } else {
            field_value(f, &engine.shape, field, false)?
        });
    }
    let locals =
        f.x.params
            .iter()
            .skip(1)
            .map(|p| {
                Ok((
                    p.x.name.clone(),
                    atom(SpannedTyped::new(&f.span, &p.x.typ, ExprX::Var(p.x.name.clone())))?,
                ))
            })
            .collect::<Result<_, VirErr>>()?;
    let mut state = State { fields, locals, active: boolean(&f.span, true) };
    let result = engine.expr(
        &mut state,
        f.x.body.as_ref().ok_or_else(|| unsupported(&f.span, "missing record body"))?,
    )?;
    let mut pre = boolean(&f.span, true);
    for p in &engine.requirements {
        pre = logical(&f.span, LogicalOp::And, pre, p.clone())?;
    }
    let mut post = boolean(&f.span, true);
    if !mutable(f) {
        let output = atom(SpannedTyped::new(
            &f.span,
            &f.x.inner_ret.x.typ,
            ExprX::Var(f.x.inner_ret.x.name.clone()),
        ))?;
        let result=if eligibility::integer(&f.x.inner_ret.x.typ) && eligibility::integer(&result.expr.typ) {result} else {coerce(&f.span,result,&f.x.inner_ret.x.typ)?};
        post = equal(&f.span, output, result)?;
    }
    for (i, field) in engine.shape.fields.iter().enumerate().filter(|_| mutable(f)) {
        let after = if field.element.is_some() {
            engine.view(i, true)?
        } else {
            field_value(f, &engine.shape, field, true)?
        };
        post = logical(
            &f.span,
            LogicalOp::And,
            post,
            equal(&f.span, after, state.fields[i].clone())?,
        )?;
    }
    // Count actual generated nodes as well as builder estimates, including
    // reference wrappers inserted when instantiating standard models.
    let pre = atom(pre.expr)?;
    let post = atom(post.expr)?;
    Contract::new(pre.expr, post.expr)
}

/// A closed state-summary language, with one final equality for every field.
/// Proof evidence/fingerprints are validated separately by the import boundary.
pub(super) fn validate_summary(
    f: &Function,
    all: &HashMap<Fun, Function>,
    datatypes: &[Datatype],
) -> Result<(), VirErr> {
    crate::infer_proof::CandidateId::from_function(f)?;
    let invalid = || unsupported(&f.span, "invalid imported record state summary");
    if f.x.body.is_some()
        || f.x.require.len() != 1
        || f.x.ensure.0.len() != 1
        || !f.x.ensure.1.is_empty()
    {
        return Err(invalid());
    }
    let mut signature = f.x.clone();
    signature.require = Arc::new(vec![]);
    signature.ensure = (Arc::new(vec![]), Arc::new(vec![]));
    let shape = validate_signature(&f.new_x(signature), datatypes, all)?;
    let engine =
        Interpreter { function: f, shape, all, datatypes, requirements: vec![], remaining: 0 };
    fn clauses(e: &Expr, out: &mut Vec<Expr>) {
        if let ExprX::Logical(LogicalOp::And, a, b) = &e.x {
            clauses(a, out);
            clauses(b, out);
        } else {
            out.push(e.clone());
        }
    }
    let validate_value = |root: &Expr| -> Result<(), VirErr> {
        let mut remaining = FORMULA_BUDGET;
        crate::ast_visitor::expr_visitor_check(root, &mut |_, e| {
            remaining = remaining.checked_sub(1).ok_or_else(invalid)?;
            let valid = match &e.x {
                _ if arithmetic::arch_constant(e).is_some() => true,
                ExprX::Const(Constant::Bool(_)) => true,
                ExprX::Const(Constant::Int(n)) => n.bits() <= 4096,
                ExprX::Var(v) => {
                    f.x.params.iter().any(|p| &p.x.name == v && same(&p.x.typ, &e.typ))
                }
                ExprX::Old(a) => matches!(&a.x,ExprX::Var(v) if v==&f.x.params[0].x.name),
                ExprX::Unary(UnaryOp::MutRefCurrent, a) => matches!(&a.x, ExprX::Old(_)),
                ExprX::Unary(UnaryOp::Not | UnaryOp::Clip { .. }, _) => true,
                ExprX::UnaryOpr(UnaryOpr::Field(op), a) => {
                    op.check == VariantCheck::None
                        && !op.get_variant
                        && (engine.shape.fields.iter().any(|field| {
                            field.path.iter().any(|p| {
                                p.op.datatype == op.datatype
                                    && p.op.variant == op.variant
                                    && p.op.field == op.field
                                    && same(&p.typ, &e.typ)
                            })
                        }) || constructor_fields(
                            &f.span,
                            &a.typ,
                            &op.datatype,
                            &op.variant,
                            datatypes,
                        )
                        .is_ok_and(|fs| fs.iter().any(|(n, t)| n == &op.field && same(t, &e.typ))))
                }
                ExprX::UnaryOpr(UnaryOpr::IsVariant { datatype, variant }, a) => {
                    constructor_fields(&f.span, &a.typ, datatype, variant, datatypes).is_ok()
                }
                ExprX::ReadPlace(p, k) => {
                    matches!(k.preliminary_kind, ReadKind::Spec)
                        && k.id == u64::MAX
                        && matches!(&p.x, PlaceX::Temporary(_))
                }
                ExprX::Call {
                    target:
                        CallTarget::Fun(
                            CallTargetKind::DynamicResolved {
                                resolved,
                                is_trait_default: false,
                                ..
                            },
                            name,
                            ..,
                        ),
                    post_args: None,
                    body: None,
                    ..
                } => {
                    models::path_is(&name.path, CrateId::Vstd, &["view", "View", "view"])
                        && all.get(resolved).is_some_and(|f|f.x.attrs.infer_sequence_view || collections::model(f) == Some(Model::View))
                }
                ExprX::Call {
                    target: CallTarget::Fun(CallTargetKind::Static, name, ..),
                    post_args: None,
                    body: None,
                    ..
                } => matches!(
                    all.get(name).and_then(collections::model),
                    Some(
                        Model::View
                            | Model::SpecLen
                            | Model::SeqLen
                            | Model::SeqIndex
                            | Model::SeqUpdate
                    )
                ),
                ExprX::Ctor(dt, v, fields, None) => {
                    value_type(&e.typ, &f.x.typ_params, datatypes, 4)
                        && constructor_fields(&f.span, &e.typ, dt, v, datatypes)
                            .is_ok_and(|fs| fs.len() == fields.len())
                }
                ExprX::Binary(op, _, _) => matches!(
                    op,
                    BinaryOp::Eq(_)
                        | BinaryOp::Ne
                        | BinaryOp::Inequality(_)
                        | BinaryOp::Arith(
                            ArithOp::Add(OverflowBehavior::Allow)
                                | ArithOp::Sub(OverflowBehavior::Allow)
                                | ArithOp::Mul(OverflowBehavior::Allow)
                        )
                ),
                ExprX::Logical(_, _, _) | ExprX::If(_, _, Some(_)) => true,
                _ => false,
            };
            if valid {
                Ok(())
            } else {
                Err(unsupported(&f.span, "invalid imported record state value"))
            }
        })
    };
    validate_value(&f.x.require[0])?;
    if !mutable(f) {
        let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &f.x.ensure.0[0].x else {
            return Err(invalid());
        };
        if !matches!(&lhs.x,ExprX::Var(n) if n==&f.x.inner_ret.x.name) || !(same(&lhs.typ, &rhs.typ) || eligibility::integer(&lhs.typ)&&eligibility::integer(&rhs.typ))
        {
            return Err(invalid());
        }
        return validate_value(rhs);
    }
    let mut posts = Vec::new();
    clauses(&f.x.ensure.0[0], &mut posts);
    if posts.len() != engine.shape.fields.len() {
        return Err(invalid());
    }
    for (i, (clause, field)) in posts.iter().zip(&engine.shape.fields).enumerate() {
        let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &clause.x else {
            return Err(invalid());
        };
        let expected = if field.element.is_some() {
            engine.view(i, true)?
        } else {
            field_value(f, &engine.shape, field, true)?
        };
        if format!("{:?}", expected.expr) != format!("{lhs:?}") {
            return Err(unsupported(
                &f.span,
                &format!(
                    "invalid imported record frame for {:?}",
                    field.path.iter().map(|p| &p.op.field).collect::<Vec<_>>()
                ),
            ));
        }
        validate_value(rhs)?;
    }
    Ok(())
}
