//! Precision checking using the original executable's verifier-generated SST.
//!
//! These checks do not symbolically reimplement Rust arithmetic. In particular,
//! the necessary-domain query uses the safety conditions emitted by ast_to_sst,
//! independently of the predicates proposed by the inference engine.

use crate::ast::{SpannedTyped, Typ, VarIdent, VarIdentDisambiguate, VirErr};
use crate::ast_util::bool_typ;
use crate::def::Spanned;
use crate::messages::{Span, error};
use crate::sst::*;
use std::collections::HashMap;
use std::sync::Arc;

mod state;

#[derive(Clone, Copy, Debug)]
pub enum PrecisionCheck {
    Maximality,
    Exactness,
}

/// Reject a precision-transformed body presented as an ordinary soundness
/// obligation (and any other mismatch between a query label and its theorem).
pub fn validate_query_kind(
    check: &FuncCheckSst,
    precision: Option<PrecisionCheck>,
    span: &Span,
) -> Result<(), VirErr> {
    // Invariant and decreasing-measure checks are emitted by the ordinary
    // SST-to-AIR loop lowering. Never accept a loop with those checks disabled.
    crate::sst_visitor::map_stm_visitor(&check.body, &mut |stm| {
        if let StmX::Loop { loop_isolation, cond, invs, decrease, is_for_loop, .. } = &stm.x {
            if *loop_isolation
                || *is_for_loop
                || cond.is_some()
                || invs.len() != 2
                || invs.iter().any(|inv| !inv.at_entry || !inv.at_exit)
                || decrease.len() != 1
            {
                return Err(error(
                    &stm.span,
                    "Verus Infer: missing or incompatible counted-loop proof annotations",
                ));
            }
        }
        Ok(stm.clone())
    })?;
    let valid = matches!(
        (precision, check.post_condition.kind),
        (None, PostConditionKind::Ensures)
            | (Some(PrecisionCheck::Maximality), PostConditionKind::InferMaximality)
            | (Some(PrecisionCheck::Exactness), PostConditionKind::InferExactness)
    );
    if valid {
        Ok(())
    } else {
        Err(error(span, "Verus Infer: query kind does not match verification body"))
    }
}

impl PrecisionCheck {
    pub fn label(self) -> &'static str {
        match self {
            Self::Maximality => "safe-domain maximality",
            Self::Exactness => "output exactness",
        }
    }
}

pub fn precision_check(
    ctx: &crate::context::Ctx,
    function: &FunctionSst,
    original: &FuncCheckSst,
    kind: PrecisionCheck,
) -> Result<FuncCheckSst, VirErr> {
    if crate::infer::is_owned_summary(&ctx.func_map[&function.x.name]) && matches!(kind,PrecisionCheck::Exactness){
        return state::returned(ctx,function,original);
    }
    if function.x.pars.iter().any(|p| matches!(&*p.x.typ, crate::ast::TypX::MutRef(_))) {
        if !crate::infer::is_record_summary(&ctx.func_map[&function.x.name]) {
            return Err(error(
                &function.span,
                "Verus Infer incomplete: mutable-state precision requires a supported record",
            ));
        }
        if matches!(kind, PrecisionCheck::Exactness) {
            return state::exactness(ctx, function, original);
        }
    }
    let mut check =
        precision_check_body(&function.span, &function.x.inner_ret.x.typ, original, kind)?;
    if matches!(kind, PrecisionCheck::Maximality) {
        // Preconditions describe entry state. Mutable-reference lowering can
        // update the parameter's current value; evaluate P against its snapshot.
        Arc::make_mut(&mut check.post_condition).ens_exps =
            entry_requirements(&function.x.pars, &original.reqs);
        if crate::infer::is_wrapper_trait(&ctx.func_map[&function.x.name]) {
            check = trait_boundary(
                &function.span, &function.x.pars, &function.x.decl.reqs, original, check,
            );
        }
    }
    Ok(check)
}

/// The ordinary body query keeps the inherited trait precondition. Before any
/// successful-execution assumptions in maximality, independently prove that it
/// equals the candidate domain used by inferred callers. The assertion is added
/// AFTER the original safety assertions have been converted to assumptions.
fn trait_boundary(
    span: &Span, pars: &Pars, proposed: &Exps, original: &FuncCheckSst,
    mut check: FuncCheckSst,
) -> FuncCheckSst {
    let proposed = entry_requirements(pars, proposed);
    let inherited = entry_requirements(pars, &original.reqs);
    let relation = SpannedTyped::new(span, &bool_typ(), ExpX::Binary(
        BinaryOp::Eq, crate::sst_util::sst_conjoin(span, &proposed),
        crate::sst_util::sst_conjoin(span, &inherited),
    ));
    let assertion = crate::def::Spanned::new(span.clone(), StmX::Assert(None, Some(error(
        span, "Verus Infer: inherited trait precondition must equal the inferred safe domain",
    )), relation));
    check.body = crate::def::Spanned::new(
        span.clone(), StmX::Block(Arc::new(vec![assertion, check.body])),
    );
    Arc::make_mut(&mut check.post_condition).ens_exps = proposed;
    check
}

