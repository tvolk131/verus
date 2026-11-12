# F1 implementation checkpoint — admission only, native stage inactive

F1 is implemented and focused tests pass. All three unchanged frozen bodies
pass typed local-unit admission in ordinary and Infer, then fail at the explicit
later-phase inactive gate. No representation, native checked Function, field
capability, client state diagnostic, body proof or exported fact is enabled.
F1 is not independently reviewed/accepted; the full native stage remains
INACTIVE/UNACCEPTED and has zero proof credit.

Scoped commits: 5d2bf3ff freezes the interface/expected outcomes before trials;
1d4bd257 implements the gate, shares the existing header-erasure identity test,
and incorporates the exact R1 hunks from immutable a360518d73dd62da035a0ab55a8d38273270b093;
b5a175f6 adds tests, scoped runtime guards and extra-input support for existing
execution helpers. The final evidence commit records this report/results.
Only immutable 4515419e findings and the routed R1 commit were read. Root's
routed 66146252 preflight review does not review this F1 implementation.

The new checked_owned_cell_admission.rs contains a private, nonserializable
AdmittedUnit inventory of actual native/local DefIds, visited body owners and
canonical header call/closure/type-only/predicate identities. Its only consumer
returns the later-phase error; it is explicitly not a proof or field authority.
Native metadata/signature/typed-MIR preflight remains first. Existing early
option/export refusal is unchanged and still runs before the new gate.
Everything here runs before VIR imports, proof selection/pruning and Infer cache
reuse, independently of helper counts and --infer-goals.

Admission uses rustc's whole-crate item and body inventories, including nested
owners, and independently visits source types. It rejects unsupported local
ADTs/traits/foreign items/statics/opaque/projection forms and explicit external,
trusted or skipped bodies/ancestors. Every admitted local function/constant is
checked for verification awareness, region-only generic parameters/bounds,
normalized signatures and complete typed expressions/patterns/locals/adjustments.
All body/nested owners must be visited. Native/unverified and unknown bodies
cannot disappear through frontend external filtering. Actual local function and
constant dependencies must be acyclic.

The executable type grammar is S=unit/bool/u64/recursive scalar tuples, or actual
native C=UnsafeCell<u64>, &mut C, &mut u64. Alias/reexport identity is resolved by
rustc; source associated projections are separately forbidden so normalization
cannot hide them. Spec expressions, binders and semantic captures must be S.
Every executable call resolves freshly to an original native3 instance or an
admitted local of the right mode. Indirect, imported, abstract, virtual,
callback and other native calls reject. Unknown expressions/adjustments/patterns
reject. Casts, unsafe blocks, raw/shared references, executable function values/
closures/coroutines, containers, drop-bearing types, inline assembly and unknown
unwind operations receive no exception. Primitive arithmetic still needs the
ordinary safety queries in a future enabled stage; admission is not a safety,
termination, lifetime, WF, mode, erasure or body proof. The closed expression
subset is deliberately conservative (for example loops, arrays outside canonical
headers and arbitrary callback-shaped spec syntax are unsupported).

Canonical header admission is separate. It resolves actual nonlocal builtin
diagnostic identities in the loaded builtin crate, validates declaration ABI/
arity and exact ensures/requires shapes, and checks bool arrays/result binders.
A constrained result must use the exact builtin constrain_type with the original
direct self target and original parameter places. Only that validated type-only
subtree is excluded from call/capture analysis. Real predicates are recursively
scanned, including blocks, scalar spec helpers, quantifier binders/bodies and
captures. A C capture only in the erased inference self-call does not invalidate
the frozen scalar postcondition; a C capture in a real predicate does reject.
Executable captures/recursion, fake headers and macro expansions have no exemption.

fn_call_to_vir::is_erased_constrain_type factors the existing actual identity
predicate. Both admission and extract_ensures's existing recorder use it. The
normal MiscEraseAbsolutely and BodyErasure behavior is unchanged; F1 does not
perform executable erasure or introduce a second erasure mapping. Default-off
ordinary/Infer scalar --compile positive/false-return controls pass. No compiler
field privilege, owner-module forgery, public live selector or assumed law exists.

Final focused results (not native body acceptance):

