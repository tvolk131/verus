# Trust and boundaries

The ordinary Verus verifier, Rust frontend, SMT solver and existing vstd library
contracts remain the trusted foundation. The implementation adds no application
axioms, trusted application summaries, unchecked hints, trusted external bodies,
unsafe casts or new vstd contracts. The existing canonical core panic model is
newly recognized at the Infer source boundary; its existing requires-false and
never-returning semantics, rather than a new specification, are used.

`Range<u64>` support inspects the existing external type specification and exact
transparent public layout. It does not admit arbitrary external proxies,
destructors, invariants, hidden fields or opaque representations. Recognition
remains gated on crate/path/proxy identities, parameter bounds, types, modes,
visibility and exact fields. Unknown panic operands retain eager evaluation and
are rejected by the closed profile instead of being dropped.

Inference proposes contracts; it does not certify them. Every accepted helper
still requires ordinary-body soundness, safe-domain maximality and independent
output exactness. Publication remains conditional on successful whole-fixture
verification. Imported summaries retain policy, proof, dependency and trusted
model binding. Record/owned candidates are regenerated, including in warm runs;
cache poison is not interpreted as trusted formulas.

Generic caller properties are ordinary proofs. Optional constant arithmetic
assistance emits independently checked theorems, preserving original safety and
publication barriers. An auxiliary failure is neither an ordinary semantic
counterexample nor a safety-precondition failure. Native runtime checks provide
finite supporting evidence only. Actual debug-on/off configurations and native
release wrapping behavior are recorded separately. No general mutable-state,
whole-database or Rust-library correctness claim follows from this slice.
