# Full seven-field Fedimint deadline transfer

This experiment retains the actual `InterceptPaymentRequest`, including
`payment_hash: bitcoin::hashes::sha256::Hash`, both amount fields and the optional
short channel. Both original executable helper items are byte-preserved.
The application is the clean read-only pin
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`. The starting checkout is
`a4046f687952f6e929e34cabf58a0cf5cb524d88`; its relevant verifier sources equal
production engine `3b05c191`. No parent utility consolidation or concurrent
word-assistance change is included. No other worktree, application source,
shared build/bundle, prior evidence, push, PR or delegate was used or changed.

## Frozen target and observed baseline

Commit `1246650588549b9631d27bc9db857b2a93551fbb` freezes `baseline.json`,
`original.rs` and `goals.rs` before any proof attempt or engine change. The
inventory covers the complete domain, six authored safety/coverage/boundary/
monotonicity clauses, intended adaptations, trusted boundary and exclusions.
The original struct, complete implementation, methods and literal tuple
constant have independent source hashes. The runner re-extracts them from the
pinned application and checks exact equality, not a handwritten approximation.

Three stages have distinct outcomes on the unchanged engine:

1. The complete native extraction compiles with the actual Bitcoin type and
   Debug/Serialize/Deserialize/Clone derives. All **21** selected registry
   package versions and checksums match Fedimint's lockfile, including Bitcoin
   **0.32.8** and bitcoin_hashes **0.14.0**.
2. Without an external declaration, Verus rejects the seven-field request with
   `` `bitcoin_hashes::sha256::Hash` is not supported `` and requests an
   `external_type_specification`.
3. An existing explicit opaque declaration gets the full request and both
   original bodies through ordinary verification (**3 verified, 0 errors**),
   but Infer rejects record eligibility: "record state requires visible
   transparent record fields or a certified direct sequence view, bounded
   depth and at most sixteen leaves". Ordinary verification here has no
   authored policy or inferred contracts; it is not the application proof.

The successful unchanged-engine probe receipt is retained at
`/tmp/fedimint-full-request-research-20261109/baseline/full-request-2b5t7_ag`.
An earlier controller attempt incorrectly required inference-aware publication
for the ordinary translation probe. That probe itself verified successfully;
the corrected runner distinguishes its ordinary `.vir` export and
`no_inference` status. The failed controller receipt is retained separately.
An initial build from the checkout root missed `source/.cargo/config.toml` and
was stopped; all proof inputs come from the corrected private build.

## Minimal inference boundary

The existing frontend mechanism is used unchanged:

```rust
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExternalSha256(bitcoin::hashes::sha256::Hash);
```

This proxy supplies only an opaque declaration of the **real external type**.
It is not a field replacement. No hash representation, digest equation, crypto
law, assumed helper, external function contract or rewritten dependency is
introduced.

The reusable change is an inference type-eligibility predicate and one use in
the existing record walker. A monomorphic, always-sized opaque foreign type
with its checked proxy identity can be retained as a whole leaf **nested** in a
visible record for an **exact shared-reference query**. It requires exec mode,
no type bounds, no exposed fields, no destructor, no invariant and no
extensional-equality declaration. Existing first-party opacity is unchanged.
The new path grants no root opaque receiver, mutable/by-value or sound-only
profile, generic foreign leaf, field representation, constructor or method
model. All inference candidates still prove the original bodies, independent
maximality and exactness before publication.

## Application proof and semantic controls

The full replay succeeds at
`/tmp/fedimint-full-request-research-20261109/final/full-request-3sc9m4r1`.
Cold and warm selected-scope proofs each have **10 verified, 0 errors** and
check all five executable bodies: the two original inferred helpers and the
three authored policy/boundary/monotonicity functions. Each helper has
`requires true` and fresh required soundness, maximality and exactness proofs.
Cold/warm summaries agree, and interfaces are freshly published. These record
candidates honestly report regenerated-state cache **misses**, not cache hits.

Imported cold/warm consumers each verify all three authored functions
(**3 verified, 0 errors**) using the certified full-request helper interface.
There are **24 fresh mandatory helper checks** across the four local inferred
runs (two positive runs and two semantic controls).

Deleting only the original intercepted-forward guard gives **9 verified,
1 authored-policy error**. The independent false goal has the same 9/1
outcome. Both helpers remain exactly inferred with `requires true`; neither
failure is a frontend, precondition, timeout or resource failure. Seeded valid
interfaces are removed and nothing is published. A declaration for the wrong
SHA type separately fails actual SHA-256 translation and removes its seeded
export; that is an identity/frontend control, not semantic bug detection.
The bare unsupported-type control still fails with the extended engine, so an
external declaration is still mandatory.

The native oracle executes Verus-erased compiled code against independently
compiled original bodies over **8,448** boundary/payload cases with the real
hash type and all seven fields. It also checks the guard-deletion witness
(fixed budget **0**, mutant **10**) and the two-block `u32::MAX` coverage witness.
Finite execution tests correspondence; formal verification establishes the
selected full-domain policy.

## Trust, adaptations and remaining author work

See `TRUST.md`. The proof omits surrounding serde/Debug/Clone derives, surrounding
modules and unrelated dependencies. Native compilation checks those derives
separately. The field set and both executable helper bodies are unchanged;
inference attributes sit outside the original items and the extraction is
wrapped in `verus!`. The supplied external declaration and Rust-to-Verus type
correspondence are explicit trusted boundaries. Existing vstd/library
specifications, compiler, backend, SMT encoding and solver remain trusted.

This proves the selected deadline-helper policy for the complete request type
under that opaque boundary. It proves no funding, amount authenticity,
authentication, settlement, cryptography, whole crate, dependency-feature
equivalence or callsite wiring. Authors still need separate real-flow policies
and proofs connecting gateway funding/settlement decisions to these helpers.
The earlier three-field common transfer front door remains unchanged; adoption
of this full-request replay there is future integration work.

The experiment leaves candidate cache **v69**, metadata schema **38** and local
policy **v63** untouched. Parent policy **v64** is reserved for its independently
identified full-scope publication fix. Before distribution, reconcile this
boundary with that fix and the concurrent word-assistance changes, allocate a
combined policy **v65 or later**, and review the combined cache-generation bump.
These private artifacts are tied to recorded binary/source hashes and are not
production distribution inputs. Every application run uses full verification.

## Replay

Use Rust 1.99.1 and `/home/dev/.cargo/bin` first on PATH. Builds run from
`source/`, offline/locked, with two Cargo jobs and private target/build/bundle
paths. The solver is an independent copy of the parent's standalone `source/z3`,
SHA-256 `e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765`.
The parent's rebuilding bundle is never a proof input.

```sh
python3 -B evidence/transfer-fedimint-full-request/build.py \
  --output /tmp/private-full-request-build --solver /path/to/z3
