#!/usr/bin/env python3
"""Review-only adversarial replay. Writes fresh private outputs, never publishes."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
BRIDGE = ROOT / 'source/tools/research/native_semantic_bridge'
READY = Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', required=True, type=Path)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    freeze = json.loads((BRIDGE / 'parent-replay-freeze.json').read_text())
    freeze['candidate_directory'] = 'evidence/semantic-foundation-review'
    freeze['modules'] = ['assumption_probe']
    name = 'evidence/semantic-foundation-review/assumption_probe.v'
    freeze['files'][name] = sha(ROOT / name)
    freeze['purpose'] = 'ADVERSARIAL REVIEW ONLY: imported assumption, reduced inventory'
    frozen = a.output / 'freeze.json'
    frozen.write_text(json.dumps(freeze, indent=2) + '\n')
    result = subprocess.run([
        sys.executable, '-B', '-O', str(BRIDGE / 'check.py'),
        '--ready', str(READY), '--freeze', str(frozen),
        '--output', str(a.output / 'trial'),
    ], cwd=ROOT, capture_output=True, text=True)
    (a.output / 'stdout').write_text(result.stdout)
    (a.output / 'stderr').write_text(result.stderr)
    receipt = json.loads((a.output / 'trial/receipt.json').read_text())
    printed = (a.output / 'trial/compile-assumption_probe.stdout').read_text()
    kernel = (a.output / 'trial/kernel-check.stderr').read_text()
    if not (result.returncode == 0 and receipt['success'] is True
            and receipt['logical_axioms'] == [] and 'classic :' in printed
            and 'Coq.Logic.Classical_Prop.classic' in kernel):
        raise RuntimeError('Expected extra-assumption false acceptance did not reproduce')
    print(json.dumps(dict(reproduced=True, publication=receipt['publication'],
                          missed_axiom='Coq.Logic.Classical_Prop.classic',
                          receipt_sha256=sha(a.output / 'trial/receipt.json'))))


if __name__ == '__main__':
    main()
