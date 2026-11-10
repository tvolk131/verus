# Interpret the frozen native constructor operations in the existing heap language

The selected unchanged no-write body is the frozen native `drop.rs`. Its
authenticated source data covers six concrete instances and 44 MIR operations,
plus separate pre-erasure/optimization and scope/cleanup phase obligations.
The earlier Verus trace proves transitions in an authored fragment machine;
it does not connect those transitions to VerusBelt's physical heap language.

Build a checked connection for a precisely stated supported subset: a typed
source-operation syntax and execution relation, a translation into existing
VerusBelt expressions, and operational/physical simulation proofs. A producer
may reify the actual frozen MIR, but source matching and hashes cannot supply
an effect theorem. Inspect real typed enum payloads, CTFE branch values,
resolved call bodies, layouts and operands. Generated proof artifacts must be
ordinary checked Coq terms, not unchecked trace certificates.

The first useful supported operations are the exact constructor's metadata
extraction, pointer thinning/NonNull representation and bounded end-pointer
Offset, preserving the same physical location and byte heap. Prove the pointer
arithmetic bounds and one-past/no-dereference distinction for arbitrary strict
interior array subobjects, not only allocation-base pointers or zeroed bytes.
Copies duplicate metadata, not linear ownership. Use the existing model-loan
protocol at ownership boundaries rather than inventing ghost authority.

Do not define native retag as identity or erase unresolved reservation,
activation, FakeRead, cleanup/unwind, drop, source-phase or provenance issues.
Keep unsupported operations explicit, and reject them in any claimed complete
translation. Query-only drop glue is not an extra executed destructor. A
simulation between an independently authored source-fragment semantics and
the existing model is useful but remains conditional until the source
semantics is linked to the actual Rust execution/typing context. Report that
remaining link explicitly; no source hash or name is a proof of it.

Freeze interfaces and candidate statements before trials. Normal compilation,
complete assumption/obligation inventory and recursive kernel checking are
required, using the existing pinned read-only toolchain. No new assumptions,
unsafe kernel modes, installs, application premises, compiler/vstd changes,
body/domain substitutions or production publication. Keep exact coverage of
proved versus unsupported operations; do not claim all 44/101 effects from a
small pointer law. The original native-drop and full writer remain unproved
until the original source loan, execution and C/F observations are connected.
