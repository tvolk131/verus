# redb: source-connected page-size inference

## Result and scope

The unchanged `PageNumber::page_size_bytes` at redb commit
`c82ab865bb362bc4af067d332adc44978ff64813` receives a reusable exact inferred
contract. Ordinary callers prove, for **every** `u32` page size and every order
at most 20:

```text
result = (1u64 << page_order) * page_size       (mathematical multiplication)
result <= 1_048_576 * page_size
result <= 4_503_599_626_321_920
```

Only `page_order <= 20` appears in the canonical caller's requirements. Region,
page index, and page size have no extra restrictions. A second caller proves
the equation on the full safe domain: order below 64 and product at most
`u64::MAX`. Order 63 with page sizes zero and one is also accepted.

The inferred method itself has no assumed canonical-order invariant. Its
requirements retain the eager shift-width check and checked-multiplication
range check. Independent callers for order 64/size zero, order 255/size zero,
the quantified domain order at least 64/size zero, and order 63/size two fail
their call obligations. A false equation fails semantically. All five controls
receive fresh successful soundness, maximality, and exactness checks before
their ordinary caller fails. Each starts with a previously verified interface
at its **own** export path, which must be removed. All five positive exports
remain present afterward.

## Reconstruction

From this repository, supply a redb checkout at the pinned application commit
and a verifier built from the integrated source:

```sh
python3 source/tools/check_redb_page_size.py \
  --application /path/to/redb \
  --verus /path/to/verus \
  --solver /path/to/z3 \
  --output /path/to/receipts \
  --timeout 180
```

The last stdout line is a fresh receipt directory. Its `receipt.json` includes
front-door transfer metadata, commands, classifications, all inferred-function
reports, source hashes, and tool hashes. Raw logs use `label.stdout` and
`label.stderr`. Every application proof uses the selected verifier with
`--no-cheating --output-json --export`; no application proof bypasses checking.
The receipt contains the original source, unchanged extracted items, generated
fixtures, positive exports, and an input snapshot. Reconstruction uses no
historical receipt or other worktree.

The runner verifies the application HEAD and entire `base.rs` hash before
extraction. Its hashes, with a single appended newline for extracted items, are:

| Source | SHA-256 |
| --- | --- |
| `src/tree_store/page_store/base.rs` | `ce2ae1188b5bad2babe2f7b7df19d82adc2b8baff6b41b957e978437686938c4` |
| `PageNumber` declaration, line 34 | `2f35314985c2cba913372168d6fc73f5633b0de1bfc720f73fc5418460b9d867` |
| `page_size_bytes`, line 158 | `2a0e31afa33b709b29c216b83b73ad5f69eb5501b8a3330a819996581573b5b8` |

Direct fixtures preserve the declaration, method signature, and executable
body byte-for-byte; they add only the Verus wrapper, `#[verifier::infer]`, and
separate callers/proofs. Imported-provider fixtures additionally change
`pub(crate)` to `pub` for cross-crate visibility, explicitly recorded as an
adaptation rather than an unchanged public API claim.

## Reusable extension and trust

The new bounded profile handles straight-line bitwise expressions on borrowed
transparent records returning finite integers. It reuses `bitwise::source`,
`bitwise::coerce`, and `bitwise::shift_safe`; arithmetic requirements still
come from the existing record interpreter. Source control flow, mutable
receivers, and effects within operands are not admitted by this profile.
Owned record-to-array codecs retain their literal-shift restriction.

The canonical caller uses an explicit **checked ghost lemma**: one bit-vector
shift bound followed by nonlinear-arithmetic multiplication bounds. A further
checked bit-vector assertion evaluates the order-63 witness. These are ordinary
proofs, not assumptions, axioms, trusted application specifications, or changes
to the extracted method. Trust remains the ordinary Verus/Rust/SMT pipeline
and existing primitive conversion contracts; primitive implementation bodies
are not proved here.

Cold, warm, and imported-summary invocations each rerun all required checks.
Readonly-record candidates currently report `cache_policy=regenerated_state`
and `cache=miss`, including warm runs; this is not evidence of cache hits.
Semantics advance to cache 63 / policy 57, leaving schema 38 unchanged.

Native support checks cover 1,548,477 canonical inputs, all 192 invalid `u8`
shift amounts with zero page size, and the multiplication-overflow witness.
An independent `u128` oracle and a faithful shifted-constant mutation detect
wrong execution. These finite tests do **not** replace the formal all-`u32`
domain proof.

No address-range, debug-assert, whole-database, arbitrary-control-flow, or
mutable-record milestone is claimed. All historical evidence and tests remain
unchanged. The handoff delta is relative to the hash-verified restored redb
snapshot, not repository HEAD; no commits, PRs, or branch changes are made.

## Validation commands

```sh
cd source
RUSTUP_TOOLCHAIN=1.99.1 CARGO_BUILD_JOBS=2 cargo test --release --offline --locked -p vir --lib -- --test-threads=2
RUSTUP_TOOLCHAIN=1.99.1 CARGO_BUILD_JOBS=2 cargo test --release --offline --locked -p rust_verify_test --test infer_record_shifts --test infer_codec_values --test infer_integer_conversions --test infer_record_returns --test infer_transfer_composition --test infer_byte_apis --test infer_records -- --test-threads=2
cd ..
python3 -m unittest discover -s source/tools -p 'test_redb*.py'
python3 -O -m unittest discover -s source/tools -p 'test_redb*.py'
```

Builds use the locked offline Rust 1.99.1 toolchain and two jobs. The target and
solver are independent APFS copies, never shared mutable target symlinks.
Concrete final validation counts, artifact hashes, and receipt provenance are
recorded in `manifest.json`.
