# AlephBFT source-slice evaluations

These fixtures and `eval_verus_infer_alephbft*.py` scripts exercise bounded
source-connected slices, supplied semantic views and caller properties. They do
not prove consensus safety, cryptographic correctness or the whole application.
Every scenario records its source fidelity, adaptations, models and remaining
boundaries in fresh output. See the main [capability table](../../docs/guide/src/infer_capabilities.md).

## Run a scenario

Use a matching built Verus/vstd/solver and a local AlephBFT checkout containing
the exact commit pinned by the scenario. Run `--help` on the selected script for
its additional source/dependency paths and optional native comparisons.

```sh
python3 -B source/tools/eval_verus_infer_alephbft.py \
  --alephbft /path/to/AlephBFT \
  --verus /path/to/matching/verus --z3 /path/to/matching/z3 \
  --output /tmp/new-aleph-vote-evaluation --native-tests
```

Each output must be fresh. Native comparisons support the proof investigation;
they are finite executions, not universal source proofs. Modified runtime
adapters and specification inputs must remain disclosed separately.

## Scenario families

| Script suffix | Selected area |
| --- | --- |
| base script | Common-vote representation and scalar dependency boundaries |
| `_parents` | Quorum/parent selection with an explicit collector source |
| `_collections`, `_counting` | Bounded collection views, effects and supplied invariants |
| `_records`, `_wrappers`, `_private` | Record/wrapper and private-view composition |
| `_traits` | Trait getters and associated payloads |
| `_lifecycle`, `_selection` | Construction/advancement and borrowed queries |
| `_units` | Unit/Hasher body and observation interfaces |
| `_native_collector` | Original collector and selected finite parent queries |
| `_validation` | Typed noninitial validation, construction and public guards |

Helper contracts and checked native sub-bodies are not substitutes for the
registered caller property. Failures, unsupported code and model assumptions
remain part of the result. Use `test_infer_alephbft*.py` through the maintained
[tooling runner](../README.md) to validate evaluator behavior without a full
application build.

The chronological experiments, concrete earlier outcomes and counterexamples
are preserved in the prior version of this file at
`archive/infer-research-2026-11-12` (commit
`7dcabedaa93299a3607746b6ec25f5e4778288a4`). Their recorded source commits and
runtime identities remain historical; consult current fresh results before
claiming an enabled feature or improvement. Keep generated full reports in
`/tmp` or CI artifacts, not beside these fixtures.
