# Shared checked observation ablation

Private diagnostic builds against committed parent
`d35c31fae9d05d7c0e797a60612a8be4716c2dc9`; no production proposal.
`freeze.json` and the two patches were committed before application trials.
The pretrial controller revisions fix spec-definition scope spelling, diagnostic
header counts, the normal vstd output spelling, the ambiguous aggregate error
flag, and explicit preparation/mandatory-status gates. No application trial
preceded these corrections. The initial freeze is preserved in
commit `f519a410`; the final pretrial freeze is committed separately before all application trials.

The question is whether checked observer proposals perform necessary mechanical
proof work for the unchanged **full canonical encoder** and the unchanged
**Zip prefix diagnostic**. The full Zip HMAC writer also remains in the matrix,
with its arbitrary initialized 128-byte buffer, arbitrary key length and required
unchanged suffix. These are byte-writer claims, not HMAC/security proofs.

## Intervention

- `observer-off.patch`: return the input functions unchanged at the observer's
  preparation entry. Numeric assistance and all other production components
  retain their parent source.
- `capacity-off.patch`: omit only the observer's
  `len(iterator.seq()) == capacity` invariant (for Zip, capacity is
  `min(128, key.len())`). Keep both entry/final snapshots, their length facts,
  and the element correspondence. Adjust the independent projection's exact
  invariant insertion count from four to three; preserve all projection, ID,
  purity, ordinary WF/mode/lifetime/trait and proof checks.

The production JSON observer inventory hardcodes four invariant proposals.
Under capacity-off this selection report remains four although the patch and
whole-function projection bind the actual count to three. That report is never
used as proof evidence.

The build controller hashes the entire baseline tracked source/tool/dependency
inventory, records the sole altered file, builds its own release compiler and
macros with `CARGO_BUILD_JOBS=2`, and verifies matching vstd normally. It does not
pass application `--no-cheating` to vstd. It copies actual compiler, launcher,
macros, builtin/vstd libraries and solver to a private bundle, checks its version,
then restores only the modified observer file to its exact base bytes in a
`finally` block. Every application invocation uses `--no-cheating` and the
hash-bound bundle/solver. No version constants or trusted facts are changed.

## Replay and validation

Use the checked-in `experiment.py` actions from the repository root. Raw output
must go into a new private directory; existing output directories are rejected.
The delivered baseline bundle must be present and match its frozen manifest.
The setup action validates that manifest, all eight runtime artifacts, baseline
engine-source identities, Rust 1.99.1, Verus version and Z3 4.16.0.

```sh
python3 source/tools/research/shared_observation_ablation/experiment.py setup --output /tmp/NEW/setup
python3 source/tools/research/shared_observation_ablation/experiment.py build --variant observer-off --output /tmp/NEW/observer-off-build
python3 source/tools/research/shared_observation_ablation/experiment.py trials --variant baseline --output /tmp/NEW/baseline-trials
python3 source/tools/research/shared_observation_ablation/experiment.py trials --variant observer-off --bundle /tmp/NEW/observer-off-build/bundle --output /tmp/NEW/observer-off-trials
python3 source/tools/research/shared_observation_ablation/experiment.py build --variant capacity-off --output /tmp/NEW/capacity-off-build
python3 source/tools/research/shared_observation_ablation/experiment.py trials --variant capacity-off --bundle /tmp/NEW/capacity-off-build/bundle --output /tmp/NEW/capacity-off-trials
python3 source/tools/research/shared_observation_ablation/summarize.py --raw /tmp/NEW --output /tmp/NEW/summary.json
```

Do not rerun `freeze`: that action is intentionally restricted to an unmodified
base HEAD before the first trial. Replay consumes the committed freeze.

Each runtime checks canonical/prefix/full with goals enabled and disabled, cold
then warm at the same source path and private cache path. Zero inferred helpers
are explicit, so cold/warm does not claim a helper-cache speed benefit. Local
executable inventories are exactly `{u64}` and `{xor_key}`, proof inventories
are empty, and original spec definitions are inventoried separately by the
production scope report. Strict false-property and overflow controls use the
existing observation replay's exact source transformations, plus the existing
plain-writer positive control. No application-local proof glue is added.

Shared `infer_experiment` validators bind success, full selected scope, exact
body/helper inventories, mandatory exact-helper checks and export behavior.
Ablated failures use `validate_run` and an explicit multi-obligation diagnostic
gate; they are not strict single-kind semantic negatives. Frontend, mode,
resource, recommendation, panic or partial-scope failures are invalid causal
evidence. The compact auditor rechecks raw receipts and healthy preparation/
mandatory statuses, stores hashes and exact diagnostic text once per pair, and
requires identical cold/warm outcomes. Raw setup/build/failed-trial logs remain
private with receipts. Export gates here check fresh output presence/absence;
seeded valid-export removal and cache/metadata poison controls remain in the
parent matching regression scope.

## Acceptance boundaries

Full HMAC suffix remains required and unproved. This ablation cannot complete
the shared-automation goal, even if necessary observer work is confirmed for
canonical and the prefix diagnostic. Native correspondences remain **0**; all
**101** native effect obligations remain open. **Goal complete: false.
Publication: false.** Successful ordinary local export artifacts are only a
publication-gate control, not whole-goal acceptance. Fresh parent matching
regressions and independent acceptance remain required.
