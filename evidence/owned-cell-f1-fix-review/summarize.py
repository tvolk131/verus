#!/usr/bin/env python3
"""Compact committed summary; repeated raw files/hashes stay private."""
import sys
sys.dont_write_bytecode=True
import json,subprocess,datetime
from pathlib import Path
import recheck as r
import probes as p
from infer_eval_support import digest
from infer_experiment import require

def main():
 w=r.WORK;h=r.HERE
 receipts={n:json.loads((w/n/'receipt.json').read_text()) for n in ['focused','probes','compatibility','coverage']}
 for n,x in receipts.items():require(x['success'] and x['guard_passed'],n+' guard')
 require(not receipts['probes']['unexpected'],'independent unexpected')
 require(not receipts['compatibility']['unexpected'],'compatibility unexpected')
 focused=sorted((w/'runs').glob('*/receipt.json'));require(len(focused)==166,'166 focused invocations')
 for f in focused:require(json.loads(f.read_text())['success'],'focused wrapper')
 marker=[x for x in receipts['probes']['cases'] if x['expected']=='early']
 require(len(marker)==52 and all(x['ok'] and x['actual']=='early' and x['empty_inventory'] for x in marker),'independent marker refusals')
 for n in ['probes','coverage']:
  for c in receipts[n]['cases']:
   if c['flag']:
    require(c['exit']!=0 and c['empty_inventory'],'honest request refusal')
    require(not list(Path(c['dir']).glob('*.rlib')) and not list(Path(c['dir']).glob('*.vir')),'no direct export/output')
 cover=receipts['coverage']['cases']
 for label in ['reveal_group_fn','empty_broadcast']:
  for c in [x for x in cover if x['label']==label]:
   require(c['later'] and c['empty_inventory'] if c['flag'] else c['exit']==0 and c['result']['verified']==0,label)
   if label=='reveal_group_fn' and not c['flag']:
    require(c['scope']['native_functions'][0]['body_status']=='selected_item_without_translated_body','missing translated function')
 auth=json.loads((h/'AUTHENTICATION.json').read_text())
 for n,v in auth['tools'].items():
  require(digest(w/'bundle'/n)==v,'private tool stable '+n)
  require(digest(r.SNAP/n)==v,'frozen snapshot stable '+n)
 require(digest(r.SNAP/'receipt.json')==auth['receipt_sha256'],'frozen receipt stable')
 sources=['source/rust_verify/src/'+n for n in ['checked_owned_cell_admission.rs','checked_owned_cell.rs','rust_to_vir.rs','rust_to_vir_func.rs','rust_to_vir_global.rs','external.rs','attributes.rs','verifier.rs','infer_report.rs','fn_call_to_vir.rs']]
 sources += ['source/rust_verify_test/tests/'+n for n in ['local_checked_owned_cell.rs','local_checked_owned_cell_admission.rs']]
 sources += ['source/tools/infer_experiment.py','source/tools/infer_eval_support.py','source/tools/research/redb_crash_protocol/native_state_link/implementation/check.py','source/tools/research/redb_crash_protocol/native_state_link/owned_goals.rs']
 sources += ['source/tools/research/redb_crash_protocol/'+n for n in ['GOAL.md','CURRENT.md','WORKER_BRIEFS.md','native_state_link/ACCEPTANCE_REFINEMENTS.md','native_state_link/implementation/F1_FIX.md','native_state_link/implementation/F1_FIX_RESULTS.json']]
 import hashlib
 source_hash={}
 for name in sources:
  b=subprocess.check_output(['git','show',auth['identities']['HEAD']+':'+name],cwd=r.ROOT)
  require(hashlib.sha256(b).hexdigest()==digest(r.ROOT/name),'frozen source '+name)
  source_hash[name]=digest(r.ROOT/name)
 (h/'SOURCE_HASHES.json').write_text(json.dumps(source_hash,indent=2)+'\n')
 # Verify unaltered test oracles plus the two declared mechanical adaptations.
 for name in ['local_checked_owned_cell','local_checked_owned_cell_admission']:
  original=(r.ROOT/f'source/rust_verify_test/tests/{name}.rs').read_text()
  expected=original.replace('#![feature(rustc_private)]','').replace('"../../tools/research/redb_crash_protocol/native_state_link/owned_goals.rs"',json.dumps(str(r.ROOT/'source/tools/research/redb_crash_protocol/native_state_link/owned_goals.rs')))
  require((w/f'harness/tests/{name}.rs').read_text()==expected,'unchanged focused oracle')
 manifest={str(x.relative_to(w)):dict(sha256=digest(x),bytes=x.stat().st_size) for x in sorted(w.rglob('*')) if x.is_file() and not any(k in x.relative_to(w).parts for k in ['bundle','old-bundle','environment'])}
 (w/'RAW_HASHES.json').write_text(json.dumps(manifest,indent=2)+'\n')
 def ref(f):return dict(path=str(f),sha256=digest(f),bytes=f.stat().st_size)
 coverage=[]
 for x in cover:
  err=(Path(x['dir'])/'coverage.stderr').read_text()
  coverage.append({k:x[k] for k in ['label','infer','flag','exit','later','source_sha256']} | dict(verified=x['result'].get('verified'),checked_exec=x['scope']['checked_exec_bodies'],statuses=[f['body_status'] for f in x['scope']['native_functions']],excluded_panic='panicked at' in err,excluded_frontend=not x['flag'] and x['exit']!=0 and 'panicked at' not in err))
 costs={n:dict(logged_commands=len(x['commands']),seconds=sum(c['seconds'] for c in x['commands'])) for n,x in receipts.items()}
 costs['focused']['nested_verifier_seconds_not_added']=sum(json.loads(f.read_text())['commands'][0]['seconds'] for f in focused)
 costs['focused']['harness_compilation_seconds_nested']=5.89
 costs['wall_scope_start_utc']='2026-11-12T16:56:00Z'
 costs['checkpoint_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
 costs['active_upper_bound_seconds_including_all_build_setup_waits']=(datetime.datetime.now(datetime.timezone.utc)-datetime.datetime(2026,11,12,16,56,tzinfo=datetime.timezone.utc)).total_seconds()
 costs['timing_note']='Command sums are resource costs, not elapsed wall time: direct groups overlap; focused nested compiler/build times are not added again. No CPU/dollar meter. No compiler/vstd build.'
 res=dict(status='original_two_marker_finding_closed_new_F1_coverage_blocker',early_scope_commit='09dec7606',focused=dict(tests=10,invocations=166,passed=True,original_oracles_unchanged=True,new_marker_refusals=16,unmarked_false_contract_controls=2),independent=dict(invocations=84,classified=76,classification_failures=0,exploratory=8,marker_early_refusals=52,late_message_on_bad_markers=0),before_after=dict(old_invocations=24,default_off_comparisons=12,matches=12,old_feature_on_later=12,new_marker_early=8),coverage=dict(invocations=32,observations=coverage),total_verifier_invocations=306,receipts={n:ref(w/n/'receipt.json') for n in receipts},raw_manifest=ref(w/'RAW_HASHES.json'),source_manifest=ref(h/'SOURCE_HASHES.json'),controllers={f.name:digest(f) for f in sorted(h.glob('*.py')) if f.name!='summarize.py'},costs=costs,exclusions=['Sandbox startup failed before command execution; approved escalated commands used thereafter.','Six default-off coverage invocations panicked (structural wrapper +/- client and raw scalar const, both modes); no semantic/skip-success credit.','Two malformed size_of default-off cases and two false-contract reveal_group default-off cases are frontend errors, not semantic failures or successful skip witnesses.','Supporting imported fixture native rustc compile inside focused suite not a native proof or separate verifier invocation.'],vstd=dict(verified=2257,errors=0,inherited=True,rebuilt=False),R1_independently_executed=False,native=0,native_body_proofs=0,state_proofs=0,CP1=False,CP2=False,source_coupling_proved=False,stage_accepted=False,publication=False,goal_complete=False,hmac_native='DEFERRED AND UNACHIEVED')
 (h/'RESULTS.json').write_text(json.dumps(res,indent=2)+'\n')
 print('summary validated: 306 invocations; original finding closed, new F1 blocker; compact summary', (h/'RESULTS.json').stat().st_size,'bytes')
if __name__=='__main__':main()
