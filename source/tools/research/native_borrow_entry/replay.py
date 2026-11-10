#!/usr/bin/env python3
"""Guarded ordinary entry-law/probe replay; no fact publication."""
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
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
receipt = dict(schema=1, commands=[], cases=[], native_bridge_proved=False, publication=False)
with replay_receipt(a.output, receipt):
    bundle = json.loads((HERE / 'bundle-identity.json').read_text())
    for name, item in bundle.items():
        path = a.solver if name == 'z3' else a.verus.parent / name
        require(digest(path) == item['sha256'], 'bundle mismatch: ' + name)
    cases = []
    accepted = json.loads((HERE / 'accepted.json').read_text())['cases']
    inputs = [Path(__file__), HERE / 'bundle-identity.json', HERE / 'accepted.json']
    for name in a.case:
        require(name.replace('-', '').isalnum(), 'invalid case name')
        case = HERE / 'candidates' / name
        frozen = json.loads((case / 'freeze.json').read_text())
        require(frozen['id'] == name and frozen['before_first_trial'] is True, 'missing freeze')
        require(digest(case / 'probe.rs') == frozen['source_sha256'], 'frozen source changed')
        expectation = accepted.get(name, frozen)
        require(expectation['classification'] == frozen['classification'], 'classification changed since freeze')
        require(expectation['source_sha256'] == frozen['source_sha256'], 'accepted source changed')
        if 'expected_message' in expectation:
            frozen = dict(frozen, expected_message=expectation['expected_message'])
        require(frozen['expected_helpers'] == [], 'zero-helper inventory required')
        require(frozen['required_helper_checks'] == ['soundness','maximality','exactness'], 'helper statuses')
        cases.append((name, case, frozen))
        inputs += [case / 'probe.rs', case / 'freeze.json']
        for rel, sha in frozen.get('dependencies', {}).items():
            require(digest(ROOT / rel) == sha, 'dependency changed: ' + rel)
            inputs.append(ROOT / rel)
    guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(), inputs))
    receipt['artifacts'] = guard.initial
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
        RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.solver.resolve()),
        VERUS_INFER_CACHE=str(a.output.resolve() / 'cache'))
    run = LoggedCommands(a.output, ROOT, receipt['commands'], timeout=180,
                         guard=guard, status_key='returncode')
    for name, case, frozen in cases:
        command = [str(a.verus.resolve()), str(case / 'probe.rs'), '--crate-type=lib',
                   '--no-cheating', '--output-json', '--multiple-errors', '8']
        rc = run(name, command, env)
        result = dict(returncode=rc, stdout=(a.output / (name+'.stdout')).read_text(),
                      stderr=(a.output / (name+'.stderr')).read_text())
        entry = dict(id=name, returncode=rc, freeze=frozen,
            stdout_sha256=digest(a.output / (name+'.stdout')),
            stderr_sha256=digest(a.output / (name+'.stderr')))
        receipt['cases'].append(entry)
        if frozen['classification'] in ('frontend-rejection','lifetime-rejection'):
            data = validate_run(result, False, allow_frontend=True)
            require(data['verification-results'].get('encountered-vir-error') is True
                    or data['verification-results'].get('encountered-error') is True, 'frontend status')
            require(not any(s in result['stderr'] for s in ('postcondition not satisfied', 'precondition not satisfied', 'assertion failed')), 'frontend/semantic mixed failure')
            if frozen.get('expected_message'):
                require(frozen['expected_message'] in result['stderr'], 'wrong frontend diagnostic')
            helpers = data.get('verus-infer', {}).get('functions', [])
            require_function_coverage(helpers, set())
            require_exact(helpers)
        else:
            expected = frozen['classification'] == 'positive'
            data = validate_goal_run(result, expected, expected_functions=set(),
                exec_bodies=frozen['exec_bodies'], proof_bodies=frozen['proof_bodies'],
                failure_kind=frozen.get('failure_kind'))
        expectation = accepted.get(name, {})
        for key in ('verified', 'errors'):
            if key in expectation:
                require(data['verification-results'].get(key) == expectation[key], 'wrong exact query count: ' + name)
        entry['verification'] = data['verification-results']
        entry['verification_scope'] = data.get('verus-infer', {}).get('verification_scope')
        guard.check()
        print(json.dumps(dict(id=name, result=entry['verification'])))
