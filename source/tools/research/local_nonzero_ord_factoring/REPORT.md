# Stage1 local checked NonZero<u64> Ord carrier

Implementation and local ordinary proof; independent fresh parent review required.
No ordering certificate, import format, wrapper profile extension or publication.

## Result and exact scope

The unmodified frozen `a.cmp(b)` caller on two arbitrary borrowed genuine
`NonZero<u64>` values proves its numeric Ordering contract with true requires.
Final ordinary total is 2/0: ONE actual body-backed logical helper plus ONE original
caller. Getter1/getter2 and actual nested u64 Ord remain ordinary modeled calls.
The helper has no precondition, including no 61-bit premise. The two shared
arguments may alias; no fabricated logical NonZero or representation/view law
was introduced. Full `Less/Equal/Greater` equivalences also check 2/0. Ten ordinary
boundary observations (both orders, 1/equal/2^61/highbit/MAX-1/MAX) plus the same
helper check 11/0; these are supporting samples, not the domain theorem.

Literal identical-invocation cold/warm runs both prove 2/0 with identical source,
eight tools, environment, complete actual Function log and normal helper query.
Helper query SHA256 `3eceee37382c804d0cf0ef17ccda5c0438a073e16246ce3e150baf32b4e2f809`;
actual full Function log SHA256
`08e303f1af845f40ab45af0a2f0f77f46da47fee1ba18a2820d919aa433234f2`.
Each run issues a fresh ordinary body query. There is no ordering candidate-cache
kind or cache-hit claim. Earlier cold/warm runs with different log paths are kept
separately and are not the identical-invocation acceptance pair.

Goals off reproduces Ord0/1. Existing direct PartialOrd remains5/0. False and
swapped caller contracts each reject named `local_order::total`, 1/1, with the
exact helper body healthy. A weak true caller checks only that property, 2/0;
it does not earn an exact numeric caller contract or any ordering certificate.
Alias caller checks2/0 over all genuine inputs, including identical addresses.

Genuine BorTag Ord/PartialOrd and original wrapper clients remain UNPROVED.
Macro/profile eligibility, seven original derives, private field, closed
`value=self.0@`, and ONE existing checked Eq/value broadcast lemma + ordinary
client import are unchanged. No native refinement, lookup theorem, source-wide
ordering import or full-goal completion follows.

## Implementation and correspondence inventory

`checked_nonzero_ord.rs` is a separate closed path. The bool semantic resolver,
single-cast recognizer, one-input Function identity hashing, certificates and
imports remain byte-identical to basefc0479a6. The preregistered binding-struct
sketch was replaced by direct Result-returning revalidation of the actual Function;
there is no unused proof token or serialized ordering evidence.

* Source binding: actual direct HIR call/caller/argument expression identities,
  source Expr identity, checked receiver adjustments and concrete Instance are
  bound and re-resolved on final call mapping. Only builtin shared adjustments
  are eligible. Function values, coercions and overloaded dereferences receive
  no redirect. Semantic core Ord/NonZero diagnostic identities are used; the
  comparison Ordering ADT is taken from actual Ord::cmp's signature (the diagnostic
  item named Ordering denotes the atomic enum and was deliberately not used).
* Actual carrier: nine typed locals, two entire blocks, fourteen ordered entry
  statements, original primitive call and unwind Continue, four ordered storage
  deaths, Return. All body properties, statements, projections, operands, constants,
  mentioned targets, cleanup/unreachable blocks and local Copy/no-drop types enter
  the closed check. Callee constant is a literal zero-sized FnDef item, not an
  unevaluated constant. Generic-origin flags remain true for carrier/get; concrete
  signatures, substituted types/targets/consts and scoped reference handling are
  checked independently. The bool flag check is unchanged.
