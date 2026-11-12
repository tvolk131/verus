#!/usr/bin/env bash
set -euo pipefail
# Run from repository root; argument must name a fresh private output directory.
exec python3 -B -O source/tools/research/practical_caller_transitions/redb/replay_v2.py --output "${1:?fresh output directory required}" --full --controls
