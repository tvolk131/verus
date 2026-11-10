#!/usr/bin/env python3
"""Private durable command receipts; no proof-acceptance shortcuts."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path('/tmp/stbor-checked-foundation-20261110')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def run(name, cwd, command):
    log = ROOT / 'logs' / (name + '.log')
    receipt = ROOT / 'receipts' / (name + '.json')
    if receipt.exists() or log.exists():
        raise SystemExit('refuse to overwrite trial ' + name)
    info = {'name': name, 'command': command, 'cwd': cwd,
            'runner_sha256': sha(__file__), 'started_unix': time.time(),
            'pid': os.getpid(), 'pgid': os.getpgrp(),
            'startticks': Path('/proc/self/stat').read_text().split()[21],
            'log': str(log), 'state': 'running',
            'environment': {k: v for k, v in os.environ.items() if k in
                            ('PATH', 'OPAMROOT', 'OPAMSWITCH', 'OPAMJOBS',
                             'OPAMDOWNLOADJOBS', 'XDG_CACHE_HOME',
                             'COQPATH', 'OCAMLPATH', 'TMPDIR', 'MAKEFLAGS')}}
    write(receipt, info)
    with log.open('wb') as out:
        child = subprocess.Popen(command, cwd=cwd, stdout=out, stderr=subprocess.STDOUT)
        info['child_pid'] = child.pid
        write(receipt, info)
        code = child.wait()
    info.update(state='finished', exit_status=code, finished_unix=time.time(),
                log_sha256=sha(log))
    write(receipt, info)
    return code


if __name__ == '__main__':
    if len(sys.argv) < 5:
        raise SystemExit('usage: run.py NAME CWD COMMAND ARG...')
    sys.exit(run(sys.argv[1], sys.argv[2], sys.argv[3:]))
