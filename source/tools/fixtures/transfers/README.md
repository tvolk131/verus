# Transfer test inputs

These are source slices, authored specifications, source ledgers and expected
scope/contract data used by the maintained transfer oracles and their tests.
Source bodies and contracts are copied unchanged from `evidence/` at commit
`37fe7b453e621fa9b0ff37e8d18c813a9e1888a8`, preserved by tag
`archive/verus-infer-pre-integration-2026-11-13`. The native-composition Rust
input is shared with the canonical Rust fixture directory rather than duplicated.

Run outputs, old receipts, review prose and private-machine handoff packages
remain in that historical revision. These inputs do not certify a fresh run,
native correspondence, crash durability or cryptographic correctness. Scenario
oracles retain their exact source/domain/contract expectations. Write fresh logs
and receipts outside Git.

The opaque-selector expectation file rebases fixture paths/helper hashes and
current engine identity. Historical certificate JSON remains unchanged; unit
tests construct explicitly synthetic current-identity parser inputs in memory.
Those are rejection-oracle tests, never fresh compiler proof evidence.
