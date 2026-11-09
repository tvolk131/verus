# Independent review — native-resource cycle 1

Date: November 7, 2026. Read-only review of the implemented, frozen candidate,
not the earlier design alone. Shared parent:
`/Users/tommyvolk/.t3/worktrees/verus/t3-60978068`.

## Verdict and decision boundary

**Qualified support for the bounded, hypothetical one-token/two-slot IR
transport fragment. No blocking soundness defect was found in the supported
constructor-generated programs. This is not native Rust verification, full E1,
E2 correspondence, primitive-law adoption, or a database acceptance result.**

The constructor emits real ordinary VIR bodies. The inspected tests send their
normal simplified/elaborated SST/AIR obligations to AIR's Z3 interface; they are
not a host ownership state machine or a report-only VC generator. Affine state
is conditional on the explicitly supplied incoming capability, not discovered
or established from native Rust. There is no Rust-front-end producer.

The production/import/export guards are correctly placed relative to the
ordinary filters and body stripping inspected here. The tests establish their
shared predicate, not an end-to-end driver/publication exercise. The findings
below limit stronger evidence claims; none supplies an exploit of the currently
supported fragment or warrants widening its admission policy.

**All eighteen cells remain OPEN; trust and S0 are unchanged. No acceptance
certificate is issued.** No recommendation here resumes zero-growth050,
growth049, mutation030, or the refused drop pipeline.

## Authentication, scope, and restrictions

Frozen inputs:

- `/private/tmp/verus-native-resource-cycle1-20261107-2l2gwdwr/CANDIDATE.json`
  SHA256 `aa037f8c19348b8728e899cc18029f8fb6fe30c4f51bbcb66e4088dee72c381d`.
- `/private/tmp/verus-native-resource-cycle1-20261107-2l2gwdwr/BASELINE.json`
  SHA256 `8c62e7d70aa0a5c52aeb8d5b7891b150b9d6f4267187f733a14e5c16583c787e`.
- Candidate/baseline declared HEAD:
  `b32846b61dcf724618f7345e219e5007a0c79643`.

Entry verification at `2026-11-07T21:19:38.290732+00:00` independently recomputed
SHA256 of all **14 candidate source/context files and all 9 logs**: zero missing
files or mismatches. Recomputed the other **2,354 baseline-file hashes**: zero
missing files or unrelated baseline drift. This establishes the specified
content boundary, not a closed compiler/toolchain provenance certificate.

Read the cycle ledger, GOAL.md and STATUS.json under
`evidence/redb-crash-consistency94`, and the prior design report
`/private/tmp/verus-e1-design-20261107-Kmiey8/REPORT.md`; their native-proof and
authorization qualifications are retained. Checked for applicable AGENTS.md
instructions; none were found in the inspected ancestry or source/evidence
trees.

Only source/log reading, hashing, read-only Git inspection, disk checks, fresh
report-directory creation and this report write occurred. **Zero builds, tests,
verifier/solver/native executions, network access, Git writes, source changes,
or subdelegations.** The only new file is this report, written via apply_patch.
Entry free space was approximately 118 GiB; the later check still exceeded
16 GiB substantially. Exit authentication is recorded at the end.

## Prioritized findings and missing controls

### F1 — P2 evidence/defense gap: predicate tests are not boundary executions

Anchors: `source/vir/src/native_resource/tests.rs:109`, `:128`, `:142`;
`source/rust_verify/src/import_export.rs:111`, `:273`;
`source/rust_verify/src/verifier.rs:2911`;
`source/vir/src/ast_to_sst_func.rs:747`, `:1059`;
`source/vir/src/ast_to_sst_crate.rs:21`.

The publication test removes the body's contents, JSON-round-trips the crate,
then invokes `reject_crate` three times with different diagnostic strings.
Changing that string to "import", "export", or "native source admission" does
not exercise the corresponding rust_verify entrypoint. Likewise the default-off
and spec/safe-API tests call `check_lowering` directly. There is no matched
actual import/export, filtered/no-verify/no-lifetime driver control in these
resource tests. The cycle ledger correctly acknowledges this limitation.

