"""Independently bind observation and actual provider evidence; Python -B -O."""
import argparse,sys,json,hashlib,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import require,validate_goal_run,validate_run,require_exact,require_function_coverage,require_publication
from infer_eval_support import digest
p=argparse.ArgumentParser();p.add_argument('--raw',type=Path,required=True);a=p.parse_args();raw=a.raw.resolve()
obs=next((raw/'observations').glob('*/receipt.json'));d=json.loads(obs.read_text());require(d['success'] is True,'incomplete observations')
# Explicit independent identities and classes, including zero inference helpers.
expect={
 'prefix_cold':('success',{'xor_key'},set()),'plain_cold':('success',{'plain'},set()),'full_cold':('postcondition',{'xor_key'},set()),
 'prefix_warm':('success',{'xor_key'},set()),'plain_warm':('success',{'plain'},set()),'full_warm':('postcondition',{'xor_key'},set()),
 'prefix_disabled':('postcondition',{'xor_key'},set()),'plain_disabled':('success',{'plain'},set()),
 'canonical_length':('success',{'u64'},{'radix_positive','quotient_step','byte_step'}),
 'canonical_disabled':('success',{'u64'},{'radix_positive','quotient_step','byte_step'}),
 'two_functions':('success',{'plain','second'},set()),'wrong_value':('postcondition',{'xor_key'},set()),
 'wrong_length':('postcondition',{'xor_key'},set()),'wrong_current':('invariant',{'xor_key'},set()),
 'wrong_index':('invariant',{'xor_key'},set()),'wrong_ref':('invariant',{'xor_key'},set()),
 'safety':('safety',{'overflow'},set()),'custom':('postcondition',{'iter__mut','client'},set()),
 'wrong_snapshot_state':('invariant',{'xor_key'},set()),'mode_invalid_snapshot':('frontend',{'xor_key'},set()),
 'ghost_executable_write':('frontend',{'xor_key'},set()),'seed_prefix':('success',{'xor_key'},set()),
 'remove_failed_full':('postcondition',{'xor_key'},set())}
require([r['label'] for r in d['runs']]==list(expect),'observation exact inventory/order')
for path,info in d['artifacts'].items():require(digest(path)==info['sha256'],'artifact changed')
summary=[]
for r in d['runs']:
 label=r['label'];kind,execs,proofs=expect[label];cmd=next(c for c in d['commands'] if c['label']==label)
 require(digest(cmd['argv'][1])==r['source_sha256'],'source drift')
 require('--no-cheating' in cmd['argv'] and not any(x in cmd['argv'] for x in ['--no-lifetime','--no-verify','--verify-root','--verify-function','--no-trait-conflicts','--no-erasure-check']),'bypass')
 for ext in ['stdout','stderr']:require(digest(obs.parent/(label+'.'+ext))==r[ext+'_sha256'],'log drift')
 run=dict(returncode=cmd['returncode'],stdout=(obs.parent/(label+'.stdout')).read_text(),stderr=(obs.parent/(label+'.stderr')).read_text());data=json.loads(run['stdout']);info=data['verus-infer'];v=data['verification-results']
 require_function_coverage(info['functions'],set());require_exact(info['functions'])
 if kind in ['success','postcondition']:
  validate_goal_run(run,kind=='success',expected_functions=set(),exec_bodies=execs,proof_bodies=proofs,failure_kind='postcondition not satisfied' if kind=='postcondition' else None)
 else:
  validate_run(run,False,allow_frontend=kind=='frontend')
  if kind=='frontend':require(v.get('encountered-vir-error') is True and v.get('errors',0)==0,'frontend class')
  else:
   require(v['is-verifying-entire-crate'] is True and not v.get('encountered-vir-error') and 'error[E' not in run['stderr'],'incomplete semantic diagnostic')
   require_function_coverage([b for b in info['verification_scope']['native_functions'] if b['body_status']=='selected_body_not_certified_by_successful_full_run'],execs|proofs)
   require(('invariant not satisfied' if kind=='invariant' else 'possible arithmetic underflow/overflow') in run['stderr'],'diagnostic class')
 if label=='remove_failed_full':require_publication(data,False,obs.parent/'seeded.vir')
 summary.append(dict(label=label,classification=kind,execs=sorted(execs),proofs=sorted(proofs),helpers=[],verification=v))
