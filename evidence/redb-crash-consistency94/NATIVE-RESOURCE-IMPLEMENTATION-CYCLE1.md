# Native-resource implementation cycle 1 — November 7, 2026

User authorization: “Approved. Are you able to keep going?” approves one bounded
implementation cycle, builds, targeted tests and independent review. It does
not adopt primitive assumptions, change S0 admission, or reset old allowances.

## Baseline and limits

Current dirty parent HEAD: `b32846b61dcf724618f7345e219e5007a0c79643`.
Fresh SHA256 baseline of 2,364 files:
`/private/tmp/verus-native-resource-cycle1-20261107-2l2gwdwr/BASELINE.json`.
Preserve inherited edits; deliberately port rather than copy old snapshots.

Fresh limits: one candidate, at most four compile/check commands and six test
commands, at most 96 solver submissions, ten minutes per command, one initial
read-only design delegate and one independent soundness-review delegate.
Allow at most three focused correction rounds within these totals. No copying
toolchains, dependency downloads, native redb execution or historical replay.
Stop before free disk falls below 16 GiB. Record failures, not just successes.
These limits do not replenish any earlier task's allowance.

## Scope

Implement a default-OFF compiler-internal transfer-only affine IR fragment.
Generate private mutable live flags, descriptor equality checks, normal-exit
checks and branch control flow in real VIR bodies; run ordinary VIR/SST/AIR
lowering and SMT checking. The internal constructor takes an explicit
hypothetical entry capability, not an inferred fact about native Rust.
Retain a function marker across lowering and refuse production native
admission and all artifact import/export of marked functions.

This is conditional IR-transport evidence only, not E2 native correspondence,
Arc/Weak semantics, actual Drop, NLL loan ends, byte authority, E1 completion,
or any database acceptance. No new axioms or external-body resource laws.
The former prototype's finite checker/capture pipeline is not reinstated.

Initial design report (completed, no parent edits/execution):
`/private/tmp/verus-e1-design-20261107-Kmiey8/REPORT.md`.
The final independent review is separate and will inspect the actual candidate.

S0, ordinary Infer/full-scope/import/export barriers and all18 OPEN remain.
Do not retry/reroute zero-growth050, exhausted growth049, mutation030, or
refused drop-pipeline work.

## Execution ledger

Implementation is an internal constructor producing ordinary VIR function
bodies, not a new AST effect variant, reporting sidecar or finite resource
checker. The constructor accepts only an empty exec/unit fixture and explicit
hypothetical incoming descriptor. Its private flags and assertions travel
through production simplification, SST elaboration and AIR query generation.
The reserved marker namespace in existing `infer_candidate_id` preserves the
ordinary attribute serialization layout. A new private variable disambiguator
is appended to the enum; old discriminants are not reordered.

Compiler source entry points, import and export reject marked crates. There is
no Rust-front-end producer or native correspondence claim. Source documentation:
`source/vir/NATIVE_RESOURCE_IR.md`; implementation and tests:
`source/vir/src/native_resource.rs`, `source/vir/src/native_resource/tests.rs`.

### Attempts, including failures

All commands are offline; logs are in the fresh baseline directory.

1. `compile1.log`: feature-enabled VIR unit-test compilation PASS.
2. `test1.log`: 11/12 tests PASS, branch diagnostic control FAIL. Its generated
   program had both invalid use and invalid exit; backend reported EXIT_SHAPE,
   not the intended SOURCE_LIVE. All 21 submissions are retained and counted.
   This is a test-design failure, not evidence of accepted invalid authority.
3. Correction round 1: normalize the two branch states after the invalid use
   and consume before exit, isolating the source-live obligation. Add distinct
   event spans and check the precise branch-use failure occurrence.
4. `compile2.log`: FAIL, invoked from repository root via `--manifest-path`.
   Cargo did not load source/.cargo/config.toml; stable feature errors E0554
   occurred in rustc_mir_build and rustc_hir_analysis. No success credit.
5. `test2.log`: feature-enabled targeted controls PASS, 12 tests / 22 actual
   CheckValid submissions (4 positive, 18 expected backend rejections).
6. `compile3.log`: FAIL in correct source directory: missing field in the
   second explicit attribute initializer and missing borrow at export guard.
7. Correction round 2: fix export borrowing; replace the proposed new Boolean
   attribute with a reserved existing-field marker, avoiding attribute-layout
   drift and removing the missing-initializer issue. Marker serialization
   round-trip and actual nonempty-fixture refusal controls are added.
8. `test3.log`: pre-marker-correction default-off full VIR suite PASS, 192
   tests; retained as earlier-candidate evidence, not final-source validation.
9. `compile4.log`: current feature-enabled rust_verify check PASS. The four
   dedicated compile/check-command allowance is exhausted.
10. `test4.log`: final-source feature-enabled full VIR suite PASS, 201 tests,
    including 22 actual resource CheckValid submissions: 4 positives and 18
    matched backend rejections. Conditional IR evidence only.
11. `test5.log`: final-source default-off full VIR suite PASS, 192 tests,
    including default-off, spec/safe-API refusal and marker round-trip controls.

