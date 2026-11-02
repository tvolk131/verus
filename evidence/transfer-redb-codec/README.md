# First source-connected redb PageNumber codec proof

Result: **proved over the frozen canonical machine domain**, not whole-database
correctness. Application pin: `c82ab865bb362bc4af067d332adc44978ff64813`.
Engine base: `55bd67ad980e90e92c77efe4453a927f097c3386`, plus the integrated
uncommitted snapshot restored before implementation. All 22 snapshot hashes
matched its manifest before any extension. The older failed-baseline evidence,
parent integration evidence and existing parent record tests remain unchanged.

Authoritative replay:
`/private/tmp/verus-redb-codec-7597ec83/replays/redb-codec-bp47idhd/receipt.json`.
Persistent builds, discovery runs and validation logs:
`/private/tmp/verus-redb-codec-7597ec83`.

## Source and property boundary

The runner reads the pinned application's `src/tree_store/page_store/base.rs`
without modifying the checkout. It checks the whole file and independently
frozen hashes of the PageNumber declaration and both methods. Their executable
bodies are unchanged, including `u32::try_from(...).unwrap()`, mutation of the
encoding accumulator, byte calls, masks and shifts. The original unassisted
`codec.rs` fixture remains byte-for-byte identical to the frozen baseline
(`639d1032cc0fb061cf3e4bf7c8b05b14b13e7d856ce047b09e1f740d6565a3cb`).

The authored property quantifies over every machine value satisfying:

- `region <= 0xfffff`;
- `page_order <= 20`;
- `page_index <= (0xfffffu32 >> page_order)`.

Decode after encode preserves all three fields. A second ordinary caller proves
encoding injectivity on exactly that domain. No input restriction is added.
The canonical encoder clears reserved bits; arbitrary reserved-bit behavior and
the entire database are outside this new proof's property boundary.

The frozen unaided caller now obtains exact implementation contracts but does
not discharge their bitwise composition automatically: both runs have 66
verified and one caller body with precondition/postcondition diagnostics. These
are neither a codec counterexample nor a frontend blocker. The successful
authored caller adds checked ghost bitvector packing facts and the existing
vstd byte-inverse lemma, and names the intermediate encoding. It has the same
requires/ensures and the same two executable method calls. Neither application
method acquires a handwritten summary or proof block.

The public-provider fixture changes only declaration/field/method visibility,
not executable bodies. An imported inferred decoder relay composes the actual
certified codec interface. Its ordinary callers prove the same roundtrip and
injectivity properties through the import.

## Bounded reusable implementation

`integer_conversions.rs` recognizes a contiguous canonical primitive
`TryFrom(...).unwrap()` chain, checking core identity, vstd proxies, concrete
integer/Result/error types and resolved trait dispatch. The untrusted candidate
preserves operand obligations and requires destination representability before
returning the cast. It does not synthesize an arbitrary error payload or assume
unwrap success. Original-body soundness asserts the existing vstd requirement.
Only safe-domain maximality admits that canonical success requirement when
describing successful executions. Output exactness remains independent.

The next bounded blocker was record/array composition. Transparent by-value
record-to-fixed-scalar-array methods now use the existing record interpreter
for straight-line scalar accumulators and literal finite shifts. Arrays remain
bounded; control-flow mutation, variable record shifts, reference aliases,
type invariants and effects retain rejection boundaries. Readonly records and
complete constructors reuse the already checked native-byte model. Imported
record summaries validate the corresponding bounded array/finite-bit vocabulary.

No new vstd specification or axiom is added. The integrated vstd source is
unchanged. Cache **62**, policy **56**, schema **38**; the parent must coordinate
these shared version markers with its other transfer work.

## Final checked results

- vstd rebuild: **2,219 verified, 0 errors**.
- VIR unit suite: **122 passed**, including two new conversion tests. An
  independent Rust conversion oracle checks **196,608** signed/unsigned cases.
- New integration suites: **5 conversion + 5 record/array tests passed**.
- Adjacent suites: **9 infer_binary, 6 infer_byte_apis, 7 infer_record_returns,
  21 infer_records, 1 infer_transfer_composition passed**: **54 integration
  tests total**, including the ten new ones.
- Replay Python tests: **4 passed**, both normally and with `python3 -O`.
- Authored roundtrip: **68 verified, 0 errors**, cold and warm.
- Roundtrip plus injectivity: **71 verified, 0 errors**, cold and warm.
- Exported codec provider: **71 verified, 0 errors**, cold and warm.
- Imported decoder/callers: **8 verified, 0 errors**, cold and warm.

All required soundness/maximality/exactness checks are freshly proved in both
phases. Record/owned summaries report `miss / regenerated_state`, not proof-cache
hits. Conversion-provider warm candidates report hits; imported conversion
relay candidates can miss after provider regeneration. No proof result is cached.

The faithful decoder mutation changes only region extraction from `>> 20` to
`>> 21`. Its two inferred methods still pass all exact checks, while the authored
region postcondition fails: **67 verified, 1 error**. The independent false-goal
control has the same counts. Both delete seeded stale exports and publish no
interface. An unrestricted conversion caller fails its actual panic-related
precondition. Separate malformed-type and policy/proof/summary/model import
controls are frontend/import rejections, never semantic counterexamples; actual
previously verified interfaces are seeded for early frontend/import invalidation.
All eight seeded-stale-interface cases reject publication and remove the export.
Poisoned conversion caches independently fail fresh soundness/maximality checks.

Native execution covers **2,097,467** canonical codec cases against an independent
shift-based byte oracle. It enumerates every legal index/order and several region
boundaries, not the full region cross-product. The same decoder mutation fails
the native region-roundtrip assertion. Empirical native coverage is not the
arbitrary-input formal proof.

## Replay and integration

From the engine checkout, with an already built matching verifier/vstd and the
pinned application available locally:

```text
RUSTUP_TOOLCHAIN=1.99.1 PYTHONDONTWRITEBYTECODE=1 python3 source/tools/check_redb_codec.py \
  --application /path/to/pinned/redb \
  --verus /path/to/verus --solver /path/to/z3 \
  --output /private/tmp/redb-codec-replays --timeout 180
```

`run.py` here is a thin entrypoint with the same flags. The last stdout line is
the fresh directory containing `receipt.json`. The runner has no dependency on
historical worktrees, raw receipts or network access. Its `transfer` summary
separates controller success from application status; a blocked replay never
reports a proved application. Raw reports, commands, source hashes, tool/native
artifact hashes and stability checks are retained in each fresh directory.

Builds/tests run offline/locked from `source/`, toolchain `1.99.1`, two jobs.
`source/target-verus` is an independent APFS copy-on-write copy of the integration
target, not a symlink or shared mutable build tree. `source/z3` is also a copy.
No parent or other child checkout was modified. No commit, branch or PR was made.

Recommendation: integrate this milestone and replay through the parent's shared
front door before another engine extension. If a second application slice is
desired, isolate `page_size_bytes` variable-shift handling as its own bounded
task. Address-range debug assertions/panic handling and database correctness
remain separate work; do not broaden this milestone to absorb them.
