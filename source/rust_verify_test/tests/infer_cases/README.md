# Infer integration tests

`infer.rs` loads these scenario modules into the same integration test binary.
`interfaces.rs` covers cross-crate publication and metadata, and `reporting.rs`
covers structured outcomes, cache reuse, and failed publication. Domain-specific
fixtures and negative controls stay with their scenarios.

`common/infer.rs` provides the reusable compiler/export harness, stable source
names, JSON report parsing, and exact-proof assertions. Exact assertions check
each independently reported obligation; they do not call production acceptance
logic. Tests of diagnostics can still assert diagnostic text directly.

Run a scenario with `cargo test --release -p rust_verify_test --test infer --
interface_cases` from `source`, after rebuilding the verifier and vstd. The
separate `infer_owned`, `infer_traversals`, and wrapper test binaries reuse the
same support module.
