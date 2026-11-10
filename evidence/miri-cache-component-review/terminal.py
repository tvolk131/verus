"""Retain exact controller argv, terminal status, and private raw log hashes."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

directory = Path(sys.argv[1]).resolve()
directory.mkdir(exist_ok=True)
label, argv = sys.argv[2], sys.argv[3:]
record = directory / (label + '.json')
if record.exists():
    raise ValueError('terminal label already used')
with (directory / (label + '.stdout')).open('wb') as out, (directory / (label + '.stderr')).open('wb') as err:
    p = subprocess.run(argv, stdout=out, stderr=err)
logs = {suffix: hashlib.sha256((directory / (label + '.' + suffix)).read_bytes()).hexdigest() for suffix in ['stdout', 'stderr']}
record.write_text(json.dumps(dict(argv=argv, cwd=str(Path.cwd()), exit_code=p.returncode, logs=logs), indent=2) + '\n')
print(json.dumps(dict(label=label, exit_code=p.returncode, terminal=str(record))))
sys.exit(p.returncode)
