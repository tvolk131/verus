"""Package only new review evidence; historical raw files are compared by hash after reproduction."""
from pathlib import Path
import hashlib, json, shutil, subprocess, sys
ROOT=Path(__file__).resolve().parents[2]; HERE=Path(__file__).resolve().parent
C=ROOT/'source/tools/research/redb_crash_protocol'
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import require, require_exact, require_failure_kind

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
labels=['selection','storage-positive','storage-negative','model-normal-positive','model-optimized-positive','model-normal-negative','model-optimized-negative','import-ordinary']
runs=[]; artifacts=[]
for label in labels:
 out=Path('/tmp/redb-new-review-03fd44ed-'+label)
 receipt=json.loads((out/'receipt.json').read_text())
 require(receipt['success'],'failed reproduction '+label)
 dest=HERE/'receipts'/label; dest.mkdir(parents=True,exist_ok=True)
 for p in sorted(out.rglob('*')):
  if not p.is_file(): continue
  rel=p.relative_to(out)
  artifacts.append(dict(path=str(p),sha256=sha(p),bytes=p.stat().st_size))
  # Text products only, excluding runtime/caches/binary exports and rlibs.
  if len(rel.parts)==1 and p.suffix in {'.json','.stdout','.stderr','.rs'}:
   shutil.copyfile(p,dest/rel)
 records=[]
 for x in receipt.get('outcomes',receipt.get('runs',[])):
  d=x.get('data'); n=x['label']
  if d is None and 'verification' not in x:
   require('failed_import' in n,'unexpected missing result')
   require(not (out/(n+'.vir')).exists(),'failed import retained seeded export')
   records.append(dict(label=n,expected='missing failed export refused'))
   continue
  v=d['verification-results'] if d else x['verification']
  inf=d['verus-infer'] if d else None
  scope=inf['verification_scope'] if inf else x['scope']
  helpers=inf['functions'] if inf else x['helpers']
  require(v['is-verifying-entire-crate'] and scope['native_collection_completed'],'full scope')
  require_exact(helpers)
  if label=='selection':
   positive='provider_' in n or n.endswith('_healthy')
   require(v['verified']==(23 if 'provider_' in n else 22 if n.endswith(('_wrong','_unsafe')) and '_client_' not in n else 1 if positive else 0),'selection count')
   require(v['errors']==(0 if positive else 1),'selection errors')
   if helpers: require(len(helpers)==1 and helpers[0]['function']=='selection_checked::TransactionId::eq','exact Eq target')
   if not positive:
    require_failure_kind((out/(n+'.stderr')).read_text(),'postcondition not satisfied' if n.endswith('_wrong') else 'precondition not satisfied')
    require(not (out/(n+'.vir')).exists(),'failed export survived')
  elif label.startswith('storage'):
   positive=label.endswith('positive'); require(v['verified']==(15 if positive else 14) and v['errors']==(0 if positive else 1),'backend count')
   require(not helpers,'backend helper')
   abstract=[b for b in scope['native_functions'] if b['body_status']=='interface_without_checked_body']
   require({b['rust_function'] for b in abstract}=={'StorageBackend::'+n for n in ['len','read','set_len','sync_data','write']},'backend interfaces')
   if not positive: require_failure_kind((out/(n+'.stderr')).read_text(),'postcondition not satisfied')
  elif label.startswith('model'):
   positive=label.endswith('positive'); require(v['verified']==26 and v['errors']==(0 if positive else 1),'model count')
   require(not helpers and scope['checked_exec_bodies']==0,'model exec/helpers')
   f=receipt['freeze']; require(f['parent']=='c008326bfed88eab958335412cdc5ec0e9c8130e','historical parent')
   if not positive: require_failure_kind((out/(n+'.stderr')).read_text(),'postcondition not satisfied')
  records.append(dict(label=n,verification=v,checked_exec_bodies=scope['checked_exec_bodies'],checked_proof_bodies=scope['checked_proof_bodies'],
                      helpers=helpers,full_scope=scope,
                      imported_inference_dependencies=inf['imported_inference_dependencies'] if inf else [],
                      optional_infer_goals='--infer-goals' in next(t['argv'] for t in receipt['commands'] if t['label']==n.split('_provider_no_optional_infer_client')[0] if label=='import-ordinary') if label=='import-ordinary' else
                        ('--infer-goals' in next(t['argv'] for t in receipt['commands'] if t['label']==n))))
 commands=receipt['commands']
 for t in commands:
  args=t['argv']; require('--no-cheating' in args,'no-cheating')
  require(not any(a.startswith(('--no-verify','--no-lifetime','--verify-function','--verify-only-module','--no-erasure')) for a in args),'restricted checking')
 runs.append(dict(label=label,path=str(out),receipt_sha256=sha(out/'receipt.json'),commands=len(commands),compiler_seconds=sum(t['seconds'] for t in commands),
                  outcomes=records,setup_seconds=receipt.get('setup_seconds'),total_seconds=receipt.get('total_seconds')))
# Hash comparisons only: performed after all fresh reproductions above succeeded.
historical=[]
for sub in ['selection_contract','storage_boundary']:
 results=json.loads((C/sub/'RESULTS.json').read_text())
 for run in results['runs']:
  for name,expected in run['raw_files'].items():
   p=Path(name); actual=sha(p) if p.is_file() else None
   historical.append(dict(path=name,expected_sha256=expected,actual_sha256=actual,matches=expected==actual,kind='historical hash comparison only'))
for run in json.loads((C/'model/REPLAY_FIX_RESULTS.json').read_text())['runs']:
 p=Path(run['path']); actual=sha(p) if p.is_file() else None
 historical.append(dict(path=str(p),expected_sha256=run['sha256'],actual_sha256=actual,matches=run['sha256']==actual,kind='historical hash comparison only'))
(HERE/'HISTORICAL_COMPARISON.json').write_text(json.dumps(historical,indent=2)+'\n')
(HERE/'PRIVATE_ARTIFACTS.json').write_text(json.dumps(artifacts,indent=2)+'\n')
result=dict(reviewed_parent='03fd44eddf5562645c79e5be89ab0e16f2706db1',
 recommendation='ACCEPT exact partial source components and replay controller correction with reporting qualifications',
 source_coupling_proved=False,goal_complete=False,CP1='UNPROVED',CP2='UNPROVED',HMAC_native='DEFERRED AND UNACHIEVED',
 runs=runs,compiler_seconds=sum(r['compiler_seconds'] for r in runs),
 historical_hash_comparisons=len(historical),historical_hash_mismatches=[x for x in historical if not x['matches']],
 qualifications=['Strict imported-client commands all enable optional --infer-goals; two additional ordinary clients without it pass 1/0.',
                 'Warm means cache-directory reuse only: every derived Eq reports cache miss; no cache-hit/speedup claim.'],
 retained_setup_failures=['Sandbox bwrap loopback RTM_NEWADDR failed before any shell command.',
                         'Initial scope commit lacked configured author identity; committed with existing parent identity using per-command settings.'])
(HERE/'RESULTS.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(runs=len(runs),commands=sum(r['commands'] for r in runs),compiler_seconds=result['compiler_seconds'],historical_comparisons=len(historical),historical_mismatches=len(result['historical_hash_mismatches'])),indent=2))
