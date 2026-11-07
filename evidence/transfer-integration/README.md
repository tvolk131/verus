# Combined Infer transfer integration

November 1, 2026. Both extensions are integrated into this task worktree, based on
fetched `origin/verus-infer` at `55bd67ad980e90e92c77efe4453a927f097c3386`.
No commits or PRs were created. Neither primary application checkout was edited.

## Scope and trust

Readonly record queries now accumulate guarded non-unit early returns and combine
them with continuing tail paths. Executed expressions retain their safety
requirements; unreachable later operations do not restrict the inferred domain.
The existing readonly/value-type and mutable unit-return boundaries remain.

Native `u64::to_le_bytes` and `u64::from_le_bytes` now have concrete fixed-array
contracts and typed Infer models using existing byte definitions and inverse
proofs. Exactly two primitive specifications, including `no_unwind`, are new
trusted boundaries. Rust std implementations are **not** formally verified and no
roundtrip axiom is added. See the [trust inventory](../transfer-redb-byte-apis/TRUST.md).

The integrated build uses metadata schema **38**, inference policy **55** and
candidate cache **61**. Two independent reviews of the isolated production
candidates reported no substantive findings. Parent integration also adds a
positive/negative [composition regression](../../source/rust_verify_test/tests/infer_transfer_composition.rs)
that combines a readonly early-return query with an inferred native byte roundtrip.

## Parent validation

All final targeted checks pass:

- **120 VIR unit tests**, including independent native arithmetic/byte oracles.
- **392 integration tests**: the main Infer suite (296), byte APIs (6), record
  returns (7), cross-feature composition (1), and adjacent suites (82).
- **2,219 vstd verifications, zero errors** in the full rebuilt library.
  The final Cargo invocation then reused that verified library output.
- **108 shared Python tests**, plus four replay tests normally and four under
  Python `-O`.
- Rust formatting, vstd formatting and `git diff --check`.

This is not a complete-workspace green claim. The byte candidate's 18 doctest
failures in unchanged examples remain documented; parent Rust unit checks use
`--lib`. Initial solver-location and test-fixture naming/field-label integration
errors were corrected. Their superseded logs are retained, not counted as passes.

## Integrated application replays

### Fedimint

The byte-identical, explicitly desugared deadline candidate yields **8 verified,
zero errors**, with both helpers exactly inferred over the original unrestricted
machine-value input domain. Cold/warm invocations establish fresh required proofs;
the record profile regenerates candidates rather than reporting cache hits.

Removing the intercepted-forward guard still produces **7 verified and one
authored postcondition failure** while both inferred descriptions remain exact.
The failure removes a seeded stale interface. An unrelated false goal and a
malformed goal also block publication, with their failure categories distinguished.
The native original/adapted differential control passes 2,112 inputs.

This is extracted/refactored decision-boundary verification. Original combinator
and tuple-constant frontend blockers remain. Native equivalence of those
desugarings is tested, not universally proved. Untouched gateway, database
atomicity, concurrency, recovery and whole-flow fund safety remain unproved.

### redb

All frozen extracted bodies, intended domains and native controls match their
baseline hashes. Both primitive byte frontend errors are gone. The unchanged codec
now stops at `Result::unwrap` in `u32::try_from(...).unwrap()`. No redb inferred
contract or interface is published; its false formal application property is
still unassessed. Independent record-shift and panic-boundary blockers remain.

The focused byte replay proves **60 required checks**, with cold/warm/imported
composition and semantic/metadata controls. Native controls pass 6,144 byte
boundary cases, 2,097,467 page identifiers and 12,222 composed address checks;
the decoder-shift mutation fails its intended native assertion. Native testing is
not a formal database proof. Controller `success: true` means expected outcomes
were checked, not that the blocked redb application verified.

## Evidence and reproduction

[manifest.json](manifest.json) records final combined source/artifact identities,
test counts, receipt hashes, scope and trust. The two imported extension manifests
retain their original isolated-candidate identities; they are historical records,
not hashes of the parent integration's descriptive-name cleanup or composition test.

The raw parent bundle is `/private/tmp/verus-infer-integration.EbudBf`.
Its redb replay includes a 1,552-file source/tool/dependency ledger and the captured
production/test patch. The base SHA alone does not identify the dirty integrated
engine; use that ledger, patch and built-artifact hashes.

Integrated receipts:

- `/private/tmp/verus-infer-integration.EbudBf/fedimint-replays/run-7n_o966_/receipt.json`
- `/private/tmp/verus-infer-integration.EbudBf/redb-replays/run-_slw3nj7/receipt.json`

Build and tests run from `source` with Rust 1.99.1, Z3 4.16.0, offline/locked
Cargo and two build jobs:

```sh
cargo build --offline --locked --release -p rust_verify -p verus -p cargo-verus -p vstd_build
./target-verus/release/cargo-verus verus build --offline --release --manifest-path vstd/Cargo.toml -- --num-threads 2
cargo test --offline --locked --release -p vir --lib
cargo test --offline --locked --release -p rust_verify_test --test infer --test infer_byte_apis --test infer_record_returns --test infer_transfer_composition --test infer_records --test infer_wrappers --test infer_selection --test infer_error_values --test infer_wrapper_traits --test infer_traversals
```

The replay controllers are `source/tools/check_fedimint_early_returns.py` and
`evidence/transfer-redb-byte-apis/run.py`. Their exact argv, read-only frozen
baseline locations and pinned scratch clones are recorded in the receipts.
Temporary clones/raw logs are retained but may eventually be cleaned externally;
reconstruction requires the recorded source commits and frozen baseline bundles.
