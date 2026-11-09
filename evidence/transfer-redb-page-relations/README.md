# redb: source-connected page relations

November 1, 2026. Application pin
`c82ab865bb362bc4af067d332adc44978ff64813`; read-only clone
`/private/tmp/verus-transfer-redb-miuzm2w3/redb`. This is a new report; historical
reports and parent artifacts are preserved.

## Baseline and source fidelity

The complete supplied dirty baseline was restored with `apply_patch` before
editing. All **87 source/evidence hashes** and all **nine immutable runtime
hashes** matched. Baseline manifest SHA-256:
`1a52b38c47530e13bf056c0be1c4b0af5635e8236469ec4767f5c248ecee7894`.
HEAD and origin/verus-infer remain
`55bd67ad980e90e92c77efe4453a927f097c3386`. The differently positioned local
verus-infer ref was not substituted or moved. No commits, PRs, manual branches,
subagents, parent-checkout edits, or application edits were made.

No applicable Verus-worktree AGENTS.md exists. The redb clone's instructions
were read; redb's `just test` is not an acceptance claim here because no redb
code was changed. All commands below operate on extracted verification inputs.

`PageNumber` retains its original u32/u32/u8 fields. Both original executable
method bodies are byte-preserved, including `debug_assert!`, both calls to
page_size_bytes, and `start..end: Range<u64>`. Extraction starts at the struct
item after its derive attribute, as in the established runner; the remaining
PageNumber implementations are not part of this slice. The provider changes
`pub(crate)` declarations to `pub` for cross-crate visibility only. A caller
may return a pair of ranges; the original helper still returns Range<u64>.
The runner checks the full base.rs hash and all three extracted item hashes,
and requires a clean application checkout at the exact pin before and after.

Production identity is **cache70 / policy64 / schema38**. Parent integration's
72/66 reservation and the Fedimint sibling's 71/65 reservation are untouched.
The patch is relative to the restored 87-file baseline, not clean HEAD or the
parent's changing worktree. Shared controller, guide and parent CI/preparation
changes are excluded; registration and guide proposals are separate scratch
artifacts.

## Authored relational properties

For a common region, page_order, page_size, data_section_offset, region_size and
region_pages_start, with positive u32 page_size:

- increasing page_index gives a strictly increasing range.start;
- distinct page_index gives distinct starting addresses;
- in the increasing-index case, first.end <= second.start.

These are authored ordinary caller goals. They are not an inferred allocator
invariant. Cold, warm and imported relay callers contain no handwritten packing,
bit-vector or nonlinear-arithmetic proof blocks.

Each page explicitly carries the full checked-arithmetic domain. With B =
`(1u64 << order) * size` interpreted mathematically: order < 64, B <= u64::MAX,
index*B <= u64::MAX, regional = pages_start+index*B <= u64::MAX, regional <
region_size, region*region_size <= u64::MAX, offset+region*region_size <=
u64::MAX, start <= u64::MAX, and start+B <= u64::MAX. All inputs keep their
original machine types. There is **no order<=20 premise**. Positivity is needed
for strict/injective starts, not for endpoint nonoverlap: a fourth generic
caller proves ordered nonoverlap for all u32 sizes, including zero.

The original method proves the zero-size boundary: both ranges are empty and
have equal starts, while first.end <= second.start still holds. Separate false
strict-start and injective-start postconditions fail semantically. Zero size is
not presented as an overlap witness.

The original method also proves the specified mixed-order witness:
order1/index0 gives **0..8192**, order0/index1 gives **4096..8192**, for region0,
page_size4096, region_size8192, pages_start0 and offset0. A deliberately false
mixed-order endpoint-nonoverlap postcondition fails semantically. Both helper
summaries remain exact; every seeded stale export is removed. Distinct page
numbers alone do not establish allocator disjointness.

## Reproduced blocker and reusable implementation

The frozen baseline69/63 ordinary solver fails all three generic positive
relation goals after all exact helper checks succeed. These are proof blockers,
not semantic counterexamples. The retained source-connected strict.rs probe has
only the original two helpers, full domains and one relational caller. Existing
checked BV assistance on strict/nonoverlap exhausts its resource limit; those
runs are diagnostic failures and are not counted as semantic evidence. Selecting
NIA alone proves nonoverlap but does not establish strict-start positivity.

The new bounded strategy recognizes degree-three/four mathematical polynomials
through the existing exact-summary expansion. It admits finite unsigned inputs,
small constants, casts, comparisons, Boolean connectives, +, -, *, and unsigned
one-hot shifts of a finite variable or literal. Unsupported bitwise expressions,
branches, calls, excessive degrees, architecture-dependent inputs, exhausted
budgets and excess shift occurrences add no polynomial fact. Existing
degree-two packing/page-size queries retain their BV strategy.

For each requested polynomial it proposes at most four conditional shift-
positivity facts, independently checked by the existing BV backend with empty
requires. The complete conditional arithmetic theorem, including caller/type
premises and those conditional facts, is independently checked by the existing
NIA backend in an assertion-only isolated query. The exact same Arc-shared
polynomial fact is used in the ordinary query after its check. Substitution
need not be trusted: the original calls, safety checks and authored goals must
still prove. Nothing rewrites a real assertion or installs an unchecked helper
summary. All three mandatory inference checks and publication/import gates
remain unchanged.

