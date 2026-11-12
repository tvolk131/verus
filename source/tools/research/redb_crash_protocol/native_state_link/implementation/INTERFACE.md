# Prospective implementation interface (before trials)

Base: 0dd6130f. Design: all five files in ../design/. This checkpoint is a plan,
not evidence of an installed native bridge. Conservative clock start:
2026-11-12 14:39:00 UTC; 45 active minutes per blocker, builds excluded.

Only production/test files listed in ../design/IMPLEMENTATION_TARGET.md (and its
explicitly permitted new checked_owned_cell.rs/lib.rs declaration) are owned.
Research changes stay in this directory. No delegation, live sibling/parent
reads, shared mutable caches, remote publication, trusted laws or version bumps.

## Requested interface and admission grammar

Explicit --experimental-owned-unsafe-cell, false by default, requests checking
in ordinary and Infer alike; it supplies no authority. Reject incomplete checks,
missing --no-cheating, imports/exports of experimental authority, including
zero-helper and both explicit/Cargo output paths. Invalidate stale outputs before
reuse. The experiment stays inactive/fail-closed if any mandatory gate is missing.

Normalize complete local typed unit BEFORE pruning, summary inference or caches:
S := () | u64 | bool | tuple(S,...)
C := actual loaded core UnsafeCell<u64>
V := S | C | &mut C | &mut u64
No cell tuples, shared references, raw pointers, generics/projections, other ADTs,
containers, closures/captures, function values, callbacks, drop/unwind, external
or abstract bodies. Specs contain scalars/tuples only. Calls are the three fully
resolved original methods or acyclic fully checked locals. Unknown rejects.
Normal Rust native !Sync, privacy, WF, lifetime, trait, safety, mode, erasure and
full ordinary body checks remain required.

## Source/proof relation

Validate loaded native lang item/core identity, native generic ADT/private field,
method identity, ABI/signatures including get_mut's equal input/output region,
and every typed MIR operation/local/terminator. Body-backed ordinary VIR must
construct/read/reborrow the SAME nominal native datatype/field/place, preserving
moves and get_mut writeback. Independent correspondence checks actual emitted
Function, types, places and read kinds. A source attribute, owner module, path
spelling, layout, hash, cached proof or descriptor is not permission.

Field access authority must be local compiler-owned, operation-bound,
nonserializable and rechecked at WF/backend. Rev remains unchanged. No PCell,
world state/token premise, native substitution, new axiom, assume_specification,
external_body, scalar-only elimination or shared selector.

## Prospective checked inventory and state goal

Unchanged original clients: owned_roundtrip, exclusive_update, exclusive_borrow.
Native checked bodies: actual new<u64>, into_inner<u64>, get_mut<u64>.
Mandatory diagnostic: SAME exclusive_borrow runtime body with additional final
native input field = next, arbitrary initialized entry and no new premise.
Record actual generated body/helper counts after inspection, not a guessed count.
Delete store/writeback while returning next: state proof AND correspondence must
fail. Wrong-place, field/read/move/type/region corruptions must reject.

## Gates and stop

All gates are ../design/ACCEPTANCE.md. Authenticate frozen 11 inputs and eight
accepted78 tools, retaining known ordinary/Infer unsupported baseline without
rerunning it. New builds require distinct private verified compiler/macros/vstd
identity and fresh outputs; never replace accepted78. Strict inventories, both
modes repeated, semantic false return/frame/state, typed escape and imports,
privacy/corruption, native lifetime, export stale-output, original --compile and
erased native calls must all pass before acceptance. Incomplete remains inactive.

Full Mutex (including atomic Transmute), sharing/acquisition/poison/guard/drop,
LRUWriteCache and full REDB crash theorem remain UNPROVED. native_bridge_proved,
source_coupling_proved, CP1, CP2 remain false. HMAC/native DEFERRED AND UNACHIEVED.
