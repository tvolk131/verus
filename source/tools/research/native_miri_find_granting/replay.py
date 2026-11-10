#!/usr/bin/env python3
"""Frozen private verifier epochs. Preserve every attempt; Python -O safe gates."""
import hashlib,json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,tool_artifacts
from infer_experiment import LoggedCommands,replay_receipt,require
BUNDLE=Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
ACCEPTED=Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
ACCEPTED_SHA='2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a'

TOOLCHAIN=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
def compiler_artifacts():
 libs=list((TOOLCHAIN/'lib').glob('librustc_driver*.so'))
 require(len(libs)==1,'compiler driver inventory')
 return [TOOLCHAIN/'bin/rustc',Path('/home/dev/.cargo/bin/rustup'),*libs]

def inputs(plan):
 pins=ROOT/'source/tools/research/native_miri_policy_pin/pins.json'
 selected=json.loads((HERE/'SOURCE_FREEZE.json').read_text())
 source=[Path(q['path']) for q in selected['sources'].values()]
 # machine.rs supplies the exact ProvenanceExtra enum.
 source.append(Path(json.loads(pins.read_text())['sources']['src/tools/miri/src/machine.rs']['path']))
 local=[p for p in HERE.iterdir() if p.is_file() and p.suffix in ('.py','.rs','.md','.json') and p.name not in ('RESULTS.json',)]
 policy=[Path(p) for p in json.loads((HERE/'SOURCE_POLICY.json').read_text())['delivered_files']]
 return local+policy+[pins,ACCEPTED,Path(sys.executable).resolve(),*compiler_artifacts(),*source,*tool_artifacts(BUNDLE/'verus',BUNDLE/'z3'),Path(plan),*[Path(c['source']) for c in json.loads(Path(plan).read_text())['cases']]]

def freeze(plan,dest):
 a=ArtifactGuard(inputs(plan));Path(dest).write_text(json.dumps({'schema':1,'before_first_trial':True,'inputs':a.initial},indent=2)+'\n')
 print(dest)

def run(plan,frozen,output):
 output=Path(output).resolve();output.mkdir(exist_ok=False)
 receipt=dict(schema=1,commands=[],runs=[],native_correspondences=0,publication=False,goal_complete=False)
 with replay_receipt(output,receipt):
  f=json.loads(Path(frozen).read_text());require(f['before_first_trial'] is True,'not pretrial')
  require(set(map(lambda p:str(p.resolve()),inputs(plan)))<=set(f['inputs']),'incomplete freeze')
  guard=ArtifactGuard([Path(p) for p in f['inputs']]+[Path(frozen)])
  require(all(guard.initial[p]==v for p,v in f['inputs'].items()),'freeze drift')
  require(digest(ACCEPTED)==ACCEPTED_SHA,'baseline receipt identity')
  accepted=json.loads(ACCEPTED.read_text());require(accepted['success'] is True,'baseline success')
  names=['verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','z3']
  for n in names:
   p=BUNDLE/n;require(digest(p)==accepted['artifacts'][str(p)]['sha256'],'bundle identity '+n)
  for p,h in json.loads((HERE/'SOURCE_POLICY.json').read_text())['delivered_files'].items():
   require(digest(p)==h,'delivered source-policy identity')
  pins=json.loads((ROOT/'source/tools/research/native_miri_policy_pin/pins.json').read_text())
  for q in pins['sources'].values():
   if q['path'] in f['inputs']:require(digest(q['path'])==q['sha256'],'primary source identity')
  receipt['frozen_inputs']=f;receipt['frozen_file_sha256']=digest(frozen)
  receipt['authenticated_bundle_names']=names;receipt['canonical_receipt_sha256']=ACCEPTED_SHA
  env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(BUNDLE/'z3'),PYTHONDONTWRITEBYTECODE='1')
  for n in ['tmp','cache','target','xdg']: (output/n).mkdir()
  env.update(TMPDIR=str(output/'tmp'),XDG_CACHE_HOME=str(output/'xdg'),CARGO_TARGET_DIR=str(output/'target'))
  runner=LoggedCommands(output,output,receipt['commands'],timeout=90,guard=guard)
  for c in json.loads(Path(plan).read_text())['cases']:
   require(digest(c['source'])==c['source_sha256'],'planned source changed')
   env['VERUS_INFER_CACHE']=str(output/'cache'/c.get('cache',c['name']))
   argv=[str(BUNDLE/'verus'),c['source'],'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20']+c.get('extra',[])
   if c.get('infer',False):argv.append('--infer-goals')
   code=runner(c['name'],argv,env)
   stdout=(output/(c['name']+'.stdout')).read_text();stderr=(output/(c['name']+'.stderr')).read_text()
   try:data=json.loads(stdout)
   except ValueError:data={}
   summary=data.get('verification-results',{})
   if any(x in stderr.lower() for x in ['resource limit','timed out','panicked at']):kind='resource_or_internal_failure'
   elif summary.get('errors',0)>0 and not summary.get('encountered-vir-error'):kind='proof_failure'
   elif summary.get('encountered-error') or summary.get('encountered-vir-error') or not summary:kind='frontend_failure'
   elif code==0 and summary.get('success') is True:kind='ordinary_success'
   elif summary.get('errors',0)>0:kind='proof_failure'
   else:kind='unclassified_failure'
   r=dict(name=c['name'],source_sha256=digest(c['source']),exit_code=code,classification=kind,verification=summary,inference=data.get('verus-infer'),log_hashes={s:digest(output/(c['name']+'.'+s)) for s in ['stdout','stderr']})
   receipt['runs'].append(r)
   # Always preserve terminal state before a subsequent command.
   (output/'progress.json').write_text(json.dumps(receipt,indent=2)+'\n')
   print(json.dumps({k:r[k] for k in ['name','exit_code','classification','verification']}),flush=True)
  guard.check();receipt['all_final_guards_passed']=True
  receipt['products']={str(p.relative_to(output)):digest(p) for p in output.rglob('*') if p.is_file() and p.name not in ['progress.json','receipt.json']}
 print('receipt',output/'receipt.json',digest(output/'receipt.json'))
if __name__=='__main__':
 if sys.argv[1]=='freeze':freeze(sys.argv[2],sys.argv[3])
 elif sys.argv[1]=='run':run(*sys.argv[2:5])
 else:raise ValueError('unknown mode')