python3 -B evidence/transfer-fedimint-full-request/run.py \
  --application /path/to/clean/pinned/fedimint \
  --verus /tmp/private-full-request-build/target-verus/release/verus \
  --solver /path/to/z3 --output /tmp/private-full-request-replays
python3 -B -m unittest discover -s evidence/transfer-fedimint-full-request -p test_replay.py
python3 -B -O -m unittest discover -s evidence/transfer-fedimint-full-request -p test_replay.py
```

`--probe` is an unchanged-engine diagnostic mode, for the saved baseline tools;
it is expected to reject inference before this eligibility extension. The
normal replay verifies the complete selected scope and all acceptance controls.
Receipts record command argv/environment, source/tool fingerprints, raw
outcomes, scopes, required checks, cache accounting and publication results.
Raw receipts, logs, dependency artifacts and independent baseline/final tools
remain in the private persistent directory; binaries and dumps are not in Git.

## Regression validation

All **131 Rust tests** pass: 102 inference units (including the two new closed
foreign-descriptor/receiver controls), four unrelated IPv4/IPv6 integration
cases, 18 native Option cases and seven record-return cases. All **10 Python
test executions** pass (five normal and five optimized). Both independently
checked vstd builds report **2,221 verified, 0 errors**; incremental build
messages are not counted as additional proofs.

The first integration invocation had three setup failures because Cargo's
separate build-dir put test executables away from the copied verifier binaries.
The positive foreign-type case passed. The corrected builder provisions matching
private copies beside those executables; all four cases then pass. These setup
failures are retained in the raw logs and never classified as semantic negatives.

The Rust commands run from `source/`, with `/home/dev/.cargo/bin` first on PATH,
`CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR=/tmp/fedimint-full-request-research-20261109/target`
and `CARGO_BUILD_BUILD_DIR=/tmp/fedimint-full-request-research-20261109/build`:

```sh
cargo test --offline --locked --release -p vir infer::
cargo test --offline --locked --release -p rust_verify_test \
  --test infer_foreign_opaque --test infer_native_option --test infer_record_returns \
  -- --test-threads=2
```

`manifest.json` records compact outcomes, frozen/source/tool/dependency
fingerprints, log hashes, baseline and final receipts, setup corrections and
version reconciliation. Native replay dependencies must already be available
in Cargo's registry cache for its offline/locked build.
