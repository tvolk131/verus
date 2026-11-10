#!/bin/bash
# Continue after preserving the original probe-hypothesis failure and adjudication.
set -euo pipefail
ROOT=$(git rev-parse --show-toplevel)
RAW=/tmp/stable-key-review-e84eed6c
cd "$ROOT"
export PATH=/home/dev/.cargo/bin:$PATH CARGO_BUILD_JOBS=2 RUSTUP_TOOLCHAIN=1.99.1 VERUS_Z3_PATH="$ROOT/source/z3"
python3 -B -O source/tools/research/varint_canonical_goal/replay.py --bundle "$RAW/bundle" --output "$RAW/canonical" > "$RAW/canonical.log" 2>&1
CANONICAL=$(find "$RAW/canonical" -name receipt.json)
python3 -B -O source/tools/research/varint_canonical_goal/audit.py "$CANONICAL" --output "$RAW/canonical-audit.json" > "$RAW/canonical-audit.log" 2>&1
python3 -B -O source/tools/research/buffer_observation_goal/replay.py --verus "$RAW/bundle/verus" --solver "$RAW/bundle/z3" --output "$RAW/observations" > "$RAW/observations.log" 2>&1
python3 -B -O evidence/stable-key-review/audit.py > "$RAW/independent-audit.log" 2>&1
