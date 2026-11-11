# Independent Stage1 local NonZero Ord review

**REQUEST CHANGES: F1, dual selected output invalidation; F2, embedded type dependency coverage.** Reviewed prepared
`7fde950cf87f67c2fe755c4f5f1e96f358436a37` in the review's own isolated worktree.
No implementation edits or remote publication. The exact carrier/getter/actual
Function route has substantial independent source and fresh ordinary validation;
no semantic false-success was found. A further required full-Function dependency
coverage gap is confirmed synthetically (F2). The mandatory all-selected
output requirement is not met when explicit and Cargo destinations coexist.

## F1 — remove both selected destinations on failure (blocking)

Locations: `source/rust_verify/src/import_export.rs:291` and `:361`, reused by
`source/rust_verify/src/verifier.rs:2913`'s new local-only barrier.

The invalidator calls `export_path`, which chooses
`args.export.as_ref().map(PathBuf::from).or_else(|| via_cargo.clone())`.
When Cargo supplied its metadata sibling and `--export` also supplies another
path, only the explicit path is removed. The normal local-only diagnostic is
issued, but Cargo's stale `.vir` remains.

Independently reproduced twice, including a genuine fresh same-runtime Bool
Cargo artifact, not only a dummy sentinel. The prior Bool provider checks2/0
and produces schema39/policy73 metadata. The next invocation uses the original
direct Ord caller, Cargo mode and an additional explicit export path; it rejects
with “NonZero Ord factoring is local-only; selected VIR export is unsupported”.
Explicit output disappears; the valid old Cargo interface survives byte-identical:
`f040dd4531604ac2b3cdccbd198bfd87510d02e185d5dd7de34aba78b854d393`.
No inference that this is a new ordering certificate or that a consumer accepted
it is made. It is a failed invalidation of a genuine previously valid artifact.
All selected leftovers from this control were removed after recording evidence.

Private receipts, all under `/tmp/local-nonzero-ord-09t1tshb`:

- `review-dual-valid-prior/receipt.json`:
  `4d0f08c564b3a39236b03c125d09e87711f6746660820a9e13f1d2cfdf90ad74`.
- `review-dual-valid-refusal/receipt.json`:
  `6a13915f6929ed89c50f900bbd19b2341a8b48a5ed0ee5321fef353be9cd461e`.
- `dual-valid-result.json`, `dual-valid-retained-prior.vir` preserve the compact
  state record and private previous artifact. `dual_valid.py` is the reproducer.

This utility predates the feature, so this is not claimed as a newly introduced
regression. The new path nevertheless depends on it to satisfy an explicit hard
requirement. Recommended parent fix: enumerate both distinct destinations during
invalidation, deleting each or returning an error; keep destination selection
for actual export separate. Recheck dual-path refusal with a fresh valid prior
artifact, both path orders/identity as relevant, and existing single-route
invalidation controls. Do not weaken the local-only barrier or add an Ord format.
No implementation fix was made in this review; parent owns the fix/re-review.

## F2 — validate embedded implementation paths in actual VIR types

Locations: `checked_nonzero_ord.rs:885–903,937–997,1020` uses `types_equal` for
parameter/local/read/contract types; `vir/src/ast_util.rs:153` explicitly ignores
Datatype ImplPaths. These are structured dependency witnesses, not source spans.
CallTarget and explicit extra-dependency comparisons are complete, but the type
comparisons silently omit this additional field.

Independent actual-loaded-compiler control: clone the real constructed Function,
retain the first shared parameter's complete NonZero<u64> datatype/arguments and
reference decoration, and clear only its previously nonempty embedded impl paths.
`check_ordering_function` still returns Ok. The control exits normally1/0 and
prints `REVIEW_EMBEDDED_TYPE_DEPENDENCY_CONTROL accepted=true`; no panic or fake
serialized descriptor supplies this result. It runs after the ordinary original
Function passes, against the same actual loaded MIR/model bindings.

