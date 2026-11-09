#!/usr/bin/env python3
"""Guarded real-compiler extraction. No successful native-proof publication exists."""
import argparse
import json
import os
from pathlib import Path
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
from freeze import freeze

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--sysroot', type=Path, default=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu'))
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, native_bridge_proved=False, commands=[])
    with replay_receipt(a.output, receipt):
        frozen = freeze(a.output / 'inputs.json')
        source_map_path = HERE.parent / 'native_iterator_refinement/source-map.json'
        source_map = json.loads(source_map_path.read_text())
        native_root = a.sysroot / 'lib/rustlib/src/rust'
        for name, expected in source_map['native_files'].items():
            require(digest(native_root / name) == expected, f'pinned Rust source mismatch: {name}')
        selected = HERE.parent / 'buffer_transfer_goal/original-selected.rs'
        selected_loop = '\n'.join(line.strip() for line in selected.read_text().splitlines())
        original = HERE / 'fixtures/original.rs'
        require(selected_loop in '\n'.join(line.strip() for line in original.read_text().splitlines()), 'original loop changed')
        tools = [a.sysroot / 'bin/rustc', a.sysroot / 'bin/cargo',
                 *a.sysroot.glob('lib/librustc_driver-*.so'),
                 *a.sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/libcore-*.rlib'),
                 *a.sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/librustc_middle-*.rmeta')]
        inputs = [HERE / name for name in frozen['files']]
        inputs += [source_map_path, selected, a.output / 'inputs.json',
                   *[native_root / name for name in source_map['native_files']]]
        guard = ArtifactGuard([*tools, *inputs, *support_artifacts()])
        receipt['artifacts'] = guard.initial
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
                   RUSTUP_TOOLCHAIN='1.99.1', RUSTC_BOOTSTRAP='1', CARGO_BUILD_JOBS='2',
                   CARGO_TARGET_DIR=str(a.output / 'target'),
                   LD_LIBRARY_PATH=str(a.sysroot / 'lib'))
        run = LoggedCommands(a.output, ROOT / 'source', receipt['commands'], timeout=240, guard=guard)
        require(run('rustc-version', [str(a.sysroot / 'bin/rustc'), '-vV'], env) == 0, 'rustc identity')
        require('release: 1.99.1' in (a.output / 'rustc-version.stdout').read_text(), 'wrong compiler version')
        require(run('build', [str(a.sysroot / 'bin/cargo'), 'build', '--offline', '--locked',
            '--manifest-path', str(HERE / 'Cargo.toml'), '--config',
            f'build.build-dir="{a.output / "build"}"'], env) == 0, 'extractor build failed')
        binary = a.output / 'target/debug/native-iterator-extract'
        binary_guard = ArtifactGuard([binary])
        receipt['extractor'] = binary_guard.initial
        evidence = a.output / 'original.json'
        require(run('extract-original', [str(binary), str(evidence), '256', str(original),
            '--crate-name', 'native_writer', '--edition', '2024', '--sysroot', str(a.sysroot),
            '-Zmir-opt-level=0', '-Coverflow-checks=on'], env) == 0, 'extraction failed')
        binary_guard.check()
        data = json.loads(evidence.read_text())
        require(data['native_bridge_proved'] is False, 'extractor must not certify')
        require(not data['pending_instances'], 'call graph incomplete')
        summary = dict(native_bridge_proved=False, root=data['root'], target=data['target'],
            pointer_bits=data['pointer_bits'], instances=len(data['instances']), operations=data['operations'],
            extraction_blockers=data['extraction_blockers'],
            calls=[dict(display=i['display'], kind=i['kind'], blocks=len(i.get('blocks', []))) for i in data['instances']],
            evidence_sha256=digest(evidence))
        receipt['summary'] = summary
        (a.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        guard.check()

if __name__ == '__main__':
    main()
