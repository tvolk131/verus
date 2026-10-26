//! Evidence binding and acceptance for locally checked inference candidates.
//! Imported metadata uses the shared policy but remains a trusted artifact.
use crate::ast::{Fun, Function, FunctionAttrs, VirErr};
use crate::infer::ProofStatus;
use crate::messages::{Span, error};
use std::collections::HashMap;

pub mod policy;
pub use policy::{Aggregate, Obligation, Outcome};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateId {
    function: Fun,
    stamp: String,
}

impl CandidateId {
    pub fn from_parts(name: &Fun, attrs: &FunctionAttrs, span: &Span) -> Result<Self, VirErr> {
        let stamp = attrs
            .infer_candidate_id
            .as_ref()
            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or_else(|| error(span, "Verus Infer: missing or invalid candidate identity"))?;
        Ok(Self { function: name.clone(), stamp: stamp.clone() })
    }

    pub fn from_function(f: &Function) -> Result<Self, VirErr> {
        Self::from_parts(&f.x.name, &f.x.attrs, &f.span)
    }
}

#[derive(Clone, Debug)]
struct Entry {
    id: CandidateId,
    evidence: policy::Evidence<CandidateId>,
}

#[derive(Clone, Debug, Default)]
pub struct Proofs {
    entries: HashMap<Fun, Entry>,
    invalid: bool,
}

impl Proofs {
    pub fn register(&mut self, f: &Function) -> Result<(), VirErr> {
        let id = match CandidateId::from_function(f) {
            Ok(id) => id,
            Err(e) => {
                self.invalid = true;
                return Err(e);
            }
        };
        if self.invalid
            || !f.x.attrs.infer
            || f.x.body.is_none()
            || self.entries.contains_key(&id.function)
        {
            self.invalid = true;
            return Err(error(&f.span, "Verus Infer: invalid or duplicate candidate registration"));
        }
        self.entries
            .insert(id.function.clone(), Entry { evidence: policy::Evidence::new(id.clone()), id });
        Ok(())
    }

    pub fn record(
        &mut self,
        id: &CandidateId,
        kind: Obligation,
        outcome: Outcome,
        span: &Span,
    ) -> Result<ProofStatus, VirErr> {
        if self.invalid || !self.entries.contains_key(&id.function) {
            self.invalid = true;
            return Err(error(span, "Verus Infer: outcome for unregistered or invalid candidate"));
        }
        let entry = self.entries.get_mut(&id.function).unwrap();
        if let Err(e) = entry.evidence.record(id, kind, outcome) {
            self.invalid = true;
            return Err(error(span, format!("Verus Infer: invalid proof evidence: {e:?}")));
        }
        Ok(status(&entry.evidence))
    }

    pub fn status(&self, f: &Function) -> Result<ProofStatus, VirErr> {
        if self.invalid {
            return Err(error(&f.span, "Verus Infer: invalidated proof evidence"));
        }
        let id = CandidateId::from_function(f)?;
        let entry =
            self.entries.get(&id.function).filter(|entry| entry.id == id).ok_or_else(|| {
                error(&f.span, "Verus Infer: candidate does not match proof evidence")
            })?;
        Ok(status(&entry.evidence))
    }

    /// Reporting uses the same candidate identity and poison checks as acceptance.
    /// No caller can create or modify evidence through this interface.
    pub fn outcomes(&self, f: &Function) -> Result<[Option<Outcome>; 3], VirErr> {
        self.status(f)?;
        self.entries[&f.x.name]
            .evidence
            .outcomes()
            .ok_or_else(|| error(&f.span, "Verus Infer: invalidated proof evidence"))
    }

    /// Worker verifiers receive identities, never outcomes from another run.
    pub fn fresh(&self) -> Self {
        Self {
            invalid: self.invalid,
            entries: self
                .entries
                .iter()
                .map(|(name, e)| {
                    (
                        name.clone(),
                        Entry { id: e.id.clone(), evidence: policy::Evidence::new(e.id.clone()) },
                    )
                })
                .collect(),
        }
    }

    pub fn merge(&mut self, other: Self) -> bool {
        // Inferred buckets run sequentially. Ordinary parallel workers must not
        // produce inference evidence; never overwrite results while merging.
        self.invalid |= other.invalid || other.entries.values().any(|e| !e.evidence.is_empty());
        !self.invalid
    }
}

