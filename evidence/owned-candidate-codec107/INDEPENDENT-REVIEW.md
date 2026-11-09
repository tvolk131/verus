# Independent review: actual VIR owned-candidate codec107

Date: November 7, 2026 America/Chicago / November 8 UTC.
Review start: 2026-11-08T02:59:48Z. Fresh scratch only:
`/private/tmp/verus-codec107-review-20261107-t9U1yK`.
Parent and candidate were read-only throughout. No subdelegation or Git writes.

## Integration verdict

**REQUEST CHANGES for the private, test-only prototype as submitted.**
The architecture is appropriate for an untrusted proposal-data experiment:
actual VIR conversion, a private `cfg(test)` module, and no persistence or
admission integration. The narrow objections below should be fixed or explicitly
narrowed before integrating that prototype. This is not a production-cache,
warm-cache, frontend-correspondence, certification, or readiness approval.

No proof unsoundness, installed false contract, solver authority, production
cache acceptance, or change to the existing owned-cache bypass was demonstrated.

## Findings and objections

### R1 — P2: static observation eligibility is not enforced (reproduced)

Candidate `source/vir/src/infer/owned/candidate_data.rs:166` validates observation
mode, generic parameters/bounds, proxy/external-body status, parameter modes,
name uniqueness, and call-free bodies, but never checks `FunctionKind::Static`
or `ItemKind::Function`. `is_pure_observation` does not provide those checks.
The encoder accepts only Static call nodes and the decoder unconditionally
constructs Static call nodes at candidate line 665; that does not make the
current callee definition static.

Reviewer test `reviewer_tests.rs:134` changes an otherwise accepted observation
definition to `TraitMethodDecl`, supplies it to the actual Context, and successfully
encodes/decodes a Static call to it. That counterexample passed and is recorded
in raw stdout. The fixture is synthetic actual VIR, not a claim that this exact
monomorphic trait declaration was emitted by an authenticated frontend.
Other nonstatic definition kinds are equally unchecked by inspection.

Require an actual static ordinary function definition at Context construction,
and add a negative regression for nonstatic definitions. Review return-mode and
const/static-item restrictions at the same boundary. The explicit static-only
profile must not depend on an undocumented caller prefilter.

### R2 — P2: resolved self-target is ignored (static finding, not end-to-end reproduced)

Candidate lines 103-107 discard CallTargetKind and compare only the nominal
function name to the current function. For DynamicResolved calls the actual
callee is `resolved`; normal existing owned paths compare that target instead
(parent `source/vir/src/infer/owned.rs:122`, `:1476`, and `:2485`). Thus this helper
does not reject a resolved self-call whose nominal trait-method name differs.
The existing source signature gate permits certain monomorphic From implementations,
so nonstatic current-source functions are not universally ruled out.

The reviewer attempted an actual Context probe with a recursive From source,
but its retained Envelope return fixture failed the earlier owned-source gate.
The test failed at `reviewer_tests.rs:164`; it is NOT evidence of an accepted
Context or an end-to-end recursion bypass. A simple eligible value-enum-returning
From source is a possible follow-up fixture, not an executed result.

Normalize resolved targets before the direct-self comparison, or conservatively
reject nonstatic/dynamic source calls in this restricted profile. A later parent
test must demonstrate both eligibility of its fixture and the desired refusal.
General transitive call-graph recursion remains the already disclosed limitation.

### R3 — P2: memoized type lookup can bypass the advertised depth bound (static finding)

Candidate `add_type` checks `active.len() >= 12` at line 276, but returns for an
already interned type at line 279 without accounting for that type's own depth.
Consequently a previously accepted type can be wrapped by one new Option type
at a time; each subsequent traversal stops at the known child. The combined
structural depth can exceed 12 while every fresh traversal has depth one or two.
Multiple approved observation signatures are processed sequentially and can
supply such a sequence inside Context construction, not merely through test-only
direct helper calls. True structural cycles still appear to be rejected because
types are only interned after their children complete.

This is a static control-flow finding, not an executed counterexample in the
single reviewer build. It contradicts a context-wide structural-depth guarantee,
but does not let untrusted bytes inject an arbitrary new type: all slots still
come from current caller-supplied source. Cache structural heights and check
combined height on reuse, or independently validate each root's bounded type
depth. Preserve the existing type-count and cycle checks.

## Independent tests: exactly one actual-VIR harness build

Command and complete sanitized child environment: `logs/cargo-launch.json`.
The command is offline and locked, `-p vir --lib`, focused on
`infer::owned::candidate_data::tests`, jobs 2, incremental disabled, debug info
disabled, test threads 1. Only the authorized library test harness executes.
No compiler/verifier/vstd/runtime executable build or solver/native application
run occurred. Cargo used existing local dependencies without copying caches or
toolchains. The source workspace contains only air, vir, and vir_macros.

**Actual Cargo child exit: 101. Compilation succeeded; tests: 13 passed,
2 failed, 192 filtered out. No retry or second build occurred.**

All nine author tests passed again. Passing reviewer tests add actual typed
positive coverage for Some/None, variant tests, nested record/Result/tuple
projections including the second error-tuple field, conditionals, arithmetic,
bitwise/unary/clip/logical expressions, signed-byte boundaries, and Result-taint
rejection underneath a precondition conditional. Exact fixture Debug round trips
and deterministic re-encoding are checked. The R1 counterexample is the fourth
passing reviewer test; its pass demonstrates the unwanted acceptance.

The two reviewer failures are accurately retained:

1. `positive_value_enum_construction_and_variant` fails Context construction
   with `ineligible owned source` at reviewer line 100. Reusing the nested-Result
   Envelope shape as enum payloads does not produce a valid positive fixture for
   the existing owned layout/value-type gate. No successful enum codec branch
   coverage is claimed, and this is not established as a candidate regression.
