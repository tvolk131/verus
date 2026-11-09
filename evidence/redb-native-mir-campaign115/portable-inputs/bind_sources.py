import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import tarfile


ROOT = Path('/private/tmp/verus-pinned-rust-source-20261107-9FT58F')
INSTALLED = Path('/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin')
PRIOR = Path('/private/tmp/verus-redb-growth049-independent-3VahHb/TOOLCHAIN-BINDINGS.json')
ARCHIVE = ROOT / 'rust-src-1.99.1.tar.xz'
EXPECTED_ARCHIVE = 'e485a88c45db546b73571df7bebfcb4454ba4fb552c492e25fc5b00eeb5bf22d'
MANIFEST = INSTALLED / 'lib/rustlib/multirust-channel-manifest.toml'
EXPECTED_MANIFEST = '48419307eb32e389a25ac360b6bccd4d196f3c4f35ce9d67a223319eff017c6c'
RELEASE = '1.99.1 (58cf8f9c3 2026-10-13)'
SOURCE_URL = 'https://static.rust-lang.org/dist/2026-10-15/rust-src-1.99.1.tar.xz'


def digest_file(path):
    hasher = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            hasher.update(chunk)
    return hasher.hexdigest()


def metadata(path):
    return {'sha256': digest_file(path), 'size': path.stat().st_size}


def section(text, name):
    pattern = r'(?ms)^\[' + re.escape(name) + r'\]\n(.*?)(?=^\[|\Z)'
    match = re.search(pattern, text)
    if match is None:
        raise ValueError('Missing known manifest section: ' + name)
    return match.group(1)


assert shutil.disk_usage(ROOT).free >= 16 * 1024 ** 3
assert digest_file(MANIFEST) == EXPECTED_MANIFEST
manifest_text = MANIFEST.read_text()
assert 'version = "' + RELEASE + '"' in section(manifest_text, 'pkg.rust-src')
assert 'version = "' + RELEASE + '"' in section(manifest_text, 'pkg.rustc')
source_package = section(manifest_text, 'pkg.rust-src.target."*"')
assert 'xz_url = "' + SOURCE_URL + '"' in source_package
assert 'xz_hash = "' + EXPECTED_ARCHIVE + '"' in source_package
assert digest_file(ARCHIVE) == EXPECTED_ARCHIVE

inventory = {}
directory_count = 0
with tarfile.open(str(ARCHIVE), 'r:xz') as package:
    members = package.getmembers()
    assert len(members) <= 20000
    assert sum(member.size for member in members) <= 128 * 1024 ** 2
    for member in members:
        relative = PurePosixPath(member.name)
        assert not relative.is_absolute() and '..' not in relative.parts
        assert relative.parts[0] == 'rust-src-1.99.1'
        assert member.isfile() or member.isdir()
        extracted = ROOT.joinpath(*relative.parts)
        assert not extracted.is_symlink()
        if member.isdir():
            assert extracted.is_dir()
            directory_count += 1
            continue
        assert member.name not in inventory
        assert extracted.is_file() and extracted.stat().st_size == member.size
        with package.extractfile(member) as original:
            original_digest = hashlib.sha256(original.read()).hexdigest()
        assert digest_file(extracted) == original_digest
        inventory[member.name] = {'sha256': original_digest, 'size': member.size}

extracted_paths = list((ROOT / 'rust-src-1.99.1').rglob('*'))
assert all(not path.is_symlink() for path in extracted_paths)
actual_files = {path.relative_to(ROOT).as_posix() for path in extracted_paths if path.is_file()}
assert actual_files == set(inventory)
inventory_path = ROOT / 'SOURCE-INVENTORY.json'
inventory_path.write_text(json.dumps(inventory, indent=2, sort_keys=True) + '\n')

prior_records = json.loads(PRIOR.read_text())
selected_runtime = {}
for relative, record in prior_records.items():
    basename = Path(relative).name
    selected = relative == 'toolchain/bin/rustc'
    selected = selected or basename.startswith('librustc_driver-')
    selected = selected or (
        basename.startswith(('libcore-', 'liballoc-', 'libstd-', 'libpanic_abort-'))
        and Path(basename).suffix in ('.rlib', '.rmeta')
    )
    if not selected:
        continue
    assert relative.startswith('toolchain/')
    installed_path = INSTALLED / relative[len('toolchain/'):]
    observed = metadata(installed_path)
    assert observed['sha256'] == record['sha256']
    assert observed['size'] == record['size']
    selected_runtime[relative] = observed
assert 'toolchain/bin/rustc' in selected_runtime
assert len(selected_runtime) >= 9

version_path = ROOT / 'rustc-version.txt'
version = version_path.read_text()
assert 'release: 1.99.1\n' in version
assert 'commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67\n' in version
assert (ROOT / 'rustc-version.stderr').read_bytes() == b''
summary = {
    'status': 'EXACT_RELEASE_SOURCE_PACKAGE_AVAILABLE_NOT_IMPLEMENTATION_PROOF',
    'rust_release': '1.99.1',
    'rust_commit': '58cf8f9c35b355199727627d07ae34a4e2846d67',
    'source_url': SOURCE_URL,
    'archive': dict(path=str(ARCHIVE), **metadata(ARCHIVE)),
    'installed_manifest': dict(path=str(MANIFEST), **metadata(MANIFEST)),
    'library_root': str(ROOT / 'rust-src-1.99.1/rust-src/lib/rustlib/src/rust/library'),
    'source_inventory': dict(path=str(inventory_path), files=len(inventory),
                             directories=directory_count, **metadata(inventory_path)),
    'source_bytes': sum(record['size'] for record in inventory.values()),
    'prior_toolchain_binding': dict(path=str(PRIOR), **metadata(PRIOR)),
    'selected_runtime_inputs_match_prior': selected_runtime,
    'version_only_metadata_query': dict(path=str(version_path), **metadata(version_path)),
    'qualifications': [
        'Archive digest matches the installed release manifest; no detached signature was newly verified.',
        'Only selected runtime inputs were rehashed; this is not full toolchain reauthentication.',
        'Release/package association is not a source-to-binary reproducible build theorem.',
        'Installed 1.99.0 source was rejected as a substitute for the pinned 1.99.1 source.',
        'No implementation semantics, primitive schema, native resource theorem, or database cell is certified.',
        'The earlier tomllib metadata probe failed; this script validates only explicit known manifest sections.',
    ],
    'compiler_builds': 0,
    'verifier_solver_runs': 0,
    'native_target_executions': 0,
    'new_trust': 0,
    'all_eighteen_property_history_cells': 'OPEN',
}
summary_path = ROOT / 'SOURCE-BINDING.json'
summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + '\n')
print(json.dumps({'status': summary['status'], 'source_files': len(inventory),
                  'source_bytes': summary['source_bytes'],
                  'selected_runtime_inputs': len(selected_runtime),
                  'summary': str(summary_path), 'summary_sha256': digest_file(summary_path)}, indent=2))
