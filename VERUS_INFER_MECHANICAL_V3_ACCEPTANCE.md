# Mechanical native adoption v3 acceptance

Status: complete, October 30, 2026. The authorized independent audit concludes
**MET** for the [frozen finite milestone](VERUS_INFER_MECHANICAL_V3.md), with no
blocking objections, source corrections or requirement changes.

## Accepted implementation and results

The audit used clean fixed Verus `fbab46ec4c5a1dd600c80d094ea7fe4fb90e34dd` and
Aleph proposal/evidence `53a8917b40bf934a4e3b10668d6f93c6c0ef3e98`, whose native
implementation is `8b7e0f108a5568e82db7efe898c50380cdff29d6`. It read the actual
source, authored semantic requirements, general engine guidance and final
evidence, without previous Aleph proof solutions or parent hints. It performed
the review read-only and did not mutate the shared source or pinned artifacts.

Fresh offline/locked Cargo verification under Rust 1.99.1, a new target directory
and a new candidate cache establishes:

- 154 verified results with zero errors across crypto, types and consensus;
- 35 exact helpers, all 105 required soundness/maximality/exactness checks proved,
  with 35 fresh candidate misses;
- 83 checked executable bodies and one compiler-generated proof body;
- separate same-source native testing under Rust 1.85.1: 192 unit tests and one
  doctest passed, with zero failed, ignored or filtered tests;
- 17 independent direct generic cases and the unrelated generic Cargo fixture,
  covering composition, semantic negatives and withheld publication.

The two removed production helpers account for the lower body/result counts
relative to v2. Types contributes interfaces and zero checked executable bodies.
The compiled inventory separately identifies 671 native functions outside the
selected proof closure; the 83 bodies include generated methods and coverage
callers, not 83 production APIs. Successful native compilation does not certify
outside-scope bodies.

## Three native patterns and semantic acceptance

1. **Derived assignment.** The actual derive_more AddAssign implementation for
   NodeCount is selected and checked. Its extension model is generated, its
   inherited domain is checked, and all three exact obligations pass. The
   collector composes its original `direct_parents += NodeCount(1)`. There is no
   authored arithmetic model or primitive-field rewrite. Unrelated unsigned and
   signed wrappers, nested consumers and cross-crate use exercise the mechanism.
2. **Inline constructor fold.** The complete original `ControlHash::new` body
   initializes the local map, traverses sparse parents and constructs the record
   with its opaque hash suffix. Its ordinary contract proves the full positional
   round projection for arbitrary hash payloads and all member counts. No
   extracted helper or exact whole-result/hash claim is used.
3. **Growth and mutable return.** The original Creator selector retains its
   while loop and indexed mutable return. Generated checked support establishes
   selected-value identity, state frames and conditional validity restoration.
   Earlier collectors/configuration, requested target and membership are
   preserved throughout the raw `round < u16::MAX` domain. Checked terminal
   rejection remains explicit; no extracted growth helper is used.

Complete constructor/selector executable tokens match the pinned original
export. The audit also reviews the retained quorum domain, initial arbitrary
creator IDs and zero membership, positive creation coverage, raw/checked round
boundaries, complete checked error ordering and actual creation-output numeric
validation eligibility. Generic Hash Clone uses its checked relation, without
identity. Opaque hash comparisons retain only their conditional existing
equality-law guarantee, without determinism or encoding equations.

The authored-source inventory finds semantic definitions and explicit
application goals, with no local supporting proof blocks, executable-loop
annotations, manual trigger hints, assumed first-party implementation, mechanical
getter clauses or authored mutable-selector frames. Views, validity predicates,
semantic interfaces, policies and coverage remain author input. The compiler's
proposals still face ordinary body proof and required exact certificate checks.

The auditor inspected the raw final evidence for three deliberate native edits:
wrong counter increment, wrong projected round and wrong selected coordinate.
All fail ordinary application proof after healthy helper certificates and
withhold publication. It reconstructs their source hashes and checks the
restored full run, then independently replays the accepted proof and native
tests. No-verify and filtered application attempts also cannot publish. The
mutations were reviewed without rerunning a controller against shared source;
they are fault injections, not original Aleph bug discoveries. The parent engine
suite's 631 passing checks remain separate recorded evidence.

## Preserved independent evidence and closing checks

The [audit archive](evidence/verus-infer-mechanical-v3/independent-audit.tar.gz)
preserves the independent report byte-for-byte as `ACCEPTANCE_REPORT.md`, every
one of its 183 indexed evidence files, its original index, frozen requirements,
pin receipt, original exports and separate parent additions. The
[bundle index](evidence/verus-infer-mechanical-v3/bundle-index.json) records all
193 member sizes/hashes and the archive SHA256. All report-relative evidence
links resolve after extraction into a fresh directory. Read `commands.md` there
for replay commands; the recorded absolute engine/application paths require the
specified local checkouts and toolchains.

The [parent closure receipt](evidence/verus-infer-mechanical-v3/parent-closure-receipt.json)
records independent-output validation, all member/hash checks and current input
integrity. All nine pinned compiler/library/Z3 artifacts, 1,739 tracked engine
files, 154 tracked proposal files and original exports match the audit inputs;
the fixed repositories were clean at those commits at the independent audit and
initial closure. The frozen documents are preserved before the completion
update. Full identities and historical parent evidence remain in
[VERUS_INFER_MECHANICAL_V3_EVIDENCE.json](VERUS_INFER_MECHANICAL_V3_EVIDENCE.json).

The auditor compared the supplied original export independently but could not
resolve its original commit in the proposal repository. A separate parent check
after the audit resolves actual local Git objects at
`f35c7bba6d628bcafdf0d65dd471cc6f868ec034` in the existing original worktree and
matches all three exports and committed references. Its receipt is archived as
`parent-original-git-provenance.json`. This supplements the original-input
provenance without changing the independent report or asserting remote origin.

## Scope and remaining limits

This accepts the selected finite proof closure and three supported patterns.
Verus, Rust, Z3 and imported vstd models remain trusted, along with the explicitly
weak codec/hash/lock interfaces. No stronger dependency law, first-party
assumption, narrower input domain or Signed/UncheckedSigned serializer exclusion
was introduced. Unverified callers are not automatically proved to establish
the contracts' preconditions.

Protocol agreement/liveness, outer signed validation, cryptographic correctness,
serialization correctness, hash-cache consistency, arbitrary loops/recursion
and unrelated codebases remain outside this milestone. Native tests and mutation
checks are finite regressions; the semantic proofs quantify over their stated
domains. All completed work remains local; nothing is pushed.

## Native evidence mirror

After acceptance, native fixture commit
`a2de660d5467ce3611584768285b9227a1e2832b` additionally preserves the independent
report and indexed raw evidence alongside the application. It changes only
`verification/mechanical_v3` evidence/documentation. The proof/native source
continues to equal the audited implementation; the audited input remains
`53a8917b`. The parent confirms current source/artifact integrity in
[the final receipt](evidence/verus-infer-mechanical-v3/post-audit-integrity.json).
