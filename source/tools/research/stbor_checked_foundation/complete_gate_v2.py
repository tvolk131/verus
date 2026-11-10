#!/usr/bin/env python3
"""Check the additive inventory-support epoch, then the frozen main gate.

The main completed receipt binds kernel-coverage.json, which binds this epoch's
manifest, which in turn binds both additional controller sources. The original
build controllers and initial tool/source freeze stay unchanged.
"""
from pathlib import Path
import json
import subprocess

from evidence import ROOT, OWN, guard, sha

path = ROOT/'manifests/supplemental-controller-freeze.json'
supplemental = json.loads(path.read_text())
guard(Path('/'), supplemental['files'])
coverage = json.loads((ROOT/'manifests/kernel-coverage.json').read_text())
if coverage['supplemental_controller_freeze_sha256'] != sha(path):
    raise SystemExit('inventory-support epoch guard failed')
controller = coverage['inventory_controller']
if sha(controller['path']) != controller['sha256']:
    raise SystemExit('loaded inventory controller guard failed')
inventory = json.loads((ROOT/'manifests/declaration-inventory.json').read_text())
for name, entry in inventory.items():
    if entry.get('print_full_declaration'):
        out = ROOT/f'declaration-output/{entry["index"]}-declaration.out'
        if sha(out) != entry['declaration_sha256']:
            raise SystemExit('inductive/constructor/projection inventory guard failed: '+name)
graph = json.loads((ROOT/'manifests/dependency-graph-complete.json').read_text())
stage = ROOT/'manifests/dependency-controller-freeze.json'
if graph['controller_freeze_sha256'] != sha(stage) or not graph['all_diagnostics_empty']:
    raise SystemExit('complete dependency map guard failed')
guard(Path('/'), json.loads(stage.read_text())['files'])
subprocess.run(['/usr/bin/python3', '-B', str(OWN/'ready_gate_complete.py')], check=True)
guard(Path('/'), supplemental['files'])
