import hashlib
import json
import re
import stat
import tarfile
from datetime import datetime, timezone
from pathlib import Path


root = Path('/private/tmp/redb-g3-scalar171-20261108-dtCiyX')
stage = Path('/private/tmp/redb-parent-layout171-20261109T0318Z')
expected = {
    'g3_scalar171.rs': 'b40a1fd8b4c4601bbd79558418fb67a5d2b92efac75f70a0c6af776e74a8281b',
    'consumer156.patch': '4ac3e2ab9af2a3337a7e4a0c4f6f79a14b16f8a355c9c0bf9ccf393e6671f0c1',
    'g3_cases171.rs': 'b11250a3911c2091b4b0bd8d43fdb113167c8daa93f847f9e306f3251897fc54',
    'README171.md': '5e8128d22490b4d6bca8f57c72e4edca66b4804aceece2c41dbf3427579e3014',
    'manifest171.json': '8600cfb4f7f5f625a1deae19dfc3448bfe96ff4adf4e45e426c56116e69a7f0e',
    'accounting171.json': '0358cf6b64834ec5037ca18366ffed51ca171969173cf1cc9a1ce090f0f894e3',
    'SEAL_FAILURE171.txt': 'ef6d51518bef6c97be17f979baef51179cacb7fdfe996cec3f623919ac045bbf',
    'redb-g3-scalar171-20261108-dtCiyX.tar': '71d2bf2fd4329518dd846ee1b5e15e33f89006f34a075398c1c948a1189c7330',
}
errors = []
artifacts = []
payloads = {}
for name, digest in expected.items():
    path = root / name
    if not stat.S_ISREG(path.lstat().st_mode):
        errors.append('Nonregular payload: ' + name)
        continue
    content = path.read_bytes()
    actual_digest = hashlib.sha256(content).hexdigest()
    payloads[name] = content
    artifacts.append({'path': str(path), 'bytes': len(content), 'sha256': actual_digest})
    if actual_digest != digest:
        errors.append('Payload digest mismatch: ' + name)
if errors:
    raise SystemExit(json.dumps(errors))

manifest = json.loads(payloads['manifest171.json'])
for record in manifest['authored_payloads']:
    content = payloads[record['archive_member']]
    if len(content) != record['bytes'] or hashlib.sha256(content).hexdigest() != record['sha256']:
        errors.append('Manifest mismatch: ' + record['archive_member'])
archive_names = []
with tarfile.open(str(root / 'redb-g3-scalar171-20261108-dtCiyX.tar'), 'r:') as archive:
    for member in archive:
        if not member.isreg() or member.name not in manifest['archive']['members']:
            errors.append('Unexpected or nonregular archive member: ' + member.name)
            continue
        if member.name in archive_names:
            errors.append('Duplicate archive member: ' + member.name)
            continue
        archive_names.append(member.name)
        if archive.extractfile(member).read() != payloads[member.name]:
            errors.append('Archive byte mismatch: ' + member.name)
if set(archive_names) != set(manifest['archive']['members']):
    errors.append('Incomplete archive')

inputs = []
for record in manifest['inputs']:
    path = Path(record['path'])
    content = path.read_bytes()
    digest = hashlib.sha256(content).hexdigest()
    matched = all([
        len(content) == record['entry_bytes'],
        len(content) == record['exit_bytes'],
        digest == record['entry_sha256'],
        digest == record['exit_sha256'],
    ])
    inputs.append({'path': str(path), 'bytes': len(content), 'sha256': digest, 'matched': matched})
    if not matched:
        errors.append('Current input mismatch: ' + str(path))

baseline = Path(manifest['consumer_delta']['baseline']).read_text().splitlines(keepends=True)
patch = payloads['consumer156.patch'].decode().splitlines(keepends=True)
result = []
cursor = 0
hunks = 0
patch_cursor = 2
while patch_cursor < len(patch):
    match = re.fullmatch(r'@@ -(\d+),(\d+) \+(\d+),(\d+) @@\n', patch[patch_cursor])
    if match is None:
        raise SystemExit('Unsupported patch header')
    start, old_count, new_start, new_count = map(int, match.groups())
    result.extend(baseline[cursor:start - 1])
    cursor = start - 1
    consumed = 0
    produced = 0
    patch_cursor += 1
    while patch_cursor < len(patch) and not patch[patch_cursor].startswith('@@ '):
        line = patch[patch_cursor]
        marker, text = line[:1], line[1:]
        if marker in (' ', '-'):
            if baseline[cursor] != text:
                raise SystemExit('Patch baseline text mismatch')
            cursor += 1
            consumed += 1
        if marker in (' ', '+'):
            result.append(text)
            produced += 1
        if marker not in (' ', '-', '+'):
            raise SystemExit('Unsupported patch line')
        patch_cursor += 1
    if consumed != old_count or produced != new_count:
        raise SystemExit('Patch count mismatch')
    hunks += 1
result.extend(baseline[cursor:])
staged_consumer = (stage / 'consumer171.rs').read_bytes()
if ''.join(result).encode() != staged_consumer:
    errors.append('Staged consumer is not the exact171 patch result')
for name in ('g3_scalar171.rs', 'g3_cases171.rs'):
    if (stage / name).read_bytes() != payloads[name]:
        errors.append('Staged source mismatch: ' + name)

report = {
    'utc': datetime.now(timezone.utc).isoformat(),
    'scope': 'Post-discovery current endpoint authentication and text-only staging, not execution or continuous custody',
    'initial_discovery_and_source_reads_preceded_full_parent_authentication': True,
    'artifacts': artifacts,
    'archive_unique_regular_byte_matches': len(archive_names),
    'inputs': inputs,
    'patch_hunks': hunks,
    'consumer_sha256': hashlib.sha256(staged_consumer).hexdigest(),
    'errors': errors,
    'compiler_slot_acquired': False,
    'compiler_verifier_solver_native_runs': 0,
    'operative_or_native_credit': 0,
    'all18': 'OPEN',
}
(stage / 'AUTHENTICATION171.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({key: report[key] for key in ('utc', 'archive_unique_regular_byte_matches', 'patch_hunks', 'consumer_sha256', 'errors')}))
if errors:
    raise SystemExit(1)
