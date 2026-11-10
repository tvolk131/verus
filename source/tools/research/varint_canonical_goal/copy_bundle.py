#!/usr/bin/env python3
"""Create a private immutable COPY; reject symlinks, drift and existing outputs."""
import argparse
import json
from pathlib import Path
import shutil
import sys
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import require

ENGINE_SOURCES = (
    'source/vir/src/infer/buffer_goals.rs',
    'source/vir/src/infer/buffer_observation_goals.rs',
    'source/vir/src/infer/native_mutable_for.rs',
    'source/vir/src/infer.rs',
    'source/vir/src/infer/pipeline.rs',
    'source/vir/src/infer/report.rs',
    'source/vir/src/infer/registry.rs',
    'source/rust_verify/src/query.rs',
    'source/rust_verify/src/import_export.rs',
)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--from', dest='directory', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    names = ['verus', 'rust_verify', 'vstd.vir', 'libvstd.rlib', 'libverus_builtin.rlib',
             'libverus_builtin_macros.so', 'libverus_state_machines_macros.so']
    inputs = [a.directory / name for name in names] + [a.solver]
    require(not any(f.is_symlink() for f in inputs), 'bundle inputs must be files, not links')
    guard = ArtifactGuard(tool_artifacts(a.directory / 'verus', a.solver))
    a.output.mkdir(mode=0o700)
    manifest = dict(source_artifacts=guard.initial, files={},
                    engine_sources={name: digest(ROOT / name) for name in ENGINE_SOURCES})
    for name, source in zip(names + ['z3'], inputs):
        target = a.output / name
        shutil.copy2(source, target)
        target.chmod(0o555 if name in ('verus', 'rust_verify', 'z3') else 0o444)
        require(digest(source) == digest(target) and not target.is_symlink(), 'copy drift')
        require(source.stat().st_ino != target.stat().st_ino, 'hard link is not an independent copy')
        manifest['files'][name] = dict(source=str(source.resolve()), sha256=digest(target))
    guard.check()
    (a.output / 'copy-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(a.output)

if __name__ == '__main__':
    main()
