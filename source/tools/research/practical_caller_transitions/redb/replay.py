#!/usr/bin/env python3
"""Small R3 replay using the existing experiment/measurement helpers."""
import argparse, importlib.util, json, os, shutil, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_publication
from infer_eval_support import ArtifactGuard, digest

TOOLS = ['verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','z3']
R2_EXEC = [owner+'::'+name for owner,names in [
 ('BtreeBitmap',['clear','get_height','get_level','get_level_mut','set','update_to_root']),
 ('U64GroupedBitmap',['clear','data_index_of','get','len','select_mask','set'])] for name in names]
R2_PROOF = ['layer_edit','leaf_edit','clear_not_full','finish_mutation','propagation','ancestor_next']
CALLERS = ['BtreeBitmap::get','BtreeBitmap::len','next_higher_order','buddy_page',
 'BuddyAllocator::get_order_free_mut','BuddyAllocator::record_alloc_inner','BuddyAllocator::free_inner']


def load_replay(path, name):
 spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module


def fidelity(freeze, output):
 prior=HERE.parents[1]/'practical_redb_fedimint'
 r2=load_replay(prior/'redb_tree/replay.py','r2_fidelity')
 # Reuse the accepted projection and metadata against the authenticated pinned
 # source, never the historical mutable private output.
 meta=json.loads((prior/'redb_tree/private.json').read_text());meta['private_artifacts']=str(output)
 (output/'source').mkdir();shutil.copy2(Path(freeze['project_root'])/'src/tree_store/page_store/bitmap.rs',output/'source/bitmap.rs')
 result=r2.fidelity(meta,HERE/'manual.rs')
 text=(HERE/'manual.rs').read_text()
 for name,item in freeze['new_original_items'].items():
  start=text.index('fn '+name.split('::')[-1]+'(',text.index('struct BuddyAllocator'))
  begin=text.index('\n{',start)+1;_,end=r2.block(text,begin)
  require(r2.projection(text[begin:end])==r2.tokens(item['text'][item['text'].index('{'):]),'new original executable body '+name)
  result[name]=item['sha256']
 return result

