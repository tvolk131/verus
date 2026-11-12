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
from infer_experiment import LoggedCommands, replay_receipt, require, require_function_coverage

NOMINAL = '''
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIoError(std::io::Error);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPanicLocation<'a>(core::panic::Location<'a>);
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
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    result = dict(kind='ACTUAL_BACKEND_NOMINAL_ONLY_CONTINUATION', commands=[], outcomes=[],
                  source_coupling_proved=False)
    with replay_receipt(out, result):
        freeze = json.loads((HERE / 'FROZEN.json').read_text())
        for path, value in freeze['inputs'].items():
            require(digest(Path(path)) == value, 'frozen input ' + path)
        source = out / 'backend_nominal.rs'
        source.write_text(fixture(out))
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
        env = dict(os.environ, PATH=str(tc / 'bin') + ':/usr/bin:/bin',
                   RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=str(tc),
                   LD_LIBRARY_PATH=str(tc / 'lib'), VERUS_Z3_PATH=str(bundle / 'z3'))
        guard = ArtifactGuard([Path(p) for p in freeze['inputs']] + [HERE / 'FROZEN.json', source] +
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
            result['outcomes'].append(dict(label=label, exit_code=code, data=data,
                                           stderr_sha256=digest(out / (label + '.stderr'))))
            print(label, code, data['verification-results'], flush=True)
        guard.check()
        result['guard_passed'] = True


if __name__ == '__main__':
    main()
