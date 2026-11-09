# Explicit bitmap state proof (cycle77, no engine changes)

This standalone application-led pilot uses the reviewed dirty cycle75 baseline
at clean checkpoint `3b05c1912b9d7314ad00c832c4065f4362562e02` and sealed
tools75 (cache75 / policy69 / schema38). It does not change the shared transfer
controller, guide, CI, engine, or vstd. The reserved candidate77 identity is not
used. There is no additional trusted application summary or axiom.

## Actual source-connected result

At redb commit `c82ab865bb362bc4af067d332adc44978ff64813`, the **original
executable bodies** of `U64GroupedBitmap::get`, `set`, and `clear` verify against
the explicit, author-supplied abstraction in `abstraction.verus.rs`:

```text
valid(bitmap) := len <= 64 * data@.len()
live(bitmap, i) := data@[i / 64] & (1u64 << (i % 64)) != 0
```

Multiplication, division and indexing in this relation are mathematical. The
representation is equivalent to `data.len() >= ceil(len / 64)`, not equality;
spare stored words, including after a shrink, are allowed. `live` is only used
as a logical observation for `0 <= i < len` under `valid`. The invariant is an
explicit method pre/postcondition, **not** an automatically assumed type invariant.

`get` returns this view at every valid live index. `set` and `clear` preserve
representation, logical length and vector length, change the selected bit to
true/false, and preserve **all other live bits**. Their contracts also specify
the exact vector update, so every other stored word is unchanged. Set's return
flag is equivalent to the **selected full stored u64 being u64::MAX**, including
padding. It does not mean all logical bits or all bitmap words are full.

The method domain is precisely representation validity plus `bit < len`, with
no one-word bound, array replacement, fixed-size model, special length bound,
or additional padding-value constraint. The generic proofs cover u32 lengths
and arbitrary represented Vec lengths. Example clients exercise bit 64, spare
words and the one-live-bit/padding distinction; they do not replace the proofs.

## Authorship and inference boundary

- `contracts.json`: author-written high-level method contracts and an exact
  tuple-result contract for `data_index_of`, all checked against original bodies.
- `word_facts.verus.rs`: **two explicitly authored proof lemmas**, each checked
  with the existing bit-vector backend, then broadcast to ordinary SMT queries.
  These are proof assistance, not trusted summaries, inferred lemmas, or axioms.
- `select_mask`: the only inferred provider function. Existing Infer supplies
  its exact shift-safe domain and result, with mandatory soundness, maximality
  and exactness checks. Imported clients additionally infer a transparent relay.
- `get`, `set`, `clear`: **ordinary verified authored contracts, not inferred
  mutable-record summaries**. This pilot does not add or claim broader mutable
  Infer engine support. The goal-assistance flag is retained, not credited as
  synthesizing the authored frame lemmas.

The pre-engine freeze and original rejected tuple-inference probe are retained
in `/private/tmp/verus-bitmap77.NZMBuEPR`. Native `data_index_of` returns a pair,
which the existing scalar Infer signature profile rejects. An author-written,
ordinarily verified tuple contract avoids unnecessary engine expansion. The
remaining native operations need no engine improvement. Earlier experiments
with malformed mutable-postcondition syntax, broadcast scope, and insufficient
client constant facts are preserved as failed probes, not accepted proofs.

All six original items (struct plus five methods) are exact hashed extractions
in `*.source.rs`. The renderer exposes struct/field/helper visibility, inserts
contracts and result binders, and annotates `select_mask`. All five method
body byte strings remain identical, **including clear's formatted assertion**.
The struct only gains visibility. No native body or pinned checkout is edited.
The complete original source file, item/body hashes, and source line numbers
are captured on every replay. Extraction fidelity is checked, not formally
proved as a Rust-to-Verus compiler correspondence theorem.

## Replay and fail-closed controls

```sh
GIT_OPTIONAL_LOCKS=0 python3 -B source/tools/check_redb_bitmap_state.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-redb-next75.sOFIw9yU/tools75/verus \
  --solver /private/tmp/verus-redb-next75.sOFIw9yU/tools75/z3 \
  --compiler /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin/rustc \
  --output /private/tmp/fresh-redb-bitmap-state77
```

The fresh output directory is mandatory and must be outside source/application/
runtime trees. Cargo home/target and candidate caches are private; jobs and
verifier threads are one. Git context is sanitized and optional locks disabled.
No Cargo engine rebuild or redundant target tree is needed. All existing native
erasure checks and `--no-cheating` remain enabled.

The 54-case manifest requires ten local/provider/import positives across debug
and release, cold-to-warm cache hits with fresh mandatory rechecks, and 44
negative exports seeded with a valid old interface and required to disappear.
Positive provider/client output paths are emptied by archiving between phases;
compiler arguments and source paths remain identical for real warm cache hits.
Imports are bound to the checked warm provider certificate fingerprint, with
immutable rlib/interface guards. The cache/policy/schema identities are observed.

Negative coverage includes wrong get/set/clear behavior, length/other-word
corruption, the false whole-logical-bitmap return interpretation, invalid bits
and indices, empty/short storage, missing representation, Rust aliasing E0499,
an intentional solver budget failure, and four distinct corrupt-import faults.
Removing the two checked word lemmas is a **proof-dependency control on a true
theorem**, not a claim of false native behavior. It is retained separately from
the actual semantic body mutants. Diagnostics, classification, scope, exactness,
publication, seeding, source hashes, and completeness are checked with explicit
Python exceptions, including under `python -O`.

Independent native Rust tests execute unchanged methods and compare bit
observations to a division/modulo oracle: 1,228,980 live-bit observations per
mode, exact stored-word frames, flag/padding counterexamples, invalid inputs,
and 65,536 helper index decompositions. Debug and release are distinct builds.
`select_mask(64)` panics in native debug and wraps in native release; Verus's
inferred scalar contract intentionally describes the checked shift-safe domain.
This does **not** narrow any valid bitmap operation: decomposition modulo 64
always supplies an offset below 64. Original method assertions are unconditional
and apply in both native modes.

## Explicit boundaries

No constructor, deserializer, caller population, actual resize loop, allocation,
allocator occupancy/disjointness, alias escape, storage/fsync, IO, concurrency,
crash recovery, or whole-database property is established. Native callers must
establish `valid`; exposing fields for proof clients is not an encapsulation
proof. Neither an extracted observer nor these method proofs certify resize.
Rust/Verus, the ordinary solver/backend, existing Vec and panic specifications,
the pinned runtime/toolchain, and OS execution remain trusted. No trust is added
for application behavior. The invariant and high-level contracts are authored
here, not represented as independently reviewed.