fn entry_requirements(pars: &Pars, reqs: &Exps) -> Exps {
    let inputs = pars.iter().map(|p| p.x.name.clone()).collect();
    Arc::new(reqs.iter().map(|e| crate::sst_util::exp_with_vars_at_pre_state(e, &inputs)).collect())
}

fn precision_check_body(
    span: &Span,
    return_typ: &Typ,
    original: &FuncCheckSst,
    kind: PrecisionCheck,
) -> Result<FuncCheckSst, VirErr> {
    validate_query_kind(original, None, span)?;
    let mut check = original.clone();
    let mut post = (*original.post_condition).clone();
    post.ens_spec_precondition_stms = Arc::new(vec![]);
    match kind {
        PrecisionCheck::Maximality => {
            // Safe execution implies P. Remove P from the assumptions and Q from
            // the conclusion. Every built-in safety check becomes a constraint
            // selecting successful executions; it is *not* removed from the
            // ordinary soundness query. Eligibility excludes arbitrary loops,
            // explicit assumptions, and proof code in the source body.
            check.reqs = Arc::new(vec![]);
            post.ens_exps = original.reqs.clone();
            post.kind = PostConditionKind::InferMaximality;
            check.body =
                crate::sst_visitor::map_stm_visitor(&original.body, &mut |stm| match &stm.x {
                    StmX::Assert(_, _, condition) => {
                        Ok(Spanned::new(stm.span.clone(), StmX::Assume(condition.clone())))
                    }
                    StmX::Call { .. }
                    | StmX::Assume(_)
                    | StmX::Assign { .. }
                    | StmX::Return { .. }
                    | StmX::If(..)
                    // Non-isolated while lowering inserts its exit branch.
                    | StmX::BreakOrContinue { .. }
                    // Only prefix invariants occur here. The backend still
                    // checks their initialization/preservation and termination
                    // without the candidate precondition in this query.
                    | StmX::Loop { .. }
                    | StmX::Block(_) => Ok(stm.clone()),
                    _ => Err(error(
                        &stm.span,
                        "Verus Infer incomplete: unexpected control flow in precision checker",
                    )),
                })?;
        }
        PrecisionCheck::Exactness => {
            // For an arbitrary result y, prove Q(inputs,y) iff y is the observed
            // return value of the ORIGINAL executable. This rejects true/range
            // summaries that are sound but admit additional results. A fresh
            // unconstrained SMT constant acts as a universally quantified y.
            let dest = post
                .dest
                .as_ref()
                .ok_or_else(|| error(span, "Verus Infer incomplete: missing return value"))?;
            let name = VarIdent(
                Arc::new("__verus_infer_output".to_owned()),
                VarIdentDisambiguate::AirLocal,
            );
            if original.local_decls.iter().any(|d| d.ident == name) {
                return Err(error(span, "Verus Infer incomplete: precision variable collision"));
            }
            let typ = return_typ;
            let other = SpannedTyped::new(span, typ, ExpX::Var(name.clone()));
            let actual = SpannedTyped::new(span, typ, ExpX::Var(dest.clone()));
            let substitutions = HashMap::from([(dest.clone(), other.clone())]);
            let relation = crate::sst_util::sst_conjoin(span, &original.post_condition.ens_exps);
            let relation = crate::sst_util::subst_exp(&HashMap::new(), &substitutions, &relation);
            // Rust fixed arrays are values determined by every element. Ordinary
            // ensures lowering already uses Verus's extensional equality for
            // these arrays; use the same whole-value equality for arbitrary y.
            // Opaque SMT array identity would distinguish even empty arrays.
            let equality =
                if matches!(&**typ, crate::ast::TypX::Primitive(crate::ast::Primitive::Array, _)) {
                    let boxed = |value: Exp| {
                        SpannedTyped::new(
                            span,
                            &Arc::new(crate::ast::TypX::Boxed(typ.clone())),
                            ExpX::UnaryOpr(crate::ast::UnaryOpr::Box(typ.clone()), value),
                        )
                    };
                    ExpX::BinaryOpr(
                        crate::ast::BinaryOpr::ExtEq(false, typ.clone()),
                        boxed(other),
                        boxed(actual),
                    )
                } else {
                    ExpX::Binary(BinaryOp::Eq, other, actual)
                };
            let equals_actual = SpannedTyped::new(span, &bool_typ(), equality);
            post.ens_exps = Arc::new(vec![SpannedTyped::new(
                span,
                &bool_typ(),
                ExpX::Binary(BinaryOp::Eq, relation, equals_actual),
            )]);
            post.kind = PostConditionKind::InferExactness;
            // Quantify over valid Rust values, including enum payload types.
            let typed_other = SpannedTyped::new(
                span,
                &bool_typ(),
                ExpX::UnaryOpr(
                    crate::ast::UnaryOpr::HasType(typ.clone()),
                    SpannedTyped::new(span, typ, ExpX::Var(name.clone())),
                ),
            );
            Arc::make_mut(&mut check.reqs).push(typed_other);
            Arc::make_mut(&mut check.local_decls).push(Arc::new(LocalDeclX {
                ident: name,
                typ: typ.clone(),
                kind: LocalDeclKind::Assert,
            }));
        }
    }
    check.post_condition = Arc::new(post);
    Ok(check)
}

#[cfg(test)]
mod tests;
