# Localized semantic controls

The frozen positive is buffered_flush109.rs, SHA256
29485224c99ba2f994012ff53b2ddc98c9cab79daaf7b3fc2a855b952d2fdc4f.
Each mutant changes only the indicated backend/control semantics, not theorem
premises, proof assertions, verifier options, or imports. The primary source is
restored with apply_patch after each mutation. Per-attempt input.rs snapshots,
raw diagnostics, invocation and receipts retain all actual variants.

1. Rollback-on-write-Err mutant: in write_visible replace
   `decision.ok || decision.changed.contains(address)` with `decision.ok`.
   The localized partial-write witness must fail: a failed write is permitted
   to change only byte zero of a two-byte range and that byte may persist.
2. Eager-drain-on-write-Err mutant: the failure branch of model_stripes replaces
   the failing pending stripe by an empty sequence. Retention and the arbitrary
   successful-prefix theorem must fail.
3. No-op-successful-sync mutant: backend_sync returns before on Ok rather than
   clean(before.visible). The symbolic sync/durability contract must fail.

The final restored positive must have the exact frozen SHA256 and pass a full
crate check. These are semantic controls of authored model definitions, not
tests executing the native application or a native compiler/source binding.
