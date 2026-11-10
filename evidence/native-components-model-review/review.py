#!/usr/bin/env python3
"""Independent exact-scope replay/authentication. Raw outputs remain private."""
import argparse, hashlib, json, os, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
RAW = Path('/tmp/native-components-model-review-3f865efc')
SPECS = {
 'caller': ('model_caller_ordinary_writer', 'trial003-freeze.json', '/tmp/verus-infer-buffer-cycle-20261109/parent-caller-ordinary-003/proofs', '1b9063d276c3ef505562143614531a0191230ee91bf3ca07d5b2c63d2af82f4c', '/home/dev/projects/verus'),
 'constructor': ('native_constructor_erasure', 'final-freeze.json', '/tmp/native-constructor-erasure-6bb72bfb-trial006', '2f9679b44cded4fcb68052c062b8a6a4942b92d99bc099f2cac4d07625e84cb3', '/home/dev/.t3/worktrees/verus/t3-4e03cb31'),
 'distance': ('native_pointer_distance', 'final-freeze.json', '/tmp/native-pointer-distance-efea4e77-trial011', '04aedea513457712d9afaf49addbf694c7d33f2676408adec426b19ba6dca47b', '/home/dev/.t3/worktrees/verus/t3-2f7ed70e')}
def need(ok, msg):
 if not ok: raise RuntimeError(msg)
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p): return json.loads(Path(p).read_text())
def write(p,d): Path(p).write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
def mapsha(d): return hashlib.sha256(json.dumps(d,sort_keys=True,separators=(',',':')).encode()).hexdigest()
def remap(p, old):
 p=Path(p)
 if p.is_relative_to(old): return ROOT/p.relative_to(old)
 need(p.is_relative_to('/tmp') or p.is_relative_to(ROOT) or p.is_relative_to('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu'), 'unapproved input path '+str(p))
 return p