Source inspection does support the present blanket barriers: import refuses
immediately after actual bincode decoding, export refuses before visibility
retention/body removal, and source admission refuses before UserFilter creation.
But passing the helper tests would not detect a future misplaced/missing call
in those entrypoints.

There is also a concrete limitation of the *lower-level* hook: it is inside
`func_def_to_sst`, not at every marked-function entry. A marked bodyless
exec/spec declaration never reaches it via `function_to_sst`; a function with
`is_decrease_by` can be skipped even earlier. Thus it is not a universal
default-off/spec refusal firewall for arbitrary internal VIR. This does not
contradict safety of the supported producer: `instrument` requires a body and
exec mode, the constructor is absent when the feature is off, and the real
production/import/export barriers reject marked crates independently of bodies.
No reachable production bypass was found. Do not promote the helper tests into
broader lowerer-admission evidence.

Minimum control before a stronger fail-closed claim: one boundary-regression
group using the real APIs and actual bincode format, including a private/bodyless
marked member, a marked member excluded by a function/module filter, and the
no-verify/no-lifetime cases. Exercise actual lowerer entrypoints for the
default-off and pure-context controls, not only the predicate. Keep all this
internal; adding a Rust resource attribute is not necessary or authorized here.

### F2 — P2 evidence qualification: rejection is not necessarily raw SMT SAT

Anchors: `source/vir/src/native_resource/tests.rs:278`, `:281`, `:297`;
`source/air/src/context.rs:46`;
`source/air/src/smt_verify.rs:288`, `:331`, `:372`, `:443`, `:499`.

The fixture requires exactly one CheckValid command. For positive cases it
accepts only `ValidityResult::Valid`; for negatives it requires a localized
`Invalid` result whose diagnostics contain the resource label (and, for the
two asymmetric joins, the intended event span). Every other result panics;
non-query command results must also be Valid. There is **no ignored result
turning into a positive proof** in this helper.

However AIR's Invalid is deliberately a verifier-rejection category, not a
raw-SAT-only type. Z3 `unknown` with an incomplete-quantifiers reason can proceed
through the model/localization path and return Invalid with a message. The
fixture does not separately record the raw solver result. Consequently the
logs support **18 expected localized backend rejections**, not an independently
auditable assertion that every one had a definitive SAT counterexample. No
evidence was found that any such unknown actually occurred. The formulas are
small, but that is not a substitute for a retained status.

The existing conservative rejection semantics are sound for acceptance: unknown
does not become Valid. If stronger negative-control/counterexample evidence is
wanted, retain raw statuses and the actual localized outcome per submission,
without changing unknown into success. Retain the current wording "expected
rejections" unless that extra evidence exists.

### F3 — P3 coverage gap: several guards lack independently matched controls

Anchors: `source/vir/src/native_resource.rs:154`, `:171`, `:253`, `:304`;
`source/vir/src/native_resource/tests.rs:317`, `:359`, `:405`, `:448`, `:467`;
`source/vir/NATIVE_RESOURCE_IR.md:44`.

- All ten wrong-identity controls exercise **Use**. Transfer and Consume call
  the same inspected `check_authority` helper, but have no wrong-descriptor
  matched backend controls of their own. Positive descriptors are the incoming
  variables themselves; there is no positive using distinct parameter names
  connected by a nontrivial equality precondition. Such a pair would distinguish
  successful symbolic binding from mere same-variable equalities.
- `NRE101_DESTINATION_EMPTY` is emitted, but lacks an isolated occupied-distinct-
  destination negative. This is not a counterexample to the one-token model:
  after successful assertions both slots cannot be live. The self-transfer
  negative tests `NRE105`, not independent NRE101 behavior. The documentation
  truthfully records this limitation. Do not introduce an authority-mint or a
  two-token entry contract merely to obtain that test; that changes the frozen
  semantics. A separately labeled corrupted-internal-IR guard test would require
  separate, explicit test-design approval and would not be producer evidence.
- Operand rejection tests cover an undeclared and wrongly typed incoming field,
  not mutable/spec operands, event-expected operands or a bad condition/frame.
  The implementation checks these through shared helpers, but distinct failure
  paths are not all exercised. The node budget has a test; the depth limit does
  not. Namespace/privacy freshness has no two-body or spelling-collision control.
