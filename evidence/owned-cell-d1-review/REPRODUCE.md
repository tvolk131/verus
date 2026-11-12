# Exact replay scope

All executed argv/cwd, individual elapsed times, exits, source guard outcomes,
private receipt/stdout/stderr paths and SHA256 values are in RESULTS.json.
Private output is `/tmp/owned-cell-d1-review-973ca52d`; do not reuse/overwrite it.
Prepared production source is exactly `5c503b74955b2594e91d7f0b2809541936f53d5b`.

For a fresh replay, change only the OUT constant in the two evidence Python
controllers to the same new private directory before starting commands. Keep
the committed scripts as the original record. The existing check.py setup
creates a new directory, authenticates all eight accepted baseline tools and
offline archives/index/git against frozen manifests/Cargo.lock, and copies a
private dependency cache. Then run these actions in order:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 source/tools/research/redb_crash_protocol/native_state_link/implementation/check.py setup --output NEW_PRIVATE_OUT
PYTHONDONTWRITEBYTECODE=1 python3 evidence/owned-cell-d1-review/review.py build
PYTHONDONTWRITEBYTECODE=1 python3 evidence/owned-cell-d1-review/review.py native
PYTHONDONTWRITEBYTECODE=1 python3 evidence/owned-cell-d1-review/review.py vir
PYTHONDONTWRITEBYTECODE=1 python3 evidence/owned-cell-d1-review/review.py rev
PYTHONDONTWRITEBYTECODE=1 python3 evidence/owned-cell-d1-review/review.py combined
PYTHONDONTWRITEBYTECODE=1 python3 evidence/owned-cell-d1-review/boundary.py
```

Use the pinned Rust1.99.1 toolchain/core artifact and immutable D1 snapshot
receipt `c15b5585f14cb9c5441b25b3667d3aa31ac34fed3ad37830a3edfdaacfcfe0ed`.
The seven supporting runtime tools are reused only after all eight frozen
hashes authenticate. The compiler must be built from prepared combined source;
never substitute the old D1 compiler for combined/F1-fixed runtime claims.
No full vstd rebuild is performed or claimed. check.py/infer_experiment.py/
infer_eval_support.py supply execution and guards; boundary.py separately
guards its external freshly compiled rlibs since check.py's source manifest
accepts repository-relative paths only. Every compiler test invocation uses
private outputs and caches. Do not edit a guarded controller while it runs.

The final boundary fixture intentionally asserts the three observed omissions
so the witness passes on reviewed D1. After hardening, change those assertions
to require rejection; do not interpret this diagnostic witness as permanent
acceptance of malformed metadata. Its Ctx tests use descriptor-only baseline
GlobalCtx, not a claim of complete malformed-crate frontend acceptance.
