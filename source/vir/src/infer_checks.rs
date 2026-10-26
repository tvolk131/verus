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
    function: &FunctionSst,
    original: &FuncCheckSst,
    kind: PrecisionCheck,
) -> Result<FuncCheckSst, VirErr> {
    precision_check_body(&function.span, &function.x.inner_ret.x.typ, original, kind)
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
            // ordinary soundness query. Eligibility excludes loops,
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
            let equals_actual =
                SpannedTyped::new(span, &bool_typ(), ExpX::Binary(BinaryOp::Eq, other, actual));
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