2. The R2 recursive From probe fails its Context-acceptance assertion as described
   above. Its name expresses the attempted probe, not an observed acceptance.

The reviewer-only source modification is precisely three lines adding a nested
test module under the existing candidate tests. Candidate codec semantics are
unchanged; the external reviewer file supplies six additional tests. The tested
source manifest was captured before Cargo launch and must remain immutable.
That manifest covers 190 files under the copied source workspace. The external
reviewer_tests.rs file is outside that tree: its unchanged contents are retained
by the creation transcript and final sealed payload hash, not by a separate
prelaunch manifest entry. No reviewer test source was edited after the build.

The author's original command remains separately recorded as outer wrapper exit
1 after sandbox-denied `time` sysctl. Its Cargo child exit was not captured.
This review does not relabel that command as exit 0.

## Invariant review and remaining limits

The flat wire is genuinely data-only. Strictly backward edges, eight-child
limits, checked expanded-size accumulation, 32-node expression depth, 2048 table
nodes, 4096 expanded nodes per expression, bounded input bytes and current types,
root checking, and orphan rejection bound untrusted expression reconstruction.
Serde default recursion limits remain enabled. Canonical reserialization rejects
whitespace/key-order/alternate integer spellings and ignored input. Exact integer
range checks reject machine-word literals instead of assuming a target width.

Every accepted node has a typed reconstruction branch. Current signatures and
datatype shapes supply identifiers/types, constructor/variant/field layouts,
and approved observation signatures. Result dependency propagates through every
child edge and is refused in preconditions. Complete owned equality is retained;
the author's actual nested-error fixture passed again. Arbitrary typed AST metadata
is not promised byte-identical: spans are intentionally rebound, and the existing
types_equal helper ignores datatype ImplPaths. That does not establish missing
owned fields or lost error payloads.

The wire has no accepted/proved bits, solver outcomes, certificates, executable
bodies, or authority. Strict unknown-field checks reject authority injections.
Well-typed but false proposals can decode by design; decoding proves no contract.

Schema/key/dependency mismatches fail closed. Keys include full Function Debug,
datatype/observation definitions, sorted dependency keys, and all supplied policy
fields; output/export paths are not stripped. This is intentionally conservative,
not a stable cross-run source/compiler receipt. The trusted caller still must
supply authentic complete compiler identity, configuration/features/options,
target/architecture, trust/model policy, and transitive dependency identities.
Nonempty arbitrary strings are not authentication or completeness checks.

Actual retained hex130 candidate correspondence is still UNTESTED. Value-enum
positive coverage remains open. Neither fixture equality nor this review proves
universal semantic exactness or full actual-hex formula representability.

Every hypothetical future load still requires THREE FRESH certificates:
soundness, safe-domain maximality, and output exactness, against current source
and through unchanged installation/body-preservation gates. No certification,
export, admission, persistence, primitive axiom, producer/runtime/trust/S0 adoption,
or path stripping is enabled here. Warm-cache gate UNMET; all 18 redb cells OPEN.
Excluded growth049, zero-growth050, mutation030, refused-padding95, and
drop-pipeline work was neither resumed nor rerouted.

## Authentication and dependency accounting

Entry authenticated all 194 INPUT_MANIFEST identities against current files,
all 194 TESTED_MANIFEST records including installed tool binaries and the retained
test harness, all 24 OUTPUT_MANIFEST payloads, and all 25 archive members byte
for byte. No archive extraction or Git checkout was manufactured. No scoped
AGENTS.md existed in the inspected ancestor/source/evidence/candidate/scratch
scopes. Parent and candidate were never edited.

Trusted entry hashes matched:

- OUTPUT_MANIFEST: 40c5c029b16088004c519d84c97fe9fcdc979560d7d6fcdee9b397a5ba3348a2
- candidate.patch: 9fec2bce61c0bbe6d7e7f51ac3d13ac94c25e96ab49860cf98a79bc92490a4ab
- REPORT: 040de0a0ba809f185ae54d6f76ddaf08eb3ff492dd4ea323d11ad660bd6426a5
- retained archive: 1c3593aa605890b3240965a9e8e567fb258d74c187471d406c88632896834eac

Before reviewer edits, all 190 copied workspace/source files matched actual
candidate tested-source identities. Parent owned.rs matches the retained baseline;
the candidate patch adds only the private cfg(test) module declaration and new
codec file. The only workspace harness delta is the narrowed member list.

Original lock hash: d7027da39db272c32cedb76629d812879719e129bd2a9091ebc1166e89a71190
(393 package records). Candidate resolved/reviewer lock hash:
f2d91d6ad4720090566e03e54fa898c9fc7333b0b5528223cb7292c93f775d77
(139 package records). All 139 retained name/version/source/checksum identities
exist in the pinned original; 254 identities are omitted, zero added. Twenty
retained full package stanzas differ, including dependency-reference formatting
and/or resolved dependency lists; this is not a claim of lockfile byte equality
or a completely identical full-workspace feature graph. Reviewer uses that exact
resolved lock with --locked. Full identity comparison: `lock-comparison.json`.

Build elapsed: 33.600 seconds (Cargo compilation reports 32.11 seconds).
Maximum observed scratch sample: 351272 KiB; final build scratch: 330160 KiB.
Minimum sampled global free disk: 49644937216 bytes, above 16 GiB throughout
sampling. One-second samples do not claim a hard filesystem quota or peak RSS.
Scratch remained far below 3 GiB; no metrics wrapper masks the direct child exit.

Exit authentication, tested-source reauthentication, final disk/time accounting,
review payload hashes, and the sealed review archive are emitted by seal_review.py.
