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

`candidates/resources.v` proves four existing-foundation MODEL laws:

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
* `pending_heap_normal_exit` composes that observation with normal typed release,
  consumes precisely the pending reference and hands its full future observation,
  actual MODEL bytes, heap authority, same L/I and ancestor Frame to a
  continuation. It makes no claim that every arbitrary live parent is eligible
  for release. Source integration must prove eligibility/no-escape at the actual
  exit cut; the remaining caller parent belongs in Frame and is not resolved.

All resources are premises of MODEL judgments only. This work does not convert
an actual input reference into model ownership, add client resource premises,
initialize a replacement heap/world, or impose a Rust runtime credit bound.
The incoming argument B0, internal reborrow A, and caller parent P are distinct.
At native normal exit the desired equation concerns F(B0); recovering a live P
changes C(P) while preserving its independent original F(P).

## Remaining source premises and integration

No inspected existing mechanized definition interprets actual lawful Rust entry,
original operand evaluation, runtime provenance/history, and frontend C/F values
together. The source worker must supply a syntax-indexed derivation for the
unchanged function, evaluating original MIR `_1` into the same interior subobject
and dynamic loan. Its reborrow/merge proof must retain the original future and
caller continuations. Its normal-exit case must prove the whole pending current
is the actual final bytes and the pending value/loans are eligible for the
checked release; physical preservation alone does not prove that case.

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

See `results.json` and `TRUST_INVENTORY.json` for the final accepted local trial,
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
