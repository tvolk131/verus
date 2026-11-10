#!/usr/bin/env python3
"""Guarded ordinary reference-boundary probes. Never publishes native facts."""
import argparse, json, os, sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
    require_exact, require_function_coverage, validate_goal_run, validate_run)
p = argparse.ArgumentParser()
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--freeze', type=Path, required=True)
p.add_argument('--cases', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output = a.output.resolve(); a.output.mkdir(parents=True, exist_ok=False)
r = dict(schema=1, commands=[], cases=[], publication=False, native_bridge_proved=False)
with replay_receipt(a.output, r):
    f = json.loads(a.freeze.read_text())
    require(f['before_first_trial'] is True, 'missing pretrial freeze')
    for n, sha in f['files'].items():
        require(digest(ROOT/n) == sha, 'source changed: '+n)
    for q in (Path(__file__), a.cases):
        require(str(q.resolve().relative_to(ROOT)) in f['files'], 'unfrozen controller/cases')
    cases = json.loads(a.cases.read_text())
    ident_path = ROOT/'source/tools/research/native_heap_lowering/bundle-identity.json'
    ident = json.loads(ident_path.read_text())
    originals = []
    for n, x in ident.items():
        q = a.bundle/n; original = Path(x['source']); originals.append(original)
        require(digest(q) == digest(original) == x['sha256'], 'runtime hash: '+n)
        require(not q.is_symlink() and q.stat().st_ino != original.stat().st_ino, 'runtime copy: '+n)
    guard = ArtifactGuard(tool_artifacts(a.bundle/'verus', a.bundle/'z3',
        [*[ROOT/n for n in f['files']], a.freeze, ident_path, *originals]))
    r['artifacts'] = guard.initial
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],
        RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.bundle.resolve()/'z3'),
        VERUS_INFER_CACHE=str(a.output/'cache'))
    run = LoggedCommands(a.output, a.output, r['commands'], timeout=180,
                         guard=guard, status_key='returncode')
    for c in cases:
        source = (ROOT/c['file']).resolve()
        require(str(source.relative_to(ROOT)) in f['files'], 'unfrozen input')
        log = a.output/c['id']; log.mkdir()
        rc = run(c['id'], [str(a.bundle.resolve()/'verus'), str(source), '--crate-type=lib',
            '--no-cheating', '--output-json', '--multiple-errors', '8',
            '--log', 'vir', '--log', 'vir-sst', '--log', 'air-final', '--log-dir', str(log)], env)
        result = dict(returncode=rc, stdout=(a.output/(c['id']+'.stdout')).read_text(),
                      stderr=(a.output/(c['id']+'.stderr')).read_text())
        entry = dict(id=c['id'], classification=c['classification'], source_sha256=digest(source),
            stdout_sha256=digest(a.output/(c['id']+'.stdout')),
            stderr_sha256=digest(a.output/(c['id']+'.stderr')))
        r['cases'].append(entry)
        if c['classification'] in ('frontend-rejection', 'lifetime-rejection'):
            data = validate_run(result, False, allow_frontend=True)
            require(c['message'] in result['stderr'], 'wrong diagnostic: '+c['id'])
            require(not any(s in result['stderr'] for s in ('postcondition not satisfied',
                'precondition not satisfied', 'assertion failed')), 'mixed frontend/semantic failures')
            helpers = data.get('verus-infer', {}).get('functions', [])
            require_function_coverage(helpers, set()); require_exact(helpers)
        else:
            data = validate_goal_run(result, c['classification'] == 'positive',
                expected_functions=set(), exec_bodies=c['exec_bodies'],
                proof_bodies=c['proof_bodies'], failure_kind=c.get('failure_kind'))
        for key in ('verified', 'errors'):
            if key in c:
                require(data['verification-results'].get(key) == c[key], 'wrong count: '+c['id']+' '+key)
        entry['verification'] = data['verification-results']
        entry['scope'] = data.get('verus-infer', {}).get('verification_scope')
        entry['logs'] = {str(q.relative_to(a.output)): digest(q) for q in log.rglob('*') if q.is_file()}
        guard.check()
        print(json.dumps(dict(id=c['id'], verification=entry['verification'])), flush=True)
