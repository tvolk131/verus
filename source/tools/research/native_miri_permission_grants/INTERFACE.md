# Current Miri permission predicate

Verify the exact `Permission::grants` executable body from pinned Rust/Miri
58cf8f9c35b355199727627d07ae34a4e2846d67 against an authored exhaustive
permission/access table. Inputs range over every Permission and AccessKind;
there is no precondition, model ownership or application-domain restriction.

The two enum declarations are copied from pinned sources. The adapter places
them inside `verus!`, adds only verifier Structural derivation and exposes the
method for local checking. The selected method's executable tokens stay exact.
Derives, namespaces and the authored contract are inventoried separately.
No proof blocks, theorem calls, ghost snapshots or inferred helpers are needed.

This internal component is used by native grant lookup. Checking it does not
check lookup, a stack/cache, unknown provenance, grant success, source typing,
argument transport, physical bytes, retag/protector effects or original C/F.
The whole Miri package and its semantics are not claimed verified.

The accepted release bundle remains read-only. Bind its eight files to the
successful existing canonical replay, freeze source/controller/tools before
trials, verify with --no-cheating and full local body scope, and separately
reject changed predicates at the semantic postcondition. Ordinary native
execution over the finite eight-input domain is supporting evidence only.

Canonical6/0 and full frozen HMAC1/1 remain inherited, not rerun. Native=0,
44 minimal+3 phase/101 writer obligations open, publication=false,
goal_complete=false. Independent review remains required.
