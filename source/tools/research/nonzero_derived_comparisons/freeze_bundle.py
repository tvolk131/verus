"""Retain immutable eight-file owned tools after normal matching vstd verification."""
import json
from pathlib import Path
import shutil
import sys
import tempfile
sys.dont_write_bytecode=True
from trial import authenticate,BUNDLE,TOOLS
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import replay_receipt,require

build=Path(sys.argv[1]);vstd=Path(sys.argv[2]);authenticate()
b=json.loads((build/'receipt.json').read_bytes());v=json.loads((vstd/'receipt.json').read_bytes())
require(b['success'] is True and v['success'] is True,'build failure')
profile=Path(b.get('target',str(build/'target')))/'release'
out=Path(tempfile.mkdtemp(prefix='nonzero-derive-frozen-tools-',dir='/tmp'));out.chmod(0o700)
r=dict(schema=1,build_receipt=str(build/'receipt.json'),vstd_receipt=str(vstd/'receipt.json'))
with replay_receipt(out,r):
    files={name:(BUNDLE/name if name=='z3' else profile/name) for name in TOOLS}
    for name,p in files.items():
        expected=(v['matching_vstd_products'][str(p)] if name in ['libvstd.rlib','vstd.vir'] else
                  digest(BUNDLE/'z3') if name=='z3' else b['products'][str(p)])
        require(digest(p)==expected,'owned artifact drift: '+name)
    guard=ArtifactGuard([*files.values(),build/'receipt.json',vstd/'receipt.json',Path(__file__)])
    for name,p in files.items():shutil.copy2(p,out/name)
    guard.check();r['final_guards_passed']=True
    r['inputs']=guard.initial;r['tools']={name:digest(out/name) for name in TOOLS}
print(out)