def prepare():
 RAW.mkdir(mode=0o700,exist_ok=True)
 need(not (RAW/'pretrial-inputs.json').exists(), 'pretrial manifest already exists')
 pins={}; summaries={}
 def retain(p,expected=None):
  p=Path(p); h=sha(p)
  need(expected is None or h==expected,'identity mismatch '+str(p))
  need(str(p) not in pins or pins[str(p)]==h,'inconsistent pin '+str(p))
  pins[str(p)]=h
 for tag,(scope,freeze,worker,expected,old) in SPECS.items():
  src=ROOT/'source/tools/research'/scope; worker=Path(worker)
  retain(worker/'receipt.json',expected)
  r=read(worker/'receipt.json'); need(r['success'] is True,'failed delivered receipt')
  f=read(src/freeze);need(f['before_first_trial'] is True,'pretrial freeze missing')
  retain(src/freeze);retain(src/'results.json');retain(src/'REPORT.md')
  replay_root=ROOT
  if tag=='constructor':
   replay_root=RAW/'constructor-delivered-root'
   for n,h in f['files'].items():
    target=replay_root/n;target.parent.mkdir(parents=True,exist_ok=True)
    data=(ROOT/n).read_bytes()
    if hashlib.sha256(data).hexdigest()!=h:
     need(n=='source/tools/research/mutable_buffer_goal/TARGET.md','unexpected integrated drift')
     retain(ROOT/n)
     data=subprocess.check_output(['git','show','3d668d29:'+n],cwd=ROOT)
    target.write_bytes(data);retain(target,h)
   target=replay_root/'source/tools/research'/scope/freeze
   target.write_bytes((src/freeze).read_bytes());retain(target)
  for n,h in f['files'].items(): retain(replay_root/n,h)
  for n,facts in r['inputs'].items():
   target=remap(n,old)
   if tag=='constructor' and target.is_relative_to(ROOT):
    relative=target.relative_to(ROOT)
    if (replay_root/relative).is_file(): target=replay_root/relative
   retain(target,facts['sha256'])
  for n,facts in r['proof_artifacts'].items(): retain(n,facts['sha256'])
  for c in r['commands']:
   need(c['exit_code']==0,'failed delivered command')
   for ext in ['stdout','stderr']: retain(worker/(c['label']+'.'+ext))
  d=read(src/'results.json')
  if tag=='caller':
   retain(d['parent_receipt']['path'],d['parent_receipt']['sha256'])
   retain(src/freeze,d['freeze']['sha256'])
   for k,ext in [('proof_sources','.v'),('proof_objects','.vo')]:
    for n,h in d[k].items(): retain(worker/'proofs'/n,h)
  elif tag=='constructor':
   retain(src/'TRUST_INVENTORY.json')
   retain(src/freeze,d['freeze_sha256'])
   for n,h in d['products'].items(): retain(worker/n,h)
   for k in ['declaration_statements','module_prints']:
    for facts in d[k].values(): retain(facts['path'],facts['sha256'])
   for n,h in d['command_logs'].items(): retain(worker/n,h)
   retain(ROOT/'source/tools/research/native_constructor_erasure/trial-006.json')
  else:
   retain(src/freeze,d['final_freeze_sha256'])
   for n,h in d['proof_objects'].items(): retain(worker/'proofs'/n,h)
   for n,facts in d['logs'].items():
    for ext in ['stdout','stderr']: retain(worker/(n+'.'+ext),facts[ext+'_sha256'])
  # Exact retained pretrial bytes; caller's controller calls this freeze.json.
  trial_freeze = worker/'freeze.json' if tag!='constructor' else ROOT/'source/tools/research/native_constructor_erasure/trial-006.json'
  retain(trial_freeze,sha(src/freeze))
  summaries[tag]={'replay_root':str(replay_root),'worker_receipt':str(worker/'receipt.json'),'sha256':expected,'frozen_files':len(f['files']),'source_declarations':sum(map(len,f['constants'].values())),'worker_inputs':len(r['inputs']),'worker_products':len(r['proof_artifacts'])}
 for p in [HERE/'review.py',HERE/'INTERFACE.md']: retain(p)
 write(RAW/'pretrial-inputs.json',pins)
 write(HERE/'pretrial-freeze.json',{'before_first_review_replay':True,'prepared_head':'3f865efc','raw':str(RAW),'manifest_sha256':sha(RAW/'pretrial-inputs.json'),'input_count':len(pins),'input_map_sha256':mapsha(pins),'scopes':summaries})
 print(json.dumps(summaries))

def guards():
 f=read(HERE/'pretrial-freeze.json')
 need(sha(RAW/'pretrial-inputs.json')==f['manifest_sha256'],'review input manifest changed')
 pins=read(RAW/'pretrial-inputs.json')
 for p,h in pins.items(): need(sha(p)==h,'final input changed '+p)
 return len(pins)

def replay(tag):
 guards()
 scope,freeze,*_=SPECS[tag];replay_root=Path(read(HERE/'pretrial-freeze.json')['scopes'][tag]['replay_root']);src=replay_root/'source/tools/research'/scope
 env=dict(os.environ)
 for k in ['COQPATH','COQLIB','COQCORELIB','OCAMLPATH','CAML_LD_LIBRARY_PATH','LD_PRELOAD','LD_LIBRARY_PATH']: env.pop(k,None)
 argv=[sys.executable,'-B',str(src/'check.py'),'--freeze',str(src/freeze),'--output',str(RAW/tag)]
 with (RAW/(tag+'.controller.stdout')).open('wb') as out,(RAW/(tag+'.controller.stderr')).open('wb') as err:
  p=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err)
 write(RAW/(tag+'.launch.json'),{'argv':argv,'exit_code':p.returncode,'removed_environment_overrides':True})
 need(p.returncode==0,'controller replay failed: '+tag)
 need(read(RAW/tag/'receipt.json')['success'] is True,'fresh receipt failed')
 guards()
 print(tag+' fresh controller passed',flush=True)

if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('mode',choices=['prepare','replay','guards']);a.add_argument('scope',nargs='?',choices=list(SPECS));v=a.parse_args()
 if v.mode=='prepare':prepare()
 elif v.mode=='guards':print(guards())
 else:replay(v.scope)
