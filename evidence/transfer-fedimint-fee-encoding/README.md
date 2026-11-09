# Bounded application-led cycle: exact native fee transport

This is a new source-connected **scalar fee-encoding** proof, not a proof of
fee calculation, amount accounting, registration, whole-gateway behavior or
funding safety. Historical deadline results remain baseline, not new progress.
The fee slice adds a standalone replay, tests and this evidence directory, with
no fee-specific engine or native application changes. Its original baseline
was cache74 / policy68 / schema38; reserved candidate76/70 was unused.

The combined parent integration includes shared documentation in
`source/tools/INFER_TRANSFERS.md` and the redb computed-shift extension at
cache75 / policy69 / schema38. The active fee runner observes and checks that
identity from candidate-cache entries and the provider certificate, rather
than asserting an engine identity or an unchanged-engine result. The scalar
proof itself introduces no new engine capability or trust.

## Start from the actual next caller

The original `try_handle_lightning_payment_lnv2` first constructs
`PaymentImage::Hash(htlc_request.payment_hash)` and awaits
`get_registered_incoming_contract_and_client_v2`. That method awaits a database
lookup, checks the registered amount against **incoming_amount_msat**, checks
receive enablement and selects a client. Both subsequent funding arguments use
`incoming_amount_msat`, not the onion forward `amount_msat`. This is inspected
source wiring, not a verified funding theorem. The preserved runner's
`caller_evidence` is executed again and hashes all four complete native caller
items, including the relay and its pure operation-plan helper.

Minimum author boundary for continuing the whole caller: a database snapshot
and lookup-result relation to the payment-image key, registered incoming amount
and contract; receive-enable/client-selection effects; external-height result
and freshness; operation existence across receive, completion and legacy
completion identities, including concurrency and transaction effects. Relating
contract amount to the registered amount further requires a proved registration
and fee-provenance invariant. Hash construction/access is outside the existing
borrowed-only UNUSED opaque model. No hash identity, authenticity, crypto,
persistence, liveness or irreversible-funding contract is assumed here.

## Adjacent arithmetic and the stopping point

Registration calls `payment_info.receive_fee.subtract_from(payload.amount.msats)`;
the actual `PaymentFee::absolute_fee` chain includes native `saturating_div`.
A frozen probe retains the exact Amount/PaymentFee declarations and complete
original method item. Baseline tools reject it at the frontend because that
native division lacks a vstd specification. Adding an unchecked external
summary would introduce new external-behavior trust and requires author
approval; none is added. The compiler's suggestion of `assume_specification`
is a diagnostic, not permission to use it. No numeric-wrapper or Option blocker
is claimed repaired, and no compiler defect is claimed found.

The next concrete pure path is fee transport: recurringdv2 encodes
`routing_info.receive_fee.fee(amount).msats` into a contract expiration, and
the LNv2 receive state machine decodes that expiration for its event fee.
The complete original `fee_encoded_expiration` and `fee_from_expiration`
functions in `fedimint-lnv2-common/src/contracts.rs` need no additional trusted
model. An initial round-trip with unannotated original functions fails its
authored postcondition. Adding only existing Infer annotations gives checked
interfaces. This is application proof wiring with **zero engine changes**, not
a newly implemented mechanical inference capability. At most two permitted
engine blockers are therefore not spent on a speculative extension.

## Authored intent and original safety domain

For every native u64 fee and expiration, without authored entry preconditions:

- decode(encode(fee)) equals fee;
- encode(decode(expiration)) equals expiration;
- encoding reverses order and reflects equality;
- encode(fee) is strictly after `now` iff the mathematical sum fee + now is
  strictly below u64::MAX;
- encode(0) is u64::MAX and encode(u64::MAX) is zero.

The temporal equivalence intentionally does **not** say encoding always produces
a future timestamp. A separate authored unconditional-future control is false
at fee=u64::MAX. The code's nearby prose about near-maximum values needs a
separate practical-fee/clock bound to justify that stronger application claim.
Neither valid encoding nor round-trip recovery proves that a supplied expiration
was created by the recurring daemon, authenticates an event, or distinguishes
fee-encoded values from ordinary timestamps. The imported caller is an authored
mathematical harness using checked interfaces, not the async receive state
machine or the real recurring daemon verified as a whole.

## Fidelity, trust and replay protocol

