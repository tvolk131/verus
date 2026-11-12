#!/usr/bin/env python3
"""One existing-type-declaration continuation of the actual backend fixture."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
CRASH = HERE.parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import (LoggedCommands, replay_receipt, require, require_function_coverage,
                             require_failure_kind)

NOMINAL = '''
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIoError(std::io::Error);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPanicLocation<'a>(core::panic::Location<'a>);
'''
ADAPTER = '''
impl vstd::std_specs::convert::FromSpecImpl<std::io::Error> for StorageError {
    open spec fn obeys_from_spec() -> bool { true }
    closed spec fn from_spec(error: std::io::Error) -> Self { StorageError::Io(error) }
}
'''


def fixture(directory):
    sys.path.insert(0, str(CRASH / 'source'))
    spec = importlib.util.spec_from_file_location('original_redb_probes', CRASH / 'source/probes.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.generate(directory)
    text = (directory / 'backend_sync.rs').read_text()
    return text.replace('verus! {', 'verus! {\n' + NOMINAL, 1)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--freeze', type=Path, default=HERE / 'FROZEN.json')
    parser.add_argument('--adapter', action='store_true')
    parser.add_argument('--wrong-adapter', action='store_true')
    args = parser.parse_args()
    require(not args.wrong_adapter or args.adapter, 'wrong adapter requires the adapter case')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    result = dict(kind='ACTUAL_BACKEND_NOMINAL_ONLY_CONTINUATION', commands=[], outcomes=[],
                  source_coupling_proved=False)
    with replay_receipt(out, result):
        freeze = json.loads(args.freeze.read_text())
        for path, value in freeze['inputs'].items():
            require(digest(Path(path)) == value, 'frozen input ' + path)
        source = out / 'backend_nominal.rs'
        text = fixture(out)
        if args.adapter:
            text = text.rsplit('}', 1)[0] + ADAPTER + '\n}\n'
        if args.wrong_adapter:
            text = text.replace('Self { StorageError::Io(error) }', 'Self { StorageError::Unsupported }')
        source.write_text(text)
        require(digest(source) == freeze['fixture_sha256'], 'prospective fixture')
        cases = json.loads((CRASH / 'CASES.json').read_text())
        rt = freeze['runtime']
        original = Path(rt['bundle'])
        require(digest(original / 'receipt.json') == rt['receipt_sha256'], 'accepted receipt')
        bundle = out / 'bundle'
        bundle.mkdir()
        for name, value in rt['tool_hashes'].items():
            require(digest(original / name) == value, 'accepted tool ' + name)
            shutil.copy2(original / name, bundle / name)
            require(digest(bundle / name) == value, 'private tool ' + name)
        shutil.copy2(original / 'verus-root', bundle / 'verus-root')
        tc = Path(freeze['toolchain'])
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + str(tc / 'bin') + ':/usr/bin:/bin',
                   RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=str(tc),
                   LD_LIBRARY_PATH=str(tc / 'lib'), VERUS_Z3_PATH=str(bundle / 'z3'))
        guard = ArtifactGuard([Path(p) for p in freeze['inputs']] + [args.freeze, source] +
                              [bundle / name for name in rt['tool_hashes']])
        run = LoggedCommands(out, out, result['commands'], timeout=120, guard=guard)
        for infer in [False, True]:
            label = 'infer' if infer else 'ordinary'
            argv = [str(bundle / 'verus'), str(source), '--crate-type=lib', '--edition=2024',
                    '--no-cheating', '--output-json', '--time', '--multiple-errors', '8']
            if infer:
                argv.append('--infer-goals')
            code = run(label, argv, dict(env, VERUS_INFER_CACHE=str(out / ('cache_' + label))))
            stdout = (out / (label + '.stdout')).read_text()
            stderr = (out / (label + '.stderr')).read_text()
            require(not any(x in stderr.lower() for x in ['panicked at', 'resource limit', 'timed out']),
                    'resource/setup error is not a source boundary')
            data = json.loads(stdout)
            require_function_coverage(data.get('verus-infer', {}).get('functions', []), [])
            if args.adapter:
                positive = not args.wrong_adapter
                verification = data['verification-results']
                require((code == 0) is positive and verification.get('success') is positive
                        and verification['is-verifying-entire-crate'] is True,
                        'checked application adapter outcome')
                scope = data['verus-infer']['verification_scope']
                status = 'checked_exec_body' if positive else 'selected_body_not_certified_by_successful_full_run'
                actual = {b['rust_function'] for b in scope['native_functions'] if b['body_status'] == status}
                require(actual == set(freeze['expected_exec_bodies']), 'exact checked original/default bodies')
                require(scope['native_collection_completed'] is True, 'complete native inventory')
                if positive:
                    require(verification['verified'] == 15 and scope['checked_exec_bodies'] == 10
                            and scope['checked_proof_bodies'] == 0 and scope['complete_selected_scope_verified'],
                            'positive adapter counts')
                else:
                    require(verification['errors'] == 1 and not verification.get('encountered-vir-error'),
                            'isolated conversion failure')
                    require_failure_kind(stderr, 'postcondition not satisfied')
            result['outcomes'].append(dict(label=label, exit_code=code, data=data,
                                           stderr_sha256=digest(out / (label + '.stderr'))))
            print(label, code, data['verification-results'], flush=True)
        guard.check()
        result['guard_passed'] = True


if __name__ == '__main__':
    main()
