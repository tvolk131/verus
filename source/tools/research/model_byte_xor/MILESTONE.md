# Typed milestone (before trial001)

The frozen `candidates/byte_xor.v` proposes a compile-time 256-by-256 literal
`Case` table, two symbolic substitution laws, two symbolic lookup laws, and a
`PureExec True 3` theorem returning the literal `Z.lxor a b` for all byte inputs.
The execution uses only existing application and Case reductions. No client
heap or credits occur in the operation theorem.

`wp_byte_xor` targets the completed ModelBufferTransform binary-operation
interface. `wp_xor_memory` targets arbitrary byte lists at their minimum length,
with the full semantic prefix, original suffix, preserved key and byte closure.
`wp_xor_memory_128` keeps all key lengths. The physical loop's internal
permissions and store credits are explicit. No public raw-heap adequacy is
claimed here, and no native Rust/MIR correspondence is claimed.

This milestone is a statement/source/trust freeze, not proof success. Trial001
has not run. `trial001-freeze.json` binds the controller, support inputs, actual
candidate, dependency-set digest and declaration inventory. Read-only loop
receipt authentication and ready AND successful durable-foundation authentication
are required by the controller before normal compilation and recursive checking.
No constructors, Program declarations or generated obligations are introduced.
