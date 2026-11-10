#!/usr/bin/env python3
"""Replay only the two frozen libraries in a new private /tmp output root."""
import json
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
review = runpy.run_path(str(HERE/'review.py'), run_name='review_module')
ROOT, SPECS = review['ROOT'], review['SPECS']
frozen = json.loads((HERE/'pretrial.json').read_bytes())
for name, expected in frozen['files'].items():
    review['need'](review['digest'](ROOT/name) == expected, 'frozen review input mismatch: ' + name)
for key, (directory, revision, completed, expected, count) in SPECS.items():
    review['need'](review['digest'](Path(completed)/'receipt.json') == expected, 'completed receipt mismatch')
private = Path(tempfile.mkdtemp(prefix='pair-shared-reader-review-', dir='/tmp'))
processes = []
streams = []
try:
    for key, (directory, revision, completed, expected, count) in SPECS.items():
        stdout = (private/(key+'-controller.stdout')).open('w')
        stderr = (private/(key+'-controller.stderr')).open('w')
        streams.extend([stdout, stderr])
        scope = ROOT/'source/tools/research'/directory
        command = [sys.executable, '-B', str(scope/'check.py'), '--freeze',
                   str(scope/'final-freeze.json'), '--output', str(private/key)]
        processes.append((key, subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr)))
    statuses = {key: process.wait() for key, process in processes}
finally:
    for stream in streams:
        stream.close()
print(json.dumps(dict(private=str(private), exit_codes=statuses)))
review['need'](all(code == 0 for code in statuses.values()), 'development/setup replay failure; retained and excluded')
review['finish'](private)
