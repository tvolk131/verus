# First-cell target139: original two-phase acknowledged durability

## Required end state, not a new weaker cell

Target the original acknowledged-durability/two-phase cell of campaign94.
Use the original single-process, exclusive-writer, sequential-transaction,
current-format byte-key/value, Durability::Immediate configuration and arbitrary
finite two-phase histories within actual native resource/representation limits.
Keep naturally reachable growth, shrink, allocation, buffering, errors, close,
reopen and interrupted-recovery dependencies. Do not replace this by fixed file
geometry, a tiny tree, header-byte persistence alone, a selected body, or an
assumption that the next helper starts in a valid state.

For a native publicly acknowledged transaction, allowed crashes and recovery must
not lose its committed effects, except for changes legitimately superseded by
later transactions under the original history specification. The exact native
roots and the data reachable from them matter, not just model payload labels or
an encoded transaction ID. Failed/interrupted operations and syncs retain their
allowed partial effects; only actual successful barriers provide durability.
The final claim must compose with the remaining original properties rather than
borrowing an assumed native validation/recovery oracle or checksum injectivity.

## Existing checked work to reuse

- storage109/110: complete buffered flush and barrier snapshot reasoning;
- acknowledgement109/110: ordered barriers through public acknowledgement,
  including post-publication errors and explicit callback frame gates;
- header/recovery111: original nominal serialization/staging and recovery
  composition under their explicit inherited boundary and byte/domain premises.

These are dependencies, NOT accepted native producers. The native bodies have
not established all their input premises. A caller frame gate is not proof that
native callbacks preserve that frame. Model payload labels are not physical slot
indices, and a header ID is not a logical key/value refinement theorem.

## Immediate implementation milestone

Establish a checked source-connected resource path for the original
WritablePage::mem_mut/copy/drop and its cache reservation/return, so actual
write_header serialization can be related to actual buffered bytes. Begin with
the original memory-loan methods and exact primitive requirements; avoid another
general-purpose resource feature that has no native caller consumer.

Use existing VIR/SST/AIR resource infrastructure where applicable. Native source
identity, typed call instances, actual places/activations/loan ends and cleanup
contexts must be bound by a checked producer. Drop identity129 is supporting
evidence, not that producer. ScopeNone/zero exact e78 Scope edges must remain
explicit; no span heuristic or hardcoded e78/local20 becomes a certificate.

The parallel primitive-contract task must specify only what this path needs,
with exact native API/cfg/profile identities, affine resources and outcome/frame
effects. Unused Arc downgrade/Weak/progress work is not a priority. Its absence
may be claimed only with actual closure evidence, not assumed. None of the
native LRU/cache/backend application methods may be externalized as primitive
contracts to make the proof pass.

## Closure gate

Before closing the cell, independently check exact primitive dispositions,
original executable-body scope, source-to-resource correspondence, native
checkout/reservation/copy/return/buffer/flush/public-acknowledgement effects,
later-history and crash/recovery preservation, required tree/allocator effects,
and inherited source/compiler/trust/admission provenance. A verified local loan
method is useful progress, not the cell. Mark missing items OPEN, keep the full
18-cell matrix, and report the next actual application dependency after each
milestone. All eighteen cells remain OPEN at authorization139.

## Parent discovery failures retained

Read-only discovery in this round guessed nonexistent
source/rust_verify/src/checked_normal_drop.rs and acquisition128 INPUTS.json.
Actual matching source/evidence paths were then used. These are discovery
failures, not verification outcomes or permission to reset any old allowance.
