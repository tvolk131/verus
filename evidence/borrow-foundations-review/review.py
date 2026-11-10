#!/usr/bin/env python3
"""Independent, source-only fresh model replay. No baseline controllers executed."""
import os, sys, json, hashlib, pathlib, subprocess, time, re, shlex
P=pathlib.Path
OWN=P(__file__).resolve().parent
REPO=OWN.parent.parent
ROOT=P('/tmp/borrow-foundations-review-20261110-ba6890b6')
SB=P('/tmp/stbor-checked-foundation-20261110'); TB=P('/tmp/tree-borrows-checked-foundation-20261110')
DEL=REPO/'source/tools/research'
SPEC={'sb':(SB,SB/'src/stacked-borrows',DEL/'stbor_checked_foundation','opamroot/legacy','stbor'), 'tb':(TB,TB/'simuliris',DEL/'tree_borrows_checked_foundation','opamroot/tb','simuliris')}
def sha(p):
 with P(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def load(p):return json.loads(P(p).read_text())
def save(p,d):
 p=P(p);p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
def checked(p,h):
 actual=('symlink:'+str(P(p).readlink())) if h.startswith('symlink:') else sha(p)
 assert actual==h,('hash mismatch',str(p));return h
def guard(base,files):
 for p,h in files.items():checked(P(base)/p,h)
 return {'count':len(files),'map_sha256':hashlib.sha256(json.dumps(files,sort_keys=True).encode()).hexdigest()}
def safe_artifact(p):
 p=P(p);assert any(p.is_relative_to(b) for b in [SB,TB,OWN,DEL]),str(p);return p
def linked(x,counts):
 if isinstance(x,dict):
  path=x.get('path') or x.get('receipt');h=x.get('sha256')
  if path and h:
   checked(safe_artifact(path),h);counts['linked_files']+=1
  for k,v in x.items():linked(v,counts)
 elif isinstance(x,list):
  for v in x:linked(v,counts)
def authenticate():
 ROOT.mkdir(exist_ok=True)
 out={}
 for key,(base,src,delivery,prefix,logical) in SPEC.items():
  ready=base/'baseline-ready.json';checked(ready,{'sb':'9259b8f303e52e38f1b47709429f2be23a81a4e44ed557b6058184a95e653e51','tb':'f56490bb8441dbd73c37fe46fe970dc008bb8b2b67ee2b651a04c34d2b1ca523'}[key])
  data=load(ready);counts={'linked_files':0};linked(data,counts)
  if key=='sb':
   completed=base/'completed-build-receipt.json';checked(completed,'0d85e9ec05f29413973137f03bc840ccb1e22170d50169b2837331e58770f649');d=load(completed);linked(d,counts)
   receipts={k:load(v['path']) for k,v in d['build_and_check_receipts'].items()}
   frozen=load(base/'manifests/upstream-tool-freeze.json')
   guards={n:guard(b,files) for n,b,files in [('installed',base/prefix,frozen['installed_files']),('initial_sources',base/'preflight-sources',frozen['dependency_initial_sources']),('build_snapshots',base/'source-snapshots',frozen['dependency_build_sources']),('downloads',base/'downloads',frozen['downloads']),('own_controller',delivery,frozen['controller'])]}
   guards['platform']=guard('/',load(base/'manifests/platform-toolchain.json')['files'])
   source=load(delivery/'upstream.json')['tracked_source_sha256']
   for p,e in load(base/'manifests/kernel-loaded-objects.json').items():checked(safe_artifact(p),e['object_sha256']);checked(safe_artifact(e['source']),e['source_sha256'])
   for n,e in load(base/'manifests/declaration-inventory.json').items():
    for suff in ['type','assumptions']+(['declaration'] if e.get('print_full_declaration') else []):checked(base/f'declaration-output/{e["index"]}-{suff}.out',e[suff+'_sha256'])
   installed={str(base/prefix):frozen['installed_files']}
  else:
   receipts={k:v['details'] for k,v in data['receipts'].items()}
   for file,h in data['hashes'].items():
    p=base/file if (base/file).is_file() else delivery/file
    checked(p,h)
   installed=load(base/'installed-pretrial.json');guards={p:guard(p,files) for p,files in installed.items()}
   guards['own_controller']=guard(delivery,data['pretrial_versions_and_loadpaths']['support_files'])
   source=load(delivery/'upstream-source.json')['files']
   for p,h in load(base/'model-objects.json').items():checked(src/p,h)
   linked(load(delivery/'INVENTORY.json'),counts)
  for name,r in receipts.items():
   assert r.get('exit_status',r.get('exit_code'))==0,(name,r)
   checked(safe_artifact(r['log']),r['log_sha256'])
  guards['upstream']=guard(src,source)
  dst=ROOT/key/'src';dst.mkdir(parents=True,exist_ok=True)
  for p,h in source.items():
   q=dst/p;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes((src/p).read_bytes());checked(q,h)
  assert not list(dst.rglob('*.vo'))
  save(ROOT/key/'installed-before.json',installed);save(ROOT/key/'source.json',source)
  out[key]={'ready_sha256':sha(ready),'guards':guards,'receipts':{n:{'argv':r.get('command',r.get('argv')),'exit':r.get('exit_status',r.get('exit_code')),'log_sha256':r['log_sha256']} for n,r in receipts.items()},**counts}
 save(ROOT/'authentication.json',out)
 print('authenticated sources, tools, receipts and linked inventories',flush=True)
def env_for(key):
 base,_,_,prefix,_=SPEC[key];work=ROOT/key
 for n in ['tmp','cache']: (work/n).mkdir(exist_ok=True)
 e={'LANG':'C.UTF-8'}
 e.update(PATH=str(base/prefix/'bin')+':/usr/bin:/bin',TMPDIR=str(work/'tmp'),XDG_CACHE_HOME=str(work/'cache'),PYTHONDONTWRITEBYTECODE='1',OPAMJOBS='2')
 if key=='tb':e['LD_LIBRARY_PATH']=str(base/'deps/lib')
 return e
def run(key,name,argv):
 work=ROOT/key;env=env_for(key);receipt=work/(name+'.json');log=work/(name+'.log')
 d={'argv':list(map(str,argv)),'cwd':str(work/'src'),'environment':env,'controller_sha256':sha(__file__),'started_unix':time.time(),'state':'running'}
 with log.open('wb') as f:
  p=subprocess.Popen(d['argv'],cwd=d['cwd'],env=env,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
  d.update(pid=p.pid,pgid=p.pid,startticks=P(f'/proc/{p.pid}/stat').read_text().split(') ',1)[1].split()[19]);save(receipt,d)
  code=p.wait()
 d.update(exit_code=code,finished_unix=time.time(),state='finished',log=str(log),log_sha256=sha(log));save(receipt,d)
 print(key,name,code,flush=True)
 assert code==0,(name,code)
 return log
def modules(key):
 if key=='sb':
  files=[x for x in shlex.split((ROOT/key/'src/_CoqProject').read_text(),comments=True) if x.endswith('.v')]
  return sorted('stbor.'+x.removeprefix('theories/').removesuffix('.v').replace('/','.') for x in files)
 return load(DEL/'tree_borrows_checked_foundation/closure.json')['roots']
def freeze(key):
 work=ROOT/key
 d={'controller_sha256':sha(__file__),'source_guard':guard(work/'src',load(work/'source.json')),'installed_guard':{b:guard(b,f) for b,f in load(work/'installed-before.json').items()},'roots':modules(key),'max_jobs':2}
 save(work/'pretrial.json',d)
def build(key):
 freeze(key)
 if key=='sb':run(key,'build',['/usr/bin/make','-j2'])
 else:
  run(key,'makefile',['/usr/bin/make','Makefile.rocq'])
  run(key,'build',['/usr/bin/make','-f','Makefile.rocq','-j2']+['theories/'+m.removeprefix('simuliris.').replace('.','/')+'.vo' for m in modules(key)])
 save(ROOT/key/'objects.json',{str(p.relative_to(ROOT/key/'src')):sha(p) for p in (ROOT/key/'src').rglob('*.vo')})
def kernel(key):
 base,_,_,prefix,logical=SPEC[key];src=ROOT/key/'src';bin=base/prefix/'bin'
 args=['-Q',str(src/'theories'),logical,'-o']
 if key=='sb':
  libs=base/prefix/'lib/coq';deps=sorted('.'.join(p.relative_to(libs/'user-contrib').with_suffix('').parts) for p in (libs/'user-contrib').rglob('*.vo'))
  argv=[bin/'coqchk','-coqlib',libs]+args+sorted(set(modules(key)+deps))
 else:argv=[bin/'rocq','check']+args+modules(key)
 run(key,'kernel',argv)
def inventory(key):
 base,_,_,prefix,logical=SPEC[key];work=ROOT/key;src=work/'src';text=(work/'kernel.log').read_text()
 libraries=sorted(set(re.findall(r'^Checking library: (\S+)',text,re.M)));constants=sorted(set(re.findall(r'checking cst:(\S+)',text)));blocks=sorted(set(re.findall(r'checking mutind block: (\S+)',text)))
 own={n:{'kind':'constant'} for n in constants if n.startswith(logical+'.')}
 for n in blocks:
  if n.startswith(logical+'.'):own.setdefault(n,{'kind':'inductive_block'})
 for p in sorted(src.rglob('*.glob')):
  module=logical+'.'+str(p.relative_to(src/'theories').with_suffix('')).replace('/','.')
  for line in p.read_text().splitlines():
   m=re.match(r'^(ind|constr|rec) (\d+):(\d+) (\S+) (\S+)$',line)
   if m:
    kind,a,b,scope,name=m.groups();n=module+('.'+scope if scope!='<>' else '')+'.'+name
    own.setdefault(n,{'kind':kind});own[n]['glob_kind']=kind
 # Do not inherit the delivered declaration list: names come from this fresh kernel/glob walk.
 out=work/'queries';out.mkdir();lines=['Require '+' '.join(modules(key))+'.','Set Printing Width 1000000.','Set Printing All.']
 if key=='sb':lines+=['Set Printing Universes.']
 else:lines+=['Set Printing Depth 1000000.']
 signatures={}
 for i,m in enumerate(libraries):
  p=out/f'module-{i}';lines.append(f'Redirect "{p}" Print Module {m}.');signatures[m]=str(p)+'.out'
 for i,(n,e) in enumerate(sorted(own.items())):
  e['index']=i
  for suff,cmd in [('type',('Check @' if key=='tb' else 'Check ')+n),('assumptions','Print Assumptions '+n)]:
   p=out/f'decl-{i}-{suff}';lines.append(f'Redirect "{p}" {cmd}.');e[suff+'_path']=str(p)+'.out'
  if e['kind']=='inductive_block' or e.get('glob_kind') in ['ind','rec']:
   p=out/f'decl-{i}-full';lines.append(f'Redirect "{p}" Print {n}.');e['declaration_path']=str(p)+'.out'
 query=work/'inventory.v';query.write_text('\n'.join(lines)+'\n')
 save(work/'discovery.json',{'libraries':libraries,'constants':constants,'blocks':blocks,'declarations':own,'signatures':signatures,'query_sha256':sha(query)})
 bin=base/prefix/'bin';args=['-q','-batch'] if key=='tb' else ['-quiet','-batch','-coqlib',str(base/prefix/'lib/coq')]
 run(key,'inventory',([bin/'rocq','repl'] if key=='tb' else [bin/'coqtop'])+args+['-Q',str(src/'theories'),logical,'-l',str(query)])
 for n,e in own.items():
  for suff in ['type','assumptions']+(['declaration'] if 'declaration_path' in e else []):
   p=P(e[suff+'_path']);t=p.read_text();assert t.strip() and 'Error:' not in t,(n,suff);e[suff+'_sha256']=sha(p)
  a=P(e['assumptions_path']).read_text();assert 'Closed under the global context' in a or 'Axioms:' in a,n
 for m,p in signatures.items():assert P(p).read_text().strip() and 'Error:' not in P(p).read_text(),m
 save(work/'inventory.json',{'libraries':libraries,'constants':constants,'blocks':blocks,'declarations':own,'signatures':{m:{'path':p,'sha256':sha(p)} for m,p in signatures.items()}})
def final_guard(key):
 work=ROOT/key
 result={'source':guard(work/'src',load(work/'source.json')),'installed':{b:guard(b,f) for b,f in load(work/'installed-before.json').items()},'objects':guard(work/'src',load(work/'objects.json')),'controller_sha256':sha(__file__)}
 assert result['controller_sha256']==load(work/'pretrial.json')['controller_sha256']
 save(work/'final-guards.json',result)
if __name__=='__main__':
 action=sys.argv[1]
 if action=='authenticate':authenticate()
 elif action=='all':
  for k in ['sb','tb']:build(k);kernel(k);inventory(k);final_guard(k)
 else:globals()[action](sys.argv[2])
