"""Strict compact audit. All checks are explicit (safe under python -O)."""
import json, re, shutil, sys
from pathlib import Path
import run as t
sys.path.insert(0,str(t.HERE.parent/'item_tag_constructor_composition'))
from inventory import inventory,signature,end,field
R=t.root()
def stable(name,src):
 base=R/name;base.mkdir();p=base/'component.rs';p.write_text(src);d=base/'run'
 argv=[R/'bundle/verus',p,'--crate-name=component','--crate-type=lib','--no-cheating','--compile','--infer-goals','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir','-o',d/'libcomponent.rlib']
 for temp in ['cold','warm']:
  j=t.run(name+'/run',argv,{'VERUS_INFER_CACHE':str(base/'candidate-cache')})
  shutil.copytree(d,base/temp);t.need((base/temp/'receipt.json').exists(),'archive');shutil.rmtree(d)
  t.need(j['exit']==0 and j['verification']['errors']==0 and j['verification']['is-verifying-entire-crate'],'full verification '+temp)
 a,b=[json.loads((base/temp/'freeze.json').read_text()) for temp in ['cold','warm']]
 t.need(a==b,'cold/warm frozen inputs')
 t.need(signature(base/'cold')==signature(base/'warm'),'body identity')
 return base

def source_bodies(d):
 frozen=json.loads((d/'freeze.json').read_text());fixtures=list(frozen['fixtures'])
 t.need(len(fixtures)==1,'exact source mapping')
 txt=(d/'vir/crate.vir').read_text();out={}
 for m in re.finditer(r'\(Function\s',txt):
  f=txt[m.start():end(txt,m.start())];name=field(f,'name');body=field(f,'body')
  if name.startswith('(Fun :path component::'):
   body=body.replace(fixtures[0],'@FROZEN_SOURCE@')
   out[name]=re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',body)
 return out
def checked(base,full=False):
 queries={p.name:t.sha(p) for p in (base/'cold/vir').glob('*-final.air')}
 warm={p.name:t.sha(p) for p in (base/'warm/vir').glob('*-final.air')}
 t.need(queries==warm,'every actual AIR query identical')
 inv=inventory(base/'cold');inv['query_identities']=queries;rows=inv['ordinary_original_functions']
 t.need(all(v['mode']!='Proof' for v in rows.values()),'local proof body')
 if full:
  orig=inventory(R/'baseline2-full-on')
  t.need(set(rows)==set(orig['ordinary_original_functions']),'exact ordinary inventory')
  t.need(inv['local_function_details'].keys()==orig['local_function_details'].keys(),'exact function details')
  t.need(source_bodies(base/'cold')==source_bodies(R/'baseline2-full-on'),'unchanged original typed bodies (only exact fixture path rebound)')
  native=inv['native_scope']['native_functions'];onative=orig['native_scope']['native_functions']
  def names(n):return sorted((x['function'],x.get('mode')) for x in n)
  t.need(names(native)==names(onative),'exact native inventory')
  for temp in ['cold','warm']:
   j=json.loads((base/temp/'stdout').read_text());fs=j['verus-infer']['functions'];t.need(len(fs)==1,'exact helper inventory')
   eq=fs[0];t.need(eq['function']=='component::BorTag::eq' and eq['contract']['requires']=='true','full Eq domain')
   for prop in ['soundness','maximality','exactness']:t.need(eq['proofs'][prop]['required'] and eq['proofs'][prop]['status']=='proved','Eq '+prop)
  a,b=[json.loads((base/temp/'stdout').read_text())['verus-infer']['functions'][0] for temp in ['cold','warm']]
  t.need(a['candidate_id']==b['candidate_id'],'candidate identity');t.need(a['metrics']['cache']=='miss' and b['metrics']['cache']=='hit','cold/warm cache')
 for temp in ['cold','warm']:
  air=(base/temp/'vir/root-final.air').read_text()
  helper=';; Function-Def component::checked_foreign_bool_to_u64%\n';start=air.index(helper);stop=air.find('\n;; Function-',start+1)
  h=air[start:stop if stop>=0 else len(air)]
  if temp=='cold':first=h
  else:t.need(h==first,'checked helper query identity')
 inv['helper_query_sha256']=t.hashlib.sha256(first.encode()).hexdigest()
 return inv
if __name__=='__main__':
 t.need(sys.flags.dont_write_bytecode,'use -B')
 if sys.argv[1]=='stable':stable(sys.argv[2],(R/'full.rs').read_text() if sys.argv[3]=='full' else (t.HERE/'reuse.rs').read_text())
 elif sys.argv[1]=='audit':
  a=checked(R/sys.argv[2],True);b=checked(R/sys.argv[3]);t.write(t.HERE/'INVENTORY.json',dict(full=a,reuse=b))
