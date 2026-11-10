#!/usr/bin/env python3
"""Adversarial source/readiness eligibility probes; never semantic negatives."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
BRIDGE = ROOT / 'source/tools/research/native_semantic_bridge'
sys.path.insert(0, str(BRIDGE))
from source_map import build


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', required=True, type=Path)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    extraction = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')
    native = json.loads((extraction / 'drop-native.json').read_text())
    borrowck = json.loads((extraction / 'drop-borrowck.json').read_text())
    source = ROOT / 'source/tools/research/native_heap_lowering/fixtures/drop.rs'
    cases = []
    for name in ('wrong-borrowck-unsize-operand', 'wrong-borrowck-call-operand'):
        mutated = copy.deepcopy(borrowck)
        if name == 'wrong-borrowck-unsize-operand':
            mutated['blocks'][0]['statements'][4]['ir']['value']['operand']['place']['local'] = 1
        else:
            mutated['blocks'][0]['terminator']['arguments'][0]['place']['local'] = 1
        (a.output / (name + '.json')).write_text(json.dumps(mutated, indent=2) + '\n')
        result = build(copy.deepcopy(native), mutated, source)
        if not (result['operation_count'] == 44 and result['publication'] is False
                and result['native_effects_discharged'] == 0):
            raise RuntimeError('Unexpected source-probe outcome')
        cases.append(dict(name=name, eligibility_accepted=True, publication=False,
                          native_effects_discharged=0,
                          classification='source_identity_audit_gap_not_semantic_acceptance'))
    # A forged success record must never count as baseline provenance. The actual
    # tools/baseline stay read only; modules=[] stops before any proof compilation.
    ready_path = Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json')
    ready = json.loads(ready_path.read_text())
    fake = a.output / 'synthetic-baseline-receipt.json'
    fake.write_text(json.dumps(dict(success=True, copied_sources={}, commands=[
        dict(label=label, returncode=0) for label in
        ('prover-version', 'build-baseline', 'kernel-check-baseline')])) + '\n')
    ready['receipt'] = str(fake)
    (a.output / 'ready.json').write_text(json.dumps(ready, indent=2) + '\n')
    (a.output / 'run-rocq').symlink_to(ready_path.parent / 'run-rocq')
    (a.output / 'gmp-private').symlink_to(ready_path.parent / 'gmp-private', target_is_directory=True)
    freeze = json.loads((BRIDGE / 'parent-replay-freeze.json').read_text())
    freeze['modules'] = []
    (a.output / 'freeze.json').write_text(json.dumps(freeze, indent=2) + '\n')
    run = subprocess.run([sys.executable, '-B', '-O', str(BRIDGE / 'check.py'),
                          '--ready', str(a.output / 'ready.json'),
                          '--freeze', str(a.output / 'freeze.json'),
                          '--output', str(a.output / 'readiness-trial')],
                         cwd=ROOT, capture_output=True, text=True)
    (a.output / 'stdout').write_text(run.stdout)
    (a.output / 'stderr').write_text(run.stderr)
    receipt_path = a.output / 'readiness-trial/receipt.json'
    receipt = json.loads(receipt_path.read_text())
    if not (run.returncode != 0 and receipt['failure']['message'] == 'module inventory'
            and [c['label'] for c in receipt['commands']] == ['version']
            and receipt['commands'][0]['exit_code'] == 0
            and receipt['model_proofs_kernel_checked'] is False):
        raise RuntimeError('Unexpected readiness-probe outcome')
    cases.append(dict(name='synthetic-success-empty-source-manifest', readiness_accepted=True,
                      stopped_after_prover_version=True, proof_compilation_invoked=False,
                      publication=False, success=False, receipt_sha256=sha(receipt_path),
                      classification='baseline_provenance_gate_gap_not_semantic_acceptance'))
    (a.output / 'probes.json').write_text(json.dumps(dict(cases=cases, semantic_negatives=0), indent=2) + '\n')
    print(json.dumps(cases, indent=2))


if __name__ == '__main__':
    main()
