"""Fresh independent reviewer: no baseline build, no historical checkout reads."""
import hashlib, json, os, pathlib, re, runpy, shutil, subprocess, sys, time, traceback
P = pathlib.Path
E = P(__file__).resolve().parent
REPO = E.parents[1]
SRC = REPO/'source/tools/research'
OUT = P('/tmp/borrow-entry-handoff-review-20261110-t3-a6eab20d')
ST = P('/tmp/stbor-entry-handoff-20261110/accepted025')
TB = P('/tmp/tree-borrows-entry-handoff-20261110')
SB = P('/tmp/stbor-checked-foundation-20261110')
BT = P('/tmp/tree-borrows-checked-foundation-20261110')
def sha(p):
 h=hashlib.sha256()
 with P(p).open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''): h.update(b)
 return h.hexdigest()
def load(p): return json.loads(P(p).read_text())
def save(p,x): P(p).write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def need(c,m):
 if not c: raise RuntimeError(m)
def verify(p,h): need(sha(p)==h,'hash mismatch: '+str(p)); return h
def clean(d,tool):
 for n in ['tmp','cache','config']: (d/n).mkdir(exist_ok=True)
 return dict(PATH=str(tool)+':/usr/bin:/bin',LANG='C.UTF-8',LC_ALL='C.UTF-8',TMPDIR=str(d/'tmp'),XDG_CACHE_HOME=str(d/'cache'),XDG_CONFIG_HOME=str(d/'config'),PYTHONDONTWRITEBYTECODE='1',OPAMJOBS='2')
def run(d,name,argv,env):
 receipt=d/(name+'.json'); log=d/(name+'.log')
 need(not receipt.exists() and not log.exists(),'already exists '+name)
 r=dict(argv=list(map(str,argv)),cwd=str(d),environment=env,started=time.time(),runner_sha256=sha(__file__),state='running',input_files={str(p):sha(p) for p in map(P,argv) if p.is_absolute() and p.is_file()})
 save(receipt,r)
 try:
  with log.open('wb') as f:
   p=subprocess.Popen(r['argv'],cwd=d,env=env,stdout=f,stderr=subprocess.STDOUT)
   r['pid']=p.pid; save(receipt,r); rc=p.wait()
  r.update(exit_code=rc,state='finished',log_sha256=sha(log),finished=time.time()); save(receipt,r)
  print(name,rc,flush=True); need(rc==0,'command failed '+name)
 except BaseException as ex:
  r['failure']=repr(ex); save(receipt,r); raise
 return log

