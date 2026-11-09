# Campaign92 single-cycle implementation freeze

## Gate and scope

The NEW independent reference/design gate at
`/private/tmp/verus-allocation92-gate-final-3vfAI2` accepts the conditional
reference and gives conditional GO for one narrow implementation cycle. Its
seal SHA256 is
`2a9612bd18c42480757af86d7f55e7d8ea5fceaffe39dcec04d827c0187394a7`.
Parent authentication verifies all 98 review payloads and the design digest
`0c1be9feb6762cf0d15356b1b797c845bb5f573cbe5c880fbceb21edcfd2de58`.
The exact reviewed strategy is copied to the private campaign as
`DESIGN-REVIEWED.md`; that strategy and this freeze are jointly binding.

Rows S/P/F/C/N pass for the authored conditional reference. D remains PARTIAL:
starting-size, header/layout identity, constructor-to-state correspondence and
mutation invariants are not established. A is design-only, with zero automation
credit until the new feature passes a NEW final independent review. No old goal
or full campaign acceptance follows from this gate.

## Exact production change

Authorized production/test paths only:

- `source/vir/src/infer/goals/constants.rs`: one separately bounded local-shift
  decorator and its unit tests, preserving the existing closed-expression path.
- `source/vir/src/infer/goals.rs`: invoke the decorator immediately after
  `requests.stm(&check.body)` and before existing packing/xor/entry decorators.
- `source/rust_verify_test/tests/infer_computed_shifts.rs`: focused integration
  cases using the existing harness, without weakening its six original methods.

No eligibility, reporting/schema, summary, registry, application, standard
library, BV backend or existing checked-pair implementation changes. No second
tactic. Keep `bitvector_hints::checked` and its exact Arc-identity validation
unchanged. If the reviewed strategy cannot fit these paths and conditions,
stop PARTIAL rather than broadening the feature.

Recognize only a pure original unsigned fixed-width left shift
`literal_C << current_count`, with result/left width W in 8/16/32/64/128,
0 <= C < 2^W, an original unsigned fixed-width or usize count, and an already
established exact equality of that binding/current generation to k, 0 <= k < W.
No signed or architecture-sized left operand, right shift, fabricated cast,
symbolic left operand, projection/call or merely ranged count is eligible.

Compute an untrusted bounded proposal R = (C << k) mod 2^W. Keep the original
shift subtree/types in a guarded implication concluding E == R. A fixed count
uses its finite-type range and equality premises. A usize count retains USize
and produces separate arch_word_bits==32 and ==64 implications, each with the
corresponding finite-type range and exact count equality. Never use host pointer
width or relabel/cast the native count as u64. The ordinary VC still has to
establish the equality; the hint grants no equality or domain premise.

Each complete implication is one shared Exp passed to unchanged checked(fact),
which checks and uses that identical implication. All original executable,
assertion, safety, return and callee obligations remain unchanged. Failed hint
queries fail verification; no fact is assumed from a failed proof.

## Provenance, placement and resources

Use exact entry-current parameter requirements only until their first write,
retained already-checked ordinary Assert equalities, and exact equality guards
inside their own If arm. Recover lowered assertion formulas only through bounded
pure boolean aliases with dependency generations, and promote the formula only
after the original Assert. Initializing a boolean is not proving it. Never use
arbitrary Assume, a future assertion or an isolated proof query as provenance.

Track VarIdent, original type and generation. Invalidate facts/dependent aliases
on writes; clear on unknown writes/calls. Process an assignment RHS before
invalidating its destination. Branches clone bounded state without hoisting;
discard it at joins. Preserve sequential blocks without leaking block locals.
Do not descend into loops, dead ends, isolated proof/BV queries, closures,
open invariants, binders or unsupported scopes; clear at their boundaries.
Skip expression-level If/binders and unsupported operands. Insert complete
checked pairs immediately before the original first pure safety/RHS statement
that uses the eligible shift, preserving evaluation/control guards and spans.

Separate new limits: at most 8 complete pairs/body (usize costs 2), 8192 total
statement/expression visits including provenance/alias/copy/matching work,
depth 64, 32 live fact/alias/dedup entries and alias chain 8. Constants <=128
bits; shifts <=127. Exhaustion returns the original subtree without a partial
pair and never weakens verification. Existing request budgets stay unchanged.

## Exact annotation allowance and proof fixtures

The native reduced input is the already sealed `ablations/mask-equality-only.rs`
from `/private/tmp/verus-bounded-allocation92-QFQIcB`: delete ONLY
`assert((1u64 << bit) == 1u64) by(bit_vector) requires bit == 0;` from the
accepted manual provider. Retain `assert(bit == 0)`, `mask == 0`, all other
contracts, spec definitions, lemmas, triggers, snapshots, loop invariants,
proof statements and native executable bodies. No rescue annotations, domain
changes or alternative algorithm are permitted. The generic manual/reduced
pair is copied from the same sealed ablations. Preserve exact bytes in the
permanent fixtures, with their provenance manifest. Ordinary clients are copied
unchanged and import fresh new-provider artifacts.

