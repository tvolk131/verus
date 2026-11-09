import datetime
import difflib
import hashlib
import json
import pathlib
import shutil


PARENT = pathlib.Path('/Users/tommyvolk/.t3/worktrees/verus/t3-60978068')
ROUND = PARENT / 'evidence/redb-lifecycle-compiler-validation114'


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def record(path):
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': digest(path)}


def save(path, value):
    assert not path.exists(), str(path)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def main():
    baseline = json.loads((ROUND / 'ENTRY-PARENT-INPUTS.json').read_bytes())
    expected_documents = {
        PARENT / 'evidence/redb-crash-consistency94/GOAL.md': 'PRE-FINAL-GOAL.md',
        PARENT / 'evidence/redb-crash-consistency94/STATUS.json': 'PRE-FINAL-STATUS.json',
    }
    changes = []
    unchanged = 0
    source_unchanged = 0
    for previous in baseline['records']:
        path = pathlib.Path(previous['path'])
        current = record(path)
        if current == previous:
            unchanged += 1
            source_unchanged += path.is_relative_to(PARENT / 'source')
            continue
        assert path in expected_documents, str(path)
        preserved = ROUND / expected_documents[path]
        assert digest(preserved) == previous['sha256']
        assert preserved.stat().st_size == previous['bytes']
        changes.append({'before': previous, 'after': current, 'previous_bytes_at': str(preserved)})
    assert {entry['before']['path'] for entry in changes} == {str(path) for path in expected_documents}
    assert unchanged == 1476 and source_unchanged == 1474
    status_path = PARENT / 'evidence/redb-crash-consistency94/STATUS.json'
    status = json.loads(status_path.read_bytes())
    previous_status = json.loads((ROUND / 'PRE-FINAL-STATUS.json').read_bytes())
    assert status['completion_properties'] == previous_status['completion_properties']
    assert status['required_histories'] == previous_status['required_histories']
    assert len(status['completion_properties']) * len(status['required_histories']) == 18
    assert status['all_eighteen_property_history_cells'] == 'OPEN'
    assert status['whole_goal_acceptance'] is False
    assert status['lifecycle_compiler_validation114_compiler_commands_used'] == 2
    assert not status['lifecycle_compiler_validation114_new_trust_or_parent_integration_authorized']
    changed_status = sorted(key for key in status if status[key] != previous_status.get(key))
    assert all(key.startswith(('lifecycle_compiler_validation114_', 'writablepage_lifecycle113_repaired_source_'))
               or key in {'active_goal_state', 'active_goal_blocker'} for key in changed_status)
    sealed113 = PARENT / 'evidence/redb-writablepage-lifecycle113/ROUND-MANIFEST.json'
    assert digest(sealed113) == 'a4cfa51212f0a684d418b4d722e5a701eed897052b6437060c334589b8c2c3fe'
    for source, target in [('/private/tmp/redb-validation114-parent-retention-attempt1.py', 'PARENT-RETENTION-ATTEMPT1.py'),
                           ('/private/tmp/redb-validation114-parent-retention.py', 'PARENT-RETENTION.py'),
                           (__file__, 'FINALIZE.py')]:
        destination = ROUND / target
        assert not destination.exists(), str(destination)
        shutil.copyfile(source, destination)
        assert digest(destination) == digest(pathlib.Path(source))
    previous_script = (ROUND / 'PARENT-RETENTION-ATTEMPT1.py').read_text()
    repaired_script = (ROUND / 'PARENT-RETENTION.py').read_text()
    correction = ''.join(difflib.unified_diff(previous_script.splitlines(True), repaired_script.splitlines(True),
        fromfile='parent-retention-attempt1.py', tofile='parent-retention-corrected.py'))
    (ROUND / 'PARENT-RETENTION-CORRECTION.patch').write_text(correction)
    save(ROUND / 'DOCUMENT-RECONCILIATION.json', {
        'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'entry_baseline_records': 1478,
        'unchanged_records': unchanged,
        'unchanged_source_records': source_unchanged,
        'only_two_disclosed_document_changes': changes,
        'changed_or_added_status_fields': changed_status,
        'sealed113_manifest_unchanged': record(sealed113),
        'all18': 'OPEN', 'whole_goal_acceptance': False,
        'native_runs': 0, 'parent_compiler_test_runs': 0,
        'goal_service_observed': 'BLOCKED',
        'new_trust_or_source_integration': False,
    })
    free_bytes = shutil.disk_usage(ROUND).free
    assert free_bytes > 16 * 1024 ** 3
    paths = sorted(path for path in ROUND.rglob('*') if path.is_file())
    records = [{'path': str(path.relative_to(ROUND)), 'bytes': path.stat().st_size, 'sha256': digest(path)}
               for path in paths]
    assert sum(row['bytes'] for row in records) < 8 * 1024 ** 3
    save(ROUND / 'ROUND-MANIFEST.json', {
        'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'records': records, 'payload_count': len(records),
        'free_bytes_sampled': free_bytes,
        'classification': 'REVIEWED_COMPILER_CHECK_AND_PURE_METADATA_TESTS_NOT_NATIVE_ACCEPTANCE',
        'all18': 'OPEN', 'source_path': 'FAILED', 'git_continuity': 'UNAVAILABLE',
        'compiler_source_correspondence': 'UNESTABLISHED', 'goal_service': 'BLOCKED',
        'native_proof': False, 'trust_adoption': False,
    })
    for row in records:
        assert digest(ROUND / row['path']) == row['sha256']
    print(json.dumps({'status': 'PASS', 'payloads': len(records),
        'round_manifest_sha256': digest(ROUND / 'ROUND-MANIFEST.json'),
        'unchanged_source_records': source_unchanged, 'document_changes': len(changes),
        'retained_bytes': sum(row['bytes'] for row in records), 'all18': 'OPEN'}))


if __name__ == '__main__':
    main()
