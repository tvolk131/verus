# Native redb reference proof88: approved bounded continuation

Authorized November 4, 2026. The user answered “Cool, set it and rip!” to the
explicit request for a narrowly scoped trusted `u32::div_ceil` std-library
correspondence and a goal with one reference-proof/review cycle, followed only
on success by at most one justified Infer automation/review cycle.

The existing bitmap goal remains unfinished. Its tracker cannot be overwritten
by a second goal; this document records the approved continuation, not a claim
that historical NO-GOs, the strict83 twenty rows, or consolidation87 proved resize.

## Frozen baseline and trust exception

Start from the independently reviewed consolidation87 candidate: 2,235 source
paths, identity `a1e169d8bd9535fdeb54ddde8fbb0353096e7e7549a37c43e1082318315f2126`.
The complete baseline and compiler maps, own Git state, and campaign evidence
are retained under `/private/tmp/verus-native-reference88.w3Ws2g/`.
Preserve baseline bytes except explicitly inventoried changes necessary for
the authorized bridge, proof workflow, controls and compatibility.

Native redb remains clean at `c82ab865bb362bc4af067d332adc44978ff64813`, with
bitmap SHA256 `d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff`.
Fedimint remains clean at `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.
Retain all original 1,401 immutable hashes and both original native source maps.

The **only newly authorized trust** is the exact Rust core `u32::div_ceil` method:
all `u32` numerators and nonzero `u32` divisors, exact mathematical ceiling, and
the divisor-zero safety requirement. This is a trusted std correspondence, not
a proof of core's implementation. Audit the resolved method/type identity,
extreme values, overflow-sensitive cases, zero-divisor rejection, and unrelated
method/type negatives. Do not extend the numeric macro to all integer types,
add an application wrapper axiom, assume a bitmap theorem, or silently authorize
another method. Existing vstd trust stays disclosed and distinct from the new
exception. A proved arithmetic lemma is not evidence of native correspondence.

## Required reference proof

Check the unchanged native struct and all nine bodies: `bits_in_range`,
`required_words`, `data_index_of`, `select_mask`, `len`, `resize`, `get`, `set`,
and `clear`. Erasure/source audit must preserve native executable tokens,
helper calls, assertions, inclusive for-loop, mutation and branches; no algorithm
replacement or handwritten equivalent helper receives native-proof credit.

Retain every `u32` old/new logical length, every stored `u64` value, and arbitrary
valid storage with `old_len <= 64 * old_word_count`. No minimal-storage, zero-
padding, one-word, bounded-length, convenient ordering, or artificial nonoverflow
assumptions. Retain actual safe indexing/shift premises for access operations.
On the pinned target, prove:

- Result length exactly `new_len`, with representation validity preserved.
- Result word count `max(old_word_count, ceil(new_len / 64))`.
- Growth changes exactly exposed logical bits to `full`.
- All unaffected old stored bits, including padding/spare storage, survive.
- Newly allocated words and their padding have native zero/all-one defaults.
- Shrink neither truncates nor clears stored words.
- Actual native reads and updates compose, including shrink then regrow.

Explicit semantic contracts, ghost snapshots, checked mask/arithmetic lemmas,
loop invariants and proof blocks are allowed and must be inventoried as author
work. No `assume`, `admit`, app axioms, unchecked summaries or proof-erasing
semantic restrictions beyond the explicitly approved std correspondence.
Existing inference assistance can be used but is not new capability credit.

## Completion matrix

| ID | Required result |
| --- | --- |
| B1 | Correct exact u32 std bridge, safety/identity/extreme controls, trust inventory |
| P1 | All nine source-faithful native bodies backend checked, no omitted failed scope |
| P2 | Full domains and complete stored-bit/length/allocation/padding/frame theorem |
| P3 | Native get/set/clear and shrink-regrow clients; genuine fresh import where supported |
| N1 | Intended semantic, safety, frame, premise, and false-goal controls reject |
| A1 | Explicit proof burden quantified; no new automation claim without ablation |
| R1 | Source-bound matched runtime, relevant regressions, cache/publication controls |
| R2 | New independent review, repaired findings, reconstruction and preservation |

Ordinary imports retain their ordinary-contract boundary. A certified inference
claim requires a genuine supported inference export and import, not a dummy
helper or a receipt label; it does not silently certify all ordinary contracts.
Successful query totals alone do not establish complete executable coverage.
Reject unrelated/frontend/infrastructure failures masquerading as semantic controls.

## Hard budgets and stop rule

Reference cycle: at most **96 direct application verifier launches**, including
setup failures, divided initially into 64 implementation, 16 parent/final and
16 independent-review launches; at most **two compiler builds and two matched-
vstd runs**. Test-suite invocations are separate, never native theorem credit.
Use private offline caches/targets/homes and canonical local toolchain selection.

One reference implementation/review cycle, including bounded fixes of review
findings. Stop with an exact blocker if the complete checked reference is not
established; do not extend automatically or count component/harness work as
completion. No optional automation cycle after a failed reference.

Only after a successful independently reviewed full reference, freeze one clear
repetitive mechanical step. At most one optional automation/review cycle, with
80 additional direct application launches, two builds and two matched-vstd runs,
is authorized. Require a discriminating before/after ablation, one structurally
different generic example, unchanged safety/semantic negatives and new review.
No forced feature if a full explicit proof reveals no justified automation.

No network, native edits, commits, pushes, manual branches/refs, shared Cargo or
toolchain writes, third implementation cycle, or expansion into Fedimint,
allocator correctness, storage, concurrency, recovery, or durability. Preserve
sealed history and report shared-ref observations without actor attribution.

All eight matrix rows start OPEN. Freeze source/runtime/command ledgers and
author burden before claiming any completed row. Finish with a sealed reviewed
handoff and honest full-reference/automation/overall-goal status separately.
