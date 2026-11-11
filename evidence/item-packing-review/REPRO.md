# Exact independent reproduction

Run from this checkout's repository root. All drivers are new review-owned code; only the pure `fixture()` reconstruction is imported from the old runner. No historical writing driver is executed. Commands use private sources/caches/targets/build-dir and completed immutable labels.

```
python3 -B evidence/item-packing-review/review.py setup
python3 -B evidence/item-packing-review/review.py build shipping-build
python3 -B evidence/item-packing-review/review.py vstd shipping-vstd
python3 -B evidence/item-packing-review/review.py stable full full.rs
python3 -B evidence/item-packing-review/review.py stable packet packet.rs
python3 -B evidence/item-packing-review/review.py stable packe packet.rs
python3 -B evidence/item-packing-review/audit.py
python3 -B evidence/item-packing-review/strict_bodies.py
python3 -B -O evidence/item-packing-review/audit.py
python3 -B -O evidence/item-packing-review/strict_bodies.py
python3 -B evidence/item-packing-review/authenticate.py
python3 -B evidence/item-packing-review/math_check.py
python3 -B evidence/item-packing-review/regressions.py
python3 -B evidence/item-packing-review/probes.py
python3 -B evidence/item-packing-review/variants.py ablation
python3 -B evidence/item-packing-review/variants.py false-constant
python3 -B evidence/item-packing-review/review.py build restored-build
python3 -B evidence/item-packing-review/review.py vstd restored-vstd
python3 -B - <<'PYCODE'
import sys
sys.path.insert(0,'evidence/item-packing-review')
import review as t
for name in ['full','packet']:
    j=t.trial('restored-'+name,t.root()/name/'component.rs')
    t.need(j['exit']==0 and j['verification']['errors']==0,'restored full scope')
PYCODE
```

After restored replay, run `python3 -B evidence/item-packing-review/aux_only.py` then `python3 -B evidence/item-packing-review/restored_warm.py` and `python3 -B evidence/item-packing-review/finish.py` to reproduce the additional auxiliary-only rejection and final receipt index. The returned original/masked values and actual flag are real outputs; no application assertion is weakened.

The recorded run used shipping-build5 following four excluded setup attempts, not a hidden successful first build. Details and exact argv/environment/source/tool/fixture hashes are in each private freeze.json/receipt.json indexed by RESULTS.json. The final runner includes the setup corrections. `setup` creates a new private snapshot repository solely because the normal version build scripts require Git metadata. Its source bytes match the review checkout; its private metadata commit is a label, not the parent commit. No parent/sibling worktree/build/cache/config is consulted.

The first packet path has one extra character relative to the completed worker fixture, causing only a pretty-printer line wrap in CHANNEL_MASK. The separate `packe` path has equal length in this recorded private root and matches every committed raw typed-body hash after replacing only the exact fixture filename. For a new randomized root of another length, choose a private relative fixture directory name that preserves the total filename length52, keep it outside the completed worker root, and update strict_bodies.py's local target label. Do not normalize coordinates, operators, types or whitespace. Both packet invocations retain identical source bytes and32/0 cold/warm; all30 actual AIR files match within each pair.

Cargo invocations are offline/locked/jobs2; CARGO_HOME, CARGO_TARGET_DIR and CARGO_BUILD_BUILD_DIR are distinct new private directories. Only221 lockfile-checksummed archives,337 recorded sparse-index files and20 recorded getopts Git input files are copied. The empty Git refs directory is recreated. Toolchain installation is read-only; compiler/macros are rebuilt from private matching sources. Standard vstd's ordinary driver has no --no-cheating. Application trials use --no-cheating --compile and full scope; inherited intentional incomplete/export-refusal regression inputs are diagnostics, not successful applications.

Stable trials use precisely identical argv, environment, source and tools for cold/warm. Each completed run directory is moved into cold/warm before the same run path is reused. Source variants modify only the private arithmetic source file, archive tools before/after, build matching standard vstd, and restore the original bytes in finally. Never rerun a completed immutable label. No sourcecontrol or helper result from a failed/setup-invalid trial is accepted.
