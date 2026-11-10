# Frozen interface, revision 1

Status: implementation boundary investigation. No foreign contract is available.
Frozen before production edits; this lane must fail closed if any stage is absent.

```text
resolve(TyCtxt, caller: DefId, callee: FnDef, args: GenericArgs)
  -> Result<ResolvedCoreBoolFrom, IdentityRejection>
inspect(ResolvedCoreBoolFrom, instantiated rustc::mir::Body)
  -> Result<PrimitiveBody, UnsupportedMir>
translate(PrimitiveBody, fresh binder/read/span supply)
  -> OrdinaryVIR(ReadPlace(Copy(bool)), If(_, 1u64, 0u64))
correspond(original typed MIR, original resolved instance, OrdinaryVIR)
  -> Result<Correspondence, TranslationMismatch>
register(Correspondence, checked ordinary function, exact direct HIR call)
  -> Result<BodyBackedCall, RegistrationRejection>
prove(BodyBackedCall, no requires, result == if b { 1 } else { 0 })
  -> Result<CheckedBody, ProofFailure>
publish(CheckedBody, complete crate/lifetime/trait checks,
        compiler/core/target/MIR/model/dependency fingerprints)
  -> Result<ForeignBodyCertificate, PublicationRejection>
import(certificate, freshly resolved actual foreign body and dependencies)
  -> Result<CheckedInstanceContract, ImportRejection>
```

Identity is semantic: diagnostic From trait in the actual core crate (identified
through a core lang item), its associated function and resolved impl, exact bool
input and u64 Self/output, safe Rust ABI, monomorphic Item instance, empty final
substitutions. DefId numbers and impl display names are diagnostic labels only.
The driver must query the typing environment of the actual caller. Rustc's
compiler/core/target identities and all tool/artifact bytes are frozen separately.

The body has exactly two non-dropping locals, one argument, one non-cleanup block,
one `_0 = Copy(_1) as u64 (IntToInt)` assignment and Return. No omitted executable
operations are permitted. The source-origin span is retained separately from
fresh internal VIR IDs; the helper is never a replacement Rust FnDef value.

The ordinary contract is a proposal independent of body admission. Missing,
false, restricted-domain or wrong-typed proposals cannot yield an available
contract. A recognizer result or structural correspondence is not a proof token.
The ordinary body must undergo the normal mode/type/WF/body pipeline. If inference
is used, all three exact mandatory checks are fresh, including cache/import paths.

Publication must be delayed until complete success even without --infer-goals.
Ordinary legacy early exports, unchecked/filter/no-lifetime runs, private-helper
pruning, and transitive consumers must not discard foreign dependencies. Imports
must validate actual current MIR/semantic identity before closed facts are usable.
Until register/prove/publish/import exist together, admission is diagnostic only.

No vstd model, FromSpecImpl<bool>, obeys_from_spec=true, axiom, source replacement,
external body, no-unwind premise, FnDef equivalence or native theorem is authorized.
This extends a trusted frontend translation of rustc MIR, not a Rust safety,
borrow/provenance or physical-permission theorem. Parent owns version changes.
