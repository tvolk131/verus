# Reproduce the independent MODEL review

Read REPORT.md and INTERFACE.md for the accepted scope. Existing committed
review-freeze.json authenticates the named completed run; never overwrite it or
its output. The original wrapper invocation was (record only, do not rerun into
this completed output):

```sh
python3 -B evidence/source-unsize-model-review/review.py \
  --output /tmp/native-source-unsize-model-review-58ea8ca4-001
```

The wrapper is single-use. To make a fresh proof replay, use the root's unchanged documented command with an unused
private output (normal coqc is used even though Python runs with -O):

```sh
python3 -B -O source/tools/research/native_source_unsize/check.py \
  --freeze source/tools/research/native_source_unsize/trial004-freeze.json \
  --output /tmp/native-source-unsize-model-review-NEW-UNUSED
python3 -B source/tools/research/native_source_unsize/test_source_binding.py -v
python3 -B -O source/tools/research/native_source_unsize/test_source_binding.py -v
python3 -B source/tools/research/mutable_buffer_goal/test_goal_scope.py -v
python3 -B -O source/tools/research/mutable_buffer_goal/test_goal_scope.py -v
```

The original independent prepare/run commands and every actual coqc/coqchk
invocation are retained in commands.json; review-freeze.json was committed
before the proof/control trials. The completed raw input map is retained privately, not in Git.
The wrapper's prepare/run lifecycle is single-use: a new independently frozen
review would need a newly assigned evidence directory and an unused output.
No Rust suite or shared build mutation is needed.