Reviewed-checkpoint totals: dedicated compile/check commands 4/4, test commands 5/6,
resource solver submissions 65/96. Earlier regressions perform host structural
checks, not additional resource solver queries. Two correction rounds used.
The independently reviewed checkpoint used five test commands and two
correction rounds. The final reconciliation below spends the remaining test
command/round; no old task allowance is reset.

Formatting: targeted rustfmt and git diff --check PASS. Baseline recheck found
no missing file or unrelated content drift. Cargo's configured build-dir puts
intermediates under ignored source/target-verus even with CARGO_TARGET_DIR set;
the root-invoked failed attempt also created fresh private intermediates. No
toolchains were copied. Free space at last check was 121 GiB (>16 GiB floor).

### Deliberate limits

The ten descriptor components are symbolic identity fields, not a canonical
Rust type/instance binding or established heap authority. One intact token and
two slots do not cover split permissions, fresh allocations, loan end, multiple
call activations or evolving generations. The framed scalar is not a byte or
heap frame. The distinct occupied-destination state is unreachable in this
one-token fragment: its guard is emitted but no reachable distinct-occupied
negative is claimed. No native source erasure/CFG/drop correspondence, caller
closure, primitive progress, runtime abort law or E2–E6 has been established.
Final source/log hashes and preservation results are sealed in `CANDIDATE.json`
in the fresh baseline directory. This authenticates the bounded inputs, not
compiler/toolchain reproduction or a whole-native acceptance certificate.

Default-off and publication controls call the production refusal predicate;
they are not end-to-end driver artifact executions. All18 remain OPEN.

## Independent review and final reconciliation

Unchanged report: `NATIVE-RESOURCE-IMPLEMENTATION-INDEPENDENT-REVIEW.md`,
SHA256 `5db31a37b1cef913c91ec2c6b6fe581913f867addb6e7c0e6754ffeff4b4345e`.
The reviewer authenticated all 14 candidate files, 9 logs and 2,354 unrelated
baseline files at entry and exit. Verdict: qualified support; no blocking
soundness defect found in supported constructor-generated programs. This is
not full E1 or native proof, and no acceptance certificate was issued.

Before reconciliation, all reviewed inputs were reauthenticated and retained
in the fresh cycle directory as `REVIEWED-CANDIDATE.tar.gz`, SHA256
`4e492cc19a677da85cf286cbe4ebf0e3220d45bd2c0afa3375256819b33fe306`.
It includes the 14 original source/context files, 9 logs and two manifests,
not toolchains. Original CANDIDATE.json is unchanged. Later documentation and
test changes are explicitly separate from that reviewed checkpoint.

### Finding dispositions

- F1 remains OPEN as an evidence/defense gap: helper predicate tests do not
  exercise actual bincode import/export or filtered/no-verify/no-lifetime
  driver paths. Bodyless/skipped internal declarations do not universally
  enter func_def_to_sst. Supported construction and the separately inspected
  production barriers remain fail closed; do not claim a universal internal
  VIR firewall. Full boundary regressions need a fresh bounded allowance.
- F2 retained as a qualification: Invalid is a conservative localized verifier
  rejection, not necessarily raw SMT SAT. There is no evidence unknown actually
  occurred, and no unknown becomes positive proof. No SAT/counterexample claim
  is made. Existing logs were not retrospectively supplemented with raw status.
- F3 partially addressed in parent-authored test-only correction round 3:
  distinct-name equal/disequal descriptor pairs for transfer and consume;
  successful transfer/use/consume with an ordinary input named `affine`;
  mutable/spec input, wrong frame/condition/expected binding, private-name and
  excessive-depth refusals. Occupied-distinct destination remains unreachable;
  multi-body/dynamic-activation independence and real native exclusion tests
  remain outside the evidence. No authority-mint or multi-token state added.
- The shared Infer-ID marker is source-audited, not a new identity-installation
  integration proof. Overlapping Infer installation remains unsupported. The
  main exec proof, never recommends/safe-API/precision output, supplies credit.

### Final validation and stop boundary

`test6.log` records the full command, working directory and exit status 0.
The feature-enabled full VIR suite passes **204 tests**. Its resource controls
submit **26 actual CheckValid queries: 6 symbolic positives and 20 matched,
localized backend rejections**. The four new submissions are two positive and
two disequality-negative descriptor controls. These additions were not sent
for another independent review; production code is unchanged from that review.

The reviewed-candidate default-off suite passed 192 tests in test5. It was not
rerun after the feature-gated test-only additions; the default-off active code
and all reviewed production source remain unchanged. The successful compiler
check likewise predates only those tests and documentation, not production
changes. Formatting and preservation checks are recorded in FINAL.json.

Final totals: dedicated compile/check commands **4/4**, test commands **6/6**,
resource solver submissions **91/96**, correction rounds **3/3**, design
delegates **1/1**, candidate-review delegates **1/1**. Of the 91 submissions,
70 are explicitly traced in test2/test4/test6; 21 from test1 remain source/ledger
accounting, not independently retained per-query outcomes. All earlier failures
remain. Compiler/runtime provenance is not thereby closed.

This approved cycle is complete, **not the database goal**. No further compiler
build, test command or correction round is available within it. Next useful
work is a separately bounded actual driver/import/export regression cycle,
then a specified checked native source-to-capability producer and its caller,
activation, primitive/glue/NLL/heap closure. No primitive adoption or admission
widening is requested or inferred by that recommendation. All18 remain OPEN.