Private result: `type-dependency-control-result.json`; build receipt
`6e61a95a8d9a7c6718d0b16a7024b823b4bb3c760929ba48bf65c621ee974479`;
run receipt `f456119657332d0cfa439978d792f4c2a9e5acab171cb3515eb615577213dabd`.
Reproducer: `type_dependency_control.py`. Only cfg(test) code in a separate own
private source snapshot changed. No production source or frozen positive-proof
runtime changed; no core/toolchain patch. The executed synthetic variant is
recorded separately from original-source proofs and the original seven units.

This is a required field-complete checker coverage defect, not evidence that a
Rust source false numeric property verifies. `types_equal` intentionally provides
semantic type equality elsewhere in VIR, so its behavior is not itself a generic
verifier bug. Here the design specifically requires independent complete typed
dependency correspondence. Parent should compare or independently validate the
embedded paths recursively for all checked types, or establish explicit checked
irrelevance where appropriate; add independent controls for removed/changed/type
nested dependency fields at both validation boundaries. Existing exact call
model/dependency comparison must remain. No implementation fix was made here.

## Independent correctness review

[SOURCE_REVIEW.md](SOURCE_REVIEW.md) records the detailed field/trace analysis.
The closed checker inspects entire normalized MIR for real core Ord/NonZero/get
and primitive u64 instances, typed getter formal/result substitution, ordered
storage/Copy/borrow operations and original primitive call/unwind. Transmute
stays uninterpreted on both sides. Shared arguments may alias; fresh genuine
copies are not logical representations fabricated from integers. Concrete types
and signatures, not generic-origin flags or inline labels, authorize the route.

The actual full Function validator independently parses every FunctionX field,
all binders/reads/IDs/statements/arguments/model targets/effects/contract and
whole body, with the embedded type-path exception F2. It runs before registration and after ordinary WF/autospec/mode
processing. An independently compiled ordinary get/get/u64cmp mirror provides
identical typed call targets; its implicit-reference/declaration differences are
explained against ordinary frontend code. It is diagnostic only, never the proof
of the original carrier. The real original std caller's body remains unchanged.

Source call binding retains actual caller/HIR/expression/argument identities and
re-resolves the concrete instance at redirect. The hook retains original argument
translation/evaluation. Independently authored mixed generic/concrete contexts,
multiple callers/aliases/reversed calls and effectful argument expressions pass;
a false effect assertion fails with the helper healthy. Existing ignored genuine
wrapper derives do not activate this path. Genuine wrapper ordering stays unproved.

Existing imported get/spec_get/opaque View and actual u64 Ord contracts are used.
The actual u64 obeys predicate is true and numeric unsigned cmp_spec is inherited
from unchanged vstd source; the helper proves its authored exhaustive numeric
relation using these models. No new StdModel admission, NonZeroOrdSpecImpl,
representation/view law, primitiveCmp lowering, std/vstd edit or trust stub.
Normal BodyBackedForeign Query and the successful fresh Normal helper-query set
remain required. AIR hashes are diagnostic observations of those invocations,
not a replacement for the inherited health gate.

## Fresh validation and exact classifications

Own offline locked Rust1.99.1 jobs2 rustup-first build; own matching macros/vstd;
eight tools frozen before source proofs. All positive providers/ordinary consumers
use that same runtime. Historical VIR was never a positive input. Exact receipts,
source/compiler/core/tool identities and typed/query diagnostic hashes are in
[RESULTS.json](RESULTS.json) and [SOURCE_AUTHENTICATION.json](SOURCE_AUTHENTICATION.json).
The latter rebinds old design source through own Git and authenticates exactly
168 sealed files/7 MIR sections; it does not rerun a historical full-source audit
against bumped source. Current epochs are schema39/policy73/cache76; worker
72/75 records remain historical. Same-runtime fresh Bool schema39 works, and a
synthetic old-policy metadata compatibility input refuses before verification.

- Original direct numeric std caller: exact same-invocation cold/warm2/0 each,
  same actual helper Function/query hashes; all three relations2/0, aliases2/0.
- Boundaries11/0 are supporting observations through MAX, not the domain proof.
  Helper/direct contract has no restricted-domain precondition or 61-bit premise.
- Goals off0/1; unchanged direct PartialOrd5/0. False/swapped callers1/1 each,
  with only `local_order::total` failed and the required helper normal body healthy.
  Weak true2/0 certifies only that weaker caller property.
