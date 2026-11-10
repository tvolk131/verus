# Result: observer necessity is not shared across the frozen proofs

The experiment does **not** confirm necessary shared observer proof work for
the full frozen canonical encoder and the Zip prefix diagnostic. Removing the
entire observer leaves canonical verified. The observer is necessary as a group
for the prefix diagnostic, but its single iterator-sequence capacity proposal is
redundant even there. The preregistered necessity hypotheses were contradicted;
the intervention, fixtures and hypotheses were not changed to force a reduction.

All entries below are reproduced results, not assumed baseline successes.
Counts are Verus `verified/errors`; each main entry agrees cold and warm.

| Frozen source, goals enabled | Delivered baseline | Observer off | Only sequence capacity off |
| --- | ---: | ---: | ---: |
| Full canonical `candidate-008.rs` | 6/0 | 6/0 | 6/0 |
| Zip prefix `prefix-zip.rs` | 2/0 | 1/1 | 2/0 |
| Full Zip writer `full-zip.rs` | 1/1 | 1/1 | 1/1 |

With goals disabled, **every runtime** gives canonical 2/2, prefix 1/1 and full
writer 1/1, cold and warm. Per-runtime ordinary controls also agree: plain writer
2/0, false XOR-to-OR postcondition 1/1, and arithmetic overflow 1/1. The latter
two retain strict postcondition-only and safety-only diagnostic gates.

## What the comparison establishes

- Canonical has exact executable inventory `{u64}`, proof bodies `{}` and
  inferred helpers `{}`. Numeric assistance is applied in every enabled run,
  including observer-off. Observer-off has no observer entries and still passes
  full selected scope, with ordinary body/WF/mode/lifetime/trait checks and
  `--no-cheating`. Consequently the observer's proposed snapshots/invariants
  are **not necessary** for this frozen canonical proof under the retained
  production components and imported specifications. Checked proposal insertion
  and the matching 6/0 result cannot establish necessary shared reuse.
- Prefix has exact executable inventory `{xor_key}`, proof bodies `{}` and
  inferred helpers `{}`. Its observer-off failure is an ordinary postcondition
  failure, not a frontend, mode, resource or panic failure. Baseline proves the
  same source over the same arbitrary initialized buffer and arbitrary key.
  This is causal evidence for the observer **as a group** on this prefix proof.
- Capacity-off removes exactly `len(it.seq()) == 10` for plain canonical or
  `len(it.seq()) == min(128, key.len())` for Zip, retaining both snapshots,
  their length invariants, and element correspondence. Canonical and prefix
  still prove. Thus no single indispensable proposal has been identified.
  The retained observation group suffices for prefix; this experiment does not
  assign necessity to an individual snapshot, length fact or element equation.
  No further ablations or fixture changes were attempted.
- The full writer still fails its unchanged `pad_xor` postcondition, including
  the required unchanged suffix. Prefix success is not full transfer success.

“Necessary” here is relative to this controlled engine/fixture configuration,
not a claim that a theorem has no other proof. The canonical result is direct
measured redundancy, not an inference from parser or catalog sharing.

## Evidence and exact checks

[results.json](results.json) contains the compact audit of **45 accepted trials**:
three runtimes, three frozen main sources, goals enabled/disabled, cold/warm,
plus three controls per runtime. It binds source, controller, patch, executable,
macros, builtin/vstd, solver, stdout/stderr, export artifacts and raw receipt
hashes; complete command arrays; named body inventories and scope statuses;
exact diagnostic headers/counts and source assertion snippets; and preserved
setup/validator rejection receipts. [freeze.json](freeze.json) preserves the
original intervention hashes, source identities and hypotheses.

Every application invocation, including six retained baseline invocations
excluded after controller validation rejections, uses `--no-cheating`. Successes
use shared `validate_goal_run`. Causal multi-obligation diagnostics use shared
`validate_run`, exact helper checks, complete native collection, explicit body
inventories, recognized ordinary error headers and frontend/resource/panic
exclusion. Recommendation notes accompanying real SMT failures are recorded,
not accepted as standalone proof failures. Canonical's compiler-generated
`VERUS_UNERASED_PROXY__U64_LEN` constant proxy is inventoried explicitly; it is
not an authored executable/proof body or an inferred helper.