- The event enum cannot express calls/loops/borrows/drop/unwind, and the
  constructor rejects arbitrary nonempty bodies. This is a good structural
  exclusion, not a tested native fail-closed matrix for those features. Likewise
  the scalar frame assertion is checked, but is not a heap/frame-effect test.

These are coverage limitations, not evidence of an accepted invalid program.
Highest value within an unchanged transfer vocabulary: a distinct-name/equal-
descriptor positive paired with its disequality negative; wrong descriptor on
Transfer/Consume; two independent fixture bodies including an ordinary input
spelled `affine`; and the actual boundary controls in F1. No additional runs or
mutations were performed by this review.

## Soundness audit of the supported fragment

### Affinity, branch state, and exits

`source/vir/src/native_resource.rs:330` initializes exactly Input=true and
Output=false. This is the explicitly hypothetical entry capability; it is not a
native ownership theorem. There is no acquisition, split, clone, allocation or
fresh-epoch event.

`check_authority` at `:171` verifies the slot's live flag and equality of all ten
expected descriptor components with the single immutable incoming descriptor.
Use changes no flags; Consume changes only its live source to false. Transfer
first checks source live, identity, distinctness, and destination emptiness,
then clears the source and sets the destination. On every path where these
assertions succeed, the invariant `live(Input) + live(Output) <= 1` is preserved;
transfer preserves its count, consume decreases it, and Use preserves it.
No transition evolves the descriptor. Self-transfer cannot exploit its later
assignments because the preceding distinctness assertion is false.

`lower_events` at `:275` recursively constructs two ordinary VIR branch bodies;
it does not traverse them while mutating a shared host ownership map. The shared
host mutable counter only enforces the construction budget. Ordinary assignments
and branch SSA carry the live state. The positive round-trip branch preserves
the token, and the two asymmetric-join negatives isolate a bad use on each slot.
Their repair restores the expected state after the bad use with the same symbolic
condition, then consumes before exit; this removes the previously competing exit
obligation rather than weakening the source-live check.

`exit` at `:206` checks both flags against the declared result location (or both
false) and equality with the saved scalar frame. It is emitted before each
Return at `:295` and at normal fallthrough at `:336`. The early return really is
a VIR Return; subsequent statements are unreachable, not a host state merge.
The four positive queries include transfer/use/consume, a branch join, an
output-slot return, and an early consumed return. Normal and early wrong-result
shapes have backend rejections. A returned token is only this internal exit-shape
claim; the unit return value is not an exported resource-bearing call interface.

### Operand binding and freshness

`check_parameter` at `:154` binds every referenced descriptor field and frame to
an immutable Exec integer parameter, and each branch condition to an immutable
Exec Boolean parameter. The descriptor's ten components remain uninterpreted
identity integers: validity of intervals, allocation/initialization freshness,
typed native owner identity and activation uniqueness are not established.
This is accurately stated in the documentation.

Private locals use VarIdentDisambiguate::NativeResource (appended at
`source/vir/src/ast.rs:88`), rather than string spelling alone. Name formatting
is handled at `source/vir/src/def.rs:1305`; incoming parameters with that private
variant are refused at `native_resource.rs:316`. Standard AST simplification
and SST renaming then bind and distinguish variables. Fixed serials 0/1/2 are
body-local compiler names, not a proof of fresh dynamic allocations or
call-activation ownership. Multiple calls/activations and resource summaries
remain outside the fragment.

### Assertions survive the exercised normal proof path

`assertion` at `native_resource.rs:105` always emits AssertAssume with
`is_assume=false`, a resource label, and the provided event span. No resource
success axiom or external-body resource law is emitted. State declarations and
exact assignments are definitional, not unchecked success assumptions.

