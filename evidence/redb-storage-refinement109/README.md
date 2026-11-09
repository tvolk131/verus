# Native storage-refinement109 — approved implementation round

The user approved the bounded [plan](PLAN.md): 60 minutes, 4 GiB scratch,
parallel implementation/review, no compiler/runtime builds, no new trusted
assumptions adopted, no source integration and no weakened admission gates.
The separate compiler/Arc diagnostic is not approved by this decision.

Two proof lanes target buffered-storage transitions and public two-phase
acknowledgement/error ordering. A third independent lane supplies finite
source/outcome controls. The actual handed-off candidates then receive one
independent review within the same round, without an automatic audit chain.

[Authorization](AUTHORIZATION.json) and [selected-input hashes](INPUTS.json)
bind the scope and baseline. Existing native recovery, source-path/Git and
frozen compiler/source qualifications remain. Checked reference fragments
must not be reported as verified original Rust bodies or native producers.

All eighteen final cells remain **OPEN**; candidates and controls are not
acceptance. Global free space stays above 16 GiB. Excluded tasks remain excluded.

## Independent finite controls retained

The [controls report](INDEPENDENT-CONTROLS-REPORT.md) and
[branch/boundary coverage](INDEPENDENT-CONTROLS-COVERAGE.md) accompany actual
Python implementations, 21 passing tests, 844 storage parameterizations,
195 commit/opaque parameterizations and 12,644 raw-image crash observations.
Six semantic mutant/corrected pairs and seven source mutant/corrected pairs
distinguish intended mismatches from refusal, lexical failures and crashes.
These are finite controls, not native executions, recovery results or proofs.

All 133 payload hashes and all 135 archive members authenticate byte-exact;
all 369 current frozen inputs still match. The five historical document drifts
remain disclosed. [Retention results](INDEPENDENT-CONTROLS-RESULTS.json) bind the
archive/report and [golden outcomes](INDEPENDENT-CONTROLS-EXPECTED.json).
`INDEPENDENT-CONTROLS-HANDOFF.tar.gz` retains the actual source, raw attempts
and complete finite inventories, not just this summary.

Important compatibility limits: native write admission can evict/issue writes
before explicit flush; pure pending-enqueue coalescing is not that native
producer. Opaque callbacks are not assumed inert. The real post-publication
continuation includes fallible epilogue stages, so Err must not imply rollback.
Missing resource/loan/glue, backend, panic/abort and history bindings stay OPEN.
The subsequent sections retain both candidates and the completed combined review.

## Acknowledgement CFG candidate retained

The [candidate report](ACK-CANDIDATE-REPORT.md),
[native-call coverage](ACK-CANDIDATE-COVERAGE.md) and
[composition obligations](ACK-CANDIDATE-ADAPTER.md) describe a checked authored
CFG/reference program, not verification of original Rust executable bodies.
Its symbolic theorem generates the event order from its step function: checked
public Ok follows first header write/prepare barrier, second write/final barrier
and manager publication. Checked witnesses retain post-publication Err and a
valid-but-stale early secondary. Four semantic negatives fail at their intended
postconditions; corrected counterparts restore identical positive source.

Fourteen verifier applications were charged. Final candidate scope is 15 checked
proof bodies; the public source-module consumer checks 17. **Zero original native
executable bodies are checked.** Native bytes, resources, allocator/history and
the adapter to the buffered-storage lane remain OPEN. All 111 sealed payloads
and the 113 parent-created archive members authenticate byte-exact; all 369
current frozen inputs still match. [Retention results](ACK-CANDIDATE-RESULTS.json)
bind `ACK-CANDIDATE-HANDOFF.tar.gz` and the exact proposed source patch.

**Parent publication correction:** the unchanged report/handoff incorrectly say
no `.vir` was emitted. Two binary `.vir` files are actually sealed at the scratch
root; the runner scanned only per-attempt directories although `--export` named
root destinations. [Correction and observations](ACK-PUBLICATION-CORRECTION.json)
preserve the raw facts without rewriting original evidence. The four negative
invocations did not request exports, so they do not test requested-export refusal.
No `.rlib`/binary import or native acceptance is established. The reserved review
must observe fresh explicit positive/negative/restored export destinations.

## Buffered-storage candidate retained

The [candidate report](BUFFERED-CANDIDATE-REPORT.md),
[source correspondence](BUFFERED-CANDIDATE-CORRESPONDENCE.md) and
[semantic controls](BUFFERED-CANDIDATE-CONTROLS.md) describe an authored
ghost/specification library: final positive and byte-identical restoration check
31 obligations with zero errors. Zero original native executable bodies are
checked. The symbolic results cover arbitrary finite pending stripes and write
decisions, same-offset coalescing, successful-prefix errors with retained failing
stripes, drain-before-sync ordering and modeled partial effects without rollback.
Three semantic mutations are rejected. Thirteen attempts are charged, including
one mutation-setup failure; twelve actual verifier starts are retained.

