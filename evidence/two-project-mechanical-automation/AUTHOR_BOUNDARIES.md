# Author work versus mechanical automation

November 3, 2026. Cycle 1 is in progress. This is an accounting of the frozen
goal and the installed combined candidates, not final acceptance or a new scope.
`FEDIMINT_SCOPE.md`, `REDB_SCOPE.md` and `COMPLETION_MATRIX.md` remain authoritative.

## What remains the author's job

- Choose the theorem: fee-validation decisions and bounds, or the exact
  byte-ID/slot-policy/field-preservation relation. Infer does not select business
  requirements, crash models or the meaning of a valid transaction.
- State intent and abstraction relations in checked specifications. Keep the
  original executable bodies, representations and safe domains; identify every
  extracted block, omitted caller operation and authored adapter.
- Supply semantic invariants and checked domain lemmas where needed. Fedimint's
  existing component-monotonicity theorem remains authored and checked; reusing
  it is not newly automatic discovery of fee policy.
- Make external boundaries explicit. Existing standard-library correspondences
  remain trusted. Native-item hashing and caller correspondence audits are not
  a verified extraction compiler. The two approved redb opaque datatype
  declarations give no new error, destructor or storage behavior contract.

## Fedimint — target scope reviewed; final integration pending

The baseline needed two handwritten contracts for the original native
`Amount::checked_add` and `PaymentFee::checked_add` bodies. The manual control
verified 6 obligations with those contracts; removing them exposed the frozen
nested-Option inference limitation. Named-field comparison derives and the
second borrowed record also needed reusable support.

The installed candidate instead requests Infer on the unchanged native helper
bodies and checks the real generated named-field comparison bodies. It
synthesizes total-domain checked-add summaries, nested `Option`/`?` payload
relations and independent borrowed-field relations. Both manual helper contracts
are removed. No nonoverflow precondition replaces the `None` behavior.

The author still writes the acceptance/error-order characterization, the
fee-bound intent and the pure adapter corresponding to the two pinned native
validation regions. The candidate uses inferred helper summaries and the
existing checked monotonicity theorem to discharge those caller properties.
Its provider55/0 and imported-consumer4/0 results have separate target review;
the two reported acceptance-gate defects were repaired and independently accepted.
New final integrated-runtime validation and independent integration acceptance
remain required. Generic u8/u64 and import regressions do not replace those checks.

The pure/error-payload adapter is not either whole native async/configuration
caller. Lookup, formatting, assignments, save/commit and backend behavior stay
outside the theorem. No funds-safety or application-wide correctness claim follows.

## redb — frozen R1 accepted; final integration pending

The delivered reusable automation covers the canonical slice-to-array conversion,
little-endian decoding, original `size_of`/constant/helper dependencies and
composition/frame reasoning. The target must retain both exact native IDs,
complete field/corrupt-byte preservation, original branch-sensitive safe domains,
exact errors and the frozen clean-result characterization.

The author supplies the desired byte/record relation, the explicitly labeled
adapters for three exact native blocks and arbitrary root/corruption/layout
context inputs. These inputs are not assumed checksum results. No authored
byte/cast/overflow/algebraic proof plumbing or narrowed buffer/index domain may
stand in for the frozen automation requirement. The delivered candidate reports
47 verified obligations and zero errors for the complete composed adapter in
both debug modes, with no authored local mechanical lemmas or assertions. That
result is independently accepted at80/74/38. Fresh final-runtime replay and a new
independent integration review remain required; this is not final goal acceptance.

Whole decoding/finalization methods, checksum correctness, root parsing,
layout reconciliation, storage callers, recovery and durability remain unproved.

## Combined source integration

Both independently reconstructed application deltas are installed. Their only
overlapping production edits were cache/policy identity declarations; those are
reconciled to cache82/policy76/schema38. No sibling implementation is substituted
for another, and the two repaired Fedimint decision gates remain installed.
The complete candidate source ledger has 2,191 paths before final compatibility
fixtures and parent-only files. Extra parent work and the Git index are preserved.

The final integrated compiler/macros build passes offline and locked, with all
1,100 guarded Rust/build/test inputs unchanged. Matching vstd freshly verifies
2,221 obligations. A new final producer/model qualification preserves the accepted
43-row native scope and all 7,440 exported library definitions. The model inventory
grows from381 to382 solely because the existing canonical slice-to-array
correspondence is now eligible. Actual certificate fingerprints are pinned; no
successful raw-AST fingerprint recomputation is claimed. Both complete target
matrices, cross-feature regressions and final independent integration review
still require their own final-runtime evidence.

## Finish-line discipline

Both targets must pass their full frozen positive/negative controls on a matched
integrated runtime, preserve the accepted baseline/native sources, pass regression
and evidence-acceptance tests, and receive new independent integration acceptance.
Only then can the goal be complete. Neither a helper-only proof nor a polished
receipt may substitute for either composed application theorem.
