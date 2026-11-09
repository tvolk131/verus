# Frozen exact scalar tuple output profile

Frozen before implementation on combined parent db26cfc7, 2026-11-09.

Extend exact-mode inference only with flat Rust tuple values of arity 2,
with each component bool or an existing machine scalar integer
(u8/u16/u32/u64/u128/usize/i8/i16/i32/i64/i128/isize). No nested tuple,
Option/Result tuple payload, generic, recursive, heap, reference tuple or
approximate tuple inference. Inputs remain the pre-existing input vocabulary;
nonempty tuples are not new input types. Immutable whole-tuple locals and
scalar field reads, existing if/else and checked exact callee composition
may carry the new output. Tuple destructuring is outside this small profile.
No new loop or effect form is supported. Every constructor operand is evaluated
and contributes its safety obligation, including unused components.

Use native typed tuple constructors/projections and whole-value equality.
No authored accessor laws or observations. Original-body soundness, independent
maximal-safe-domain and arbitrary WHOLE-result exactness remain mandatory and
must not depend on candidate-selected projections. Missing/weak/swapped
components must reject in independent precision tests. Source/type/dependency
identity and parent policy66 full-scope/provenance guards remain enforced.
Reserve cache70 and policy67 if needed; no schema change unless needed.
Cache entries remain candidates and require fresh proofs; honest misses allowed.

Target: unchanged original data_index_of(bit:u32)->(usize,usize) body
((bit as usize)/64,(bit as usize)%64), no authored contract; separate 32/64
usize model proofs, zero/max u32 cases and separate native execution evidence.
Add unrelated tuple arithmetic, operand safety, branches/locals/callee tests,
cache poison, metadata/import/failing-export tests. Create a NEW stateful fixture
from automatic_inferred_mask.rs with only the tuple helper equations removed
and infer added. Keep both historical experiment directories unchanged.
Canonical word definitions remain explicit; literal baseline and forwarding
wrappers remain blocked. No automatic discovery of semantic policy is claimed.
