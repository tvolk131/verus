# Infer proof composition: integrated acceptance

November 1, 2026. This report consolidates the checked redb proof-assistance,
shared native Option, array-callback, caller-indexing and address-range work.
Historical reports are preserved; their test totals are not added to the fresh
counts below.

## Source and artifact identity

- HEAD and `origin/verus-infer` remain
  `55bd67ad980e90e92c77efe4453a927f097c3386`. The assigned worktree branch is
  `t3/identify-next-priorities`; no manual branch/ref changes, commits or PRs
  were made. Work remains uncommitted on the `verus-infer`-based checkout.
- The original dirty 64/58 baseline was preserved. The two implementation
  deltas were reconciled at 67/61; the address extension used its reserved
  68/62 identity. Final parent identity is **cache 69 / policy 63 / schema 38**.
- The independently reviewed snapshot contains 85 changed/new files and nine
  matching frozen runtime artifacts. Subsequent differences are exactly two
  test repairs and the transfer-documentation correction listed in the manifest.
  No production code changed after that independent review.
- Independent matching runtime copies and the proof-input snapshot are in
  `/private/tmp/verus-infer-proof-assistance-0d9p49mm/final69`. The complete final
  reconstruction, including the documentation correction and this report, is in
  `/private/tmp/verus-infer-proof-assistance-0d9p49mm/final-package`. The manifest
  records the base commit plus every changed source/evidence hash, runtime hash,
  command, log hash, review receipt and fresh application receipt. The generated
  report/manifest exclude themselves from the source hash map to avoid recursion.

## What became inferrable or automatic

### Checked redb proof assistance

The original codec callers no longer supply packing assertions or byte-inverse
lemma calls. The page-size callers no longer supply handwritten shift-bound or
nonlinear-product proof blocks. With `--infer-goals`, bounded demand-driven
expansion uses existing exact helper equations, transparent constructors,
fields, literal in-bounds array projections and finite scalar operations.

Every proposed conditional theorem is independently checked by the existing
bit-vector backend with empty requires. The ordinary query receives that same
fact and must still prove the original call requirements, safety checks and
authored goal. No application identity, unchecked application summary or axiom
is introduced. Failed auxiliary proofs fail verification, not just the tactic.
Budgets remain 8 requests, 8,192 nodes, depth 64, 8,192 statement visits and
131,072 shared work units per ordinary function. This is not a time bound.

The independent review's ordinary indexing limitation is also repaired:

```rust
#[verifier::infer]
pub fn pair(value: u8) -> [u8; 2] { [value & 15, value & 7] }

fn caller(value: u8) {
    let result = pair(value);
    assert(result[0] < 16);
    assert(result[1] < 8);
}
```

Canonical vstd array-view/sequence-index lowering participates in the existing
checked proposal path. Lookalike identities, mismatched types, nonconstant or
out-of-bounds literal projections do not gain guessed facts. The wrong scalar
bound fails its auxiliary theorem; an actual out-of-bounds executable access
fails its separate bounds precondition. Both remove seeded stale exports.

### Shared native Option and complete array callbacks

Canonical `then_some`, `map_or` and `is_some` share checked readonly machinery
across scalar, scalar-newtype and record profiles. Eager values/defaults remain
eager, callback requirements apply only to `Some`, and exact callback-domain
facts remain unavailable to sound-only candidates. Effects, mutable callbacks,
authored callback contracts and unsupported construction forms fail closed.

Projected wrapper callers now retain projected helper contracts during
composition, rather than lifting them twice back to the source wrapper type:

```rust
#[verifier::infer]
pub fn base(wrapper: &Wrapper) -> u8 { wrapper.value }

#[verifier::infer]
pub fn query(wrapper: &Wrapper, condition: bool) -> Option<u8> {
    condition.then_some(base(wrapper))
}
```

By-value native byte-array callbacks also pass complete independent precision:

```rust
#[verifier::infer]
pub fn bytes(input: Option<u64>) -> [u8; 8] {
    input.map_or(0u64.to_le_bytes(), |value| value.to_le_bytes())
}
```

Generated callback equations and independent whole-output observations use
shallow extensional equality for actual by-value arrays, not opaque array
identity or a candidate-selected prefix. Before/after-poly boxing is kept
separate. Shared-reference observations retain identity; unsupported shared
array returns remain unsupported. The existing array-equality model is an
explicit fingerprinted dependency even when the enclosing function returns a
scalar after decoding an array callback result. Last-byte false assertions,
callback overflow, imported model tampering and stale exports are tested.

Exact inference of a scalar byte-decoding equation does not mean the ordinary
solver automatically proves every higher-level byte-inverse identity through
an Option input. That unaided caller-composition limitation remains distinct
from the now-successful callback soundness/maximality/exactness checks.

### Original redb address-range composition

The narrow canonical literal-message panic recognizer reuses the existing core
requires-false specification. The original panic call and its obligation remain;
unknown or effectful message construction is not discarded. Canonical public
transparent `Range<u64>` uses the existing `ExRange` layout specification; the
exception does not admit arbitrary proxies, destructors or user invariants.

