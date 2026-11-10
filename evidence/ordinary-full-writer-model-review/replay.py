#!/usr/bin/env python3
"""Exact, private MODEL replay and independent worker/inventory comparison."""
import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
TARGET = ROOT / 'source/tools/research/model_ordinary_full_writer'
WORKER = Path('/tmp/model-ordinary-full-writer-trial-012')
WORKER_SHA = '5733f94896797c6b3b7b432500ae0839711843e979e8c05f683516d349909ed8'
HISTORICAL_ROOT = '/home/dev/.t3/worktrees/verus/t3-2e73bda1/'
MODULES = ['interface', 'vectors', 'writer', 'controls']
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import replay_receipt, require
sys.path.insert(0, str(ROOT / 'source/tools/research/native_semantic_bridge'))
from proof_checks import parse_axioms, parse_kernel_summary


def data_hash(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def relabel(label):
    # No filesystem operation on a receipt's historical checkout path.
    if label.startswith(HISTORICAL_ROOT):
        return ROOT / label[len(HISTORICAL_ROOT):]
    require(label.startswith('/tmp/'), 'unexpected historical input label: ' + label)
    return Path(label)


def stable_label(path):
    return 'repo:' + str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)


def git_blob(commit, path):
    return subprocess.check_output(['git', 'show', commit + ':' + str(path.relative_to(ROOT))], cwd=ROOT)


def authenticated_inputs():
    receipt_path = WORKER / 'receipt.json'
    require(digest(receipt_path) == WORKER_SHA, 'worker receipt authentication')
    worker = json.loads(receipt_path.read_bytes())
    require(worker['success'] and worker['model_proofs_kernel_checked'], 'worker not successful')
    paths = [receipt_path, HERE / 'SCOPE.md', HERE / 'replay.py', TARGET / 'REPORT.md', TARGET / 'results.json']
    expected = {}
    for label, record in worker['inputs'].items():
        path = relabel(label)
        paths.append(path)
        expected[path] = record['sha256']
    for label, record in worker['proof_artifacts'].items():
        path = Path(label)
        require(path.parent == WORKER / 'proofs', 'unexpected worker product')
        paths.append(path)
        expected[path] = record['sha256']
    results = json.loads((TARGET / 'results.json').read_text())
    for name, sha in results['logs'].items():
        path = WORKER / name
        paths.append(path)
        expected[path] = sha
    for path, sha in expected.items():
        require(digest(path) == sha, 'authenticated input differs: ' + str(path))
    # Exact completed commit and early typed freeze, using this checkout's history.
    for path in [TARGET / 'REPORT.md', TARGET / 'results.json', TARGET / 'final-freeze.json',
                 TARGET / 'check.py', *[TARGET / 'candidates' / (m + '.v') for m in MODULES]]:
        require(path.read_bytes() == git_blob('d7156dec', path), 'completed commit identity: ' + str(path))
    path = TARGET / 'candidates/interface.v'
    require(path.read_bytes() == git_blob('d50fe08e', path), 'early typed freeze identity')
    return worker, sorted(set(paths))


