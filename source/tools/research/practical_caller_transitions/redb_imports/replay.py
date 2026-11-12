#!/usr/bin/env python3
"""Additional R3 import validation using the existing replay utilities."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
TOOLS = HERE.parents[2]
sys.path.insert(0, str(TOOLS))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import (LoggedCommands, replay_receipt, require, validate_run,
                              require_exact, require_function_coverage, require_publication,
                              require_failure_kind)


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    freeze = json.loads((HERE / 'FREEZE.json').read_text())
    root = HERE.parents[4]
    redb = HERE.parent / 'redb'
    scope = load(HERE.parents[1] / 'practical_redb_fedimint/common_set/replay_s1.py', 'r3_import_scope')
    receipt = {'kind': 'R3_ADDITIONAL_IMPORTED_CALLER_NOT_INDEPENDENT_ACCEPTANCE', 'commands': [], 'outcomes': []}
    with replay_receipt(out, receipt):
        frozen = []
        for name, expected in freeze['inputs'].items():
            path = root / name
            require(digest(path) == expected, 'frozen source ' + name)
            frozen.append(path)
        bundle = Path(freeze['runtime'])
        require(digest(bundle / 'receipt.json') == freeze['runtime_receipt'], 'runtime receipt')
        for name, expected in freeze['runtime_tools'].items():
            require(digest(bundle / name) == expected, 'runtime ' + name)
        text = '\n'.join((redb / n).read_text() for n in freeze['provider_parts'])
        original = text
        for old, new in freeze['visibility_changes']:
            require(text.count(old) == 1, 'unique visibility change ' + old)
            text = text.replace(old, new)
        reverse = text
        for old, new in reversed(freeze['visibility_changes']):
            require(reverse.count(new) == 1, 'unique inverse visibility change ' + new)
            reverse = reverse.replace(new, old)
        require(reverse == original, 'provider changed beyond registered visibility')
        source = out / 'provider.rs'
        source.write_text(text)
        client = out / 'client.rs'
        shutil.copy2(HERE / 'client.rs', client)
        receipt['provider_original_sha256'] = __import__('hashlib').sha256(original.encode()).hexdigest()
        receipt['provider_api_sha256'] = digest(source)
        receipt['client_sha256'] = digest(client)
        inventory = json.loads((redb / 'RUN_FREEZE_V2.json').read_text())
        guard = ArtifactGuard(frozen + [HERE / 'FREEZE.json', source, client, bundle / 'receipt.json'] +
                              [bundle / n for n in freeze['runtime_tools']])
        run = LoggedCommands(out, out, receipt['commands'], timeout=180, guard=guard)
        base_env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ.get('PATH', ''),
                        RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu', VERUS_Z3_PATH=str(bundle / 'z3'))

        def check(label, src, goals, extra, executable, proofs, failures=(), seed=None):
            artifact = out / (label + '.vir')
            if seed:
                shutil.copy2(seed, artifact)
            cmd = [str(bundle / 'verus'), str(src), '--crate-type=lib', '--edition=2024',
                   '--crate-name', 'r3_public' if src == source else 'r3_client',
                   '--no-cheating', '--output-json', '--time', '--multiple-errors', '20',
                   '--export', str(artifact)] + (['--infer-goals'] if goals else []) + extra
            code = run(label, cmd, dict(base_env, VERUS_INFER_CACHE=str(out / (label + '_cache'))))
            raw = {'returncode': code, 'stdout': (out / (label + '.stdout')).read_text(),
                   'stderr': (out / (label + '.stderr')).read_text()}
            data = validate_run(raw, not failures)
            require(data['verification-results'].get('is-verifying-entire-crate') is True, 'full crate checking')
            inf = data['verus-infer']
            require_exact(inf['functions'])
            require_function_coverage(inf['functions'], [])
            native = {x['function']: x['rust_function'] for x in inf['verification_scope']['native_functions']}
            pairs = {native[x['function']]: x['pairs'] for x in inf['set_pair_assistance']}
            scope.require_native_scope(data, src, executable, proofs, list(failures), pairs)
            require([native[x] for x in inf['failed_functions']] == list(failures), 'isolated failing body')
            if goals:
                require_publication(data, not failures, artifact)
            else:
                require(code == 0 and artifact.is_file(), 'ordinary successful export')
            if failures:
                diagnostic = 'precondition not satisfied' if failures == ('unchecked_allocator',) else 'postcondition not satisfied'
                require_failure_kind(raw['stderr'], diagnostic)
                require('error[E' not in raw['stderr'], 'control frontend failure')
            item = {'label': label, 'verification': data['verification-results'], 'failed_functions': list(failures),
                    'scope': inf['verification_scope'], 'inferred_helpers': inf['functions'],
                    'set_pair_assistance': inf['set_pair_assistance'], 'times_ms': data.get('times-ms'),
                    'stdout_sha256': digest(out / (label + '.stdout')), 'stderr_sha256': digest(out / (label + '.stderr'))}
            receipt['outcomes'].append(item)
            print(label, data['verification-results'], flush=True)
            return artifact

        for provider_goals in [False, True]:
            label = 'provider_' + ('infer' if provider_goals else 'ordinary')
            rlib = out / ('lib' + label + '.rlib')
            provider = check(label, source, provider_goals,
                             ['--cfg', 'full_semantics', '--compile', '-o', str(rlib)],
                             inventory['v1_exec'] + ['record_alloc_semantic', 'free_semantic', 'semantic_roundtrip'],
                             inventory['v1_proof'] + inventory['v2_proof'])
            require(rlib.is_file(), 'compiled provider')
            guard.check()
            guard = ArtifactGuard([Path(n) for n in guard.paths] + [provider, rlib])
            run.guard = guard
            imports = ['--extern', 'r3_public=' + str(rlib), '--import', 'r3_public=' + str(provider)]
            for goals in [False, True]:
                good = check(label + '_client_' + str(goals), client, goals, imports, ['repeated_allocation'], [])
            for cfg, bad in [('negative_value', 'wrong_simultaneous_success'), ('negative_safety', 'unchecked_allocator')]:
                check(label + '_' + cfg, client, True, imports + ['--cfg', cfg],
                      ['repeated_allocation', bad], [], (bad,), seed=good)
        guard.check()
        receipt['original_proof_unchanged'] = True
        receipt['infer_annotation_reduction'] = 0
        receipt['independent_acceptance'] = False
        receipt['hmac_native'] = 'DEFERRED AND UNACHIEVED'


if __name__ == '__main__':
    main()