* Actual getter: two locals, one Copy/Transmute, Return, no hidden call/effect/exit.
  Exact typed formal1/result0 maps are (5,4)/(8,7), at carrier indices4/11. Transmute
  remains uninterpreted on both sides. Original copies from *arg1/*arg2 at3/10,
  borrow sites and remaining storage/call order are preserved. The exact trace
  establishes initialization/live/no-escape obligations: copied genuine NZ temps
  are never borrowed/observed; u64 locals remain live through the primitive call;
  all fresh temps are Copy/no-drop, with no address comparison or mutation.
* Primitive boundary: actual nested resolved core u64 Ord instance, signature,
  all five local types and entire seven-statement one-block MIR are inspected.
  Cmp is a dependency observation only, not a new encoding/proof. The ordinary
  resolved-trait-call form retains the trait declaration's unwind and invariant
  contracts. No stronger no-unwind fact is introduced for the primitive call.
* Actual full VIR Function: two ordered shared NZ params, distinct return, six
  fresh local binders mapped injectively to MIR locals; six ordered initializers
  (genuine copy/get/shared borrow twice), then actual primitive resolved trait
  call. All read types/kinds/IDs, argument/target/dependency order, signature,
  return, empty requires, exact exhaustive numeric ensures, attributes, visibility,
  effects and complete body shape are independently parsed against original MIR.
  Checks run both before registration and after normal WF/autospec/mode processing.
  This is not a hash or descriptor roundtrip. Read IDs are unique; map bounds and
  binder collisions reject.
* Ordinary pipeline: BodyBackedForeign feeds normal Query, WF/mode, trait/lifetime,
  termination, helper and selected application body checks. Existing fresh normal
  helper-query identity gate applies to every new helper before local success.
  No application proof block/function, theorem call, snapshot or replacement
  executable helper is inserted. Final erased Rust checks and compilation run.

## Existing model closure, trust and manual inputs

Semantic actual core get/primitive IDs map to their existing imported vstd proxy
Functions, not invented stubs. Get's existing autospec binds spec_get, its actual
View call, and inherited opaque NonZero View. Core Ord's external-trait-extension
metadata binds actual unsigned u64 numeric cmp_spec and obeys_cmp_spec=true.
Ordinary import normalizes authored OrdSpecImpl methods to OrdSpec; that metadata
identity is used. Model call type/trait dependencies use the same existing typed
frontend API as ordinary source calls. View/obeys/cmp definitions are additionally
rooted in the helper's explicit dependencies before pruning. Whole matching vstd
bytes and compiler/core byte identities are inventoried in RESULTS/private freezes.
No new StdModel admission, assumed NonZeroOrdSpecImpl, std/core/vstd edit, assumed
transmute/view law or primitive enum/discriminant law exists.

TCB: the new bounded frontend/checker code joins existing compiler TCB; it is NOT
a kernel theorem. Existing Rust-to-instantiated-MIR, builtin shared-copy/reference
handling, VIR/logical encoding, inherited get/Ord models, solver and erased-Rust
compiler trust remain. Matching vstd and inherited View/get/primitive declarations
are existing trust, not ordinary new standard-body proofs. Diagnostic metadata
and hashes bind bytes and invalidate changes; they are not proof objects.

Manual inputs: the accepted design/conditions; authored numeric relation; this
bounded compiler translation/checker; typed model/dependency routing; private
source/test/receipt controllers and classification/audit/report work. No new
application semantic premise or manual application proof was added.

Native/source inventory: direct source remains exactly the frozen baseline text,
with original std cmp call. Selected original named body is `total`; synthetic
helper exists only in verified VIR. Native symbol and selected-function inventories
exclude the synthetic helper. These are source/compilation observations, not a
native correspondence theorem. Native=0.

## Validation and local-only barrier

The final own Rust1.99.1 compiler/macros and matching vstd were built offline,
locked, jobs2, rustup-first PATH from own source and explicitly authenticated copied
completed Cargo/solver inputs. Final eight tools were frozen before final source
proofs. Executing verifier SHA256:
`324cf33252e7d07df7b33ad680eb23988a48f59a95617955bff4265d9d259fcf`.
All positive providers/ordinary imports use this SAME new runtime. No historical
VIR is a positive input. Final-source rebuild confirms unchanged frozen production
bytes after cfg(test)-only structural-control additions. Raw sources/MIR/VIR/AIR,
binaries and logs remain private at PRIVATE_ROOT; compact hashes/receipts only here.

Successful newpath crates remain LOCAL ONLY. Before any private pruning or bool
certificate construction, all selected explicit/Cargo VIR outputs are invalidated
and refused. This covers success/failure, false caller, mixed bool/order and private
helper/public-unrelated forms. Full source/body/WF/mode/lifetime/trait/termination/
erasure gates remain. Valid noverify/nolifetime/notrait/noerasure and function/module
filters refuse selected outputs. No usable ordering interface exists to re-export.
The future explicit kind-dispatched model/certificate closure is parent-owned Stage2,
not a TODO bypass. Global schema39/policy72/cache75 are unchanged.

Final matching vstd2250/0; VIR149/0; compiler library7/0 (all6 existing + new structural
unit). Focused40 pass: certified_foreign_bool3, local_checked_foreign5, local_checked_packing4,
infer_goals12, infer_nonzero_derive3, constructor_function_values13; see RESULTS. Existing bool provider2/0,
ordinary goals-off imported client1/0, false imported client0/1 retain schema39 bool
certificates. Isolated bool auxiliary controls fail10/1 with failed_functions=[],
and late erased Rust controls have2/0 followed by E0308. Both explicit/Cargo routes
withhold selected stale outputs, preserving the existing global barrier.

Function-value source is frontend-unsupported, not a semantic negative. Generic,
user-lookalike and overloaded-deref sources can check ordinary weak properties
(1/0,3/0,3/0 respectively); none registers the concrete helper or earns its numeric
contract. Their classifications are retained separately in RESULTS. The original
unsupported-source controller wrongly expected the function-value frontend case
to be a semantic0/1; that expectation failure is retained, not reclassified as a
source proof.

Structural unit uses in-memory clones of actual compiler-loaded MIR and actual
constructed Function, never patched core/toolchain. It checks wrong maps/bounds,
width/sign, swapped/repeated reads, extra operations/unreachable block, cleanup,
read-after-dead/storage changes, argument order, changed return/unwind, wrong getter/
model type arguments, borrow kind, reused read IDs, enum variant, signature/contract/
attribute and whole-body changes. These are synthetic structural controls, not source
proofs or security/race experiments. Required ordinary helper health is checked
separately from semantic caller failures and frontend/setup failures.

## Exclusions and process record

Preregistration local commit4e2914f7 preceded implementation trials. Read-only inherited
integrity passed normal and optimized Python,168 sealed artifacts/7 MIR/11 baseline
cases. Initial shell sandbox failed before commands (bwrap loopback); subsequent
local shell work used approved escalation. Initial report-path typo was read-only.
Only own checkout/Git and explicitly named completed/pinned inputs were accessed;
no live parent/sibling source, target, build, state, cache or config, no network,
subagents, push/PR or publication approval request.

Retained failures: first two compiler builds had pinned-API mismatches; first unit
build used Args::new instead of Arc<ArgsX>; epoch01 model binding searched authored
OrdSpecImpl instead of normalized OrdSpec; epoch02 static trait-impl call triggered
an internal compiler invariant/panic. The panic is explicitly EXCLUDED, not a proof
or semantic negative; corrected ordinary resolved trait calls pass final validation.
Private model-decoder setup first used the wrong own artifact directory and then
mismatched serde-feature artifacts; corrected own artifacts decoded normally.

Two control CLI setups were invalid: verify-function omitted verify-root; explicit
no-erasure was combined with compile. They failed before the engine selected-output
gate, leaving only dummy stale sentinels. Sentinels were explicitly removed; invalid
runs are excluded. Valid corrected controls have distinct immutable labels and remove
selected outputs. No count/acceptance gate was weakened to pass a run. Earlier source
snapshots were reused for iterative builds; each run preserves its exact source hash
inventory, source fixture bytes, tools, raw logs and freeze/receipt. Only final own
source plus frozen tools constitute the delivered implementation/proof epoch.

Final audit is read-only by default, exception-gated under both Python -B and -O -B.
No assert-based Python acceptance. Raw receipts retain all failures separately.

Original global goal: canonical6/0 and HMAC1/1 UNPROVED are inherited, not rerun.
44 minimal+3 phase/101 effects remain OPEN; necessary shared assistance on BOTH full
targets is unconfirmed. Whole-goal publication=false;goal_complete=false. Fresh
independent parent review is required before any acceptance or future stage.
