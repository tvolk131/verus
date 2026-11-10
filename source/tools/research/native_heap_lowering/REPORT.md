# Checked source-driven fragment interpreter; native interpretation still open

This lane implements a new interpreter and generated ordinary proofs for the
actual minimal native constructor/drop graph. It does not prove a theorem about
Rust references or the full frozen HMAC writer. Native publication is always
false. No production fact, engine/library modification, new axiom, external body,
assume/admit, permission constructor, executable replacement, push, PR, global
setting or subdelegate is included. Only native_heap_lowering is edited.

The early interface/candidate is commit `2491e132`. The final candidate is
`candidates/trace-v9/trace.rs`, importing `candidates/kernel-v7/kernel.rs`.
`results.json` binds exact receipts and excluded trials; raw MIR/logs/binaries
remain in `/tmp/native-heap-lowering-c3bf5d36/`.

## What is implemented and checked

The unchanged rustc-private extractors are built as two binaries in a standalone
nested workspace. They consume actual MIR enums, CTFE constants, layouts,
resolved instances and local borrowck facts through their existing encoders.
No debug-string parsing supplies semantics. Normal Rust analysis and the original
borrowck query run. The fixture only renames native_handoff to xor_key and erases
verifier syntax/prelude for the existing root selector. Its executable body is
`{ let _it = buf.iter_mut(); }`, with the same single exclusive array operand.
The ordinary native diagnostic is retained byte-for-byte separately.

Fresh extraction reproduces six instances, 44 statements/terminators, two call
edges, no pending instances and no extraction blockers. Three instances are
empty-return drop-glue queries, **not extra runtime calls**. The runtime chain
is root -> slice::iter_mut -> IterMut::new. The actual u8 CTFE Boolean is zero;
its case chooses bb2. The two operations in bb1 (usize-to-pointer transmute and
goto) are excluded as CTFE-unreachable, never given a general cast rule.
There is no unavailable panic or intrinsic leaf in this minimal graph. The full
writer's pointer-distance intrinsic and unavailable panic bodies remain open.

`lower.py` is an **untrusted candidate producer** over structured payloads. It
checks the source-loan/reborrow/unsize operand binding, finite closed local type
and layout eligibility, each native call/drop target and operation/exit coverage.
Every typed local is checked, including caller destinations; a mutation control
found the earlier missing caller-local check. Unknown types, stores, calls,
cleanup, arithmetic, retags or casts reject. Names/identities bind definitions;
none supplies an effect or a permission. This Python producer is not a proved
Rust-to-model compiler.

`witness.py` proposes explicit normalized model states. Ordinary Verus checks
49 instruction transitions, all proposed next-state equalities and their typing,
then stitches the transitions by checked recursion. The final run is **115
verified / 0 errors**, with zero inferred helpers and full selected-body checking.
This includes 49 edge proofs, 50 suffix-path proofs, start/finish, the trace and
window corollary, ten general kernel proof bodies and two recursive spec-body
termination queries. Every new proof/application uses `--no-cheating`; no body,
mode, WF, lifetime, trait or termination checks are disabled. Opacity commands
only control SMT unfolding; every hidden definition's applicable law is checked.
The three empty-glue query observations contribute model DropReturn identity
steps after the root trace; they do not pretend to be runtime destructor calls.

The kernel uses explicit values (reference metadata, raw pointers, NonNull and
IterMut records) with object/loan identity and relative offsets. Construction,
metadata extraction, pointer thinning/transparent conversions and end-pointer
arithmetic preserve these identities in the model. Moves invalidate the source
register. Copies duplicate **metadata**, never a physical permission. One scalar
custodian and a stack of suspended frames follow the actual constructor operands
and returns. Re-entering the same custodian or returning through the wrong frame
is rejected. There is no additional Rust reference field or second live borrow.

All 128 arbitrary bytes remain in the model throughout the actual selected trace.
There is no zero-byte premise, key truncation or supplied full-owned heap. The
model embedding theorem constructs initial observations from any length-128
sequence and arbitrary object/loan IDs. **These are freely describable ghost
observations, not physical ownership or Rust provenance.** The custodian is a
mathematical transition-state component, not a tracked unforgeable capability.
Copying that whole ghost state cannot authorize native operations or publication.
This is deliberately distinct from the earlier supplied-PointsTo resource module.

A Store operation is included in the semantics. The ordinary one-step and
recursive trace frame proofs derive preservation outside the interpreted write
footprint. `unissued_suffix` derives preservation of all slots at or above an
issued boundary if that footprint is below it. It is a general model theorem,
not an assumed native frame or a proof of the frozen writer's footprint/loop.
The actual generated no-write path checks every model guard, including its
end-pointer offset of 128; it is more than a syntactic no-store scan.

