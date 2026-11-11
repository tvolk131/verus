"""Same-input cold/warm trials; counts derived from exact retained inventories."""
from trials import *
sys.path.insert(0,str(ROOT/'source/tools/research/item_tag_constructor_composition'))
from inventory import inventory,signature
from baseline import queries

def stable(kind,source):
 base=R/('stable-'+kind);base.mkdir(exist_ok=True);d=base/'run';src=fixture('stable-'+kind+'.rs',source)
 argv=[R/'bundle/verus',src,'--crate-name=component','--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir','--compile','-o',d/'libcomponent.rlib','--export',d/'component.vir','--infer-goals']
 for epoch in ['cold','warm']:
  need(not (base/epoch).exists(),'immutable epoch')
  rec=run(str(d.relative_to(R)),argv,{'VERUS_INFER_CACHE':str(base/'candidate-cache')});require(str(d.relative_to(R)),rec,None)
  shutil.move(d,base/epoch)
 fs=[json.loads((base/e/'freeze.json').read_text()) for e in ['cold','warm']];need(fs[0]==fs[1],'identical inputs '+kind)
 need(signature(base/'cold')==signature(base/'warm'),'typed inventory '+kind)
 cert=[metadata(base/e/'component.vir')[1] for e in ['cold','warm']]
 need(cert[0]['foreign_dependencies']==cert[1]['foreign_dependencies'],'foreign identity')
 need({k for k in cert[0] if cert[0][k]!=cert[1][k]} <= {'summaries'},'non-summary metadata')
 airs=[{p.name:sha(p) for p in sorted((base/e/'vir').glob('*.air'))} for e in ['cold','warm']];need(airs[0]==airs[1],'all AIR identities '+kind)
 inv=[inventory(base/e) for e in ['cold','warm']]
 counts=[]
 for epoch in ['cold','warm']:
  files=list((base/epoch/'vir').glob('*final.air'));eq=[p for p in files if '__eq-' in p.name];bv=[p for p in files if p.name!='root-final.air' and p not in eq]
  counts.append(dict(total=sum(p.read_text().count('(check-valid') for p in files),bv=sum(p.read_text().count('(check-valid') for p in bv),eq=sum(p.read_text().count('(check-valid') for p in eq),files=len(files)))
 if kind=='full':
  need(counts[0]['eq']==3,'three fresh Eq checks')
  a,b=[v['helpers'][0] for v in inv];need(len(inv[0]['helpers'])==len(inv[1]['helpers'])==1,'Eq only')
  need(a['candidate_id']==b['candidate_id'] and a['metrics']['cache']=='miss' and b['metrics']['cache']=='hit','real Eq miss/hit')
  for h in [a,b]:
   need(h['contract']['requires']=='true','Eq full domain')
   for k in ['soundness','maximality','exactness']:need(h['proofs'][k]['required'] and h['proofs'][k]['status']=='proved','fresh precision '+k)
  # The genuine helper remains a normal checked query, with identical C/W AIR.
  air=(base/'cold/vir/root-final.air').read_text();a=air.index(';; Function-Def component::checked_foreign_bool_to_u64%\n');b=air.find('\n;; Function-',a+1)
  need('(check-valid' in air[a:b if b>=0 else len(air)],'helper checked')
 write(base/'comparison.json',dict(certificates=cert,inventories=inv,query_counts=counts,air_sha256=airs[0],identical_inputs=True,queries=queries(base/'cold/vir')))
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B')
 mode=sys.argv[1]
 source=(R/'full.rs').read_text() if mode=='full' else BOOL if mode=='bool' else (ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text()
 if mode=='packet-public-api':source=source.replace('fn packet(','pub fn packet(').replace('const CHANNEL_MASK','pub const CHANNEL_MASK')
 stable(mode,source)
