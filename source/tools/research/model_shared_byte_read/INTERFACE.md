# Shared byte-key read interface

Implement a checked MODEL indexed read from the existing semantic shared
reference type `&shr{κ} (bytes_ty n)`, using its genuine ordinary E/L context.
The view is `((q, repeat [] n), x)` with `x : ~~(bytes_sort n)`; a tctx binding
at `#q` or the equivalent `ty_own` plus explicit time/depth witnesses is welcome.
For every `i<n`, execute the actual existing `Read ScOrd #(q +ₗ i)` and return
the corresponding byte, unchanged shared view and restored ordinary context.
Keep arbitrary separating frames, including other copies of the shared key
capability. Derive byte range from the existing byte type. State every credit,
guard and step cost; do not replace the shared resource with supplied exclusive
PointsTo or require a caller-provided guard. Empty keys have no valid index;
there must be no unconditional first-cell access. Support arbitrary key lengths.

The parent needs a reusable persistent/key read capability or equivalent law
that it can pass through a mutable-pad callback. A concrete XOR model loop may
use the existing checked operation law, but the primary handoff is the smallest
working read interface and its exact premises. A same-heap observation lemma
for the unchanged key is useful if derivable from the shared guard and genuine
heap context. Do not turn it into unsupported exclusive ownership extraction.

Reuse the exact checked byte representation from NativeSemanticBridge and
ModelInputContext. No edits to baseline/vstd/compiler/shared tools. Only this
new directory is owned. Freeze before trials, normal compile, inventory all
declarations and obligations, recursive kernel check, compact receipts/hashes;
raw logs and objects remain private. No axioms, trusted summaries, unsafe
settings, client application premises or fixture changes. Actual Rust/native
correspondence remains zero and publication/full-goal completion false.
