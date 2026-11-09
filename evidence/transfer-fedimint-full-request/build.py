"""Build an isolated matching engine/macros/checked vstd bundle (Rust 1.99.1)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--solver', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
           CARGO_BUILD_JOBS='2', RUSTUP_TOOLCHAIN='1.99.1',
           CARGO_TARGET_DIR=str(out / 'target'), CARGO_BUILD_BUILD_DIR=str(out / 'build'),
           VERUS_Z3_PATH=str(args.solver.resolve()), RUST_MIN_STACK='20971520')
commands = []

def run(label, command):
    commands.append(dict(label=label, argv=list(map(str, command)), cwd=str(ROOT / 'source')))
    with (out / (label + '.stdout')).open('w') as stdout, (out / (label + '.stderr')).open('w') as stderr:
        result = subprocess.run(list(map(str, command)), env=env, cwd=ROOT / 'source',
                                stdout=stdout, stderr=stderr, timeout=2400)
    commands[-1]['returncode'] = result.returncode
    (out / 'build-commands.json').write_text(json.dumps(commands, indent=2) + '\n')
    result.check_returncode()

run('engine', ['cargo', 'build', '--offline', '--locked', '--release',
    '-p', 'rust_verify', '-p', 'verus', '-p', 'vstd_build', '-p', 'verus_builtin',
    '-p', 'verus_builtin_macros', '-p', 'verus_state_machines_macros'])
bundle = out / 'target-verus/release'
bundle.mkdir(parents=True, exist_ok=True)
for name in ['rust_verify', 'verus', 'vstd_build', 'libverus_builtin.rlib',
             'libverus_builtin_macros.so', 'libverus_state_machines_macros.so']:
    shutil.copy2(out / 'target/release' / name, bundle / name)
(bundle / 'verus-root').touch()
env['VERUS_IN_VARGO'] = '1'
# Direct vstd_build needs rustc_driver discoverable independently of the wrapper.
sysroot = subprocess.check_output(['rustc', '--print', 'sysroot'], env=env, text=True).strip()
env['LD_LIBRARY_PATH'] = str(Path(sysroot) / 'lib') + ':' + env.get('LD_LIBRARY_PATH', '')
run('vstd', [bundle / 'vstd_build', bundle, '--release'])
# InferCrates and the older verification test helper locate artifacts differently.
# Both locations are private independent copies with the same hashes.
for name in ['vstd.vir', 'libvstd.rlib']:
    shutil.copy2(bundle / name, out / 'target/release' / name)
# Cargo's separate build-dir owns integration test executables and their
# InferCrates helper resolves verifier binaries relative to those executables.
for path in bundle.iterdir():
    if path.is_file():
        shutil.copy2(path, out / 'build/release' / path.name)
hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in bundle.iterdir() if p.is_file()}
(out / 'bundle-sha256.json').write_text(json.dumps(hashes, indent=2) + '\n')
print(bundle)
