# Original redb address-range inference

November 1, 2026. This is a new report, not an amendment of historical evidence.
The implementation restores the integrated dirty **67/61/schema-38 baseline**
at HEAD/origin `55bd67ad980e90e92c77efe4453a927f097c3386`. All 70 manifest source
hashes and nine frozen tool hashes match. An independent baseline snapshot,
Cargo target, runtime bundle, validation bundle, logs and replay live under
`/private/tmp/verus-redb-address-range-64ed8cc5`. The final patch is relative to
that combined baseline, not clean HEAD/master. No local branch/ref was updated.

No applicable Verus-worktree `AGENTS.md` exists. The clean read-only redb clone's
instructions were read. No application source, parent artifact, commit, PR,
branch or subagent was created or modified. Parent Option follow-on fixes are
deliberately not incorporated. This candidate uses reserved cache **68** and
policy **62**, leaving parent integration's **69/63** reservation alone.

## Source fidelity and implementation

Application pin: `c82ab865bb362bc4af067d332adc44978ff64813`, original
`src/tree_store/page_store/base.rs`, SHA-256
`ce2ae1188b5bad2babe2f7b7df19d82adc2b8baff6b41b957e978437686938c4`.
`PageNumber` fields and both executable method bodies are byte-preserved.
The original `debug_assert!` and `start..end` remain; there is no tuple rewrite.
The import provider changes declaration visibility from `pub(crate)` to `pub`
only. Each extracted item is checked against a pinned hash.

The extension recognizes the existing canonical `core::panicking::panic`
requires-false, never-returning vstd specification with an exact literal-string
operand. It reuses the closed literal panic recognizer already used for
`panic_fmt`/literal formatting, across scalar, owned and read-only record
profiles. Scalar exit tracking includes never-to-any coercions. A panic has no
successful domain and no normal output; the original verifier call obligation
and maximality checks remain. Dynamic/formatted effectful operands are rejected,
not discarded. Existing integer, byte and native Option models are unchanged.

The sole proxy-layout exception is canonical, public, transparent
`core::ops::range::Range<u64>` with the existing `ExRange` specification, exact
generic/Sized signature and public executable `start`/`end` fields. Destructor,
user-invariant, opaque, wrong-identity and other proxy types remain rejected.
There is no new trusted panic specification or application axiom.

## Exact safe domains and useful callers

Let `maximum = u64::MAX`, with all following arithmetic mathematical:

- `page_order < 64`, even when `page_size == 0`;
- `page_bytes = 2^page_order * page_size <= maximum`;
- `index_bytes = page_index * page_bytes <= maximum`;
- `regional_start = region_pages_start + index_bytes <= maximum`;
- `region_base = region * region_size <= maximum`;
- `offset_base = data_section_offset + region_base <= maximum`;
- `start = offset_base + regional_start <= maximum`;
- `end = start + page_bytes <= maximum`.

All values are nonnegative by their machine types and the checked preceding
operations. Debug-on additionally requires `regional_start < region_size`;
debug-off does not. Both method summaries pass soundness, safe-domain maximality
and output exactness. No blanket `page_order <= 20` restriction is inferred.

Ordinary generic callers prove `end - start == page_size_bytes(page_size)` and
the exact starting address. A containment caller adds sufficient conditions
`regional_start + page_bytes <= region_size` and
`data_section_offset + region_base + region_size <= maximum`. The debug guard
alone does **not** imply end containment: the original body verifies the exact
counterexample `4090..8186` for page size 4096, regional start 4090 and region
size 4096. A separate incorrect universal containment postcondition fails its
ordinary semantic obligation with fresh exact helper checks and a seeded export
removed. The counterexample is not an altered diagnostic body.

Generic caller properties need no proof blocks or optional assistance. Constant
boundary callers use the existing `--infer-goals` checked assistance, also with
no handwritten solver ghost blocks. Enabling that optional tactic on the large
generic arithmetic callers produced resource-exhausting universal bit-vector
proposals during development; these diagnostic runs are not accepted evidence.
No broad arithmetic engine change or proof-suppression workaround is introduced.

## Configuration and negative controls

Both `-C debug-assertions=yes` and `no` have actual cfg witnesses and independently
proved original method summaries. The former rejects a guard-violating zero-size
range and an empty region; the latter accepts them. The generic release-domain
caller proves without the guard, and fails its real call precondition when
compiled with debug assertions on. Initial frontend rejection under both flags
was not evidence of identical configuration semantics.

Safety controls cover eager shifts 64 and 255 with size zero; page-size,
index-size and region-base multiplication overflow; regional-start, offset-base,
start and end addition overflow; the active guard; empty regions; and the
debug-on release-domain mismatch. Valid order-63 sizes zero and one, exact end
boundaries, zero-length ranges, and the counterexample are proved. Native
debug/release executables exercise the original bodies over 16,719 valid inputs
each, plus explicit guard, shift and counterexample checks. They are finite
supporting oracles, not formal proofs. Native release arithmetic wraps outside
the verifier's checked-arithmetic domain; no total release no-panic theorem is
claimed, and invalid shifts still fail Verus's arithmetic safety obligations.

New-slice controls: **14 positive**, **2 semantic**, **12 safety**, **1 auxiliary**
and **3 poisoned-metadata** cases; **168 required inference obligations** pass.
All 18 negative exports are seeded and removed. Imported Range forwarding is
checked cold/warm. Policy, output-exactness and trusted-model metadata poisoning
are rejected at import. Cache poison cannot provide a candidate in these
profiles: cold/warm/poisoned runs all regenerate and reprove their record/owned
summaries; this is explicitly not a claim of candidate-cache hits.

The front door independently distinguishes ordinary postcondition failures,
ordinary precondition failures and failed checked auxiliary theorems. Auxiliary
rejection cannot substitute for the required semantic or safety evidence.
Historical reports and the previous three scenario controllers are untouched.

## Validation and replay

- Offline, locked, release compiler build from `source/`, toolchain 1.99.1,
  two Cargo jobs, isolated target/build directory, 20-MiB Rust stack.
- Final-compiler vstd validation: **2221 verified, 0 errors**.
- **132** VIR unit tests and **106** integration tests across **12** binaries pass.
- **120** infer Python-discovery tests, **36** targeted transfer tests and
  **17** optimized-mode focused tests pass; these groups overlap.
- All **four** front-door slices report `proved` against the isolated bundle.
- New Rust files pass configured rustfmt checks; `git diff --check` passes.

`manifest.json` records exact source/tool/log hashes, source-fidelity receipts,
domains, trust delta, cold/warm checks and the final raw combined receipt.
The scratch `handoff.json` identifies the baseline-relative `delta.apply_patch`,
its SHA-256, and its replay verification. Replay reconstructs HEAD plus the
frozen combined restoration before applying this delta and compares resulting
file hashes. Parent merges must preserve their separate Option fixes, especially
overlaps in `registry.rs` and `records.rs`, and apply their own final 69/63 bump.

This is an extracted helper/caller theorem, not whole-database verification or
a formal proof of the Rust standard library.
