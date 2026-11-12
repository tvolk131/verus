# D2 frozen milestone: private native body queries, user stage INACTIVE

D2 checks the THREE actual loaded native UnsafeCell<u64> method bodies in an
isolated ordinary compiler proof context. Three full positive body queries pass;
three fresh full-Function negative-contract queries fail as solver postconditions.
No native user helper/datatype is installed, no user call rewritten, no source
selector enabled and no native fact published/exported. EVERY experimental user
request still refuses. This is NOT native bridge/field/phase/user acceptance.

Scoped source milestone3eba70cc0 and runner e50581bd3 follow prospective
f458b28bf and negative-control refinement0fe3805d2. Four separate identical
root-code/test adoptions can be skipped by parent: 41cd47e8f from9cc0f65c;
1d01d6a37 from70a9c390; 469981eab fromef662ee3; 10cae0f24 fromc4df45e8.
Only those immutable Git diffs were read, not live root files, reviews or patches.
F1 and root metadata fixes are incorporated but NOT independently accepted here.

The closed method authority is nonserializable, not Clone, and never stored in
Function/FunctionStub, attributes, wire interfaces or caches. It pins the exact
current Krate, Datatype and full Function objects. Immutable Arc aliases do not
create a new grant; reconstruction/mutation/pruning requires a fresh native
metadata/signature/region/MIR acquisition and independent correspondence. Strong
ownership prevents pointer reuse from serving as an identity receipt. A different
contract/query kind cannot inherit the positive authority.

The only frontend acquisition path freshly validates the real loaded native lang
item, generic declaration, private field, !Sync/type metadata, native method
identities/signatures/regions and complete typed MIR, then compares the actual
current Function/types/places/read modes to it. The separate VIR factory checks
all Function attributes, signatures, real local owner/visibility, complete bodies
and exact scalar field/current/future relationships. Identities and descriptor
shape alone grant nothing. These compiler-internal factories/validators are NEW
TCB requiring independent production review; their compiler-only interface is
not a source authorization or a machine proof of source-to-MIR simulation.

