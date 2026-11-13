# Pinned Infer transfers

These programs verify selected source-connected REDB/Fedimint helpers and caller
properties. Each scenario owns its source pin, full input domain, adaptations,
contracts, body inventory and rejection controls. A successful slice is not a
whole-project, crash-durability, signature-validation or cryptographic theorem.

## Automatic transfer matrix

`.github/workflows/infer-transfers.yml` runs six scenarios with a freshly built
matching compiler/vstd/solver: Fedimint request helpers, REDB codecs, page size,
address ranges, page relations, and the full seven-field Fedimint request slice.
Application pins and the exact runner names are in `check_infer_transfers.py`.
Preparation fetches those exact revisions into fresh, disjoint destinations;
registry preparation separately authenticates the full-request dependencies.

```sh
python3 -B source/tools/prepare_infer_transfers.py \
  --destination /tmp/new-infer-applications --output /tmp/new-infer-preparation
python3 -B source/tools/prepare_infer_dependencies.py \
  --application /tmp/new-infer-applications/fedimint \
  --destination /tmp/new-infer-dependencies --output /tmp/new-infer-dependency-setup
python3 -B source/tools/check_infer_transfers.py \
  --fedimint /tmp/new-infer-applications/fedimint \
  --redb /tmp/new-infer-applications/redb \
  --verus source/target-verus/release/verus --solver source/z3 \
  --dependency-home /tmp/new-infer-dependencies/cargo-home \
  --output /tmp/new-infer-transfer-results
```

Use `--help` for timeout/scenario selection. Every output must be new. Full
reports and logs belong in temporary output or CI artifacts. Setup, frontend,
solver/resource and intended semantic/safety failures remain distinct outcomes.
The scenario oracles check named bodies, mandatory helper proofs, import/export
behavior and relevant failure kinds rather than trusting a success count alone.

## Maintained inputs and local tests

`fixtures/transfers/` holds source slices, authored definitions, source ledgers
and expectation data used by the transfer oracles. Its README identifies the
historical source revision. Rust regression inputs shared by Python and the
compiler harness live under `source/rust_verify_test/fixtures/infer/`.

Run `run_infer_tests.py --suite tooling` for the normal/optimized Python tests.
Use its compiler and application suites against a matching bundle for real
proofs; see [the test reference](README.md). The external application/registry
setup above is separate from those local fixture tests.

Other `check_redb_*` and `check_fedimint_*` programs retain more specialized
scopes, including stateful bitmap updates, payment-fee arithmetic, prefix
boundaries, signer insertion and header selection. Some require a specifically
sealed historical baseline or model context. Read their own CLI and source
premises before using them; existence of a script does not imply a portable,
completed native proof. The `stateful_bitmap_reference88` and workflow modules
also contain historical execution adapters with machine-specific defaults.
Their reusable source audits and tests do not establish a fresh native run.

## Interpretation and archive

Source fidelity checks, native finite comparisons and checked mathematical models
provide different evidence. Keep private representation declarations, abstract
callbacks, semantic views, model assumptions and executable adapters explicit.
Do not turn a model-only proof into a native source claim, or shrink the domain
when a helper is unsupported. Failed ordinary exports are not certified providers.

Full research/review transcripts and run products are preserved in Git at
`archive/infer-research-2026-11-12` and
`archive/verus-infer-pre-integration-2026-11-13`. Historical replay commands belong
to their recorded revisions and may need private inputs. Current supported scope
is summarized in the [capability table](../docs/guide/src/infer_capabilities.md).
HMAC/native and REDB crash/publication/recovery guarantees remain deferred and
unachieved; no archive or test-harness migration changes that status.
