import sys,json,hashlib,re,argparse
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--raw',type=Path,required=True);args=parser.parse_args()
root=Path(__file__).resolve().parents[2];out=args.raw.resolve();sys.path.insert(0,str(root/'source/tools'))
from infer_experiment import require,validate_goal_run,require_function_coverage,require_exact,validate_run,require_publication
from infer_eval_support import digest
obs=next((out/'observations').glob('*/receipt.json')); d=json.loads(obs.read_text());require(d['success'] is True,'observation replay incomplete')
labels=['prefix_cold','plain_cold','full_cold','prefix_warm','plain_warm','full_warm','prefix_disabled','plain_disabled','canonical_length','canonical_disabled','two_functions','wrong_value','wrong_length','wrong_current','wrong_index','wrong_ref','safety','custom','wrong_snapshot_state','mode_invalid_snapshot','ghost_executable_write','seed_prefix','remove_failed_full']
require([r['label'] for r in d['runs']]==labels,'observation inventory')
for name,info in d['artifacts'].items():require(digest(name)==info['sha256'],'observation artifact drift')
summary=[]
for r in d['runs']:
 label=r['label'];cmd=next(c for c in d['commands'] if c['label']==label);p=Path(cmd['argv'][1]);require(digest(p)==r['source_sha256'],'observation source drift')
 require('--no-cheating' in cmd['argv'] and not any(s in cmd['argv'] for s in ['--no-lifetime','--no-verify','--verify-root','--verify-function','--no-trait-conflicts']),'bypass')
 for ext in ['stdout','stderr']:require(digest(obs.parent/(label+'.'+ext))==r[ext+'_sha256'],'observation raw log drift')
 run=dict(returncode=cmd['returncode'],stdout=(obs.parent/(label+'.stdout')).read_text(),stderr=(obs.parent/(label+'.stderr')).read_text())
 v=json.loads(run['stdout']);helpers=v.get('verus-infer',{}).get('functions',[]);require_function_coverage(helpers,set());require_exact(helpers)
 if r['kind'] in ['success','postcondition']:
  validate_goal_run(run,r['kind']=='success',expected_functions=set(),exec_bodies=set(r['exec_bodies']),proof_bodies=set(r['proof_bodies']),failure_kind=None if r['kind']=='success' else 'postcondition not satisfied')
 else:
  validate_run(run,False,allow_frontend=r['kind']=='frontend')
  require((r['kind']=='frontend' and v['verification-results'].get('encountered-vir-error') is True) or (r['kind']!='frontend' and v['verification-results'].get('is-verifying-entire-crate') is True),'diagnostic category')
 if label=='remove_failed_full':require_publication(v,False,obs.parent/'seeded.vir')
 summary.append(dict(label=label,classification=r['kind'],verification=v['verification-results'],helpers=[],exec_bodies=r['exec_bodies'],proof_bodies=r['proof_bodies'],source_sha256=r['source_sha256']))
(out/'observation-audit.json').write_text(json.dumps(dict(success=True,case_count=len(summary),raw_receipt=str(obs),sha256=digest(obs),runs=summary),indent=2)+'\n')
can=next((out/'canonical').glob('*/receipt.json')); c=json.loads(can.read_text());require(c['success'] is True and len(c['runs'])==26,'canonical inventory')
provider=can.parent/'provider.vir'; b=provider.read_bytes();require(b.endswith(b'VRINFER1'),'missing real trailer'); start=b.rindex(b'{"schema":');meta=json.loads(b[start:-8]);require((meta['schema'],meta['policy'])==(38,'candidate-bound-evidence-v68'),'schema/policy')
require(meta['soundness'] is True and meta['maximality'] is True and meta['exactness'] is True,'proof guarantees');require(len(meta['proofs'])==1 and meta['trusted_models'],'helper/model inventory')
require(meta['proofs'][0][0]['path']['segments']==['identity'] and all(meta['proofs'][0][1][k] is True for k in ['soundness','maximality','exactness']),'published helper identity/status')
consumer=json.loads((can.parent/'consumer.stdout').read_text());deps=consumer['verus-infer']['imported_inference_dependencies'];require(len(deps)==1 and deps[0]['fingerprint']==hashlib.sha256(b[start:-8]).hexdigest(),'actual provider/dependency binding')
(out/'artifact-audit.json').write_text(json.dumps(dict(success=True,provider_sha256=digest(provider),rlib_sha256=digest(can.parent/'libprovider.rlib'),metadata_sha256=hashlib.sha256(b[start:-8]).hexdigest(),schema=meta['schema'],policy=meta['policy'],producer=meta['producer'],proofs=meta['proofs'],summary_binding=meta['summaries'],trusted_model_count=len(meta['trusted_models']),consumer_dependencies=deps,consumer_scope=consumer['verus-infer']['verification_scope']),indent=2)+'\n')
print('independent audit: 23 observation cases; provider/rlib/trailer/helper/model/dependency binding passed')
