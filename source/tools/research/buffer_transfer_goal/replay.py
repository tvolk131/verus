#!/usr/bin/env python3
"""Replay the independently frozen source-slice baseline, never a completion claim."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile
import textwrap
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require, require_exact,
    require_failure_kind, require_function_coverage, validate_run)


def fidelity():
    frozen = json.loads((HERE / 'freeze.json').read_text())
    for path, sha in frozen['files'].items():
        require(digest(HERE / path) == sha, 'frozen source changed: ' + path)
    upstream = (HERE / 'upstream/src/hmac.rs').read_text()
    selected = (HERE / 'original-selected.rs').read_text()
    require(selected in upstream, 'selected loop missing from upstream')
    baseline = (HERE / 'baseline.rs').read_text()
    start = baseline.index('    for (b_i, b_h)')
    end = baseline.index('\n}', start)
    require(textwrap.dedent(selected).strip() == textwrap.dedent(baseline[start:end]).strip(),
            'loop source mismatch')


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix='baseline-', dir=a.output.resolve()))
    receipt = {'commands': [], 'runs': [], 'goal_complete': False}
    with replay_receipt(out, receipt):
        fidelity()
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
            RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.solver.resolve()))
        guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(),
            extra=[HERE / p for p in json.loads((HERE / 'freeze.json').read_text())['files']] +
            [HERE / 'freeze.json', Path(__file__)]))
        receipt['tool_inputs'] = guard.initial
        call = LoggedCommands(out, out, receipt['commands'], timeout=180, guard=guard,
            status_key='returncode')
        require(call('version', [str(a.verus.resolve()), '--version'], env) == 0, 'version failed')
        receipt['version'] = (out / 'version.stdout').read_text()
        receipt['frozen_source_commit'] = '7fe03428'
        def run(label, text, expected, flags, error=None, cache='none'):
            path = out / (label + '.rs')
            path.write_text(text)
            cmd = [str(a.verus.resolve()), str(path), '--no-cheating', '--output-json',
                   '--multiple-errors', '20', *flags]
            env['VERUS_INFER_CACHE'] = str(out / ('cache-' + cache))
            rc = call(label, cmd, env)
            result = dict(returncode=rc, stdout=(out / (label + '.stdout')).read_text(),
                stderr=(out / (label + '.stderr')).read_text())
            record = dict(label=label, returncode=rc, source_sha256=digest(path),
                stdout_sha256=digest(out / (label + '.stdout')),
                stderr_sha256=digest(out / (label + '.stderr')))
            receipt['runs'].append(record)
            data = validate_run(result, expected)
            record['result'] = data
            require(data['verification-results'].get('is-verifying-entire-crate') is True,
                    'verification scope incomplete')
            functions = data.get('verus-infer', {}).get('functions', [])
            require_function_coverage(functions, set())
            require_exact(functions)
            record['expected_helpers'] = []
            if error == 'possible arithmetic underflow/overflow':
                require(error in result['stderr'], 'missing overflow safety diagnostic')
                require(not any(x in result['stderr'] for x in ['postcondition not satisfied', 'precondition not satisfied', 'assertion failed']), 'mixed safety failure')
            elif error:
                require_failure_kind(result['stderr'], error)
            print(label, rc, data['verification-results'], flush=True)
        baseline = (HERE / 'baseline.rs').read_text()
        for goals in [False, True]:
            for phase in ['cold', 'warm']:
                run(f'full_goals_{goals}_{phase}', baseline, False,
                    ['--infer-goals'] if goals else [], 'postcondition not satisfied',
                    cache=str(goals))
        safety = baseline.replace('    ensures pad_xor(old(ipad)@, key@, final(ipad)@),\n', '')
        run('safety_only', safety, True, ['--infer-goals'])
        # False semantic goal on a concrete instance, separated from writer safety.
        model = baseline[:baseline.index('pub fn xor_key')]
        model += '''proof fn literal_empty_key(before: Seq<u8>)
    requires before.len() == 128,
    ensures pad_xor(before, Seq::empty(), before),
{}
proof fn wrong_frame() {
    assert(pad_xor(Seq::new(128, |i:int| 0u8), Seq::empty(), Seq::new(128, |i:int| 1u8)));
}
} fn main() {}
'''
        run('semantic_wrong_frame', model, False, ['--infer-goals'], 'assertion failed')
        # Overflow is a safety control, explicitly not a semantic rejection.
        run('safety_overflow', safety.replace('*b_i ^= *b_h;', '*b_i += *b_h;'),
            False, ['--infer-goals'], 'possible arithmetic underflow/overflow')
        receipt['classification'] = 'expected baseline failures reproduced; no transfer automation proved'
        guard.check()
    print(out / 'receipt.json')

if __name__ == '__main__':
    main()
