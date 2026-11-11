#!/usr/bin/env python3
"""R1 accepted-runtime baseline; raw compiler evidence stays private in /tmp."""
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from infer_experiment import LoggedCommands, replay_receipt, require, require_exact
from infer_eval_support import ArtifactGuard, digest


def main():
    freeze = json.loads((HERE / 'freeze.json').read_text())
    out = Path(sys.argv[1]).resolve()
    require(str(out).startswith('/tmp/redb-r1-'), 'use own private R1 output')
    out.mkdir(mode=0o700, parents=True, exist_ok=True)
    runtime = Path(freeze['private_artifacts']) / 'runtime'
    expected = freeze['runtime']
    require(digest(runtime / 'receipt.json') == expected['receipt_sha256'], 'receipt mismatch')
    for name, sha in expected['tools'].items():
        require(digest(runtime / name) == sha, 'runtime mismatch: ' + name)
    require(digest(HERE / 'component.rs') == freeze['fixture_sha256'], 'frozen fixture changed')
    source = Path(freeze['private_artifacts']) / 'source'
    for name, item in freeze['source_files'].items():
        path = source / Path(name).name
        require(digest(path) == item['sha256'] and path.stat().st_size == item['bytes'], 'source copy changed')
    original = (source / 'bitmap.rs').read_text()
    fixture = (HERE / 'component.rs').read_text()
    for name, start in [('struct', 'pub(crate) struct U64GroupedBitmap'),
                        ('bits_in_range', 'fn bits_in_range('),
                        ('required_words', 'fn required_words('), ('resize', 'pub fn resize(')]:
        p = original.index(start)
        b = original.index('{', p)
        end, depth = b + 1, 1
        while depth:
            depth += (original[end] == '{') - (original[end] == '}')
            end += 1
        import hashlib
        require(hashlib.sha256(original[p:end].encode()).hexdigest() == freeze['items'][name]['sha256'], 'extraction mismatch')
        require(original[b:end] in fixture, 'executable body changed: ' + name)
    require(original[original.index('pub(crate) struct U64GroupedBitmap'):original.index('impl U64GroupedBitmap')].strip() in fixture, 'representation changed')
    for token in ('assume(', 'admit(', 'external_body', 'assume_specification', 'external_fn_specification', 'verify=false'):
        require(token not in fixture, 'forbidden trust: ' + token)
    frozen = out / 'component.rs'
    shutil.copyfile(HERE / 'component.rs', frozen)
    env = dict(os.environ, RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1',
               PATH='/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin:' + os.environ['PATH'],
               VERUS_Z3_PATH=str(runtime / 'z3'), VERUS_INFER_CACHE=str(out / 'infer-cache'))
    guard = ArtifactGuard([runtime / n for n in expected['tools']] + [runtime / 'receipt.json', frozen])
    receipt = dict(case='R1', application_property_proved=False, commands=[], outcomes=[],
                   fixture_sha256=digest(frozen), exec_inventory=['bits_in_range', 'required_words', 'resize', 'main'],
                   proof_inventory=[], spec_inventory=['words', 'physical_bit'], inferred_helpers=[])
    with replay_receipt(out, receipt):
        run = LoggedCommands(out, out, receipt['commands'], timeout=120, guard=guard)
        for mode, flags in [('ordinary', []), ('infer', ['--infer-goals'])]:
            code = run(mode, [str(runtime / 'verus'), str(frozen), '--edition=2024', '--no-cheating',
                              '--output-json', '--multiple-errors', '20', *flags], env)
            stdout = (out / (mode + '.stdout')).read_text()
            stderr = (out / (mode + '.stderr')).read_text()
            require(not any(t in stderr.lower() for t in ('panicked at', 'resource limit', 'timed out')), 'resource/internal outcome')
            data = json.loads(stdout)
            verification = data['verification-results']
            # A frontend rejection remains an unsupported boundary, never proof failure/success evidence.
            frontend = verification.get('encountered-vir-error') is True or ('success' not in verification and verification.get('encountered-error') is True)
            outcome = 'FRONTEND_UNSUPPORTED' if frontend else ('VERIFIED' if code == 0 and verification.get('success') is True else 'VERIFICATION_FAILED')
            helpers = data.get('verus-infer', {}).get('functions', [])
            require_exact(helpers)
            receipt['outcomes'].append(dict(mode=mode, outcome=outcome, verification=verification,
                                            inferred_helpers=helpers, scope=data.get('verus-infer', {}).get('verification_scope'),
                                            stderr_sha256=digest(out / (mode + '.stderr'))))
        guard.check()
    print(out)


if __name__ == '__main__':
    main()
