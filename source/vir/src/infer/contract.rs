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
        let id = Sha256::digest(format!(
            "verus-infer-candidate-v3:{original:?}:{:?}:{:?}:{body:?}:{nonlinear}:{provenance}",
            pre, post,
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