Counts of verification queries differ from diagnostic-line counts. In each
canonical-disabled run, Verus reports 2/2 and emits thirteen ordinary diagnostic
headers: seven loop-invariant, three invariant, one precondition, one overflow
and one postcondition failure. These are causal/disabled diagnostics, not
strict single-kind semantic negatives. Complete snippets and locations are in
the compact manifest and hash-bound raw stderr. Prefix/full failures emit one
postcondition error each. No frontend/resource/panic/mode failure enters the
accepted matrix, and all mandatory/preparation statuses are healthy with zero
requested/generated inferred helpers.

Both private runtimes were built normally in this worktree with
`CARGO_BUILD_JOBS=2`, fresh matching compiler/macros and normally verified vstd
**2,226/0**. No application `--no-cheating` flag was forwarded into vstd.
Each build records all **2,091** tracked source/tool/dependency inputs. Exactly
one production file differs from d35c31fa: the frozen observer patch. Its complete
altered-source identity is recorded; all other inputs match base. The capacity
patch adjusts the independent whole-function projection's exact invariant count
from four to three while retaining its structural, freshness and purity checks.
The production JSON report hardcodes four proposals, so its capacity-off count
is explicitly not used as insertion/proof evidence.

Actual private version labels are `0.2026.11.10.e8a195c.dirty` (observer-off) and
`0.2026.11.10.136017a.dirty` (capacity-off): research HEAD metadata and temporary
private source alteration, not production version changes. Executable/macros/
vstd hashes are bound to each actual build. All solvers are independent copies
of the guarded delivered Z3 4.16.0 artifact. The delivered baseline is checked
against its copy manifest and expected parent version before use and guarded
throughout. `source/z3` was absent in the prepared checkout; the exact delivered
bundle solver was used without modifying a shared path or downloading one.

Each immutable runtime copy was completed before restoring the sole altered
production file byte-for-byte to d35c31fa. The entire source inventory agrees
before and after restoration. Only this new research directory is committed;
no production alteration, global version change, new assumed fact, local proof
glue, input restriction or semantic/postcondition change to a frozen fixture
is delivered.

## Retained validation corrections

The first normal compiler/vstd run passed, but an old-spelling vstd count gate
rejected its output. That setup receipt is retained, source restoration passed,
and a corrected normal build/vstd run produced the accepted observer-off copy.
Before any ablated application trial, two baseline receipts were also excluded:
first for an overly strict recommendation-note gate and exact constant-proxy
scope handling, then for the documented legacy no-flag ordinary export boundary.
All three rejected receipts/logs remain under
`/tmp/shared-observation-ablation-78d09abf/`, with paths and hashes in the manifest.
They are not causal evidence. Initial freeze `f519a410`, pretrial corrections
through `4e3094b7`, and validator corrections `06ed97df`/`136017af` remain in commit
history; the sources, intervention hashes and hypotheses are unchanged.

The command matrix retains `--export` throughout. Goals-enabled runs enforce
inference-aware success/failure publication gates. Goals-disabled failed runs
may leave legacy ordinary `.vir` files while `interface_written=false`, as
TARGET.md warns. These outputs are recorded/hashed, never imported and never
accepted as proof certificates. The overall goal's publication status is false.
Seeded valid-export removal, imports and cache/metadata poison regressions are
left to the parent's existing matching replay scope, not claimed from this
fresh-output ablation matrix. No timing/cache speed benefit is claimed: helper
inventories and helper cache contents are explicitly empty.

## Next minimal interface question and limits

The native slice iterator specification already exposes remaining length and
current/final element relationships (`source/vstd/std_specs/slice.rs:515`), and
the wrapper stores its iterator snapshot and preserves `seq()` across `next`
(`source/vstd/std_specs/iter.rs:962`). These existing contracts are a plausible
source of canonical redundancy; this experiment does not identify a minimal
solver dependency or prove a new native correspondence.

The next question is which actually required native observation/ownership fact
could be supplied by a common checked refinement interface to **both unchanged
full properties**, rather than restated redundantly in a decorator. Trace the
existing canonical fact provenance before proposing such an interface. This
record does not nominate an indispensable common fact or justify disabling
standard checks/contracts or modifying fixtures to manufacture one. The full
writer's unused-tail frame obligation remains separate and required.

**Full HMAC suffix: required and unproved. Native correspondences: 0. All 101
native effect obligations: open. Goal complete: false. Publication: false.**
These are byte-writer proofs/diagnostics, not HMAC/security proofs. Fresh parent
matching regression acceptance and independent review remain required.
