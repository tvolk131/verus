# Replay

From the repository root, without loading another checkout:

```
python3 -B source/tools/research/item_packing_goal/run.py setup
python3 -B source/tools/research/item_packing_goal/run.py baseline
python3 -B source/tools/research/item_packing_goal/run.py build replay-build
python3 -B source/tools/research/item_packing_goal/run.py vstd replay-vstd
python3 -B source/tools/research/item_packing_goal/check.py stable stable-full full
python3 -B source/tools/research/item_packing_goal/check.py stable stable-reuse reuse
python3 -B source/tools/research/item_packing_goal/check.py audit stable-full stable-reuse
python3 -B -O source/tools/research/item_packing_goal/check.py audit stable-full stable-reuse
python3 -B source/tools/research/item_packing_goal/controls.py
```

Use a fresh private root from setup; completed labels cannot be reused. setup
rebinds the pure fixture from the explicitly permitted immutable completed
review, authenticates the parent bundle receipt/all8 tools and locked archive
inputs, and permits only the owned arithmetic/new-test production differences
from accepted Git35bc7b12. It never executes a historical writer/controller or
follows historical checkout/upstream paths. Cargo build-dir, target-dir and
CARGO_HOME all point into the new root. Index/getopts objects come only from the
named completed review inputs. Compiler/core/toolchain identity is recorded in
the delivered private evidence. No network/install/config/host changes are needed.

Every application command uses --compile --no-cheating, full ordinary scope and
--log-all into a new private directory; vstd uses its normal standard driver.
Raw source/AIR/MIR/binaries/logs remain private. Commands/env/source/tool/fixture
hashes and completed outputs are retained in each freeze.json/receipt.json.
check.py stable archives cold output before identical working-path reuse.

For compiler regressions use the private Cargo invocation/environment in
RESULTS.json receipts. The inherited harness locates common tools under
`<root>/target-verus/debug` but InferCrates tests look beside the test executable.
Copy the authenticated own bundle into both private locations before executing
the focused binaries. The inherited InferCrates solver path also needs a
transient own-checkout `source/z3` link to the private solver; remove it after
those tests. With Cargo's explicit build-dir, binary paths may be under
cargo-build/debug/deps rather than target/debug/deps. No harness source change
or weakened assertion is needed. Use fresh test profile/log paths on retry.

Source-control transformations are exactly preregistered in SOURCE_CONTROLS.md;
retained shipping/ablation/false-constant snapshots and their hashes are private.
variants.py builds/checks each variant, archives its tools, then restores the
shipping source. The final restored shipping compiler and vstd are rebuilt and
applications replayed before delivery. These source controls are ordinary proof
regressions. All production source outside checked_foreign_packing.rs is
unchanged; only the new tests/research directory are added.
