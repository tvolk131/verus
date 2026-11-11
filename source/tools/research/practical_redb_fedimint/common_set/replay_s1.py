#!/usr/bin/env python3
"""Focused S1 replay; uses existing execution, pin, stability and result utilities."""
import argparse, json, os, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_publication
from infer_eval_support import ArtifactGuard, digest

R2_EXEC = [owner+'::'+name for owner,names in [
 ('BtreeBitmap',['clear','get_height','get_level','get_level_mut','set','update_to_root']),
 ('U64GroupedBitmap',['clear','data_index_of','get','len','select_mask','set'])] for name in names]
R2_PROOF = ['word_bits','word_frame','layer_edit','leaf_edit','clear_not_full','finish_mutation','propagation','ancestor_next']
F2_EXEC = ['<NumPeers as std::clone::Clone>::clone',
 '<NumPeers as std::convert::From<usize>>::from']+[
 'NumPeers::'+name for name in ['degree','max_evil','one_honest','threshold','total']]+['policy_client']
F2_PROOF = ['quorum_intersection','quorum_has_honest','one_honest_margin']
GENERIC_PROOF = ['subset_size','extensional','inclusion','unsupported_mutable']


def body_inventory(label):
 if label.startswith('r2_import_'):
  return ['free_membership','record_alloc_membership','repeat_alloc_membership'],[]
 if label.startswith('r2_'):
  ex=list(R2_EXEC)
  if label=='r2_export' or label.startswith('r2_negative_'):
   ex+=['CheckedBitmap::mark_allocated','CheckedBitmap::mark_free']
  if label.startswith('r2_negative_'):
   ex+=[{'leaf':'wrong_set_membership','count':'wrong_repeat_count','frame':'wrong_clear_frame'}[label.removeprefix('r2_negative_')]]
  return ex,R2_PROOF
 if label.startswith('f2_import_'):
  return ['admission_observation','boundary_observations'],[]
 if label=='f2_zero':
  return [name for name in F2_EXEC if name!='policy_client'],[]
 if label.startswith('f2_'):
  return F2_EXEC,F2_PROOF
 if label in ['precision','safety']:
  return ['bump','client' if label=='precision' else 'unsafe_client'],[]
 if label=='generic_relations':
  return [],GENERIC_PROOF
 if label=='generic_false':
  return [],GENERIC_PROOF+['false_cardinality','false_equality']
 if label=='generic_caller':
  return [],['false_caller']
 raise ValueError('unregistered replay scope '+label)


def require_native_scope(data, source, executable, proofs, failures, pairs):
 """Use full Rust identities, preserving duplicate method basenames and modes.

 Negative runs enumerate selected bodies without certifying them; the SMT
 breakdown must still show that only the intended functions failed.
 """
 inf=data['verus-infer'];scope=inf['verification_scope']
 require(scope['native_collection_completed'] is True,'native collection required')
 statuses={'checked_exec_body','checked_proof_body','selected_body_not_certified_by_successful_full_run'}
 all_bodies=scope['native_functions']
 require(not any(x['selected'] and x['body_status'] in {
  'unverified_body_with_contract','external_specification','interface_without_checked_body'} for x in all_bodies),
  'selected contract has no ordinary checked body')
 bodies=[x for x in all_bodies if x['body_status'] in statuses]
 require(len(set(executable+proofs))==len(executable)+len(proofs),'ambiguous expected body scope')
 require(sorted(x['rust_function'] for x in bodies)==sorted(executable+proofs),'exact native body inventory mismatch')
 require(all(x['selected'] is True and x['source'].startswith(str(source)+':') for x in bodies),
  'native body selection/source mismatch')
 by_function={x['function']:x['rust_function'] for x in bodies}
 require(len(by_function)==len(bodies),'duplicate backend function identity')
 failed=inf['failed_functions']
 require(len(set(failed))==len(failed) and all(x in by_function for x in failed),'invalid failed function identities')
 require(sorted(by_function[x] for x in failed)==sorted(failures),'wrong failed functions')
 if not failures:
  require(scope['complete_selected_scope_verified'] is True,'complete selected scope required')
  for kind,names in [('exec',executable),('proof',proofs)]:
   found=[x['rust_function'] for x in bodies if x['body_status']=='checked_'+kind+'_body']
   require(sorted(found)==sorted(names) and scope['checked_'+kind+'_bodies']==len(found),
    'checked body mode/count mismatch')
 else:
  require(scope['complete_selected_scope_verified'] is False,'failed scope incorrectly certified')
  require(all(x['body_status']=='selected_body_not_certified_by_successful_full_run' for x in bodies)
   and scope['checked_exec_bodies']==scope['checked_proof_bodies']==0,'negative scope incorrectly certified')
 breakdown=[f for m in data['times-ms']['smt']['smt-run-module-times'] for f in m.get('function-breakdown',[])]
 require({f['function'] for f in breakdown if not f['success']}==set(failed),
  'SMT failures disagree with intended function identities')
 entries=inf.get('set_pair_assistance',[])
 require(all(x['function'] in by_function for x in entries),'pair scope outside native body inventory')
 observed={by_function[x['function']]:x['pairs'] for x in entries}
 require(len(observed)==len(entries) and observed==pairs,'exact pair function inventory mismatch')


def expected_pairs(label, goals, baseline):
 if not goals or baseline:return {}
 if label.startswith('r2_') and not label.startswith('r2_import_'):
  return {'finish_mutation':20}
 if label.startswith('f2_') and label!='f2_zero' and not label.startswith('f2_import_'):
  return dict(zip(F2_PROOF,[20,6,6]))
 if label in ['generic_relations','generic_false']:
  pairs=dict(zip(GENERIC_PROOF[:3],[2,2,12]))
  if label=='generic_false':pairs.update(false_cardinality=2,false_equality=2)
  return pairs
 return {}


def main():
 p=argparse.ArgumentParser()
 p.add_argument('--bundle',type=Path,required=True)
 p.add_argument('--output',type=Path,required=True)
 p.add_argument('--baseline',action='store_true')
 p.add_argument('--policy',type=int,choices=[77,78],default=77)
 a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
 receipt={'kind':'S1_BOUNDED_LOCAL_CHECK_NOT_PARENT_ACCEPTANCE','baseline':a.baseline,'policy':a.policy,'inputs':{},'commands':[],'outcomes':[]}
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
   inf=data['verus-infer']
   require_exact(inf.get('functions',[]));require_function_coverage(inf.get('functions',[]),inventory)
   executable,proofs=body_inventory(label)
   require_native_scope(data,source,executable,proofs,failures,expected_pairs(label,goals,a.baseline))
   actual=set(failures)
   if failures:
    require(diagnostic in stderr,'expected semantic failure')
    require(not any(s in stderr for s in [s for s in ['error[E','precondition not satisfied','possible arithmetic underflow/overflow','assertion failed'] if s!=diagnostic]),'unhealthy negative')
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
    require(original_artifact.count(('candidate-bound-evidence-v'+str(a.policy)).encode())==1,'policy trailer pin')
    wrong=a.output/(case+'_policy75.vir');wrong.write_bytes(original_artifact.replace(('candidate-bound-evidence-v'+str(a.policy)).encode(),b'candidate-bound-evidence-v75'))
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
   check('f2_zero',src,True,{'NumPeers::max_evil'},diagnostic='possible arithmetic underflow/overflow')
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


if __name__=='__main__':
 main()