The original `PageNumber`, `page_size_bytes` and `address_range` bodies remain
source-preserved, including `debug_assert!` and `start..end`. They infer every
intermediate shift/product/sum safety requirement without an order-20 shortcut.
Actual debug-on and debug-off configurations have separately checked domains.
The results concern Verus's checked-arithmetic domain, not total native release
no-panic behavior outside that domain.

## Useful application results, freshly replayed

| Slice | Established result | Representative positive verifier result |
| --- | --- | --- |
| Fedimint deadline | Original extracted helpers satisfy the authored intercepted-forward/unknown-height policy, deadline coverage and monotonicity/boundary follow-up | 8/0 cold and warm; 10/0 follow-up |
| redb codec | Roundtrip on the original region/order/index domain, injectivity, and imported-client composition without caller packing/inverse glue | 71/0 codec; 75/0 provider/injectivity; 12/0 imported caller |
| redb page size | Equation and maximum bound for every `u32` size with order at most 20, plus valid large-order boundary callers | 17/0 cold/warm and imported callers |
| redb address range | Exact checked-arithmetic domain, length/start equations, and containment under sufficient end-containment/address-space conditions | 8/0 original helpers; generic and boundary callers separately checked |

The codec domain is unchanged: `region <= 0xfffff`, `order <= 20`, and
`index <= (0xfffff >> order)`. The native codec oracle's 2,097,467 cases are
supporting finite tests, not exhaustive enumeration of that formal domain.

The range debug guard alone does not establish end containment. The unchanged
method proves the concrete witness `4090..8186` for a 4096-byte region with
regional start 4090 and page size 4096. A false universal containment goal
fails its authored postcondition while both helper summaries remain exact.
This is a counterexample to that proposed property, not a claimed redb security
vulnerability or a whole-storage correctness result.

The four-slice acceptance receipt independently checks **61 declared main
controls**: 37 positives, 7 semantic postcondition negatives, 16 safety negatives
and 1 auxiliary-BV negative. Their 315 repeated required inference checks are
not 315 distinct application theorems. Additional controller metadata/auxiliary
controls remain in the nested receipts and are not relabeled semantic proofs.
Warm native value summaries regenerate and reprove; these are not cache hits.

Fedimint's fresh common frontdoor is the explicit three-field extraction, not
the full request. Earlier child evidence additionally verifies a six-field
diagnostic and imported policy with only the external hash field omitted. The
real seven-field request remains blocked by `bitcoin::hashes::sha256::Hash`
translation. No opaque cryptographic specification or frontend bypass was
added. No gateway fund-safety, amount-authenticity, cryptographic, whole-database,
concurrency or crash-recovery theorem is claimed.

## Independent review and final validation

The final independent review reports no production soundness defect in the
new deltas/interactions. It ran 59 scratch compiler probes, the native Option,
panic/Range and relational suites, 17 Python tests normally and optimized, and
the immutable address frontdoor. It did not rerun the entire parent suites.
Its frozen version had two real test findings: an undeclared unit-test enum
alias and an incorrectly expected bounds-error string. Both were fixed locally;
the subsequent complete results below are parent validation, not reviewer reruns.

- **134 VIR unit tests** pass, including the independent complete-array versus
  reference-identity observation test and proposal identity/bounds tests.
- **573 Rust integration tests across all 27 root `infer*.rs` binaries** pass;
  none are ignored. This includes 18 native Option, 8 relational-goal and
  4 literal-panic/Range tests.
- **144 Python test executions normally and 144 under `-O`** pass across the
  disjoint infer/redb/Fedimint discovery patterns.
- Matching release compiler build and fresh **2,221 vstd verifications, 0 errors**
  pass. Rust formatting and `git diff --check` pass.
- All **four** application frontdoor slices report `proved` against the frozen
  matching nine-artifact runtime. Source/tool guards and pinned clean clones
  remain intact.

VIR library testing deliberately excludes the 18 documented unchanged doctest
failures. This is not a claim that every workspace test or unrelated example
passes. No new axioms or trusted application specifications were added by these
extensions. Existing primitive-byte contracts, array equality, checked native
frontend correspondence, imported specifications and the Verus backend remain
explicit trust boundaries; Rust std/libcore implementation bodies are unverified.

## Replay and evidence

```sh
python3 -B source/tools/check_infer_transfers.py \
  --fedimint /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint \
  --redb /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-infer-proof-assistance-0d9p49mm/final69/tools/verus \
  --solver /private/tmp/verus-infer-proof-assistance-0d9p49mm/final69/tools/z3 \
  --output /private/tmp/new-infer-transfer-replays
```

Fresh combined receipt:
`/private/tmp/verus-infer-proof-assistance-0d9p49mm/replays/transfer-suite-euj5ek05/receipt.json`.
Final parent command/log receipts are under the same scratch parent's
`validation/`, with `reviewfix_` test and `final69_` replay prefixes. Compiler/vstd
commands use `combined69_`; production sources and runtime hashes are unchanged
since that build. Exact commands and hashes are in `manifest.json`.

Independent review:
`/private/tmp/verus-independent-final-review-TP43iCFM/REPORT.md` and `receipt.json`.
See also the preserved `native-option-unification`,
`transfer-redb-proof-assistance` and `transfer-redb-address-range` reports for
their individual source, domain, trust and historical implementation evidence.
