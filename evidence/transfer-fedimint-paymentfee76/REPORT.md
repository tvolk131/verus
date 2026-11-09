# Actual native PaymentFee arithmetic: bounded candidate76

## Verdict and scope

The unchanged extracted `PaymentFee::{absolute_fee,fee,add_to,subtract_from}`
bodies at Fedimint b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236 verify with
the actual nested `PaymentFee { base: Amount, parts_per_million: u64 }` and
`Amount { msats: u64 }` representations. The unchanged native constant
`Amount::from_msats` constructor is inferred too. No executable body,
parameter, field type, return type, native denominator, or arithmetic order is
rewritten. The pinned application checkout is read-only.

This is arithmetic verification of extracted items, not verification of the
Fedimint dependency graph or its running application. There are no application
axioms, assumed application summaries, state/funding/crypto hypotheses, or
claims of full gateway/funds safety. Fee limits and acceptance/charging policy
remain author-owned. No durability/state proof work follows this cycle.

## Native properties, over complete u64 parameter domains

Let M = 2^64 - 1, D = 1,000,000, m = input millisatoshis, p = ppm,
b = base millisatoshis, and cap(x) = min(M,x) for the nonnegative expressions
used here. The authored goals establish:

- F(b,p,m) = cap(cap(m*p)/D + b), with integer multiplication/division.
- `fee(m).msats = F(b,p,m)` and F >= b.
- When m*p <= M and m*p/D+b <= M, F = m*p/D+b.
- `add_to(m).msats = cap(m+F)` and its result is at least m.
- `subtract_from(m).msats = max(0,m-F)` and its result is at most m.
- Under m+F <= M, adding is exactly m+F; under F <= m, subtracting and
  then adding F mathematically recovers m.
- F is nondecreasing in m and under componentwise increases of b and p.
- Zero-fee identities and multiplication/base/addition/subtraction saturation
  endpoints hold, including F(0,M,M) = floor(M/D), not M.

These are explicit, independently authored intent contracts, not inferred
business policy. Componentwise-order monotonicity supplies the arithmetic
consequence intended by `is_within`; the actual `is_within` method and derived
`Amount::Ord` implementation are not extracted/proved here. No converse,
strict monotonicity, policy-limit enforcement, or gross/net inverse without the
stated domain is claimed. In particular saturation occurs BEFORE division in
the real native method; it is not silently replaced by a wider product.

## Restoration and frozen baseline

The isolated checkout starts clean at
3b05c1912b9d7314ad00c832c4065f4362562e02. The COMPLETE reviewed dirty
cycle75 patch is applied exactly once to that clean HEAD. All 66 changed-file,
1,559 source, 1,084 Rust/build, nine matching runtime hashes, and both native
ledgers (1,199 Fedimint and 132 redb files) are verified. The original handoff,
sealed tools75, and pinned native inputs are not edited.

Frozen full native items precede candidate work. Tools75 reject these same
native fee bodies specifically because unsigned native `saturating_div` is
unsupported. The final replay repeats that baseline blocker with the same
scaffolded provider input. `native-items.json` records exact extraction bytes
and source/item hashes. Existing Infer singleton-wrapper projection is a
candidate-generation implementation detail: equations are lifted back and
all required checks run against the original typed bodies, not projected
executables. Only Copy/Clone derives needed by this scaffold are retained;
serde/encoding/error derives, unrelated methods/constants, and preceding
documentation are omitted. The comment INSIDE `absolute_fee` is preserved.

## New trusted standard correspondence

The sole new native specification family is unsigned `saturating_div` in
`vstd::std_specs::num`, expanded for u8/u16/u32/u64/u128/usize alongside the
existing unsigned arithmetic models. Its contract requires divisor != 0,
returns `(dividend/divisor) as unsigned`, and is no-unwind ONLY under that
precondition. Divisor zero still panics; there is no total/no-panic or zero-result
specification. For unsigned operands no nonzero division can overflow, so
ordinary unsigned division and unsigned saturating division coincide.

The exact native/std correspondence trusted for this closed family includes
both sufficiency and necessity of divisor != 0 for a normal successful return,
and quotient equality on that domain. The maximality gate uses necessity only
on its counterfactual normal-return path, as other closed native models do.
Ordinary verification and soundness continue ASSERTING the precondition.
This necessity is native/Rust-library trust, not something proved merely by
writing an `assume_specification`. Signed saturating division is not admitted.
Runtime cross-checks cover every nonzero u8 pair, a real u64 divide-by-zero
panic, and fee saturation vectors; they are supporting tests, not an exhaustive
u64 runtime proof or a proof of Rust's standard-library implementation.

