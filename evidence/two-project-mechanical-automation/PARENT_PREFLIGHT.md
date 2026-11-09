# Parent preflight evidence — not target completion

November 3, 2026. The two target scopes are frozen; cycle 1 implementation and
subsequent independent review remain pending. No new Infer capability or formal
application theorem is claimed by these parent preflight checks.

## Baseline regressions

Fresh general discovery executes 464 methods successfully in each normal and
optimized Python mode. Its one skipped entry is the receipt-dependent
`ReceiptGuards` class, not a hidden passing test. A separate invocation executes
all 43 methods of that class successfully in each mode, using the immutable
normal/optimized historical redb repair receipts. Thus 507 methods execute per
mode across these two invocations. Historical verifier results underlying those
receipts are not freshly replayed or counted as new formal evidence.

Commands, raw logs, hashes and explicit skip/provenance qualification are in
`/private/tmp/verus-two-project-goal.AZF2UH/baseline-regressions`.
All 2,174 accepted baseline file hashes match before and after general discovery.
These are baseline results, not a substitute for final candidate regressions.

## Independent source authenticity audit

The parent independently checks original native sources, not just the research
package seals. All 18 Fedimint ledger items, 19 redb ledger items and five redb
ledger fragments occur exactly at their recorded native lines. This authenticates
research inputs, including explicitly out-of-target contextual items; it does
not promote all those items into verified scope. All 1,331 native source hashes
match, both native pins/roots/cleanliness match, and all 2,174 parent baseline
hashes remain unchanged. Normal and optimized audit executions pass.

The initial audit attempt incorrectly hashed redb item containers including the
extra terminal LF added by the research packager. The corrected audit explicitly
checks that single-LF envelope convention and also checks the full container
against native source. No native/source mismatch was found; failed initial audit
attempts are not counted as successful checks.

Reproducer and successful receipts:
`/private/tmp/verus-two-project-goal.AZF2UH/audit_frozen_sources.py`,
`/private/tmp/verus-two-project-goal.AZF2UH/source-audit-normal-v2/receipt.json`,
`/private/tmp/verus-two-project-goal.AZF2UH/source-audit-optimized-v2/receipt.json`.

## Native gateway supporting oracle

An independent parent Rust harness compiles the unchanged native fee/Amount
helpers, relevant actual built-in derives, original limit constants and two
native Display implementations. An explicitly authored pure wrapper contains
the exact pinned gateway fee-validation block. A one-variant error context is
scaffolding, not the complete native AdminGatewayError or whole async caller.

Debug and release builds each check 83,521 four-component boundary combinations
against an independently authored u128 decision oracle. Both runs observe the
same classifications: 2,576 accepted, 30,160 overflow, 49,517 send-limit rejects,
and 1,268 receive-limit rejects. Concrete equality, either-component bounds,
exact-M addition and overflow-before-limit cases pass. Accepted cases check
12,880 sampled fee/amount pairs per build against actual native fee computation
and the intended limit consequences. Input hashes remain unchanged.

This is finite native supporting evidence, with ZERO universal Verus proof
credit. It does not verify async/configuration provenance, backend operations,
or all u64 values. It strengthens the explicit caller-correspondence seam;
the frozen formal full-domain target is still required.

Reproducer, exact source, compile/test commands, binary hashes, classifications
and raw logs are in
`/private/tmp/verus-two-project-goal.AZF2UH/native_gateway_oracle.py` and
`/private/tmp/verus-two-project-goal.AZF2UH/native-gateway-oracle/receipt.json`.
This evidence was shared with the Fedimint implementation track, without a new
proof claim or a request to weaken the target.

## Native redb composed supporting oracle

An independent parent Rust harness retains the exact selected native methods,
record declarations, named constant DAG and all three contiguous native blocks.
Explicitly authored wrappers compose the two transaction tails through physical
assembly and the original selector/final return block, including the actual
StorageError-to-DatabaseError conversion. Relevant native derives remain;
unrelated implementations are omitted and an extra PageNumber Debug derive is
test scaffolding. Supplied roots/corruption/layout contexts are arbitrary inputs,
not checksum oracles or assumed omitted-callee facts.

Normal and optimized Python orchestration each compile/run debug and release
native fixtures. Every build checks 179,200 finite combinations against an
independently authored u128 byte oracle and selection policy table: 71,040 keep,
51,200 two-phase corruption errors, 25,600 both-corrupt errors and 31,360 swaps.
This includes 76,800 safe invalid-index early branches and 96,000 cases with a
clear-corruption tail shorter than 128 bytes. Complete field/root/physical-slot
and corrupt-byte preservation, exact error strings/wrapping and the clean flag
are checked on successful outcomes. Separate tests reproduce 27 expected native
panics for short buffers or genuinely unsafe indexing; length-eight/longer and
clear tails of every length 112–127 also succeed.

