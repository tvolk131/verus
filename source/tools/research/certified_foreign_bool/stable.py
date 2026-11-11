"""Identical-path trials: archive completed outputs before reuse."""
from trials import *
sys.path.insert(0,str(ROOT/'source/tools/research/item_tag_constructor_composition'))
from source import generate,fidelity
from inventory import inventory,signature

def stable(kind,source,expected, analyze_only=False):
 base=R/('stable-'+kind);base.mkdir(exist_ok=analyze_only);d=base/'run';src=fixture('stable-'+kind+'.rs',source)
 argv=[R/'bundle/verus',src,'--crate-name=component','--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir','--compile','-o',d/'libcomponent.rlib','--export',d/'component.vir','--infer-goals']
 frozen=[];reports=[];invs=[];certs=[]
 for epoch in ['cold','warm']:
  archive=base/epoch
  if not analyze_only:
   rec=run(str(d.relative_to(R)),argv,{'VERUS_INFER_CACHE':str(base/'candidate-cache')})
   require(str(d.relative_to(R)),rec,expected)
   shutil.copytree(d,archive)
  frozen.append(json.loads((archive/'freeze.json').read_text()))
  reports.append(json.loads((archive/'stdout').read_text()))
  invs.append(inventory(archive));certs.append(metadata(archive/'component.vir')[1])
  if epoch=='cold' and not analyze_only:shutil.rmtree(d)
 need(frozen[0]==frozen[1],'identical frozen inputs '+kind)
 need(signature(base/'cold')==signature(base/'warm'),'ordinary typed inventory changed '+kind)
 need(certs[0]['foreign_dependencies']==certs[1]['foreign_dependencies'],'required foreign certificate identity changed '+kind)
 need({k for k in certs[0] if certs[0][k]!=certs[1][k]} <= {'summaries'},'non-summary certificate fields changed')
 def query(epoch):
  s=(base/epoch/'vir/root-final.air').read_text();a=s.index(';; Function-Def component::checked_foreign_bool_to_u64%\n');b=s.find('\n;; Function-',a+1)
  return s[a:b if b>=0 else len(s)]
 need(query('cold')==query('warm'),'actual helper query changed '+kind)
 comparison=dict(raw_api_summary_fingerprints=[j['summaries'] for j in certs],identical_inputs=True,certificate=certs[0]['foreign_dependencies'],helper_query_sha256=hashlib.sha256(query('cold').encode()).hexdigest(),helper_query_count=query('cold').count('(check-valid'),inventories=invs)
 if kind=='full':
  for j in reports:need(len(j['verus-infer']['functions'])==1,'exact inferred Eq inventory')
  a,b=[j['verus-infer']['functions'][0] for j in reports]
  need(a['candidate_id']==b['candidate_id'],'Eq identity changed')
  need(a['metrics']['cache']=='miss' and b['metrics']['cache']=='hit','cold/warm cache')
  for h in [a,b]:
   need(h['contract']['requires']=='true','Eq domain')
   for k in ['soundness','maximality','exactness']:need(h['proofs'][k]['required'] and h['proofs'][k]['status']=='proved','fresh '+k)
  eq_air=[]
  for epoch in ['cold','warm']:
   paths=list((base/epoch/'vir').glob('*__eq-final.air'));need(len(paths)==1,'exact Eq query file')
   text=paths[0].read_text();need(text.count('(check-valid')==3,'all original Eq queries')
   eq_air.append(dict(sha256=sha(paths[0]),check_valid_count=3))
  need(eq_air[0]==eq_air[1],'Eq query identity changed')
  comparison['Eq']=[a,b];comparison['Eq_air']=eq_air
 write(base/'comparison.json',comparison)
 print('identical',kind,comparison['helper_query_sha256'],flush=True)
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B')
 if sys.argv[1]=='bool':stable('bool',BOOL,2)
 else:
  s=generate('full-help');fidelity(s);need(s==Path('/tmp/checked-bool-build-klx7abqu/source-full-help.rs').read_text(),'baseline');stable('full',s,52,sys.argv[1]=='analyze-full')
