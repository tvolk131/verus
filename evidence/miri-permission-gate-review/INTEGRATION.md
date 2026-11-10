# Historical review evidence integration

The complete original review is immutable ancestor `c0b56494` (worker
`3a9885c0`). ARCHIVE.json binds every raw transcript, generated fixture/receipt
and repeated static inventory moved out of the live tree. All archived bytes
were compared with that Git snapshot before removal; the evidence is unchanged.

For example, recover a raw receipt with:

```sh
git show c0b56494:evidence/miri-permission-gate-review/replay002/receipt.json
```

REPORT.md and HISTORICAL_FINAL_INVENTORY.json retain their original bytes and
historical relative paths. They describe that snapshot, not a current replay
recipe. Repeated raw outputs stay accessible in reachable Git history without
adding nearly100,000 lines to the live research context. The worker committed
raw outputs despite the compact-evidence instruction; this integration corrects
the live layout and records the deviation without discarding evidence.

The original result accepts the exact source/helper matrix and controller
revision, and requests changes to two helper replay gates. Parent `3f7303c9`
fixes those gates; subsequent parent replay and independent acceptance remain
separate evidence. No mathematical source, original receipt value, native fact,
trust boundary or application status is changed by archival.
