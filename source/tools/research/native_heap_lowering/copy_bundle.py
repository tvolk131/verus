#!/usr/bin/env python3
"""Guarded immutable private COPY; never link or mutate the parent's runtime."""
import argparse,json,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import require,replay_receipt
p=argparse.ArgumentParser();p.add_argument('--destination',type=Path,required=True);a=p.parse_args();a.destination.mkdir(parents=True,exist_ok=False)
r=dict(schema=1,copies={})
with replay_receipt(a.destination,r):
 ident=json.loads((HERE/'bundle-identity.json').read_text());g=ArtifactGuard([HERE/'bundle-identity.json',Path(__file__),*[Path(x['source']) for x in ident.values()]])
 for n,x in ident.items():
  p=Path(x['source']);q=a.destination/n;require(digest(p)==x['sha256'],'original runtime changed')
  shutil.copy2(p,q);q.chmod(q.stat().st_mode & ~0o222)
  require(digest(q)==x['sha256'] and not q.is_symlink() and q.stat().st_ino!=p.stat().st_ino,'not a byte-for-byte independent copy')
  r['copies'][n]=dict(source=str(p),copy=str(q),sha256=digest(q))
 g.check()
