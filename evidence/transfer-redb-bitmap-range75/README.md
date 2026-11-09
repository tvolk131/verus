# redb interval-mask membership and update framing

This bounded cycle starts from checkpoint
`3b05c1912b9d7314ad00c832c4065f4362562e02` plus the complete restored **57-file**
follow-up74 snapshot. All 57 changed hashes, 1,554 source hashes and nine runtime
hashes were checked before candidate changes. The original restoration and all
prior evidence remain unchanged. Candidate identity is **cache75/policy69/schema38**;
the parent's combined77/71 reservation is not used.

Parent integration adds the unchanged Fedimint fee-encoding slice and reconciles
the common receipt readers to policy69. It also removes the standalone bitmap
runner's optional native-erasure bypass: current local/repeated/imported proofs
keep default erasure checking, while selected providers additionally compile.
The preserved implementation handoff predates these runner/documentation edits;
parent replay receipts distinguish their fresh results from the counts below.

An independent Codex review of this production extension found no substantive
soundness/publication defect in its bounded scope. Its P3 diagnostic finding
was fail-closed: the shared classifier missed the backend's actual
`bitvector assertion not satisfied` wording. Parent now recognizes that message
as an auxiliary failure, retains mixed-failure classification, and adds
regression coverage for both. No engine or application proof was changed by
this diagnostic repair. Valid callers beyond conservative width/request budgets
or requiring additional nonlinear/cast reasoning remain explicitly unverified.

## Actual source and authored theorem

The pinned clean application is redb
`c82ab865bb362bc4af067d332adc44978ff64813`. `TransactionalMemory::get_page`
reaches `self.storage.read(range.start, len, hint)?` after the previously proved
range/length prefix. No useful remaining whole-method theorem is claimed without
a storage and reference-tracking contract. Rather than assume such facts, this
cycle selects the adjacent pure `bits_in_range(lo, hi)` at
`src/tree_store/page_store/bitmap.rs:223`, actually called by
`U64GroupedBitmap::resize` at line 355.

The original body is copied byte-for-byte, including the debug assertion, two
shifts, subtraction and final AND. The only fixture adapter adds public visibility
and `#[verifier::infer]`; no application file is edited. The full bitmap.rs hash
is `d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff`;
the extracted item plus newline hash is
`ec376aba7d0f4d7c1cc1e1054f13c3c045174efe3ea9d2c78125b0cfee935fa5`.

For every observed bit `bit < 64`, the independently authored ordinary caller
proves

```text
((bits_in_range(lo, hi) >> bit) & 1 == 1) <==> lo <= bit && bit < hi
```

An additional **authored by-value observer**, using resize's OR/AND expressions,
proves that `word | mask` sets all interval bits and `word & !mask` clears them,
while every bit outside the interval retains its original value. It is not an
extracted whole resize method, nor proof of its vector indexing or allocation.
No caller has a handwritten proof block or arithmetic/bitvector ghost lemma.

The original exact domains are preserved:

- debug assertions enabled: `lo < hi <= 64`;
- debug assertions disabled: `lo < 64` and `1 <= hi <= 64`, including equal and
  reversed intervals, whose masks are empty.

The bit observation's `bit < 64` is its own shift safety condition, not a
narrowing of the helper. Release-mode checked subtraction/shift obligations are
not replaced with Rust wrapping semantics. No total no-panic claim is made for
unchecked release executions outside these domains.

## Frozen failures and two linked mechanical repairs

The matching tools74 baseline proves helper soundness, safe-domain maximality
and output exactness, then rejects the generic membership proof with
`not supported: bit-shift with possibly negative shift`, pointing to the
unchanged `u64::MAX >> (64 - hi)`. The summary correctly contains mathematical
subtraction, but the BV encoding treats its result as signed. This is proof
plumbing failure, not a false theorem or a helper safety counterexample.

