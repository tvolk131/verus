"""Hash only the eleven explicitly delivered original receipt/log sets."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import digest
from infer_experiment import require
D=ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation/D2_RESULTS.json'
S=Path('/tmp/owned-cell-D2-cc9b088c5-parent-snapshot')
require(digest(S/'receipt.json')=='662f02352b98a6f58a479528245aa51183ba3649d031e43ef1acca178054ec2d','snapshot')
s=json.loads((S/'receipt.json').read_text())
require(digest(D)==s['D2_RESULTS_sha256'],'original results manifest')
d=json.loads(D.read_text())
records=[]
for c in d['commands']:
    p=Path(c['receipt'])
    # No actor checkout/build/bundle reads. These exact terminal logs are
    # authorized for hash authentication only, and are never interpreted here.
    require(p.parent.parent==Path('/tmp/owned-cell-implementation-20261112'),'delivered log location')
    paths={p:c['receipt_sha256'],p.parent/(c['label']+'.stdout'):c['stdout_sha256'],p.parent/(c['label']+'.stderr'):c['stderr_sha256']}
    for path,h in paths.items(): require(digest(path)==h,'original log '+str(path))
    records.append(dict(label=c['label'],files={str(p):h for p,h in paths.items()},exit=c['exit']))
require(len(records)==11,'eleven sets')
out=Path(sys.argv[1]);out.mkdir(exist_ok=False)
(out/'receipt.json').write_text(json.dumps(dict(success=True,manifest_sha256=digest(D),commands=records,content_interpretation=False),indent=2)+'\n')
print(json.dumps(dict(output=str(out),receipt_sha256=digest(out/'receipt.json'))))
