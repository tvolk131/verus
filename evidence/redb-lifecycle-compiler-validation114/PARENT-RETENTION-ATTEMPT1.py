import datetime
import hashlib
import json
import pathlib
import shutil
import tarfile


PARENT = pathlib.Path('/Users/tommyvolk/.t3/worktrees/verus/t3-60978068')
ROUND = PARENT / 'evidence/redb-lifecycle-compiler-validation114'
WORKER = pathlib.Path('/private/tmp/redb-compiler-validation114-worker-20261108T151020Z')
REVIEW = pathlib.Path('/private/tmp/redb-compiler-validation114-review-20261108T152149Z-independent')
REVISION = pathlib.Path('/private/tmp/redb-lifecycle113-origin-revision2-20261108T145432Z')


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def load(path):
    return json.loads(path.read_bytes())


def save(path, value):
    assert not path.exists(), str(path)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def stamp(path):
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': digest(path)}


def authenticate(root, name, expected, count):
    manifest_path = root / name
    assert digest(manifest_path) == expected, str(manifest_path)
    rows = load(manifest_path)['records']
    assert len(rows) == count
    assert len({row['path'] for row in rows}) == count
    for row in rows:
        relative = pathlib.PurePosixPath(row['path'])
        assert not relative.is_absolute() and '..' not in relative.parts
        path = root / relative
        assert path.stat().st_size == row['bytes'], str(path)
        assert digest(path) == row['sha256'], str(path)
    return rows


def authenticate_parent():
    rows = load(ROUND / 'ENTRY-PARENT-INPUTS.json')['records']
    assert len(rows) == 1478
    for row in rows:
        path = pathlib.Path(row['path'])
        assert path.is_relative_to(PARENT)
        assert path.stat().st_size == row['bytes'], str(path)
        assert digest(path) == row['sha256'], str(path)
    return len(rows)


def archive_payloads(root, rows, name):
    destination = ROUND / name
    assert not destination.exists()
    with tarfile.open(destination, 'w:gz') as archive:
        for row in rows:
            info = tarfile.TarInfo(row['path'])
            info.size = row['bytes']
            info.mode = 0o644
            with (root / row['path']).open('rb') as stream:
                archive.addfile(info, stream)
    expected = {row['path']: row for row in rows}
    observed = set()
    with tarfile.open(destination, 'r:gz') as archive:
        for member in archive:
            assert member.isfile() and member.name in expected
            assert member.name not in observed
            observed.add(member.name)
            row = expected[member.name]
            assert member.size == row['bytes']
            stream = archive.extractfile(member)
            result = hashlib.sha256()
            for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                result.update(chunk)
            assert result.hexdigest() == row['sha256'], member.name
    assert observed == set(expected)
    return {'archive': stamp(destination), 'members_byte_compared': len(observed)}


def copy_exact(root, source, destination):
    target = ROUND / destination
    assert not target.exists(), str(target)
    shutil.copyfile(root / source, target)
    assert digest(target) == digest(root / source)


