# Full-request trust boundary

The source-connected request always retains `payment_hash: sha256::Hash`, where
`sha256` is imported from the real pinned `bitcoin` crate. The additional
`ExternalSha256` tuple struct is a Verus `external_type_specification` proxy,
not the request's field type or a substitute digest implementation. Together
with `external_body`, it supplies an opaque logical declaration of the original
Rust type. It supplies no digest representation, constructor equation, equality
law, crypto property, method contract, first-party helper summary or application
precondition. Rust and Verus check correspondence to that named foreign type;
this frontend/type-encoding correspondence remains trusted.

The proof fixture omits the surrounding Debug, Serialize, Deserialize and Clone
derives. The native extraction separately compiles them with pinned serde and
Bitcoin serde support. No derive body or trait behavior is proved. The fixture
extracts the complete seven-field struct, tuple constant and two helpers;
annotations are inserted outside the byte-preserved helper items. This is not a
whole gateway, crate, dependency-feature or callsite proof.

The selected authored deadline policy covers all native scalar values and all
Rust-inhabited foreign hash values without a precondition. Its six clauses are
listed in the frozen inventory. The hash, both amount fields and optional short
channel are present but unread. No funding, amount authenticity, authentication,
settlement, cryptography, asynchronous flow or stateful-fund-safety property is
claimed. Inference checks implementation relations; authors supply the intended
policy and remain responsible for wiring that policy into a real flow.

The existing canonical native Option and integer arithmetic specifications,
checked vstd, Rust compiler/borrow checker/erasure, Verus frontend/backend, SMT
encoding and Z3 remain trusted. Native execution compares finite boundary cases
and is a correspondence test, not a universal proof of Rust or libcore.

The replay always performs full verification, including ordinary authored
bodies, and uses no `--no-verify`, `--verify-only`, module/function selection,
`assume_specification`, admitted goal or unchecked application summary. Outputs,
caches, binaries and raw diagnostics belong in a fresh private persistent `/tmp`
directory. The application clone is read only. `baseline.json`, `original.rs`
and `goals.rs` are frozen by commit before any proof attempt or engine edit.

The inference boundary extension preserves opaque fields only as whole logical
leaves nested inside a visible record borrowed through `&self`. It grants no
root opaque receiver, generic foreign type,
mutable/by-value receiver, sound-only profile, destructor/invariant model,
representation access, constructor or foreign method behavior. External logical
declarations are supplied explicitly; omitting one or naming another Rust type
must fail. The report will distinguish baseline translation support from any
new inference eligibility support and name the executed controls.

Production reconciliation must combine the parent's full-scope publication
fix (policy v64) with this experiment if adopted. This work does not repurpose
v64 or edit shared cache/policy constants. The parent should allocate v65 or
later for the combined new boundary and review a cache-generation change with
its concurrent word-assistance work. Local replay interfaces are experimental
artifacts tied to the recorded binary/source hashes, not distribution inputs.
