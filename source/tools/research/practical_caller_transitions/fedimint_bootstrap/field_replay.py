#!/usr/bin/env python3
"""Two-mode field/type diagnostic, never full bootstrap acceptance."""
import argparse
import json
import os
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage
from infer_eval_support import ArtifactGuard, digest


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--context', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--source', type=Path, default=HERE / 'field-eligibility.rs')
    p.add_argument('--freeze', type=Path, default=HERE / 'FIELD_FREEZE.json')
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    r = {'kind': 'F3C_GENUINE_FIELD_ELIGIBILITY_NOT_CALLER_PROOF', 'commands': [], 'outcomes': [], 'bootstrap_proved': False}
    with replay_receipt(a.output, r):
        freeze = json.loads(a.freeze.read_text())
        for name, expected in freeze['inputs'].items():
            path = Path(name) if Path(name).is_absolute() else ROOT / name
            require(digest(path) == expected, 'frozen input changed: ' + name)
        context_receipt = a.context / 'receipt.json'
        require(str(context_receipt) in freeze['inputs'] and
                digest(context_receipt) == freeze['inputs'][str(context_receipt)], 'native context is not registered in this freeze')
        context = json.loads(context_receipt.read_text())
        require(context['success'] is True and context['native_context_only'] is True and context['crypto_proved'] is False, 'wrong native context')
        libs = [Path(x) for x in context['artifacts']]
        for path in libs:
            require(digest(path) == context['artifacts'][str(path)], 'native context artifact changed')
        bundle = Path('/tmp/practical-f3-parent-47a9643d/auth/bundle')
        manifest = json.loads((bundle / 'receipt.json').read_text())
        require(digest(bundle / 'receipt.json') == '3e04cec7545fd1b44781ba6e8c2447e2109339a9f5257ef7b605d81896a16452', 'accepted runtime identity')
        for name, expected in manifest['tools'].items():
            require(digest(bundle / name) == expected, 'runtime tool changed')
        source = a.source.resolve()
        inputs = libs + [a.context / 'receipt.json', a.freeze, source, Path(__file__)] + [bundle / n for n in manifest['tools']]
        guard = ArtifactGuard(inputs)
        r['inputs'] = {str(x): digest(x) for x in inputs}
        r['manual_nominal_type_declarations'] = source.read_text().count('#[verifier::external_type_specification]')
        run = LoggedCommands(a.output, a.output, r['commands'], timeout=180, guard=guard)
        tc = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + str(tc / 'bin') + ':/usr/bin:/bin', RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=str(tc), LD_LIBRARY_PATH=str(tc / 'lib'), VERUS_Z3_PATH=str(bundle / 'z3'), VERUS_INFER_CACHE=str(a.output / 'cache'))
        for mode in ['ordinary', 'infer']:
            cmd = [str(bundle / 'verus'), str(source), '--edition=2024', '--crate-type=lib', '--crate-name', 'f3c_fields', '--no-cheating', '--output-json', '--time', '--extern', 'secp256k1=' + str(a.context / 'libs/libsecp256k1.rlib'), '-L', 'dependency=' + str(a.context / 'libs')]
            if mode == 'infer':
                cmd += ['--infer-goals']
            code = run(mode, cmd, env)
            stdout = (a.output / (mode + '.stdout')).read_text()
            stderr = (a.output / (mode + '.stderr')).read_text()
            data = json.loads(stdout) if stdout.strip() else {}
            verification = data.get('verification-results', {})
            if code == 0:
                validate_run({'returncode': code, 'stdout': stdout, 'stderr': stderr}, True)
                inf = data['verus-infer']
                require_exact(inf['functions'])
                require_function_coverage(inf['functions'], [])
                scope = inf['verification_scope']
                require(verification['is-verifying-entire-crate'] is True and scope['native_collection_completed'] is True and scope['complete_selected_scope_verified'] is True, 'incomplete field diagnostic')
                checked = [x for x in scope['native_functions'] if x['body_status'] == 'checked_exec_body']
                require_function_coverage(checked, ['configured_length'])
                require(all(x['selected'] is True and x['rust_function'] == 'Keychain::configured_length' and x['source'].startswith(str(source) + ':') for x in checked), 'field diagnostic identity/source mismatch')
                require(scope['checked_exec_bodies'] == 1 and scope['checked_proof_bodies'] == 0, 'wrong field diagnostic inventory')
            r['outcomes'].append({'mode': mode, 'returncode': code, 'verification': verification, 'scope': data.get('verus-infer', {}).get('verification_scope'), 'stderr_sha256': digest(a.output / (mode + '.stderr'))})
            print(mode, verification, flush=True)
        guard.check()
        r['field_eligibility_passes_both_modes'] = all(x['returncode'] == 0 for x in r['outcomes'])


if __name__ == '__main__':
    main()
