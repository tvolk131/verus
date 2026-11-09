# Pinned library source availability — November 7, 2026

The exact Rust 1.99.1 source package is now available for independent review.
This resolves an input-availability gap, not an implementation theorem or a
primitive-contract approval. Historical missing-source findings remain accurate
for their original snapshots; this is a new supplemental source intake.

## Observed binding

The installed release manifest identifies Rust 1.99.1, commit prefix
`58cf8f9c3`, and the dated rust-src archive. Its archive SHA256 is
`e485a88c45db546b73571df7bebfcb4454ba4fb552c492e25fc5b00eeb5bf22d`.
The 5,863,216-byte archive downloaded from the official distribution endpoint
matches that digest. No detached release signature was newly verified.

Before extraction, the archive was checked for bounded size, absolute/traversal
paths, and nonregular/non-directory entries. After extraction all 3,701 files
(73,147,458 bytes) match their archived contents, with no extra files under
the extracted package root. The full inventory remains in the private intake.

A version-only compiler query reports full commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`, host aarch64-apple-darwin,
release 1.99.1. Eleven selected installed runtime inputs, including the compiler,
driver and core/alloc/std/panic-abort libraries, match their hashes and sizes in
the prior growth049 toolchain binding. This is not reauthentication of the
entire toolchain or a source-to-binary reproducible-build proof.

Installed 1.99.0 source was rejected as a substitute. No rustup installation,
toolchain modification, compiler build, proof run, or native-target execution
was performed. The single compiler invocation queried version metadata only.
An initial Python metadata probe lacked `tomllib`; no dependency was installed.
The binding script subsequently validated explicit known manifest sections.

## Evidence and next check

`LIBRARY-SOURCE-BINDING.json` is an exact retained copy of the intake summary.
It binds the archive, source inventory, release manifest, selected runtime
inputs, and version-query output, retaining their qualifications.

Private intake: `/private/tmp/verus-pinned-rust-source-20261107-9FT58F`.
Library sources:
`/private/tmp/verus-pinned-rust-source-20261107-9FT58F/rust-src-1.99.1/rust-src/lib/rustlib/src/rust/library`.

A separate source-conformance review will compare the proposed Arc/Option/slice
and alias/glue laws with these exact implementation sources. The existing
design review remains distinct. Neither review adopts assumptions or replaces
canonical typed-instance binding, caller-premise discharge, checked E1–E6,
actual native resource verification, or database acceptance.

No primitive trust or scope policy changes. All 18 final redb cells remain OPEN.
