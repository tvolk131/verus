# Reconstruction and replay

Implementation worktree:
`/Users/tommyvolk/.t3/worktrees/verus/t3-e661febe`.

The parent baseline is
`/private/tmp/verus-cycle77-review-fixes.XCkOUU/handoff/combined.apply_patch`,
SHA-256 `545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12`.
It was applied exactly once to the tool-provided clean checkpoint
`3b05c1912b9d7314ad00c832c4065f4362562e02`. Do not reapply it here.

The incremental patch at
`/private/tmp/redb-slot-selection77-aofyb_lo/handoff/delta.apply_patch`
applies **only to that dirty77 baseline**, once. It contains additions only.
Its SHA is in the adjacent `delta.sha256`; `changed-files.json` enumerates each
path, size and SHA-256. `reconstruction.json` verifies every baseline and added
file against an in-memory baseline-plus-delta reconstruction. No new checkout,
branch, staging, commit, compiler rebuild or dependency fetch is needed.

From an authorized, already reconstructed dirty77 worktree, apply just this
delta using `apply_patch`, then run:

```sh
python3 -B -m unittest discover -s source/tools -p test_redb_slot_selection.py
python3 -B -O -m unittest discover -s source/tools -p test_redb_slot_selection.py
python3 -B source/tools/check_redb_slot_selection.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/verus \
  --output /private/tmp/redb-slot-selection-fresh-normal
python3 -B -O source/tools/check_redb_slot_selection.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/verus \
  --output /private/tmp/redb-slot-selection-fresh-optimized
```

Each output must be new. The runner selects the sealed Rust compiler from the
frozen toolchain ledger for scratch harness compilation, uses private unused
Cargo paths and sanitized `GIT_*` context, and runs no Cargo/build/fetch command.
The runner never applies the baseline or changes refs.

The final executed outputs for this cycle are
`/private/tmp/redb-slot-selection77-aofyb_lo/final-normal/` and
`/private/tmp/redb-slot-selection77-aofyb_lo/final-optimized/`. Their
`receipt.json` files contain all command/status/report details; `artifacts.json`
binds generated sources, raw logs, libraries, exports, import mutants, native
binaries and cache entries. `replay-summary.json` binds those receipts and gives
compact per-case results. The full artifacts remain at those exact paths rather
than being misrepresented as embedded binaries in this source-only delta.

The blocked full-enum probe remains blocked by design. A successful replay means
the reduced-error scaffold and controls passed and the expected native blocker
was reproduced. It explicitly does **not** mean full native selection/recovery
verification or independent baseline approval.
