# Relate the frozen byte-XOR instruction to the checked operation

The concrete model byte-XOR and its full-domain memory-loop WP are checked.
Relate the actual frozen writer's bb7:6 binary rvalue to that operation through
an independently stated eight-bit Boolean/word semantics. The selected assignment
loads its left operand through `_17`, moves byte local `_19`, computes BitXor,
and stores through the SAME `_17`. Separate scalar computation from the load,
move and store effects; do not claim that the entire assignment is pure.

Prove the eight-bit result agrees with literal Z.lxor for every pair of bytes,
has byte range, and is realized by the existing deterministic three-step model
operation. A useful conditional physical statement should execute the actual
model cell read, checked XOR call and same-cell store, preserve every other
resource, and make any supplied physical cell permission explicit. No pointer
or copied register may manufacture that permission. If register movement is
modeled, the moved scalar becomes unavailable rather than being silently copied.

Authenticate SOURCE.json and its already-pinned extraction/receipt chain before
selecting structured MIR enum payloads. Unknown widths, operations, operand
places and destinations reject. Source identity selects the intended instruction
but never supplies its semantic proof. The scalar interpretation is still
conditional until incoming Rust contexts, native reference loads/stores,
provenance/retags, source phases and original C/F observations are linked.

Use only existing checked libraries and unchanged heap-language semantics.
Freeze statements and inputs before trials, normally compile, inventory every
declaration/constructor/eliminator/obligation and recursively kernel-check. No
new primitive, axiom, trusted contract, unsafe mode, original application premise,
body/domain rewrite or production fact. Native correspondence and publication
remain 0/false; the full original writer and global goal remain unproved.