can=next((raw/'canonical').glob('*/receipt.json'));provider=can.parent/'provider.vir';b=provider.read_bytes();require(b.endswith(b'VRINFER1'),'missing actual trailer');start=b.rindex(b'{"schema":');meta=json.loads(b[start:-8])
require(meta['producer'].endswith(':c9d072adba4f7c70a48d63768ddf203a39d15103'),'runtime producer binding')
require((meta['schema'],meta['policy'])==(38,'candidate-bound-evidence-v68'),'changed schema/policy')
require(all(meta[k] is True for k in ['soundness','maximality','exactness']),'export proof guarantees')
require(len(meta['trusted_models'])==359,'trusted model inventory')
require(len(meta['proofs'])==1 and meta['proofs'][0][0]['path']['segments']==['identity'],'helper identity')
require(all(meta['proofs'][0][1][k] is True for k in ['soundness','maximality','exactness']),'helper statuses')
consumer=json.loads((can.parent/'consumer.stdout').read_text());deps=consumer['verus-infer']['imported_inference_dependencies'];require(len(deps)==1 and deps[0]['fingerprint']==hashlib.sha256(b[start:-8]).hexdigest(),'provider identity binding')
probes=json.loads((raw/'probes/results.json').read_text())
require(len(probes['runs'])==36,'complete private probe inventory')
for path,info in probes['artifacts'].items():require(digest(path)==info['sha256'],'probe input drift')
for case in json.loads((raw/'probes/freeze.json').read_text()):
 private=Path(case['source']);committed=Path(case['committed_source'])
 require(private.is_relative_to(raw/'probes/sources') and not private.is_symlink() and private.stat().st_ino!=committed.stat().st_ino,'private source copy')
 require(digest(private)==digest(committed)==case['source_sha256'],'private/committed source binding')
residual=['move-let-after','pattern-ref-before','pattern-ref-after','match-move-after']
for label in residual:
 for on in [False,True]:
  name=label+('-on' if on else '-off');r=next(r for r in probes['runs'] if r['label']==name);cmd=next(c for c in probes['commands'] if c['label']==name)
  require(digest(cmd['argv'][1])==r['source_sha256'],'probe source drift')
  for ext in ['stdout','stderr']:require(digest(raw/'probes'/(name+'.'+ext))==r[ext+'_sha256'],'probe log drift')
  run=dict(returncode=cmd['returncode'],stdout=(raw/'probes'/(name+'.stdout')).read_text(),stderr=(raw/'probes'/(name+'.stderr')).read_text())
  data=validate_run(run,not on);v=data['verification-results'];info=data['verus-infer']
  require_function_coverage(info['functions'],set());require_exact(info['functions'])
  require(v.get('is-verifying-entire-crate') is True and (v['verified'],v['errors'])==((1,1) if on else (2,0)),'residual counts or scope')
  require(not v.get('encountered-vir-error') and 'error[E' not in run['stderr'],'residual is frontend')
  bodies=[b for b in info['verification_scope']['native_functions'] if b['body_status']==('selected_body_not_certified_by_successful_full_run' if on else 'checked_exec_body')]
  require_function_coverage(bodies,{'client'})
  require(r['selection']==(['client'] if on else []),'residual selection')
  if on:
   require('invariant not satisfied before loop' in run['stderr'],'residual diagnostic')
   require(not any(x in run['stderr'] for x in ['postcondition not satisfied','possible arithmetic underflow/overflow','recommendation not met']),'mixed residual diagnostic')
result=dict(success=True,private_probe_results_sha256=digest(raw/'probes/results.json'),residual_valid_program_rejections=residual,observation_cases=summary,observation_receipt_sha256=digest(obs),canonical_receipt_sha256=digest(can),provider=dict(sha256=digest(provider),rlib_sha256=digest(can.parent/'libprovider.rlib'),metadata_sha256=hashlib.sha256(b[start:-8]).hexdigest(),producer=meta['producer'],proofs=meta['proofs'],trusted_model_count=len(meta['trusted_models']),dependencies=deps))
(raw/'independent-audit.json').write_text(json.dumps(result,indent=2)+'\n');print('23 observation outcomes, explicit bodies and zero helpers; actual provider/trailer and identity statuses verified')
