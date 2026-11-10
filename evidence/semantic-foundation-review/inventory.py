#!/usr/bin/env python3
"""Independent complete Print Assumptions coverage for the frozen 18+6 inventory."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shlex
import subprocess

ROOT = Path(__file__).resolve().parents[2]
BRIDGE = ROOT / 'source/tools/research/native_semantic_bridge'
EXPECTED = {
    'borrow_handoff': ['source_loan_access', 'source_loan_release', 'source_context_access'],
    'byte_array': ['byte_list_length', 'bytes_size', 'bytes_physical', 'bytes_initialized',
                   'byte_array128_size', 'byte_array128_release',
                   'byte_array128_scoped_handoff', 'byte_array128_context_handoff'],
    'raw_view': ['typed_source_raw_view', 'typed_source_raw_offset', 'typed_source_loan_release'],
    'model_limits': ['arbitrary_raw_pointer_typed', 'raw_read_empty_head_stuck',
                     'model_offset_unrestricted', 'empty_heap_offset_adequate'],
}


def need(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--proofs', type=Path, required=True, help='fresh check.py output directory')
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    ready = Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json')
    r = json.loads(ready.read_text())
    baseline = Path(r['build'])
    args = []
    for line in (baseline / '_CoqProject').read_text().splitlines():
        words = shlex.split(line, comments=True)
        if words and words[0] in ('-Q', '-R'):
            args += [words[0], str(baseline / words[1]), words[2]]
    args += ['-Q', str(a.proofs.resolve() / 'proofs'), 'NativeSemanticBridge']
    names = []
    identities = {}
    for module, expected in EXPECTED.items():
        source = BRIDGE / 'candidates/loan-v12' / (module + '.v')
        actual = re.findall(r'^\s*(?:Lemma|Theorem)\s+(\w+)', source.read_text(), re.M)
        need(actual == expected, 'named inventory mismatch ' + module)
        compiled = a.proofs / 'proofs' / (module + '.v')
        need(sha(source) == sha(compiled), 'compiled source mismatch ' + module)
        identities[module] = dict(source_sha256=sha(source),
                                  object_sha256=sha(compiled.with_suffix('.vo')))
        names += [module + '.' + name for name in expected]
    obligations = ['byte_array.byte_ty_obligation_' + str(i) for i in range(1, 7)]
    names += obligations + ['byte_array.byte_ty']
    source = 'From NativeSemanticBridge Require Import ' + ' '.join(EXPECTED) + '.\n'
    for name in names:
        source += f'Goal True. idtac "BEGIN {name}". exact I. Qed.\n'
        source += f'Print Assumptions NativeSemanticBridge.{name}.\n'
    source += 'Fail Check NativeSemanticBridge.byte_array.byte_ty_obligation_7.\n'
    trial = a.output / 'Inventory.v'
    trial.write_text(source)
    command = [str(ready.parent / 'run-rocq'), str(Path(r['switch']) / '_opam/bin/coqc'),
               '-q', *args, str(trial)]
    run = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    (a.output / 'stdout').write_text(run.stdout)
    (a.output / 'stderr').write_text(run.stderr)
    need(run.returncode == 0, 'independent inventory compilation failed')
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)(?=^BEGIN |\Z)', run.stdout, re.M | re.S)
    need([n for n, _ in sections] == names, 'incomplete assumption output coverage')
    assumptions = {}
    for name, text in sections:
        if 'Closed under the global context' in text:
            axioms = []
            need('Axioms:' not in text, 'ambiguous assumption output')
        else:
            need('Axioms:' in text, 'unclassified output ' + name)
            axioms = re.findall(r'^([\w.]+)\s*:', text, re.M)
            axioms = [x for x in axioms if x != 'Axioms']
            need(axioms in (['functional_extensionality_dep'],
                            ['FunctionalExtensionality.functional_extensionality_dep'],
                            ['Coq.Logic.FunctionalExtensionality.functional_extensionality_dep']),
                 'unexpected assumption ' + name + ': ' + str(axioms))
        assumptions[name] = axioms
    need(all(not assumptions[m + '.' + n] for m in ('borrow_handoff', 'raw_view', 'model_limits')
             for n in EXPECTED[m]), 'generic/raw/control theorem not closed')
    result = dict(named_theorems=EXPECTED, named_count=18, obligations=obligations,
                  obligation_count=6, assumptions=assumptions, identities=identities,
                  command=command, stdout_sha256=sha(a.output / 'stdout'),
                  stderr_sha256=sha(a.output / 'stderr'), source_sha256=sha(trial), success=True)
    (a.output / 'inventory.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