- Mixed generic/concrete caller-context control6/0; effectful original argument
  expressions2/0; false effect assertion1/1. Ordinary mirror1/0, no concrete helper.
- Local Ord auxiliary10/1, `failed_functions=[]`, no selected export, helper
  healthy. It separately exercises global rejection, unlike pre-proof Ord export
  refusals. Bool auxiliary10/1 and late erased Rust2/0 then E0308, each on explicit
  and Cargo routes, remove selected outputs.
- Single-route refusal matrix24: direct/false/mixed/private-newhelper crates;
  valid noverify/nolifetime/notrait/noerasure without compile; valid function and
  module filters; existing Bool auxiliary and late erasure failures. F1 is the
  additional simultaneous-two-destination counterexample.
- Bool provider2/0, ordinary goals-off consumer1/0, false consumer0/1, all on this
  runtime. Unchanged exact proofless Item132/0 and packet31/0 export locally;
  excluded wrapper Ord is not selected or credited. Eq ignored-Ord control6/0.
  Original genuine wrapper clients stay2/6, no concrete carrier helper acquired.
- Function value is frontend unsupported, not a semantic0/1. Generic/lookalike/
  overloaded-deref weak properties are ordinary-only, no numeric helper acquired.
- Matching vstd2250/0; requested VIR149/0 and compiler library7/0 (includes loaded
  actual MIR/Function synthetic structural unit). Focused40/0 passes (3+5+4+12+3+13), recorded
  by the final audit. A separate1/0 synthetic actual-Function clone control
  confirms F2; it is not part of the original lib7 or a semantic proof. No broad717/compiler-Python148/canonical/observer/feature
  matrix result is claimed here; those are parent-owned.

No panic, timeout or resource failure is credited as evidence. Structural unit
mutations are synthetic typed-checker controls; false source postconditions are
semantic controls; function values/old policy/disabled checking are frontend or
compatibility refusals. Partial-filter CLI forms and noerasure use valid options.

## Trust, manual inputs, assumptions and limits

The bounded translator/checker, including the reviewed gaps F1/F2, joins the existing compiler TCB. It is not a
kernel theorem or an expansion of std axioms. Inherited trust remains Rust→MIR,
normal VIR shared-copy/reference handling and transformations, imported opaque
View/get/u64Ord models, logical encoding/solver and final erased Rust compiler.
Manual inputs are the design acceptance conditions, authored numeric relation,
compiler implementation/model routing and review/controllers/classification.
Hashes bind bytes and invalidation identity, not independent proof objects.

Assumptions include the pinned compiler/core and matching fresh vstd identities,
ordinary verifier/model pipeline and unmodified source inputs. Native/source
scope and symbol checks exclude the synthetic helper as executable/native; original
std cmp remains in a separate rustc-only macro-erasure diagnostic (no native proof). Native correspondence=0. No wrapper ordering/import,
lookup/native/provenance/borrow-frame theorem or whole-goal acceptance follows.
Canonical6/0, HMAC1/1 UNPROVED, native0 and 44+3/101 OPEN are inherited, not rerun.
`publication=false`, `goal_complete=false` remain.

## Process and local artifacts

Preregistration local commit `56210c34` records the plan before positive source
proofs. Build raw artifacts/logs/source copies remain in the private root above.
Only this new evidence directory is changed. No subdelegates, network, push, PR,
publication command or publication approval request. No active parent build,
source, cache, configuration, state, replay or process paths were accessed.
Only named completed authenticated Cargo/solver inputs, exact selected policy pin
source files, sealed baseline/MIR files and shared pinned compiler files were read.

Excluded setup issues: sandbox initialization failed before the first shell;
subsequent commands used approved escalation. Initial local evidence commit lacked
Git identity; corrected with command-scoped identity, no config writes. The copied
freeze controller's default vstd receipt label was corrected before proofs. The
new source audit initially expected only production+two docs changes relative to
a3b1e280 and rejected additional worker-report documentation; corrected to assert
the exact two production epoch files, with the full source diff recorded. No
semantic case, acceptance count, source fixture or checker was weakened to pass.
