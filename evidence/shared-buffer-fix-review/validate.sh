#!/bin/bash
set -euo pipefail
ROOT=$(git rev-parse --show-toplevel)
RAW=${REVIEW_RAW:-/tmp/shared-buffer-fix-review-c9d072ad}
export PATH=/home/dev/.cargo/bin:$PATH CARGO_BUILD_JOBS=2 VERUS_Z3_PATH=$RAW/z3 VERUS_IN_VARGO=1 RUST_MIN_STACK=33554432
export VERUS_INFER_CACHE="$RAW/compiler-cache" VERUS_TEST_LOG_DIR="$RAW/compiler-inputs"
cd "$ROOT/source"
if [ "$#" -eq 0 ]; then
 cargo test --release --locked -p vir --lib > "$RAW/vir.log" 2>&1
 review_targets=(infer_buffer_observation_goals infer_mutable_buffer_goals infer)
else
 review_targets=("$@")
fi
for target in "${review_targets[@]}"; do
 cargo test --release --locked -p rust_verify_test --test "$target" -- --test-threads=2 2>&1 | tee "$RAW/$target.log"
done
