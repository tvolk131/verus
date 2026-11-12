# Maintained Infer inputs

These files are source inputs to the ordinary Rust integration harness, not
stored compiler output. `tests/infer_*` and `tests/constructor_function_values.rs`
keep the options and expected outcomes. `provenance.toml` records the archived
origin and SHA-256 of every fixture. Move existing sources without changing
bodies, specifications or domains; review intentional changes to these inputs.

- `applications/redb_allocator.rs` joins the six previously checked allocator
  components in their original order. `full_semantics` selects the accepted
  interval, validity, rejection and page-conservation wrappers. Six negative
  cfgs retain their original semantic/safety controls. Counting, database
  integration and crash/durability correctness remain unproved.
- `applications/fedimint_quorum.rs` retains the original numeric methods and
  authored quorum/honesty predicates, with checked set assistance. This is not
  signature validation, signer admission or protocol correctness.
- `buffer_observation` retains checked prefixes, known-unproved full suffixes,
  alias/origin regressions and semantic negatives.
- `varint` retains both manual and assisted canonical-u64 proofs, including their
  authored semantic invariants. `word_sequence` retains bitmap propagation.
- `constructor_functions` exercises the original tuple-constructor function value.
- `native_cell` is the source input to the separately archived native experiment;
  its original clients have no accepted body/state/erasure proof.

The complete historical reports, upstream pins, adaptations and limitations are
preserved at commit `7dcabedaa93299a3607746b6ec25f5e4778288a4`, local tag
`archive/infer-research-2026-11-12`. The unfinished native experiment is also
preserved on `experiments/infer-native-cell`. To inspect a historical file:

```sh
git show archive/infer-research-2026-11-12:path/from/provenance.toml
```

Historical replays belong to that revision and require their recorded inputs;
the tag alone does not preserve private tool binaries or guarantee portability.
Fresh runs write receipts/logs to temporary directories or CI artifacts. Do not
check generated run transcripts back into this directory.
