#!/usr/bin/env python3
"""Freeze development candidates before executing them; do not overwrite a freeze."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1]))
from infer_eval_support import digest
from infer_experiment import require

def freeze(destination):
    require(not destination.exists(), 'freeze already exists')
    files = sorted(p for p in HERE.rglob('*') if p.is_file()
                   and p.suffix in ('.rs', '.py', '.toml', '.lock'))
    data = dict(schema=1, native_bridge_proved=False,
                files={str(p.relative_to(HERE)): digest(p) for p in files},
                target_sha256=digest(HERE.parent / 'mutable_buffer_goal/transfer-freeze.json'),
                source_map_sha256=digest(HERE.parent / 'native_iterator_refinement/source-map.json'))
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(data, indent=2) + '\n')
    return data

if __name__ == '__main__':
    freeze(Path(sys.argv[1]))