The checked Window lemmas allow an array to be a **subregion** of an allocation.
They derive isize offset bounds, mathematical address <= usize::MAX, and
same-containing-allocation endpoints from allocation-base/size and containment
conditions. They allow array_base != allocation_base, and prove strict interior
bounds for dereferenced byte offsets below 128. The one-past pointer is never
read. Two examples cover both an allocation prefix and a strict subregion.
The laws use the supplied [pointer-add](https://doc.rust-lang.org/std/primitive.pointer.html#method.add) and [reference-validity](https://doc.rust-lang.org/reference/behavior-considered-undefined.html#invalid-values) documentation as the
basis of an explicit mathematical interpretation; they do not construct liveness
or ownership. No separate-allocation premise is imposed. The Rust fixture also
compiles a caller with pad and a 256-byte key in disjoint fields of one allocation.
This validates caller eligibility, not a key heap or full Zip semantics.

## Exact conditional-versus-native boundary

The source-connected result is an operation-covered candidate and checked
execution **in the authored fragment semantics**. It is not a checked simulation
from Rust execution. Backend checking prevents a producer from forging a failing
model transition, but does not establish that the meaning assigned to the Rust
primitive instruction is correct. Python source matching is additional untrusted
extraction/eligibility machinery, not a metatheorem about that translation.

Explicit premises/trust:

* `embedding`: bytes.len() == 128; object/loan IDs are arbitrary model IDs.
* `step_preserves_typing`: a well-typed model state and a nonfailed model step.
  Generated edge proofs separately establish nonfailure for the selected trace.
* `unissued_suffix`: the interpreted footprint is below the issued boundary.
  That premise is not established for the original full writer.
* `checked_drop_in_window`: a containing-allocation Window satisfying numeric
  bounds and containment. This is a model interpretation premise, not a new
  requires on either native application. Its derivation from every valid input
  Rust reference is unproved.
* Existing Verus/rustc frontend, solver, inherited Seq/Map/Set models, and the
  authorized runtime/vstd remain trusted. No raw PointsTo primitive is reused.
  The supplied inherited vstd verification/admission inventory is not changed
  or claimed re-proved. The bundle is a guarded byte-for-byte independent COPY
  of parent-shared68-bundle-676b99bf, with matching macros/vstd/Z3 4.16.0.

The **first unresolved lemma** is total input interpretation for the original
`&mut [u8;128]`: from the existing valid Rust reference and source borrow/region
facts, obtain a containing live allocation, the same array subobject and loan,
exact initial bytes and the interpretation of the native constructor operand,
with no second live reference and no supplied physical permission premise.
A freely constructed `initial(bytes,origin,loan)` proves none of that authority.
The local extractor records a reserved/issued loan and typed operand chain;
its summarized region counts are not a proof of a Rust borrow semantics.

Subsequent unresolved obligations are primitive and compositional simulation:
reference validity and suspension/retag rules; raw-pointer derivation/provenance;
transparent NonNull valid-range and layout interpretation (not just equal sizes);
unsizing/field representation; native call returns and exception freedom under
that interpretation; and the logical meaning of scope exit/loan release.
The per-step model guards do not independently prove native nonpanic/non-UB.
Finally an output adequacy theorem must identify the reconstructed model bytes
with **final of the same original executable array reference**. No such future
view equation is stipulated. This last link is not implied by an empty drop body.
The model carries no native prophecy/future-view witness.

Full reference alias/liveness semantics are not completely specified by the cited
Rust documentation. These gaps remain unproved, not solved by assigning trusted
names to the model. No claim of complete Rust memory-model verification is made.
Independent review by the parent is still required, and cannot itself substitute
for missing proofs. `audit.publish` rejects unconditionally, even for forged
all-proved statuses and correct model proofs.

## Outcomes and controls

The final clean suite regenerates the exact frozen trace from freshly extracted
actual source. The ordinary unchanged native-drop diagnostic remains **0/1**.
The unchanged full-domain annotated native writer remains **1/1**. Thus neither
actual native theorem is proved or publishable. The full XOR prefix plus untouched
suffix for arbitrary initialized [u8;128] and every valid key slice (0 and >128
included) is still required. No HMAC/security theorem is claimed.

The 28 source/forgery controls distinguish eligibility from semantics. Actual,
honestly rehashed source variants include tail, helper and destructor writes,
wrong original array, a valid same-allocation caller, and an invalid duplicate
mutable-borrow caller. The latter fails real lifetime checking with E0499 under
both extractors. Native-IR controls cover wrong layout/original place/arithmetic,
unknown retag/cleanup/instruction, omitted operation/helper/drop/exit, loan/shape
mismatch and altered plans. Failed/incomplete/timeout/resource/proved statuses,
and ghost-only copied model state cannot publish. These are not SMT negatives.

Twelve separate ordinary model negatives fail only the intended postcondition,
with all imported kernel bodies checked: wrong object/loan, out-of-bounds offset,
initial/future mapping, tail write, non-reference call, wrong branch, repeated
custodian, wrong return and invalid subregion Window. Earlier controls retain
6 healthy queries; the final-kernel controls retain 12. They refute model claims,
not Rust programs. The source rejects are independently compiled and rehashed;
identity mismatch alone is not counted as an effect test.

All candidates are frozen before their trials. Excluded syntax/annotation,
missing proof-extensionality/invariant, resource, candidate-producer and audit
failures are retained; see results.json. An initial extraction control run was
correctly rejected by ArtifactGuard when unrelated frozen producer sources were
edited during it; a new clean run passes. No such failure is semantic evidence.
Controllers use infer_experiment/infer_eval_support durable receipts and guards
under Python -B -O, explicit require checks, exact helper inventory {} and the
mandatory soundness/maximality/exactness checker (vacuous for zero helpers).

## Reproduction

From the repository root, with new private paths:

```sh
python3 -B -O source/tools/research/native_heap_lowering/copy_bundle.py \
  --destination /tmp/heap-lowering-review-bundle
python3 -B -O source/tools/research/native_heap_lowering/suite.py \
  --freeze source/tools/research/native_heap_lowering/final-freeze.json \
  --bundle /tmp/heap-lowering-review-bundle \
  --output /tmp/heap-lowering-review
```

Cargo runs from source/ with Rust 1.99.1, two jobs, --offline --locked and private
Cargo build/target directories. Policy 68/cache 71/schema 38 are untouched.
