import datetime
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile


EVIDENCE = Path(__file__).resolve().parent
KERNEL = Path('/private/tmp/redb-native-implementation170-20261108-204838')
BASELINE = Path('/private/tmp/redb-native-implementation166-20261109-015200/candidate')
LAYOUT = Path('/private/tmp/redb-parent-layout171-20261109T0318Z')
CLEANUP = Path('/private/tmp/redb-git-backup-cleanup-20261109')
EXPECTED_SOURCE = 'b70461c1ab57075caa07f42f8566c2c791c4b854cf539ae259cf4e26685f2fb3'
EXPECTED_BINARY = 'ddc50f3a576728df4d4f8f39793dc26e4d5c7f60b8832e66f472939142d8202c'


def checksum(path):
    result = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1048576), b''):
            result.update(chunk)
    return result.hexdigest()


def write_json(path, value):
    with path.open('x') as output:
        json.dump(value, output, indent=2, sort_keys=True)
        output.write('\n')


def records(root, paths):
    result = []
    for path in sorted(paths):
        if path.is_symlink() or not path.is_file():
            raise ValueError('not a regular input: ' + str(path))
        result.append(dict(path=str(path.relative_to(root)), bytes=path.stat().st_size,
                           sha256=checksum(path)))
    return result


def archive(name, root, paths, qualification):
    inputs = records(root, paths)
    manifest = dict(created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    qualification=qualification, inputs=inputs)
    manifest_bytes = (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode()
    destination = EVIDENCE / name
    with tarfile.open(destination, 'x:gz') as output:
        for entry in inputs:
            path = root / entry['path']
            output.add(path, arcname=entry['path'], recursive=False)
        information = tarfile.TarInfo('PARENT-SNAPSHOT-MANIFEST.json')
        information.size = len(manifest_bytes)
        information.mode = 0o644
        output.addfile(information, io.BytesIO(manifest_bytes))
    with tarfile.open(destination, 'r:gz') as saved:
        members = saved.getmembers()
        if len(members) != len(inputs) + 1 or len({member.name for member in members}) != len(members):
            raise ValueError('archive member count or uniqueness failure')
        for member in members:
            if not member.isfile():
                raise ValueError('nonregular archive member')
            content = saved.extractfile(member).read()
            if member.name == 'PARENT-SNAPSHOT-MANIFEST.json':
                if content != manifest_bytes:
                    raise ValueError('archive manifest mismatch')
            elif content != (root / member.name).read_bytes():
                raise ValueError('archive byte readback mismatch: ' + member.name)
    if records(root, paths) != inputs:
        raise ValueError('snapshot input endpoint drift')
    return dict(path=name, bytes=destination.stat().st_size, sha256=checksum(destination),
                payloads=len(inputs), regular_unique_members=len(inputs) + 1)


def main():
    source_root = KERNEL / 'candidate'
    source_paths = [path for path in source_root.rglob('*') if path.is_file()]
    inventory = records(source_root, source_paths)
    aggregate = hashlib.sha256(json.dumps(inventory, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    if aggregate != EXPECTED_SOURCE or len(inventory) != 1122:
        raise ValueError('final kernel170 source pin mismatch')
    binary = KERNEL / 'target/debug/continuation160'
    if checksum(binary) != EXPECTED_BINARY:
        raise ValueError('retained final binary pin mismatch')
    changed = []
    patch = bytearray()
    for entry in inventory:
        relative = entry['path']
        current = source_root / relative
        baseline = BASELINE / relative
        if baseline.is_file() and checksum(baseline) == entry['sha256']:
            continue
        changed.append(entry)
        destination = EVIDENCE / 'candidates/kernel170' / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(current, destination)
        compared = subprocess.run(['git', 'diff', '--no-index', '--binary', '--no-ext-diff', '--',
                                   str(baseline) if baseline.exists() else '/dev/null', str(current)],
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if compared.returncode != 1:
            raise ValueError('source delta failed: ' + compared.stderr.decode(errors='replace'))
        difference = compared.stdout
        for original in (baseline, current):
            for prefix in ('a', 'b'):
                difference = difference.replace((prefix + str(original)).encode(),
                                                (prefix + '/' + relative).encode())
        patch.extend(difference)
    removed = [str(path.relative_to(BASELINE)) for path in BASELINE.rglob('*')
               if path.is_file() and not (source_root / path.relative_to(BASELINE)).is_file()]
    if removed or len(changed) != 11:
        raise ValueError('unexpected kernel170 delta')
    (EVIDENCE / 'KERNEL170-BASELINE166.patch').write_bytes(patch)
    write_json(EVIDENCE / 'KERNEL170-SOURCE-INVENTORY.json', dict(
        qualification='Parent endpoint snapshot, not a worker seal or native acceptance',
        aggregate_sha256=aggregate, source_files=inventory, baseline166_changed_files=changed))
    layout_destination = EVIDENCE / 'candidates/layout171'
    layout_destination.mkdir(parents=True, exist_ok=False)
    for path in sorted(LAYOUT.iterdir()):
        if path.is_file():
            shutil.copy2(path, layout_destination / path.name)
    qualification = ('Parent preservation of completed-but-unsealed worker170. '
                     'No independent exact-code review, code activation, original native proof or '
                     'retroactive worker seal. All18 and G1/G2 OPEN; native export REFUSED.')
    kernel_paths = [path for path in KERNEL.rglob('*')
                    if path.is_file() and path.relative_to(KERNEL).parts[0] not in {'target', 'tmp'}
                    and path.relative_to(KERNEL) != Path('runtime/canonical-export170')]
    kernel_archive = archive('KERNEL170-PARENT-SNAPSHOT.tar.gz', KERNEL, kernel_paths, qualification)
    cleanup_script = CLEANUP / 'SCRIPT.py'
    shutil.copy2('/private/tmp/redb-git-backup-cleanup.py', cleanup_script)
    cleanup_paths = [CLEANUP / name for name in (
        'SCRIPT.py', 'INVENTORY.json', 'PROTECTED.json', 'REMOVALS.jsonl', 'RESULT.json')]
    cleanup_archive = archive('GIT-BACKUP-CLEANUP.tar.gz', CLEANUP, cleanup_paths,
                              'User-approved cleanup of only target164/166/170; not additional cleanup authority')
    runtime = records(KERNEL, [binary, KERNEL / 'runtime/canonical-export170'])
    write_json(EVIDENCE / 'GIT-BACKUP-SNAPSHOT.json', dict(
        created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        authorization='User requested all-code commit/push and approved the specific cache cleanup',
        qualification=qualification,
        source_aggregate_sha256=aggregate,
        readable_candidate_directory='candidates/kernel170',
        kernel170_parent_snapshot=kernel_archive,
        kernel170_worker_sealed=False,
        runtime_binaries_retained_locally_not_committed=runtime,
        layout171=records(layout_destination, list(layout_destination.iterdir())),
        layout171_status='SOURCE_ONLY_UNCOMPILED_UNVERIFIED_UNEXECUTED',
        cleanup_archive=cleanup_archive,
        cleanup_result=json.loads((CLEANUP / 'RESULT.json').read_text()),
        current_heavy_run_floor_met=False,
        builds_verifiers_solvers_started_for_backup=0,
        laws_activated=0, native_acceptance=0, all18='OPEN'))
    print(json.dumps(dict(kernel170=kernel_archive, cleanup=cleanup_archive,
                          source_files=len(inventory), readable_changed_files=len(changed))))


if __name__ == '__main__':
    main()
