#!/usr/bin/env python3
"""Bounded native Rust type context only; no verifier or crypto calls."""
import argparse
import json
import os
from pathlib import Path
import sys
import tarfile
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_experiment import LoggedCommands, replay_receipt, require
from infer_eval_support import ArtifactGuard, digest


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--freeze', type=Path, default=HERE / 'ELIGIBILITY_FREEZE.json')
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    r = {'kind': 'NATIVE_UNUSED_CRYPTO_TYPE_CONTEXT_NOT_PROOF', 'commands': [], 'crypto_proved': False}
    with replay_receipt(a.output, r):
        freeze = json.loads(a.freeze.read_text())
        for name, expected in freeze['inputs'].items():
            path = Path(name) if Path(name).is_absolute() else ROOT / name
            require(digest(path) == expected, 'frozen input changed: ' + name)
        lock = tomllib.loads(Path(freeze['lock']).read_text())
        cache = Path('/home/dev/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f')
        wanted = [('arrayvec', '0.7.6'), ('hex-conservative', '0.2.2'), ('bitcoin_hashes', '0.14.0'), ('secp256k1-sys', '0.10.1'), ('secp256k1', '0.29.1')]
        packages = {}
        pins = {}
        for name, version in wanted:
            record = next(x for x in lock['package'] if x['name'] == name and x['version'] == version)
            archive = cache / (name + '-' + version + '.crate')
            require(digest(archive) == record['checksum'], 'archive does not match project lock')
            pins[str(archive)] = record['checksum']
            with tarfile.open(archive) as tar:
                for member in tar.getmembers():
                    if not member.isfile():
                        continue
                    rel = Path(member.name)
                    require(not rel.is_absolute() and '..' not in rel.parts, 'invalid archive path')
                    dest = a.output / 'sources' / rel
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    dest.write_bytes(tar.extractfile(member).read())
            packages[name] = a.output / 'sources' / (name + '-' + version)
        toolchain = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
        rustc = toolchain / 'bin/rustc'
        source_files = [x for d in packages.values() for x in d.rglob('*') if x.is_file()]
        guard = ArtifactGuard([rustc, HERE / 'prepare_types.py', a.freeze] + [Path(x) for x in pins] + source_files)
        r['inputs'] = {str(x): digest(x) for x in [rustc] + source_files}
        r['archives'] = pins
        libs = a.output / 'libs'
        libs.mkdir()
        env = dict(os.environ, PATH=str(toolchain / 'bin') + ':/usr/bin:/bin', RUSTUP_TOOLCHAIN=str(toolchain), LD_LIBRARY_PATH=str(toolchain / 'lib'))
        run = LoggedCommands(a.output, a.output, r['commands'], timeout=180, guard=guard)
        deps = {'arrayvec': {}, 'hex-conservative': {'arrayvec': 'arrayvec'}, 'bitcoin_hashes': {'hex': 'hex-conservative'}, 'secp256k1-sys': {}, 'secp256k1': {'secp256k1_sys': 'secp256k1-sys', 'hashes': 'bitcoin_hashes'}}
        artifacts = {}
        for name, version in wanted:
            folder = packages[name]
            meta = tomllib.loads((folder / 'Cargo.toml').read_text())
            crate = name.replace('-', '_')
            output = libs / ('lib' + crate + '.rlib')
            cmd = [str(rustc), '--crate-name', crate, '--crate-type=rlib', '--edition=' + meta['package'].get('edition', '2015'), str(folder / 'src/lib.rs'), '-L', 'dependency=' + str(libs), '-o', str(output)]
            features = ([] if name == 'arrayvec' else ['alloc']) + (['hashes'] if name == 'secp256k1' else [])
            require(all(feature in meta.get('features', {}) or
                        meta.get('dependencies', {}).get(feature, {}).get('optional') is True
                        for feature in features), 'undeclared feature')
            for feature in features:
                cmd += ['--cfg', 'feature="' + feature + '"']
            for alias, dependency in deps[name].items():
                cmd += ['--extern', alias + '=' + str(artifacts[dependency])]
            package_env = dict(env, CARGO_MANIFEST_DIR=str(folder), CARGO_PKG_NAME=name, CARGO_PKG_VERSION=version)
            code = run(crate, cmd, package_env)
            require(code == 0 and output.is_file(), 'native type-context build failed: ' + name)
            artifacts[name] = output
        guard.check()
        r['artifacts'] = {str(x): digest(x) for x in artifacts.values()}
        r['native_context_only'] = True
        print(json.dumps({'output': str(a.output), 'rust_crates': len(artifacts), 'proofs': 0}))


if __name__ == '__main__':
    main()
