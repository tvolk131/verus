# Frozen goal: native bitmap resize and stateful proof automation

Status: **FROZEN FOR IMPLEMENTATION, NOT PROVED OR ACCEPTED**. Frozen on
November 3, 2026 against accepted cache82/policy76/schema38. Feasibility is
cycle zero; implementation/review cycle one begins only after this freeze.
Reassess after at most three implementation/review cycles. A surviving blocker,
review finding, or cycle limit is not success. Do not substitute a smaller goal.

## Native source and unchanged executable target

Native redb is pinned at `c82ab865bb362bc4af067d332adc44978ff64813`.
`src/tree_store/page_store/bitmap.rs` has SHA256
`d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff`.
The frozen source ledger identifies the complete `U64GroupedBitmap` declaration
and original `bits_in_range`, `required_words`, `data_index_of`, `select_mask`,
`len`, `resize`, `get`, `set`, and `clear` items and bodies.

Retain the native inclusive `for`, vector growth, mutable element borrow,
arithmetic, unconditional/debug assertions, branches and return values. No
helper implementation substitutions, native source edits, while-loop rewrite,
array model, fixed-size example, or weakened assertion may discharge this goal.
Explicitly accounted erased annotations, named returns, visibility adjustments,
and the ghost iterator label are allowed. Audit executable projections and
actual erasure/compilation separately; hashes are not an extraction theorem.

## Representation, domains, and operation intent

Authors own `valid`, stored-bit observation, and high-level operation contracts:

- `valid(b) := b.len as int <= 64 * b.data@.len()`.
- `bit(b,q) := b.data@[q/64] & (1u64 << ((q%64) as u64)) != 0` on stored bits.
- Initial words, spare storage and padding are arbitrary. All `u32` old/new
  lengths remain included. No minimal-storage equality, zero-padding condition,
  growth-only restriction, small bound or overflow-hiding assumption is allowed.
- Native get/set/clear require validity and a live index. Set's result means the
  **entire final selected stored u64 equals MAX**, including padding, not that
  the logical bitmap is full.

For native resize from entry state E, new length n, full flag f, old stored
length L, D=MAX if f else 0, and M=max(L,ceil(n/64)), prove:

1. Result is valid, logical length is n, and stored length is exactly M.
2. For every original stored bit q, its result is f iff E.len<=q<n; otherwise
   its original value is preserved. This includes all padding and spare words.
3. Every appended **entire stored word**, including padding, is exactly D.
4. Every newly live bit equals f, including growth entirely inside old storage.
5. Native shrink preserves the entire stored sequence. Arbitrary shrink/regrow
   reinitializes the hidden interval. Compose native resize with get/set/clear
   at arbitrary valid indices, preserving exact frames and the true set flag.

Universal proofs use Verus's default Either32Or64 treatment (or explicitly
qualified equivalent proofs), without cheating/global-size assumptions.
Debug and release helper domains stay distinct, and no valid operation is
excluded. Host64 finite tests are not 32-bit execution or universal evidence.
Allocation/Clone/resource/abort semantics remain the existing vstd boundary:
returning-call correctness is not allocation availability or OOM recovery.

## Frozen author budget and proof kind

This goal is automation of **ordinary authored high-level state contracts**,
not synthesis of an exact mutable-state specification. Certified imports must
say precisely which inferred pure-helper contracts and ordinary state contracts
they use. Sound-only mutation summaries are not exact-state synthesis.

Allowed authored proof intent and annotations:

- The representation/observations, high-level operation contracts, and client
  statements of the properties above; no routine helper contracts.
- One entry ghost snapshot E, and one ghost label for the native iterator.
- One semantic loop predicate, at most three flattened clauses: logical length
  equals n; stored length equals M; and the pointwise stored-bit relation below.

With k=(E.len as int)/64+cursor.index(), the semantic relation is, for every
0<=q<64*M:

`bit(current,q) == if E.len<=q<n && q<64*k { f }
 else if q<64*L { bit(E,q) } else { f }`.

