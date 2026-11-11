#!/usr/bin/env python3
"""Focused F1 replay. Source and tool bytes stay private; shared replay helpers only."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
TOOLS = HERE.parents[2]
sys.path.insert(0, str(TOOLS))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require

RUST = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')

def method(text, name):
    found = re.search(r'    (?:pub )?(?:const )?fn ' + re.escape(name) + r'\(', text)
    require(found is not None, 'method missing: ' + name)
    start = found.start()
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path)
    parser.add_argument('--scenario', choices=('frozen', 'full_plain_derive', 'absolute_fee_only', 'policy_subset'), default='frozen')
    args = parser.parse_args()
    out = args.output or Path(tempfile.mkdtemp(prefix='fedimint-f1-replay-'))
    out.mkdir(parents=True, exist_ok=True)
    require(not (out / 'receipt.json').exists(), 'use a fresh replay directory')
    receipt = dict(case='F1', commands=[], outcomes=[], goal_complete=False,
                   source_domain='all u64', new_engine_edits=False, scenario=args.scenario)
    with replay_receipt(out, receipt):
        selection = json.loads((HERE.parent / 'SELECTION.json').read_text())
        runtime = selection['runtime']
        bundle = out / 'bundle'
        bundle.mkdir()
        origin = Path(runtime['path'])
        require(digest(origin / 'receipt.json') == runtime['receipt_sha256'], 'runtime receipt hash')
        for name, expected in runtime['tools'].items():
            require(digest(origin / name) == expected, 'runtime hash: ' + name)
            shutil.copyfile(origin / name, bundle / name)
            (bundle / name).chmod(0o700 if name in ('verus', 'rust_verify', 'z3') or name.endswith('.so') else 0o600)
            require(digest(bundle / name) == expected, 'copied runtime hash: ' + name)
        (bundle / 'verus-root').touch()
        source = selection['sources']['fedimint']
        root = Path(source['root'])
        require(subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
                == source['commit'], 'source commit pin')
        for name, expected in source['files'].items():
            src = root / name
            require(src.stat().st_size == expected['bytes'] and digest(src) == expected['sha256'], 'source pin: ' + name)
            dst = out / 'source' / name
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dst)
        fixture = (HERE / 'component.rs').read_text()
        fidelity = json.loads((HERE / 'fidelity.json').read_text())
        for identity, expected in fidelity['method_sha256'].items():
            owner, name = identity.split('::')
            extracted = method(fixture.split('impl ' + owner + ' {', 1)[1], name)
            import hashlib
            require(hashlib.sha256(extracted.encode()).hexdigest() == expected, 'fixture body drift: ' + identity)
            src = out / 'source' / ('fedimint-core/src/amount.rs' if owner == 'Amount' else 'modules/fedimint-lnv2-common/src/gateway_api.rs')
            require(extracted == method(src.read_text(), name), 'source body mismatch: ' + identity)
        if args.scenario != 'frozen':
            fixture = fixture.replace('#[verus_verify(infer_derive(PartialEq, PartialOrd, Ord))]', '#[verus_verify]')
        if args.scenario in ('absolute_fee_only', 'policy_subset'):
            header = fixture.split('\n#[verus_verify]\nimpl Amount {', 1)[0]
            fixture = header
            for owner, names in [('Amount', ('from_msats', 'checked_add')),
                    ('PaymentFee', ('absolute_fee',) if args.scenario == 'absolute_fee_only' else ('is_within', 'checked_add'))]:
                fixture += '\n#[verus_verify]\nimpl ' + owner + ' {\n'
                for name in names:
                    original = (HERE / 'component.rs').read_text().split('impl ' + owner + ' {', 1)[1]
                    body = method(original, name)
                    start = original.index(body)
                    attr = original.rfind('    #[verus_spec(', 0, start)
                    require(attr >= 0, 'missing original method contract')
                    fixture += original[attr:start] + body + '\n'
                fixture += '}\n'
        (out / 'component.rs').write_text(fixture)
        receipt['fixture_sha256'] = digest(out / 'component.rs')
        receipt['runtime_tools'] = runtime['tools']
        receipt['source_pin'] = source['commit']
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + str(RUST / 'bin') + ':' + os.environ.get('PATH', ''), RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=str(RUST),
                   VERUS_Z3_PATH=str(bundle / 'z3'), VERUS_INFER_CACHE=str(out / 'cache'),
                   LD_LIBRARY_PATH=str(RUST / 'lib') + ':' + os.environ.get('LD_LIBRARY_PATH', ''))
        guard = ArtifactGuard([bundle / name for name in runtime['tools']] +
                              [HERE / 'component.rs', HERE / 'fidelity.json', TOOLS / 'infer_experiment.py', TOOLS / 'infer_eval_support.py'])
        run = LoggedCommands(out, out, receipt['commands'], timeout=120, guard=guard)
        for mode in ('ordinary', 'infer'):
            cmd = [str(bundle / 'verus'), str(out / 'component.rs'), '--crate-type=lib', '--edition=2024',
                   '--crate-name', 'f1_' + mode, '--no-cheating', '--output-json', '--time', '--compile',
                   '--export', str(out / (mode + '.vir')), '-o', str(out / ('lib' + mode + '.rlib'))]
            if mode == 'infer':
                cmd += ['--infer-goals']
            code = run(mode, cmd, env)
            stdout = (out / (mode + '.stdout')).read_text()
            stderr = (out / (mode + '.stderr')).read_text()
            try:
                data = json.loads(stdout)
            except json.JSONDecodeError:
                data = None
            receipt['outcomes'].append(dict(mode=mode, exit_code=code, data=data,
                diagnostic=stderr, published=(out / (mode + '.vir')).exists()))
            require(not any(x in stderr for x in ('panicked at', 'internal compiler error')), 'compiler crash')
        guard.check()
    print(out)

if __name__ == '__main__':
    main()
