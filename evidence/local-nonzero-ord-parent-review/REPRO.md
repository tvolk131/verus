# Reproduction and evidence locations

All commands are local. No network, push, PR, or remote publication. Start only
in a new authorized isolated checkout of prepared7fde950c with the review files;
controllers use immutable labels and will reject overwriting completed receipts.
`build.py setup` allocates an own private root, copies authenticated completed
Cargo archives/index/git and Z3, and snapshots own tracked source. It writes this
review directory's PRIVATE_ROOT. It does not use historical positive VIR.

```sh
python3 -B evidence/local-nonzero-ord-parent-review/build.py setup
python3 -B evidence/local-nonzero-ord-parent-review/build.py build
python3 -B evidence/local-nonzero-ord-parent-review/build.py vstd
python3 -B evidence/local-nonzero-ord-parent-review/source_audit.py --write
python3 -B evidence/local-nonzero-ord-parent-review/trials.py freeze
python3 -B evidence/local-nonzero-ord-parent-review/build_tests.py focused
python3 -B evidence/local-nonzero-ord-parent-review/build_tests.py units
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py exact
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py local
python3 -B evidence/local-nonzero-ord-parent-review/independent.py
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py bool
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py boundaries
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py exports
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py filters-v2
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py global-gates
python3 -B evidence/local-nonzero-ord-parent-review/matrix.py unsupported-v2
python3 -B evidence/local-nonzero-ord-parent-review/compatibility.py
python3 -B evidence/local-nonzero-ord-parent-review/packet.py
python3 -B evidence/local-nonzero-ord-parent-review/additional.py relation
python3 -B evidence/local-nonzero-ord-parent-review/additional.py old-policy
python3 -B evidence/local-nonzero-ord-parent-review/additional.py function-value
python3 -B evidence/local-nonzero-ord-parent-review/dual_valid.py
python3 -B evidence/local-nonzero-ord-parent-review/erasure_diagnostic.py
python3 -B evidence/local-nonzero-ord-parent-review/type_dependency_control.py
python3 -B evidence/local-nonzero-ord-parent-review/tests.py vir
python3 -B evidence/local-nonzero-ord-parent-review/tests.py units
python3 -B evidence/local-nonzero-ord-parent-review/tests.py focused
```

The redundant sentinel-only `additional.py dual-output` is not needed to reproduce
F1; `dual_valid.py` uses a genuine fresh same-runtime Bool interface and cleans up
the stale selected output after recording evidence. Audit RESULT generation uses
`audit.py --write` once after all requested runs. Default audits are read-only:

```sh
python3 -B evidence/local-nonzero-ord-parent-review/audit.py
python3 -O -B evidence/local-nonzero-ord-parent-review/audit.py
```

Own delivered private root: `/tmp/local-nonzero-ord-09t1tshb` (mode0700).
Each executed label has `freeze.json`, `receipt.json`, `stdout`, `stderr` and, for
ordinary verifier cases, private VIR/AIR. Exact cold/warm retained logs are
`bundle-exact-{cold,warm}/retained-vir/`; the common command writes
`bundle-same-invocation/vir/`. Diagnostic whole Function/query hashes are not
kernel proof objects. `bundle-final-tools.json` is the before-proof freeze;
source_inventory_sha256 is the canonical sorted-JSON source-map hash, distinct
from the pretty physical freeze-file hash. Review evidence is compact; raw
source/MIR/VIR/AIR/logs/binaries remain private.
