# Checked proof assistance for the frozen redb transfers

November 1, 2026. This is a new evidence record, not a revision of a historical
transfer report. The implementation starts from the **dirty frozen parent
baseline** at HEAD `55bd67ad980e90e92c77efe4453a927f097c3386`, not clean HEAD or
master. All 60 supplied baseline file hashes were checked before editing and
archived independently. The supplied restore patch used numbered hunk headers
unsupported by `apply_patch`; only those headers were normalized in a private
copy, and the resulting restoration matched all 60 hashes. The final delta is
against that restoration, not HEAD.

No applicable worktree `AGENTS.md` was found. The read-only application clone's
`AGENTS.md` was read; no application files were changed. No commits, PRs,
branches, subagents, parent tools or other agents' artifacts were created or
modified. The existing assigned branch remains unchanged.

## Scope and fidelity

Application pin: `c82ab865bb362bc4af067d332adc44978ff64813`. The clean clone is
`/private/tmp/verus-transfer-redb-miuzm2w3/redb`. Each runner checks the original
source and extracted item hashes. Import-provider visibility changes remain
the only application declaration adaptation; executable bodies remain original.

The original unaided codec fixture remains byte-identical at SHA-256
`639d1032cc0fb061cf3e4bf7c8b05b14b13e7d856ce047b09e1f740d6565a3cb`.
Its domain remains every `region <= 0xfffff`, `page_order <= 20`, and
`page_index <= 0xfffff >> page_order`. Its three authored roundtrip goals are
unchanged. Codec injectivity and cross-crate callers also have no handwritten
packing or byte-inverse proof blocks.

The page-size positive fixtures remove `BOUND`, its invocation, and the
order-63 constant-shift ghost assertion. They keep the original executable
`page_size_bytes` body, all `u32` page sizes, arbitrary region/index fields, and
the original order-at-most-20 equality and upper-bound goals. The full safe
domain caller and valid order-63 sizes zero and one are also retained.

## Reusable implementation and trust boundary

The ordinary-caller demand pass lazily expands existing exact inferred
relations through transparent constructors, fields and constant in-bounds
literal-array projections. It supports scalar shifts/casts in requested
assertions, inferred-callee requirements and return obligations. Canonical
readonly equality of finite scalar arrays can supply a proposed branch
condition. That proposed equality is **not** installed as a call summary or an
assumption; it is used only to construct a mathematical conditional theorem.

The existing `AssertBitVector` checker independently checks each complete
theorem with **empty requires**, then the ordinary query receives the identical
`Arc`-shared fact. Caller/domain and finite-type conditions are inside the
theorem as premises. Original calls, authored goals, executable safety checks,
all three exactness checks, import validation and publication gating remain.
Unsupported expressions and exhausted budgets install no facts.

Byte reconstruction and packing cancellation are checked directly from the
published byte-array relations. No new byte axiom, trusted application summary,
application identity, vstd contract or byte-inverse library law is introduced.
The byte-inverse ghost call disappears from accepted callers without extending
the library trust inventory. Standard array equality already occurs in the
original injectivity caller; its contract is not replaced by a trusted model.

The page-size result uses the same bounded BV/cast/product expansion, not a new
nonlinear proof mechanism. The demand pass caps accepted requests at eight,
expression expansion at 8,192 nodes, depth at 64, statement visits at 8,192,
and shared conversion/expansion work at 131,072 units per ordinary function.
Rejected attempts consume the shared work budget. Existing independent packing
assistance is unchanged; these two applications do not add packing-shape queries.

Cache version is **65**, import policy **59**, schema **38**. The parent must
reconcile its combined versions **67/61**; this implementation does not change
them on the parent's behalf.

## Formal observations

All numbers are the verifier's `verified/errors` counters, not a claim that
every count is a separate application theorem. Every relevant inferred helper
passes fresh soundness, safe-domain maximality and output exactness checks.

| Fixture | Frozen baseline | New cold / warm |
| --- | --- | --- |
| Byte-identical unaided codec | 66/1, reproduced | 71/0, 71/0 |
| Historical manually assisted codec | 68/0, reproduced | 68/0, 68/0 |
| Codec plus unaided injectivity | Historical assisted 71/0 | 75/0, 75/0 |
| Public codec provider, no caller ghost blocks | Historical assisted provider | 75/0, 75/0 |
| Imported codec client, no caller ghost blocks | Historical assisted 8/0 | 12/0, 12/0 |
| Unaided page-size complete callers | Initial new-pass probe 12/1 | 17/0, 17/0 |
| Historical assisted page-size reference | 12/0 | 12/0 |
| Unaided page-size provider | Historical assisted provider | 17/0 |
| Unaided imported page-size client | Historical assisted client | 17/0, 17/0 |

