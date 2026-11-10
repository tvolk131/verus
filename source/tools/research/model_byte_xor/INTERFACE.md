# A checked executable byte-XOR operation in the existing heap language

The generic physical memory-loop law is checked, but its binary-operation WP
is an explicit premise. Implement that operation concretely for byte XOR:
for every a,b in [0,256), ordinary execution returns exactly Z.lxor a b and
preserves the heap. Prove byte-range closure too. Use the unchanged existing
VerusBelt language; it has integer arithmetic/comparison/Case but no built-in
XOR instruction. A checked finite table or an eight-bit arithmetic expansion
is acceptable. Do not add a primitive, assume a summary, or use nondeterministic
selection/divergence to evade computing XOR. Establish deterministic finite
pure reduction, not only a partial-correctness WP for a possibly diverging op.

Match ModelBufferTransform.transform.binary_operation_spec: two integer
arguments as literal expressions, result the literal Z.lxor, no client heap
resource or credit premise. The completed generic library is namespace
ModelBufferTransform, trial007. Instantiate its memory law at
min(length pad,length key), deriving the full XOR prefix plus untouched suffix
and unchanged key. Include arbitrary byte vectors and pad length 128 with every
key length, including 0 and >128. Keep the operation implementation proof and
the composed full-domain MODEL loop theorem explicitly separate from native
Rust/MIR correspondence and the unchanged original writer theorem.

Freeze actual candidate statements/source/trust inputs before trials. Normally
compile, inventory all definitions/constructors/generated obligations and
assumptions, and recursively kernel-check. All output is private. No new axiom,
trusted model, semantic primitive, unsafe mode, operational change, application
premise, original-body rewrite or domain restriction. Source assignment and
native instruction/loop/loan/C-F interpretation remain open; publication false.
