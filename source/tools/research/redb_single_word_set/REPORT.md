# redb single-word allocation membership transition

The frozen original-body authored invariant is proved for the selected domain.
The unaided current-engine baseline remains blocked; this is not an automatic
stateful inference success. Final raw receipt:
`/tmp/redb-stateful-research-20261109/single-word-set-37u9m8gf/receipt.json`.
`RESULTS.json` contains compact outcomes, source/tool/log hashes, commands and
all trial receipt locations. No prior transfer evidence was changed.

## Source and scope fixed before proofs

redb pin: `c82ab865bb362bc4af067d332adc44978ff64813`, clean shared checkout
`/tmp/verus-infer-cycle-20261109/applications/redb` (read only throughout).
Source: `src/tree_store/page_store/bitmap.rs`, SHA-256
`d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff`.
Exact original items and source locations are in `original/` and `freeze.json`:
U64GroupedBitmap (232), data_index_of (299), select_mask (303), get (365),
set (373). The transition closure is set plus its two scalar helpers and the
imported Vec operations; get is a source-connected observation.

Target/domain/policy were committed at `6a8f807b`, and the proof-free semantic
fixture at `72898742`, before the first solver attempt. Both descend from the
prepared Verus source `3b05c1912b9d7314ad00c832c4065f4362562e02`.

The domain is exactly **one physical word, 1..=64 logical pages, allocated
padding, and bit < len**. All logical bit patterns and already-allocated target
bits are supported. This was selected as a meaningful bounded leaf transition,
not extracted from a later counterexample. No freshness restriction was added.
A caller supplies the invariant; initialization by new_full is not proved here.

The abstract view is the sequence of logical bit memberships. The abstract
allocation count is its count of true entries, not a new implementation field.
The proof establishes:

- membership is the previous view updated at the target to true;
- every other logical page is unchanged;
- count increases by one iff the previous target was free, otherwise unchanged;
- logical length, word count and allocated padding are preserved;
- return is true iff all logical pages are allocated;
- get returns the corresponding membership.

## Fidelity and frontend adaptations

All selected executable bodies and the original Vec/u32 struct are retained.
`replay.py` checks exact extraction bytes against the pinned source, frozen
hashes/line locations, and original bodies after removing marked erased proof
additions. It also checks that the assisted fixture has precisely the baseline
contracts and executable code after those additions are erased.

Extraction adds the Verus wrapper/prelude, semantic definitions, authored
requires/ensures and named return binders. Rust's std Vec prelude and the
original alloc::vec::Vec denote the same vector type. Unselected redb imports,
other methods, checksumming, constructors and tests are outside this closure.

Initial frontend rejections were retained, not counted as failed properties:
public open specs exposed the non-public datatype; intermediate pub(crate)
spec visibility spellings were rejected too. The final standalone fixture uses
private get/set and private wf/view specs. This is a visibility-only extraction
adaptation, not a cross-crate interface result. The current frontend also requires
`final(self)` for post-state references; it replaces the initial fixture's old
post-state spelling without changing the authored policy or domain. The original
executable bodies did not require frontend rewriting. No weaker executable
comparison fixture was substituted.

## Checked baseline, assistance and results

Every JSON proof run checks the entire crate with `--infer-goals --no-cheating
--output-json --multiple-errors 20`. There are no #[infer] items or exported
inferred interfaces in this authored-invariant experiment; no claim about exact
inference checks, interface caching or imported inferred summaries is made.

The baseline contains no proof blocks, reveals, solver annotations or local
lemmas. It reports **3 verified, 1 failed function**. Five postcondition
diagnostics in set remain: padding/invariant, abstract view update, unaffected
membership, count conservation, and return/all-allocated equivalence. Arithmetic,
index safety, the scalar helper contracts and get's membership contract need no
handwritten proof glue. Length and physical word count are framed automatically.
An early controller incorrectly classified `encountered-error` as a frontend
error; its raw receipt remains. The corrected gate distinguishes the ordinary
proof result (`encountered-vir-error: false`, verified count present) and requires
full-crate checking.

`word_laws.rs` adds reusable **library proof assistance**, not an engine rule:

- checked bit-vector laws for OR-singleton membership and full-word membership;
- a bounded word-to-sequence update/count bridge using the existing verified
  Seq::lemma_count_prefix_update;
- full logical view iff full word, under explicitly allocated padding.

