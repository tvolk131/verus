# Trust inventory and exclusions

## Untrusted proposals checked normally

- The shared bounded interpreter proposes safety and value formulas. Scalar
  signature checks, datatype checks, traversal/formula budgets and dependency
  binding remain mandatory.
- Newtype projection is candidate-only. Contracts/captures are lifted and
  ordinary verification checks the original executable body and closure binders.
- Callback decorations have separate ordinary safety/equation verification and
  caller-independent safe-domain/output precision checks. Exact-only inverse
  facts are enabled by one shared simplification/lowering predicate.
- Cached formulas are not proof evidence. Soundness, maximality and exactness
  remain fresh; failed goals/imports remove stale interfaces.

## Existing trusted boundaries retained

- The Rust frontend selects canonical native APIs and their checked bridge
  definitions. This correspondence is trusted implementation code, not a proof
  of core's implementation. Checked `then_some` and `map_or` bridge bodies and
  existing Option specifications are unchanged.
- Existing vstd primitive arithmetic, integer conversion, unwrap, byte and
  Option specifications retain their prior assumptions. No specification is
  added or strengthened in this patch.
- Rust type/borrow checking, Verus lowering/backend, the SMT encoding and Z3 are
  part of the verification trust base. Metadata authenticates matching evidence
  and artifacts; it is not a standalone formal proof certificate.
- The integer-unwrap and Option-map successful-execution assumptions remain
  restricted to their canonical maximality gates. Soundness still asserts
  original executable safety, and complete callback precision remains required.

## Deliberate exclusions

Native mutable/stateful Option profiles, callbacks with effects, handwritten
closure contracts, closures stored in arbitrary locals, nontrivial closure
construction, nested closures, callback returns, unsafe code and sound-only
exact-domain facts are not introduced. Allowing a pure finite array literal in
a generated equation does not allow any executable child expression.

The actual Fedimint request still has an unmodeled external SHA-256 hash type.
It receives no invented representation, trusted accessor law or external type
specification. Its successful native compilation is frontend evidence only.
The six-field proof is labeled as a diagnostic subset and is not evidence for
the full request, external-crate equivalence or the gateway. The unchanged
authored policy is the requested intent; exact inference alone does not infer
that intent. No amount, settlement, authentication or stateful funding policy is
claimed.
