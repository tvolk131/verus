//! Precision of the modeled final state of an exclusively borrowed record.
//!
//! Enumerate observations from the datatype, never from the candidate's clauses.
//! The executable SST and its borrow/final-value constraints remain unchanged.
use super::*;
use crate::ast::{
    BinaryOpr, FieldOpr, Mode, MutRefFutureSourceName, TypX, UnaryOp, UnaryOpr, VarAt, VariantCheck,
};
use crate::context::Ctx;

/// Observe a complete returned value independently of the candidate's clauses.
pub(crate) fn output_equivalence(ctx: &Ctx, span: &Span, typ: &Typ, a: Exp, b: Exp) -> Result<Exp, VirErr> {
    let mut equalities = vec![];
    observations(ctx, span, typ, a, b, 9, &mut equalities)?;
    Ok(crate::sst_util::sst_conjoin(span, &Arc::new(equalities)))
}

/// Observe a complete returned value independently of the candidate's clauses.
pub(super) fn returned(
    ctx: &Ctx,
    function: &FunctionSst,
    original: &FuncCheckSst,
) -> Result<FuncCheckSst, VirErr> {
    returned_body_kind(ctx, &function.span, &function.x.inner_ret.x.typ, original,
        crate::infer::identity_borrow_return(&ctx.func_map[&function.x.name]))
}

#[cfg(test)]
pub(super) fn returned_body(
    ctx: &Ctx,
    span: &Span,
    typ: &Typ,
    original: &FuncCheckSst,
) -> Result<FuncCheckSst, VirErr> {
    returned_body_kind(ctx,span,typ,original,false)
}

fn returned_body_kind(ctx:&Ctx,span:&Span,typ:&Typ,original:&FuncCheckSst,identity:bool)->Result<FuncCheckSst,VirErr>{
    validate_query_kind(original, None, span)?;
    let name =
        VarIdent(Arc::new("__verus_infer_owned_output".into()), VarIdentDisambiguate::AirLocal);
    if original.local_decls.iter().any(|d| d.ident == name) {
        return Err(error(span, "owned precision variable collision"));
    }
    let dest = original
        .post_condition
        .dest
        .as_ref()
        .ok_or_else(|| error(span, "owned precision return missing"))?;
    let other = SpannedTyped::new(span, typ, ExpX::Var(name.clone()));
    let actual = SpannedTyped::new(span, typ, ExpX::Var(dest.clone()));
    let mut equalities = vec![];
    if identity {
        equalities.push(SpannedTyped::new(span,&bool_typ(),ExpX::Binary(BinaryOp::Eq,other.clone(),actual)));
    } else { observations(ctx, span, typ, other.clone(), actual, 9, &mut equalities)?; }
    let vars = [(name.clone(), typ.clone()), (dest.clone(), typ.clone())];
    let equals = crate::poly::visit_exp_native_for_pure_exp_with_vars(
        ctx,
        &crate::sst_util::sst_conjoin(span, &Arc::new(equalities)),
        &vars,
    );
    let relation = crate::sst_util::sst_conjoin(span, &original.post_condition.ens_exps);
    let relation = crate::sst_util::subst_exp(
        &HashMap::new(),
        &HashMap::from([(dest.clone(), other.clone())]),
        &relation,
    );
    let mut check = original.clone();
    let post = Arc::make_mut(&mut check.post_condition);
    post.kind = PostConditionKind::InferExactness;
    post.ens_spec_precondition_stms = Arc::new(vec![]);
    post.ens_exps = Arc::new(vec![SpannedTyped::new(
        span,
        &bool_typ(),
        ExpX::Binary(BinaryOp::Eq, relation, equals),
    )]);
    Arc::make_mut(&mut check.reqs).push(SpannedTyped::new(
        span,
        &bool_typ(),
        ExpX::UnaryOpr(UnaryOpr::HasType(typ.clone()), other),
    ));
    Arc::make_mut(&mut check.local_decls).push(Arc::new(LocalDeclX {
        ident: name,
        typ: typ.clone(),
        kind: LocalDeclKind::Assert,
    }));
    Ok(check)
}

