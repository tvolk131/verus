# Checked conditional linear resource laws; no native theorem

All work is confined to native_effect_soundness. The frozen application body,
full input domain, XOR-prefix plus unchanged-suffix property, shared harness,
engine, vstd, policies/cache/schema and sibling directories are unchanged.
No new axiom, assume, external body, permission constructor, publication rule,
push, PR, global setting or delegated worker was added.

Early interface/pretrial candidate: `1d8a8df4`. First checked next/None module:
`d31409ae` (3/0). Expanded checked module and controls: `423d31d3` (18/0).
The source audit and entry-interface reconciliation are separate diagnostic work.

The complete native bridge and full frozen writer remain unproved.
`native_bridge_proved=false`. There are **zero checked actual native operation
correspondences**, zero actual native ownership theorems, and no full writer
proof. Neither a good hash nor a checked conditional owner can publish a fact.
Both the inherited certificate barrier and this lane's publication barrier reject
even forged all-proofs-complete inputs unconditionally.

## Exact checked result

`candidates/v8/laws.rs` checks **18 verified / 0 errors** using the immutable
private byte-for-byte copy of runtime 61f21823 (displayed as 61f2182), fully
verified inherited vstd, Rust 1.99.1 and Z3 4.16.0. Every new proof/application
trial uses `--no-cheating`, ordinary body, lifetime, mode, WF and trait checking.
No engine/library change or vstd rebuild is needed. Schema38/policy68/cache71
are inherited unchanged. A fresh copy made by the committed copy controller
reproduces 18/0 separately.

The frozen explicit ordinary inventories contain two executable bodies,
`observe_key` and `write_loan`, and these sixteen proof bodies:

- `accept_owner`, `issue`, `none`, `resolve`, `reconstruct`, `partition`;
- `pointer_separation`, `retained_tail`, `empty_key_preserves_all`;
- `projection_step`, `constructor_extent`, `world_owner`, `world_issue`,
  `world_resolve`, `resolve_all`, `close_world`.

The inferred helper inventory is exactly empty. Every ordinary outcome still
passes the shared helper-identity and mandatory soundness/maximality/exactness
checks; no inference or assistance improvement is claimed.

`Carrier` owns tracked maps of inherited **linear PointsTo<u8> tokens**, retained
or returned, plus non-authoritative pending receipts. A returned `Loan` carries
its actual token. `World` additionally owns the outstanding loan-token map, whose
domain equals pending. Its full inventory contains all 128 initialized tokens,
not merely the min-length pair prefix. This is resource checking beyond a
copyable spec-state trace; no token is manufactured from a pointer or sequence.

`accept_owner` preserves the supplied pad map. `constructor_extent` computes
full physical pad/key end metadata and the min extent without changing tokens.
These are conditional ghost constructor laws, **not actual native constructor
laws**. Entry keeps arbitrary initial bytes and the entire key sequence. Key
length can be empty, 128, or larger; only min(128,key.len()) read tokens are needed.
Rust-valid address/length scalar bounds do not restrict the valid application
domain, but interpreting those bounds from native Rust references remains open.

`issue` transfers exactly the old Zip index token, preserves every other retained
token and all returned tokens, records the loan, and increments only the physical
descriptor's index. `world_issue` proves equality of the entire token inventory
across this transition. `none` is identity, preserving even the empty-key tail.
`projection_step` uses Zip.index while inner bases/end extents stay fixed.
No raw-inner-field projection is identified with a shrinking logical sequence.

`observe_key` borrows a supplied shared read token through inherited ptr_ref;
`write_loan` uses inherited ptr_mut_ref and proves the arbitrary final byte is
reflected in that very loan token. These are checked local wrappers, **not new
native getter specifications**. No proof-mode write to an executable location
is attempted. Shared key authority and its original-key relation are inputs;
no mutable alias to that shared token is supplied.

`resolve` restores exactly the returned loan token. `world_resolve` preserves
all token values. `resolve_all` proves termination by the outstanding map's
length and preserves the inventory through arbitrary resolution order.
`reconstruct` and `close_world` return all 128 tokens, including arbitrary
permitted final values in issued slots and the original values in every
unissued slot. This supports early abstract scope release as well as None.
The retained-value relation is initialized from full_owner and proved invariant
under these checked operations. No XOR formula or requested pad_xor predicate
is assumed. The generic unchanged tail follows from resource conservation.
Map reconstruction is **not reconstruction of the native executable array
borrow**, and abstract scope release is not a native drop/unwind theorem.

## Conditional premises and inherited trust

`accept_owner` needs an already supplied tracked map with all 128 initialized
permissions, exact initial values and matching full-provenance slot pointers.
Entry.valid supplies mathematical byte-stride, address bounds, prefix-read
pointer/provenance relations, key extent bounds and byte-footprint separation.
It does not itself prove allocation validity, pointer arithmetic, Rust retags,
source-loan identity, original/future array linkage or reference validity.
Those are explicit additional source-interpretation obligations, not hidden
application requires clauses. No input token is obtained by spec choice.

The key reader needs borrowed prefix permissions and their actual-key value
relation. Valid immutable slice interpretation remains conditional, including
complete metadata/extent and all native size paths. Pad/key byte footprints
must be disjoint; their allocations/provenances may coincide. The sibling's
same-allocation disjoint-field caller is compatible with these conditions.

Trust includes the inherited verifier/Rust frontend/SMT model, reference and
pointer models, PointsTo abstraction, ptr_ref/ptr_mut_ref external primitives,
and tracked Map empty/remove/insert/borrow/union axioms plus existing collection
laws. The inherited vstd 2226/0 result is supplied by the accepted runtime;
its 83 inherited admissions are not removed or rechecked here. New bodies are
ordinary checked; none extends those primitives.
All Entry/Physical projections, carrier invariants, helper contracts and proof
steps are authored manual input. There is no claimed automatic native lowering.

