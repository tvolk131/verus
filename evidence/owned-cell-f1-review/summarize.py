#!/usr/bin/env python3
import sys
sys.dont_write_bytecode=True
import json,time,datetime
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest

def normalize(value):
 if isinstance(value,dict):return {k:normalize(v) for k,v in value.items() if k!='source'}
 if isinstance(value,list):return [normalize(v) for v in value]
 return value

def main():
 comp=json.loads((HERE/'compatibility/receipt.json').read_text())
 compared=[dict(label=c['label'],infer=c['infer'],equal_after_removing_only_source_paths=normalize(c['observed'][0])==normalize(c['observed'][1]),observed=normalize(c['observed'])) for c in comp['cases']]
 (HERE/'compatibility/normalized.json').write_text(json.dumps(dict(normalization='Remove only per-function source path labels. Preserve exit/result/verification scope/body count/function inventory and diagnostic category.',cases=compared),indent=2)+'\n')
 source_paths=['source/rust_verify/src/'+n+'.rs' for n in ['checked_owned_cell_admission','checked_owned_cell','fn_call_to_vir','verifier','context','config','attributes','verus_items','rust_to_vir']]
 source_paths+=['source/rust_verify_test/tests/'+n+'.rs' for n in ['local_checked_owned_cell_admission','local_checked_owned_cell']]
 base='source/tools/research/redb_crash_protocol/'
 source_paths += [base+n+'.md' for n in ['GOAL','CURRENT','WORKER_BRIEFS']]
 ns=base+'native_state_link/'
 source_paths += [ns+'ACCEPTANCE_REFINEMENTS.md',ns+'design/IMPLEMENTATION_TARGET.md',ns+'design/ACCEPTANCE.md',ns+'owned_goals.rs']
 source_paths += [ns+'implementation/'+n for n in ['F1_REPORT.md','F1_RESULTS.json','F1_INTERFACE.md','f1.py','check.py']]
 source_paths += ['source/tools/'+n+'.py' for n in ['infer_experiment','infer_eval_support']]
 source_paths += ['source/builtin/src/lib.rs','source/builtin_macros/src/syntax.rs','source/rust_verify_test/tests/common/infer.rs','source/rust_verify_test/tests/common/mod.rs','source/rust_verify_test/Cargo.toml','source/Cargo.lock','source/Cargo.toml']
 sh={n:digest(ROOT/n) for n in source_paths};(HERE/'SOURCE_HASHES.json').write_text(json.dumps(sh,indent=2)+'\n')
 worker=json.loads((ROOT/(ns+'implementation/F1_RESULTS.json')).read_text())
 assert all(sh[n]==h for n,h in worker['source_sha256'].items())
 auth=json.loads((HERE/'AUTHENTICATION.json').read_text());s=Path(auth['snapshot']);g=ArtifactGuard([s/'receipt.json',*[s/n for n in auth['tools']]])
 assert g.initial==auth['guard'],'immutable F1 snapshot changed since first authentication'
 b=HERE/'private/bundle'
 assert all(digest(b/n)==h for n,h in auth['tools'].items())
 final=dict(snapshot_guard_unchanged=True,private_F1_tools_unchanged=True,receipt_sha256=digest(s/'receipt.json'),utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()))
 (HERE/'FINAL_AUTHENTICATION.json').write_text(json.dumps(final,indent=2)+'\n')
 original=[json.loads(p.read_text()) for p in (HERE/'runs').glob('*/receipt.json')]
 assert len(original)==148
 indep=json.loads((HERE/'independent-2/receipt.json').read_text());follow=json.loads((HERE/'followup/receipt.json').read_text());confirm=json.loads((HERE/'confirmation/receipt.json').read_text())
 for sub in ['independent-2','followup']:
  assert not list((HERE/sub).rglob('*.rlib')) and not list((HERE/sub).rglob('*.vir'))
 for c in confirm['cases']:
  if c['flag']:
   d=HERE/'confirmation'/f"{c['label']}-{c['infer']}-{c['flag']}"
   j=json.loads((d/(c['label']+'.stdout')).read_text());vi=j['verus-infer']
   assert j['verification-results']['success'] is False and j['func-details']=={} and vi['functions']==[] and vi['requested_functions']==0 and vi['generated_functions']==0 and vi['verification_scope']['checked_exec_bodies']==0 and vi['verification_scope']['checked_proof_bodies']==0 and vi['interface_written'] is False
   assert not list(d.glob('*.rlib'))
 costs={}
 for label in ['focused','focused-2','independent','independent-2','followup','confirmation','compatibility']:
  r=json.loads((HERE/label/'receipt.json').read_text());costs[label]=dict(commands=len(r['commands']),command_wall_seconds=sum(c['seconds'] for c in r['commands']),success=r['success'])
 costs['original_148_nested_compiler_calls']=dict(commands=148,command_wall_seconds=sum(r['commands'][0]['seconds'] for r in original),note='Nested inside focused-2; do not add twice.')
 freeze=time.time();start=datetime.datetime(2026,11,12,16,23,tzinfo=datetime.timezone.utc).timestamp()
 result=dict(kind='INDEPENDENT_FROZEN_F1_REVIEW_REQUEST_CHANGES',prepared_root='f95473bb8376e9f5ef05885c2d18d276b2363b1b',implementation_checkpoint='2cb432c6',early_interface_commit='26682715',finding='F1-REVIEW-1: source internal const_body/reveal_fn and ancestor skip markers pass admission',scoped_recommendation='Request changes to F1; retain stage inactivity; no production fix in this review',independently_replayed=dict(F1_original_invocations=116,inherited_invocations=32,supporting_imported_fixture_native_compiles=1,independent_second_attempt=84,independent_typed_credit_second_attempt=82,nested_const_fixture_corrections_in_followup=True,followup_invocations=18,confirmation_invocations=24,default_off_differential_invocations=32,default_off_comparisons=16,default_off_comparisons_equal=sum(c['equal_after_removing_only_source_paths'] for c in compared)),compatibility_compared_fields='Exit, verification results, scope/body status/counts, function names and postcondition/overflow diagnostic categories; source path labels removed only',original_frozen_inventory=dict(functions=3,bodies=6,headers=3),native_MIR_corruption_controls_independently_rerun=0,native_signature_controls_independently_rerun=0,worker_native_controls_source_inspected=dict(MIR=54,signature=4),F1_generated_native_functions=0,F1_native_body_proofs=0,F1_checked_exec_bodies=0,F1_checked_proof_bodies=0,F1_interfaces_written=False,feature_active=False,full_stage_accepted=False,F2='ABSENT',F3='ABSENT',authority='ABSENT',native_body_translation='ABSENT',correspondence='ABSENT',state_query='ABSENT',CP1=False,CP2=False,source_coupling_proved=False,hmac_native='DEFERRED AND UNACHIEVED',vstd='2257/0 inherited; not rebuilt or independently reverified',costs=costs,focused_harness_dependency_build_wall_seconds=8.59,compiler_or_vstd_builds=0,conservative_start_utc='2026-11-12T16:23:00Z',freeze_utc=datetime.datetime.fromtimestamp(freeze,datetime.timezone.utc).isoformat(),conservative_active_seconds_upper_bound_without_build_subtraction=freeze-start,build_excluded_seconds=8.59,monetary_meter_available=False,excluded_attempts=dict(initial_independent_wrong_VERUS_ROOT=84,first_focused_missing_cached_serde_json_version=True,independent_missing_static_lifetime=2),authentication=final)
 assert result['conservative_active_seconds_upper_bound_without_build_subtraction']<2700
 (HERE/'RESULTS.json').write_text(json.dumps(result,indent=2)+'\n')
 # Hash all review text/receipts/sources, excluding private tools/build/caches and this manifest itself.
 logs={str(p.relative_to(HERE)):digest(p) for p in sorted(HERE.rglob('*')) if p.is_file() and p.name!='LOG_HASHES.json' and 'private' not in p.relative_to(HERE).parts and p.suffix not in ['.rlib','.rmeta'] and '__pycache__' not in p.parts}
 (HERE/'LOG_HASHES.json').write_text(json.dumps(logs,indent=2)+'\n')
 print(json.dumps(dict(compatibility_equal=result['independently_replayed']['default_off_comparisons_equal'],total_comparisons=16,active_upper_bound_seconds=result['conservative_active_seconds_upper_bound_without_build_subtraction'],costs=costs),indent=2))
if __name__=='__main__':main()