Admission requires the core num static identity and vstd unsigned-module proxy
with the exact `saturating__div` suffix, matching native unsigned operand and
return widths/modes, two parameters, no generics/body/async/unsafe/infer flag,
no-unwind, and exactly the native rhs-not-zero precondition (including the
frontend's canonical not-equality/local-read forms). Near misses are rejected.
All proposal formulas remain untrusted and must pass soundness, maximality,
and exactness. The interface binds the exported API and trusted model artifacts.

Exactly two reusable integration blockers beyond adding division are addressed:

1. Owned record-return interpretation can reuse the already identity/type-gated
   standard saturating numeric models and propagates their safety obligations.
2. Maximality admits only the newly gated unsigned native division precondition
   as an exact normal-return-domain condition, never arbitrary application
   preconditions. Scalar, read-only-record, owned-value and sound-mode paths
   all retain the divisor safety obligation.

Candidate identity is cache76/policy70/schema38. Parent reconciliation to
78/72 is outside this child. Shared controllers, hosted CI and guide files
are not changed relative to restored cycle75.

## Fresh checks and negative controls

The dedicated runner is `source/tools/check_fedimint_paymentfee76.py`; run it
with `python3 -O -B`, explicit pinned application/tools75/tools76/solver paths,
and a new `--output` directory. It uses checked exceptions, not Python asserts.
It guards native, source, runtime and Rust compiler/shared-library/std-rlib
inputs and checks the original application pin/cleanliness and engine HEAD.

Each final provider phase verifies 28 obligations and 17 exec bodies, including
all five inferred helpers with all three mandatory checks. Each imported client
phase verifies the same 11 authored arithmetic goals. There are four public
certified summaries; private `absolute_fee` is locally checked, not falsely
reported as exported. Each positive export is freshly cleared and snapshotted.
Cold/warm compiler argument arrays are identical. Warm observes a real
constructor candidate-cache hit; the other four functions honestly regenerate
record/owned proposals (`miss`, `regenerated_state`). Every inferred helper
performs nonzero fresh required-check solver work. Proof results are not cached.

Twenty final cases comprise four positives, five semantic failures, one safety
failure, three frontend near misses, six import rejections, and one genuine
resource-incomplete rejection. Every final negative begins with a seeded valid
export and must remove it. Controls cover altered denominator, actual zero
divisor, absent Infer annotations, unsaturated/strict-add/unsigned-subtraction
false policy, signed/denominator-type/identity near misses, old schema/policy,
missing/failed mandatory proof, API artifact binding and native model binding.

The absent-annotation control imports the unused certified provider to retain
Infer-aware publication scope, following cycle75. An earlier no-annotation,
NO-certified-import attempt is preserved separately: ordinary non-Infer
verification fails the goal but is outside Infer's stale-export cleanup scope.
It is NOT counted as a passing negative, and no third engine change is made.
An initially invalid rlimit=0 attempt is likewise not resource-proof evidence;
the final resource control uses the supported 0.000001 limit. Superseded
frontend/gating/harness attempts and their logs remain intact.

Focused validation includes 144 VIR tests, 70 tests across eight relevant Rust
binaries (including six new native division tests), 2,221 vstd proofs, and the
dedicated receipt tests in normal and optimized Python. The latter exercise
mandatory check removal, scope/body/domain/cache/fresh-work checks, live
certificate/cache gates and run-case-to-validator wiring. Native debug and
checked-release runs each exercise 1,331 fee vectors, 65,280 nonzero u8 division
pairs and a caught native u64 zero-divisor panic. Full-u64 claims above come
from symbolic Verus proofs, not from finite native sampling.

## Delivery and reconstruction

Final immutable logs, command argv/environment records, parsed receipts,
baseline/candidate source and Rust/build/runtime hash ledgers, exact inventory,
and the baseline-relative `candidate.apply_patch` are in the separate private
cycle76 handoff directory recorded by the completion response. Reconstruction
applies that delta to plain file fixtures copied from the restored dirty
baseline, checks every resulting candidate hash and the exact inventory, and
does not create a checkout/worktree/branch. No commit, push, ref/PR change,
shared Cargo-home write or subdelegation occurs. Private build jobs are one;
the cycle reuses its own clone-on-write Cargo cache/target and tool outputs.