All 2,174 accepted baseline files, seven native source files and the compiler
binary remain unchanged. The native root/pin and clean status match before/after.
The first orchestration attempt used an incorrect expected matrix cardinality
of 358,400; its native tests passed but its receipt failed. The corrected count
is 179,200. That failed receipt is retained, not counted as a successful run.

Reproducer and final successful receipts:
`/private/tmp/verus-two-project-goal.AZF2UH/native_redb_oracle.py`,
`/private/tmp/verus-two-project-goal.AZF2UH/native-redb-oracle-final/receipt.json`,
`/private/tmp/verus-two-project-goal.AZF2UH/native-redb-oracle-final-O/receipt.json`.
These are finite native source-correspondence controls with ZERO universal
Verus proof credit. They do not verify whole decoders/finalize, checksums,
storage, recovery or durability, and cannot discharge the frozen formal target.

## Parent Rust integration-test compilation

Both new parent integration-test crates compile against the accepted baseline
Rust dependencies. Cargo runs offline/locked with a private copied Cargo home,
private target/home/tmp directories and one build job; no shared Cargo or rustup
cache is written. All accepted baseline sources, both new test sources and the
donor cache remain unchanged. The supported `rustup run <absolute-toolchain>`
launcher discovers exactly three fee-composition tests and four byte-decoder
tests, without manually supplied dynamic-library overrides.

A direct executable launch first failed because the test binary has no
rustc_driver LC_RPATH. That loader failure is retained separately; it is not a
Verus proof failure or evidence that a production loader change is needed.
Candidate verifier execution of the seven test methods is still pending.
Compilation and discovery alone earn ZERO candidate formal proof credit.

Reproducer, offline compiler logs, executable hashes and discovery commands:
`/private/tmp/verus-two-project-goal.AZF2UH/compile_parent_tests.py`,
`/private/tmp/verus-two-project-goal.AZF2UH/test-compilation/receipt.json`,
`/private/tmp/verus-two-project-goal.AZF2UH/test-compilation/discovery.json`.

## Fresh manual-effort baseline controls

The parent independently replays both original native checked-add bodies with
the two feasibility contracts and unrestricted u64 caller theorems. Default
erasure, `--no-cheating --infer-goals --compile` succeeds at **6 verified / 0
errors** with zero inferred helper targets. Adding a false overflowing-Some
claim rejects at the assertion. Removing the two helper contracts without
requesting Infer makes the caller postconditions fail. Requesting Infer on the
same native bodies instead declines at the frozen nested Option/Result `?`
lowering boundary and removes the seeded inferred export. These four expected
outcomes pass under both normal and optimized Python orchestration.

This is fresh evidence of the pre-existing manual annotation requirement, not
new inference support or completion of fee validation. Existing matched vstd is
reused, not reverified. Exact native body hashes, source files, runtime files and
all accepted baseline files remain unchanged. No application defect follows
from the two unsupported/missing-contract outcomes.

Two initial orchestration attempts are retained as failed receipts, not passing
evidence. First, the checker incorrectly expected no metadata trailer for a
zero-Infer-target goal-assistance export; actual metadata legitimately contains
zero helper proofs/models/dependencies. Second, it incorrectly required proof
counts on a frontend decline, whose JSON correctly omits them. Final receipts
preserve absent counts as null rather than inventing zero verified obligations.

Reproducer and final receipts:
`/private/tmp/verus-two-project-goal.AZF2UH/replay_manual_effort_baseline.py`,
`/private/tmp/verus-two-project-goal.AZF2UH/manual-effort-baseline-final/receipt.json`,
`/private/tmp/verus-two-project-goal.AZF2UH/manual-effort-baseline-final-O/receipt.json`.

## Mixed-feature integration regression

The parent adds three generic cross-feature regressions in
`infer_mixed_mechanics_composition.rs`: byte decoding/const helpers followed by
record-valued checked addition and borrowed-limit composition; semantic rejection
of a wrapping-overflow claim; and an actual imported consumer followed by stale
failed-client export removal. These test reusable feature interaction rather
than either application's identifiers. They add no replacement target or trust.

The new crate compiles with the same private offline/locked Cargo setup, passes
the existing formatter check, and the supported launcher discovers all three
methods. Source hashes match before/after. Raw compile/discovery logs are in
`/private/tmp/verus-two-project-goal.AZF2UH/mixed-test-compilation`.
The generic probe runner now includes its positive source and checks complete
verification, all three mandatory inference checks, publication and nonempty
compiled artifacts. Candidate-verifier execution of all ten parent test methods
remains pending; no new inference or native proof credit is claimed here.
