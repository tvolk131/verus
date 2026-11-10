# Reproduction

Use this prepared checkout's exact frozen library files and the named completed
`/tmp` artifacts. Historical worker checkout paths are labels only. Do not use
either root live parent replay. No dependency installation or shared build is
needed or permitted by this review.

The accepted private review run was frozen before execution:

```sh
python3 -B evidence/fixed-reader-shared-loop-review/review.py run \
  --output /tmp/fixed-reader-shared-loop-review-5904e347-trial001
```

Its output directory must not exist before starting. The committed review freeze
binds that original run and local guard identities; it is not overwritten for a
replay. To reproduce the exact library checks in new unused private outputs,
run the unchanged original controllers with their own recorded freeze names:

```sh
python3 -B source/tools/research/model_pair_reader/check.py \
  --freeze source/tools/research/model_pair_reader/final-freeze.json \
  --output /tmp/fixed-reader-review-REPLAY-UNUSED
python3 -B source/tools/research/model_shared_xor_loop/check.py \
  --freeze source/tools/research/model_shared_xor_loop/trial011-freeze.json \
  --output /tmp/shared-loop-review-REPLAY-UNUSED
```

These perform normal compilation, individual assumption inventories and full
recursive coqchk. Compare every `.v`/`.vo` hash and source-assumption list with
the authenticated completed workers in `review-freeze.json`; `review.py`
documents all additional revision, baseline and retained-artifact guards.
`results.json` records the actual fresh commands, outputs and comparisons.

The supplemental inventory uses only Require/Check/Print Assumptions on those
already compiled and recursively checked objects. It adds no declarations:

```sh
python3 -B evidence/fixed-reader-shared-loop-review/inventory.py run \
  --output /tmp/fixed-reader-shared-loop-review-5904e347-inventory001
```

Its committed `inventory-freeze.json` predates the queries; that output must
also be unused. The source count is 28; total including generated marker
helpers is 108. Raw type output and helper assumption output stay private;
their hashes and full inventories are in `inventory-results.json`.

`python3 -B` is used throughout. New code, input or tool identity requires a
new review freeze and private trial, retaining failures as setup/development
outcomes. Source hashing does not rerun the inherited canonical or HMAC proofs.
