#!/usr/bin/env python3
"""Strict scopes, causal ablation and ordinary imported selection clients."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import (LoggedCommands, replay_receipt, require, require_exact,
                             require_function_coverage, require_publication, validate_run,
                             require_failure_kind)

def validate(out, label, code, good, names, helpers, artifact, failure=None):
    stdout = (out / (label + '.stdout')).read_text()
    stderr = (out / (label + '.stderr')).read_text()
    data = validate_run(dict(returncode=code, stdout=stdout, stderr=stderr), good)
    inf = data['verus-infer']; scope = inf['verification_scope']
    require_exact(inf['functions']); require_function_coverage(inf['functions'], helpers)
    require(data['verification-results']['is-verifying-entire-crate'] is True, 'full crate')
    require(scope['native_collection_completed'] is True, 'native inventory')
    statuses = {'checked_exec_body'} if good else {'checked_exec_body', 'selected_body_not_certified_by_successful_full_run'}
    actual = {b['rust_function'] for b in scope['native_functions'] if b['body_status'] in statuses}
    require(actual == set(names), 'exact body set')
    require(not any(b['selected'] and b['body_status'] in
                    ['unverified_body_with_contract', 'external_specification', 'interface_without_checked_body']
                    for b in scope['native_functions']), 'unchecked selected contract')
    if good:
        require(scope['complete_selected_scope_verified'] and scope['checked_exec_bodies'] == len(names)
                and scope['checked_proof_bodies'] == 0, 'positive scope')
    else:
        require_failure_kind(stderr, failure)
        require(not any(x in stderr for x in ['possible arithmetic underflow/overflow', 'invariant not satisfied',
                                              'recommendation not met', 'error[E']), 'mixed failure')
    require_publication(data, good, artifact)
    return data


CLIENT = r'''extern crate selection_checked;
use selection_checked::*;
use vstd::prelude::*;
verus! {
pub fn imported_selection(h: &mut UnrepairedDatabaseHeader) -> (r: Result<bool>)
    requires old(h).inner.primary_slot < 2, (old(h).inner.primary_slot ^ 1) < 2,
        old(h).inner.primary_slot != (old(h).inner.primary_slot ^ 1),
    ensures r.is_Err() == selection_errors(*old(h)),
        match r { Ok(kept) => kept == !selection_switches(*old(h)), _ => true },
        final(h).inner.primary_slot == (if selection_switches(*old(h)) {
            old(h).inner.primary_slot ^ 1 } else { old(h).inner.primary_slot }),
        final(h).inner.transaction_slots == old(h).inner.transaction_slots,
{ h.select_primary_slot() }
}
'''

UNSAFE_CLIENT = r'''extern crate selection_checked;
use selection_checked::*;
use vstd::prelude::*;
verus! {
pub fn imported_selection(h: &mut UnrepairedDatabaseHeader) -> Result<bool>
{ h.select_primary_slot() }
}
'''

def unsafe_provider(source):
    start = source.index('    pub fn select_primary_slot')
    body = source.index('{\n        // If the primary', start)
    return source[:start] + '    pub fn select_primary_slot(&mut self) -> Result<bool>\nrequires true,\n' + source[body:]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--freeze', type=Path, default=HERE / 'VALIDATION_FREEZE.json')
    args = parser.parse_args(); out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    receipt = dict(kind='CHECKED_ORIGINAL_SELECTION_STRICT_CLOSEOUT', commands=[], outcomes=[],
                   source_coupling_proved=False)
    with replay_receipt(out, receipt):
        freeze = json.loads(args.freeze.read_text())
        for name, value in freeze['inputs'].items(): require(digest(Path(name)) == value, 'input ' + name)
        rt = freeze['runtime']; original = Path(rt['bundle']); bundle = out / 'bundle'; bundle.mkdir()
        require(digest(original / 'receipt.json') == rt['receipt_sha256'], 'accepted receipt')
        for name, value in rt['tool_hashes'].items():
            require(digest(original / name) == value, 'accepted tool')
            shutil.copy2(original / name, bundle / name); require(digest(bundle / name) == value, 'private tool')
        shutil.copy2(original / 'verus-root', bundle / 'verus-root')
        source = (HERE / 'checked_derives.rs').read_text()
        sources = {'provider': source,
                   'wrong': source.replace('kept == !selection_switches(*old(self))', 'kept == selection_switches(*old(self))'),
                   'unsafe': unsafe_provider(source)}
        for name, text in sources.items():
            path = out / (name + '.rs'); path.write_text(text)
            require(digest(path) == freeze['fixtures'][name], 'frozen variant ' + name)
        guard = ArtifactGuard([args.freeze] + [Path(p) for p in freeze['inputs']] +
                              [bundle / n for n in rt['tool_hashes']] + [out / (n + '.rs') for n in sources])
        run = LoggedCommands(out, out, receipt['commands'], timeout=120, guard=guard)
        tc = freeze['toolchain']
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + tc + '/bin:/usr/bin:/bin',
                   RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=tc,
                   LD_LIBRARY_PATH=tc + '/lib', VERUS_Z3_PATH=str(bundle / 'z3'))
        providers = []
        for mode in ['ordinary', 'infer']:
            cache = str(out / ('cache_' + mode))
            for case in ['provider_cold', 'provider_warm', 'wrong', 'unsafe']:
                provider = case.startswith('provider'); source_name = 'provider' if provider else case
                label = mode + '_' + case; artifact = out / (label + '.vir')
                if not provider: shutil.copy2(out / (mode + '_provider_cold.vir'), artifact)
                argv = [str(bundle / 'verus'), str(out / (source_name + '.rs')), '--crate-type=lib',
                        '--crate-name=selection_checked', '--edition=2024', '--no-cheating', '--output-json',
                        '--time', '--multiple-errors', '8', '--export', str(artifact)]
                if case == 'provider_cold': argv += ['--compile', '-o', str(out / ('lib_' + mode + '.rlib'))]
                if mode == 'infer': argv.append('--infer-goals')
                code = run(label, argv, dict(env, VERUS_INFER_CACHE=cache if provider else str(out / ('cache_' + label))))
                data = validate(out, label, code, provider, freeze['provider_bodies'], {'eq'}, artifact,
                                'postcondition not satisfied' if case == 'wrong' else 'precondition not satisfied')
                if provider: require(data['verification-results']['verified'] == 23, 'positive obligation count')
                receipt['outcomes'].append(dict(label=label, data=data)); print(label, 'expected outcome', flush=True)
            providers.append(mode)
        for mode in providers:
            for case in ['healthy', 'wrong', 'unsafe', 'failed_import']:
                text = CLIENT
                if case == 'wrong': text = text.replace('r.is_Err() == selection_errors(*old(h))', 'r.is_Err() == !selection_errors(*old(h))')
                if case == 'unsafe': text = UNSAFE_CLIENT
                label = mode + '_client_' + case; path = out / (label + '.rs'); path.write_text(text)
                require(digest(path) == freeze['clients'][case], 'frozen client ' + case)
                artifact = out / (label + '.vir')
                shutil.copy2(out / (mode + '_provider_cold.vir'), artifact)
                imported = out / (mode + '_provider_cold.vir') if case != 'failed_import' else out / (mode + '_wrong.vir')
                argv = [str(bundle / 'verus'), str(path), '--crate-type=lib', '--crate-name=client', '--edition=2024',
                        '--no-cheating', '--output-json', '--time', '--infer-goals', '--export', str(artifact),
                        '--extern', 'selection_checked=' + str(out / ('lib_' + mode + '.rlib')),
                        '--import', 'selection_checked=' + str(imported)]
                code = run(label, argv, dict(env, VERUS_INFER_CACHE=str(out / ('cache_' + label))))
                if case == 'failed_import':
                    stderr = (out / (label + '.stderr')).read_text()
                    require(code != 0 and ('No such file' in stderr or 'failed to read' in stderr), 'missing failed export rejects')
                    data = None
                else:
                    data = validate(out, label, code, case == 'healthy', {'imported_selection'}, set(), artifact,
                                    'postcondition not satisfied' if case == 'wrong' else 'precondition not satisfied')
                receipt['outcomes'].append(dict(label=label, data=data)); print(label, 'expected outcome', flush=True)
        guard.check(); receipt['guard_passed'] = True

if __name__ == '__main__': main()
