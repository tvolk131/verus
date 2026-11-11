"""Archive restored cold outputs before same-path warm replay using final tool identities."""
import review as t
import audit as a
import shutil,json,hashlib
R=t.root();out={}
for target in ['full','packet']:
 label='restored-'+target;cold=R/(label+'-cold');work=R/label;t.need(not cold.exists(),'immutable restored cold archive');shutil.move(work,cold)
 if (cold/'candidate-cache').exists():shutil.copytree(cold/'candidate-cache',work/'candidate-cache')
 j=t.trial(label,R/target/'component.rs');t.need(j['exit']==0 and j['verification']['errors']==0,'restored warm success')
 f=json.loads((cold/'freeze.json').read_text());g=json.loads((work/'freeze.json').read_text());t.need(f==g,'identical final-tool cold/warm inputs')
 air={p.name:t.sha(p) for p in (cold/'vir').glob('*-final.air')};warm={p.name:t.sha(p) for p in (work/'vir').glob('*-final.air')};t.need(air==warm and len(air)==(118 if target=='full' else 30),'all final-tool AIR files identical');t.need(a.bodies(cold)==a.bodies(work) and a.helper(cold)==a.helper(work),'final body/helper identity')
 if target=='full':
  x,y=a.eq(cold),a.eq(work);t.need(x['candidate_id']==y['candidate_id'] and x['metrics']['cache']=='miss' and y['metrics']['cache']=='hit','final-tool real candidate miss/hit')
 out[target]=dict(cold=str(cold),warm=str(work),AIR=air,helper_query_sha256=hashlib.sha256(a.helper(work).encode()).hexdigest(),Eq=a.eq(work) if target=='full' else None,verification=j['verification'],tools=g['tools'])
t.write(t.HERE/'RESTORED_COLD_WARM.json',out)
