#!/usr/bin/env python3
"""Snapshot actual opam source trees before their first build command.

Every originally present .v is guarded before/after every build command. All
initial bytes are retained privately, including non-formal build inputs.
"""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time

root = Path('/tmp/stbor-checked-foundation-20261110')
name, version, *command = sys.argv[1:]
key = name + '.' + version
out = root / 'manifests' / ('build-source-' + key + '.json')
cwd = Path.cwd()


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


if not out.exists():
    paths = sorted(p for p in cwd.rglob('*') if p.is_file() and '.git' not in p.parts)
    manifest = {str(p.relative_to(cwd)): sha(p) for p in paths}
    snapshot = root / 'source-snapshots' / key
    shutil.copytree(cwd, snapshot, symlinks=True, ignore=shutil.ignore_patterns('.git'))
    out.write_text(json.dumps({'cwd': str(cwd), 'files': manifest,
                               'before_first_build_unix': time.time()}, indent=2) + '\n')
initial = json.loads(out.read_text())['files']
preflight = root / 'manifests' / ('preflight-' + key + '.json')
if name.startswith('coq'):
    if not preflight.exists():
        raise SystemExit('missing dependency pretrial source freeze: ' + key)
    frozen = json.loads(preflight.read_text())['files']
    formal = {p: h for p, h in frozen.items() if p.endswith('.v')}
    actual = {p: h for p, h in initial.items() if p.endswith('.v')}
    if formal != actual:
        raise SystemExit('loaded formal source differs from pretrial freeze: ' + key)


def guard():
    bad = [p for p, h in initial.items() if p.endswith('.v') and
           (not (cwd/p).is_file() or sha(cwd/p) != h)]
    if bad:
        raise SystemExit('formal source guard failed: ' + repr(bad))


guard()
stamp = str(time.time_ns())
record = {'package': key, 'command': command, 'cwd': str(cwd),
          'pid': os.getpid(), 'pgid': os.getpgrp(), 'state': 'running',
          'source_manifest_sha256': sha(out), 'hook_sha256': sha(Path(__file__))}
receipt = root / 'receipts' / ('hook-' + key + '-' + stamp + '.json')
receipt.write_text(json.dumps(record, indent=2)+'\n')
log = root / 'logs' / ('hook-' + key + '-' + stamp + '.log')
with log.open('wb') as out:
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    while True:
        block = process.stdout.read(65536)
        if not block:
            break
        out.write(block)
        out.flush()
        sys.stdout.buffer.write(block)
        sys.stdout.buffer.flush()
    code = process.wait()
guard()
record.update(state='finished', exit_status=code, formal_source_guard='matched',
              log=str(log), log_sha256=sha(log))
receipt.write_text(json.dumps(record, indent=2)+'\n')
sys.exit(code)
