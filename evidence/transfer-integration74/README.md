# Reviewed transfer composition and native-publication fix

## Scope and identities

This uncommitted follow-up is based on checkpoint `3b05c1912b9d7314ad00c832c4065f4362562e02` and its
`verus-infer` ancestor `55bd67ad980e90e92c77efe4453a927f097c3386`, not main
or the differing local verus-infer ref. PR10 and its checkpoint branch remain
unchanged. No merge authorization, follow-up commit, PR or hosted run is implied.
The reconciled engine is cache74 / policy68 / schema38.

This report preserves the pre-correction validation history. A subsequent
independent harness review found relational receipt-consistency gaps, not a
false compiler proof. The corrections and fresh acceptance/replay validation
are tracked separately in `evidence/transfer-receipt-hardening74/README.md`;
the original frozen final74 handoff and receipts remain unchanged.

The independently reviewed polynomial/unused-opaque72 composition, reviewed
numeric73 non-marker production files, and six-scenario harness are integrated.
The receipt acceptor and its fixture explicitly match policy68 instead of66;
other incompatible policies remain rejected. Current guidance documents the
new conversion and native-publication boundaries. Historical evidence is not
rewritten, and numeric access remains dedicated rather than a seventh common
CI registration. Independent harness and final-composition reviews are tracked
separately; this parent validation is not a substitute for either review.

## Native-failure publication repair

The numeric review found a pre-existing lifecycle bug: successful helper proofs
published an inference-aware interface before the requested native pass, which
could fail while leaving `interface_written=true`. This was not a false numeric
theorem or a new conversion defect. `driver::run` now finalizes pending inferred,
import-dependent and goal-assisted exports only after requested native/erasure
checking succeeds. Compiler diagnostics, verification errors and atomic export
errors still block publication. Legacy ordinary export behavior is unchanged.

The identical portable `--compile --no-cheating` reproduction fails natively
both before and after. Sealed73 leaves an interface; sealed74 does not and
reports `interface_written=false`, with all three helper obligations still
proved. Three regression tests cover fresh/inferred/ordinary seeded artifacts,
import-only and goal-only consumers, successful native publication and explicit
verification-only publication. Skipped native checks are not native evidence.

Before: `/private/tmp/verus-infer-final74._6rqc4bo/native-failure-before/receipt.json`.
After: `/private/tmp/verus-infer-final74._6rqc4bo/native-failure-after/receipt.json`.

## Fresh parent validation

- Matching offline/locked release compiler/macros build, two Cargo jobs.
- Fresh vstd: 2,221 verified, zero errors.
- VIR library: 140 passed; unrelated doctests are not included.
- 32 integration binaries: 592 passed, zero failed or ignored, including core.
- Python discovery: 213 passed normally and 213 under -O.
- YAML, seven shell blocks and ten deadlines checked; 80 active minutes,
  five upload and five margin within the 90-minute job. Formatting/diff checks.
- All six scenarios pass through the new common frontdoor with sealed74.
- Dedicated redb access: 39 controls, including portable native/no-cheating
  and configured 32/64-bit results; no native 32-bit execution claim.

Common receipt: `/private/tmp/verus-infer-final74._6rqc4bo/six-scenarios-release/transfer-suite-3vf1srhk/receipt.json`.
Access receipt: `/private/tmp/verus-infer-final74._6rqc4bo/access-prefix-release/receipt.json`.
Dependency preparation: `/private/tmp/verus-infer-final74._6rqc4bo/dependency-preparation/receipt.json`.
Preparation copied only 21 real pinned registry inputs into a fresh Cargo home,
validated checksums/archives/source/graph, and stayed offline. It is not an
online fetch or a full application feature-equivalence proof. Proof runners
record their exact commands, exits, source/runtime guards and raw reports.

The first common replay failed closed because its independent acceptor still
required policy66 while the runtime correctly emitted68. No theorem failed.
A diagnostic attempt to reuse its old receipt after the source correction also
correctly failed source-ledger validation. Neither is accepted evidence; fresh
complete replays with corrected policy and current documentation are above.
The initial setup wrapper's reserved-zsh-variable error occurred after successful
preparation; its successful receipt was inspected without overwriting it.

## Theorems and boundaries

redb codec, page-size, range, same-order ordering/nonoverlap, zero-size boundaries
and mixed-order overlap remain proved in their selected scopes. The additional
access extraction preserves original conversion syntax/helper bodies and proves
range/length consistency with full arithmetic and usize representability
conditions. It uses a three-field readonly configuration projection, appends a
length return, omits check_page_order/tracking/construction and stops before
storage.read. No allocator occupancy, IO, concurrency or whole-get_page/database
theorem follows. No order-at-most-20 shortcut is introduced.

Fedimint retains all seven real request fields and both original deadline helper
items. The policy/coverage/boundary/monotonicity/unused-field properties remain
proved; guard deletion remains an exact helper but fails authored policy.
Opaque SHA256 identity permits only unused shared-borrowed transport, not hash
observations, equality/layout/trait/crypto laws, authenticated inputs or funds
safety. Async persistent state and external effects still block the funding caller.

Existing native std/Option/Range/primitive specifications and frontend identities
remain explicit trust, with ordinary Rust/Verus typing/erasure, vstd, backend and
solvers. No new axiom or unchecked application summary is introduced. Authored
intent is not inferred. Semantic, arithmetic/panic, frontend/import and checked
auxiliary failures remain separate; cold/warm regeneration is not a cache hit.
Temporary-receiver shapes, casted literal one and U128 shift-amount completeness
limits remain documented, not unsound accepted contracts or repaired features.