The final front door independently accepts both `redb` and `redb_page_size`,
including 27 declared positive/semantic/safety proof cases. The nested runners
retain 34 verifier invocations in total, including metadata/frontend controls
and the separately classified auxiliary-theorem rejection. Measured final cold
and warm codec invocations take about 1.02/1.04 seconds, codec with injectivity
1.22/1.19 seconds, and page-size 0.78/0.75 seconds. These are one replay's wall
times, not a controlled performance comparison.

## Negative controls remain honest

All three historical codec negative fixtures and all five historical page-size
negative fixtures remain byte-identical to their restored-baseline versions.
They run without optional assistance and retain their original checked ghost
proofs. The faithful decoder mutation still passes every required exactness
check but fails the **authored semantic postcondition**, at 67/1, without a
precondition failure. The false codec goal is also 67/1. Both remove seeded
previously verified interfaces. Malformed syntax/types are not semantic evidence.

An additional unaided decoder mutant is rejected at 70/1 because a proposed
auxiliary theorem is false; its two helpers still pass every required exactness
check and its seeded valid interface is removed. It is explicitly **not counted
as authored semantic-goal failure**: the mandatory auxiliary check stops before
the ordinary query may assume its theorem. No unchecked fallback or panic
suppression is added to manufacture a preferred diagnostic.

Page-size shifts 64 and 255 with size zero, every order at least 64 with size
zero, and order 63 with size two still fail their actual caller preconditions.
Order 63 with sizes zero and one prove in the unaided positive fixture. The
historical false page-size postcondition fails at 11/1. Failed controls remove
seeded prior verified interfaces, while positive interfaces survive.

The existing poisoned-byte and poisoned-conversion candidate-cache tests pass,
as do all four poisoned import metadata controls (policy, proof status, summary
fingerprint, trusted-model fingerprint), imported composition and stale-export
tests. Warm readonly record candidates are regenerated, not claimed as cache
hits. Scalar conversion candidates genuinely exercise cold/warm cache reuse.

## Validation and independent artifacts

- `vir --lib`: **128 tests passed**.
- Ten selected Rust integration binaries: **83 tests passed**, including seven
  new general positive/negative tests with a different codec layout and a
  different shift/product bound; scope, stale substitution, work/depth limits,
  eager invalid shifts, overflow, imports and poisoned caches remain covered.
- Python normal suites: **119 inference + 9 redb + 10 Fedimint = 138 passed**.
- Optimized Python transfer suite: **11 passed**, an additional execution of
  existing transfer tests rather than 11 new distinct tests.
- Fresh vstd source verification: **2221 verified, zero errors**, not 2212.
- Changed Rust formatting and `git diff --check` pass.

Builds use toolchain `1.99.1`, offline/locked/release mode, two jobs,
`VERUS_IN_VARGO=1`, `RUST_MIN_STACK=20971520`, and `VERUS_Z3_PATH=source/z3`.
Both Cargo target/build-directory variables name the independent directory
`/private/tmp/verus-infer-proof-implementation/target`. Tools are independent
copies in `/private/tmp/verus-infer-proof-implementation/tools`; the modified
verifier and vstd are rebuilt from source. A test-harness compatibility symlink
within that same private directory points only to this independent bundle.

The final suite receipt is
`/private/tmp/verus-infer-proof-implementation/final-replays/transfer-suite-sq6epsbg/receipt.json`.
`manifest.json` records exact changed-source and tool hashes, baseline hashes,
fixture hashes, raw receipt hashes, diagnostics, proof statuses, validation logs
and test counts. The raw suite and nested directories retain full commands,
stdout/stderr, compiler/tool inputs, generated fixtures and interface artifacts.
`/private/tmp/verus-infer-proof-implementation/handoff.json` inventories those
artifacts and the independently replay-checked baseline-relative `apply_patch`
delta with its SHA-256. Historical reports and receipts are not rewritten.

## Limits

Trust delta: **zero new application axioms, unchecked hints, trusted application
summaries or vstd contracts**. The normal verifier/SMT backend, existing standard
models and existing library contracts remain trusted. Synthesis and substitution
are not claimed formally verified; the checked identical-fact boundary is the
acceptance mechanism.

This is **not formal verification of the Rust standard library**, nor a proof
of redb's database, transactions, address-range logic or overall safety. Native
codec support covers 2,097,467 finite cases, and page-size support covers
1,548,477 canonical cases plus 192 eager-shift panics and one overflow panic;
these are finite supporting oracles, not substitutes for the quantified goals.
No separate nonlinear mechanism or unresolved blocker is needed for this
bounded follow-on. False or unsupported auxiliary proposals continue to fail
closed and are not relabeled as authored semantic counterexamples.
