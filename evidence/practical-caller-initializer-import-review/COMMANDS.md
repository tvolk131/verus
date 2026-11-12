# Executed focused commands

All Python executions used normal Python (`-B`, **without `-O`**). The source controllers were unchanged. All compiler commands, exit codes, durations and mode flags are recorded exactly in RESULTS.json and the raw receipts. No native preparation command was rerun.

```sh
python3 -B source/tools/research/practical_caller_transitions/fedimint_bootstrap/replay_initializer.py --output /tmp/practical-caller-initializer-import-review-46b1fce9/a-positive
python3 -B source/tools/research/practical_caller_transitions/fedimint_bootstrap/replay_initializer.py --scenario client --provider /tmp/practical-caller-initializer-import-review-46b1fce9/a-positive --output /tmp/practical-caller-initializer-import-review-46b1fce9/a-client
```

Each of `written-none`, `length`, `others`, `count`, `safety` was executed separately with this command, substituting the same scenario for both placeholders:

```sh
python3 -B source/tools/research/practical_caller_transitions/fedimint_bootstrap/replay_initializer.py --scenario SCENARIO --output /tmp/practical-caller-initializer-import-review-46b1fce9/a-SCENARIO
```

```sh
python3 -B source/tools/research/practical_caller_transitions/fedimint_bootstrap/replay_initializer.py --scenario failed-import --provider /tmp/practical-caller-initializer-import-review-46b1fce9/a-positive --failed-export /tmp/practical-caller-initializer-import-review-46b1fce9/a-count/infer.vir --output /tmp/practical-caller-initializer-import-review-46b1fce9/a-failed-import
python3 -B source/tools/research/practical_caller_transitions/redb_imports/replay.py --freeze source/tools/research/practical_caller_transitions/redb_imports/FREEZE_3.json --output /tmp/practical-caller-initializer-import-review-46b1fce9/b-imports
```

The five A controls and dependent client/failed-import were sequenced by a local `subprocess.run(..., check=True)` loop. The independent B replay overlapped those runs; timings are not benchmarks.

Read-only audit operations used `git rev-parse HEAD`, `git status --short`, exact ancestor-file reads, bounded `rg`/`sed`/`cat`, and one-off Python SHA256/JSON/tar inspection. The hash audit checked FINAL_FILES and all registered type/expansion source/archive/library inputs, compared published macro archive members byte-for-byte, mapped historical worker source filenames to hash-identical fixed-checkout copies, authenticated named receipts, compared all three original runtime bodies and both generated conversion bodies, reversed all eleven provider declaration changes, and inspected raw scope/SMT/publication diagnostics. INPUT_HASHES.json and RESULTS.json record exact inputs and checks. The scratch audit initially used the key `runs` instead of `fresh_parent_runs` in the parent manifest; it stopped, then was corrected. This was an extraction error, not a verification failure, and changed no historical input.

Before commit: `git diff --check`, JSON parsing and rehashing of recorded fresh logs/providers and immutable input bindings; `git status --short` and staged-path inspection. Only this new directory is committed.
