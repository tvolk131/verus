# Independent conservative reborrow MODEL review

Scope and prerequisites: [INTERFACE.md](INTERFACE.md). Pretrial commit
`ceb9bce2`; frozen review-controller initial commit `0a0834ce`, corrected final freeze `261d2258`.

Replay (fresh output path required):

```
python3 -B evidence/source-reborrow-model-review/review.py \
  --output /tmp/source-reborrow-model-review-6c74c9b1-FRESH
```

Uses the current `integration001-freeze.json`; does not replay the historical
worker freeze against changed documentary inputs. Completed receipts and exact
sources/objects remain authenticated. No installs, Rust builds or native
acceptance. The verbose recursive kernel run adds declaration coverage evidence
beyond the implementation's silent recursive check.

Final measured review reused the successful fresh review002 compilation:

```
python3 -B evidence/source-reborrow-model-review/review.py \
  --reuse-completed /tmp/source-reborrow-model-review-6c74c9b1-002/replay \
  --output /tmp/source-reborrow-model-review-6c74c9b1-AUDIT-FRESH
```

Both output paths must be new; the explicit reuse path is pinned in
controller003-freeze.json and all its inputs/products are authenticated before
inspection. Original failed controller freezes remain historical evidence.

[REPORT.md](REPORT.md) and [results.json](results.json) accept the exact
conditional model and current replay controller. Native-source coverage,
activation/write, cleanup and C/F remain open; publication=false, goal incomplete.
