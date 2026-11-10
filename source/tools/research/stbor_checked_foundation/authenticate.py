#!/usr/bin/env python3
"""Match release archive payloads to exact official Git trees (NUL paths)."""
from pathlib import Path
import hashlib
import json
import subprocess
import tarfile

root = Path('/tmp/stbor-checked-foundation-20261110')
records = {}
for name, archive, commit in [
    ('coq', 'coq-V8.9.1.tar.gz', 'e1d8a33ccb9ffecde340531cbf9e8fa9d916550e'),
    ('equations', 'equations-v1.2.1-8.9.tar.gz', '69160e69b629587f9dfed88c545c1e7f13cd18b0'),
]:
    repo = root/'auth'/name
    with tarfile.open(root/'downloads'/archive) as tar:
        payload = {'/'.join(m.name.split('/')[1:]): tar.extractfile(m).read()
                   for m in tar.getmembers() if m.isfile()}
    files = {}
    names = subprocess.check_output(['git', '-C', str(repo), 'ls-tree', '-rz',
                                     '--name-only', commit]).decode().split('\0')
    for f in filter(None, names):
        b = subprocess.check_output(['git', '-C', str(repo), 'show', commit+':'+f])
        if f in payload and b != payload[f]:
            raise SystemExit('archive/source mismatch '+name+'/'+f)
        files[f] = hashlib.sha256(b).hexdigest()
    if set(payload)-set(files):
        raise SystemExit('archive has extra files')
    omitted = sorted(set(files)-set(payload))
    if any(f.endswith(('.v', '.ml', '.mli', '.c', '.h')) for f in omitted):
        raise SystemExit('archive omits program/proof source')
    records[name] = {
        'commit': commit,
        'tree': subprocess.check_output(['git', '-C', str(repo), 'rev-parse',
                                         commit+'^{tree}'], text=True).strip(),
        'tracked_source_sha256': files,
        'archive_sha256': hashlib.sha256((root/'downloads'/archive).read_bytes()).hexdigest(),
        'export_omitted': omitted,
    }
(root/'manifests'/'release-authentication.json').write_text(json.dumps(records, indent=2)+'\n')
print('Authenticated complete Coq and Equations release payloads against official Git trees')