The existing per-function limits remain 8 requests, 8192 expansion nodes,
depth64, 8192 statement visits and 131072 shared work units. The new selector,
fragment checks and rejected attempts consume that shared budget. Accepted
premises have degree at most four. Each polynomial request adds at most four
BV queries and one NIA query. These are structural/work bounds, not a promise
against solver resource exhaustion.

A false polynomial is rejected by its **auxiliary theorem** after both helpers'
exact checks pass; it is not counted as an authored semantic counterexample.
The compiler's existing coarse NIA envelope is caller_proof_failed. The dedicated
runner distinguishes its explicit `polynomial proposal not satisfied` diagnostic
from authored pre/postcondition failures; no wire/schema change is needed.

## Fresh accepted evidence

The final matching runtime is independently frozen under
`/private/tmp/verus-redb-page-relations-dbt1g78j/sealed-tools`. Modified compiler
sources were built offline/locked/release from this worktree with Rust1.99.1,
two Cargo jobs, a 20-MiB Rust stack and both Cargo directory overrides pointing
to an independent /private/tmp target. The source/.cargo settings are retained.
Unchanged builtin/macro runtime artifacts are independent immutable copies with
explicit baseline hashes; no claim is made that those macro artifacts were
rebuilt. Fresh matching vstd verification is **2221 verified, 0 errors**.
Cargo source ledgers, artifact fingerprints, exact commands, environment and
log hashes are retained in the manifest's referenced scratch receipts.

Dedicated final replay:

- relations cold/warm/poisoned-cache: **24/0** each;
- generic zero-size boundary without assistance: **7/0**;
- concrete zero/mixed witnesses: **22/0**;
- provider: **6/0**; imported inferred-relay callers: **21/0** cold/warm;
- **8 positive**, **3 semantic**, **11 safety**, **1 arithmetic auxiliary** and
  **4 metadata** controls; **19 seeded negative exports removed**;
- **132 repeated required inference checks** prove. They are not 132 distinct
  application theorems. Positive, semantic, safety and auxiliary proof cases use
  whole-crate no-cheating scope. Metadata controls reject before verification.

Warm readonly/owned summaries report cache=miss and
cache_policy=regenerated_state. They regenerate and reprove. The poison-file run
also regenerates; it is not a claim of cache reuse or of a cached arithmetic
proposal path. Imported policy, exactness, summary fingerprint and trusted-model
poisoning fail closed before publication.

The unchanged common controller independently validates the **22 main controls**
through an explicit in-memory registration; its shared source is untouched. The
additional arithmetic auxiliary and metadata controls are checked separately by
the dedicated runner and the scratch acceptance checker. The existing four
frontdoor scenarios all report proved with the same final runtime. Final validation passes **138 VIR unit tests**, **576 Rust integration tests
across all 28 root infer binaries** (none ignored), and **149 Python executions
each normally and under -O**. Cargo formatting and git diff checks pass. Exact
commands and source-guard results are enumerated in manifest.json. VIR testing uses `cargo test -p vir --lib`,
not the 18 documented unrelated failing doctests. Preliminary failed setup and
exploratory attempts remain preserved and are not added to accepted totals.

## Nearest caller and remaining blocker

The actual access method is TransactionalMemory::get_page at page_manager.rs:
1922. The source-preserved range/length statement prefix, projected through its
three readonly configuration fields with an appended return of len, proves on
both baseline and final tools (**7/0**), given the explicit full arithmetic
domain and byte_count <= usize::MAX for try_into().unwrap(). This is a partial
source adaptation, not verification of the entire method or an opaque type
correspondence with TransactionalMemory.

Inferring that unchanged prefix rejects the original `self.page_size.into()`:
`Verus Infer unsupported: only static, nongeneric inferred calls are supported`.
The original Into/TryInto syntax was not rewritten to manufacture success.
This frontend/profile blocker is separate from the positive ordinary-prefix
composition. The remaining method calls storage.read and tracks references
through mutexes/maps. allocate_helper also mutates buddy-allocator occupancy.
A real mixed-order disjointness theorem needs that occupancy invariant; the
witness demonstrates why a distinct-PageNumber premise is insufficient.

## Trust and replay

Trust remains the ordinary Rust/Verus/SMT pipeline, existing vstd conversion and
requires-false panic specifications, and the existing canonical transparent
core Range<u64>/ExRange external correspondence. Rust std/libcore bodies are
not proved here. No opaque external application type correspondence, application
axiom, unchecked summary, arithmetic assumption, skipped panic check, or new
vstd contract was added. This is not whole-allocator/database, concurrency,
crash-recovery or Rust-library correctness.

```sh
python3 -B source/tools/check_redb_page_relations.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-redb-page-relations-dbt1g78j/sealed-tools/verus \
  --solver /private/tmp/verus-redb-page-relations-dbt1g78j/sealed-tools/z3 \
  --output /private/tmp/fresh-redb-page-relations
```

The scratch handoff contains the baseline manifest, baseline restoration,
baseline-relative apply_patch delta and its SHA-256, changed-file hashes,
all nine runtime hashes, exact accepted receipts, source/command/log/Cargo
fingerprints, source adaptations and the next blocker. Its replay reconstructs
HEAD, restores and verifies all 87 baseline files, applies only this delta and
checks the entire resulting source tree against this implementation. Parent
integration and independent review remain the parent's responsibility.
