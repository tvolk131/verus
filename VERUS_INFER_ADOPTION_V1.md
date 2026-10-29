# Aleph adoption v1

Status: active. Baselines: Verus `8cdf05382`, native Aleph pilot `7410c7b`.
This is a finite adoption milestone, not completeness for arbitrary Rust.

## Objective and fixed slice

Verify the actual sequential path from lawful unit observations through collector
updates, parent selection and PreUnit construction, and the corresponding parent
metadata validation. Supply routine implementation summaries and proof
scaffolding through reusable inference and checked goal assistance.

The native pilot is the isolated `verus-infer-clean-pilot` Aleph worktree. The
original development checkout is not an input. Fixed first-party functions are:

- Crypto: the NodeCount arithmetic/conversion/comparison operations used by
  `consensus_threshold`; NodeMap `with_size`, `size`, `get`, `insert`, `iter`,
  `values`, `fill_prefix`, `try_fill_prefix`, and the required clone implementation.
- Collector: `new_initial_state`, `new_initial`, `from_previous`,
  `add_observation`, `add_unit`, `prospective_parents`.
- Creator: `new`, `current_round`, `ensure_round`,
  `get_or_initialize_collector_for_round`, `try_initialize_round`, `add_unit`,
  `try_add_unit`, `create_unit`.
- Metadata: UnitCoord construction/getters; PreUnit construction/getters;
  ControlHash construction, parent projection/iteration, member count, public
  validation and its initial/noninitial helper branches.
- Observation connection: the already checked FullUnit Unit implementation.
  Generic Unit clients must discharge its declared observation law.

Async tasks, network delivery, agreement/liveness, signature verification,
canonical encoding, hash collision resistance and hash-cache consistency are
outside this slice. SignedUnit observation delegation remains a separate,
unapplied proposal; this goal does not authorize its serializer exclusions.

## Frozen semantic requirements and coverage

Views retain membership positions and missing entries. Collector validity means
positive target round, all stored rounds below target, and direct-parent count
exactly equal to the number of occupied preceding-round slots. Creator validity
means a nonempty bounded history with correctly indexed, sized valid collectors.
Views and validity are proved connections to fields, not assumed invariants.

- Preserve the existing initialization, update, replay/fork and round-advancement
  policies, including whole-state error preservation of checked APIs.
- Parent selection succeeds exactly when the quorum test passes and the configured
  node has a preceding-round parent; success returns the actual stored map. The
  quorum error precedes the own-parent error. Older additional parents are allowed.
- Quorum uses the original `floor(2*n/3)+1` policy. Its checked multiplication
  imposes `n <= usize::MAX/2`; this is recorded before implementation, not added
  later to satisfy a proof. Exact helper inference must recover its actual safe
  domain. Application creation covers all such member counts and in-range node
  IDs; zero membership retains constructor/ingestion/error behavior but has no
  in-range creator. Historical raw lookups retain their actual bounds obligations.
- Raw advancement/ingestion supports all rounds below `u16::MAX`; checked APIs
  also cover the terminal round and reject before mutation. Metadata validation
  covers both zero and positive rounds with the original error precedence.
- Parent projection preserves collection length, occupied creator positions and
  rounds. Successful creation has the requested coordinates and eligible parent
  metadata; round zero has no parents. All documented valid states and payloads
  are covered, not just a handpicked execution trace.

Hash computation uses the existing opaque Hasher/codec boundary. No new equation
about encoding, hashes, or their determinism is authorized. Metadata validation
must state explicitly which hash comparisons it establishes under that boundary;
constructor/validator hash agreement is not claimed without an adequate checked
or separately approved library contract. Numeric/parent eligibility claims must
remain independent of hash values. A newly discovered boundary is an unresolved
goal item, not permission to weaken a requirement or assume a useful equation.

## Permitted manual input

Authors provide mathematical views, validity predicates, meaningful public
behavior/coverage/error policies, and semantic generic/dependency interfaces.
Their implementations and laws must be checked or listed as existing trust
boundaries. Ordinary application goals may be written as contracts or assertions.
Definitions may express their own mathematical recursion/decreases.

The final production slice must not need application-local supporting proof
blocks, handwritten executable-loop invariants/decreases, manual trigger hints,
or mechanical helper contracts that merely restate field access, selected-borrow
identity or unrelated-state frames. Reusable checked library lemmas and
internally generated annotations are allowed. Native test assertions and explicit
application/coverage harnesses are allowed; they are not proof scaffolding.

Executable refactors must preserve behavior and be documented/tested. Do not
replace native implementations with extracted toy copies or assumed adapters.
Every new inference profile requires independent examples and negative cases.

## Acceptance and stopping rule

1. All fixed native functions are included in the checked interface closure, with
   a machine-readable inventory of authored semantic definitions/contracts and
   existing trusted boundaries. No silent external-method exclusions.
2. The frozen application requirements and input coverage verify. Every exact
   helper passes soundness, maximality and exactness checks. Ordinary authored
   application proofs need not themselves be exact inference candidates.
3. A source audit confirms the manual-input allowance above. Remaining mechanical
   annotations are reported as failures, not relabeled as application semantics.