All 93 payloads and 96 archived members authenticate byte-exact; all 369 current
frozen inputs still match. [Retention results](BUFFERED-CANDIDATE-RESULTS.json)
bind `BUFFERED-CANDIDATE-HANDOFF.tar.gz` and the exact source patch. Backend laws
are explicit conditional definitions, not adopted native contracts. Native cache
admission, enumeration, locks/resources, backend operations, abnormal paths and
composition with the acknowledgement CFG remain OPEN. Both frozen candidates and
the finite controls received the single reserved independent review below.

## Combined independent review completed

The unchanged [independent report](INDEPENDENT-REVIEW.md) identifies no invalid
bounded-model theorem. Its [raw archive](INDEPENDENT-REVIEW-HANDOFF.tar.gz),
[handoff metadata](INDEPENDENT-REVIEW-METADATA.json) and
[parent authentication results](INDEPENDENT-REVIEW-RESULTS.json) retain the actual
review. All 90 review payloads and 92 archive members authenticate byte-exact.
The reviewer authenticated 783 paths at exit with zero mismatches. All 369
selected current inputs still matched before parent status reconciliation.

| Fresh full-crate check | Verified / errors | Requested export |
| --- | --- | --- |
| Buffered positive | 31 / 0 | Written |
| Buffered failed-write rollback mutant | 30 / 1 | Refused |
| Buffered byte-identical restoration | 31 / 0 | Written |
| Acknowledgement positive | 16 / 0 | Written |
| Acknowledgement early-ACK mutant | 14 / 2 | Refused |
| Acknowledgement byte-identical restoration | 16 / 0 | Written |

There are six actual review launches and six charges, with no timeout or launch
failure. Exact fresh export paths were inspected; these results close the local
fresh-replay publication gap, not the historical runner's incorrect scan.
Different positive export hashes are retained, not called deterministic binary
serialization. No `.rlib` or binary-import acceptance is established. The source
inventories are 26 buffered proof bodies / 31 obligations and 15 acknowledgement
proof bodies / 16 obligations. The historical source-module consumer has 17 proof
bodies / 18 obligations and was inspected, not freshly replayed in this allowance.
Zero original native executable bodies were checked throughout this round.

The independent Python replay passes all 21 tests and regenerates seven evidence
products byte-identically. This does not establish that the finite oracle and
symbolic model have identical transition domains: the oracle additionally admits
UNKNOWN failed-write bytes and tracks a persistent cross-call error latch. The
symbolic model's chosen old/new persistence relation does not exhaust arbitrary
historical/native crash images. Neither backend definition is adopted as native
semantics or a new trusted primitive law.

Four native-composition objections remain OPEN: actual enqueue/admission can
evict or issue fallible writes; backend/crash laws lack native correspondence;
opaque callbacks and public epilogue require effect/durable-history frames; and
the stateful cross-model/native adapter R is not proved. Its missing entry,
full-header serialization, flush expansion, error-prefix and public-return
bindings are specified in the candidate adapter, not assumed established.
Bootstrap utility identities observed after first use remain a disclosed
provenance qualification, not retroactive complete authentication. Compiler/source
correspondence, original native-path preservation and Git-continuity limitations
remain unchanged. No lifecycle pipeline, excluded task or separate compiler/Arc
diagnostic was resumed, implemented or authorized.

## Final accounting and handoff

This approved round is complete without native acceptance or source integration.
Thirteen buffered charges / twelve launches, fourteen acknowledgement charges /
fourteen launches, and six review charges / six launches total **33 charged
attempts and 32 actual verifier launches**. The original development failures,
failed mutation setup, numbering defect, repeated mutation and publication
reporting correction remain retained. Controls are finite tests, not additional
native proof credit. Resource observations are samples, not continuous quotas.
[Round results](ROUND-RESULTS.json) record elapsed time, sampled scratch/free
space, exact retained artifacts and the original four-GiB/60-minute bounds.

The shared GOAL/STATUS documents were updated only after child/reviewer exit.
Their byte-exact pre-reconciliation versions are [GOAL](HISTORICAL-GOAL.md) and
[STATUS](HISTORICAL-STATUS.json). Those two expected live-document changes are
separate from the five previously disclosed historical drifts. Per-lane retention
JSON records preserve the pending-review state at their earlier retention times;
the completed review and round results above are the current disposition.

**All eighteen cells remain OPEN.** S0, Infer, full-scope, import/export and trust
barriers are unchanged. No follow-on build, native binding/admission implementation,
new assumption, final-cell promotion or automatic review chain is authorized by
this completed round. The next useful milestone is a separately scoped checked
storage/acknowledgement simulation with explicit dependency cuts, not more copies
of the already checked lookup or finite outcome fixtures.
