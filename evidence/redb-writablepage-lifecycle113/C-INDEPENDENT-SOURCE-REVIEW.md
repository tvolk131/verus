Independent source-only review of frozen lifecycle113 candidate

Verdict: CHANGES REQUIRED for the diagnostic/capture slice. The source visibly
adds a conservative pre-normalization refusal, but its eager glue query is not
a total diagnostic operation and its retained identities are incomplete. This
is a review of implemented, unbuilt source, not approval of native acceptance.
There is no positive native body/proof or executed semantic-negative result.
All18 final native redb property/history cells remain OPEN.

Review root: /private/tmp/redb-lifecycle113-review-20261108T144703Z-dXq3MG
First real clock: 2026-11-08T14:47:03Z.
Own cutoff: 2026-11-08T14:57:03Z, before parent 15:17:06Z.
No subdelegation, compiler/verifier/solver/test/native executions, builds,
network, Git writes, parent/candidate edits, integration, or goal operations.
Goal service remains last observed BLOCKED, untouched.

Scope and authentication

Read the requested AUTHORIZATION.md and PLAN.md, lane B REPORT.md, all three
actual candidate sources, candidate.patch, BASELINE.json, REPORT.md,
VALIDATION.json and the validator source (not executed). Authority reopens
only the local WritablePage/Drop correspondence edge and permits an honest
scratch refusal when engineering is missing. It does not adopt laws or permit
builds/integration. This independent review uses the frozen source itself.

Candidate root C:
/private/tmp/redb-lifecycle113-parent-origin-20261108T144030Z
Historical baseline F:
/private/tmp/verus-crash-consistency94-20261105/drop-effects-parent-002/handoff/source/source
Pinned rustc source U:
/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/lib/rustlib/rustc-src/rust/compiler

Anchors below use C/candidate prefixes:
O = rustc_mir_build/src/builder/checked_drop_origins.rs
T = rustc_mir_build/src/builder/expr/as_temp.rs
D = rust_verify/src/checked_normal_drop.rs
Parent REPORT means C/REPORT.md. Supporting files and exact numbered excerpts
are identified in CONTEXT-INPUTS.json and SOURCE-EXCERPTS.txt.

Entry: MANIFEST.json SHA256 exactly
1c20cd7ccb5be7e994ecb507da4f5ce4fc61312bee3568fd1967c7cabbbe1123;
all17 listed files match both size and digest. The three historical paths in
BASELINE.json each match the declared size/digest and the corresponding frozen
baseline bytes. This is the historical source snapshot, not the changed live
parent engine. Required entry/exit records are in ENTRY.json and EXIT.json.
Entry authentication followed initial reads; no first-read/atomic snapshot is
claimed. Exit records and evidence sealing do not prove toolchain/source
correspondence.

Exact source identities, SHA256:

- O baseline: a9b5b5a485e1af6b0d86aa5cc5eddc7d8724fb4cb24f24d7fc370016853f713c
  candidate: e648441876070b5b40e1be58a1f3c4a37eb32758104f11f9542bd55db15a86d3
- T baseline: ab195508d00d13ea75bcd00c0b209445b74d8479aa0c9ad1547ed665ce2efb75
  candidate: ceb9af6c062a79591d87e4adea0b8c16c89c13c7e5f3e7a1f6110dac38c36e8b
- D baseline: d9495f9d4b70f5a3feb86aebe43392f097294e8cf8f1a9d5cb98a79a79cb5e44
  candidate: 50e77ba37351192d4fb6aea4ef3c171330e28c764fae3e72975ae5e9cfd69ba2

Prioritized source findings

1. P1, blocking diagnostic robustness: unconditionally refused sites first invoke
   an infallible codegen query, before even retaining the site's raw record.
   D:97-108, especially D:102-106; D:487-502.

   The only eligibility filter is absence of non-region parameters and bound
   variables. It is not a proof that resolution succeeds, that the body is
   untainted, or that querying codegen-dependent MIR here is harmless. The
   vector push is after resolution; STATE receives nothing until the whole
   capture completes. A resolver fatal/bug or query failure can therefore
   prevent both the promised specific Err and retention of the raw site(s).

   This is visible in the pinned API, not inferred from a formatter result:
   U/rustc_middle/src/ty/instance.rs:789-798 requires the DropGlue lang item
   and calls expect_resolve with fully_monomorphized(). At :603-641,
   expect_resolve can emit fatal diagnostics or span_bug on unsuccessful
   resolution. Its fallibility is not converted to the candidate's glue Err.
   U/rustc_ty_utils/src/instance.rs:39-68 calls needs_drop and, for Coroutine,
   optimized_mir(:48), which can request/steal elaborated MIR
   (U/rustc_mir_transform/src/lib.rs:790-800). The hook is itself inside the
   elaborated-query override, holding result.borrow() at D:953-958. Thus a
   diagnostic-only refusal newly expands query dependencies before rejecting
   unsupported code. The source does not establish a safe dependency boundary.

   Qualification: ordinary hook output has already passed PostAnalysisNormalize
   and drop elaboration (U/rustc_mir_transform/src/lib.rs:643-667). It would be
   wrong to claim every ordinary opaque type necessarily reaches this query
   unnormalized. No ICE, query cycle, resource misuse or particular failing
   input was executed/observed in this review. The blocking source defect is
   reliance on an infallible, potentially expansive operation to implement
   unconditional diagnostic refusal, without a guard or unresolved fallback.
   It cannot be certified as reliably fail-closed merely from this source.

   Concrete repair: keep raw unbound capture/refusal independent of glue
   resolution; record an explicit unresolved/not-queried reason for unsupported
   sites rather than invoking resolve_drop_glue here. Do not replace this gap
   with an assumed glue/resource law.