pub(super) fn exactness(
    ctx: &Ctx,
    function: &FunctionSst,
    original: &FuncCheckSst,
) -> Result<FuncCheckSst, VirErr> {
    if !crate::ast_util::types_equal(&function.x.inner_ret.x.typ, &crate::ast_util::unit_typ()) {
        return Err(error(
            &function.span,
            "Verus Infer incomplete: record-state precision requires unit return",
        ));
    }
    exactness_body(ctx, &function.span, &function.x.pars, original)
}

pub(super) fn exactness_body(
    ctx: &Ctx,
    span: &Span,
    pars: &Pars,
    original: &FuncCheckSst,
) -> Result<FuncCheckSst, VirErr> {
    validate_query_kind(original, None, span)?;
    let unsupported =
        || error(span, "Verus Infer incomplete: unsupported record-state precision shape");
    let root = pars.first().ok_or_else(unsupported)?;
    let TypX::MutRef(record_typ) = &*root.x.typ else { return Err(unsupported()) };
    let TypX::Datatype(dt, type_args, _) = &**record_typ else { return Err(unsupported()) };
    let datatype = ctx.datatype_map.get(dt).ok_or_else(unsupported)?;
    if datatype.x.variants.len() != 1
        || datatype.x.variants[0].fields.is_empty()
        || datatype.x.variants[0].fields.len() > 8
        || datatype.x.typ_params.len() != type_args.len()
        || pars.iter().skip(1).any(|p| matches!(&*p.x.typ, TypX::MutRef(_)))
    {
        return Err(unsupported());
    }
    let name =
        VarIdent(Arc::new("__verus_infer_record_output".into()), VarIdentDisambiguate::AirLocal);
    if original.local_decls.iter().any(|d| d.ident == name) {
        return Err(error(span, "Verus Infer incomplete: precision variable collision"));
    }
    let other = SpannedTyped::new(span, record_typ, ExpX::Var(name.clone()));
    let input = SpannedTyped::new(span, &root.x.typ, ExpX::VarAt(root.x.name.clone(), VarAt::Pre));
    let actual = SpannedTyped::new(
        span,
        record_typ,
        ExpX::Unary(UnaryOp::MutRefFuture(MutRefFutureSourceName::Final), input),
    );
    let mut equalities = vec![];
    observations(ctx, span, record_typ, other.clone(), actual, 5, &mut equalities)?;
    let equals_actual = crate::sst_util::sst_conjoin(span, &Arc::new(equalities));
    let vars = [(root.x.name.clone(), root.x.typ.clone()), (name.clone(), record_typ.clone())];
    let equals_actual =
        crate::poly::visit_exp_native_for_pure_exp_with_vars(ctx, &equals_actual, &vars);

    // Replace ONLY the final value behind the input borrow. The initial value,
    // all arguments, the body, and every borrow-expiration constraint stay fixed.
    // This substitution is performed on the already lowered/boxed relation.
    let boxed_other = crate::poly::coerce_exp_to_poly(ctx, &other);
    let relation = crate::sst_util::sst_conjoin(span, &original.post_condition.ens_exps);
    let relation = crate::sst_visitor::map_exp_visitor(&relation, &mut |e| {
        if matches!(&e.x, ExpX::Unary(UnaryOp::MutRefFuture(_), base)
            if matches!(&base.x, ExpX::VarAt(n, VarAt::Pre) if n == &root.x.name))
        {
            boxed_other.clone()
        } else {
            e.clone()
        }
    });
    let mut check = original.clone();
    let post = Arc::make_mut(&mut check.post_condition);
    post.ens_exps = Arc::new(vec![SpannedTyped::new(
        span,
        &bool_typ(),
        ExpX::Binary(BinaryOp::Eq, relation, equals_actual),
    )]);
    post.ens_spec_precondition_stms = Arc::new(vec![]);
    post.kind = PostConditionKind::InferExactness;
    // The arbitrary record is a legal typed value, without assuming the
    // candidate relation, an application invariant, or equality to the result.
    Arc::make_mut(&mut check.reqs).push(SpannedTyped::new(
        span,
        &bool_typ(),
        ExpX::UnaryOpr(UnaryOpr::HasType(record_typ.clone()), other),
    ));
    Arc::make_mut(&mut check.local_decls).push(Arc::new(LocalDeclX {
        ident: name,
        typ: record_typ.clone(),
        kind: LocalDeclKind::Assert,
    }));
    Ok(check)
}

