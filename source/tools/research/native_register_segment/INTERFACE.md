# Preserve register data flow across a supported constructor segment

The accepted primitive refinement gives independent input/output words per
edge. Connect the actual adjacent constructor assignments at bb2:1 and bb2:2:
`_6 = unwrap(_3)` then `_5 = Offset(_6,_2)`. The second operation must consume
the pointer produced by the first, with the original length retained in `_2`.

Use explicit descriptor register maps and the existing checked fragment
source_step relation. Show the two assignments, exact output register values,
preservation of every other register and physical heap, and one whole target
expression whose lexical temporary carries that pointer between operations.
Preserve arbitrary separating resources across actual existing lrust execution;
no descriptor may manufacture physical ownership. Inputs, numeric bounds and
source/model representation premises must remain explicit.

This is a conditional MODEL program composition, not a proof that the original
Rust context supplies those input descriptors. Do not cross StorageLive/Dead,
raw reference creation, reborrow, retag, call/drop, phase or cleanup gaps by
identity. The unchanged native constructor and full writer remain unproved;
all 44 minimal and 101 writer effects remain open, publication false. A source
pin identifies the requested segment but is never a semantic proof fact.

Freeze candidate statements and all proof/trust inputs before trials. Normal
compilation, complete declaration/constructor/eliminator inventories and full
recursive kernel checking are required. No new axiom, unsafe mode, operational
semantic change, application premise, original-body rewrite or production fact.
