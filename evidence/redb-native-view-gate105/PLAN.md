# Fresh source-only native Arc/view capability gate

Goal: determine whether the retained runtime/vstd already has a justified binding
for the original native `PageImpl::memory` on `Arc<[u8]>`, and a checked relation
between its native page-number type and the imported page codec. Assessment104
inspected native dataflow and seven-crate interfaces, not actual matched
vstd/engine support. This is a prerequisite audit, not a closure theorem.

Read the independently reviewed103 theorem boundaries,104 source report and
actual original native PageImpl memory/page-number bodies. Inspect the matched
midpoint runtime's authenticated vstd sources, especially std_specs/smart_ptrs,
unsized Arc/as_ref and view/borrow specifications. Keep nominal native versus
projected PageNumber definitions distinct. If inspecting the live parent native
resource admission/producer code, label it separately: current source is not
automatically the frozen midpoint binary. Check only immutable view/borrowing
support; do not investigate excluded destruction, mutation, growth or padding
pipelines, reconstruct Arc accounting, or route around any prior refusal.

Produce one concise matrix for: actual Arc<[u8]> view/as_ref availability;
borrow/view lifetime stability; original native PageImpl::memory body binding;
native PageNumber/codec representation relation; actual fetch-return buffer
provenance. For each give source anchor/hash, symbol/type/domain/profile, and
whether it is checked implementation/producer evidence, an existing external
specification/axiom, an authored projection, or absent/unestablished. Identify
what can and cannot follow under existing S0/fail-closed admission. A source
function named Arc or a trusted standard-library spec is not a checked Rust-core
resource theorem. Do not adopt laws or propose an abstract fetch oracle.

If a required justified binding is absent, give the smallest precise missing
producer/semantic obligation and an unexecuted source-anchored gap fixture only
if useful. Do not manufacture proof success with extra premises or bodies. If
all relevant support already exists, specify a separately bounded next native
verification task and what existing assumptions it would use; still execute
nothing in this audit. Distinguish conditional standard-model evidence from
native resource correspondence and surface any consequential trust/scope decision.

Work yourself, no subdelegation; all writes only in fresh `/private/tmp` scratch.
Parent/providers/runtime/sealed evidence read only. Use supplied input pins,
authenticate every newly used source before substantive inspection and at exit,
and disclose ordering limitations rather than inventing entry records. No large
source/toolchain/cache copies. Respect applicable AGENTS.md. Use apply_patch for
authored files, no inline code comments or one-letter identifiers.

Hard limit8minutes TOTAL including setup/authentication/report/seal; stop optional
analysis at minute6 and reserve2minutes for finalization. Report<=16KiB, scratch
<=8MiB, free disk>16GiB. Enforce an absolute deadline; partial is preferable to an
overrun. No verifier/solver/compiler/build/test/native execution, network, Git
writes, parent edits, new source implementation/trust/axioms/assume/admit/external
bodies, admission/export policy or runtime adoption. Do not resume/reroute
growth049/zero-growth050/mutation030/refused-padding95/drop-pipeline work. No
native or final acceptance; all18 remain OPEN. Return report, input entry/exit
hashes, accounting and manifest/seal identities.
