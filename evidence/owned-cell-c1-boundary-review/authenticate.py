"""Authenticate only frozen delivered tools/sources and explicitly listed terminal logs."""
import json,sys,subprocess
from pathlib import Path
sys.dont_write_bytecode=True
R=Path(__file__).resolve().parents[2];sys.path.insert(0,str(R/'source/tools'))
from infer_eval_support import digest
from infer_experiment import require
S=Path('/tmp/owned-cell-client-C1-20261112-freeze')
P=R/'source/tools/research/redb_crash_protocol/native_state_link/implementation/C1_RESULTS.json'
require(digest(S/'receipt.json')=='a72a986cc6cf667991f49466d8cb18643d3057530e9e2b2d46b7876070077ce0','freeze receipt')
s=json.loads((S/'receipt.json').read_text());require(digest(P)==s['C1_RESULTS_sha256'],'manifest')
d=json.loads(P.read_text())
for n,h in s['tools_sha256'].items():require(digest(S/n)==h,'tool '+n)
require(digest(S/'native-client-test')==s['native_client_test_sha256'],'original executable')
for p,h in d['source_sha256'].items():require(digest(R/p)==h,'source '+p)
logs=[]
for c in d['commands']:
 p=Path(c['receipt']);require(p.parent.parent==Path('/tmp/owned-cell-implementation-20261112'),'terminal log scope')
 files={str(p):c['receipt_sha256'],str(p.parent/(c['label']+'.stdout')):c['stdout_sha256'],str(p.parent/(c['label']+'.stderr')):c['stderr_sha256']}
 for path,h in files.items():require(digest(Path(path))==h,'terminal log '+path)
 logs.append(dict(label=c['label'],exit=c['exit'],files=files))
require(len(logs)==16,'16 completed original log sets')
parent=Path('/tmp/owned-cell-C1-parent-replay-v1/receipt.json')
require(digest(parent)=='0d90505c464354400f8e8ccda77287e1b05fe8b5125c165f93a3e7c5d2373f35','parent receipt')
extra=['source/tools/infer_eval_support.py','source/tools/infer_experiment.py','source/tools/research/redb_crash_protocol/native_state_link/implementation/check.py']
base=R/'source/tools/research/redb_crash_protocol'
extra += [str(p.relative_to(R)) for p in [base/'GOAL.md',base/'CURRENT.md',base/'WORKER_BRIEFS.md',base/'native_state_link/ACCEPTANCE_REFINEMENTS.md',*sorted((base/'native_state_link/implementation').glob('C1_*'))] if p.is_file()]
r=dict(prepared_root=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),source=d['source_sha256'],snapshot=s,snapshot_receipt_sha256=digest(S/'receipt.json'),parent_replay_receipt_sha256=digest(parent),parent_controller_sha256=digest(Path('/tmp/replay-owned-cell-C1-parent-v1.py')),instructions_and_helpers={p:digest(R/p) for p in extra},original_terminal_log_sets=logs,terminal_log_content_interpreted=False,build_provenance='Frozen worker-built rust_verify and original test; seven inherited tools, normal vstd2257/0 not reverified; replay is not independent compiler rebuild.',success=True)
Path(__file__).with_name('BINDING.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(dict(success=True,sources=len(d['source_sha256']),tools=len(s['tools_sha256']),terminal_log_sets=len(logs))))
