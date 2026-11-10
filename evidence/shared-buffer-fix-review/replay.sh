#!/bin/bash
set -eu
ROOT=$(git rev-parse --show-toplevel)
RAW=${REVIEW_RAW:-/tmp/shared-buffer-fix-review-c9d072ad}
cd "$ROOT"
export PATH=/home/dev/.cargo/bin:$PATH CARGO_BUILD_JOBS=2 VERUS_Z3_PATH=$RAW/z3 RUSTUP_TOOLCHAIN=1.99.1
python3 -B -O source/tools/research/varint_canonical_goal/copy_bundle.py --from "$ROOT/source/target-verus/release" --solver "$RAW/z3" --output "$RAW/bundle" > "$RAW/bundle.log" 2>&1
"$RAW/bundle/verus" --version > "$RAW/verus-version.txt" 2>&1
python3 -B -O evidence/shared-buffer-fix-review/probe.py --raw "$RAW" > "$RAW/probes.log" 2>&1
python3 -B -O source/tools/research/varint_canonical_goal/replay.py --bundle "$RAW/bundle" --output "$RAW/canonical" > "$RAW/canonical.log" 2>&1
CANONICAL=$(find "$RAW/canonical" -name receipt.json)
python3 -B -O source/tools/research/varint_canonical_goal/audit.py "$CANONICAL" --output "$RAW/canonical-audit.json" > "$RAW/canonical-audit.log" 2>&1
python3 -B -O source/tools/research/buffer_observation_goal/replay.py --verus "$RAW/bundle/verus" --solver "$RAW/bundle/z3" --output "$RAW/observations" > "$RAW/observations.log" 2>&1
python3 -B -O evidence/shared-buffer-fix-review/audit.py --raw "$RAW" > "$RAW/independent-audit.log" 2>&1
