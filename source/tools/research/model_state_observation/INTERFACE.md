# Relate a model WP's returned resources to the final model heap

This is a supporting mathematical prerequisite for the unchanged full buffer
goal. It is not a native Rust theorem, source translation or publication rule.

`initialized_heap_adequacy` currently concludes a value postcondition that
ignores the final physical state. Input initialization and a pure value result
alone do not establish a final buffer/frame theorem. Supply the missing model
observation rule, using the existing operational semantics and Iris state
interpretation rather than assuming the desired state relation.

For arbitrary finite initial model heap `h`, expression `e`, location `p` and
length `n`, a WP proof may return an exact `n`-element points-to interval plus
a semantic predicate of the returned value and those elements. Derive an
`adequate NotStuck` postcondition relating those SAME elements to actual final
model-heap lookups. The observation may erase lock bookkeeping; do not impose
an unnecessary final lock-count condition merely to make a proof shape fit.
Length must be explicit so an empty returned interval cannot establish a
nonempty buffer property vacuously. Support arbitrary lengths, including 10
and 128, arbitrary initialized byte values, arbitrary interior addresses and
the exact surrounding frame.

Use the checked global initializer to supply initial resources internally.
The caller must not supply an existing owner, PointsTo, chosen final state or
an uninterpreted validity/observation predicate equal to the desired result.
The WP obligation for `e` remains a genuine proof obligation. A useful extension
returns both the final interval and the exact original frame, deriving their
actual final lookup properties. Additional allocations elsewhere may remain
possible unless the WP also rules them out.

Prove operational examples that actually read and write model memory, showing
the observed final value follows the executed body. A checked example with the
same return value and different heap contents should demonstrate why a pure
value postcondition alone does not entail an unchanged-buffer theorem. Proved
resource/state countertheorems and ordinary proof failures must remain separate.

The new typed-input setup lane should expose a generic WP postcondition, so
its reference construction/access/return protocol can later feed this rule.
Do not duplicate that lane's implementation or alter shared modules. Compose
through existing committed proof artifacts and small explicit interfaces.

All new statements must compile normally, have complete assumption inventories
and pass recursive kernel checking with the pinned read-only foundation. No
new axiom, unsafe kernel mode, operational semantic change, trusted contract,
application premise, source-body rewrite or native fact is permitted. Freeze
candidate statements, source and trust inputs before trials.

The initial model convention is allRSt0/global-lock=false/empty freeability;
this is not asserted of every original Rust history. Actual Rust typing,
provenance, ancestor loans, native primitive/phase simulation and original C/F
observation import remain unproved. All 44 minimal native simulations and 101
full-writer effects remain open. Canonical is 6/0; full frozen HMAC prefix AND
untouched suffix is 1/1. This model prerequisite alone cannot complete the goal.
