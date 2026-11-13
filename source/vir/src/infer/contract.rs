//! The only boundary allowed to install synthesized or cached contracts.
use super::*;
use sha2::{Digest, Sha256};

/// Candidate generation supplies formulas. Counted loops receive only checked
/// proof decoration; the production projection gate preserves executable code.
pub(super) struct Contract {
    pub(super) pre: Expr,
    pub(super) post: Expr,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input};

    #[test]
    fn installation_preserves_every_noncontract_field_and_binds_actual_formulas() {
        let original = function("original", input("x"));
        let contract = || {
            Contract::new(boolean(&original.span, true).expr, boolean(&original.span, true).expr)
                .unwrap()
        };
        let installed = contract().install(&original, "configuration and dependencies").unwrap();
        assert!(Arc::ptr_eq(original.x.body.as_ref().unwrap(), installed.x.body.as_ref().unwrap()));
        let mut restored = installed.x.clone();
        restored.require = original.x.require.clone();
        restored.ensure = original.x.ensure.clone();
        restored.attrs = original.x.attrs.clone();
        assert_eq!(format!("{restored:?}"), format!("{:?}", original.x));
        let mut attrs = (*installed.x.attrs).clone();
        attrs.spinoff_prover = original.x.attrs.spinoff_prover;
        attrs.infer_candidate_id = original.x.attrs.infer_candidate_id.clone();
        assert_eq!(format!("{attrs:?}"), format!("{:?}", original.x.attrs));
        let other_provenance = contract().install(&original, "other dependencies").unwrap();
        let other_formula =
            Contract::new(boolean(&original.span, false).expr, boolean(&original.span, true).expr)
                .unwrap()
                .install(&original, "configuration and dependencies")
                .unwrap();
        assert_eq!(
            installed.x.attrs.infer_candidate_id,
            contract()
                .install(&original, "configuration and dependencies")
                .unwrap()
                .x
                .attrs
                .infer_candidate_id
        );
        assert_ne!(
            installed.x.attrs.infer_candidate_id,
            other_provenance.x.attrs.infer_candidate_id
        );
        assert_ne!(installed.x.attrs.infer_candidate_id, other_formula.x.attrs.infer_candidate_id);
    }

    #[test]
    fn candidate_identity_ignores_proposal_locations_but_preserves_formulas_and_source() {
        let original = function("original", input("x"));
        let cold_post = SpannedTyped::new(
            &original.span,
            &bool_typ(),
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), input("x"), input("x")),
        );
        let mut cached_span = original.span.clone();
        cached_span.id += 1;
        cached_span.as_string = "cache-decoded proposal".into();
        let cached_post = crate::ast_visitor::map_expr_visitor(&cold_post, &|e| {
            Ok(SpannedTyped::new(&cached_span, &e.typ, e.x.clone()))
        })
        .unwrap();
        let install = |source: &Function, post: Expr, provenance: &str| {
            Contract::new(boolean(&source.span, true).expr, post)
                .unwrap()
                .install_lifted(source, provenance)
                .unwrap()
        };
        let cold = install(&original, cold_post.clone(), "same dependencies");
        let cached = install(&original, cached_post.clone(), "same dependencies");
        assert_eq!(cold.x.attrs.infer_candidate_id, cached.x.attrs.infer_candidate_id);
        // Hash normalization must not replace the installed diagnostic nodes.
        assert!(Arc::ptr_eq(&cold.x.ensure.0[0], &cold_post));
        assert!(Arc::ptr_eq(&cached.x.ensure.0[0], &cached_post));
        let false_proposal =
            install(&original, boolean(&original.span, false).expr, "same dependencies");
        assert_ne!(cold.x.attrs.infer_candidate_id, false_proposal.x.attrs.infer_candidate_id);
        let other_dependencies = install(&original, cold_post.clone(), "other dependencies");
        assert_ne!(cold.x.attrs.infer_candidate_id, other_dependencies.x.attrs.infer_candidate_id);
        let mut changed_source = original.x.clone();
        changed_source.body = Some(boolean(&original.span, false).expr);
        let changed_source = original.new_x(changed_source);
        let other_body = install(&changed_source, cold_post, "same dependencies");
        assert_ne!(cold.x.attrs.infer_candidate_id, other_body.x.attrs.infer_candidate_id);
    }
}

impl Contract {
    pub(super) fn new(pre: Expr, post: Expr) -> Result<Self, VirErr> {
        if !matches!(&*pre.typ, TypX::Bool) || !matches!(&*post.typ, TypX::Bool) {
            return Err(unsupported(&pre.span, "nonboolean inferred contract"));
        }
        Ok(Self { pre, post })
    }

    pub(super) fn install(self, original: &Function, provenance: &str) -> Result<Function, VirErr> {
        let (pre, post) = simplify::contract(original, &self.pre, &self.post)?;
        Self::new(pre, post)?.install_lifted(original, provenance)
    }

    /// Scalar formulas have already been simplified before lifting. Source
    /// wrapper constructors/projections need no second scalar simplification.
    /// This shares the same body-preservation and evidence-binding boundary.
    pub(super) fn install_lifted(
        self,
        original: &Function,
        provenance: &str,
    ) -> Result<Function, VirErr> {
        self.install_body(original, provenance, loops::annotate(original)?)
    }

    pub(super) fn install_owned(
        self,
        original: &Function,
        provenance: &str,
        body: Expr,
    ) -> Result<Function, VirErr> {
        owned::validate_decoration(original, &body)?;
        self.install_body(original, provenance, Some(body))
    }

    fn install_body(
        self,
        original: &Function,
        provenance: &str,
        body: Option<Expr>,
    ) -> Result<Function, VirErr> {
        let Self { pre, post } = self;
        let nonlinear = original.x.attrs.nonlinear
            || !original.x.attrs.infer_owned
                && !records::candidate(original)
                && loops::recognize(original)?.is_some_and(|plan| plan.nonlinear);
        // Bind outcomes to the original typed body, the actual proposed formulas
        // (including cache contents), configuration and transitive dependencies.
        // Cache decoding reconstructs proposal expression locations. Normalize
        // those locations only for hashing, keeping types/operators/binders and
        // the complete original source/dependencies bound below. The installed
        // formulas retain their original diagnostic spans.
        let canonical = |e: &Expr| {
            crate::ast_visitor::map_expr_visitor(e, &|e| {
                Ok(SpannedTyped::new(&original.span, &e.typ, e.x.clone()))
            })
        };
        let pre_identity = canonical(&pre)?;
        let post_identity = canonical(&post)?;
        let id = Sha256::digest(format!(
            "verus-infer-candidate-v4:{original:?}:{pre_identity:?}:{post_identity:?}:{body:?}:{nonlinear}:{provenance}",
        ));
        let mut f = original.x.clone();
        f.body = body;
        f.require = Arc::new(vec![pre]);
        f.ensure = (Arc::new(vec![post]), Arc::new(vec![]));
        let attrs = Arc::make_mut(&mut f.attrs);
        attrs.spinoff_prover = true;
        attrs.nonlinear = nonlinear;
        attrs.infer_candidate_id = Some(id.iter().map(|b| format!("{b:02x}")).collect());
        Ok(original.new_x(f))
    }
}
