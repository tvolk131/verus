# Private continuation receipts

Final build root is the owned PRIVATE_ROOT from checkpoint462c21e9. Final matching
applications/tests are in its `sealed-validation` namespace, using `bundle-sealed`.
Original checkpoint and prototype epochs remain immutable outside that namespace.
No historical VIR or sibling runtime is a positive input.

Read-only final audits from this worktree root:

```
WRAPPER_EPOCH=sealed python3 -B source/tools/research/nonzero_wrapper_ordering/implementation_audit.py
WRAPPER_EPOCH=sealed python3 -O -B source/tools/research/nonzero_wrapper_ordering/implementation_audit.py
```

Recorded final production build is `resume.py build`, then normal `resume.py vstd`,
with WRAPPER_EPOCH=sealed. The latter verifies2250/0 and freezes eight tool hashes.
`finalize.py freeze` creates an additional before-proof freeze for the separate
receipt namespace; all subsequent finalize modes use the SAME eight bytes.
Do not rerun completed command labels or rewrite PRIVATE_ROOT. Controllers reject
existing receipts instead of overwriting them. Replay generation needs a fresh
owned receipt namespace and an offline matching build, not deletion of history.

Final modes: semantics, exports, compatibility, shapes, proposals, partial-barrier,
test-build and test-run. final_gates.py adds actual Cargo-driver prior/global/
late-erasure/filter cases; mir_inventory.py records all seven actual loaded MIR
bodies plus erased expansion/MIR. All command/environment/source/controller/tool
freezes and last guards remain private. IMPLEMENTATION_RESULTS.json contains only
compact receipt and diagnostic hashes, exact classifications and inventories.

The solver copy was authenticated before any resumed vstd/application proof:
/tmp/local-nonzero-ord-09t1tshb/bundle/z3,
SHA256 e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765.
Original and own-copy hashes/executable mode are retained in the private solver
receipt and RESUMED_BASELINE.json. No other file in that bundle was accessed.
The separately authorized checksum-listed Cargo inputs and hash-pinned Item source
remain the only other completed private inputs used.
