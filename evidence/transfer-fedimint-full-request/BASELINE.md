# Frozen full-request target

`baseline.json`, `original.rs`, and `goals.rs` are frozen before any verifier
attempt or engine change in this experiment. The target is the actual seven-field
request, including its real external SHA-256 type, with both helper methods
byte-preserved. No amount or cryptographic policy is targeted. The authored
policy has no precondition; all six intended safety, coverage, boundary and
monotonicity clauses remain required. See the machine-readable inventory for
the complete domain, proposed adaptations, trusted boundary and validation plan.