def prepare():
 OUT.mkdir()
 # Authenticate own committed file set before trusting adapters or declarations.
 freeze=load(E/'scope-freeze.json')
 for n,h in freeze['repository_files'].items(): verify(REPO/n,h)
 auth={}
 verify(ST/'acceptance.json','e178df9b7adb2714ed6ae177271db895da11e972ab2488d7088d8a1445b666e0')
 for n,h in load(ST/'products.json').items(): verify(ST/n,h)
 for n,h in load(ST/'pretrial-freeze.json')['sources'].items(): verify(SRC/'stbor_entry_handoff'/n,h)
 verify(ST/'controller.py',load(ST/'pretrial-freeze.json')['controller_sha256'])
 auth['stbor']={n:sha(ST/n) for n in ['acceptance.json','products.json','pretrial-freeze.json','controller.py','object-dependency-map.json','loaded-objects.json','declaration-inventory.json','kernel.json','kernel.log']}
 # Authenticate all compact final Tree evidence before selecting final receipts.
 for n,h in load(SRC/'tree_borrows_entry_handoff/results.json')['evidence_files'].items(): verify(SRC/'tree_borrows_entry_handoff'/n,h)
 for n in ['final-01','kernel-final-01','inventory-final-01']:
  r=load(TB/n/'receipt.json'); need(r['exit_code']==0,n)
  verify(TB/n/'pretrial.json',r['pretrial_sha256']); verify(TB/n/'stdout.log',r['log_sha256'])
  for label,h in r['products'].items():
   p=P(label); need(p.is_relative_to(TB),'unexpected product path'); verify(p,h)
 auth['tree']={n:sha(TB/n/'receipt.json') for n in ['final-01','kernel-final-01','inventory-final-01']}
 verify(TB/'final-01-source/Entry.vo','9c7b77ede542b0a95cd244ed631189c7eeb0d7915c071536048ca8b1a540d0b6')
 need(sha(TB/'final-01-source/Entry.v')==sha(SRC/'tree_borrows_entry_handoff/Entry.v'),'tree source')
 verify(P('/tmp/native-entry-type-binding-20261110-trial003/receipt.json'),'e4a94a85107c5430cd1289ec6f27a921e4879da0dbe755b50124f85a3073df75')
 nr=load('/tmp/native-entry-type-binding-20261110-trial003/receipt.json')
 need(nr['success'] and nr['all_final_guards_passed'],'compiler original acceptance')
 auth['compiler_receipt_sha256']='e4a94a85107c5430cd1289ec6f27a921e4879da0dbe755b50124f85a3073df75'
 original=SRC/'native_entry_type_binding/trial003-freeze.json'
 verify(original,load(SRC/'native_entry_type_binding/results.json')['pretrial_freeze']['sha256'])
 f=load(original); mapped={}; changes=[]
 bundle=P('/tmp/native-entry-type-binding-20261110-delivered')
 verify(bundle/'bundle.json',load(SRC/'native_entry_type_binding/results.json')['copied_completed_erasure_bundle']['sha256'])
 for n,h in load(bundle/'bundle.json')['files'].items(): verify(bundle/n,h)
 for label,h in f['inputs'].items():
  if label.startswith('/home/dev/projects/verus/source/target-verus/release/'):
   p=bundle/P(label).name
  elif label.startswith('/home/dev/projects/verus/'):
   p=REPO/P(label).relative_to('/home/dev/projects/verus')
  else:
   p=P(label); need(str(p).startswith((f['sysroot']+'/', '/tmp/native-miri-policy-pin-20261110/')),'unapproved external '+label)
  verify(p,h); mapped[str(p)]=h
  if str(p)!=label: changes.append(dict(label=label,rebound=str(p),sha256=h))
 f['inputs']=mapped; f['verus_bundle']=str(bundle)
 f['review_mapping']={'original_freeze_sha256':sha(original),'changes':changes,'before_first_trial':True,'old_paths_are_labels_only':True}
 save(E/'compiler-rebound-freeze.json',f)
 # Rehome only driver path constants; exact delivered source bytes are copied by drivers.
 for lane,source,replacements in [
  ('stbor',SRC/'stbor_entry_handoff/check.py',[("HERE = Path(__file__).resolve().parent",'HERE = Path('+repr(str(SRC/'stbor_entry_handoff'))+')'),("ROOT = Path('/tmp/stbor-entry-handoff-20261110')",'ROOT = Path('+repr(str(OUT/'stbor'))+')')]),
  ('tree',SRC/'tree_borrows_entry_handoff/check.py',[("OWN = pathlib.Path(__file__).resolve().parent",'OWN = pathlib.Path('+repr(str(SRC/'tree_borrows_entry_handoff'))+')'),("OUT = pathlib.Path('/tmp/tree-borrows-entry-handoff-20261110')",'OUT = pathlib.Path('+repr(str(OUT/'tree'))+')')])]:
  text=source.read_text()
  for a,b in replacements: need(text.count(a)==1,'adapter replacement'); text=text.replace(a,b)
  (OUT/lane).mkdir(); dst=OUT/(lane+'-driver.py'); dst.write_text(text)
  auth[lane+'_adapter']={'original':str(source),'original_sha256':sha(source),'adapter':str(dst),'adapter_sha256':sha(dst),'replacements':replacements}
 save(E/'PRETRIAL.json',{'before_formal_trials':True,'reviewer_sha256':sha(__file__),'scope_freeze_sha256':sha(E/'scope-freeze.json'),'compiler_rebound_freeze_sha256':sha(E/'compiler-rebound-freeze.json'),'authentication':auth})
 print('authenticated originals and froze path rebindings',flush=True)

def stbor():
 d=OUT/'stbor'; env=clean(d,SB/'opamroot/legacy/bin')
 run(d,'driver-accept',['/usr/bin/python3','-B',str(OUT/'stbor-driver.py'),'fresh','accept'],env)
 # Independently discover declarations from kernel AND Print Module (not worker glob alone).
 s=d/'fresh'; args=['-coqlib',str(SB/'opamroot/legacy/lib/coq'),'-Q',str(SB/'src/stacked-borrows/theories'),'stbor','-Q',str(s),'handoff']
 independent_inventory(s,'stbor',s/'kernel.log',[str(SB/'opamroot/legacy/bin/coqtop'),'-quiet','-batch']+args,env,['handoff.candidate','handoff.composition'])

def tree():
 d=OUT/'tree'; env=clean(d,BT/'opamroot/tb/bin'); env['LD_LIBRARY_PATH']=str(BT/'deps/lib')
 # Authentication function only: never invoke an old build/prepare.
 ns=runpy.run_path(str(OUT/'tree-driver.py'))
 save(d/'authentication-before.json',ns['auth']())
 s=d/'source'; s.mkdir(); shutil.copyfile(SRC/'tree_borrows_entry_handoff/Entry.v',s/'Entry.v')
 save(d/'pretrial.json',{'source_sha256':sha(s/'Entry.v'),'auth_sha256':sha(d/'authentication-before.json'),'runner_sha256':sha(__file__)})
 args=['-Q',str(BT/'simuliris/theories'),'simuliris','-Q',str(s),'EntryHandoff']
 rocq=str(BT/'opamroot/tb/bin/rocq')
 run(s,'compile',[rocq,'compile','-q']+args+[str(s/'Entry.v')],env)
 log=run(s,'kernel',[rocq,'check','-o']+args+['EntryHandoff.Entry'],env)
 independent_inventory(s,'tree',log,[rocq,'repl','-q','-batch']+args,env,['EntryHandoff.Entry'])
 save(d/'authentication-after.json',ns['auth']())
 need(load(d/'authentication-before.json')==load(d/'authentication-after.json'),'tree baseline guard changed')

