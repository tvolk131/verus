#!/usr/bin/env python3
"""Small R3 replay using the existing experiment/measurement helpers."""
import argparse, json, os, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage
from infer_eval_support import ArtifactGuard, digest

TOOLS = ['verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','z3']
R2_EXEC = [owner+'::'+name for owner,names in [
 ('BtreeBitmap',['clear','get_height','get_level','get_level_mut','set','update_to_root']),
 ('U64GroupedBitmap',['clear','data_index_of','get','len','select_mask','set'])] for name in names]
R2_PROOF = ['layer_edit','leaf_edit','clear_not_full','finish_mutation','propagation','ancestor_next']
CALLERS = ['BtreeBitmap::get','BtreeBitmap::len','next_higher_order','buddy_page',
 'BuddyAllocator::get_order_free_mut','BuddyAllocator::record_alloc_inner','BuddyAllocator::free_inner']

def main():
 p=argparse.ArgumentParser();p.add_argument('--bundle',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
 p.add_argument('--phase',choices=['baseline','manual'],default='baseline');a=p.parse_args()
 a.bundle=a.bundle.resolve();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
 receipt={'kind':'R3_LOCAL_CHECKPOINT_NOT_PARENT_ACCEPTANCE','phase':a.phase,'commands':[],'outcomes':[]}
 with replay_receipt(a.output,receipt):
  freeze=json.loads((HERE/'FREEZE.json').read_text())
  require(digest(a.bundle/'receipt.json')==freeze['runtime_receipt'],'receipt hash')
  for n,h in freeze['runtime_tools'].items():require(digest(a.bundle/n)==h,'tool hash '+n)
  require((a.bundle/'verus-root').is_file(),'runtime marker')
  require(digest(HERE/'baseline.rs')==freeze['baseline_sha256'],'frozen baseline')
  for n,h in freeze['reused'].items():require(digest(HERE.parents[4]/n)==h,'accepted provider '+n)
  guard=ArtifactGuard([a.bundle/n for n in TOOLS]+[Path(__file__),HERE/'FREEZE.json'])
  run=LoggedCommands(a.output,a.output,receipt['commands'],timeout=180,guard=guard)
  prior=HERE.parents[1]/'practical_redb_fedimint'
  sources=[('combined',prior/'common_set/r2-combined.rs'),('provider',prior/'redb_tree/manual.rs'),('caller',HERE/'corrected.rs')] if a.phase=='baseline' else [('manual',HERE/'manual.rs')]
  for label,source in sources:
   for goals in [False,True]:
    name=label+('_infer' if goals else '_ordinary')
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ.get('PATH',''),VERUS_Z3_PATH=str(a.bundle/'z3'),VERUS_INFER_CACHE=str(a.output/(name+'_cache')),
     RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu')
    cmd=[str(a.bundle/'verus'),str(source),'--crate-type=lib','--edition=2024','--crate-name','r3_'+name,
     '--no-cheating','--output-json','--time','--multiple-errors','20']+(['--infer-goals'] if goals else [])
    sg=ArtifactGuard([source]);code=run(name,cmd,env);sg.check()
    raw={'returncode':code,'stdout':(a.output/(name+'.stdout')).read_text(),'stderr':(a.output/(name+'.stderr')).read_text()}
    data=json.loads(raw['stdout']);v=data['verification-results']
    if v.get('encountered-vir-error') or 'success' not in v or 'error[E' in raw['stderr']:
     classification='FRONTEND_FAILURE'
    else:
     validate_run(raw,code==0);classification='CHECKED' if code==0 else 'VERIFICATION_FAILURE'
     require(v.get('is-verifying-entire-crate') is True,'full crate checking')
     inf=data['verus-infer'];require_exact(inf.get('functions',[]));require_function_coverage(inf.get('functions',[]),[])
    receipt['outcomes'].append({'label':name,'source':str(source),'source_sha256':digest(source),
     'classification':classification,'verification':v,'infer':data.get('verus-infer'),'times':data.get('times-ms'),
     'stdout_sha256':digest(a.output/(name+'.stdout')),'stderr_sha256':digest(a.output/(name+'.stderr'))})
    print(name,classification,v,flush=True)
  guard.check()
if __name__=='__main__':main()
