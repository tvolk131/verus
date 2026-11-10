# Functional native boundary audit: frozen scope and question

Audit baseline: committed checkout `759644e2c14430d2715889484aeb7b9cf77f32e6`.
Evidence-only ownership: `evidence/native-functional-boundary-audit/`.
No implementation, publication, rule/contract/status change, build, regression,
subdelegate, external dependency, or live proof-output inspection is authorized.
Inspect committed source/history and explicit completed-artifact inventories only.
The pending complete SC writer is restricted to its INTERFACE, not writer.v or
live outputs. No client owner/PointsTo/guard or additional application premise.

The exact proposed judgment to investigate is:

    ExistingTrust |- forall actual Rust input configurations S, p, k, A, K,
      ValidOriginalInput(S,p,k,A,K) ->
      FunctionalSimulation(original frozen MIR, checked memory projection)
      /\ NormalReturnAdequacy(original Rust/Verus C/F, projected final memory)

Here A is any initialized byte sequence of length 128, K any valid initialized
key slice (including length 0 or >128), p may be an interior array subobject,
and disjoint pad/key ranges may share a containing allocation. The conclusion
must entail XOR on exactly [0,min(128,|K|)) and the unchanged suffix, with the
same original reference's final view. ValidOriginalInput must describe source
inputs independently of model ty_own or the desired postcondition.

Question: can the functional proof erase borrow/provenance history while using
ONLY specifically demonstrated inherited safety/stdlib trust, or does that
silently assume safety/defined execution of unverified unsafe std bodies?
A normal-return-only functional statement is not permission to make defined
execution, no-unwind, retag validity, or normal termination an extra input
condition. Such premises must be classified as inherited, proved, or missing.
Ghost MutRef.ptr is never identified with the physical pointer. A MIR operand
identity can anchor a projection only with an explicit source/observation rule.

Baseline status is unchanged: canonical 6/0, full HMAC 1/1, native correspondence
0, minimal 44 + 3 phase obligations / full writer 101 effects open,
publication=false. Model theorems are evidence within their stated model only.
The root integration lead retains every success/trust/integration decision.

Deliverable: a cited static audit, exact source-validity predicate, obligations
and assumptions table, decisive acceptance/rejection of the proposed trust
shortcut, and one concrete minimal next task. Hashes/selector flags bind identity
only and will not be used as semantic proof facts.