def inventory(proofs, assumptions_text, terms_text, kernel_text):
    # Independently parse declarations in all six sources (after nested comment
    # removal); compare exact names and kinds with the kernel, not just counts.
    records = {}
    for module in MODULES + ['Assumptions', 'ProofTerms']:
        source = (proofs / (module + '.v')).read_text()
        while '(*' in source:
            stripped = re.sub(r'\(\*(?:(?!\(\*|\*\)).)*\*\)', '', source, flags=re.S)
            require(stripped != source, 'unbalanced comment')
            source = stripped
        require(not re.search(r'\b(?:Axiom|Parameter|Admitted|admit|Abort|Program|Obligation|Inductive|CoInductive|Record|Class|Equations|Scheme|Derive|Include|Load)\b', source), 'unexpected generated/unchecked declaration')
        decls = re.findall(r'^\s*(Definition|Lemma|Theorem)\s+(\w+)\b', source, re.M)
        for kind, name in decls:
            qualified = module + '.' + name
            require(qualified not in records, 'duplicate source declaration')
            records[qualified] = dict(kind=kind, assumptions=None, constructors=[], obligations=[], existing_instances=[])
        registrations = re.findall(r'^\s*Existing\s+Instance\s+(\w+)\s*\.', source, re.M)
        require(registrations == (['Outer'] if module == 'writer' else []), 'existing-instance inventory')
    for module, text in [('Assumptions', assumptions_text), ('ProofTerms', terms_text)]:
        sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', text, re.M | re.S)
        names = [n for n in records if n.split('.')[0] in MODULES]
        require([a for a, _, _ in sections] == names == [b for _, _, b in sections], 'individual print coverage')
        if module == 'Assumptions':
            for name, body, _ in sections:
                records[name]['assumptions'] = parse_axioms(body)
        tail = text.split('END ' + names[-1], 1)[1].strip().splitlines()
        helpers = [n for n in records if n.startswith(module + '.')]
        require(tail == ['Closed under the global context'] * len(helpers), 'individual helper assumptions')
        helper_source = (proofs / (module + '.v')).read_text()
        actual_prints = re.findall(r'^Print Assumptions ModelOrdinaryFullWriter\.(' + module + r'\.\w+)\.$', helper_source, re.M)
        require(actual_prints == helpers, 'helper Print Assumptions order/names')
        for name in helpers:
            records[name]['assumptions'] = []
    checked = re.findall(r'checking cst:ModelOrdinaryFullWriter\.([^\s]+)', kernel_text)
    require(len(checked) == len(set(checked)) and set(checked) == set(records), 'exhaustive kernel/source set mismatch')
    require(not re.search(r'checking (?:ind|mind):ModelOrdinaryFullWriter\.', kernel_text), 'unexpected local inductive/constructor')
    require(len(records) == 125 and sum(n.split('.')[0] in MODULES for n in records) == 25, 'unexpected scope')
    require(all(r['assumptions'] is not None for r in records.values()), 'missing individual assumptions')
    return records


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--create-freeze', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    worker, paths = authenticated_inputs()
    freeze_path = HERE / 'review-freeze.json'
    frozen = {stable_label(p): digest(p) for p in paths}
    if args.create_freeze:
        require(not freeze_path.exists(), 'review freeze exists')
        freeze_path.write_text(json.dumps(dict(before_fresh_trial=True, files=frozen), indent=2) + '\n')
        print('Review inputs frozen:', len(frozen), data_hash(frozen))
        return
    require(args.output is not None, 'private unused output required')
    out = args.output.resolve()
    require(not out.is_relative_to(ROOT), 'private output required')
    out.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, publication=False, worker_receipt_sha256=WORKER_SHA)
    with replay_receipt(out, receipt):
        guard = ArtifactGuard([*paths, freeze_path])
        freeze = json.loads(freeze_path.read_text())
        require(freeze['before_fresh_trial'] is True and frozen == freeze['files'], 'review freeze mismatch')
        receipt['inputs'] = guard.initial
        trial = out / 'trial'
        command = ['python3', '-B', str(TARGET / 'check.py'), '--freeze', str(TARGET / 'final-freeze.json'), '--output', str(trial)]
        with (out / 'replay.stdout').open('w') as stdout, (out / 'replay.stderr').open('w') as stderr:
            code = subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr).returncode
        receipt['command'] = dict(argv=command, exit_code=code)
        require(code == 0, 'fresh normal replay failed; inspect private logs')
        fresh = json.loads((trial / 'receipt.json').read_text())
        require(fresh['success'] and fresh['model_proofs_kernel_checked'], 'fresh receipt unsuccessful')
        # Keep every fresh replay input and output immutable through outer success.
        products = ArtifactGuard([trial / 'receipt.json', *[Path(p) for p in fresh['inputs']],
            *[Path(p) for p in fresh['proof_artifacts']],
            *[trial / (c['label'] + '.' + stream) for c in fresh['commands'] for stream in ['stdout', 'stderr']]])
        require(all(products.initial[p] == value for p, value in fresh['inputs'].items()), 'fresh inputs changed after child success')
        require(all(products.initial[p] == value for p, value in fresh['proof_artifacts'].items()), 'fresh products changed after child success')
        for key in ['assumptions', 'kernel_axioms', 'declaration_inventory', 'constructor_inventory',
                    'program_obligations', 'existing_instance_registrations', 'proof_terms',
                    'kernel_declarations', 'inventory_helper_theorems', 'all_kernel_declarations', 'helper_assumptions']:
            require(fresh[key] == worker[key], 'worker/fresh evidence differs: ' + key)
        comparison = {}
        for name in MODULES + ['Assumptions', 'ProofTerms']:
            comparison[name] = {}
            for suffix in ['.v', '.vo']:
                file = name + suffix
                sha = digest(trial / 'proofs' / file)
                require(sha == digest(WORKER / 'proofs' / file), 'worker/fresh artifact differs: ' + file)
                comparison[name][suffix[1:]] = sha
        records = inventory(trial / 'proofs', (trial / 'assumptions.stdout').read_text(),
                            (trial / 'proof-terms.stdout').read_text(), (trial / 'kernel.stderr').read_text())
        require(parse_kernel_summary((trial / 'kernel.stderr').read_text()) == fresh['kernel_axioms'], 'kernel summary')
        require({n:r['assumptions'] for n,r in records.items() if n.split('.')[0] in MODULES} == fresh['assumptions'], 'independent assumptions agreement')
        # Require exact normal argv including all loadpaths, only relocating our trial.
        for original, current in zip(worker['commands'], fresh['commands'], strict=True):
            require(original['label'] == current['label'] and current['exit_code'] == 0, 'command coverage')
            require([s.replace(str(WORKER), str(trial)) for s in original['argv']] == current['argv'], 'normal command/loadpath identity')
        receipt.update(fresh_receipt_sha256=digest(trial / 'receipt.json'), pairs=comparison,
            individual_inventory=records, existing_instance_registrations=fresh['existing_instance_registrations'],
            constructor_inventory=[], program_obligations=[], kernel_axioms=fresh['kernel_axioms'],
            assumptions_sha256=data_hash(fresh['assumptions']), helper_assumptions_sha256=data_hash(fresh['helper_assumptions']),
            inventory_sha256=data_hash(records), commands=[dict(label=c['label'], exit_code=c['exit_code'], seconds=c['seconds']) for c in fresh['commands']],
            logs={p.name:digest(p) for p in trial.iterdir() if p.suffix in ['.stdout','.stderr']},
            products=products.initial, all_six_pairs_identical=True, normal_recursive_kernel_no_exclusions=True)
        products.check()
        guard.check()
        receipt['all_guards_retained_through_success'] = True
    print(json.dumps(dict(success=True, receipt=str(out / 'receipt.json'), sha256=digest(out / 'receipt.json'))))


if __name__ == '__main__':
    main()
