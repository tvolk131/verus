#!/usr/bin/env python3
"""One private, offline native active-graph preparation; no verifier claims."""
import argparse
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require


def unpack(data, destination, strip=0):
    destination.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for member in archive.getmembers():
            parts = Path(member.name).parts[strip:]
            require(parts and '..' not in parts and not Path(*parts).is_absolute(), 'archive path')
            target = destination.joinpath(*parts)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.extractfile(member).read())
            else:
                require(False, 'unsupported archive member')


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--freeze', type=Path, default=HERE / 'FREEZE.json')
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    registration = json.loads(a.freeze.read_text())
    receipt = {'kind': 'NATIVE_PREPARATION_NOT_CRASH_PROOF', 'commands': []}
    with replay_receipt(out, receipt):
        for name, value in registration['inputs'].items():
            require(digest(Path(name)) == value, 'registered input ' + name)
        app = Path(registration['redb_checkout'])
        tc = Path(registration['toolchain'])
        require(subprocess.check_output(['git', '-C', str(app), 'rev-parse', 'HEAD'], text=True).strip() == registration['redb_commit'], 'source commit')
        code = subprocess.check_output(['git', '-C', str(app), 'archive', registration['redb_commit'], 'src', 'build.rs', 'Cargo.toml'])
        work = out / 'workspace'
        unpack(code, work / 'redb')
        unpack(Path(registration['libc_archive']).read_bytes(), work / 'libc', 1)
        for package in ['redb', 'libc']:
            path = work / package / 'Cargo.toml'
            path.rename(path.with_name('Cargo.toml.original'))
        redb_manifest = tomllib.loads((work / 'redb/Cargo.toml.original').read_text())
        libc_manifest = tomllib.loads((work / 'libc/Cargo.toml.original').read_text())
        require(redb_manifest['package']['name'] == 'redb' and redb_manifest['package']['version'] == '4.4.0', 'original package')
        require(redb_manifest['workspace']['package']['edition'] == '2024' and redb_manifest['workspace']['package']['rust-version'] == '1.90', 'original toolchain metadata')
        require(libc_manifest['package']['version'] == '0.2.191' and libc_manifest['features']['default'] == ['std'], 'published libc graph')
        (work / 'Cargo.toml').write_text('[workspace]\nmembers=["redb","libc","harness"]\nresolver="3"\n')
        (work / 'redb/Cargo.toml').write_text('''[package]
name="redb"
version="4.4.0"
edition="2024"
rust-version="1.90"
build="build.rs"
[dependencies]
libc={path="../libc",version="=0.2.191"}
[features]
default=["std"]
std=[]
experimental-api-5=[]
experimental-niches=["experimental-api-5"]
cache_metrics=[]
logging=[]
chrono_v0_4=[]
uuid=[]
''')
        (work / 'libc/Cargo.toml').write_text('''[package]
name="libc"
version="0.2.191"
edition="2021"
rust-version="1.65"
build="build.rs"
[lib]
name="libc"
path="src/lib.rs"
[features]
default=["std"]
std=[]
align=[]
const-extern-fn=[]
extra_traits=[]
use_std=["std"]
''')
        (work / 'harness/src').mkdir(parents=True)
        (work / 'harness/Cargo.toml').write_text('[package]\nname="redb-crash-native"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nredb={path="../redb"}\n')
        shutil.copy2(HERE / 'baseline.rs', work / 'harness/src/main.rs')
        generated = [x for x in work.rglob('*') if x.is_file()]
        receipt['prepared_inputs'] = {str(x): digest(x) for x in generated}
        receipt['manifest_projection'] = 'Only active std/Linux graph, no optional/dev/workspace extras; original Rust source and build scripts unmodified; not a feature-complete upstream build.'
        guard = ArtifactGuard([Path(x) for x in registration['inputs']] + generated)
        env = {k: v for k, v in os.environ.items() if not k.startswith('CARGO_') and k not in ['RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTC', 'RUSTDOC']}
        env.update(PATH=str(tc / 'bin') + ':/usr/bin:/bin', RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=str(tc), RUSTC=str(tc / 'bin/rustc'), LD_LIBRARY_PATH=str(tc / 'lib'), CARGO_HOME=str(out / 'cargo-home'), CARGO_TARGET_DIR=str(out / 'target'), CARGO_BUILD_JOBS='2', CARGO_NET_OFFLINE='true')
        receipt['environment'] = {k: env[k] for k in ['PATH', 'RUSTUP_TOOLCHAIN', 'RUSTC', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'CARGO_NET_OFFLINE']}
        run = LoggedCommands(out, work, receipt['commands'], timeout=600, guard=guard)
        cargo = [str(tc / 'bin/cargo'), '--config', 'build.build-dir="' + str(out / 'build') + '"']
        require(run('lock', cargo + ['generate-lockfile', '--offline'], env) == 0, 'private lock generation')
        receipt['private_lock_sha256'] = digest(work / 'Cargo.lock')
        (out / 'BUILD_FREEZE.json').write_text(json.dumps({'prepared_inputs': receipt['prepared_inputs'], 'private_lock_sha256': receipt['private_lock_sha256']}, indent=2) + '\n')
        guard.check()
        guard = ArtifactGuard([Path(x) for x in guard.paths] + [work / 'Cargo.lock', out / 'BUILD_FREEZE.json'])
        run.guard = guard
        require(run('build', cargo + ['build', '--offline', '--locked', '--release', '-p', 'redb-crash-native'], env) == 0, 'bounded native compilation')
        binary = out / 'target/release/redb-crash-native'
        require(binary.is_file(), 'native executable')
        receipt['binary'] = str(binary)
        receipt['binary_sha256'] = digest(binary)
        receipt['formal_source_proof'] = False


if __name__ == '__main__':
    main()