def independent_inventory(d,lane,log,argv,env,modules):
 text=log.read_text()
 need('Modules were successfully checked' in text,'kernel success marker')
 constants=sorted(set(re.findall(r'checking cst:([^\s]+)',text)))
 own=[n for n in constants if any(n.startswith(m+'.') for m in modules)]
 # Closed source set has no inductives except the primitive Tree record; include
 # record/constructor explicitly and cross-check generated products in module print.
 names=sorted(set(own+(['EntryHandoff.Entry.entry_result','EntryHandoff.Entry.Build_entry_result'] if lane=='tree' else [])))
 need(len(names)==(25 if lane=='stbor' else 17),'declaration count')
 q=d/'independent'; q.mkdir()
 lines=['Require '+' '.join(modules)+'.','Set Printing All.','Set Printing Width 1000000.','Set Printing Depth 1000000.','Print LoadPath.']
 for i,m in enumerate(modules): lines.append(f'Redirect "{q}/module-{i}" Print Module {m}.')
 for i,n in enumerate(names):
  lines += [f'Redirect "{q}/{i}-type" Check @{n}.',f'Redirect "{q}/{i}-assumptions" Print Assumptions {n}.']
 src=q/'queries.v'; src.write_text('\n'.join(lines)+'\n')
 run(d,'independent-inventory',argv+['-l',str(src)],env)
 rows={}
 for i,n in enumerate(names):
  a=(q/f'{i}-assumptions.out').read_text().strip(); need(a=='Closed under the global context',n+' not closed')
  rows[n]={'type':(q/f'{i}-type.out').read_text(),'assumptions':a,'type_sha256':sha(q/f'{i}-type.out'),'assumptions_sha256':sha(q/f'{i}-assumptions.out')}
 prints=[(q/f'module-{i}.out').read_text() for i in range(len(modules))]
 visible=set()
 for m,pr in zip(modules,prints):
  for n in re.findall(r'(?:Definition|Parameter)\s+([\w]+)\s*:',pr): visible.add(m+'.'+n)
 need(visible==set(own),'Print Module/kernel constants mismatch '+str(visible ^ set(own)))
 libraries=sorted(set(re.findall(r'Checking library: (\S+)',text)))
 objects={}
 for label in re.findall(r'\[intern (.*?) \.\.\.',text):
  p=P(label); p=p if p.is_absolute() else d/p
  need(p.is_relative_to(d) or p.is_relative_to(SB if lane=='stbor' else BT),'outside loaded path')
  objects[str(p)]={'sha256':sha(p),'source_sha256':sha(p.with_suffix('.v')) if p.with_suffix('.v').is_file() else None}
 save(E/(lane+'-INVENTORY.json'),{'declarations':rows,'compiled_modules':prints,'kernel_own_constants':own,'all_individually_closed':True,'checked_libraries':libraries,'kernel_constant_count':len(constants),'kernel_context':text[text.rfind('==============='):]})
 save(d/'independent-loaded-objects.json',objects)
 print(lane,len(names),'closed declarations;',len(libraries),'libraries;',len(objects),'objects',flush=True)

def compiler():
 d=OUT/'compiler-wrapper'; d.mkdir(); env=clean(d,P('/usr/bin'))
 run(d,'replay',['/usr/bin/python3','-B','-O',str(SRC/'native_entry_type_binding/check.py'),'--freeze',str(E/'compiler-rebound-freeze.json'),'--output',str(OUT/'compiler')],env)
 r=load(OUT/'compiler/receipt.json'); need(r['success'] and r['all_final_guards_passed'],'compiler acceptance')
 old=load('/tmp/native-entry-type-binding-20261110-trial003/receipt.json')
 need(r['queries']==old['queries'],'compiler mathematical result differs')
 for name in ['pinned-unpin','pinned-unsafe-unpin','cell-freeze']:
  text=(OUT/'compiler'/(name+'.stderr')).read_text()
  need(set(re.findall(r'error\[(E\d+)\]',text))=={'E0277'},'mixed rejection '+name)
 print('compiler: 9 commands, identical query products, 3 E0277 controls',flush=True)

if __name__=='__main__':
 try: globals()[sys.argv[1]]()
 except BaseException as ex:
  if OUT.exists(): save(OUT/('review-'+sys.argv[1]+'-failure.json'),{'error':repr(ex),'traceback':traceback.format_exc(),'runner_sha256':sha(__file__),'classification':'reviewer_setup_or_controller_failure_pending_log_review'})
  raise
