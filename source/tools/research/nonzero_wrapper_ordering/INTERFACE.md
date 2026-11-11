# Typed interface and original-body freeze

Design only until the baseline milestone is committed. The following are
proposed in-memory checker boundaries, never serialized evidence or proof tokens.

```
WrapperRequest { owner: rustc Ty<Adt>, ord: Instance, partial: Instance,
                 carrier: ConcreteOrderingInstances, origin: HIR identities }
resolve_wrapper_request(tcx, selected_original_derive) -> Result<WrapperRequest>
check_wrapper_bodies(tcx, request, actual_MIRs, actual_Functions) -> Result<()>
validate_derived_requests(actual_functions, datatypes) -> Result<()>
```

Resolve a local nongeneric single-field real core NonZero<u64> owner, safe Rust
signatures (&Owner,&Owner)->Ordering/Option<Ordering>, actual core traits and
original automatically-derived items. Markers only select/check candidates.
Syntactic NonZero spelling never grants semantic eligibility. Keep the private
field, closed value=self.0@ observation, all seven original native derives and
ordinary compiler type/region/Copy/no-drop/mode checks. Shared arguments may be
the same reference. Domain: every already-created genuine tag, with true requires.

Exact original source-body inventory (fixture generator in fixtures.py):
- native Copy/Clone/Hash/PartialEq/Eq/PartialOrd/Ord are retained;
- newly checked ordering bodies must be original Ord::cmp and
  PartialOrd::partial_cmp; requested PartialEq::eq and existing ordinary Clone
  handling must be inventoried separately; Hash and Eq remain excluded;
- total(&BorTag,&BorTag) -> Ordering invokes actual a.cmp(b);
- partial(&BorTag,&BorTag) -> Option<Ordering> invokes actual a.partial_cmp(b);
- less/less_equal/greater/greater_equal call inherited actual operators;
- same_reference uses a.cmp(a), a.partial_cmp(a); no distinctness premise;
- value(&BorTag)->u64 remains closed; candidate trait relations are closed.

Exact executable call/body dependency inventory to validate, not reconstruct:
1. original total: Ordering,&BorTag,&BorTag; one Call, return;
2. original partial: Option<Ordering>,&BorTag,&BorTag; one Call, return;
3. native derived Ord::cmp: Ordering,&BorTag,&BorTag,&NonZero<u64>,&NonZero<u64>;
   original shared field0 projections then carrier cmp; normal return/unwind Continue;
4. native derived PartialOrd::partial_cmp: Option<Ordering>,&BorTag,&BorTag,Ordering;
   actual wrapper cmp then Some(move result); normal return/unwind Continue;
5. actual core carrier cmp::<u64>: nine locals/two blocks,14 ordered entry
   statements, two checked inverse get expansions, actual primitive cmp,
   four ordered storage deaths, Return (Stage1 inspect_bodies/check_get_expansion);
6. actual get::<u64>: u64,NonZero<u64>; Copy/Transmute,Return, uninterpreted;
7. actual primitive u64Ord::cmp: five locals/seven statements/Return, bound to
   inherited u64 model only, not a newly proved native/std law.
8. logical ordinary checked_nonzero_u64_ord%: full actual Function checked before
   registration and after WF/autospec/mode; fresh successful Normal query required.
No source app mirror or synthetic native helper substitutes for items3/4/5.

Authored semantic relation: cmp_spec(a,b) = if value(a)<value(b) then Less else
if value(a)>value(b) then Greater else Equal; partial_cmp_spec=Some(cmp_spec).
obeys predicates propose true, and must be justified by actual original body
checks. Authored comparison candidates propose only numeric relations; each must
pass soundness, full inherited-domain safety maximality, and output exactness.
MAX/highbit/2^61 examples are regression support only, never the domain proof.

Minimal source-change candidates (not yet implemented):
- infer_derive_ops.rs: allow only PartialEq/Ord/PartialOrd for syntactically
  nongeneric one-NonZero<u64> field; emit numeric view-based closed relations;
- wrapper_binary.rs: allow only existing nonzero_owner for these three suffixes;
- wrapper_compare.rs/newtypes.rs: numeric view projection for bool ordering
  proposals only, preserve original body for all three checks;
- checked_nonzero_ord.rs: use existing exact typed source binding on actual
  generated carrier call; extend only if pinned derived HIR requires a closed
  explicit route, never broaden universal eligibility;
- existing external.rs already selects requested original Ord/PartialOrd VerifyAsIs;
  no blanket ordinary derive eligibility change is indicated by source review.
registry/global schema/certificates must remain unchanged. Independently check
actual generated bodies, all helpers and source fidelity before any success claim.

Current unresolved sequent: actual derived cmp on shared field projections
satisfies the full-domain authored numeric relation via the checked actual carrier;
then actual derived partial_cmp satisfies Some of that wrapper relation. Neither
marker selection, view projection nor Stage1 direct success proves this sequent.
