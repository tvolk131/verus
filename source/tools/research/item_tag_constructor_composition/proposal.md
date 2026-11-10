# Typed future foreign-body bridge (proposal only)

Observed pinned compiler: Rust 1.99.1, commit
58cf8f9c35b355199727627d07ae34a4e2846d67. Ordinary caller invokes actual
`<u64 as core::convert::From<bool>>::from`, not a mirror. `rustc_private`
resolves `DefId(2:40391 ~ core[efd5]::convert::num::{impl#51}::from)` to
`Instance { def: Item(...), args: [] }`, signature `fn(bool) -> u64`.
Definition path display uses std reexports; actual defining crate is core.
DefPathHash fingerprint `(17281971461272426270, 1802834199475816209)` is
compiler-specific. DefIds alone are not stable cross-build identities.

Fully instantiated original foreign MIR: one input, return local `_0:u64`,
input local `_1:bool`, one non-cleanup block, exactly one statement
`_0 = copy _1 as u64 (IntToInt)`, then `Return`. Both locals need no drop.
Definition at core/src/convert/num.rs:50, cast line51, return line52. The
original macro's body is `b as Self`; u64 invocation is line58. Full source,
caller and instantiated MIR, compiler/sysroot hashes and logs stay private.
No block, statement or terminator in this instantiated MIR was omitted.
Optimized instantiated MIR is the inspected representation, not every earlier
compiler IR or an independent validation of compiler optimizations.

Proposed narrow typed admission:

```
ForeignPrimitiveBody {
  compiler_and_core_identity, resolved_original_DefId_and_Instance,
  original_mir_hash, source_hash_and_span,
  parameters: [(b, Bool)], result: U64,
  ordinary_body: Return(CastU64(ReadCopy(b))),
  requires_candidate: true,
  ensures_candidate: result == (if b { 1u64 } else { 0u64 }),
}
```

Necessary implementation/proof work, NOT implemented in this lane:

1. Resolve the *actual* trait-call instance in the caller's typing environment;
   bind semantic core crate identity, exact compiler/core artifact, source/MIR
   fingerprints, constness, target and substitutions. Never select by spelling
   or DefId number alone. Reuse the same identity for imported dependencies.
2. Reject unavailable MIR, intrinsic/virtual/shim instances, generic/region or
   const ambiguity and any type except primitive Bool input/U64 output. Require
   exactly two locals, no drop, no cleanup, one block, one assignment and Return.
   Reject projections, references/pointers, reads/writes through memory, calls,
   unsupported casts, aggregates, asserts, branches, drops and unwind edges.
3. Translate Copy(_1) to ordinary typed parameter read, IntToInt Bool->U64 to
   the existing checked Boolean integer cast, assignment to ordinary return,
   and Return to normal function return. Preserve the entire instantiated body
   and source spans. A pure whitelist observation does not itself prove that
   mapping correct: review/validate original MIR -> ordinary VIR correspondence,
   primitive cast semantics, type/WF/mode obligations and all failure behavior.
4. Register a body-backed function at the original resolved call target, not a
   local `FromSpecImpl` or trusted replacement summary. Run normal full lifetime,
   trait, mode, WF and body checking with `--no-cheating`. Require soundness,
   full-bool inherited-domain maximality and output exactness on that translated
   original body before making the candidate available to callers.
5. Reconcile the conditional inherited From spec without asserting
   `obeys_from_spec=true`: consume the separately checked instance summary at
   this exact resolved call, or introduce a checked instance mechanism whose
   law itself follows from those obligations. Import/export must reject absent
   proofs, changed dependencies/target/body or unsupported instances; exported
   closed summaries require exact proof/model/producer fingerprints.
6. Freshly check the untouched complete Item.new with all debug checks and its
   existing bound, plus full-bool conversion and healthy false-result controls.
   Preserve larger-tag assertion outcomes in the scope statement. A normal-return
   theorem is still not whole-source/all-outcomes/native acceptance.

The local checked cast mirror diagnoses existing cast encoding only. It proves
nothing about the actual foreign body registration or usable library contract.
No axiom, assume_specification, external body or bool FromSpecImpl was added.
No physical provenance/reference lowering conclusion follows from primitive MIR.
