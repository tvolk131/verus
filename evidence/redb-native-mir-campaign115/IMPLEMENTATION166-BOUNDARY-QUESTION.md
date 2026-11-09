# Implementation166: exact boundary question; missing core NOT implemented

## Question for parent disposition, not a proposed allocator axiom

**What is the authoritative native execution/heap/provenance interpretation
against which the compiler must check the selected original MIR activation,
normal-return pointer and consumptive continuation?** In particular, is an
explicit partial native-step/heap/activation interpreter, with a checked
ghost-erasure simulation into the canonical pointer/resource model, intended to
be a newly reviewed compiler-semantic TCB component; or must that simulation be
derived from an existing pinned native operational model? No such native-state
interpretation is represented at the authenticated164 adapter interface.

This is not a request to approve a pointer/event equality, invent an occurrence
counter, execute another wrapper allocation, install a frame axiom, or weaken
any gate. Authority139 already permits implementing a gated interpretation of
the exact conditionally adopted schema. The absence of an implementation is
not proof that new mathematical allocation authority is logically necessary.
I have NOT shown that the requested implementation is impossible. I stopped
without implementing the core rather than present an output-binding or a new
opcode as its solution. This question identifies the missing semantic contract
and trust choice; it does not assert that development authorization was absent.

## Concrete interface obstruction

At `source/rust_verify/src/native_adapter164.rs`'s
`lower_native_instance(tcx, owner)` entry, the adapter obtains the selected body and initializes
`State164::default()`. That state contains only a map from native MIR locals
to expressions, optional bundle expressions and optional local loans. Its
`has_resources` quantifies only over this map. It receives no native activation,
execution prefix, global ghost-resource frontier, caller-held generations,
native heap/provenance interpretation or consumptive normal-return witness.

Consequently two executions with the same owner/body/argument types but
different prior live resource frontiers are indistinguishable to this entry
check. This is an interface-level indistinguishability argument, NOT an
executed counterexample, proof of an exploit, or proof that the retained
fail-closed gate is bypassable. `has_resources() == false` proves local bundle
absence, not that the current global frontier is empty. The latter must be
checked from an actual incoming flow, not assumed from an empty map.

At the selected call, real MIR still has `callee`, `destination`, normal target,
unwind outcome and typed operands. The emitted VIR call retains two scalar
arguments and a tuple result, but not an owned native normal-return transition.
`adapter164::sst_call` then emits an ordinary initialized SST call destination
with those two scalar arguments. Its static call target is not a dynamic event.
Adding a full pointer field, a site hash, a fresh scalar binder or an `issued`
Boolean here would not establish the required native transition identity.

The current fixed opcode's AIR branch unconditionally returns an error at
`source/vir/src/sst_to_air.rs:1839`. Merely replacing that error with havoc and
the allocation postconditions would introduce the conclusion without checking
the missing native event/full-pointer/world premises. Replacing it with a call
to `vstd::raw_ptr::allocate` would be a different allocation; instantiating
that external contract as if its wrapper executed is not allowed either.

## Affine-generation obstruction is separate from pointer predicates

`review_lower_to_sst` constructs a crate with no datatypes and rejects a native
allocator instruction before canonical initial WF. It cannot demonstrate loaded
`vstd::raw_ptr::{PointsToRaw, Dealloc}`. `ResourceTypes160::from_datatypes` checks
nominal paths/modes; fabricating datatypes with matching paths would not load
the genuine library or solve the origin problem.

The native instruction's mode branch checks scalar operands in Exec mode and
returns Exec. Ordinary mode/WF/SST checking does not by itself establish the
affine ownership of the generated tracked pair. The canonical lifetime pass
calls rustc borrow checking on Verus-aware source items; the diagnostic adapter
creates VIR from already selected native MIR in `after_analysis`, outside that
source tracked-resource path. There is no second tracked-resource lifetime pass
over these newly generated VIR values in this adapter. This does not say that
Verus's normal tracked-resource lifetime checking is absent or defective.

The required implementation must either connect generated native shadow
resources to the genuine canonical affine checking path before its relevant
checks, or implement and independently review a sound native resource-flow
checker integrated into canonical admission. Cloning `Value164` for exclusive
CFG branches is compiler bookkeeping; a non-Clone event record is likewise
not a semantic witness. No choice was implemented here.

