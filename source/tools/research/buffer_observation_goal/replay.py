#!/usr/bin/env python3
"""Strict ordinary observation replay. Prefix diagnostics never complete transfer."""
import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
    validate_goal_run, validate_run, require_exact, require_function_coverage,
    require_publication)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--stage', choices=['minimal', 'all'], default='all')
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix='observations-', dir=a.output.resolve()))
    r = dict(commands=[], runs=[], goal_complete=False, transfer_complete=False)
    with replay_receipt(out, r):
        freeze = json.loads((HERE / 'freeze-001.json').read_text())
        for name, sha in freeze['files'].items():
            require(digest(HERE / name) == sha, 'changed frozen fixture: ' + name)
        prefix = (HERE / 'prefix-zip.rs').read_text()
        plain = (HERE / 'plain.rs').read_text()
        full = (HERE / 'full-zip.rs').read_text()
        require(prefix == (HERE.parent / 'buffer_transfer_goal/partial-prefix.rs').read_text(),
                'prefix semantic/source candidate changed')
        require(full == (HERE.parent / 'buffer_transfer_goal/candidate-01.rs').read_text(),
                'full property/domain/source candidate changed')
        canonical = (HERE / 'canonical-without-length.rs').read_text()
        require(canonical == (HERE.parent / 'varint_canonical_goal/candidate-004.rs').read_text().replace(
            '                it.seq().len() == 10,\n', ''), 'canonical reduction changed other input')
        guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(),
            extra=[Path(__file__), HERE / 'freeze-001.json', *[HERE / n for n in freeze['files']]]))
        r['artifacts'] = guard.initial
        call = LoggedCommands(out, ROOT / 'source', r['commands'], timeout=240,
                              guard=guard, status_key='returncode')
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
                   RUSTUP_TOOLCHAIN='1.99.1', RUST_MIN_STACK='33554432', VERUS_Z3_PATH=str(a.solver.resolve()))

        def run(label, source, kind='success', goals=True, execs=('xor_key',),
                proofs=(), flags=(), assistance=None, artifact=None):
            path = out / (label + '.rs')
            path.write_text(source)
            trial = dict(label=label, source_sha256=digest(path), kind=kind,
                         goals=goals, expected_helpers=[], exec_bodies=execs,
                         proof_bodies=proofs, before_invocation=True)
            # Freeze even rejected/development candidates before the first invocation.
            (out / (label + '.freeze.json')).write_text(json.dumps(trial, indent=2) + '\n')
            r['runs'].append(trial)
            source_guard = ArtifactGuard([path])
            env['VERUS_INFER_CACHE'] = str(out / 'candidate-cache')
            cmd = [str(a.verus.resolve()), str(path), '--crate-type=lib', '--no-cheating',
                   '--output-json', '--multiple-errors', '20', *flags]
            if goals:
                cmd.append('--infer-goals')
            rc = call(label, cmd, env)
            source_guard.check()
            result = dict(returncode=rc, stdout=(out / (label + '.stdout')).read_text(),
                          stderr=(out / (label + '.stderr')).read_text())
            if kind in ('success', 'postcondition'):
                data = validate_goal_run(result, kind == 'success', expected_functions=set(),
                    exec_bodies=set(execs), proof_bodies=set(proofs),
                    failure_kind=None if kind == 'success' else 'postcondition not satisfied',
                    artifact=artifact)
            else:
                data = validate_run(result, False, allow_frontend=(kind == 'frontend'))
                functions = data.get('verus-infer', {}).get('functions', [])
                require_function_coverage(functions, set())
                require_exact(functions)
                stderr = result['stderr']
                if kind == 'frontend':
                    require('error:' in stderr and data['verification-results'].get('encountered-vir-error') is True, 'expected Verus frontend rejection absent')
                    require(data['verification-results'].get('errors', 0) == 0,
                            'mixed frontend/SMT evidence')
                else:
                    v = data['verification-results']
                    require(v.get('is-verifying-entire-crate') is True and v.get('errors', 0) > 0,
                            'auxiliary/safety checking incomplete')
                    scope = data['verus-infer']['verification_scope']
                    require(scope.get('native_collection_completed') is True, 'missing body inventory')
                    selected = [b for b in scope['native_functions'] if b.get('body_status') ==
                                'selected_body_not_certified_by_successful_full_run']
                    require_function_coverage(selected, set(execs) | set(proofs))
                    require(not re.search(r'error\[E\d+\]', stderr), 'frontend is not proof evidence')
                    require('recommendation not met' not in stderr, 'recommendation diagnostic')
                    if kind == 'safety':
                        require('possible arithmetic underflow/overflow' in stderr, 'missing safety failure')
                        require(not any(s in stderr for s in ('postcondition not satisfied',
                            'invariant not satisfied', 'assertion failed')), 'mixed safety diagnostic')
                    elif kind == 'invariant':
                        require('invariant not satisfied' in stderr, 'missing invariant proposal failure')
                        require('possible arithmetic underflow/overflow' not in stderr, 'mixed safety diagnostic')
                    else:
                        raise ValueError('unknown explicit failure class: ' + kind)
                if artifact is not None:
                    require_publication(data, False, artifact)
            if assistance is not None:
                entries = data['verus-infer']['buffer_observation_assistance']
                require({e['function'].rsplit('::', 1)[-1] for e in entries} == set(assistance),
                        'observation selection mismatch (not proof evidence)')
                require(all(e['snapshots'] == 2 and e['invariants'] == 4 for e in entries),
                        'decoration insertion inventory mismatch')
            trial['verification'] = data['verification-results']
            trial['stdout_sha256'] = digest(out / (label + '.stdout'))
            trial['stderr_sha256'] = digest(out / (label + '.stderr'))
            print(label, trial['verification'], flush=True)
            return data

        for phase in ('cold', 'warm'):
            run('prefix_' + phase, prefix, assistance=['xor_key'])
            run('plain_' + phase, plain, execs=['plain'], assistance=['plain'])
            run('full_' + phase, full, 'postcondition', assistance=['xor_key'])
        run('prefix_disabled', prefix, 'postcondition', goals=False)
        run('plain_disabled', plain, 'success', goals=False, execs=['plain'])
        run('canonical_length', canonical, execs=['u64'],
            proofs=['radix_positive', 'quotient_step', 'byte_step'], assistance=['u64'])
        run('canonical_disabled', canonical, goals=False, execs=['u64'],
            proofs=['radix_positive', 'quotient_step', 'byte_step'])
        if a.stage == 'all':
            two = plain.replace('fn main() {}', '') + '\n' + plain.replace(
                'use vstd::prelude::*;', '').replace('pub fn plain', 'pub fn second')
            run('two_functions', two, execs=['plain', 'second'], assistance=['plain', 'second'])
            run('wrong_value', prefix.replace('old(ipad)@[j] ^ key@[j]',
                'old(ipad)@[j] | key@[j]'), 'postcondition', assistance=['xor_key'])
            run('wrong_length', prefix.replace('ensures forall|j: int|',
                'ensures final(ipad)@.len() == 127, forall|j: int|'), 'postcondition', assistance=['xor_key'])
            run('wrong_current', prefix.replace('*final(it.seq()[j].0) ==',
                '*it.seq()[j].0 =='), 'invariant', assistance=['xor_key'])
            run('wrong_index', prefix.replace('j < it.index()', 'j < it.index() + 1 && j < it.seq().len()'),
                'invariant', assistance=['xor_key'])
            run('wrong_ref', prefix.replace('*it.seq()[j].1)', '*it.seq()[j].0)'),
                'invariant', assistance=['xor_key'])
            safety = '''use vstd::prelude::*; verus! {
                fn overflow(buf: &mut [u8;10]) { for b in buf.iter_mut() { *b += 1; } }
            } fn main() {}'''
            run('safety', safety, 'safety', execs=['overflow'], assistance=['overflow'])
            # Same spelling, custom receiver: must skip, and retain the false goal.
            custom = '''use vstd::prelude::*; verus! {
                struct Custom { value: [u8;10] }
                impl Custom { fn iter__mut(&mut self) -> (r: &[u8;10]) { &self.value } }
                fn client(c: &mut Custom) ensures final(c).value@[0] == 7 { let r=c.iter__mut(); }
            } fn main() {}'''
            run('custom', custom, 'postcondition', execs=['iter__mut', 'client'], assistance=[])
            manual = (HERE.parent / 'buffer_transfer_goal/partial-prefix-manual-bounds.rs').read_text()
            wrong_state = manual.replace('let ghost before = ipad@;', 'let ghost before = final(ipad)@;')
            run('wrong_snapshot_state', wrong_state, 'invariant', assistance=['xor_key'])
            invalid_mode = prefix.replace('\n{\n    for ', '\n{\n    let ghost ghost_byte = 0u8;\n    let exec_byte: u8 = ghost_byte;\n    for ', 1)
            run('mode_invalid_snapshot', invalid_mode, 'frontend')
            exec_write = prefix.replace('\n{\n    for ', '\n{\n    proof { *ipad = *ipad; }\n    for ', 1)
            run('ghost_executable_write', exec_write, 'frontend')
            artifact = out / 'seeded.vir'
            flags = ['--crate-name=seeded', '--export', str(artifact)]
            run('seed_prefix', prefix, flags=flags, assistance=['xor_key'], artifact=artifact)
            run('remove_failed_full', full, 'postcondition', flags=flags,
                assistance=['xor_key'], artifact=artifact)
        guard.check()
    print(out / 'receipt.json')

if __name__ == '__main__':
    main()
