//! Complete tagged results and bounded pure query matches.
use super::*;
pub(super) fn result(t: &Typ) -> Option<(&Typ, &Typ)> {
    match &**t {
        TypX::Datatype(Dt::Path(p), ts, _)
            if models::path_is(p, CrateId::Core, &["result", "Result"]) && ts.len() == 2 =>
        {
            Some((&ts[0], &ts[1]))
        }
        _ => None,
    }
}
pub(super) fn flat_enum(t: &Typ, ds: &[Datatype]) -> bool {
    let TypX::Datatype(dt, ts, _) = &**t else { return false };
    ts.is_empty()
        && ds.iter().any(|d| {
            &d.x.name == dt
                && d.x.mode == Mode::Exec
                && d.x.proxy.is_none()
                && !d.x.destructor
                && d.x.user_defined_invariant_fn.is_none()
                && d.x.typ_params.is_empty()
                && !d.x.variants.is_empty()
                && d.x.variants.len() <= 4
                && d.x.variants.iter().all(|v| v.fields.is_empty())
        })
}
pub(super) fn has_value_enum(t:&Typ,ds:&[Datatype],depth:usize)->bool {
    if depth==0{return false;}
    let t=base(t);
    records::shape::variants(t,ds).is_some()
        || option(t).is_some_and(|t|has_value_enum(t,ds,depth-1))
        || result(t).is_some_and(|(a,b)|has_value_enum(a,ds,depth-1)||has_value_enum(b,ds,depth-1))
}
pub(super) fn tag(e: &Expr, v: &str) -> Expr {
    let TypX::Datatype(dt, ..) = &*e.typ else { unreachable!() };
    SpannedTyped::new(
        &e.span,
        &bool_typ(),
        ExprX::UnaryOpr(
            UnaryOpr::IsVariant { datatype: dt.clone(), variant: Arc::new(v.into()) },
            e.clone(),
        ),
    )
}
pub(super) fn field(t: &Typ, v: &str) -> FieldOpr {
    let TypX::Datatype(dt, ..) = &**t else { unreachable!() };
    FieldOpr {
        datatype: dt.clone(),
        variant: Arc::new(v.into()),
        field: crate::def::positional_field_ident(0),
        get_variant: false,
        check: VariantCheck::None,
    }
}
impl Engine<'_> {
    pub(super) fn propagate(&mut self,s:&mut State,e:&Expr,p:propagation::Propagation)->Result<Val,VirErr> {
        if p.kind!=propagation::Carrier::Result {
            return Err(unsupported(&e.span,"structured propagation currently requires Result value payloads"));
        }
        let (ok_typ,err_typ)=result(&p.operand.typ).ok_or_else(||unsupported(&e.span,"Result propagation input"))?;
        let (_,target_err)=result(&p.return_type).ok_or_else(||unsupported(&e.span,"Result propagation output"))?;
        if !records::shape::value_type(ok_typ,&self.f.x.typ_params,self.ds,5)
            || !records::shape::value_type(err_typ,&self.f.x.typ_params,self.ds,5)
            || !records::shape::value_type(target_err,&self.f.x.typ_params,self.ds,5) {
            return Err(unsupported(&e.span,"structured ? requires bounded value payloads"));
        }
        let input=self.expr(s,&p.operand)?;
        let Node::Result(ok,a,b)=&input.node else{return Err(unsupported(&e.span,"Result propagation layout"));};
        // Only the selected constructor's payload matters. For an absent branch,
        // use its ordinary datatype projection, guarded by an impossible tag.
        let payload=|variant:&str,t:&Typ,v:&Option<Box<Val>>|->Result<Val,VirErr>{
            match v {Some(v)=>Ok((**v).clone()),None=>self.layout(field_expr(&input.scalar(&e.span)?.expr,&field(&input.typ,variant),t),5)}
        };
        let a=payload("Ok",ok_typ,a)?;
        let b=payload("Err",err_typ,b)?;
        let error=Val::atom(coerce(&e.span,b.scalar(&e.span)?,target_err)?);
        let failed=logical(&e.span,LogicalOp::And,s.active.clone(),exits::not(&e.span,ok.clone())?)?;
        self.exits.push((failed,Val{typ:p.return_type,node:Node::Result(boolean(&e.span,false),None,Some(Box::new(error)))}));
        s.active=logical(&e.span,LogicalOp::And,s.active.clone(),ok.clone())?;
        Ok(a)
    }
    pub(super) fn result_post(
        &self,
        ok: &Term,
        a: Option<&Val>,
        b: Option<&Val>,
        out: &Expr,
    ) -> Result<Term, VirErr> {
        let span = &out.span;
        let test = atom(tag(out, "Ok"))?;
        let mut p = records::equal(span, test.clone(), ok.clone())?;
        for (variant, payload, guard) in
            [("Ok", a, test.clone()), ("Err", b, exits::not(span, test)?)]
        {
            if let Some(value) = payload {
                p = logical(
                    span,
                    LogicalOp::And,
                    p,
                    logical(
                        span,
                        LogicalOp::Implies,
                        guard,
                        self.post(value, &field_expr(out, &field(&out.typ, variant), &value.typ))?,
                    )?,
                )?;
            }
        }
        Ok(p)
    }
    pub(super) fn result_decode(
        &mut self,
        s: &mut State,
        out: &Expr,
        a: Option<&Val>,
        b: Option<&Val>,
        clauses: &[Expr],
    ) -> Result<Node, VirErr> {
        let test = tag(out, "Ok");
        let rhs = find_rhs(&test, clauses)?;
        let ok = self.formula(s, &rhs)?;
        let mut values = vec![];
        for (variant, layout, guard, absent) in [
            ("Ok", a, test.clone(), false),
            ("Err", b, exits::not(&out.span, atom(test.clone())?)?.expr, true),
        ] {
            let q = clauses.iter().find_map(|e| {
                if let ExprX::Logical(LogicalOp::Implies, c, q) = &e.x {
                    (skeleton(c) == skeleton(&guard)).then_some(q.clone())
                } else {
                    None
                }
            });
            values.push(if let Some(q) = q {
                let layout =
                    layout.ok_or_else(|| unsupported(&out.span, "Result payload shape"))?;
                let mut cs = vec![];
                split(&q, &mut cs);
                Some(Box::new(self.decode_parts(
                    s,
                    &field_expr(out, &field(&out.typ, variant), &layout.typ),
                    layout,
                    &cs,
                )?))
            } else if is_bool(&ok, absent) {
                None
            } else {
                return Err(unsupported(&out.span, "Result missing guarded payload"));
            });
        }
        let b = values.pop().unwrap();
        let a = values.pop().unwrap();
        Ok(Node::Result(ok, a, b))
    }
    pub(super) fn pattern(&mut self, s: &mut State, p: &Pattern, value: Val) -> Result<Term, VirErr> {
        self.spend(&p.span)?;
        let span = &p.span;
        match &p.x {
            PatternX::Wildcard => Ok(boolean(span, true)),
            PatternX::ImmutRef(p) => self.pattern(s, p, value),
            PatternX::Expr(e) if matches!(e.x,ExprX::Const(Constant::Int(_)|Constant::Bool(_))) => {
                records::equal(span,value.scalar(span)?,self.expr(s,e)?.scalar(span)?)
            }
            PatternX::Var(b)
                if matches!(b.by_ref, ByRef::No | ByRef::ImmutRef) && b.user_mut != Some(true) =>
            {
                s.locals.insert(
                    b.name.clone(),
                    if let Node::Atom(t) = value.node {
                        Val::atom(coerce(span, t, &b.typ)?)
                    } else {
                        value
                    },
                );
                Ok(boolean(span, true))
            }
            PatternX::Constructor(dt, variant, fs) => {
                let v = value.scalar(span)?;
                let expected =
                    records::constructor_fields(span, &v.expr.typ, dt, variant, self.ds)?;
                if expected.len() != fs.len() {
                    return Err(unsupported(span, "query match field coverage"));
                }
                let mut cond = if matches!(dt, Dt::Tuple(_)) {
                    boolean(span, true)
                } else {
                    variant_test(span, v.clone(), dt, variant)?
                };
                for f in fs.iter() {
                    let op = FieldOpr {
                        datatype: dt.clone(),
                        variant: variant.clone(),
                        field: f.name.clone(),
                        get_variant: false,
                        check: VariantCheck::None,
                    };
                    let child = self.field(Val::atom(v.clone()), &op, span)?;
                    cond = logical(span, LogicalOp::And, cond, self.pattern(s, &f.a, child)?)?;
                }
                Ok(cond)
            }
            _ => Err(unsupported(span, "query patterns require immutable Option/tuple bindings")),
        }
    }
    pub(super) fn match_expr(
        &mut self,
        s: &mut State,
        e: &Expr,
        p: &Place,
        arms: &Arms,
    ) -> Result<Val, VirErr> {
        if arms.is_empty() || arms.len() > 8 {
            return Err(unsupported(&e.span, "query match arm budget"));
        }
        let value = self.place(s, p)?;
        let mut seen = boolean(&e.span, false);
        let mut values = vec![];
        let mut active = boolean(&e.span, false);
        for arm in arms.iter() {
            let mut branch = s.clone();
            let test = self.pattern(&mut branch, &arm.x.pattern, value.clone())?;
            branch.active = logical(
                &e.span,
                LogicalOp::And,
                s.active.clone(),
                logical(&e.span, LogicalOp::And, exits::not(&e.span, seen.clone())?, test.clone())?,
            )?;
            let guard = self.expr(&mut branch, &arm.x.guard)?.scalar(&e.span)?;
            let test = logical(&e.span, LogicalOp::And, test, guard.clone())?;
            // The pattern is already in branch.active. Only the evaluated
            // guard is new; repeating the full test multiplies helper formulas.
            branch.active = logical(&e.span, LogicalOp::And, branch.active, guard)?;
            let out = self.expr(&mut branch, &arm.x.body)?;
            active = logical(&e.span, LogicalOp::Or, active, branch.active)?;
            seen = logical(&e.span, LogicalOp::Or, seen, test.clone())?;
            values.push((test, out));
        }
        self.require(s, seen)?;
        s.active = active;
        let mut out = values.last().unwrap().1.clone();
        for (test, value) in values.into_iter().rev().skip(1) {
            out = choose(&e.span, test, value, out)?;
        }
        Ok(out)
    }
}