4. Independent generic regressions, source mutations, full-verification gates,
   repeated provider/caller builds and withheld failed exports pass.
5. Native tests pass on the pinned toolchain. A fresh replay from pinned baselines
   plus the allowed semantic annotations reproduces verification without borrowing
   old proof scaffolding. Store commands, source/artifact identities and results.

Commit cohesive validated chunks without pushing. Completion means all five
criteria pass, not simply that the current examples verify. Changes to coverage,
requirements or trust boundaries require an explicit recorded decision.

## Initial gaps

- Native NodeCount overloaded arithmetic and wrapper-result queries.
- Native parent selection is currently external; metadata methods and creation
  are not yet checked.
- Collector advancement still has a quantified proof block.
- Creator and checked collection contracts still contain manual trigger hints.
- Mutable selection identity/frames are still authored mechanical helper clauses.
- Opaque codec hashing limits any stronger initial-round hash agreement claim.

## Recorded scope decisions

2026-10-29, explicitly approved by the user: ConstraintError's generated Display
and std::error::Error implementations may remain unverified while the enum,
selector and error-ordering claims are checked. No formatting/error-method
contract or assumption is added. This decision does not authorize the separate
Signed/UncheckedSigned serialization proposal, which remains unapplied.

## Intermediate progress

The original NodeCount arithmetic/conversion/ordering bodies, NodeMap size and
quorum expression now verify through reusable checked derive/operator support.
The original parent selector now has an inferred contract and authored goals for
quorum, preceding-round own-parent selection, complete stored-map logical value,
returned contents and error precedence. Early quorum failure covers zero members
and arbitrary indices. Wrong quorum/round/error source mutations are rejected.

Native metadata projection/construction, all initial/noninitial validation
branches, UnitCoord and PreUnit construction/accessors now verify as well.
PreUnit's view retains its requested coordinates and the complete supplied
ControlHash value. Reusable inference preserves opaque moved inputs and composes
readonly queries through authored projection observations. Numeric metadata
construction remains separate from the checked opaque hash call; no hash law or
new assumed contract was added.

The PreUnit checkpoint has 140 proof results, 31 exact helpers and all 93 required
certificates. Three native PreUnit mutations fail their independent coverage
goals, and the pinned runtime's 40 creation tests pass. Evidence is recorded in
the native pilot's verification/units_collector/PREUNIT.md and its receipt file.

Creator::create_unit now verifies over the frozen domain, including every u16
round, initial creation with arbitrary node IDs, and zero-membership error
coverage. Its authored outcome checks coordinates, the exact selected-parent
round projection, quorum/own-parent policy and error order. Separate goals check
initial absence of parents and positive-round parent positions and bounds.

The private Creator API now returns its existing ConstraintError directly rather
than boxing it into anyhow::Error. Existing callers only inspect success or log
the same Display messages; all call sites compile and the pinned runtime tests
pass. This removes the unnecessary erased-error conversion without adding an
assumed contract or extending dependency exclusions. The algorithm and branch
order are unchanged. The new constructor tuple-view profile proves a uniqueness
postcondition and both precision checks, enabling exact opaque result forwarding
across modules/crates without exposing private constructors.

The creation checkpoint records 145 proof results, all 31 exact helpers and 93
certificates, five rejected native mutations and 41 pinned creation runtime tests.
Evidence is in the native pilot's verification/units_collector/CREATION.md and
creation-evidence.json. The engine passes 463 compiler and 114 backend tests.

Collector advancement's local proof block and every source trigger hint in the
collector, Creator and NodeMap slice have now been removed. Reusable ordinary
goal assistance supplies a checked pointwise assertion through the existing clone
relation and an internal index trigger for hidden range predicates. Original
executable statements, supplied contracts and the hash boundary are unchanged.
The native proof still has 145 results and all 93 exact-helper certificates.
Five actual-source mutations are rejected, all 41 pinned creation runtime tests
pass, and the engine passes 466 compiler and 115 backend tests. Evidence is in
the native pilot's verification/units_collector/SCAFFOLD_REMOVAL.md and
scaffold-evidence.json.

The mutable-selector helper's six mechanical identity, frame, length and
conditional-restoration clauses are now removed as well. Reusable goal assistance
proposes those ordinary postconditions from the direct vector borrow and the
checked preparation contract. The author's input conditions and the returned
collector's validity, round and size semantics remain unchanged. Ordinary Verus
checks the generated clauses against the original helper body.

The interface publication safeguard now also follows certified dependencies
through ordinary intermediate APIs. Their artifact header requires the proof
trailer even when their public API has no inferred function. Such consumers must
verify their entire crate and wait for successful verification before publishing.

This checkpoint passes 470 compiler regressions, 116 backend tests and the driver
metadata test. Native evidence retains all 145 proof results and 93 certificates,
41 pinned creation runtime tests, six rejected implementation mutants and a
no-assistance control. Evidence is in the native pilot's
verification/units_collector/SELECTOR.md and selector-evidence.json.

The hash boundary, full authored-input inventory audit, final safeguard suite and
fresh replay remain part of final acceptance. This checkpoint does not complete
adoption v1.
