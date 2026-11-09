import datetime
import hashlib
import json
import pathlib
import stat
import tarfile
import time

root = pathlib.Path('/private/tmp/redb-native-adapter164-20261109-004200')
scratch = pathlib.Path('/private/tmp/redb-native-review165-20261109-014910')
workspace = pathlib.Path('/Users/tommyvolk/.t3/worktrees/verus/t3-60978068')
start = datetime.datetime.fromisoformat('2026-11-09T01:49:10+00:00')
deadline = start + datetime.timedelta(minutes=14)
pins = {
    'HANDOFF164.json': 'ca8691451a59a2ce81acd3e0b438b443dfbcb684fb44fa0b2dd40d1f8cc1357e',
    'HANDOFF164.tar.gz': '3bd20e516bfda56de3377caf29637e1796cb1cc4179db450445e8d69f0a9b133',
    'MANIFEST164.json': 'd4042d23dea2fcaf56af5fa6f7bfeef303ec66ea6bceb3b7f5a21370efeef7c4',
    'final-source164.json': '55dfcba0c24fe60598d7fb3234768a04d6548d8645bcd18298cfec5d24f6f47b',
    'runtime/native-adapter164-built33': '7a68ac1999cad3f6088fc6cb4004295d309a2b6de6c8643d570ef085e90b5962',
    'distinct164-on-repair163.patch': 'dcd9bbf884c52f628daf2125b6fc2ccd290b866f8f0421490f835abee5bb75e7',
}
failures = []


def digest(file):
    return hashlib.sha256(file.read_bytes()).hexdigest()


def free_bytes():
    import os
    info = os.statvfs(scratch)
    return info.f_bavail * info.f_frsize


free_before = free_bytes()
for name, expected in pins.items():
    if digest(root / name) != expected:
        failures.append('outer pin: ' + name)
manifest = json.loads((root / 'MANIFEST164.json').read_text())['files']
for entry in manifest:
    file = root / entry['path']
    info = file.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_size != entry['bytes'] or digest(file) != entry['sha256']:
        failures.append('manifest: ' + entry['path'])
with tarfile.open(root / 'HANDOFF164.tar.gz', 'r:gz') as archive:
    members = archive.getmembers()
    names = [member.name for member in members]
    if len(names) != len(set(names)):
        failures.append('duplicate archive members')
    for member in members:
        name = pathlib.PurePosixPath(member.name)
        if not member.isfile() or name.is_absolute() or '..' in name.parts:
            failures.append('unsafe archive member: ' + member.name)
            continue
        if hashlib.sha256(archive.extractfile(member).read()).hexdigest() != digest(root / member.name):
            failures.append('archive/current: ' + member.name)
inventory = json.loads((root / 'final-source164.json').read_text())
for entry in inventory:
    file = root / 'candidate' / entry['path']
    if file.stat().st_size != entry['bytes'] or digest(file) != entry['sha256']:
        failures.append('final candidate: ' + entry['path'])
external = {
    str(workspace / 'evidence/redb-native-mir-campaign115/ALLOCATOR-LAW-DISPOSITION159.md'): '338b4acea0d19c6474b3258193a94319f67038f7e858cb71f198535f565a928b',
    '/private/tmp/redb-primitive158-20261108-234646/LAW158.md': '0e95e3c2a8182c12573c57d71d4b93162a1162de343580ff896b45f5bdf8443f',
}
for name, expected in external.items():
    if digest(pathlib.Path(name)) != expected:
        failures.append('external: ' + name)
raw_receipts = []
for number in range(33, 40):
    for receipt in root.glob(f'*{number:02}.receipt.json'):
        record = json.loads(receipt.read_text())
        for suffix, key in [('stdout', 'stdout_sha256'), ('stderr', 'stderr_sha256'), ('source-entry.json', 'source_entry_sha256'), ('source-exit.json', 'source_exit_sha256')]:
            file = receipt.with_name(receipt.name.replace('receipt.json', suffix))
            if digest(file) != record[key]:
                failures.append('retained endpoint: ' + file.name)
        raw_receipts.append({'receipt': receipt.name, 'returncode': record['returncode'], 'source_unchanged': record['source_unchanged'], 'reaped': record['reaped']})
if (root / 'counter164.json').read_text().strip() != '39':
    failures.append('changed164 counter')