The final selected native functions and composition clients have **zero**
handwritten local assertions/proof blocks, auxiliary lemmas, broadcast
registrations, routine helper contracts, extra scalar invariants, or executable
edits. In particular, the old authored tuple contract and word-frame lemmas
must be removed, not grandfathered or relabeled inferred. Pure helpers must
infer through reusable production support. Engine-generated erased snapshots,
assertions, bookkeeping, observation lemmas and invariant obligations are
allowed only if backend checked, resource bounded, and fully source accounted.
Test scaffolds may explicitly state desired theorems and false goals, but may
not supply hidden helper proofs to the selected native proof.

## Explicit trust decision: native unsigned div_ceil

The parent authorizes a **new trusted standard-library correspondence family**
for native unsigned `div_ceil`, in the existing vstd numeric/spec-model style.
It must use the actual resolved standard function/type identities, exact
mathematical quotient/remainder ceiling, and retain the divisor-nonzero safety
obligation. Include overflow/safety obligations if required by the native API;
do not silently replace it by unchecked `a+b-1` executable arithmetic.

This is new trust, not an already-existing model and not a proof of Rust std's
implementation. Audit precisely which unsigned types/spec declarations are
added. Counterfeit methods, wrong types/semantics, zero divisors, and bad
postconditions must not inherit it. No application axiom, assumed bitmap/frame
summary, external-body bitmap wrapper, or trusted loop invariant is authorized.
Reuse the existing Vec::resize/Clone correspondence; it is not new progress.

## Before evidence and new-credit rules

The sealed cycle-zero report establishes native div_ceil frontend failure,
tuple-inference failure, loop/context and quantified bit/frame failures. Its
manual diagnostic replaces required_words and therefore is **not** a native
helper-chain proof. The diagnostic needed four generic bit lemmas/bv asserts,
one broadcast registration, two snapshots, seven invariant clauses, three
local proof blocks/eleven asserts, and four lemma calls. Composed diagnostics
also retained two older word lemmas and authored routine helper contracts.
The final native proof must remove this mechanical assistance, not copy it into
an application-specific proof module.

Existing range support, vector growth/Clone, mutable element borrowing, four
already-automatic standalone arithmetic theorems, finite native execution,
and existing failure cleanup receive zero new-feature credit. Retain actual
same-body baseline failures, the diagnostic manual assistance inventory, and
the final author's remaining work side by side.

## Acceptance and cycle-one checkpoint

All twenty draft acceptance rows remain required; `scope.json` binds them.
Each final row needs evidence and an independent verdict, not a prose promise.
Cycle one first checks that the **unchanged helper chain** reaches a checked
loop path within the author budget. If not, report the exact blocker and
reassess; do not conceal it with a helper substitution or extra author work.

Require nonbitmap regressions for scalar products, native numerical APIs,
range/Vec growth/frame updates, and demanded bit observations; reject their
false-property/safety variants. Retain native-body mutations for wrong full
updates, skipped word, stale regrowth, changed prefix/padding/storage/logical
length, bad index and wrong set flag. Missing/weak preconditions or invariants
must not establish unsupported claims. Allocation/resource timeout is
inconclusive, never proof credit.

Require actual local/provider/imported executions, exact dependency/artifact
binding, cold/warm mandatory checks, seeded failed inferred-export removal,
and accurate stale-rlib versus interface/publication qualifications. Build a
new matching runtime/vstd privately; never rewrite runtime82/toolchain/native
inputs or shared Cargo caches. Run existing relevant regression/frontdoors,
normal and optimized reader tests, malformed real-receipt probes and isolated
guard removals protecting the actual composed gate. New independent review,
repair/recheck, exact baseline-relative reconstruction and a per-row accepted
handoff are required before completing the goal.

Preserve original accepted dirty files and pins, own HEAD/index/branch; shared
orchestration-ref observations are recorded without attribution. No commits,
manual branches/refs, pushes, PR actions, network or sub-scope expansion.
Allocator correctness, storage, concurrency, recovery, crash/torn-write,
fsync/durability and whole-database safety are explicitly outside this goal.
