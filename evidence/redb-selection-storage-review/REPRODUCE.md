# Focused reproduction

Use a clean checkout containing reviewed parent 03fd44ed, the pinned read-only accepted78 bundle/receipt and REDB checkout named by CASES, and Rust1.99.1. Do not change parent freezes or status files. `prepare.py` checks frozen source hashes against the committed parent and remaps only the named paths into this checkout. It writes review-owned freeze copies and exact remapping records.

Run from repository root, choosing unused output directories (the commands below are this review's exact paths; do not rerun over existing outputs):

```sh
python3 -B evidence/redb-selection-storage-review/prepare.py
python3 -B -O source/tools/research/redb_crash_protocol/selection_contract/validate.py --freeze evidence/redb-selection-storage-review/selection-freeze.json --output /tmp/redb-new-review-03fd44ed-selection
python3 -B -O source/tools/research/redb_crash_protocol/storage_boundary/probe.py --adapter --freeze evidence/redb-selection-storage-review/storage-positive-freeze.json --output /tmp/redb-new-review-03fd44ed-storage-positive
python3 -B -O source/tools/research/redb_crash_protocol/storage_boundary/probe.py --adapter --wrong-adapter --freeze evidence/redb-selection-storage-review/storage-negative-freeze.json --output /tmp/redb-new-review-03fd44ed-storage-negative
python3 -B source/tools/research/redb_crash_protocol/model/replay.py --label review_normal_positive --output /tmp/redb-new-review-03fd44ed-model-normal-positive
python3 -B -O source/tools/research/redb_crash_protocol/model/replay.py --label review_optimized_positive --output /tmp/redb-new-review-03fd44ed-model-optimized-positive
python3 -B source/tools/research/redb_crash_protocol/model/replay.py --candidate source/tools/research/redb_crash_protocol/model/controls/error_implies_rollback.rs --expected postcondition --label review_normal_negative --output /tmp/redb-new-review-03fd44ed-model-normal-negative
python3 -B -O source/tools/research/redb_crash_protocol/model/replay.py --candidate source/tools/research/redb_crash_protocol/model/controls/error_implies_rollback.rs --expected postcondition --label review_optimized_negative --output /tmp/redb-new-review-03fd44ed-model-optimized-negative
python3 -B evidence/redb-selection-storage-review/import_ordinary.py
python3 -B evidence/redb-selection-storage-review/audit.py
python3 -B evidence/redb-selection-storage-review/package.py
```

The supplemental importer uses the frozen strict run's two compiled providers and removes only optional `--infer-goals` from healthy client commands. Audit/package scripts make no implementation edits. Packaging reads historical raw paths solely for hash comparison after all fresh receipts have passed. Historical raw files are optional for semantic reproduction; their absence is recorded as a hash-comparison mismatch, not a theorem failure. Large private products stay in /tmp; text copies are in receipts/.
