#!/usr/bin/env python3
"""Versioned semantic-stage use of the existing R3 replay and execution helpers."""
import argparse, importlib.util, json, os, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_publication
from infer_eval_support import ArtifactGuard, digest
spec=importlib.util.spec_from_file_location('r3_v1',HERE/'replay.py');v1=importlib.util.module_from_spec(spec);spec.loader.exec_module(v1)

def main():
 p=argparse.ArgumentParser();p.add_argument('--output',required=True,type=Path);p.add_argument('--full',action='store_true');p.add_argument('--free',action='store_true');p.add_argument('--ordinary-only',action='store_true');p.add_argument('--controls',action='store_true');a=p.parse_args()
 freeze=json.loads((HERE/'FREEZE_V2.json').read_text());bundle=Path(freeze['private_runtime']);out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
 scope_helper=v1.load_replay(HERE.parents[1]/'practical_redb_fedimint/common_set/replay_s1.py','r3_semantic_scope')
 receipt={'kind':'R3_SEMANTIC_V2_NOT_PARENT_ACCEPTANCE','full':a.full,'free':a.free,'commands':[],'outcomes':[]}
 with replay_receipt(out,receipt):
  require(digest(bundle/'receipt.json')==freeze['runtime_receipt'],'runtime receipt')
  for n,h in freeze['runtime_tools'].items():require(digest(bundle/n)==h,'tool '+n)
  for n,h in freeze['inherited'].items():require(digest(HERE/n)==h,'immutable first checkpoint '+n)
  receipt['v1_fidelity']=v1.fidelity(json.loads((HERE/'FREEZE.json').read_text()),out)
  src=out/'semantic_v2.rs';src.write_text('\n'.join((HERE/n).read_text() for n in ['manual.rs','domain.rs','checks.rs','semantic_v2.rs','controls_v2.rs','cases_v2.rs']))
  guard=ArtifactGuard([bundle/n for n in v1.TOOLS]+[src,HERE/'semantic_v2.rs',HERE/'controls_v2.rs',HERE/'cases_v2.rs',HERE/'FREEZE_V2.json',HERE/'RUN_FREEZE_V2.json',Path(__file__)]+[HERE/n for n in freeze['inherited']])
  receipt['source_sha256']=digest(src);run=LoggedCommands(out,out,receipt['commands'],timeout=180,guard=guard)
  for goals,cfg in ([(False,None)] if a.ordinary_only else [(False,None),(True,None)])+([(True,'v2_negative_effect'),(True,'v2_negative_merge'),(True,'v2_negative_domain')]+([(True,'v2_negative_alloc_accept'),(True,'v2_negative_alloc_effect'),(True,'v2_negative_overlap')] if a.full else []) if a.controls else []):
   label=('infer' if goals else 'ordinary')+('_'+cfg if cfg else '');env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ.get('PATH',''),RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/(label+'_cache')))
   cmd=[str(bundle/'verus'),str(src),'--crate-type=lib','--edition=2024','--crate-name','r3_semantic_v2','--no-cheating','--output-json','--time','--multiple-errors','20']+(['--infer-goals'] if goals else [])+(['--cfg','full_semantics'] if a.full else ['--cfg','free_trial'] if a.free else [])
   artifact=out/(label+'.vir')
   if a.controls:
    cmd+=['--export',str(artifact)]
    if cfg:
     import shutil
     shutil.copy2(out/'ordinary.vir',artifact);cmd+=['--cfg',cfg]
   code=run(label,cmd,env);raw={'returncode':code,'stdout':(out/(label+'.stdout')).read_text(),'stderr':(out/(label+'.stderr')).read_text()}
   try:data=json.loads(raw['stdout']);result=data['verification-results']
   except ValueError:receipt['outcomes'].append({'label':label,'classification':'SETUP_FAILURE'});continue
   classification='RESOURCE_FAILURE' if any(x in raw['stderr'].lower() for x in ['resource limit','timed out','panicked at']) else 'FRONTEND_FAILURE' if result.get('encountered-vir-error') or 'error[E' in raw['stderr'] or 'success' not in result else 'CHECKED' if code==0 else 'VERIFICATION_FAILURE'
   if classification in ['CHECKED','VERIFICATION_FAILURE']:
    validate_run(raw,code==0);require(result['is-verifying-entire-crate'],'full ordinary checks');require_exact(data['verus-infer']['functions']);require_function_coverage(data['verus-infer']['functions'],[])
   if a.controls and classification in ['CHECKED','VERIFICATION_FAILURE']:
    inventory=json.loads((HERE/'RUN_FREEZE_V2.json').read_text())
    executable=inventory['v1_exec']+(['free_semantic'] if a.free or a.full else [])+(['record_alloc_semantic','semantic_roundtrip'] if a.full else [])
    proofs=inventory['v1_proof']+[x for x in inventory['v2_proof'] if x!='alloc_interpret' or a.full]
    if not (a.free or a.full):proofs.remove('free_interpret')
    expected={'v2_negative_effect':'wrong_free_effect','v2_negative_merge':'wrong_free_merge','v2_negative_domain':'wrong_free_domain','v2_negative_alloc_accept':'wrong_alloc_accept','v2_negative_alloc_effect':'wrong_alloc_effect','v2_negative_overlap':'wrong_overlap'}
    if cfg:(proofs if cfg=='v2_negative_overlap' else executable).append(expected[cfg])
    inf=data['verus-infer'];native={x['function']:x['rust_function'] for x in inf['verification_scope']['native_functions']}
    measured_pairs={native[x['function']]:x['pairs'] for x in inf['set_pair_assistance']}
    scope_helper.require_native_scope(data,src,executable,proofs,[expected[cfg]] if cfg else [],measured_pairs)
    frames=inf['word_sequence_assistance']
    require([native[x['function']] for x in frames]==(['U64GroupedBitmap::set'] if goals else []),'word-template exact inventory')
    require(all(x['snapshots']==x['quantified_blocks']==x['library_calls']==1 for x in frames),'word-template costs')
    if cfg:
     diagnostic='precondition not satisfied' if cfg=='v2_negative_domain' else 'assertion failed'
     require(diagnostic in raw['stderr'],'intended negative diagnostic')
     require(not any(x in raw['stderr'] for x in ['postcondition not satisfied','error[E']),'unhealthy control')
    require((code==0)==(cfg is None),'wrong control outcome')
    if goals:require_publication(data,cfg is None,artifact)
    else:require(artifact.exists(),'ordinary export')
    native={x['function']:x['rust_function'] for x in data['verus-infer']['verification_scope']['native_functions']}
    expected={'v2_negative_effect':'wrong_free_effect','v2_negative_merge':'wrong_free_merge','v2_negative_domain':'wrong_free_domain','v2_negative_alloc_accept':'wrong_alloc_accept','v2_negative_alloc_effect':'wrong_alloc_effect','v2_negative_overlap':'wrong_overlap'}
    require([native[x] for x in data['verus-infer']['failed_functions']]==([expected[cfg]] if cfg else []),'wrong failing control body')
   receipt['outcomes'].append({'label':label,'classification':classification,'verification':result,'infer':data.get('verus-infer'),'times':data.get('times-ms'),'stdout_sha256':digest(out/(label+'.stdout')),'stderr_sha256':digest(out/(label+'.stderr'))})
   print(label,classification,result,flush=True)
  guard.check()
  if a.controls:require(all(x['classification']==('VERIFICATION_FAILURE' if 'negative_' in x['label'] else 'CHECKED') for x in receipt['outcomes']),'control replay includes setup/frontend/resource failure')
if __name__=='__main__':main()