Tests at `native_resource/tests.rs:218` call simplify_krate,
ast_to_sst_krate (including ordinary elaboration), func_sst_to_air and the real
AIR solver interface. They check marker survival in both simplified VIR and SST.
`ast_to_sst.rs:2298` on the ordinary exec-check path produces Assert followed by
Assume; the following Assume is the standard checked-assert rule, not a substitute
for proving the preceding Assert. `sst_to_air.rs:2127` preserves the assertion's
error in AIR. There is no loop-prefix assert-removal path in the supported event
vocabulary.

An important existing-pass qualification: the eagerly generated *recommends*
check (`ast_to_sst_func.rs:1070`) lowers assertions to assumptions via
`ast_to_sst.rs:2299`. This review does not claim all alternate checks preserve
resource obligations. The actual tested channel explicitly selects
`exec_proof_check` at `native_resource/tests.rs:251`; that main check is generated
first with recommends checking disabled (`ast_to_sst_func.rs:1061`). No recommends
result supplies this test evidence. A future producer must not substitute a
recommends/safe-API/precision run for the main resource proof.

The test helper checks all command results, checks a real query count of one,
rejects empty-negative diagnostics, and checks exact occurrence spans for both
branch-use negatives. Only those two negatives make an exact event-span claim;
the other negative tests check labels, not exact occurrence spans. The ordinary
driver's result handling at `rust_verify/src/verifier.rs:885` onward also retains
Invalid/Canceled as unproved/error outcomes rather than positive proof credit;
the resource tests do not themselves exercise that enclosing driver accounting.

## Persistent marker, Infer identity, and production barriers

`native_resource.rs:4` reserves the `experimental-native-resource-ir:` prefix
inside existing `infer_candidate_id`. `is_marked` checks the prefix regardless
of feature state, body availability, mode, visibility or inferred status.
`instrument` refuses an existing ID, `attrs.infer`, external-body attributes,
non-exec/non-static/non-unit/generic/nonempty fixtures, then installs its marker
at `:339`. Thus it cannot silently relabel an existing ordinary Infer candidate.

The new marker does **not** add a field to FunctionAttrsX or change its serialized
field order. Appending the new variable-disambiguator variant does not reorder
old enum variants. Serialization of bodies containing that new variant is not
claimed compatible with an old executable; refusal, not cross-version resource
artifact compatibility, is the policy. The actual test round-trip is JSON, not
the production bincode format (F1).

The marker is not a valid Infer CandidateId: `infer_proof.rs:18` requires exactly
64 hexadecimal characters. Ordinary identity installation at
`infer/contract.rs:105` computes and writes such a hash; it would overwrite this
shared field if wrongly applied to a marked function. Current producer rules
prevent that overlap (`attrs.infer` is refused), and the inspected ordinary
non-Infer path preserves functions/attributes (`infer.rs:1247`, `:1352`,
`infer/registry.rs:628`). Existing AST/SST cloning preserves attrs
(`ast_to_sst_func.rs:1122`). There is no current source-level setter for the
reserved value; normal rust_to_vir initializers set the field to None at
`rust_to_vir_func.rs:226` and `:1503`. No actual marker-laundering path was found.
This is a source-audited invariant, not an identity-installation integration
regression test. Any future change that permits inferred resource functions
must revisit the shared-field collision before changing admission.

Actual refusal ordering:

1. `import_export.rs:111` decodes CrateWithMetadata; `:116` rejects the marked
   crate before model selection, Infer metadata or consumer composition.
2. `import_export.rs:273` requires an export destination; `:276` rejects before
   the public-function retain at `:283` and body stripping at `:292`. No-path
   export returns without publishing anything. A private or bodyless marked
   function still poisons the whole crate; refusal is not marker stripping.
3. `verifier.rs:2914` rejects the exportable local crate before the no-lifetime,
   imported-dependency scope and Infer full-scope logic, and well before the
   ordinary user filter at `:3049`. It does not rely on Infer's separate predicates
   recognizing the resource marker. The full exportable-crate query performs
   ordinary checking/Infer preparation first (`query.rs:497`, `:546`), so this is
   not a new front-end ingress check; in the actual fragment there is no
   Rust-front producer and no supported overlapping Infer identity installation.

The feature has no default enablement (`source/vir/Cargo.toml:23`), and the affine
constructor is compiled only under that feature (`native_resource.rs:36`). The
marker detector and production barriers remain compiled even without it.
No ordinary Infer publication policy, certificate format, primitive law or S0
native admission policy is relaxed by the inspected changes.

