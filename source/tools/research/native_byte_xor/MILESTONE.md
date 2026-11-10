# Pretrial implementation milestone

The candidate statements implement independent little-endian Boolean words,
word XOR and numeric evaluation; agreement with Z.lxor on every pair of bytes;
byte range; exactly three pure steps of the completed ModelByteXor operation;
and a MODEL read/concrete-call/write WP with supplied old-cell ownership, one
internal store credit, and arbitrary resource frame. An ordinary-data adapter
removes scalar register 19 and preserves all other registers. It is not a native
MIR semantics or a heap-language primitive.

The replay freezes this source, controller, selector, interfaces, committed
source results, shared supports, expected dependency pins and external inventory
before each trial. Compilation and recursive checking write only new private
outputs. Every declaration is inventoried; no new datatype, generated eliminator,
Program obligation or nested module is introduced. Source selection remains
untrusted. No native effect or source correspondence is discharged.
