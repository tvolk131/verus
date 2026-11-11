#!/usr/bin/env python3
"""Focused S1 replay; uses existing execution, pin, stability and result utilities."""
import argparse, json, os, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_publication
from infer_eval_support import ArtifactGuard, digest

p=argparse.ArgumentParser()
p.add_argument('--bundle',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--baseline',action='store_true')
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
receipt={'kind':'S1_BOUNDED_LOCAL_CHECK_NOT_PARENT_ACCEPTANCE','baseline':a.baseline,'inputs':{},'commands':[],'outcomes':[]}
with replay_receipt(a.output,receipt):
 freeze=json.loads((HERE/'S1_FIXTURE_FREEZE.json').read_text())
 for n,h in freeze.items():require(digest(HERE/n)==h,'fixture freeze '+n)
 inputs=[HERE/n for n in freeze]+[Path(__file__),HERE/'compiler.rs']+[a.bundle/n for n in ['verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','z3']]
 guard=ArtifactGuard(inputs)
 receipt['inputs']={str(p):digest(p) for p in inputs}
 env=dict(os.environ,VERUS_Z3_PATH=str(a.bundle/'z3'),VERUS_INFER_CACHE=str(a.output/'cache'))
 run=LoggedCommands(a.output,a.output,receipt['commands'],timeout=180,guard=guard)
 def check(label,source,goals,failures,extra=(),crate=None,diagnostic='postcondition not satisfied',inventory=()):
  cmd=[str(a.bundle/'verus'),str(source),'--crate-type=lib','--edition=2024','--crate-name',crate or 's1_'+label.replace('-','_'),'--no-cheating','--output-json','--time','--multiple-errors','20']
  if goals:cmd+=['--infer-goals']
  cmd+=list(extra)
  source_guard=ArtifactGuard([source]);receipt['inputs'][str(source)]=digest(source)
  code=run(label,cmd,env)
  source_guard.check()
  stdout=(a.output/(label+'.stdout')).read_text();stderr=(a.output/(label+'.stderr')).read_text()
  data=validate_run({'returncode':code,'stdout':stdout,'stderr':stderr},not failures)
  require(data['verification-results'].get('is-verifying-entire-crate') is True,'full crate required')
  inf=data['verus-infer'];require(inf['verification_scope']['native_collection_completed'],'native collection required')
  if not failures:require(inf['verification_scope']['complete_selected_scope_verified'],'complete selected scope required')
  require_exact(inf.get('functions',[]));require_function_coverage(inf.get('functions',[]),inventory)
  actual={n.rsplit('::',1)[-1] for n in inf['failed_functions']}
  require(actual==set(failures),'wrong failed functions '+repr(actual))
  if failures:
   require(diagnostic in stderr,'expected semantic failure')
   require(not any(s in stderr for s in [s for s in ['error[E','precondition not satisfied','possible arithmetic underflow/overflow','assertion failed'] if s!=diagnostic]),'unhealthy negative')
  breakdown=[f for m in data['times-ms']['smt']['smt-run-module-times'] for f in m.get('function-breakdown',[])]
  require(all(f['success'] for f in breakdown if f['function'].rsplit('::',1)[-1] not in failures),'unhealthy dependency')
  item={'label':label,'verification':data['verification-results'],'failed_functions':sorted(actual),'inferred_helpers':inf.get('functions',[]),'set_pairs':inf.get('set_pair_assistance',[]),'scope':inf['verification_scope'],'times':data['times-ms'],'stdout_sha256':digest(a.output/(label+'.stdout')),'stderr_sha256':digest(a.output/(label+'.stderr'))}
  receipt['outcomes'].append(item);print(label,item['verification'],flush=True)
  return data
 for case,failed in [('r2',{'finish_mutation'}),('f2',{'quorum_intersection','quorum_has_honest','one_honest_margin'})]:
  source=HERE/(case+'-reduced.rs')
  check(case+'_off',source,False,failed)
  check(case+'_cold',source,True,failed if a.baseline else set())
  if not a.baseline:
   data=check(case+'_repeat',source,True,set())
   scope=data['verus-infer']['verification_scope']
   require((scope['checked_exec_bodies'],scope['checked_proof_bodies'])==((12,8) if case=='r2' else (8,3)),'original body inventory')
   counts=[x['pairs'] for x in data['verus-infer']['set_pair_assistance']]
   require(counts==([20] if case=='r2' else [20,6,6]),'exact pair inventory')
 if not a.baseline:
  # Imported callers consume unchanged ordinary contracts, not cached helpers.
  for case,crate in [('r2','r2_api'),('f2','f2_ordinary')]:
   text=(HERE/(case+'-reduced.rs')).read_text()
   if case=='r2':text+='\n'+(HERE.parent/'redb_tree/interface.rs').read_text()
   src=a.output/(case+'_api.rs');src.write_text(text)
   artifact=a.output/(case+'.vir');rlib=a.output/('lib'+case+'.rlib')
   data=check(case+'_export',src,True,set(),['--compile','--export',str(artifact),'-o',str(rlib)],crate=crate)
   require_publication(data,True,artifact)
   client=HERE.parent/('redb_tree/callers.rs' if case=='r2' else 'fedimint_quorum/client.rs')
   for goals in [False,True]:
    check(case+'_import_'+str(goals),client,goals,set(),['--extern',crate+'='+str(rlib),'--import',crate+'='+str(artifact)])
   original_artifact=artifact.read_bytes()
   require(original_artifact.count(b'candidate-bound-evidence-v77')==1,'policy trailer pin')
   wrong=a.output/(case+'_policy75.vir');wrong.write_bytes(original_artifact.replace(b'candidate-bound-evidence-v77',b'candidate-bound-evidence-v75'))
   label=case+'_version_rejection'
   code=run(label,[str(a.bundle/'verus'),str(client),'--crate-type=lib','--edition=2024','--no-cheating','--infer-goals','--output-json','--extern',crate+'='+str(rlib),'--import',crate+'='+str(wrong)],env)
   stdout=(a.output/(label+'.stdout')).read_text();stderr=(a.output/(label+'.stderr')).read_text()
   validate_run({'returncode':code,'stdout':stdout,'stderr':stderr},False,allow_frontend=True)
   require('incompatible inference schema, safety policy, or producer' in stderr,'version gate missing')
   receipt['outcomes'].append({'label':label,'classification':'EXPECTED_VERSION_FRONTEND_REJECTION','stderr_sha256':digest(a.output/(label+'.stderr'))})
   if case=='r2':
    src=a.output/'r2_controls.rs';src.write_text(text+'\n'+(HERE.parent/'redb_tree/controls.rs').read_text())
    for kind,name in [('leaf','wrong_set_membership'),('count','wrong_repeat_count'),('frame','wrong_clear_frame')]:
     stale=a.output/('negative_'+kind+'.vir');stale.write_bytes(artifact.read_bytes())
     data=check('r2_negative_'+kind,src,True,{name},['--cfg','negative_'+kind,'--export',str(stale)])
     require_publication(data,False,stale)
   else:
    for name,old,new,target in [
      ('intersection','ensures a.intersect(b).len() > fault(n),','ensures a.intersect(b).len() > fault(n) + 1,','quorum_intersection'),
      ('fault','a.len() >= quorum(n), bad.len() <= fault(n),','a.len() >= quorum(n), bad.len() <= quorum(n),','quorum_has_honest')]:
     require(text.count(old)==1,'exact negative edit')
     src=a.output/('f2_negative_'+name+'.rs');src.write_text(text.replace(old,new))
     stale=a.output/('f2_negative_'+name+'.vir');stale.write_bytes(artifact.read_bytes())
     data=check('f2_negative_'+name,src,True,{target},['--export',str(stale)])
     require_publication(data,False,stale)
  text=(HERE/'f2-reduced.rs').read_text()
  prefix=text[:text.index('verus! {\n// Semantic BFT policy')]
  old='#[verus_spec(r => requires self.count() > 0,\n        ensures r == fault(self.count()), 3 * r < self.count(), self.count() <= 3 * r + 3)]'
  require(prefix.count(old)==1,'exact zero-domain edit')
  src=a.output/'f2_zero.rs';src.write_text(prefix.replace(old,'#[verus_spec(r => ensures r == fault(self.count()))]')+'\n'+(HERE.parent/'fedimint_quorum/adapter.rs').read_text())
  check('f2_zero',src,True,{'max_evil'},diagnostic='possible arithmetic underflow/overflow')
  gate=a.output/'precision.rs';gate.write_text('use vstd::prelude::*;#[verus_infer] pub fn bump(x:u8)->u8{x+1} verus!{fn client(){let r=bump(254);assert(r==255);}}')
  check('precision',gate,True,set(),inventory=['bump'])
  gate=a.output/'safety.rs';gate.write_text('use vstd::prelude::*;#[verus_infer] pub fn bump(x:u8)->u8{x+1} verus!{fn unsafe_client(){let r=bump(255);}}')
  check('safety',gate,True,{'unsafe_client'},diagnostic='precondition not satisfied',inventory=['bump'])
  check('generic_relations',HERE/'compiler.rs',True,set())
  # The generic corpus's strict false assertions and precondition control are
  # separate so mixed failures cannot disguise a broken helper dependency.
  text=(HERE/'compiler.rs').read_text();src=a.output/'generic_false.rs'
  text=text[:text.index('#[cfg(strict_false)]\nfn false_caller')]+text[text.rindex('}') :]
  src.write_text(text)
  check('generic_false',src,True,{'false_cardinality','false_equality'},['--cfg','strict_false'])
  src=a.output/'generic_caller.rs';src.write_text('use vstd::prelude::*;verus!{proof fn false_caller(){vstd::set_lib::lemma_len_subset(set![1int],Set::<int>::empty());}}')
  check('generic_caller',src,True,{'false_caller'},diagnostic='precondition not satisfied')
 guard.check()