end = datetime.datetime.now(datetime.timezone.utc)
free_after = free_bytes()
if min(free_before, free_after) <= 16 * 1024**3:
    failures.append('free disk not strictly above16GiB')
if end >= deadline:
    failures.append('14-minute review deadline exceeded')
handoff = {
    'review': 165, 'subject': 'sealed164 only; not live166',
    'result': 'RETAIN_DISABLED_SCALAR_SUBSET_ONLY; P2_QUARANTINE_REPAIR_NEEDED; NO_NATIVE_PROOF_OR_ISSUER_APPROVAL',
    'report': 'REPORT165.md', 'report_sha256': digest(scratch / 'REPORT165.md'),
    'findings': {'P1_established': 0, 'P2': ['F165-1 body-only serialized-opcode quarantine'], 'scoped_objections': ['O165-2 validity-changing scalar transmutes are not safety-checked']},
    'authentication': {'entry': 'all supplied pins,1507 payloads,1120 candidate files,1508 unique regular archive/current members and runtime matched', 'exit_failures': failures, 'exit_pins': pins, 'external_pins': external, 'manifest_files': len(manifest), 'archive_members': len(members), 'candidate_files': len(inventory), 'raw_final_receipts': raw_receipts, 'endpoint_only': True},
    'accounting': {'start_utc': start.isoformat(), 'seal_authentication_end_utc': end.isoformat(), 'deadline_utc': deadline.isoformat(), 'elapsed_seconds_at_authentication': (end-start).total_seconds(), 'free_bytes_entry_df': 22236112 * 1024, 'free_bytes_seal_before': free_before, 'free_bytes_seal_after': free_after, 'free_bytes_required_strictly_above': 16 * 1024**3, 'fresh_scratch': str(scratch), 'scratch_limit_bytes': 2 * 1024**2, 'scratch_enforced_upper_bound_bytes': 128 * 1024, 'heavy_commands': 0, 'compiler_build_verifier_solver_application_executions': 0, 'network': 0, 'git_operations': 0, 'original_parent_candidate_edits': 0, 'subdelegation': 0, 'heavy_slot_acquisition': 0, 'cleanup': 0, 'live166_inspection_or_polling': 0, 'setup_failures': ['SETUP-FAILURE165.txt: initial shell exit127; retained'], 'sealed164_commands_unchanged': '39/64', 'command4_result': 'LOST_UNESTABLISHED'},
    'boundary': {'mathematical_schemas_conditionally_adopted': 1, 'operative_laws': 0, 'genuine_issuance': 0, 'accepted_original_allocations_stores_bodies': [0,0,0], 'G1': 'OPEN', 'G2': 'OPEN', 'all18': 'OPEN', 'first_cell': 'two-phase acknowledged durability', 'native_export': 'REFUSED', 'issuer_approval': False, 'automatic_followup': False},
}
(scratch / 'HANDOFF165.json').write_text(json.dumps(handoff, indent=2, sort_keys=True) + '\n')
payloads = [{'path': file.name, 'bytes': file.stat().st_size, 'sha256': digest(file)} for file in sorted(scratch.iterdir()) if file.is_file() and file.name != 'SEAL165.json']
seal = {'review':165, 'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(), 'status':'SEALED_WITH_RECORDED_FAILURES' if failures else 'SEALED', 'files':payloads, 'self_excluded':'SEAL165.json', 'archive':'none; minimal report/hash/accounting seal', 'scratch_limit_bytes':2*1024**2, 'authentication_failures':failures}
(scratch / 'SEAL165.json').write_text(json.dumps(seal, indent=2, sort_keys=True) + '\n')
logical = sum(file.stat().st_size for file in scratch.iterdir() if file.is_file())
allocated = sum(file.stat().st_blocks * 512 for file in scratch.iterdir() if file.is_file())
assert logical < 128 * 1024 and allocated < 128 * 1024
assert datetime.datetime.now(datetime.timezone.utc) < deadline
print(json.dumps({'status':seal['status'], 'report_sha256':handoff['report_sha256'], 'handoff_sha256':digest(scratch/'HANDOFF165.json'), 'seal_sha256':digest(scratch/'SEAL165.json'), 'scratch_logical_bytes':logical, 'scratch_allocated_bytes':allocated, 'elapsed_seconds':(datetime.datetime.now(datetime.timezone.utc)-start).total_seconds(), 'free_bytes':free_bytes(), 'failures':failures}, indent=2))
raise SystemExit(1 if failures else 0)
