import hashlib
import json
from pathlib import Path
import shutil
import sys
import tarfile
import tempfile

sys.dont_write_bytecode = True
REPO = Path(__file__).resolve().parents[2]
CAMPAIGN = Path('/private/tmp/verus-crash-consistency94-20261105')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def main():
    if shutil.disk_usage('/private/tmp').free < 8 * 1024 ** 3:
        raise RuntimeError('Insufficient free space for new parallel lanes')
    sys.path.insert(0, str(REPO / 'source/tools'))
    from check_redb_stateful import full_source_manifest
    root = Path(tempfile.mkdtemp(prefix='verus-parallel94-', dir='/private/tmp'))
    sources = full_source_manifest(REPO)
    archive = root / 'parent-source-overlay.tar.gz'
    with tarfile.open(archive, 'w:gz', dereference=True) as output:
        for name, expected in sorted(sources.items()):
            relative = Path(name)
            if relative.is_absolute() or '..' in relative.parts or digest(REPO / relative) != expected:
                raise RuntimeError('Noncanonical or changing parent source: ' + name)
            output.add(REPO / relative, arcname=name, recursive=False)
    if full_source_manifest(REPO) != sources:
        raise RuntimeError('Parent source changed during handoff capture')
    record(root / 'parent-source-manifest.json', sources)
    runtime_root = CAMPAIGN / 'codec-runtime-005'
    binding = json.loads((runtime_root / 'runtime-binding.json').read_bytes())
    for name, expected in binding['runtime'].items():
        if digest(runtime_root / 'runtime' / name) != expected:
            raise RuntimeError('Frozen runtime mismatch: ' + name)
    recovery_root = CAMPAIGN / 'native-leaf-lookup-007'
    recovery = json.loads((recovery_root / 'recovery.json').read_bytes())
    guards = json.loads(Path('/private/tmp/verus-bitmap-stateful-goal.IR9WYXR2/baseline82/immutable-hashes.json').read_bytes())
    for original, alias in recovery['aliases'].items():
        if digest(Path(alias)) != guards[original]:
            raise RuntimeError('Recovered native content mismatch: ' + original)
    seed = CAMPAIGN / 'drop-effects-parent-002/handoff'
    seed_seal = digest(seed / 'SEAL.json')
    if seed_seal != '5fb0bcd223b0be2959adbc5b2614053c5c627ba855f548c46a92c3e38029253b':
        raise RuntimeError('Experimental seed seal mismatch')
    third = CAMPAIGN / 'mask-compiler-candidate-001/build/cargo-home/registry/src/index.crates.io-1949cf8c6b5b557f/hex-0.4.3'
    third_sources = {str(path.relative_to(third)): digest(path)
        for path in sorted(third.rglob('*')) if path.is_file()}
    record(root / 'third-repo-source-manifest.json', third_sources)
    handoff = dict(schema='parallel-proof-lanes94-frozen-handoff-v1',
        parent_repository=str(REPO), expected_parent_head='3b05c1912b9d7314ad00c832c4065f4362562e02',
        parent_source_paths=len(sources), parent_source_archive=str(archive),
        parent_source_archive_sha256=digest(archive), parent_source_archive_bytes=archive.stat().st_size,
        parent_source_manifest_sha256=digest(root / 'parent-source-manifest.json'),
        runtime=str(runtime_root / 'runtime'), runtime_binding=str(runtime_root / 'runtime-binding.json'),
        runtime_binding_sha256=digest(runtime_root / 'runtime-binding.json'),
        native_recovery=str(recovery_root / 'recovery.json'), native_recovery_sha256=digest(recovery_root / 'recovery.json'),
        recovered_native=str(recovery_root / 'recovered-native'), original_native_path_preservation='FAILED',
        recovered_native_contents=len(recovery['aliases']), experimental_cleanup_seed=str(seed),
        experimental_cleanup_seed_seal_sha256=seed_seal,
        third_repo=str(third), third_repo_manifest_sha256=digest(root / 'third-repo-source-manifest.json'),
        third_repo_version='hex0.4.3', all_eighteen_cells='OPEN', integration_requires_delta_against_frozen_parent=True,
        managed_worktree_location='TOOL_SELECTED_NOT_PROMISED_TMP_OR_AUTODELETED',
        automatic_cleanup_permitted_before_deliverables_preserved=False)
    record(root / 'HANDOFF.json', handoff)
    for path in root.iterdir():
        path.chmod(0o444)
    print(json.dumps(dict(root=str(root), source_paths=len(sources), source_archive_bytes=archive.stat().st_size,
        handoff_sha256=digest(root / 'HANDOFF.json'), original_native_path_preservation='FAILED')))


if __name__ == '__main__':
    main()