## Frozen acceptance matrix

| ID | Candidate runtime check | Required result |
| --- | --- | --- |
| M01 | Exact reduced native ON, ordinary compile/export | All 12 executable and 15 proof bodies check; complete artifacts |
| M02 | Same reduced native OFF | Original mask safety/cut backend rejection |
| M03 | Manual native OFF | All 12 executable and 15 proof bodies check |
| M04 | Generic reduced ON | Complete checked success, no authored BV bridge |
| M05 | Generic reduced OFF | Original arithmetic/postcondition backend rejection |
| M06 | Generic authored bridge OFF | Complete checked success |
| M07 | Generic authored bridge ON | Complete checked success |
| M08 | Renamed generic reduced ON | Complete checked success without symbol-specific rules |
| M09 | Fixed unsigned widths and bounded literal specialization | Checked unsigned 8/16/32/64/128 cases, including truncated results |
| M10 | u64 shifted by usize, arch 32 | Complete checked success; original types/guards inspected |
| M11 | u64 shifted by usize, arch 64 | Complete checked success; original types/guards inspected |
| M12 | Same usize case, unspecified architecture | Both architecture models remain checked |
| M13 | Fresh ordinary original consumer | Import NEW M01 VIR/library; complete checked scope |
| M14 | Fresh extra physical-frame/hierarchy/padding consumer | Import NEW M01 artifacts; complete checked scope |
| M15 | Fresh repeated reduced-native ON | Same source and obligations succeed; no unsupported cache claim |
| M16 | Wrong minimum | Intended backend semantic rejection |
| M17 | Selected bit remains clear | Intended backend semantic rejection |
| M18 | Unrelated valid leaf bit changes | Intended backend frame rejection |
| M19 | None changes whole tree | Intended backend frame rejection |
| M20 | Spare leaf word changes | Intended backend frame rejection |
| M21 | Wrong hierarchy conclusion | Intended backend semantic rejection |
| M22 | Missing padding | Intended valid-selection backend rejection |
| M23 | False ancestor repair uses set instead of clear | Intended repair/frame backend rejection |
| M24 | Unsafe select-mask shift | Original executable safety rejection |
| M25 | Nondecreasing ancestor measure | Intended termination rejection |
| M26 | Wrong ordinary postcondition | Original backend rejection |
| M27 | Wrong Some/None on nonempty free witness | Intended backend semantic rejection |
| M28 | Wrong Some on saturated witness | Intended backend semantic rejection |

Use exact sealed controls where available and preserve their intended diagnostics.
Four of 32 direct launches are reserved for setup/corrections inside this same
cycle, not additional features. Charge every attempted launch; no borrowing.

Existing Rust harness tests and unit tests must independently cover count W,
missing/wrong equalities/results, rejected signed/architecture-left/cast shapes,
before/after Assert, false boolean initializer, stale aliases, shadowing,
reassignment, mutation-capable calls, branch-local fact leakage, entry pre-state
after mutation, unsupported proof/loop scopes and guarded dangerous shifts.
Check proposal absence where required, not merely inability to prove a false
goal. Check shared theorem identity, mismatched check/use rejection, exhaustion,
deduplication and native boolean-temp lowering. Add at most 24 focused Rust
methods, preserving existing expected outcomes.

## Budgets, regression and stop

This is the ONE authorized implementation cycle: <=32 direct application
launches, <=2 compiler builds and <=2 matched-vstd runs. For regression compilation
allow <=2 VIR test-build/test commands and <=2 Rust-harness build commands;
separately itemize at most 16 regression-suite commands and all verifier
subprocesses. Existing harness subprocesses are test-only accounting, not a way
to hide custom matrix launches. Setup failures count in their actual category.

Run applicable existing VIR units/doctests, computed shifts, old Infer,
entry-goal and publication regressions. Exercise the six pinned transfer
scenarios on the matched private runtime without changing their logical
expectations; compiler/runtime/source-map qualification must be exact and any
qualification-only drift or unrun scenario must be explicit. No logical oracle
updates, no reclassification of unrelated failures as intended negatives.
All new runtime/harness/cache/build artifacts stay private. Reuse warm artifacts
only by private copies/clones, never write their source bundle/cache.

Parent has used 6/8 discriminating launches; the new reference/design gate used
12/12. Independent final review still has its separate 20-launch cap; at most
two small acceptance repair/rechecks share 8 launches. No second feature cycle.
Stop and hand off PARTIAL if the strategy, matrix or budgets fail. No rollout or
self-approval. Caller D stays partial even if the feature succeeds.