// Derive all observations independently from the datatype definitions. Nested
// records are traversed; no list of candidate equations is used as a frame.
fn observations(
    ctx: &Ctx,
    span: &Span,
    typ: &Typ,
    a: Exp,
    b: Exp,
    depth: usize,
    out: &mut Vec<Exp>,
) -> Result<(), VirErr> {
    let bad = || error(span, "Verus Infer incomplete: unsupported nested record precision shape");
    if depth == 0 || out.len() >= 64 {
        return Err(bad());
    }
    // Opaque imported datatypes have a boxed SST return type. Boxing changes
    // the solver representation, not the independent logical observation.
    if let TypX::Boxed(t) = &**typ {
        // Build a pre-poly formula. Its ordinary lowering inserts unboxing
        // at each projection; inserting it here would lower the same Box/Unbox
        // twice for generic trait returns.
        return observations(ctx, span, t, a, b, depth - 1, out);
    }
    if let TypX::Decorate(crate::ast::TypDecoration::Ref, None, _) = &**typ {
        // Shared returned payloads preserve complete logical values, including
        // when nested in Option/Result. Collection-view equality alone cannot
        // identify the object selected by the original borrow.
        out.push(SpannedTyped::new(span,&bool_typ(),ExpX::Binary(BinaryOp::Eq,a,b)));
        return Ok(());
    }
    if let TypX::Datatype(dt @ crate::ast::Dt::Path(p), args, _) = &**typ {
        // Invariant-bearing objects are preserved by complete value identity.
        // Do not expose or omit any of their hidden/cache/ghost representation.
        if ctx.datatype_map.get(dt).is_some_and(|d|d.x.user_defined_invariant_fn.is_some()) {
            out.push(SpannedTyped::new(span,&bool_typ(),ExpX::Binary(BinaryOp::Eq,a,b)));
            return Ok(());
        }
        if p.krate == crate::ast::CrateId::Core
            && p.segments.iter().map(|s| s.as_str()).eq(["result", "Result"])
            && args.len() == 2
        {
            let tag = |v: Exp| {
                SpannedTyped::new(
                    span,
                    &bool_typ(),
                    ExpX::UnaryOpr(
                        UnaryOpr::IsVariant {
                            datatype: dt.clone(),
                            variant: Arc::new("Ok".into()),
                        },
                        v,
                    ),
                )
            };
            out.push(SpannedTyped::new(
                span,
                &bool_typ(),
                ExpX::Binary(BinaryOp::Eq, tag(a.clone()), tag(b.clone())),
            ));
            for (i, variant) in ["Ok", "Err"].iter().enumerate() {
                let op = UnaryOpr::Field(FieldOpr {
                    datatype: dt.clone(),
                    variant: Arc::new((*variant).into()),
                    field: crate::def::positional_field_ident(0),
                    get_variant: false,
                    check: VariantCheck::None,
                });
                let mut payload = vec![];
                observations(
                    ctx,
                    span,
                    &args[i],
                    SpannedTyped::new(span, &args[i], ExpX::UnaryOpr(op.clone(), a.clone())),
                    SpannedTyped::new(span, &args[i], ExpX::UnaryOpr(op, b.clone())),
                    depth - 1,
                    &mut payload,
                )?;
                let guard = if i == 0 {
                    tag(a.clone())
                } else {
                    SpannedTyped::new(span, &bool_typ(), ExpX::Unary(UnaryOp::Not, tag(a.clone())))
                };
                for eq in payload {
                    out.push(SpannedTyped::new(
                        span,
                        &bool_typ(),
                        ExpX::Binary(BinaryOp::Implies, guard.clone(), eq),
                    ));
                }
            }
            return Ok(());
        }
    }
    // Independently selected structural evidence: this wrapper has exactly one
    // Vec field and its supplied View is the direct projection of that field.
    // Re-express the existing vector-content observation without exposing a
    // private field in the client. Never derive this choice from the candidate.
    if let Some(view) = crate::infer::record_sequence_view(typ, &ctx.func_map) {
        let TypX::Datatype(_, args, _) = &**typ else { return Err(bad()) };
        let ts = view.x.typ_params.iter().cloned().zip(args.iter().cloned()).collect();
        let ret = crate::sst_util::subst_typ(&ts, &view.x.inner_ret.x.typ);
        let crate::ast::FunctionKind::TraitMethodImpl { method, trait_typ_args, .. } = &view.x.kind
        else {
            return Err(bad());
        };
        let trait_args: crate::ast::Typs =
            Arc::new(trait_typ_args.iter().map(|t| crate::sst_util::subst_typ(&ts, t)).collect());
        let observe = |v: Exp| {
            SpannedTyped::new(
                span,
                &ret,
                ExpX::Call(
                    CallFun::Fun(method.clone(), Some((view.x.name.clone(), args.clone()))),
                    trait_args.clone(),
                    Arc::new(vec![v]),
                ),
            )
        };
        out.push(SpannedTyped::new(
            span,
            &bool_typ(),
            ExpX::BinaryOpr(BinaryOpr::ExtEq(false, ret.clone()), observe(a), observe(b)),
        ));
        return Ok(());
    }
    if let Some(args) = crate::infer::record_vec_args(typ) {
        let view = crate::infer::record_vec_view(&ctx.func_map).ok_or_else(bad)?;
        let ts = view.x.typ_params.iter().cloned().zip(args.iter().cloned()).collect();
        let ret = crate::sst_util::subst_typ(&ts, &view.x.inner_ret.x.typ);
        let crate::ast::FunctionKind::TraitMethodImpl { method, trait_typ_args, .. } = &view.x.kind
        else {
            return Err(bad());
        };
        let trait_args: crate::ast::Typs =
            Arc::new(trait_typ_args.iter().map(|t| crate::sst_util::subst_typ(&ts, t)).collect());
        let observe = |v: Exp| {
            SpannedTyped::new(
                span,
                &ret,
                ExpX::Call(
                    CallFun::Fun(method.clone(), Some((view.x.name.clone(), args.clone()))),
                    trait_args.clone(),
                    Arc::new(vec![v]),
                ),
            )
        };
        out.push(SpannedTyped::new(
            span,
            &bool_typ(),
            ExpX::BinaryOpr(BinaryOpr::ExtEq(false, ret.clone()), observe(a), observe(b)),
        ));
        return Ok(());
    }
    if let TypX::Datatype(dt @ crate::ast::Dt::Path(p), args, _) = &**typ {
        if p.krate == crate::ast::CrateId::Core
            && p.segments.iter().map(|s| s.as_str()).eq(["option", "Option"])
            && args.len() == 1
        {
            let tag = |v: Exp| {
                SpannedTyped::new(
                    span,
                    &bool_typ(),
                    ExpX::UnaryOpr(
                        UnaryOpr::IsVariant {
                            datatype: dt.clone(),
                            variant: Arc::new("Some".into()),
                        },
                        v,
                    ),
                )
            };
            out.push(SpannedTyped::new(
                span,
                &bool_typ(),
                ExpX::Binary(BinaryOp::Eq, tag(a.clone()), tag(b.clone())),
            ));
            let op = UnaryOpr::Field(FieldOpr {
                datatype: dt.clone(),
                variant: Arc::new("Some".into()),
                field: crate::def::positional_field_ident(0),
                get_variant: false,
                check: VariantCheck::None,
            });
            let mut payload = vec![];
            observations(
                ctx,
                span,
                &args[0],
                SpannedTyped::new(span, &args[0], ExpX::UnaryOpr(op.clone(), a.clone())),
                SpannedTyped::new(span, &args[0], ExpX::UnaryOpr(op, b)),
                depth - 1,
                &mut payload,
            )?;
            for eq in payload {
                out.push(SpannedTyped::new(
                    span,
                    &bool_typ(),
                    ExpX::Binary(BinaryOp::Implies, tag(a.clone()), eq),
                ));
            }
            return Ok(());
        }
    }
    let nested = matches!(&**typ,TypX::Datatype(crate::ast::Dt::Path(p),..) if !(p.krate==crate::ast::CrateId::Core && p.segments.iter().map(|s|s.as_str()).eq(["option","Option"])));
    if nested {
        let TypX::Datatype(dt, args, _) = &**typ else { unreachable!() };
        let d = ctx.datatype_map.get(dt).ok_or_else(bad)?;
        // A private first-party value has no representation in this module.
        // Observe its complete logical value rather than opening hidden fields
        // or weakening precision to a partial content view. Certified direct
        // sequence views above retain their existing independent observations.
        if !ctx.datatype_is_transparent.get(dt).copied().unwrap_or(false)
            && d.x.mode == Mode::Exec && d.x.proxy.is_none() && !d.x.destructor
            && d.x.typ_params.len() == args.len()
        {
            out.push(SpannedTyped::new(span,&bool_typ(),ExpX::Binary(BinaryOp::Eq,a,b)));
            return Ok(());
        }
        // Enumerate every tag and every guarded payload from the datatype,
        // independently of the candidate's equations or selected constructor.
        if (2..=8).contains(&d.x.variants.len()) && d.x.typ_params.len()==args.len()
            && d.x.variants.iter().all(|v|v.fields.len()<=4 && v.fields.iter().all(|f|f.a.1==Mode::Exec))
        {
            let ts=d.x.typ_params.iter().zip(args.iter()).map(|(p,t)|(p.0.clone(),t.clone())).collect();
            for variant in d.x.variants.iter() {
                let tag=|v:Exp|SpannedTyped::new(span,&bool_typ(),ExpX::UnaryOpr(
                    UnaryOpr::IsVariant{datatype:dt.clone(),variant:variant.name.clone()},v));
                out.push(SpannedTyped::new(span,&bool_typ(),ExpX::Binary(BinaryOp::Eq,tag(a.clone()),tag(b.clone()))));
                for f in variant.fields.iter() {
                    let t=crate::sst_util::subst_typ(&ts,&f.a.0);
                    let t=if let TypX::Boxed(t)=&*t {t.clone()}else{t};
                    let op=UnaryOpr::Field(FieldOpr{datatype:dt.clone(),variant:variant.name.clone(),field:f.name.clone(),get_variant:false,check:VariantCheck::None});
                    let mut payload=vec![];
                    observations(ctx,span,&t,SpannedTyped::new(span,&t,ExpX::UnaryOpr(op.clone(),a.clone())),
                        SpannedTyped::new(span,&t,ExpX::UnaryOpr(op,b.clone())),depth-1,&mut payload)?;
                    if out.len()+payload.len()>64{return Err(bad());}
                    for eq in payload {out.push(SpannedTyped::new(span,&bool_typ(),ExpX::Binary(BinaryOp::Implies,tag(a.clone()),eq)));}
                }
                if out.len()>64{return Err(bad());}
            }
            return Ok(());
        }
        if !d.x.variants.is_empty()
            && d.x.variants.len() <= 4
            && d.x.variants.iter().all(|v| v.fields.is_empty())
            && d.x.typ_params.is_empty()
        {
            out.push(SpannedTyped::new(span, &bool_typ(), ExpX::Binary(BinaryOp::Eq, a, b)));
            return Ok(());
        }
        if d.x.variants.len() != 1
            || d.x.variants[0].fields.is_empty()
            || d.x.variants[0].fields.len() > 8
            || d.x.typ_params.len() != args.len()
        {
            return Err(bad());
        }
        let ts =
            d.x.typ_params.iter().zip(args.iter()).map(|(p, t)| (p.0.clone(), t.clone())).collect();
        for field in d.x.variants[0].fields.iter() {
            if field.a.1 != Mode::Exec {
                return Err(bad());
            }
            let t = crate::sst_util::subst_typ(&ts, &field.a.0);
            let t = if let TypX::Boxed(t) = &*t { t.clone() } else { t };
            let op = UnaryOpr::Field(FieldOpr {
                datatype: dt.clone(),
                variant: d.x.variants[0].name.clone(),
                field: field.name.clone(),
                get_variant: false,
                check: VariantCheck::None,
            });
            let a = SpannedTyped::new(span, &t, ExpX::UnaryOpr(op.clone(), a.clone()));
            let b = SpannedTyped::new(span, &t, ExpX::UnaryOpr(op, b.clone()));
            observations(ctx, span, &t, a, b, depth - 1, out)?;
        }
    } else {
        out.push(SpannedTyped::new(span, &bool_typ(), ExpX::Binary(BinaryOp::Eq, a, b)));
    }
    Ok(())
}