def main():
    first_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
    assert shutil.disk_usage(ROUND).free > 16 * 1024 ** 3
    worker_rows = authenticate(WORKER, 'SEALED-MANIFEST.json',
        'c73d2fba54d4144d3705d78a6d5d6cf9d29bf5eaeeb80dce6d9736fd001d6dd9', 4064)
    review_rows = authenticate(REVIEW, 'CONTENT-MANIFEST.json',
        '4d44e481b849531f15b77a9eeadd568da5bc022400182d859d2da47881ae9e97', 7)
    revision_rows = authenticate(REVISION, 'MANIFEST.json',
        'a586fe4665ee832499ea233969eb2914b941093186596f849d8cfeea9321c342', 21)
    assert digest(REVIEW / 'SEAL.json') == '55dc75af62a5e1d06d241f105aaca7a3e6ec781aa91a904add411aeaa43a5958'
    parent_matches = authenticate_parent()
    selected_names = {
        'completed_capture_without_matching_local_is_explicit',
        'completed_empty_capture_is_not_missing',
        'matching_observations_do_not_establish_a_unique_occurrence',
        'missing_capture_is_explicit',
        'unfinished_capture_is_not_completed_capture',
    }
    stdout = (WORKER / 'command-2.stdout').read_text()
    actual_names = {line.split()[1].rsplit('::', 1)[-1]
                    for line in stdout.splitlines() if line.startswith('test checked_normal_drop::')}
    assert actual_names == selected_names
    assert '5 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out' in stdout
    receipt_summaries = []
    for number in (1, 2):
        receipt = load(WORKER / f'command-{number}.receipt.json')
        assert receipt['child_exit_code'] == 0 and receipt['status'] == 'EXITED_ZERO'
        for stream in ('stdout', 'stderr'):
            assert digest(WORKER / f'command-{number}.{stream}') == receipt[f'{stream}_sha256']
        stderr = (WORKER / f'command-{number}.stderr').read_text()
        assert 'warning: failed to save last-use data' in stderr
        assert 'attempt to write a readonly database' in stderr
        receipt_summaries.append({key: receipt[key] for key in
            ('argv', 'child_exit_code', 'status', 'start_utc', 'end_utc', 'elapsed_seconds')})
    for source, target in [('evidence/redb-crash-consistency94/GOAL.md', 'PRE-FINAL-GOAL.md'),
                            ('evidence/redb-crash-consistency94/STATUS.json', 'PRE-FINAL-STATUS.json')]:
        copy_exact(PARENT, source, target)
    excluded = [row for row in worker_rows if row['path'].split('/')[0] in {'target-check', 'target-tests'}]
    included = [row for row in worker_rows if row not in excluded]
    included.append({'path': 'SEALED-MANIFEST.json', **{key: value for key, value in
        stamp(WORKER / 'SEALED-MANIFEST.json').items() if key != 'path'}})
    retained_worker = archive_payloads(WORKER, included, 'WORKER-SOURCE-AND-RECEIPTS.tar.gz')
    retained_review = archive_payloads(REVIEW, review_rows + [
        {'path': name, 'bytes': (REVIEW / name).stat().st_size, 'sha256': digest(REVIEW / name)}
        for name in ('CONTENT-MANIFEST.json', 'SEAL.json')], 'INDEPENDENT-REVIEW.tar.gz')
    for source, target in [('REPORT.md', 'WORKER-REPORT.md'),
                           ('COMMAND-LEDGER.json', 'COMMAND-LEDGER.json'),
                           ('complete-source-delta.patch', 'SOURCE-DELTA.patch'),
                           ('metadata-adaptation.patch', 'SNAPSHOT-METADATA.patch'),
                           ('command-1.stdout', 'CHECK.stdout'), ('command-1.stderr', 'CHECK.stderr'),
                           ('command-2.stdout', 'TEST.stdout'), ('command-2.stderr', 'TEST.stderr'),
                           ('SEALED-MANIFEST.json', 'WORKER-SEALED-MANIFEST.json')]:
        copy_exact(WORKER, source, target)
    for source, target in [('REPORT.md', 'INDEPENDENT-REVIEW.md'),
                           ('CONTENT-MANIFEST.json', 'REVIEW-CONTENT-MANIFEST.json'),
                           ('SEAL.json', 'REVIEW-SEAL.json')]:
        copy_exact(REVIEW, source, target)
    assert authenticate_parent() == parent_matches
    save(ROUND / 'PARENT-AUDIT.json', {
        'first_utc': first_utc,
        'final_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'worker_payloads_authenticated': len(worker_rows),
        'review_payloads_authenticated': len(review_rows),
        'review_seal_hash_authenticated': True,
        'revision2_payloads_authenticated': len(revision_rows),
        'parent_records_matched_before_document_updates': parent_matches,
        'compiler_receipts': receipt_summaries,
        'five_tests_exact': sorted(selected_names),
        'seven_tests_filtered_not_passed': True,
        'cargo_readonly_bookkeeping_warning_in_both_runs': True,
        'retained_worker': retained_worker,
        'retained_review': retained_review,
        'excluded_compiler_outputs': {
            'count': len(excluded), 'bytes': sum(row['bytes'] for row in excluded),
            'path_prefixes': ['target-check/', 'target-tests/'],
            'available_at_original_worker_root': str(WORKER),
            'hashes_retained_in': 'WORKER-SEALED-MANIFEST.json',
            'copied_to_parent': False,
        },
        'parent_compiler_test_native_runs': 0,
        'all18': 'OPEN', 'native_proof': False, 'trust_adoption': False,
        'goal_service': 'BLOCKED', 'free_bytes_at_end': shutil.disk_usage(ROUND).free,
        'archive_semantics': 'File bytes only; no path, timestamp, mode, ownership, or symlink continuity claim.',
    })
    print(json.dumps({'status': 'PASS', 'worker_payloads_authenticated': len(worker_rows),
        'parent_records_matched': parent_matches, 'worker_archive_members': len(included),
        'excluded_target_files': len(excluded), 'retention_root': str(ROUND)}))


if __name__ == '__main__':
    main()