2. P2, diagnostic identity defect: the stored glue identity drops its resolved
   instance kind, and the async successor name obscures cancellation semantics.
   D:75-80, D:105-106, D:121-127.

   glue stores only def_path_hash(instance.def_id()) and Debug(args).
   InstanceKind::def_id dispatches through ShimKind and discards its variant
   (U/rustc_middle/src/ty/instance.rs:257-266). In the resolver, trivial and
   nontrivial glue are distinct DropGlue(None)/DropGlue(Some(ty)); coroutine
   behavior also depends on the selected shim. The retained pair does not
   explicitly preserve which shim was resolved, nor a glue body/call identity.
   destructor stores the ADT destructor declaration's hash, not its instantiated
   body or evidence that this drop reaches it. These observations can be useful
   diagnostics but do not justify a claim to complete concrete glue provenance.
   If glue remains unresolved after finding1's repair, say that explicitly.

   The captured Drop.drop target is cancellation cleanup while an async drop
   is suspended, not the normal continuation of the async destructor. Pinned
   MIR syntax documents that at U/rustc_middle/src/mir/syntax.rs:753-771.
   Rename/describe async_drop_target accordingly. replace is a diagnostic
   assignment-origin flag with no operational meaning(:749-751), not a separate
   replacement execution path proven by this capture. Actual normal target,
   unwind action/cleanup target and block cleanup flag are recorded; this
   finding concerns precision of their interpretation, not missing field reads.
   No acceptance bypass follows because all captured sites are still refused.

3. P2, origin-status/evidence defect: absent capture and genuinely absent binder
   or temporary origins are collapsed, so explicit missing-state information
   required by the handoff is not retained. D:86-88, D:94-95, D:117-118.

   get(def) == None becomes the same empty set as a completed empty binder
   capture. get_temporaries(def) == None becomes indistinguishable from no
   matching temporary for this local. builder_origins: [] and a generic
   "origin ... not established" message do not identify which input is missing.
   In particular, missing binder capture causes every drop root to be called
   unbound; it does not establish anonymity. Preserve availability/status for
   both maps and distinguish no local match. Continue refusing all those cases.
   No name/span guessing occurs in the implementation, and this review does
   not reinterpret an empty origin list as a proven original temporary.

   Related bound: TemporaryScope(O:21-25) contains the actual Scope expression's
   HirId and nested ExprId, but omits region_scope and TempLifetime used by T.
   It records an expression edge, not the destruction-lifetime identity or
   cleanup scheduling proof. The parent report should retain that distinction
   when saying it implements lane B's original-temporary capture action.

What the actual source does establish, within this review

T:38-50 records the enclosing THIR Scope ExprId, nested value ExprId, actual
HirId and the Local returned by unchanged in_scope recursion. T:131-135 records
the actual expr_into_dest temporary and keeps scheduling/result unchanged.
The fixed-temp reuse branch T:54-58 remains intact. O appends each observation
rather than overwriting by Local, so repeated/reused local observations survive.
D:117-118 associates all observations with equal Local; it does not pick a
unique creation event. Records omit an occurrence/block epoch and do not prove
which reused occurrence owns a particular elaborated drop. Builder-to-elaborated
local stability through transformations is not validated here. Missing origins,
new MIR locals and projected drops must remain incomplete provenance. No
MIR/VIR/AstId/resource correspondence follows from typed indices or Debug text.

The raw scan D:90-130 visits every basic block's Drop terminator, including
cleanup blocks, and preserves full Debug place/projection/type, block/local,
body phase, phase flag, targets, unwind, replace and available origins.
It skips roots in the binder map; projected places rooted at named locals
therefore continue through the inherited normalizer and can be declined there.
It does not capture the full surrounding CFG, calls already produced by async
lowering, or prove destructor/glue execution. Runtime elaborated-MIR phase
information is observed, not source ownership authority.

For any successfully collected nonempty unbound vector, D:488-500 produces
program Err before normalize and leaves origins empty. In unchanged prepare,
capture.program is checked at D:885 or :900 before the corresponding function's
rewrite_block(:910), generated call(:724) or field retirement(:732).
Unchanged check_native D:942-946 propagates a failed native capture for an
expected function. prepare returns replacements only after its entire loop;
historical verifier.rs:2933-2940 uses ? before installation. Other functions'
temporary replacement values can have been constructed earlier in that loop,
but a failed prepare does not install the returned program. The candidate
contains no successful resource effect for these unbound sites.