def main():
 p=argparse.ArgumentParser();p.add_argument('--bundle',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
 p.add_argument('--phase',choices=['baseline','full','manual'],default='baseline');a=p.parse_args()
 a.bundle=a.bundle.resolve();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
 scope_helper=load_replay(HERE.parents[1]/'practical_redb_fedimint/common_set/replay_s1.py','r2_scope')
 receipt={'kind':'R3_LOCAL_CHECKPOINT_NOT_PARENT_ACCEPTANCE','phase':a.phase,'commands':[],'outcomes':[]}
 with replay_receipt(a.output,receipt):
  freeze=json.loads((HERE/'FREEZE.json').read_text())
  require(digest(a.bundle/'receipt.json')==freeze['runtime_receipt'],'receipt hash')
  for n,h in freeze['runtime_tools'].items():require(digest(a.bundle/n)==h,'tool hash '+n)
  require((a.bundle/'verus-root').is_file(),'runtime marker')
  require(digest(HERE/'baseline.rs')==freeze['baseline_sha256'],'frozen baseline')
  for n,h in freeze['source_files'].items():require(digest(n)==h,'pinned source '+n)
  for n,h in freeze['reused'].items():require(digest(HERE.parents[4]/n)==h,'accepted provider '+n)
  guard=ArtifactGuard([a.bundle/n for n in TOOLS]+[Path(__file__),HERE/'FREEZE.json',HERE/'manual.rs',HERE/'domain.rs',HERE/'checks.rs',HERE/'corrected.rs']+[Path(n) for n in freeze['source_files']])
  run=LoggedCommands(a.output,a.output,receipt['commands'],timeout=180,guard=guard)
  prior=HERE.parents[1]/'practical_redb_fedimint'
  sources=[('combined',prior/'common_set/r2-combined.rs'),('provider',prior/'redb_tree/manual.rs'),('caller',HERE/'corrected.rs')] if a.phase=='baseline' else [('caller',HERE/'corrected.rs')] if a.phase=='full' else [('manual',HERE/'manual.rs')]
  if a.phase=='manual':receipt['source_fidelity']=fidelity(freeze,a.output)
  for label,source in sources:
   for goals,cfg in [(False,None),(True,None)]+([(True,'negative_rejection'),(True,'negative_merge'),(True,'negative_safety')] if a.phase=='manual' else []):
    if a.phase=='manual':
     source=a.output/'manual.rs'
     source.write_text((HERE/'manual.rs').read_text()+'\n'+(HERE/'domain.rs').read_text()+'\n'+(HERE/'checks.rs').read_text())
    name=label+('_infer' if goals else '_ordinary')+('_'+cfg if cfg else '')
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ.get('PATH',''),VERUS_Z3_PATH=str(a.bundle/'z3'),VERUS_INFER_CACHE=str(a.output/(name+'_cache')),
     RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu')
    cmd=[str(a.bundle/'verus'),str(source),'--crate-type=lib','--edition=2024','--crate-name','r3_'+name,
     '--no-cheating','--output-json','--time','--multiple-errors','20']+(['--infer-goals'] if goals else [])
    artifact=a.output/(name+'.vir')
    if a.phase=='manual':
     cmd+=['--export',str(artifact)]
     if cfg:
      shutil.copy2(a.output/'manual_ordinary.vir',artifact)
      cmd+=['--cfg',cfg]
    sg=ArtifactGuard([source]);code=run(name,cmd,env);sg.check()
    raw={'returncode':code,'stdout':(a.output/(name+'.stdout')).read_text(),'stderr':(a.output/(name+'.stderr')).read_text()}
    data=json.loads(raw['stdout']);v=data['verification-results']
    if any(x in raw['stderr'].lower() for x in ['resource limit','timed out','panicked at']):
     classification='RESOURCE_FAILURE'
    elif v.get('encountered-vir-error') or 'success' not in v or 'error[E' in raw['stderr']:
     classification='FRONTEND_FAILURE'
    else:
     validate_run(raw,code==0);classification='CHECKED' if code==0 else 'VERIFICATION_FAILURE'
     require(v.get('is-verifying-entire-crate') is True,'full crate checking')
     inf=data['verus-infer']
     if a.phase=='manual':
      failures={'negative_rejection':['wrong_rejection'],'negative_merge':['wrong_merge'],'negative_safety':['invalid_free']}.get(cfg,[])
      executable=R2_EXEC+CALLERS+['boundary_repeated_operations','valid_boundary_scenario','interior_one_split_merge']+failures
      proofs=R2_PROOF+['word_bits','word_frame','edit_frame','frame_trans','state_transport','width_positive','geometry_to_layout','valid_admission','order_half']
      scope_helper.require_native_scope(data,source,executable,proofs,failures,
       {'finish_mutation':20,'edit_frame':12} if goals else {})
      if goals:require_publication(data,not cfg,artifact)
      else:require(artifact.exists() and code==0,'ordinary export missing')
      frames=inf.get('word_sequence_assistance',[])
      native={b['function']:b['rust_function'] for b in inf['verification_scope']['native_functions']}
      require([native[x['function']] for x in frames]==(['U64GroupedBitmap::set'] if goals else []),'word assistance inventory')
      require(all(x['snapshots']==x['quantified_blocks']==x['library_calls']==1 for x in frames),'word assistance cost')
      if cfg:
       expected='precondition not satisfied' if cfg=='negative_safety' else 'assertion failed'
       require(expected in raw['stderr'],'negative diagnostic')
       require(not any(x in raw['stderr'] for x in ['postcondition not satisfied','error[E']),'unhealthy negative')
     require_exact(inf.get('functions',[]));require_function_coverage(inf.get('functions',[]),[])
    receipt['outcomes'].append({'label':name,'source':str(source),'source_sha256':digest(source),
     'classification':classification,'verification':v,'infer':data.get('verus-infer'),'times':data.get('times-ms'),
     'stdout_sha256':digest(a.output/(name+'.stdout')),'stderr_sha256':digest(a.output/(name+'.stderr'))})
    print(name,classification,v,flush=True)
    if a.phase=='manual':require(classification==('VERIFICATION_FAILURE' if cfg else 'CHECKED'),'unexpected manual outcome; stop before further controls')
  guard.check()
  if a.phase=='manual':require(all(x['classification']==('CHECKED' if 'negative_' not in x['label'] else 'VERIFICATION_FAILURE') for x in receipt['outcomes']),'manual replay has unresolved failures')
if __name__=='__main__':main()
