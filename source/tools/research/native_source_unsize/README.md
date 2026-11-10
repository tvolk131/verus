# Array-to-slice carrier MODEL

The checked target is the representation suboperation before reference
validation, retagging and local write hooks. The complete Rust unsizing
statement is **not** established. See [the interface](INTERFACE.md).

The frozen source site is the original constructor's root `bb0[4]`, with
source `_4: &mut [u8;128]` and destination `_3: &mut [u8]`. Generic 10/128
carrier laws do not establish a common native proof for the two applications.

Replay the current source/target-width checks and unchanged mathematical laws:

```sh
python3 -B -O source/tools/research/native_source_unsize/check.py \
  --freeze source/tools/research/native_source_unsize/trial004-freeze.json \
  --output /tmp/native-source-unsize-fresh-UNUSED
```

[Results and complete assumptions](results.json) and [the report](REPORT.md)
record the exact accepted replay. Trials001/003 passed with earlier controllers;
trial002 is retained as a source-schema setup failure, not semantic evidence.
The current replay pins the completed parent receipt directly so later review
prose does not invalidate the mathematical dependency.

Native correspondence remains zero; all44+3/101 obligations and full HMAC
remain open. Publication=false; full goal incomplete.
