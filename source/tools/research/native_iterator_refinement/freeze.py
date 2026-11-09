#!/usr/bin/env python3
"""Create an immutable pretrial case. No compiler execution occurs here."""
import argparse
import json
from pathlib import Path
import shutil
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import digest
from infer_experiment import require

p = argparse.ArgumentParser()
p.add_argument('--id', required=True)
p.add_argument('--source', type=Path, required=True)
p.add_argument('--classification', required=True, choices=[
    'checked-abstract-countermodel', 'semantic-negative', 'frontend-rejection'])
p.add_argument('--exec', dest='executables', nargs='*', default=[])
p.add_argument('--proof', dest='proofs', nargs='*', default=[])
p.add_argument('--failure-kind')
p.add_argument('--frontend-message')
a = p.parse_args()
require(a.id.replace('-', '').isalnum(), 'invalid case identity')
require(bool(a.executables + a.proofs), 'named body inventory required')
require(bool(a.frontend_message) == (a.classification == 'frontend-rejection'), 'frontend expectation')
require(bool(a.failure_kind) == (a.classification == 'semantic-negative'), 'semantic expectation')
case = HERE / 'candidates' / a.id
case.mkdir(parents=True, exist_ok=False)
shutil.copy2(a.source, case / 'probe.rs')
manifest = dict(schema=1, id=a.id, before_first_trial=True,
    source_sha256=digest(case / 'probe.rs'), classification=a.classification,
    exec_bodies=a.executables, proof_bodies=a.proofs,
    expected_helpers=[], required_helper_checks=['soundness', 'maximality', 'exactness'],
    expected_failure_kind=a.failure_kind, expected_frontend_message=a.frontend_message,
    full_checking=['--no-cheating', 'entire crate', 'lifetime', 'trait', 'body'],
    scope='Abstract observation equations or rejected bridge mechanisms; never the frozen native theorem',
    target_source_map_sha256=digest(HERE / 'source-map.json'))
(case / 'freeze.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(case)