1. Ordinary goal expansion now normalizes computed/signed shift amounts **only
   inside proposed theorems** to explicit unsigned clips. Finite unsigned atomic
   shifts stay unchanged. Mathematical expressions need a conservative width
   estimate at most 128 bits; unbounded/architecture-dependent input variables
   remain outside this ordinary-goal fragment.
2. The same bounded normalizer is reused for checked bitvector range proposals
   inside inference. Without this second change, a fresh imported forwarding
   helper still fails on the exact same subtraction-valued amount. Existing
   pointer-sized range-hint handling is retained, not newly generalized.

Every generated clip is budgeted. The exact same proposed fact is checked by
the existing BV backend and used by the ordinary query. Original expressions,
authored goals, panic/overflow/shift checks and all three required inference
proofs remain authoritative; they must connect any clipped proposal to the
original expression. No arithmetic bounds are assumed merely because a cast
was proposed. No converter semantics, application names, axioms, unchecked
summaries, export gates, wire schema or author-owned invariants are added.
Existing goal request/expansion/depth/visit/shared-work limits remain in place.

## Controls, validation and replay

The dedicated runner has **31 cases**: 11 positive, ten safety, four independent
ordinary semantic failures, four metadata rejections, one unsupported syntax
rejection and one intentional solver-budget failure. All **20 negative exports**
are first seeded and must be removed. The 25 non-frontend/non-budget cases
recheck **75 mandatory exact inference obligations**. The semantic mutant changes
the final AND to XOR; its independent full-mask postcondition fails with optional
goal assistance disabled, not merely in an auxiliary query.

Local debug/release, repeated and poisoned-cache proofs are fresh. This profile
reports cache misses even on repeats; no candidate-cache hit is claimed. Imported
forwarding is checked cold/repeated with immutable provider guards, and stale
policy, exactness, summary and trusted-model metadata are rejected. Finite native
oracles test 2,080 debug intervals and 4,096 release intervals, with respectively
1,064,960 and 2,097,152 update-bit observations. They support, not replace, the
generic proofs.

The complete command/log receipts, fresh versus audited validation counts,
baseline-relative delta, complete changed-file inventory, source/runtime hashes
and reconstruction replay are packaged under
`/private/tmp/verus-redb-next75.sOFIw9yU/handoff`. Initial setup failures and
exploratory runs remain separate from accepted final runs. Matching vstd is
verified against the new compiler; unchanged builtin/macro artifacts are copied
from tools74, not falsely reported as rebuilt. No shared Cargo home, parent
checkout, pinned application, branch/ref, commit, PR or network is changed.

```sh
GIT_OPTIONAL_LOCKS=0 python3 -B source/tools/check_redb_bitmap_range.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-redb-next75.sOFIw9yU/tools75/verus \
  --solver /private/tmp/verus-redb-next75.sOFIw9yU/tools75/z3 \
  --output /private/tmp/fresh-redb-bitmap-range75
```

## Next authored boundary and trust limits

To lift the mask theorem to real `U64GroupedBitmap::resize`, the author must
state a representation relation between `len`, `data.len()` and the bit-word
view, including valid vector indices, the final partial word, the preservation
policy for bits outside logical length, and safe arithmetic for
`word_first_bit = word * 64` / loop bounds. Inferring that relation from one mask
helper is not justified. Buddy allocator occupancy and disjoint live-page
invariants remain separate author-owned requirements.

For whole get_page, the minimum next boundary is a storage-read contract
specifying successful returned byte length and contents at the requested
range, error behavior, valid storage bounds, and its connection to the existing
reference/alias tracking. Arithmetic alone supplies none of those facts.

Trust remains the ordinary Rust/Verus frontend and backend, solver, existing
vstd native specifications and requires-false panic model. Extraction fidelity
is hash-checked, not a formal compiler correspondence proof. No whole bitmap,
allocator/database, IO, concurrency or crash-recovery theorem follows.
