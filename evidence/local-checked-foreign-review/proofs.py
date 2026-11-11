"""Fresh exact-input local proofs; completed output directories are immutable."""
from pathlib import Path
import json,shutil,sys
import review as t
sys.path.insert(0,str(t.ROOT/'source/tools/research/item_tag_constructor_composition'))
from source import generate,fidelity
from inventory import inventory,signature
R=t.R;B=R/'bundle'
def source(kind):
 if kind in ['actual','false','off']:
  return 'use vstd::prelude::*;\nverus! {\npub fn all_bool(b:bool)->(r:u64) ensures r=='+('0u64' if kind=='false' else 'if b {1u64}else{0u64}')+', { u64::from(b) }\n}\n'
 x=generate(kind);fidelity(x);return x

def trial(kind,stable=False):
 base=R/('stable-'+kind) if stable else R/kind;base.mkdir(exist_ok=True)
 src=base/'component.rs';src.write_text(source(kind));d=base/'run';d.mkdir(exist_ok=True)
 argv=[B/'verus',src,'--crate-name=component','--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir','--compile','-o',d/'libcomponent.rlib']
 if kind!='off':argv+=['--infer-goals']
 inv=[];frozen=[];reports=[]
 for temp in (['cold','warm'] if stable else ['single']):
  receipt=t.run(str(d.relative_to(R)),argv,{'VERUS_INFER_CACHE':str(base/'candidate-cache')})
  arch=base/temp;shutil.copytree(d,arch)
  inv.append(inventory(arch));reports.append(json.loads((arch/'stdout').read_text()))
  frozen.append(json.loads((arch/'freeze.json').read_text()))
  v=receipt['verification'];expected={'actual':(2,0),'false':(1,1),'off':(0,1),'full-help':(52,0),'full-helpers':(36,0)}[kind]
  t.need((v['verified'],v['errors'])==expected,'count drift '+kind)
  t.need(v['is-verifying-entire-crate'],'incomplete scope')
  if stable and temp=='cold':shutil.rmtree(d)
 t.write(base/'inventory.json',inv[0])
 if stable:
  t.need(frozen[0]==frozen[1],'identical input mismatch')
  t.need(signature(base/'cold')==signature(base/'warm'),'typed identity mismatch')
  helper='component::checked_foreign_bool_to_u64%'
  def query(temp):
   s=(base/temp/'vir/root-final.air').read_text();a=s.index(';; Function-Def '+helper+'\n');b=s.find('\n;; Function-',a+1);return s[a:b if b>=0 else len(s)]
  t.need(query('cold')==query('warm'),'helper ordinary query changed')
  result=dict(identical_inputs=True,ordinary_identities_equal=True,helper_query_sha256=__import__('hashlib').sha256(query('cold').encode()).hexdigest())
  if kind=='full-help':
   a,b=[x['verus-infer']['functions'][0] for x in reports]
   result.update(cold=a,warm=b)
   t.need(a['candidate_id']==b['candidate_id'],'Eq candidate changed')
   t.need(a['metrics']['cache']=='miss' and b['metrics']['cache']=='hit','cache miss/hit required')
   for f in [a,b]:
    t.need(f['contract']['requires']=='true','Eq domain narrowed')
    for k in ['soundness','maximality','exactness']:t.need(f['proofs'][k]['required'] and f['proofs'][k]['status']=='proved','missing fresh '+k)
  t.write(base/'comparison.json',result)
if __name__=='__main__':
 for k in ['actual','false','off','full-help','full-helpers']:trial(k,k in ['actual','full-help'])
