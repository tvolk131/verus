# Scope and trust inventory

This lane is an INCOMPLETE implementation attempt. It adds no production hook,
registered foreign function, std model, assumption, axiom or available conversion
contract. The diagnostic VIR function is never inserted into a verification crate
and is not equated with the original Rust FnDef. The structural tests are not
ordinary WF/mode/body proofs, solver proofs or native acceptance.

`inspect.rs` reads actual foreign MIR via rustc enum matches in the actual caller
typing environment. Its independent checker compares the original MIR against
an ordinary VIR AST, using the inherited Boolean cast encoding: Copy of the bool
parameter followed by If(bool, 1u64, 0u64), as a function's implicit return. Exact
MIR shape ensures the return assignment is the only operation. Fresh local AST
IDs/internal raw spans do not impersonate HIR nodes. Actual source-origin text
is retained privately; real HIR call registration remains missing.

Syntactic full-domain contract validation is kept independent of structural
correspondence. Missing/wrong/false-requires proposals reject while the source
body correspondence remains healthy. Copy-to-Move, missing body, swapped branches,
spec parameter/function mode reject structurally. These are typed diagnostic
regressions, not healthy checked-instance semantic negatives. Local cloned MIR
shape tests never change any original compiler/core/source artifact.

Original full Item trials reuse ONLY the inspected pure `source.py` generator and
`inventory.py` reader from item_tag_constructor_composition. No historical epoch
or audit driver is executed. Every ordinary selected/excluded function, original
body hash, generated constructor, local function detail and Eq helper is retained
in RESULTS. Source fidelity erases inventoried ghost blocks and compares all
remaining executable tokens with pinned original bodies. All seven BorTag derives,
private NonZero<u64>/u64 carriers, constructor spelling, u64::from(protected),
assertions and debug assertions remain unchanged. Permission retains the prior
additional Structural marker, not a new native equality proof.

Manual packing support is unchanged: one private proof function packed_fields_raw
with four bit_vector assertions; perm has two additional bit_vector assertions,
protected one. Full new additionally has one quantified packing assertion and
one checked local lemma call per quantified instantiation. Support has two inline
proof blocks; full new has three. No bool-conversion lemma, new callback, runtime
replacement or new application premise. The <=TAG_MASK normal-return bound is
internal support for the original source assertion, never an application budget
or a proof of out-of-range/error/unwind outcomes.

The inherited TCB is the exact Rust compiler/core, Verus translation, mode/type/
WF/logic encodings, matching trusted vstd, inference obligations and Z3. NonZero,
Option and native derived equality retain their prior trust boundaries. A future
registered MIR translation extends the trusted frontend; it does not prove Rust
safety, provenance, physical permissions or original reference control flow.

No full native Rust/compiler/macros/vstd engine was rebuilt or changed. Own VIR
and diagnostic binary builds are separate from the immutable accepted eight-tool
proof bundle. The original bool-cast mirror's 1/0 is only a diagnostic of inherited
cast semantics. Full source failure is a true-property library/proof gap, never a
semantic counterexample. Publication=false, native=0, goal_complete=false.