## Log reconciliation and provenance limits

All logs below are in the authenticated cycle directory; none was rerun here.

- compile1: VIR test compilation finished successfully. compile2: root invocation
  failed with E0554 in rustc_mir_build/rustc_hir_analysis. compile3: failed with
  E0308 at export borrowing and E0063 for the former added attribute field.
  compile4 has a successful rust_verify check completion. The final source
  reflects the reserved-field marker correction rather than that abandoned
  added-field layout. The log alone does not print the full feature command.
- test1: 11 passed/1 failed, with `NRE103_EXIT_SHAPE` where the branch test expected
  `NRE100_SOURCE_LIVE`. This was a competing-obligation diagnostic failure, not
  evidence that invalid authority was accepted. Its failure remains retained.
- test2: 12 passed; exactly 22 resource CheckValid submission messages, 4 marked
  positive and 18 expected rejection cases. This predates the final marker change.
- test3: 192 passed, but predates the final marker correction; not final-candidate
  default-off validation by itself.
- test4: 201 passed/0 failed; exactly 22 resource CheckValid submission messages,
  comprising 4 positive and 18 expected rejection cases. The source's checked
  match/count assertions support interpreting these as actual solver-interface
  submissions, not merely syntax counting.
- test5: 192 passed/0 failed on the final default-off suite, including the
  default-off/pure/safe-API helper and marker round-trip controls. It has no
  experimental resource solver submissions.

There are 44 explicitly traced submissions in test2/test4. The ledger reports
21 in test1 and hence 65 total. test1 has no per-submission trace, so the earlier
21 and historical total 65 are retained **as ledger accounting**, not newly
independently authenticated per-query outcomes. The initial source version is
not sealed separately in this candidate manifest. The final 4/18 count is
directly corroborated by the final source and test4 log.

Hash matching authenticates the supplied files, not causal linkage of every
incremental build to all dependency bytes. These logs do not contain complete
command/exit-status records, raw SMT transcripts or a closed executable-input
manifest. The declared Rust/Z3 versions and solver hash were not reexecuted or
reauthenticated in this review. This qualification does not erase the bounded
test outcomes or turn the failed attempts into success.

## Minimum next step and stop boundary

For the frozen one-token constructor fragment, record this qualified independent
review and preserve the frozen sources/logs and earlier failures. No source fix
is required by a demonstrated supported-fragment counterexample in this review.
Do not describe it as completed E1 or native proof.

If stronger internal robustness or end-to-end fail-closed evidence is required,
the minimum useful follow-up is the actual boundary regression in F1, followed
by the distinct-name descriptor pair in F3. Such work belongs to the parent only
within genuinely remaining authorization/budgets, or to a separately approved
bounded cycle; this reviewer spends none. The four dedicated compile/check
commands are already exhausted. This report grants no new command, retry,
primitive assumption, producer, multi-token state, or admission scope.

Native correspondence still needs a separately specified and checked source-to-
capability producer, actual caller/activation/freshness obligations, and the
appropriate primitive/glue/NLL/heap/call closure. Nothing about the four symbolic
positives or eighteen expected IR rejections discharges native Arc/Weak/Drop,
cache-return, crash consistency, recovery, durability or key/value refinement.

## Exit authentication

Final read-only rehash completed at
`2026-11-07T21:28:04.860362+00:00`: all 14 candidate files and all 9 logs
again match CANDIDATE.json; zero missing files or mismatches. All 2,354 other
baseline files again match; zero missing files or unrelated baseline drift.
Both manifest hashes are unchanged from entry. Read-only `git rev-parse HEAD`
also matches `b32846b61dcf724618f7345e219e5007a0c79643`.

Exit free space: **119,464,226,816 bytes**, comfortably above 16 GiB. Review
elapsed to final rehash: approximately 8 minutes 27 seconds. Only this exit
record was added after the rehash; no further parent source/log work follows.
Stop at report handoff. All18 OPEN; trust unchanged; no acceptance certificate.
