#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
review_root="$(mktemp -d /tmp/practical-first-checkpoint-review-XXXXXX)"
base=source/tools/research/practical_caller_transitions
python3 -B -O "$base/fedimint/prepare.py" "$review_root/auth" > "$review_root/prepare.json"
python3 -B -O "$base/redb/replay.py" --bundle "$review_root/auth/bundle" --output "$review_root/redb" --phase manual
python3 -B -O "$base/parent_probes/run.py" --freeze "$base/parent_probes/freeze-scope5.json" --output "$review_root/p0"
for scenario in manual client semantic safety duplicate client-negative omit-array omit-set count failed-import; do
  extra=()
  if [[ "$scenario" == client || "$scenario" == client-negative || "$scenario" == failed-import ]]; then
    extra+=(--provider "$review_root/f3-manual")
  fi
  if [[ "$scenario" == failed-import ]]; then
    extra+=(--rejected "$review_root/f3-duplicate/duplicate-infer.vir")
  fi
  python3 -B -O "$base/fedimint/replay.py" --private "$review_root/auth" --output "$review_root/f3-$scenario" --scenario "$scenario" "${extra[@]}"
done
printf '%s\n' "$review_root"
