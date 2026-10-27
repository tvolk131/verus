//! Precision of the modeled final state of an exclusively borrowed record.
//!
//! Enumerate observations from the datatype, never from the candidate's clauses.
//! The executable SST and its borrow/final-value constraints remain unchanged.
use super::*;
use crate::ast::{
    BinaryOpr, FieldOpr, Mode, MutRefFutureSourceName, TypX, UnaryOp, UnaryOpr, VarAt, VariantCheck,
};
use crate::context::Ctx;

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
    let substitutions = datatype
        .x
        .typ_params
        .iter()
        .zip(type_args.iter())
        .map(|(p, t)| (p.0.clone(), t.clone()))
        .collect();
    let mut equalities = Vec::new();
    for field in datatype.x.variants[0].fields.iter() {
        if field.a.1 != Mode::Exec {
            return Err(unsupported());
        }
        // The datatype table has already passed through Poly. Recover the
        // logical field type before constructing a new expression to encode.
        let typ = crate::sst_util::subst_typ(&substitutions, &field.a.0);
        let typ = if let TypX::Boxed(t) = &*typ { t.clone() } else { typ };
        let op = UnaryOpr::Field(FieldOpr {
            datatype: dt.clone(),
            variant: datatype.x.variants[0].name.clone(),
            field: field.name.clone(),
            get_variant: false,
            check: VariantCheck::None,
        });
        let observe = |record: Exp| -> Result<Exp, VirErr> {
            let value = SpannedTyped::new(span, &typ, ExpX::UnaryOpr(op.clone(), record));
            if let Some(args) = crate::infer::record_vec_args(&typ) {
                let view = crate::infer::record_vec_view(&ctx.func_map).ok_or_else(unsupported)?;
                let ts = view.x.typ_params.iter().cloned().zip(args.iter().cloned()).collect();
                let ret = crate::sst_util::subst_typ(&ts, &view.x.inner_ret.x.typ);
                let crate::ast::FunctionKind::TraitMethodImpl { method, trait_typ_args, .. } =
                    &view.x.kind
                else {
                    return Err(unsupported());
                };
                let trait_args = Arc::new(
                    trait_typ_args.iter().map(|t| crate::sst_util::subst_typ(&ts, t)).collect(),
                );
                Ok(SpannedTyped::new(
                    span,
                    &ret,
                    ExpX::Call(
                        CallFun::Fun(method.clone(), Some((view.x.name.clone(), args.clone()))),
                        trait_args,
                        Arc::new(vec![value]),
                    ),
                ))
            } else {
                Ok(value)
            }
        };
        let a = observe(other.clone())?;
        let b = observe(actual.clone())?;
        // Sequences are determined by their elements, not an opaque SMT identity.
        let equality = if crate::infer::record_vec_args(&typ).is_some() {
            ExpX::BinaryOpr(BinaryOpr::ExtEq(false, a.typ.clone()), a, b)
        } else {
            ExpX::Binary(BinaryOp::Eq, a, b)
        };
        equalities.push(SpannedTyped::new(span, &bool_typ(), equality));
    }
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
