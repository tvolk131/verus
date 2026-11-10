# Original-reference exit: conditional adapter and checked MODEL components

This lane does not establish native `ExitObservation_f`. It supplies a precise
source dependency interface and independent algebra/resource proofs. The missing
source-to-current/future and final physical-current bindings remain UNPROVED.
Publication=false, native=0, 44 minimal operations +3 phase obligations /101
writer effects remain open. Canonical6/0 and full HMAC1/1 are inherited, not
rerun; the original full goal remains incomplete.

The early interface is commit `4fe86214`; `INTERFACE.md` routes the exact need
to parent/constructor erasure. `SOURCE_BOUNDARY.md` details the actual frontend
entry/reborrow/merge equations, normal-return rules, source-phase obligations,
and the full HMAC extension. `audit-input-pins.json` binds the prepared source,
accepted MODEL records and explicitly delivered completed artifacts. These
identities are provenance only, not source-effect proofs.

## Checked statements and their exact scope

`candidates/frontend.v` interprets the inherited C/F/update equations with pairs.
Its four definitions and seven proofs are an algebraic model of those equations;
they do not interpret Rust references. `conditional_original_exit` derives the
whole original future from the resolved updated original plus the pending
reference's same-future, final-current/physical binding and release. The latter
bindings must be proved for actual source execution; they are explicit theorem
hypotheses, not new axioms, application requires, or a relation defined as the
conclusion. The drop corollary substitutes physical no-write preservation;
the full-vector corollary accepts any proved complete writer vector, retaining
unissued bytes. Two closed logical witnesses expose the insufficiency of the
old summaries and the invalidity of resolving a live parent after child release.
They are not Rust counterexamples or semantic negative executions.

`candidates/resources.v` proves three existing-foundation MODEL laws:

* `pending_reference_normal_release` specializes the real checked typed resolve
  operation to the pending unique reference. Opaque iterator resolution is not
  used and cannot resolve ghost observations.
* `model_drop_continues` runs the existing shorter no-write reborrow protocol,
  restoring the same ordinary E/L/I, original parent's exact future/current
  observation and arbitrary separating ancestor Frame to a continuation. The
  parent lifetime remains live. The model expression is not presented as the
  selected MIR constructor/drop implementation.
* `live_original_current_observation` opens the unique body using the reference's
  ordinary lifetime context, derives the whole current vector's agreement with
  the supplied MODEL heap authority at p, and recloses the same reference/body,
  returning that authority and context. p is any model subobject location, not
  an allocation-base constraint or MutRef::unpack.ptr. The reference's future
  is unchanged; there is no assumption or conclusion that a live parent's
  future equals current. One internal later credit eliminates the payload later.
The heap observer is a final adequacy-cut operation. It is not placed inside a
new runtime WP while holding heap authority outside the state interpretation.
The initial combined WP wrapper was kernel-valid as a conditional formula but
not suitable as a realizable execution bridge; it was removed on self-review.
The final source relation must connect release and observation at their actual
semantic cuts. No combined native theorem is claimed.

All resources are premises of MODEL judgments only. This work does not convert
an actual input reference into model ownership, add client resource premises,
initialize a replacement heap/world, or impose a Rust runtime credit bound.
The incoming argument B0, internal reborrow A, and caller parent P are distinct.
At native normal exit the desired equation concerns F(B0); recovering a live P
changes C(P) while preserving its independent original F(P).

## Composition with the delivered constructor interface

After the split MODEL/algebra version passed trial009, the parent delivered
constructor-erasure commit `3d668d29`. `candidates/source.v` imports only its
completed `Interface` and `Composition`, from the receipt pinned at
`/tmp/native-constructor-erasure-6bb72bfb-trial006/receipt.json` (SHA256
`2f9679b44cded4fcb68052c062b8a6a4942b92d99bc099f2cac4d07625e84cb3`).
These modules supply the small explicit source data signature, not an actual
Rust semantics or certificate. No generic source signature is duplicated here.

Three further proofs use those delivered types:

* `initialized_bytes_unique` obtains equality of complete equal-length vectors
  from initialized lookups in the same source heap at the same tagged reference's
  allocation/offset addresses. No new abstract observation relation is defined.
* `conditional_source_final_vector` combines that CURRENT/physical agreement
  with original merge, same pending future and separate pending release. It
  returns the entire original future and initialized original-subobject bytes.
  A future HMAC simulation can supply its exact full prefix/suffix vector here.
* `conditional_source_drop_exit` uses `entry_normal_bytes` and `V_entry`'s
  actual operand1 reference, preserving the full certificate's fault exclusion.
  Its normal-return observation is conditional on the still-unproved actual
  source certificate and original-entry/pending-exit C/F bindings. No lawful
  input or execution is filtered by future definedness or promised normal return.

