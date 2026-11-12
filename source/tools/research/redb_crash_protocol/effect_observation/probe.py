#!/usr/bin/env python3
"""Native-state observations under unchanged accepted contracts, diagnostic only."""
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
from infer_experiment import (LoggedCommands, replay_receipt, require, validate_goal_run,
                             validate_run, require_function_coverage)

ATOMIC = '''use vstd::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
verus! {
pub fn atomic_observation() -> (result: bool)
CONTRACT
{
    let flag = AtomicBool::new(false);
    flag.store(true, Ordering::Release);
    flag.load(Ordering::Acquire)
}
}
'''
LOCK = '''use vstd::prelude::*;
use std::sync::Mutex;
verus! {
pub fn lock_observation(lock: &Mutex<u64>) -> u64 {
    let guard = lock.lock().unwrap();
    *guard
}
}
'''
CASES = {'atomic_body': ATOMIC.replace('CONTRACT', ''),
         'atomic_read_true': ATOMIC.replace('CONTRACT', 'ensures result,'),
         'native_lock': LOCK}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(); out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    result = dict(kind='MINIMAL_NATIVE_EFFECT_OBSERVATION_DIAGNOSTIC', commands=[], outcomes=[],
                  source_coupling_proved=False, production_facts_added=0)
    with replay_receipt(out, result):
        freeze = json.loads((HERE / 'FROZEN.json').read_text())
        for path, value in freeze['inputs'].items(): require(digest(Path(path)) == value, 'input ' + path)
        runtime = freeze['runtime']; original = Path(runtime['bundle']); bundle = out / 'bundle'; bundle.mkdir()
        require(digest(original / 'receipt.json') == runtime['receipt_sha256'], 'accepted receipt')
        for name, value in runtime['tool_hashes'].items():
            require(digest(original / name) == value, 'accepted tool')
            shutil.copy2(original / name, bundle / name); require(digest(bundle / name) == value, 'private tool')
        shutil.copy2(original / 'verus-root', bundle / 'verus-root')
        for name, text in CASES.items():
            path = out / (name + '.rs'); path.write_text(text)
            require(digest(path) == freeze['fixtures'][name], 'frozen diagnostic ' + name)
        guard = ArtifactGuard([HERE / 'FROZEN.json'] + [Path(p) for p in freeze['inputs']] +
                              [bundle / name for name in runtime['tool_hashes']] +
                              [out / (name + '.rs') for name in CASES])
        run = LoggedCommands(out, out, result['commands'], timeout=120, guard=guard)
        tc = freeze['toolchain']; env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + tc + '/bin:/usr/bin:/bin',
            RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=tc, LD_LIBRARY_PATH=tc + '/lib',
            VERUS_Z3_PATH=str(bundle / 'z3'))
        for name in CASES:
            for infer in [False, True]:
                label = name + ('_infer' if infer else '_ordinary')
                argv = [str(bundle / 'verus'), str(out / (name + '.rs')), '--crate-type=lib', '--edition=2024',
                        '--no-cheating', '--output-json', '--time']
                if infer: argv.append('--infer-goals')
                code = run(label, argv, dict(env, VERUS_INFER_CACHE=str(out / ('cache_' + label))))
                stdout = (out / (label + '.stdout')).read_text(); stderr = (out / (label + '.stderr')).read_text()
                evidence = dict(returncode=code, stdout=stdout, stderr=stderr)
                if name.startswith('atomic'):
                    data = validate_goal_run(evidence, name == 'atomic_body', expected_functions=set(),
                        exec_bodies={'atomic_observation'}, proof_bodies=set(),
                        failure_kind=None if name == 'atomic_body' else 'postcondition not satisfied')
                    require(data['verification-results']['errors'] == (0 if name == 'atomic_body' else 1),
                            'isolated atomic diagnostic')
                else:
                    data = validate_run(evidence, False, allow_frontend=True)
                    require(data['verification-results'].get('encountered-vir-error') is True and
                            'Mutex' in stderr and 'unsupported' in stderr.lower(), 'actual native mutex frontier')
                    require_function_coverage(data.get('verus-infer', {}).get('functions', []), [])
                result['outcomes'].append(dict(label=label, data=data, stderr_sha256=digest(out / (label + '.stderr'))))
                print(label, data['verification-results'], flush=True)
        guard.check(); result['guard_passed'] = True

if __name__ == '__main__': main()