Regenerated three-file unified diff matches candidate.patch byte-for-byte.
SOURCE-COMPARISON.json independently records byte-identical targeted source
extractions for supported_type, drop_impl, normalize, account, rewrite_block,
checked_reference_frames, prepare, check_native and elaborated. This extraction
is a brace-based textual oracle, not Rust AST equivalence or semantic testing.
The added code contains no new assume/admit/axiom/external_body/primitive law,
native producer, or generated resource-call path. The inherited HasResolved
assumption D:771-775 is unchanged and remains experimental/unadopted.

The admission set is not broadened: a new refusal guard precedes the identical
normalizer. Whole concrete named owners, direct construction, exact binder/
parameter origins, local checked Drop, no nested glue and existing account/
rewrite checks remain required. Exact preservation of all previously accepted
programs is not proved: the new scan also sees cleanup/unreachable/unbound
erasure locals before normalizer ghost filtering(D:456-459), so it may impose
additional refusals. No executed regression is claimed.

O:9 defaults capture off; new retain functions return before locking when off.
D:483-486 returns before scanning when disabled; historical config.rs:133
defaults the feature false and verifier.rs:3322-3323 begins only when enabled.
This establishes initial default-disabled shape, not repeated-process isolation.
Both existing begin phases clear temporary records(O:27-31, D:150-164).
Mutexes protect individual containers; no new lock is held across recursion
or glue queries. Resets, binder/temporary reads and phase observations are
separate operations with no epoch or atomic snapshot. Overlapping sessions or
phase transitions could mix observations; normal driver calls are sequential
(driver.rs:356-365), and no actual overlap or race was observed here.
This is an unresolved protocol/runtime evidence limit, partly inherited, not
a demonstrated new data race under the ordinary sequential driver.

Static API/lint assessment and documentation boundary

Public origin structs/getter are reachable through the historical public
builder::checked_drop_origins module; pub(super) retain functions are visible
to the builder descendants. ExprId/Local observations have no tcx lifetime
stored in the global map. The BlockAnd tuple field and added Copy Local read
are consistent with the inspected Builder definition. TypeVisitableExt and
Drop's five fields exist in the pinned source. No definite new API/visibility
type error was established by reading; none was ruled out by a compiler.

Capture.original_unbound_drops is stored but never read after insertion.
Most private OriginalUnboundDrop fields are used only by derived Debug/Clone;
Rust dead_code warnings are plausible because derives do not count as reads.
The inspected crate/workspace files do not impose a blanket deny(warnings).
These are static lint observations, not an observed typecheck/lint failure.

C/REPORT.md truthfully says unbuilt, no native bridge, no executed original
capture and no semantic-negative result. Its claims to actual resolved glue
identity and complete implementation of lane B's next action must be bounded
by findings1-3. Parent's three direct rustfmt runs are syntax parsing only;
their stored results were authenticated, not rerun. Nine textual preservation
checks establish no AST equivalence. No source was integrated into an engine.

No candidate code checks actual Arc strong/weak/unique allocation behavior,
Mutex poisoning/guard history, LRU reservation/key identity, Option replacement,
custom hashing, exact pending-buffer payload/frame, or later flush behavior.
It does not close the native header-to-buffer bridge. Existing native-path
FAILED, native Git UNAVAILABLE, compiler/source UNESTABLISHED and inherited
conditional/unadopted TCB qualifications stay unchanged. S0/Infer/full-scope/
import/export and excluded growth049, zero-growth050, mutation030, padding95
and diagnostic107-D1 work remain unchanged. No cell is closed.

One next code/build decision

Hold this overlay as unbuilt and unintegrated. First make the unbound-drop
capture query-free with explicit unresolved/missing statuses and correct
diagnostic labels, preserving unconditional refusal and the old admission
checks. This is a concrete small source repair, not native acceptance. A pinned
typecheck/build would require separate authority; it was not started or approved
by this review. No automatic further audit/delegation chain is proposed.

Accounting and evidence limits

source0/native0/trust0 means source verifier/proof applications0, native body
verification/execution0, new trusted laws/producers0. Three source files were
reviewed, not proved. Compiler typechecks/builds0, formatter runs by reviewer0,
verifier0, solver0, tests0, semantic controls0, native runs0. Source reading,
hash/diff utilities and report generation are the only local executions.

All authored review files use apply_patch in this fresh private directory.
No runtime/toolchain/cache copied. Report is below20KiB and total scratch below
128MiB; final sizes/time/disk samples are in SEAL.json. Every sampled global
free-space value exceeds16GiB; no continuous global monitor is claimed.
Required identities are rechecked at exit, including all17 manifest files and
all3 historical baselines. Evidence manifest excludes itself and SEAL.json to
avoid recursive hashes; SEAL.json hashes that manifest. Digests give integrity,
not proof, temporal atomics or full utility provenance.

Utility endpoints/context source inputs are hashed after bootstrap, with exit
comparisons. Complete dylib/OS/shared-cache/harness closure is untraced.
FAILURES.json preserves absent-path/search statuses and output truncations.
Those are utility/source-discovery failures, not compiler/proof results.