Read-only sibling evidence at `98f428fa` is reconciled in ENTRY_HANDOFF.md and
bound in entry-evidence.json. Its checked 2/0 tracked-slice entry provides a real
tracked reference to an executable location with current/future equations.
It does **not** supply PointsTo tokens or numeric provenance. E0499 rejects
keeping that live owner beside actual iter_mut. The executable split carrier's
3/0 and duplicable heap calculus's 10/0 are not counted in this lane's 18/0.
No sibling entry/constructor handoff is duplicated or assumed solved.

## Actual typed MIR audit and precise blocker

`source_replay.py` rebuilds the read-only inherited extractor in a private
Cargo target/build directory, from source/ with two jobs, locked/offline. It
uses the inherited artifact/source guards and normal rustc analysis, queries
actual monomorphized instances and consumes the existing schema-2 structured
IR through the existing checker. No debug text is parsed as operation semantics.

The original extraction reproduces **32 instances, 408 operations, 31 call
edges and 36 local drop queries**. Each statement and terminator, including
storage, moves, branches, calls, cleanup, returns and native drops, gets a
unique location and typed-payload hash in the private source-ledger.json.
Call/drop helpers and unavailable leaves are separately inventoried. The
ledger requires exact equality to a complete regenerated inventory. This
checks source identity/coverage, **not operation or refinement soundness**.
All 408 correspondence obligations and all inherited 101 effect obligations
remain unproved. No actual native instruction is marked proved.

The first concrete getter gap is the mutable __iterator_get_unchecked body:
**bb0:s6 computes pointer Offset; bb0:s8 forms &mut from its dereferenced result**.
Its immutable counterpart has the same offset and shared borrow. To apply
ptr_mut_ref/ptr_ref, a checked lowering must relate that actual pointer (including
provenance, bounds, stride and retag) to the supplied index permission, transfer
that permission with the actual returned reference, and account for the actual
caller/native Zip.index. The local wrapper proof does not establish this.
Foreign MIR is not translated into these ordinary local checked bodies by the
current frontend. No new accepted lowering/checking rule is implemented.

Constructor metadata/size/min calculations, every native call, cleanup/unwind,
phantom/retag effect, drop and writer callback/panic path also need checked
simulation. The intrinsic ptr_offset_from_unsigned and unavailable panic and
panic_nounwind_fmt bodies remain explicit blockers. Empty-return physical drop
observations do not discharge ownership release. No unavailable branch is
silently omitted or assumed unreachable.

The minimum next implementation is a source-to-resource interpretation of the
actual single source loan handoff plus those native operations, followed by
checked generic simulation/induction and independent soundness review. The
proposal in ENTRY_HANDOFF.md permits either a justified PointsTo handoff or
one tracked executable-slice owner transferred with native IterMut; it grants
neither as an assumption or unchecked API. Final map-to-final(original array)
correspondence and the unchanged full writer proof remain required afterwards.

## Controls, freezes and replay

`proof-results.json` records the 18/0 full run plus **12 expected negative
outcomes**: lost slot, repeated issue, wrong index/origin/projection/constructor
extent, wrong initial/future owner, unresolved loan, extra tail write, key
mutation, and duplicate linear token. Eight are ordinary postcondition failures;
three are resource/precondition rejections. Duplicate physical token use is a
frontend E0382 rejection, not an SMT semantic counterexample. Healthy local laws
check 18/0 alongside each ordinary negative (18 healthy / 1 intended error).
No frontend/resource/poison outcome is mislabeled semantic.

`source-results.json` records **14 new source-ledger/publication controls**,
24 inherited certificate/scalar/resource controls, and **10 independently
compiled honestly rehashed source variants** (8 inherited, 2 new key-write and
alias-key cases). Changed XOR, lost write, extra tail write, wrong RHS/origin,
helper/drop/callback effects and key/alias writes reject by operation/effect
eligibility diagnostics. They are not native semantic proofs, and identity
mismatch alone does not satisfy a control. New controls cover omitted/duplicated
operations, missing helpers/drop/panic, forged bindings/proofs/facts, unknown
rebound operations, and failed/incomplete/timeout/resource/forged-proved statuses.
No native publication can succeed even for the original.

All candidates and semantic/source controls were hash-frozen before their
trials. Retained v1-v8 sources and private durable receipts preserve excluded
runtime-display, annotation/frontend and modular-postcondition development
failures; they are not accepted outcomes. Both controllers use shared
infer_experiment/infer_eval_support, ArtifactGuard, explicit require gates
and receipts under Python -B -O. Raw MIR/logs/binaries remain private under
`/tmp/native-effect-soundness-ced1eb20/`; no Python assert is an acceptance gate.

From the repository root, using fresh output paths:

```sh
python3 -B -O source/tools/research/native_effect_soundness/copy_bundle.py \
  --destination /tmp/fresh-effect-bundle
python3 -B -O source/tools/research/native_effect_soundness/replay.py \
  --bundle /tmp/fresh-effect-bundle --candidate controls \
  --output /tmp/fresh-effect-proofs
python3 -B -O source/tools/research/native_effect_soundness/source_replay.py \
  --solver /tmp/fresh-effect-bundle/z3 --output /tmp/fresh-effect-source
```

For only the positive ordinary module, use `--candidate v8` with a fresh output.
Bundle hashes and distinct inodes are guarded; the source replay additionally
binds actual rustc/core metadata and pinned rust-src via the inherited controller.
No full HMAC/security theorem or project-milestone acceptance is implied.