These are generic u64/sequence theorems, with no redb names, admitted bodies or
application update summaries. They factor finite bit reasoning, extensionality
and count-of-update out of callers. The authored fixture still supplies erased
snapshots, explicit theorem calls and representation/trigger bridges. It is not
annotation-free. One intermediate missing-trigger frontend rejection and one
padding-premise proof failure were fixed with explicit quantifier triggers and
an explicit padding bridge; neither fix changed a semantic contract or domain.

Final outcomes (fresh final source/tool hashes remained stable during replay):

| Fixture | Result | Checked scope |
| --- | --- | --- |
| baseline, --infer-goals only | expected postcondition failure | 3 verified, 1 failed function |
| original-body assisted invariant | proved | 4 executable + 4 proof bodies; 8 verified, 0 errors |
| return == MAX mutated to != MAX | authored return postcondition rejected | 7 verified, 1 failed function |
| unrelated permission grant/idempotence + flag-count | proved | 2 executable + 4 proof bodies; 6 verified, 0 errors |
| permission OR mutated to XOR, same domain | authored postconditions rejected | 4 verified, 2 failed functions |

The redb return mutation fails specifically the all-logical-pages-allocated
postcondition, without a precondition/assertion/frontend/resource failure.
The unrelated XOR control rejects granting an existing right and the count
policy. Its unchanged-other-right property is also mechanically unproved with
the OR-specific assistance; that ancillary diagnostic is not claimed false.
No unsafe operation, verification bypass, assumed first-party transition or
native oracle supplies a proof premise.

## Native erasure support and trust inventory

Original extracted Rust and Verus-erased assisted Rust are compiled separately,
each with debug assertions enabled and disabled/optimized. All four executions
pass **149,760 cases each**, with matching
`cases=149760 fingerprint=f1599ba3316b42b9`. The native harness covers all logical
lengths and targets, empty/full/singleton/seeded word patterns, checks membership,
count deltas, unrelated bits, padding and return semantics, then repeats each
set to check idempotence. These finite tests support erasure behavior; they are
not exhaustive proof of all 2^64 physical patterns and do not replace the proof.
The native test main is deliberately outside the selected Verus proof scope.

Semantic policy: supplied allocation view, invariant, conservation and return
meaning. Representation laws: proved generic word/sequence/padding theorems.
Mechanical proofs: checked BV reasoning, sequence extensionality, casts,
quantifier instantiation and the verified count-prefix update lemma.
Imported assumptions: Verus/Rust type and arithmetic model, verified vstd
infrastructure and standard Vec/index/mutation contracts. Rust's concrete Vec
implementation, compiler correctness and SMT solver correctness are trusted;
no redb implementation is declared external or assumed.

Own matching compiler, macros and vstd were built from unchanged engine/library
source at the prepared base. The vstd build verifies **2,221 bodies, zero errors**.
Rust is pinned 1.99.1. The standalone Z3 4.16.0 was copied, not symlinked, into
this checkout's ignored source/z3; SHA-256 is
`e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765`.
The parent checkout and its artifacts were never modified or rebuilt.

## Reproduce and remaining work

From source/, with /home/dev/.cargo/bin first on PATH and CARGO_BUILD_JOBS=2:

```sh
cargo build --release -p rust_verify -p verus -p cargo-verus
./target-verus/release/cargo-verus build --release --manifest-path vstd/Cargo.toml
```

Then from the checkout root (create a private persistent output directory):

```sh
python3 -B source/tools/research/redb_single_word_set/test_replay.py
python3 -B source/tools/research/redb_single_word_set/replay.py \
  --redb /path/to/clean-pinned-redb --output /path/to/private-existing-output
```

Five Python guard tests pass: original/assisted fidelity, changed state-update
rejection, hidden erased admission rejection, and assisted-domain narrowing
rejection. --baseline-only reruns just the proof-free baseline. Replay never
builds, fetches, writes application inputs or disables verification. Raw trials,
fixtures, commands, stdout/stderr, tool/source hashes and build logs persist in
`/tmp/redb-stateful-research-20261109`; no binaries/raw dumps are committed.

No obligation remains open **inside this frozen authored-invariant target**.
Unaided --infer-goals still cannot bridge the bit representation, quantified
view and sequence count. Automatic theorem selection/instantiation remains a
separate assistance task; this report does not claim an engine extension.

The next stateful scope can retain the same original set body while generalizing
to multiple words: define membership via index / 64 and index % 64; prove
quotient/index bounds, vector-update framing, padding preservation and the
corresponding global count-of-update. That invariant must be frozen before its
proof attempt. Nothing here proves alloc search, clear/free, resize, BtreeBitmap
parent summaries, database transactions, durability, concurrency or crash recovery.
