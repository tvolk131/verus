#!/usr/bin/env python3
"""Replay the independently frozen source-slice baseline, never a completion claim."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile
import re
import textwrap
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require, require_exact,
    require_failure_kind, require_function_coverage, require_publication, validate_run)


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
    # Explicit erasure check for the positive intended/manual fixtures: only
    # registered ghost snapshots and the for-loop annotations may disappear.
    expected = textwrap.dedent(selected).strip()
    for filename in ['candidate-01.rs', 'partial-prefix.rs', 'partial-prefix-manual.rs',
                     'partial-prefix-manual-bounds.rs', 'full-manual.rs', 'full-manual-bounds.rs']:
        source = (HERE / filename).read_text()
        body = source[source.index('\n{', source.index('pub fn xor_key')) + 2:]
        body = body[:body.rindex('\n}\n}')].strip()
        body = body.replace('    let ghost before = ipad@;\n', '')
        body = body.replace('    let ghost future = final(ipad)@;\n', '')
        body = body.removeprefix('let ghost before = ipad@;\n')
        body = body.removeprefix('let ghost future = final(ipad)@;\n')
        body = re.sub(r'for \(b_i, b_h\) in it: ipad.iter_mut\(\).zip\(key\)\n.*?\n    \{',
                      'for (b_i, b_h) in ipad.iter_mut().zip(key) {', body, count=1, flags=re.S)
        require(re.sub(r'\s+', '', body) == re.sub(r'\s+', '', expected),
                'executable source adaptation changed: ' + filename)



def main():
    p = argparse.ArgumentParser()
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--stage', choices=['baseline', 'all'], default='all')
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix='baseline-', dir=a.output.resolve()))
    receipt = {'commands': [], 'runs': [], 'frozen_trials': [], 'goal_complete': False}
    with replay_receipt(out, receipt):
        fidelity()
        # Retain each authored-input freeze; never rewrite earlier candidates.
        candidate_sources = []
        for manifest in sorted(HERE.glob('*.freeze.json')):
            record = json.loads(manifest.read_text())
            entries = record.get('files', {record.get('file'): record})
            for filename, entry in entries.items():
                require(digest(HERE / filename) == entry['sha256'], 'changed candidate: ' + filename)
                candidate_sources.append(HERE / filename)
            candidate_sources.append(manifest)
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
            RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.solver.resolve()))
        guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(),
            extra=[HERE / p for p in json.loads((HERE / 'freeze.json').read_text())['files']] +
            [HERE / 'freeze.json', Path(__file__), *candidate_sources]))
        receipt['tool_inputs'] = guard.initial
        call = LoggedCommands(out, out, receipt['commands'], timeout=180, guard=guard,
            status_key='returncode')
        require(call('version', [str(a.verus.resolve()), '--version'], env) == 0, 'version failed')
        receipt['version'] = (out / 'version.stdout').read_text()
        require(call('solver_version', [str(a.solver.resolve()), '--version'], env) == 0,
                'solver version failed')
        receipt['solver_version'] = (out / 'solver_version.stdout').read_text()
        receipt['frozen_source_commit'] = '7fe03428'
        def run(label, text, expected, flags, error=None, cache='none', counts=None):
            path = out / (label + '.rs')
            path.write_text(text)
            cmd = [str(a.verus.resolve()), str(path), '--no-cheating', '--output-json',
                   '--multiple-errors', '20', *flags]
            env['VERUS_INFER_CACHE'] = str(out / ('cache-' + cache))
            source_guard = ArtifactGuard([path])
            receipt['frozen_trials'].append(dict(label=label, source_sha256=digest(path),
                expected_helpers=[], expected_success=expected, expected_error=error,
                expected_query_counts=counts, before_invocation=True))
            (out / 'frozen-trials.json').write_text(json.dumps(receipt['frozen_trials'], indent=2) + '\n')
            rc = call(label, cmd, env)
            source_guard.check()
            result = dict(returncode=rc, stdout=(out / (label + '.stdout')).read_text(),
                stderr=(out / (label + '.stderr')).read_text())
            record = dict(label=label, returncode=rc, source_sha256=digest(path),
                stdout_sha256=digest(out / (label + '.stdout')),
                stderr_sha256=digest(out / (label + '.stderr')))
            receipt['runs'].append(record)
            data = validate_run(result, expected, allow_frontend=(error == 'E0502'))
            record['result'] = data
            require(data['verification-results'].get('is-verifying-entire-crate') is True,
                    'verification scope incomplete')
            functions = data.get('verus-infer', {}).get('functions', [])
            require_function_coverage(functions, set())
            require_exact(functions)
            record['expected_helpers'] = []
            if counts is not None:
                require((data['verification-results']['verified'], data['verification-results']['errors']) == counts,
                        'unexpected proof-query counts')
            if expected:
                require(data['verus-infer']['verification_scope']['complete_selected_scope_verified'] is True,
                        'positive did not certify complete scope')
            if error == 'E0502':
                require('error[E0502]' in result['stderr'], 'missing lifetime rejection')
                require(not any(x in result['stderr'] for x in ['postcondition not satisfied',
                    'precondition not satisfied', 'assertion failed', 'possible arithmetic']), 'mixed lifetime failure')
            elif error == 'possible arithmetic underflow/overflow':
                require(error in result['stderr'], 'missing overflow safety diagnostic')
                require(not any(x in result['stderr'] for x in ['postcondition not satisfied', 'precondition not satisfied', 'assertion failed']), 'mixed safety failure')
            elif error:
                require_failure_kind(result['stderr'], error)
                require(not any(x in result['stderr'] for x in ['possible arithmetic underflow/overflow',
                    'invariant not satisfied', 'recommendation not met', 'error[E']), 'mixed ordinary failure')
            print(label, rc, data['verification-results'], flush=True)
            return data
        baseline = (HERE / 'baseline.rs').read_text()
        for goals in [False, True]:
            for phase in ['cold', 'warm']:
                run(f'full_goals_{goals}_{phase}', baseline, False,
                    ['--infer-goals'] if goals else [], 'postcondition not satisfied',
                    cache=str(goals), counts=(1, 1))
        safety = baseline.replace('    ensures pad_xor(old(ipad)@, key@, final(ipad)@),\n', '')
        run('safety_only', safety, True, ['--infer-goals'], counts=(2, 0))
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
        run('semantic_wrong_frame', model, False, ['--infer-goals'], 'assertion failed', counts=(1, 1))
        # Overflow is a safety control, explicitly not a semantic rejection.
        run('safety_overflow', safety.replace('*b_i ^= *b_h;', '*b_i += *b_h;'),
            False, ['--infer-goals'], 'possible arithmetic underflow/overflow', counts=(1, 1))
        if a.stage == 'all':
            run('candidate_01', (HERE / 'candidate-01.rs').read_text(), False, [],
                'postcondition not satisfied', counts=(1, 1))
            run('candidate_02_lifetime', (HERE / 'candidate-02.rs').read_text(), False, [],
                'E0502', counts=(0, 0))
            run('prefix_without_bridges', (HERE / 'partial-prefix.rs').read_text(), False, [],
                'postcondition not satisfied', counts=(1, 1))
            partial = (HERE / 'partial-prefix-manual-bounds.rs').read_text()
            full = (HERE / 'full-manual-bounds.rs').read_text()
            for goals in [False, True]:
                for phase in ['cold', 'warm']:
                    flags = ['--infer-goals'] if goals else []
                    run(f'partial_manual_{goals}_{phase}', partial, True, flags,
                        cache='partial-' + str(goals), counts=(2, 0))
                    run(f'full_manual_{goals}_{phase}', full, False, flags,
                        'postcondition not satisfied', cache='full-' + str(goals), counts=(1, 1))
            run('missing_frame_interface', (HERE / 'frame-repro.rs').read_text(), False,
                ['--infer-goals'], 'postcondition not satisfied', counts=(0, 2))
            # This is a negative of the proved partial theorem, not the already-failing full goal.
            wrong = partial.replace('final(ipad)@[j] == (old(ipad)@[j] ^ key@[j])',
                                    'final(ipad)@[j] == (old(ipad)@[j] | key@[j])')
            require(wrong != partial, 'partial semantic mutant did not apply')
            run('partial_wrong_byte', wrong, False, ['--infer-goals'],
                'postcondition not satisfied', counts=(1, 1))
            artifact = out / 'seeded.vir'
            flags = ['--infer-goals', '--crate-type=lib', '--crate-name=seeded', '--export', str(artifact)]
            data = run('seed_partial_interface', partial, True, flags, counts=(2, 0))
            require_publication(data, True, artifact)
            data = run('remove_failed_full_interface', full, False, flags,
                'postcondition not satisfied', counts=(1, 1))
            require_publication(data, False, artifact)
        receipt['classification'] = 'expected baseline failures reproduced; no transfer automation proved'
        guard.check()
    print(out / 'receipt.json')

if __name__ == '__main__':
    main()