The entry certificate is a proof argument. This lane constructs no actual
`source_semantics` instance or certificate; `root_reborrow_need` and source
storage-prefix/phase work remain unproved. The record's field types do not bind
it to real Rust or to the frozen body. That source interpretation and the
syntax-indexed compiler transport remain separate missing theorems. Neither
source selection nor receipt/hash/flag matching proves them. All native counts
and publication remain unchanged. Pointer-distance2c0b8f23 is not needed by drop
and is not imported; its lawful-source-to-span and intrinsic interpretation
remain obligations of the HMAC extension.

## Remaining source premises and integration

No inspected existing mechanized definition interprets actual lawful Rust entry,
original operand evaluation, runtime provenance/history, and frontend C/F values
together. The early `UNPROVED_source_erasure_and_observation` name in
`INTERFACE.md` denotes the **joint** missing theorem bundle. Its complete list
must not be treated as work already assigned to, or supplied by, constructor
erasure. The minimal constructor-erasure dependency is only:

```
UNPROVED_SourceErase_drop(actual_function_syntax, original_operand)
  independently lawful entry -> actual operand/subobject/initial bytes;
  every run including faults -> source validity/phase preservation;
  every normal exit -> same original subobject's bytes unchanged;
  cleanup/unwind handled explicitly, preserving caller access/continuations.
```

This lane still separately owes `UNPROVED_OriginalOperandCF` (source entry
current and original future interpretation through reborrow/merge) and
`UNPROVED_PendingExitCF` (last physical bytes to pending-current interpretation,
same future, and release eligibility/no-escape at the actual normal-exit cut).
Those are not consequences of constructor erasure's byte stuttering. The
checked algebra/resource laws here identify their exact required outputs, but
cannot prove these source bindings without the missing source interpretation.
They remain this handoff's explicit unresolved observation obligations for
parent integration, not obligations silently transferred to the other worker.
All three names specify proof obligations; none is a new Coq axiom or predicate
installed into production. Root must reconcile the concrete semantic types
before any native theorem can be claimed.

Retag/protector, two-phase activation, unsize, storage, FakeRead, optimized-phase,
empty-glue and cleanup/unwind obligations remain separate. Lawful entry must be
defined independently of successful model execution; runs include faults. This
adapter's normal-return cut does not establish no-UB, exclude unwinding or add a
normal-run-only domain filter. Empty runtime drop is not logical loan release.

A future proof object must be indexed by the actual function syntax and original
operand, with checked transport from that syntax's source execution into these
interpretations. Compiler integration then checks the derivation and its input
syntax. Hash matching can bind bytes of those inputs/objects but cannot replace
that semantic derivation. No production import or unchecked native summary is
installed here. A functional-projection implementation need not reconstruct the
whole caller ty_own context, but it still owes equivalent physical/current and
pending-release correctness from the independent source semantics.

The full HMAC extension is specified, not proved: every valid key length,
arbitrary initialized[128], actual Zip index/getters/ordinary XOR store/None,
all pending and unissued slots, full future vector, key and invocation footprint,
and lawful interior/ancestor histories. The parent-delivered committed ordinary
caller5 (`a8df10a0`) supplies completed MODEL support under existing typeG/E/L/I;
its independent acceptance remains pending. It was inspected from that commit
and its named completed receipt only; it is not imported as source authority.
The accepted caller9, complete ordinary25 and raw initializer remain MODEL
prerequisites. The shared automation still has to prove both full frozen source
properties; this lane makes no reduction or completion claim.

## Validation and replay

See `results.json` and `TRUST_INVENTORY.json` for the final checked local trial,
complete declaration/helper/constructor/obligation assumptions, object hashes
and kernel result. `development.json` retains failed attempts; none is semantic
negative evidence. All raw logs, copied proof sources, products and durable
receipts remain in private trial directories. Pretrial freezes are committed.

The controller derives complete local inventories, rejects untracked generated
constructs and unchecked proof escapes, compiles with normal coqc and recursively
runs coqchk with no exclusions/unsafe flags. Declaration visitation is checked
exactly once and in source order within each module; independent modules may be
visited in either order. Every source declaration and every generated inventory
helper has a printed assumptions inventory. No new constructors or Program
obligations occur. Only inherited dependent functional extensionality and
Eq_rect_eq are permitted; exact observed assumptions are recorded.

Both baseline readiness and its successful named durable receipt, immutable
VerusBelt115a618d's 523 source files/284 objects, pinned runner/coqc/coqchk/opam,
loadpaths and GMP manifest332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37
are authenticated. All retained inputs and each produced source/object are
guarded through success. No shared build/install or compiler/vstd/global edit
was performed. No subdelegate, push, PR or publication was performed.

Replay into an unused private directory:

```sh
python3 -B -O source/tools/research/native_original_reference_exit/check.py \
  --freeze source/tools/research/native_original_reference_exit/final-freeze.json \
  --output /tmp/native-original-reference-exit-replay-UNUSED
```

Root integration and fresh independent review remain required. This local
handoff is not source-boundary acceptance or authorization to change native
status/publication.
