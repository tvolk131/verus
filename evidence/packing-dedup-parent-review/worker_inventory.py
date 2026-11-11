"""Deterministic exception gates, usable under -O; no old audits are executed."""
from trials import *
from baseline import queries
sys.path.insert(0,str(ROOT/'source/tools/research/item_tag_constructor_composition'))
from inventory import end,inventory
A=Path('/tmp/packing-dedup-cb7lqob8')
def field(t,key):
 depth=0;quoted=False;escape=False
 for i,c in enumerate(t):
  if quoted:
   if escape:escape=False
   elif c=='\\':escape=True
   elif c=='"':quoted=False
  elif c=='"':quoted=True
  elif c=='(':depth+=1
  elif c==')':depth-=1
  elif depth==1 and t.startswith(':'+key,i) and t[i+len(key)+1].isspace():
   a=i+len(key)+1
   while t[a].isspace():a+=1
   return t[a:end(t,a)] if t[a]=='(' else t[a:].split()[0]
 raise RuntimeError('missing field '+key)
def token_hash(s,fixture):return hashlib.sha256(json.dumps(re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',s.replace(fixture,'@FIXTURE@')),separators=(',',':')).encode()).hexdigest()
def typed(d,raw=False,cw=False):
 f=json.loads((d/'freeze.json').read_text());paths=list(f['fixtures']);need(len(paths)==1,'one fixture');text=(d/'vir/crate.vir').read_text();out={}
 allowed={h['candidate_id'] for h in json.loads((d/'stdout').read_text())['verus-infer']['functions']}
 def candidate(m):
  need(m[1] in allowed,'generated candidate ID must match fresh query identity');return ':infer_candidate_id "@GENERATED_CANDIDATE_ID@"'
 for m in re.finditer(r'\(Function\s',text):
  s=text[m.start():end(text,m.start())];n=field(s,'name');path=n[len('(Fun :path '):-1]
  if not path.startswith('component::'):continue
  if cw and path=='component::impl&%5::eq':
   ensure=field(s,'ensure');old='"'+paths[0]+':5:19: 5:31 (#18)"';new='"'+paths[0]+':4:29: 4:38 (#18)"'
   need(ensure.count(old)<=1,'only the known single Eq ensure diagnostic relocation')
   s=s.replace(ensure,ensure.replace(old,new),1)
  out[path]={k:token_hash(field(s,k),paths[0]) for k in ['mode','body','require','ensure','visibility','body_visibility']}
  out[path]['mode']=field(s,'mode');out[path]['whole_typed_function_sha256']=token_hash(s if raw else re.sub(r':infer_candidate_id "([a-f0-9]{64})"',candidate,s),paths[0])
 if raw:return {path:v['whole_typed_function_sha256'] for path,v in out.items()}
 inv=inventory(d)
 native=inv['native_scope']['native_functions']
 # Preserve every native classification and source location, rebinding only
 # the exact fixture filename between independent private roots.
 native=[{k:(v.replace(paths[0],'@FIXTURE@') if k=='source' else v) for k,v in b.items()} for b in native]
 return dict(functions=out,native=native)
def receipts(r):
 final=json.loads((r/'FINAL_TOOLS.json').read_text());tools=final['tools'];out={}
 need(tools=={n:sha(r/'bundle'/n) for n in TOOLS},'immutable final tools')
 need(final['source']==manifest(r/'src'),'immutable final source')
 for p in sorted(r.rglob('receipt.json')):
  if not (p.parent/'freeze.json').exists():continue
  j=json.loads(p.read_text());f=json.loads((p.parent/'freeze.json').read_text());label=str(p.parent.relative_to(r))
  need(j['guards'],'guards '+label)
  for name in ['freeze.json','stdout','stderr']:need(sha(p.parent/name)==j[name.split('.')[0]+'_sha256'],'authenticated '+label+' '+name)
  need(f['source']==final['source'],'same source '+label)
  if label in ['test-build','unit-build']:need(all(tools[n]==h for n,h in f['tools'].items()),'immutable build prerequisites '+label)
  elif label not in ['build','vstd']:need(f['tools']==tools,'same tools '+label)
  out[label]=dict(receipt_sha256=sha(p),exit=j['exit'],seconds=j['seconds'],verification=j.get('verification'),freeze_sha256=j['freeze_sha256'],stdout_sha256=j['stdout_sha256'],stderr_sha256=j['stderr_sha256'])
 return tools,out
