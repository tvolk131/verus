# Reproduction interface

Frozen source root ce0f00897bec7f8ea143cd42409563440d9c8784, correction70a9c390.
Exact hashes and full commands: AUTHENTICATION.json, RESULTS.json,
SOURCE_HASHES.json. All raw inputs, stdout/stderr, commands and binaries are
private in /tmp/owned-cell-f1-directive-review-ce0f0089/; RAW_HASHES.json binds1505
files/controllers. These raw repeated assets are intentionally not committed.

Private controllers and their SHA256 are listed in RESULTS.json. For a fresh
replay, copy those small controllers privately and replace the original output
path with a new private path consistently. Use a checkout whose production bytes
match frozen root (evidence commits do not change those bytes); replace ROOT/R
path only if needed. Never reuse mutable parent or original review outputs.
Authenticate all8 immutable baseline tools before copying7 supports. Compiler
must be rebuilt: the old baseline cannot prove this correction.

Commands run, with PYTHONDONTWRITEBYTECODE=1:

1. python3 /tmp/f1-directive-review-build.py
   Uses implementation/check.py env/run and existing LoggedCommands/replay_receipt/
   ArtifactGuard/digest/require. Baseline8 hashes must match receipt
   9b414f0493385c649d856fd181625ec74778173e10623ee6ba3c3fda959b945f and
   F1_FIX_RESULTS.json. Copies7 supports; builds only rust_verify with
   Rust1.99.1 cargo build --offline --locked -p rust_verify in frozen source/.
   Offline bootstrap uses exactly check.py setup's path/archive/index/git/lock
   checks; no parent build-cache copy. Compiler source guard checked before/after
   and all source hashes compared to frozen git blobs. Copy fresh compiler into
   private bundle and require a distinct identity.
2. python3 /tmp/f1-directive-review-harness.py
   Preparation can occur before compiler completion; rerun after build to execute.
   Optional independent small compile: f1-directive-review-compile-harness.py.
   Standalone manifest pins serde_json1.0.151 and tempfile3.27.0; generated lock
   hash is recorded. Only modifications to both original test files:
   delete `#![feature(rustc_private)]` and replace include_str relative path by
   absolute frozen owned_goals.rs. Inverse transform is asserted byte-identical.
   Shim provides only InferCrates new/directory.path/profile/vir: same tempdir,
   explicit private tool-profile path, keeps tempdirs instead of deleting them.
   Profile launchers transparently record argv/status/elapsed/stdout/stderr then
   relay unchanged output/status from authenticated bundle tools. Oracles and
   test bodies remain unchanged. Expected5+6 tests,198 tool calls,166 honest JSON
   admission refusals,32 new directive and16 old-marker precise early negatives.
3. python3 /tmp/f1-directive-review-replay.py
  128 guarded calls. Old immutable8-tool bundle is read-only; new bundle replaces
   only compiler. Batch tool guard surrounds both matrices, with per-case source/
   controller guards. Each direct call has a separate fresh cache and output.
4. python3 /tmp/f1-directive-review-root-probe.py old
   python3 /tmp/f1-directive-review-root-probe.py new
  8 guarded old/new crate-root observations. Their exact mismatch oracles are
   asserted by summarize.py (not inferred merely from process exit).
5. python3 /tmp/f1-directive-review-classification-controls.py
  24 guarded item/module/default-verify/macro/encoded-const controls.
6. python3 /tmp/f1-directive-review-summarize.py
   Asserts counts/zero credit, all10 stable old/new default-off comparisons,
  8 root witness oracles, exact frozen build-source hashes, final8-tool identity
   and conservative time budget. Writes compact receipts and private hash map.

Each direct verifier call uses private verus, input.rs, --crate-type=lib
--edition=2024 --no-cheating --compile --output-json -o <private .rlib>, with
--infer-goals and/or --experimental-owned-unsafe-cell as recorded. Baseline
matrix fixes crate name review_probe. All early directive refusals must include
`frontend directive/special-const local item or ancestor is forbidden`, exclude
`inactive: F1 admission passed;`, award zero proof/helper/export credit and emit
no .rlib/.vir. Mixed old/new markers must retain the earlier two-marker reason.
Unmarked scalar/header/unit cases only reach INACTIVE when enabled; default-off
unit/scalar check and false contract fails actual postcondition. Valid reveal/
broadcast default-off succeed with zero ordinary bodies. Malformed frontend
errors/panics are never positive skip/proof evidence.

No broad compiler suite, full vstd rebuild, D1/D2 review, native activation,
shared mutable output/cache, source authority, push or PR is part of this replay.