fn status(e: &policy::Evidence<CandidateId>) -> ProofStatus {
    let [soundness, maximality, exactness] = e.passed();
    ProofStatus { soundness, maximality, exactness }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input};
    use std::sync::Arc;

    fn candidate(name: &str) -> Function {
        crate::infer::infer_function(&function(name, input("x"))).unwrap()
    }

    fn record_all(proofs: &mut Proofs, f: &Function) {
        let id = CandidateId::from_function(f).unwrap();
        for kind in [Obligation::Soundness, Obligation::Maximality, Obligation::Exactness] {
            proofs.record(&id, kind, Outcome::Valid, &f.span).unwrap();
        }
    }

    #[test]
    fn evidence_cannot_move_between_candidates_functions_or_workers() {
        let first = candidate("first");
        let second = candidate("second");
        let mut proofs = Proofs::default();
        proofs.register(&first).unwrap();
        proofs.register(&second).unwrap();
        record_all(&mut proofs, &first);
        assert!(proofs.status(&first).unwrap().satisfies(false));
        assert!(!proofs.status(&second).unwrap().satisfies(true));
        assert!(!proofs.fresh().status(&first).unwrap().satisfies(true));
        assert!(proofs.merge(proofs.fresh()));
        let mut worker = proofs.fresh();
        record_all(&mut worker, &first);
        let mut merge_target = proofs.clone();
        assert!(!merge_target.merge(worker));
        assert!(merge_target.status(&first).is_err());
        let mut changed = first.x.clone();
        Arc::make_mut(&mut changed.attrs).infer_candidate_id = Some("a".repeat(64));
        let changed = first.new_x(changed);
        assert!(proofs.status(&changed).is_err());
        assert!(
            proofs
                .record(
                    &CandidateId::from_function(&changed).unwrap(),
                    Obligation::Soundness,
                    Outcome::Valid,
                    &first.span
                )
                .is_err()
        );
        assert!(proofs.status(&first).is_err());
        assert!(
            Proofs::default()
                .record(
                    &CandidateId::from_function(&first).unwrap(),
                    Obligation::Soundness,
                    Outcome::Valid,
                    &first.span
                )
                .is_err()
        );
    }

    #[test]
    fn registration_and_identity_are_required_and_duplicates_do_not_overwrite() {
        let f = candidate("candidate");
        let mut proofs = Proofs::default();
        assert!(proofs.register(&function("unstamped", input("x"))).is_err());
        assert!(proofs.register(&f).is_err());
        let mut proofs = Proofs::default();
        proofs.register(&f).unwrap();
        record_all(&mut proofs, &f);
        let mut duplicate = proofs.clone();
        assert!(duplicate.register(&f).is_err());
        assert!(duplicate.status(&f).is_err());
        let id = CandidateId::from_function(&f).unwrap();
        assert!(proofs.record(&id, Obligation::Exactness, Outcome::Invalid, &f.span).is_err());
        assert!(proofs.status(&f).is_err());
    }

    #[test]
    fn reporting_preserves_outcomes_and_cannot_supply_evidence() {
        let f = candidate("reported");
        let mut proofs = Proofs::default();
        assert!(proofs.outcomes(&f).is_err());
        proofs.register(&f).unwrap();
        assert_eq!(proofs.outcomes(&f).unwrap(), [None; 3]);
        let id = CandidateId::from_function(&f).unwrap();
        proofs.record(&id, Obligation::Soundness, Outcome::Valid, &f.span).unwrap();
        proofs.record(&id, Obligation::Maximality, Outcome::Incomplete, &f.span).unwrap();
        proofs.record(&id, Obligation::Exactness, Outcome::Invalid, &f.span).unwrap();
        let snapshot = proofs.outcomes(&f).unwrap();
        assert_eq!(
            snapshot,
            [Some(Outcome::Valid), Some(Outcome::Incomplete), Some(Outcome::Invalid)]
        );
        let mut edited = snapshot;
        edited[1] = Some(Outcome::Valid);
        assert_ne!(proofs.outcomes(&f).unwrap(), edited);
        assert!(!proofs.status(&f).unwrap().satisfies(false));
        assert!(proofs.status(&f).unwrap().satisfies(true));
        assert_eq!(proofs.fresh().outcomes(&f).unwrap(), [None; 3]);
        assert!(proofs.record(&id, Obligation::Exactness, Outcome::Valid, &f.span).is_err());
        assert!(proofs.outcomes(&f).is_err());
    }
}
