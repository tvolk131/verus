#!/usr/bin/env python3
"""One frozen ordinary diagnostic; raw output is private, all failures retained."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
    require_exact, require_function_coverage, validate_run)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('source', type=Path)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--expect', choices=('success', 'failure', 'frontend'), required=True)
    p.add_argument('--goals', action='store_true')
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix=a.source.stem + '-', dir=a.output.resolve()))
    source = out / 'candidate.rs'
    source.write_bytes(a.source.read_bytes())
    # This file exists before any compiler invocation, including unsuccessful ones.
    freeze = dict(source=str(a.source.resolve()), sha256=digest(source),
        expected_helpers=[], ordinary=True, infer_goals=a.goals,
        expected_outcome=a.expect)
    (out / 'input-freeze.json').write_text(json.dumps(freeze, indent=2) + '\n')
    receipt = dict(input=freeze, output=str(out), commands=[])
    with replay_receipt(out, receipt):
        guard = ArtifactGuard(tool_artifacts(a.bundle / 'verus', a.bundle / 'z3',
            extra=[source, Path(__file__)]))
        receipt['artifacts'] = guard.initial
        invoke = LoggedCommands(out, out, receipt['commands'], timeout=180, guard=guard,
            status_key='returncode')
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
            RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.bundle / 'z3'),
            VERUS_INFER_CACHE=str(out / 'cache'))
        command = [str(a.bundle / 'verus'), str(source), '--crate-type=lib',
            '--no-cheating', '--output-json', '--multiple-errors', '20']
        if a.goals:
            command.append('--infer-goals')
        rc = invoke('verify', command, env)
        result = dict(returncode=rc, stdout=(out / 'verify.stdout').read_text(),
            stderr=(out / 'verify.stderr').read_text())
        data = json.loads(result['stdout'])
        receipt['verification'] = data.get('verification-results')
        receipt['verifier_json'] = data
        receipt['stdout_sha256'] = digest(out / 'verify.stdout')
        receipt['stderr_sha256'] = digest(out / 'verify.stderr')
        functions = data.get('verus-infer', {}).get('functions', [])
        require_function_coverage(functions, set())
        require_exact(functions)
        validate_run(result, a.expect == 'success', allow_frontend=a.expect == 'frontend')
        require(data['verification-results'].get('is-verifying-entire-crate'), 'partial scope')
        require(not any(flag in command for flag in ('--no-lifetime', '--no-verify',
            '--verify-function', '--verify-root', '--verify-only-module')), 'disabled checks')
        print(result['stderr'])
    print(out / 'receipt.json')

if __name__ == '__main__':
    main()