`check_fedimint_fee_encoding.py` requires the clean exact Fedimint pin
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`, complete source-file SHA256s
for both helper definitions and both real call paths, and exact item hashes.
Both extracted items retain their bytes, visibility, types and executable bodies;
their preceding non-executable doc comments are omitted. Adaptations are
extraction, Infer attributes outside items, vstd/verus wrapping and independently
authored goals. The diagnostic PaymentFee probe alone omits
irrelevant derives by extracting the original declarations; its body is unchanged.
Native Rust compilation uses overflow checks. Eight boundary samples supplement,
but do not replace, universal proofs.

Only existing Rust typing/erasure, frontend correspondence, Verus/backend, Z3
and existing scalar machine-arithmetic semantics are trusted. No new axiom,
external body, assumption, application summary or dependency is introduced.
No Cargo build or registry input is needed for this new scalar replay. Matching
compiler/macros/vstd/solver tools originally used the sealed tools74 package.
The combined parent uses sealed tools75 with matching production Rust sources;
the fee slice itself still requires no engine build.

The replay checks cold/warm providers, mandatory soundness/maximality/exactness
and nonzero reported solver work even on candidate-cache hits, complete-crate verification, exact
positive proof/body counts, inferred true entry domains, imported caller
composition and the identities of the provider's two certified helpers.
It independently classifies an unannotated before-control, a total off-by-one
decoder mutant, false future semantics, actual overflow/panic safety failures,
malformed/frontend failures, a solver-resource failure and seven corrupted
interface controls in a separate `import` category. The malformed control must
report Rust E0277 with no verified goals or verification errors, not merely echo
the malformed expression in a semantic failure. Every failing run is seeded with a successful export and
must remove it. No frontend, import, budget or safety failure counts as a
semantic negative. Complete engine-source and native application enumeration,
generated proof inputs, tools and imported provider artifacts are guarded before
and after subprocesses. The fee guard additionally hashes the rustup toolchain's
compiler, shared libraries (including rustc_driver and LLVM where present), and
standard-library rlibs; these are recorded separately from the nine sealed
Verus/solver artifacts. System libraries, OS execution and library loading remain
trusted; this is not an exhaustive runtime attestation. No selected-function
bypass or weakened source guard.
The fresh unannotated before-control loads an unused checked provider solely to
exercise the inference-aware stale-export barrier; its locally named original
helpers still have no contracts. The initial unimported pilot is separately
preserved without a seeded export. Cold/warm candidate IDs are recorded
separately (cached formula reconstruction can change provenance); exact
contracts agree and the imported certificate fingerprint binds the actual warm
provider artifact, not an assumed stable ID across candidate reconstruction.
Before every positive run the working export is removed; each provider run
also removes its working rlib. Successful outputs are archived under previously
absent phase-specific names. Working paths and compiler arguments stay identical
between provider phases so the warm run exercises the same candidate-cache key;
a cold interface or rlib cannot stand in for missing warm publication.
Candidate IDs in the JSON report are diagnostic: this replay does not decode
the binary VIR to bind those IDs independently to its summary hash. As with
the other local receipts, this is consistency checking with trusted tool
execution, not cryptographic attestation against a substituted verifier.
The unannotated before-control is explicitly categorized `before`, not as a
semantic counterexample: the intended round-trip is true but initially lacks a
usable proof interface. Only the total decoder mutant and false future claim
are semantic-negative controls.

Run after finalizing source/docs, with fresh private outputs:

```sh
GIT_OPTIONAL_LOCKS=0 PYTHONDONTWRITEBYTECODE=1 python3 source/tools/check_fedimint_fee_encoding.py \
  --application /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint \
  --verus /private/tmp/verus-redb-next75.sOFIw9yU/tools75/verus \
  --solver /private/tmp/verus-redb-next75.sOFIw9yU/tools75/z3 \
  --output /private/tmp/fresh-parent-fedimint-fee
```

Measured outcomes, exact commands/stdout/stderr, original pilot diagnostics,
fresh-versus-audited counts, source/runtime guards and replay-checked
baseline-relative reconstruction live in the accompanying private handoff.
Historical results are not counted as fresh tests. This report states the
scope/protocol; the fresh receipt is authoritative about success or failure.
The implementation handoff predates parent integration. The first Opus review
stopped at provider quota and remains incomplete. A fresh Opus review after the
reset completed, approved the scalar theorem, and replayed all 19 cases. It
identified replay-level regression gaps and hardening/documentation items.
Parent follow-up factors those decisions into tested functions, exercises the
actual per-case wiring, guards the toolchain libraries, observes cache identity,
and separates fresh exports and frontend/import classifications. These repairs
are parent-validated, not a new independent review. The preserved
handoff's unimported pilot description also needs qualification: configuration
removed the seeded inferred export, then ordinary legacy pre-verification
publication wrote a new export. It did not retain the original seeded bytes.
That non-inference-aware behavior is outside this replay's inference-aware
publication claim and was not changed here. No commit, PR or ref update.
