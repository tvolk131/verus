# Bounded frozen-engine Arc instance/binding trace

Resolve whether existing authenticated artifacts can establish the frozen
compiler's treatment of original `<Arc<[u8]> as AsRef<[u8]>>::as_ref` used by
native `PageImpl::memory`. This is a source/provenance trace, not proof execution
or verification of Rust-core ownership, lifetimes or allocator semantics.

First spend at most two minutes locating an already available build-corresponding
engine source basis from existing runtime/build receipts. Inspect midpoint
runtime-binding.json, build invocation/receipt and any explicitly named base
compiler provenance. Its source-original.json captures vstd, not a whole engine
checkout. runtime-binding.source contains source hashes, but a vstd-build input
manifest does not by itself prove that unchanged compiler binaries were compiled
from those source bytes. Trace the actual base-binary build relationship. Do not
assume current Git HEAD, live engine source or equal filenames supply that link.
Never fetch/reconstruct a checkout, change Git or copy a source/toolchain tree.

If the source basis is not established within that bounded search, stop this
dependent trace and report the missing link precisely; do not keep scanning all
temporary files. If established, authenticate each relevant available source
before reading and trace only the Arc as_ref/unsized deref/instance specification
path needed by the original immutable memory body. Separate ordinary call/model
lowering, generic trait resolution, unsized borrowed slice view and actual
native source-to-resource production. Do not infer runtime behavior from a
symbol name or silently equate a vstd external specification with checked native
resource semantics. Existing external/library assumptions are not newly adopted.

Deliver a compact decision matrix: frozen binary identity/build source basis;
original call instance/domain/cfg; existing lowering/spec route and guarantees;
what remains absent for native buffer provenance and nominal PageNumber/codec
binding. Conclude whether existing support justifies any immediate checked task.
If not, specify ONE narrowly bounded future engineering authorization with exact
affected source areas, validation/semantic controls, no new assumptions and
unchanged S0/export/adoption barriers. Do not implement it or claim authorization.
The conclusion must distinguish a missing source receipt from a missing semantic
producer, and conditional model verification from a native resource theorem.

Work yourself with no subdelegation, only fresh `/private/tmp` output. Parent,
providers, frozen inputs/runtimes read only. Authenticate supplied inputs at entry
and exit, all newly used files before substantive read and at exit; disclose
ordering/availability limitations honestly. Historical105's planning README was
reconciled only after its endpoint checks; exact old bytes are preserved. Use the
new input table rather than treating its old212 context hashes as current.

Hard6minutes TOTAL including setup/authentication/report/seal; stop optional
analysis at4minutes and reserve2minutes for finalization. Report<=10KiB, scratch
<=8MiB, free disk>16GiB. Establish/enforce an absolute deadline, retain accounting
and input/output hashes. No verifier/solver/compiler/build/test/native execution,
network, Git writes, parent edits, source implementation, runtime/toolchain/cache
copies, subdelegation, trust/axioms/assume/admit/external bodies, admission/export
changes or adoption. Do not resume/reroute growth049/zero-growth050/mutation030/
refused-padding95/drop-pipeline work; inspect only immutable view call binding,
not destruction or Arc accounting. No proof credit/acceptance; all18 OPEN.