* Nine positive source families in both modes, plus two additional frozen-trio
  repetitions: alias/acyclic locals, explicit exclusive lifetime helper, acyclic
  scalar spec helper, scalar tuple, scalar quantifier, requires, nested checked
  local and user macro. Frozen inventory is exactly 3 local functions, 6 visited
  bodies including header closures and 3 canonical headers.
* 35 typed negatives, 8 macro/header/skipped-item negatives and 5 imported
  direct/generic/abstract/dyn/projection negatives, all in both modes. The full
  F1 matrix makes116 compiler invocations. The imported native mutator can
  legally write through &UnsafeCell; its client asks for the false constructor
  frame and is refused by admission before contracts can frame anything.
* Every new admission test asserts failure JSON, empty function/helper inventory,
  zero checked exec/proof bodies and no written interface. Positive admission
  reaches the exact later inactive message; negative admission never reaches it.
* All 5 inherited refusal/default-off/export tests pass (32 compiler invocations),
  alongside the 4 new F1 test groups. The final strengthened zero-proof-inventory
  rerun passes all 4 groups. These are focused suites only.
* The actual native unit passes after R1: 54 MIR corruption controls plus 4
  signature/region controls =58, including terminator=None for each native method.
  This is runtime validation of the routed patch, not independent review and not
  a checked native method contract.

All old raw receipts/logs and old REPORT/RESULTS are retained unchanged. Their
historical /tmp/owned-cell-implementation-20261112/bundle is mutable continuing
build output. Use the immutable relocated eight-tool snapshot
/tmp/owned-cell-preflight-6e5fb54d-parent-snapshot for the 6e5fb54d identity.
Its receipt/eight hashes were authenticated and guarded; it was never modified.
New builds/trials use only F1- labels. The new private rust_verify identity is in
F1_RESULTS.json. The other seven artifacts exactly match that snapshot, including
its fully verified vstd2257/0. There was no type/schema/library/macro/vstd source
change, so no full vstd rebuild was run. No accepted78 substitution, shared cache
write, new dependency, global schema39/policy78/cache81 or trusted-registry bump,
new evidence/security subsystem, delegation, sibling/live-root read, push or PR.

Costs/failed attempts are exact in F1_RESULTS.json and the single private raw
log set under /tmp/owned-cell-implementation-20261112/F1-*. All18 command receipts
sum277.151 wall seconds; 11 compiler build attempts total164.318s. The native
unit command takes52.955s including test compilation; its actual test runs0.08s.
The combined focused command takes41.379s; the strengthened F1-only command6.461s.
These are command wall times, not CPU or dollar costs. No monetary meter exists.
Start is2026-11-12 15:26:47 UTC; freeze/end and conservative charged time are in
F1_RESULTS.json. Real builds are excluded if needed to bound active time.

Retained failures: pinned API mismatches (normalization wrapper, autoref, FnDef
binder, Const kind, HIR AmbigArg, clause-query rename), an intermediate inventory
formatting error, and test fixture/diagnostic errors. Three initial F1 matrix
runs failed: Rust-2018 panic formatting hid the first diagnostics; ghost parameter/
return syntax and macro-transformed manually written headers were corrected to
valid typed fixtures; generic diagnostics were prioritized ahead of implicit
trait bounds. No negative was credited merely for a parser failure, and no
accepted escaping program or false body theorem was observed. Old failures/logs
were not overwritten and no broad unrelated suite was run.

Gate status: gate1 historical authentication holds; gate2 native preflight/R1
holds but actual emitted-Function correspondence remains pending; gate3 F1 is
implemented/tested on the closed unit and awaits independent production review.
Gates4/6/8 remain unimplemented: native checked body lowering/contracts,
independent actual Function/place/type/read/writeback validation, F2's distinct
same-client-body private final-input-state goal (arbitrary entry, final=next,
false final=old and store/writeback deletion while returning next), and F3's
original-name native erasure-mode support. Gate5 option refusal/default-off
regressions and gate7 early export/import refusal hold; they do not replace the
remaining normal native proof/lifetime/privacy/corruption/import/cache/export
and erased-call acceptance gates. No descriptor/authority API was activated.

Full Mutex construction including atomic Transmute, shared acquisition/guard/
poison/drop, LRUWriteCache and the complete REDB crash theorem remain UNPROVED.
Native bridge, source coupling, CP1 and CP2 remain false. HMAC/native remains
DEFERRED AND UNACHIEVED. Admission counts are no progress credit for that theorem.