Ordinary WF has an explicit exact-unit method entry with no_cheating=true and
no_verify=false. Source/private access remains denied at the ordinary entry.
It checks the actual descriptor object from WF lookup, not a cached side shape.
Helpers stay in their REAL local root module; none claims core's ownership.
Backend creation checks current Function attribute/descriptor maps and module.
A private backend binding retains the exact unit; all ordinary Function AST-to-SST
entries and whole-unit lowering recheck current objects. Stubs, copied bodies,
changed lookup descriptors, stale Function maps and reconstructed units refuse.
A metadata-only Ctx remains inert (needed for root's good/bad metadata tests):
without method authority it refuses any Function lowering. This preserves the
root fix's valid u64 metadata case without making a selector/body available.

Normal mode checking exposed two D1 proposal defects: old UnaryOpr field syntax
in contracts, then duplicate Temporary IDs. D2 uses the existing canonical spec
ReadPlace form and SpanContext fresh-ID allocator. No mode check was disabled.
Normal resolution adds an EvalAndResolve on get_mut with its ordinary generated
HasResolved on the EXACT original mutable input. Both independent validators
check that complete typed statement/input/read kind; it is retained in the full
Function queried. D2 does not insert a new assume, input equality, constructor
history or permission premise. This is existing normal resolution bookkeeping,
not a native summary; a different predicate/input/type/read refuses. Ordinary
simplification's canonical empty zero-tuple is the only extra datatype allowed.

The complete private query path uses initial/merged/final crate WF, full per-
Function WF, modes, resolution, simplification, ordinary pruning retaining all
three functions, fresh correspondence/authority, normal Ctx, AST-to-SST with
elaboration, poly conversion, ordinary full-body AIR queries and Z3. Native
requires are empty. Original native typed signatures/regions and normal rustc
analysis remain; no lifetime/trait/safety/erasure-disabling option was used.
Normal safety checking is enabled in the context. A unit proof does NOT discharge
the remaining original-user lifetime/erasure and source-to-emitted-client gates.

Exact original/manual/body inventory:

* Three original native definitions: new, into_inner, get_mut, concretized only
  at u64. Full Function computations are the real native nominal constructor,
  consumed native field read and exclusive native input-field reborrow.
* Four positive postconditions: new result.field=input; into_inner result=
  input.field; get_mut current(result)=current(input).field and future(input).
  field=future(result). Three complete ordinary body queries pass, all u64 values.
* Three separate diagnostic Function objects reuse those same full bodies and
  negate ONLY each first canonical equality; get_mut's future relation stays.
  Fresh diagnostic authority is distinct from normal authority. Three complete
  ordinary queries reject as Invalid postconditions, not frontend failures.
* Source clients owned_roundtrip, exclusive_update and exclusive_borrow are
  unchanged and remain uninstalled/unproved. ZERO source client body proofs,
  ZERO F2 state queries, ZERO original-name F3 erasure entries or enabled user
  helpers. No PCell/token/world-state premise or runtime replacement exists.

Controls: native54 MIR+4 signature/region corruptions remain; inherited45 actual
emitted body/place/type/read/origin controls remain. New26 full Function/contract
corruptions include both current/future relationships; five independent normal
resolution predicate/input/type/read corruptions reject. Exact copied unit/
Function/lookup descriptor/stale backend map checks and normal privacy refusal
are exercised. Descriptor shape/type-argument/wire controls remain. Final focused
regression outcomes and exact receipts are in D2_RESULTS.json. Source/lifetime/
state/frame/erasure acceptance cases that still cannot reach user body checking
are NOT counted as semantic proof failures or completed bridge gates.

All immutable snapshots and old receipts/logs are preserved. The D1 snapshot
receipt is c15b5585f14cb9c5441b25b3667d3aa31ac34fed3ad37830a3edfdaacfcfe0ed;
all eight hashes and earlier snapshots are authenticated/guarded by the runner.
Only the private continuing bundle/cache is used; new D2 labels never overwrite
old trials. Seven noncompiler tools remain identical to D1, including inherited
vstd2257/0. No library/macro source or serialized native datatype changed and no
full vstd rebuild was run. The new compiler is a distinct private identity, never
an accepted78 replacement. config.rs remains in the cumulative authorized flag
inventory, unchanged this phase; default off and schema39/policy78/cache81 stay.

Retained failures: snapshot receipt-shape startup error (no command receipt),
ordinary field-expression mode failure; borrow-of-Datatype compile error; ENOSPC;
get_mut temporary-ID mode failure; and negative diagnostic-kind backend mismatch.
Cancelled fifth attempt has NO receipt and NO passed-build credit. Recovery
confirmed no surviving own build before restarting. Only own disposable11GiB
incremental cache was removed; snapshots, bundles and raw evidence were untouched.
CARGO_INCREMENTAL=0 is used for D2 with max2 jobs; no build tree copied. Disk is
checked before builds and must stay at least25GiB. No subdelegates, live sibling/
root reads, shared mutable outputs, remote actions, unrelated suites or new
security/evidence framework occurred. Costs/restart ledger are in D2_RESULTS.json.

First still-unproved application-facing obligation: full original client-body
lowering/correspondence and native call binding, including retained field store/
load and all normal checks. F2 is DISTINCT: exact same exclusive_borrow body and
original incoming &mut C, arbitrary initialized entry, final field=next; false
final=old and deletion of store/writeback while scalar result=next must fail.
Method authority cannot authorize that client. F3 original unresolved/resolved
native-name mode/erasure mapping and real original --compile remain unimplemented.
Complete whole-unit admitted-versus-translated coverage, native/client lifetime,
frame/state/privacy/import/export/cache gates and independent review remain
mandatory before ANY source activation. Positive unit queries do not close them.

Full Mutex constructor including atomic Transmute, shared guard/acquisition/
poison/drop, LRUWriteCache and the complete REDB crash theorem remain UNPROVED.
CP1=false, CP2=false, source_coupling_proved=false, native_bridge_proved=false.
HMAC/native remains DEFERRED AND UNACHIEVED.

Final focused outcome: 9/9 Rev/metadata tests (including nine u64/u32 metadata
placements), 12/12 source tests /226 compiler invocations pass. These include
default-off scalar positive/false-contract checks, ordinary/Infer inactive
requests, source privacy/escape/skip refusals and stale dual export invalidation.

Costs: conservative carried active charge 2436.263s (40.604min),
below45min; 11 completed command receipts total 1203.518s.
Only 1152 documented build seconds are excluded. Cancellation
downtime17:45..18:08:18 is excluded, while prior time is deliberately overcharged
through17:45 despite the17:43:04 last source edit. Frozen at 2026-11-12T18:33:33.263407+00:00.
Missing/cancelled receipts receive no credit; no CPU/dollar meter exists.
The exact eight tools, native query executable, source and per-command receipt/
log SHA256 identities are in D2_RESULTS.json. The continuing bundle is mutable;
parent must authenticate/relocate it before any later build.

Manual file/TCB scope: checked_owned_cell_lowering.rs contains actual native
construction/correspondence/fresh acquisition and cfg(test) full positive/negative
Function pipelines; vir/internal_repr.rs contains closed shape/contract/query-kind
validation and nonserializable bindings; well_formed.rs only adds the explicit
scoped native method check; context.rs only adds exact backend object/scope
binding and keeps metadata-only contexts inert; ast_to_sst_func.rs and
ast_to_sst_crate.rs add the actual-use rechecks. The four routed root corrections
stay separate commits. d1.py reuses existing check.py/ArtifactGuard/LoggedCommands
with D2 labels, historical/current runtime guards, complete relevant source
inputs and restart-safe no-incremental/max2-job configuration. Research output
stays implementation/. No production registration/installer or runtime source
change exists. All of these compiler TCB changes remain subject to NEW review.