## Minimal missing semantic interface (requirements, NOT installed equations)

The reviewed implementation needs one partial interpretation judgment relating:

1. The exact authenticated typed native MIR state and selected terminator,
   monomorphized caller stack/activation, actual control-flow edge and actual
   size/nominal Alignment operand; the runtime/target/cfg/ABI/allocator and paired
   deallocator binding must be checked, not supplied by symbol spelling.
2. A source-derived normal-return step carrying its actual destination value
   in the native pointer domain, and the corresponding canonical full pointer
   including provenance and metadata. There is one native return, not an
   independent proof pointer constrained to the same address.
3. An incoming current affine frontier with justified global emptiness for an
   empty-entry subset, or precise checked current-flow framing otherwise. The
   frontier must account for caller-held raw/bytes/loans/Dealloc and stale/live
   generations, not just locals in this one MIR body.
4. A consumptive transition available only in that dynamic native normal
   continuation. It distinguishes repeated/recursive activations and reused
   addresses without identifying logical events with physical allocations.
   Construction of a static descriptor cannot discharge this requirement.
5. Actual loaded tracked resource types, initialized bindings and checked
   affine generation/transfer. Only after these premises hold may the exact
   law158 schema supply its raw range, provenance, Dealloc data, alignment/end
   bounds and generation-ownership conclusion. The same full pointer must flow
   onward to the original allocation156/store154 consumer.

For null return, error, unwind, abort and divergence, keep distinct native
outcomes and introduce no success pair. No rule may turn a compiler `Assume`,
unsupported cleanup edge, loop or callback into a success/termination/frame
assumption. An empty-entry subset must reject a nonempty or unknown current
frontier and cannot claim general global-resource preservation.

This is the missing interpreter/simulation and affine-flow implementation
contract. It is not a suggested second mathematical allocation schema. If a
trusted extra native-to-spec or owned-memory frame equation is proposed to
replace any checked premise, its exact text/scope needs separate authority and
review; none is proposed or installed by166.

## Connection to the original allocation156/store154 route

`source/rust_verify/src/allocation_selection160.rs` follows the actual selected
Arc allocation, validates the raw payload projection and binds the original
store's allocation local. It is not consumed by `native_adapter164.rs` or its
diagnostic binary. The adapter's `proof_type` supports words, bool, unit and
`*mut u8`, not the full Arc/layout/aggregate continuation. No new original-MIR
analysis was run, and none of the earlier selection metadata is promoted to
semantic coverage here. Old160 is unchanged and remains disabled/unrepaired.

The next safe core implementation must preserve that selected original native
state/continuation through the above judgment. It must not select an unrelated
allocator fixture. Header initialization, Arc/layout/iterator/prior-value
obligations, current range/provenance flow and the payload store remain checked
work, not allocator postconditions or an assumed caller invariant.

## Disposition and result

Core implementation delta: **none**. After the parent steering update,166 also
consumed authenticated source-only167 and added the conservative O165-2 scalar
validity guard in both transmute handlers. These five changed source files are
safety repairs, not the missing core or a new milestone. Baseline164-relative
patch/full changed sources are sealed separately. No new opcode, AIR body,
event/pointer/frame axiom, tracked constructor or issuer was added. In particular,
the request to implement AIR lowering is NOT satisfied.
This is a boundary handoff, not a semantic milestone, attempted core completion,
or independent exact-code issuer review. Compiler builds and compiler-only
diagnostics for the separately requested safety repairs are recorded in the
final report; no original application or allocator fixture function executes.

One mathematical schema remains conditionally adopted; operative issuers0,
issuance0, solver runs0, native acceptance0, native export REFUSED, original
application/test executions0, original accepted allocations/stores/bodies0.
The first cell remains full original two-phase acknowledged durability, all18
OPEN. No assumptions externalize Arc/layout/iterator/cache/constructor/caller
obligations, no native source integration occurs, and all Infer/S0/full-scope,
default-off, import/export and serialized/spoofed-origin gates are unchanged.
