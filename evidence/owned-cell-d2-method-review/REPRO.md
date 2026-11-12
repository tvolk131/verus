# Reproduction

Run from the frozen prepared checkout plus these evidence-only commits, with
Rust 1.99.1 and the authenticated snapshots still available. Choose NEW output
names; controllers refuse existing directories. No network or shared build cache
is required. Maintain at least 25 GiB free. Commands below are local only.

```sh
python3 evidence/owned-cell-d2-method-review/authenticate.py /tmp/d2-review-auth-NEW
python3 evidence/owned-cell-d2-method-review/validate.py replay /tmp/d2-review-replay-NEW
python3 evidence/owned-cell-d2-method-review/validate.py vir /tmp/d2-review-vir-NEW
python3 evidence/owned-cell-d2-method-review/validate.py vir-lib /tmp/d2-review-vir-NEW
python3 evidence/owned-cell-d2-method-review/compatibility.py /tmp/d2-review-replay-NEW/snapshot /tmp/d2-review-rev-NEW
python3 evidence/owned-cell-d2-method-review/boundary.py /tmp/d2-review-boundary-NEW /tmp/d2-review-vir-NEW
```

The `vir` command deliberately preserves the complete Cargo result: 172 unit
tests pass, then 18 documentation snippets fail. `vir-lib` records the separate
complete successful unit scope using this review's own fresh build. It does not
repair or mask the earlier failure. The boundary test catches and explicitly
reports D2-R1; its successful harness exit is not a claim of no panic.

Replay uses the original native executable exactly as requested:

```text
checks/native-query-tests --exact checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions --nocapture --test-threads=1
```

The original root refusal/admission executables are authenticated separately,
then relocated unchanged beside D2 tools. Their 5/7 test counts and 226 CLI
invocation inventory follow the original harness; the replay logs record test
summaries, not 226 new standalone command receipts. The native unit includes
both query groups. It is a compiler-unit test, not a successful CLI feature request.

Every subprocess uses existing LoggedCommands/ArtifactGuard/replay_receipt.
Exact argv, environment overrides, tool/source hashes, wall seconds, raw stdout/
stderr paths and SHA256 receipts are in the private receipts referenced by
VALIDATION.json. BINDING.json pins the eight tools and original native test.
Receipt paths include unique labels, not asserted timestamps.

The synthetic boundary fixture uses canonical builder scaffolding extracted
from the frozen source and the old descriptor fixture, adapted to synthetic
spans/IDs. The new boundary tail supplies the actual independent API challenges.
It never authenticates loaded native MIR, runs SMT or earns source/native proof
credit. Its generated source is private and hashed in its receipt.

The first wrapper failure and first full-Cargo failure remain at the original
paths in VALIDATION.json. Neither may be relabeled successful. No old native or
VIR executable is used for D2 validation; the fresh VIR-only build and worker-built
compiler/native-test artifact replay have separate provenance.
