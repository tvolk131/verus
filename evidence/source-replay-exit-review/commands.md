# Exact review commands

Executed from this isolated checkout, Python bytecode disabled throughout:

```sh
python3 -B evidence/source-replay-exit-review/review.py prepare
python3 -B evidence/source-replay-exit-review/review.py tests
python3 -B evidence/source-replay-exit-review/review.py replay constructor
python3 -B evidence/source-replay-exit-review/review.py replay pointer
python3 -B evidence/source-replay-exit-review/review.py replay exit
python3 -B evidence/source-replay-exit-review/review.py supplement constructor
python3 -B evidence/source-replay-exit-review/review.py supplement pointer
python3 -B evidence/source-replay-exit-review/review.py supplement exit
python3 -B evidence/source-replay-exit-review/finalize.py history
python3 -B evidence/source-replay-exit-review/seal.py
```

Each replay expands to its current README's Python -B -O command with the
current integration freeze and fresh `/tmp/source-replay-exit-review-bc2a0b37-002/`
output. That private root is single-use. To reproduce, choose another unused
RAW path in review.py, freeze that harness before preparation, then execute
these commands. No changes to reviewed controllers, candidates or shared
artifacts are needed. Supplements require their corresponding replay complete.
History and tests are independent of replay order. Freeze finalize.py and seal.py
with their adjacent SHA256 files before running them.

All actual expanded coqc/coqchk argv vectors are retained in the named replay
receipts. Every top-level review command vector, exact cwd, elapsed time, status
and stdout/stderr hash is in results.json and private `*-command.json`.
Seven provided test_goal_scope.py cases run with normal Python and -O.
Source inspection supplies the controller guard review; no security experiment
or source-on-disk mutation test is conducted.

The initial pretrial attempt used the private root without `-002` and stopped
before any proof command due to the historical-TARGET mapping error documented
in REPORT.md and preparation-failure.json. No output from that attempt is
mathematical or semantic evidence.
