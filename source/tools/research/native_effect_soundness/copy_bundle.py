#!/usr/bin/env python3
"""Make a byte-for-byte private COPY of the explicitly authorized runtime."""
import argparse
import json
from pathlib import Path
import shutil
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import require,replay_receipt
p=argparse.ArgumentParser()
p.add_argument('--destination',type=Path,required=True)
a=p.parse_args();a.destination.mkdir(parents=True,exist_ok=False)
manifest=json.loads((HERE/'bundle-identity.json').read_text())
r=dict(schema=1,source='authorized 61f21823 runtime',copies={})
with replay_receipt(a.destination,r):
    guard=ArtifactGuard([Path(v['source']) for v in manifest.values()])
    for name,identity in manifest.items():
        source=Path(identity['source']);out=a.destination/name
        require(digest(source)==identity['sha256'],'original runtime changed')
        shutil.copy2(source,out)
        require(digest(out)==identity['sha256'] and out.stat().st_ino!=source.stat().st_ino and not out.is_symlink(),'not an immutable byte copy')
        r['copies'][name]=dict(sha256=digest(out),source=str(source),copy=str(out))
    guard.check()
