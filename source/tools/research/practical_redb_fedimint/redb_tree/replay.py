#!/usr/bin/env python3
"""Focused R2 replay using the existing execution/evidence helpers; no build/network."""
import hashlib
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
    meta = json.loads((HERE / 'private.json').read_text())
    out = Path(sys.argv[1]).resolve()
    require(str(out).startswith('/tmp/redb-r2-'), 'private R2 output required')
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    private = Path(meta['private_artifacts'])
    runtime = private / 'runtime'
    pins = {**meta['runtime']['tools'], 'receipt.json': meta['runtime']['receipt_sha256']}
    for name, sha in pins.items():
        require(digest(runtime / name) == sha, 'runtime pin: ' + name)
    for name, item in meta['source']['files'].items():
        p = private / 'source' / Path(name).name
        require(digest(p) == item['sha256'] and p.stat().st_size == item['bytes'], 'source pin: ' + name)
    original = (private / 'source/bitmap.rs').read_text()
    fixture = HERE / ('component.rs' if len(sys.argv) == 2 else sys.argv[2])
    frozen = fixture.read_text()
    if fixture.name == 'component.rs':
        require(digest(fixture) == meta['fixture_sha256'], 'freeze changed')
    for name, item in meta['items'].items():
        extracted = original[item['start_byte']:item['end_byte']]
        require(hashlib.sha256(extracted.encode()).hexdigest() == item['sha256'], 'extraction: ' + name)
        if not name[0].isupper():
            body = extracted[extracted.index('{'):]
            # The manual variant may insert only proof annotations; validate its
            # unchanged exec projection separately before enabling it here.
            if fixture.name == 'component.rs':
                require(body in frozen, 'body changed: ' + name)
    for token in ('assume(', 'admit(', 'external_body', 'assume_specification', 'external_fn_specification', 'verify=false'):
        require(token not in frozen, 'forbidden trust: ' + token)
    target = out / fixture.name
    shutil.copyfile(fixture, target)
    env = dict(os.environ, RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1',
               PATH='/home/dev/.cargo/bin:/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin:' + os.environ['PATH'],
               VERUS_Z3_PATH=str(runtime / 'z3'), VERUS_INFER_CACHE=str(out / 'cache'))
    tools = [runtime / n for n in pins]
    guard = ArtifactGuard([*tools, target, HERE / 'component.rs', HERE / 'private.json',
                           *[Path(__file__).resolve().parents[2] / n for n in ()]])
    receipt = dict(case='R2', application_property_proved=False, fixture_sha256=digest(target),
                   exec_inventory=['BtreeBitmap::get_level','BtreeBitmap::get_height','BtreeBitmap::set',
                                   'BtreeBitmap::clear','BtreeBitmap::get_level_mut','BtreeBitmap::update_to_root',
                                   'U64GroupedBitmap::data_index_of','U64GroupedBitmap::select_mask',
                                   'U64GroupedBitmap::len','U64GroupedBitmap::get','U64GroupedBitmap::set',
                                   'U64GroupedBitmap::clear','main'], proof_inventory=[], commands=[], outcomes=[])
    with replay_receipt(out, receipt):
        run = LoggedCommands(out, out, receipt['commands'], timeout=120, guard=guard)
        for mode, flags in [('ordinary', []), ('infer', ['--infer-goals'])]:
            code = run(mode, [str(runtime / 'verus'), str(target), '--edition=2024', '--no-cheating',
                              '--output-json', '--multiple-errors', '20', *flags], env)
            stdout = out / (mode + '.stdout'); stderr = out / (mode + '.stderr')
            data = json.loads(stdout.read_text()); error = stderr.read_text()
            verification = data['verification-results']
            helpers = data.get('verus-infer', {}).get('functions', [])
            require_exact(helpers)
            require(not any(t in error.lower() for t in ('panicked at', 'timed out')), 'internal/timeout outcome')
            frontend = verification.get('encountered-vir-error') is True or ('success' not in verification and verification.get('encountered-error') is True)
            outcome = 'FRONTEND_REJECTED' if frontend else ('VERIFIED' if code == 0 and verification.get('success') is True else 'VERIFICATION_FAILED')
            require((code == 0) == (outcome == 'VERIFIED'), 'inconsistent compiler status')
            receipt['outcomes'].append(dict(mode=mode, outcome=outcome, verification=verification,
                                            inferred_helpers=helpers, scope=data.get('verus-infer', {}).get('verification_scope'),
                                            stdout_sha256=digest(stdout), stderr_sha256=digest(stderr),
                                            resource_failure='resource limit' in error.lower()))
        guard.check()
    print(out)


if __name__ == '__main__':
    main()
