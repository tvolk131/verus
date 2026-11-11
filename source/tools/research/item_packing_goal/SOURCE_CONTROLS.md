# Preregistered arithmetic source controls

After the shipping-source cold/warm full146/0 and packet32/0 runs, use the
same frozen application bytes and unchanged compiler/models/runtime translation,
changing only owned checked_foreign_packing.rs:

1. Proposal ablation: replace the final `for fact in proposed` with iteration
   over an empty `Vec<Exp>`. This disables only arithmetic assistance; bool-body
   verification, Eq proof checks and every ordinary body/check remain enabled.
   Require both full targets to fail while the ordinary bool helper and original
   full-domain Eq checks remain healthy. Compare exact original typed bodies.
2. False closed constant proposal: add one to the proposed literal RHS of each
   constant-fold equality. Require ordinary isolated BV failures to reject the
   complete verification; the healthy bool helper remains checked. This tests
   untrusted recognition/proposal acceptance, without changing application bytes.

Each variant gets its own source snapshot, build, matching standard vstd, fresh
application paths and frozen receipts. Archive the completed shipping tools
before work-path reuse; restore shipping bytes afterwards and rebuild matching
compiler/vstd before a final production replay. No fault is committed or used as
positive evidence. Missing or failed proofs cannot count as acceptance.
