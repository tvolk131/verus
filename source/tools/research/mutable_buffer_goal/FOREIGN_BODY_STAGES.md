# Checked foreign bool conversion: staged implementation boundary

The actual core `From<bool> for u64` instance and its whole typed optimized MIR
have been inspected. The proposed ordinary VIR has not run through the verifier.
The [parent record](parent-bool-mir-diagnostics.json) authenticates the diagnostic
evidence and a fresh0/1 true-property failure. No foreign law is available.

The first implementation milestone is local verification under `--infer-goals`.
Resolve the actual direct call in its caller typing environment; admit only the
semantic core From instance with exact bool input/u64 output and empty final
substitutions. Inspect every typed MIR operation and all locals/blocks/drop
properties. Independently check original MIR against the generated ordinary VIR.
Use fresh internal identities, normal Copy(bool)/cast/return semantics, and
preregister the full function and stub before Query collects function stubs.

Check that function and the original caller with normal WF, modes, trait,
lifetime, resolution and ordinary solver/body checking. The complete bool-domain
functional contract is a proposal checked against the actual lowered body. A
shape match, generated AST or local Rust mirror supplies no proof. If inference
is used, all mandatory soundness/maximality/exactness checks remain required.
Original runtime call identity and erasure stay intact. Do not equate original
FnDef types with a synthetic function. Other conversion shapes gain no law.

For this bounded milestone, EVERY export path for a crate using the capability
must refuse and remove its selected stale VIR output, including ordinary export,
cargo export, private-helper pruning, failure and incomplete verification paths.
Reject no-verify, no-lifetime and function/module-filter runs of the capability.
No foreign contract may cross a crate boundary. Unused translation proposals
must not alter ordinary crates or their inherited standard models.

This deliberately stages the earlier all-four-interface research charter:
checked same-crate calls may be demonstrated while exports remain unavailable.
It does not weaken the required eventual publication/import guarantees. Local
success must be reported separately from a publishable capability.

The next milestone, after the local checked path exists, binds foreign-body,
semantic-instance, compiler/core/target and transitive model identities in
post-proof metadata. Imports must freshly resolve/revalidate current identities
before any dependent closed facts become usable, including transitive imports.
Parent owns policy70/cache73 and any schema reconciliation. No new std model,
FromSpecImpl<bool>, obeys-from-spec law, assumed specification, external body,
application precondition or native/provenance theorem is authorized.

Acceptance requires actual From success, a healthy false-result control,
unchanged original Item.new with its protector assertion, explicit generated
body/query identities, goals-off baseline, cold/warm checks, and enforced export
refusal. Certified import and whole-goal acceptance remain later obligations.
