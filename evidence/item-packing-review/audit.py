"""Read-only audit; writes compact identities only into this review directory."""
import sys,json,re,hashlib
from pathlib import Path
import review as t
sys.path.insert(0,str(t.ROOT/'source/tools/research/item_tag_constructor_composition'))
from inventory import inventory,end,field
R=t.root()
def bodies(d):
 f=json.loads((d/'freeze.json').read_text());paths=list(f['fixtures']);t.need(len(paths)==1,'one fixture');txt=(d/'vir/crate.vir').read_text();out={}
 for m in re.finditer(r'\(Function\s',txt):
  s=txt[m.start():end(txt,m.start())];n=field(s,'name')
  if n.startswith('(Fun :path component::'):
   b=field(s,'body').replace(paths[0],'@EXACT_FIXTURE@')
   # Whitespace tokenization preserves all operators, types and source coordinates.
   out[n]=dict(mode=field(s,'mode'),tokens=re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',b))
 return out
def helper(d):
 air=(d/'vir/root-final.air').read_text();s=air.index(';; Function-Def component::checked_foreign_bool_to_u64%\n');e=air.find('\n;; Function-',s+1);return air[s:e if e>=0 else len(air)]
def eq(d):
 j=json.loads((d/'stdout').read_text());fs=j['verus-infer']['functions'];t.need(len(fs)==1,'one Eq');e=fs[0];t.need(e['function']=='component::BorTag::eq' and e['contract']['requires']=='true','Eq identity/domain')
 for p in ['soundness','maximality','exactness']:t.need(e['proofs'][p]['required'] and e['proofs'][p]['status']=='proved','Eq '+p)
 return e

def stable(name,full):
 b=R/name;c=b/'cold';w=b/'warm';a=json.loads((c/'freeze.json').read_text());z=json.loads((w/'freeze.json').read_text());t.need(a==z,'frozen inputs')
 air={p.name:t.sha(p) for p in (c/'vir').glob('*-final.air')};wa={p.name:t.sha(p) for p in (w/'vir').glob('*-final.air')};t.need(air==wa,'ALL actual AIR');t.need(len(air)==(118 if full else 30),'AIR count')
 t.need(bodies(c)==bodies(w),'typed bodies');t.need(helper(c)==helper(w),'bool helper query');iv=inventory(c)
 if full:
  # Completed prior baseline is evidence bytes, never a historical source checkout.
  old=t.OLD/'baseline2-full-on';of=json.loads((old/'freeze.json').read_text());oj=json.loads((old/'receipt.json').read_text());t.need(t.sha(old/'freeze.json')==oj['freeze_sha256'] and t.sha(old/'stdout')==oj['stdout_sha256'],'baseline receipt');t.need(bodies(c)==bodies(old),'unchanged baseline typed bodies')
  e,f=eq(c),eq(w);t.need(e['candidate_id']==f['candidate_id'],'candidate ID');t.need(e['metrics']['cache']=='miss' and f['metrics']['cache']=='hit','real cache miss/hit')
  ov=inventory(old);t.need(iv['ordinary_original_functions'].keys()==ov['ordinary_original_functions'].keys(),'ordinary names')
  def native(v):return [{k:x for k,x in row.items() if k!='source'} for row in v['native_scope']['native_functions']]
  baseline_native=native(ov)
  for row in baseline_native:
   if row['body_status']=='selected_body_not_certified_by_successful_full_run':row['body_status']='checked_exec_body'
  t.need(native(iv)==baseline_native,'exact native owners and expected failed-baseline status transition')
  rows=iv['ordinary_original_functions'];t.need(len(rows)==38 and sum(v['mode']=='Exec' for v in rows.values())==19 and sum(v['mode']=='Spec' for v in rows.values())==19,'38=19+19');t.need(len(native(iv))==47,'native47')
  # Compare expected names/modes/bodies against committed shipping inventory after exact filename rebinding.
  expected=json.loads((t.ROOT/'source/tools/research/item_packing_goal/INVENTORY.json').read_text())['full'];t.need(rows.keys()==expected['ordinary_original_functions'].keys(),'committed names');t.need(native(iv)==native(expected),'committed native inventory')
  for n,v in rows.items():t.need(v['mode']==expected['ordinary_original_functions'][n]['mode'] and v['body_present']==expected['ordinary_original_functions'][n]['body_present'],'mode/body '+n)
 iv['query_sha256']=air;iv['helper_query_sha256']=hashlib.sha256(helper(c).encode()).hexdigest();iv['typed_exact_filename_rebound']={n:dict(mode=v['mode'],body_sha256=hashlib.sha256(json.dumps(v['tokens'],separators=(',',':')).encode()).hexdigest()) for n,v in bodies(c).items()}
 return iv
if __name__=='__main__':
 out={n:stable(n,n=='full') for n in ['full','packet']};t.write(t.HERE/'INVENTORY.json',out);print('full/packet exact identity audit passed')
