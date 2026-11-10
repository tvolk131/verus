# Frozen implementation scope

Base: bc3317f327f99b2f8b18d1268c8e1a8e0c844445.

Proposed translation: preserve rustc's constructor DefId, generic arguments, and
`FnDef` singleton. Register an ordinary checked exec function whose ordered exec
parameters feed the existing VIR datatype constructor. Its exact result equality
is proved from that body. Existing FnDef lowering derives Fn-family traits and
requires/ensures; Option::map uses its inherited callback contract. No application
closure, callback axiom, source rewrite, or function-type equivalence is added.

Generate local supported tuple-struct and tuple-variant functions during early
frontend registration, with normal generic bounds, visibility, modes, and body
checks. Unsupported ghost/mixed-mode and opaque/external constructors must be
rejected clearly. Type invariants must remain checked (or the shape rejected).
Function-pointer coercion remains unsupported. Rust lifetime/move/visibility
checks remain authoritative for the unchanged application.

Proof target: `bortag.rs`, retaining literal `NonZero::new(i).map(BorTag)` and actual
`core::num::NonZero<u64>`. Standard Option::map requires the callback precondition
on Some and ensures the callback postcondition. Inherited NonZero::new specifies
Some payload view equals the argument and argument nonzero, or None iff zero.
These inherited vstd assume-specification/axiom boundaries are not native proofs.

Evidence: epoch.py freezes tracked source plus trial/tool/helper identities before
each command and checks them after; logs/binaries remain private under the named
/tmp directory. Failed setup/frontend/resource/proof trials are separate epochs.
No inference gain, native safety, runtime permission, C/F, HMAC, or full goal
completion follows from this frontend capability. publication=false;
goal_complete=false. Independent review is still required.
