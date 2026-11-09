#!/usr/bin/env python3
"""Compile pinned whole dependency and compare real macro-erased source fixtures."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import tarfile
import tempfile
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--archive', type=Path, required=True)
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(exist_ok=True, parents=True)
    out = Path(tempfile.mkdtemp(prefix='native-', dir=a.output.resolve()))
    r = {'commands': [], 'goal_complete': False}
    with replay_receipt(out, r):
        frozen = json.loads((HERE / 'freeze.json').read_text())
        require(digest(a.archive) == frozen['provenance']['archive_sha256'], 'wrong package archive')
        fixtures = [HERE / f for f in ['native.rs', 'native.Cargo.toml', 'native.Cargo.lock',
            'baseline.rs', 'full-manual.rs', 'upstream/src/hmac.rs', 'freeze.json']]
        guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(),
            extra=[a.archive, Path(__file__), *fixtures]))
        r['inputs'] = guard.initial
        extracted = out / 'upstream-crate'
        extracted.mkdir()
        with tarfile.open(a.archive) as archive:
            archive.extractall(extracted, filter='data')
        package = extracted / 'bitcoin_hashes-0.14.0'
        require((package / 'src/hmac.rs').read_bytes() == (HERE / 'upstream/src/hmac.rs').read_bytes(),
                'archive source mismatch')
        r['package_source_hashes'] = {str(f.relative_to(package)): digest(f)
            for f in sorted(package.rglob('*')) if f.is_file()}
        project = out / 'native'
        (project / 'src').mkdir(parents=True)
        shutil.copyfile(HERE / 'native.Cargo.toml', project / 'Cargo.toml')
        shutil.copyfile(HERE / 'native.Cargo.lock', project / 'Cargo.lock')
        (project / 'src/main.rs').write_text('fn main() {}\n')
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
            CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR=str(out / 'target'),
            CARGO_BUILD_BUILD_DIR=str(out / 'build'), RUSTFLAGS='')
        r['environment_overrides'] = {k: env[k] for k in ['CARGO_BUILD_JOBS',
            'CARGO_TARGET_DIR', 'CARGO_BUILD_BUILD_DIR', 'RUSTFLAGS']}
        call = LoggedCommands(out, ROOT / 'source', r['commands'], timeout=180, guard=guard)
        require(call('rustc-version', ['rustc', '+1.99.1', '--version', '--verbose'], env) == 0,
                'rustc identity unavailable')
        native = out / 'native.rs'
        shutil.copyfile(HERE / 'native.rs', native)
        shutil.copyfile(HERE / 'baseline.rs', out / 'baseline-adapter.rs')
        shutil.copyfile(HERE / 'full-manual.rs', out / 'manual-adapter.rs')
        (out / 'mutant-adapter.rs').write_text((HERE / 'baseline.rs').read_text().replace('*b_i ^= *b_h;', '*b_i |= *b_h;'))
        for profile, flags in [('debug', []), ('release', ['--release'])]:
            require(call('cargo-' + profile, ['cargo', '+1.99.1', 'build', '--locked', '--offline',
                '--manifest-path', str(project / 'Cargo.toml'), *flags], env) == 0,
                'whole dependency build failed')
            libraries = list((out / 'build' / profile / 'deps').glob('libbitcoin_hashes-*.rlib'))
            require(len(libraries) == 1, 'ambiguous dependency artifact')
            binary = out / ('compare-' + profile)
            cmd = ['rustc', '+1.99.1', '--edition=2021', str(native),
                '--extern', 'vstd=' + str(a.verus.resolve().parent / 'libvstd.rlib'),
                '--extern', 'bitcoin_hashes=' + str(libraries[0]),
                '-L', 'dependency=' + str(a.verus.resolve().parent),
                '-L', 'dependency=' + str(libraries[0].parent), '-o', str(binary)]
            if profile == 'release':
                cmd += ['-O']
            require(call('rustc-' + profile, cmd, env) == 0, 'actual adapter compilation failed')
            require(call('compare-' + profile, [str(binary)], env) == 0, 'native comparisons failed')
            r[profile] = (out / ('compare-' + profile + '.stdout')).read_text()
        require(digest(project / 'Cargo.lock') == digest(HERE / 'native.Cargo.lock'), 'native resolution changed')
        require(r['package_source_hashes'] == {str(f.relative_to(package)): digest(f)
            for f in sorted(package.rglob('*')) if f.is_file()}, 'package source changed')
        guard.check()
    print(out / 'receipt.json')

if __name__ == '__main__':
    main()
