import hashlib,json,subprocess
from pathlib import Path
H=Path('evidence/nonzero-wrapper-parent-review');S=Path('source/tools/research/nonzero_wrapper_ordering');W=Path('/tmp/nonzero-wrapper-ordering-94jlhknm');N=W/'sealed-validation'
def need(x,m):
 if not x:raise RuntimeError(m)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def git(p):return subprocess.check_output(['git','show','c4ddc83e:'+p])
def load(p):return json.loads(Path(p).read_text())
seal=load(S/'CURRENT_SEAL.json')
for p,h in seal.items():need(sha(S/p)==h,'committed seal '+p)
f=load(N/'sealed-eight-tools.json')
# One cat-file process, no historical checkout/source reads.
proc=subprocess.Popen(['git','cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
for p,h in f['source'].items():
 proc.stdin.write(('c4ddc83e:'+p+'\n').encode());proc.stdin.flush();head=proc.stdout.readline().split();need(head[-1]!=b'missing','git missing '+p);b=proc.stdout.read(int(head[-1]));proc.stdout.read(1);need(hashlib.sha256(b).hexdigest()==h,'historical source '+p)
proc.stdin.close();proc.wait()
for n,h in f['tools'].items():need(sha(W/'bundle-sealed'/n)==h,'historical tool '+n)
r=load(S/'IMPLEMENTATION_RESULTS.json');need(r['tools']==f['tools'],'result tool set');rows={}
for p in N.glob('*/receipt.json'):
 j=load(p);d=p.parent;fr=load(d/'freeze.json');need(j['guards'],'last guard '+d.name)
 for n,k in [('freeze.json','freeze_sha256'),('stdout','stdout_sha256'),('stderr','stderr_sha256')]:need(sha(d/n)==j[k],'receipt '+str(p))
 need(fr['source']==f['source'],'receipt source '+d.name)
 for n,h in fr['controllers'].items():need(sha(N/'controller-snapshots'/(h+'.py'))==h,'controller snapshot')
 for path,h in (fr['fixtures']|fr.get('dependencies',{})).items():
  q=Path(path)
  # Completed namespace and frozen final runtime only; never follow historical checkout labels.
  if q.is_relative_to(N) or q.is_relative_to(W/'bundle-sealed'):need(sha(q)==h,'fixture/dependency '+path)
  elif (N/'controller-snapshots'/(h+'.py')).exists():need(sha(N/'controller-snapshots'/(h+'.py'))==h,'captured dependency controller')
  else:pass # inaccessible historical test executable label: record only, never read
 rows[d.name]=sha(p)
for label,entry in r['runs'].items():need(rows[label]==entry['receipt_sha256'],'reported receipt')
for p,h in r['actual_loaded_MIR']['seven_loaded_bodies'].items():need(sha(N/'loaded-mir'/p)==h,'MIR')
for label,row in r['export_refusals'].items():
 need(rows[label]==row['receipt_sha256'],'refusal receipt')
 for p in row['selection'].get('paths',row['selection'].get('removed',[])):need(not Path(p).exists(),'historical stale ordering output')
need(not (W/'counter-partial-only/wrapper.vir').exists(),'counterexample removed')
# Read every required committed document fully, retaining hash identities.
required=['IMPLEMENTATION_REPORT.md','IMPLEMENTATION_RESULTS.json','IMPLEMENTATION_VALIDATION.json','CURRENT_SEAL.json','PREREGISTRATION.md','INTERFACE.md','RESUMED_BASELINE.json','EXCLUDED_RUNS.json']
out=dict(historical_git='c4ddc83e',historical_source_files=len(f['source']),historical_receipts=rows,historical_tools=f['tools'],required_documents={n:sha(S/n) for n in required},counterexample=load(W/'partial-only-counterexample.json'),historical_source_manifest_sha256=hashlib.sha256(json.dumps(f['source'],sort_keys=True).encode()).hexdigest(),parent='d3b2aa6e63cbc440e09cd5764d99df94e315e959',publication=False,goal_complete=False)
(H/'HISTORICAL_AUTHENTICATION.json').write_text(json.dumps(out,indent=2,sort_keys=True)+'\n')
print('authenticated',len(f['source']),len(rows))
