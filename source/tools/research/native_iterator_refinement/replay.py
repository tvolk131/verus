#!/usr/bin/env python3
"""Ordinary full-checking probes, separate frontend classification, guarded receipts."""
import argparse
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
    require_exact, require_function_coverage, validate_goal_run, validate_run)

p = argparse.ArgumentParser()
p.add_argument('--verus', type=Path, required=True)
p.add_argument('--solver', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--case', action='append', required=True)
p.add_argument('--goals', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
receipt = dict(schema=1, commands=[], cases=[])
with replay_receipt(a.output, receipt):
    source_map = json.loads((HERE / 'source-map.json').read_text())
    native_root = Path(source_map['rust_source_root'])
    for name, checksum in source_map['repository_files'].items():
        require(digest(ROOT / name) == checksum, f'repository input changed: {name}')
    for name, checksum in source_map['native_files'].items():
        require(digest(native_root / name) == checksum, f'pinned Rust source changed: {name}')
    additional = json.loads((HERE / 'additional-source-evidence.json').read_text())
    for name, checksum in additional.items():
        require(digest(ROOT / name) == checksum, f'additional source changed: {name}')
    bundle = json.loads((HERE / 'bundle-identity.json').read_text())
    for name, data in bundle.items():
        path = a.solver if name == 'z3' else a.verus.parent / name
        require(digest(path) == data['sha256'], f'accepted bundle mismatch: {name}')
    cases = []
    inputs = [Path(__file__), HERE / 'freeze.py', HERE / 'source-map.json', HERE / 'bundle-identity.json',
        HERE / 'additional-source-evidence.json', *[ROOT / name for name in additional],
        *[ROOT / name for name in source_map['repository_files']],
        *[native_root / name for name in source_map['native_files']]]
    for name in a.case:
        require(name.replace('-', '').isalnum(), 'invalid case identity')
        case = HERE / 'candidates' / name
        frozen = json.loads((case / 'freeze.json').read_text())
        require(frozen['id'] == name and frozen['before_first_trial'] is True, 'case freeze')
        require(frozen['target_source_map_sha256'] == digest(HERE / 'source-map.json'), 'target freeze')
        require(digest(case / 'probe.rs') == frozen['source_sha256'], 'case source hash')
        require(frozen['expected_helpers'] == [], 'explicit zero helper identity set required')
        require(frozen['required_helper_checks'] == ['soundness', 'maximality', 'exactness'], 'helper gates')
        # Bind the copied semantic predicate to the original frozen full property.
        code = (case / 'probe.rs').read_text()
        if 'pub open spec fn pad_xor(' in code:
            start = 'pub open spec fn pad_xor('
            predicate = lambda s: s[s.index(start):s.index('\n}', s.index(start)) + 2]
            require(predicate(code) == predicate((ROOT / 'source/tools/research/buffer_transfer_goal/baseline.rs').read_text()),
                    'full target predicate changed')
        cases.append((name, case, frozen))
        inputs += [case / 'probe.rs', case / 'freeze.json']
    guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(), inputs))
    receipt['artifacts'] = guard.initial
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
        RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.solver.resolve()),
        VERUS_INFER_CACHE=str(a.output.resolve() / 'cache'))
    run = LoggedCommands(a.output, ROOT, receipt['commands'], timeout=180,
                         guard=guard, status_key='returncode')
    require(run('verus-version', [str(a.verus.resolve()), '--version'], env) == 0, 'runtime identity')
    version = (a.output / 'verus-version.stdout').read_text()
    require('0c00b6e' in version and '1.99.1' in version, 'unexpected accepted runtime')
    require(run('z3-version', [str(a.solver.resolve()), '--version'], env) == 0, 'solver identity')
    solver_version = (a.output / 'z3-version.stdout').read_text()
    require('4.16.0' in solver_version, 'unexpected accepted solver')
    receipt['runtime'] = dict(verus=version, solver=solver_version)
    for name, case, frozen in cases:
        command = [str(a.verus.resolve()), str(case / 'probe.rs'), '--crate-type=lib',
                   '--no-cheating', '--output-json', '--multiple-errors', '8']
        if a.goals:
            command.append('--infer-goals')
        rc = run(name, command, env)
        result = dict(returncode=rc, stdout=(a.output / (name + '.stdout')).read_text(),
                      stderr=(a.output / (name + '.stderr')).read_text())
        entry = dict(id=name, freeze=frozen, returncode=rc,
            stdout_sha256=digest(a.output / (name + '.stdout')),
            stderr_sha256=digest(a.output / (name + '.stderr')))
        receipt['cases'].append(entry)
        if frozen['classification'] == 'frontend-rejection':
            data = validate_run(result, False, allow_frontend=True)
            require(frozen['expected_frontend_message'] in result['stderr'], 'wrong frontend rejection')
            require(data['verification-results'].get('encountered-vir-error') is True
                    or data['verification-results'].get('encountered-error') is True, 'frontend class')
            require(data['verification-results'].get('errors', 0) == 0, 'frontend is not SMT failure')
            helpers = data.get('verus-infer', {}).get('functions', [])
            require_function_coverage(helpers, set())
            require_exact(helpers)
        else:
            success = frozen['classification'] == 'checked-abstract-countermodel'
            data = validate_goal_run(result, success, expected_functions=set(),
                exec_bodies=frozen['exec_bodies'], proof_bodies=frozen['proof_bodies'],
                failure_kind=frozen['expected_failure_kind'])
        entry['verification'] = data['verification-results']
        entry['verification_scope'] = data.get('verus-infer', {}).get('verification_scope')
        guard.check()
        print(json.dumps(dict(id=name, result=entry['verification'])))
