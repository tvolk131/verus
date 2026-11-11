"""Deterministic exception gates, usable under -O; no old audits are executed."""
from trials import *
from baseline import queries
sys.path.insert(0,str(ROOT/'source/tools/research/item_tag_constructor_composition'))
from inventory import end,inventory
A=Path((HERE/'ABLATION_ROOT').read_text().strip())
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

def main():
 need(sys.flags.dont_write_bytecode,'use -B');tools,rs=receipts(R);atools,ars=receipts(A)
 current=manifest(ROOT);need(current==manifest(R/'src'),'shipping source equals tested source')
 changed=subprocess.check_output(['git','diff','--name-only','efd9fb56','--','source','dependencies','tools','rust-toolchain.toml'],cwd=ROOT).decode().splitlines()
 need({p for p in changed if not p.startswith('source/tools/research/packing_dedup/')}=={'source/rust_verify/src/checked_foreign_packing.rs'},'only authorized production file')
 original=subprocess.check_output(['git','show','efd9fb56:source/rust_verify/src/checked_foreign_packing.rs'],cwd=ROOT).decode()
 tested=(R/'src/source/rust_verify/src/checked_foreign_packing.rs').read_text()
 need(original[original.index('fn number('):original.index('    for fact in proposed {')]==tested[tested.index('fn number('):tested.index('    append_checked(&mut stms, proposed);')],'all discovery/formula generation and bounds unchanged')
 atext=(A/'src/source/rust_verify/src/checked_foreign_packing.rs').read_text();text=(R/'src/source/rust_verify/src/checked_foreign_packing.rs').read_text()
 selector='if closed_key(&fact).is_some_and(|key| !seen.insert(key)) {'
 need(atext==text.replace(selector,'if false && closed_key(&fact).is_some_and(|key| !seen.insert(key)) {'),'only selector ablation')
 am=manifest(A/'src');need({p for p in current if current[p]!=am[p]}=={'source/rust_verify/src/checked_foreign_packing.rs'},'only packing source differs')
 authpath=Path('/tmp/verus-infer-buffer-cycle-20261109/certificate-combined-review-auth/receipt.json');auth=json.loads(authpath.read_text());old=ROOT/'evidence/certified-packing-combined-review'
 need(sha(old/'RESULTS.json')==auth['result_sha256'] and sha(old/'SEAL.json')==auth['seal_sha256'],'completed review authentication')
 seal=json.loads((old/'SEAL.json').read_text())
 for name,h in seal['files'].items():need(sha(old/name)==h,'sealed completed review '+name)
 result=dict(base='efd9fb5680ece75f871058677afe10c9ce7e64eb',preregistration_commit='8c2445a0',tools=tools,ablation_tools=atools,receipts=rs,ablation_receipts=ars,comparisons={},publication=False,goal_complete=False,native_correspondences=0)
 result['completed_review_authentication_sha256']=sha(authpath)
 for kind in ['full','packet']:
  ds=[r/('stable-'+kind)/'cold' for r in [A,R]]
  rows=[queries(d/'vir') for d in ds]
  before,after=rows
  historical=json.loads((HERE/'BASELINE.json').read_text())['cases'][kind]['queries']
  need([(x['owner'],x['formula_sha256']) for x in before]==[(x['owner'],x['formula_sha256']) for x in historical],'fresh disabled reproduces historical original formulas '+kind)
  want=[(x['owner'],x['formula_sha256']) for x in before if x['keep']];actual=[(x['owner'],x['formula_sha256']) for x in after]
  need(want==actual,'ordered exact retained AIR formulas '+kind)
  need(all(x['keep'] for x in after),'no surviving duplicate AIR '+kind)
  ts=[typed(d) for d in ds];need(ts[0]==ts[1],'all typed bodies/domains/native classifications '+kind)
  vcs=[json.loads((d/'receipt.json').read_text())['verification'] for d in ds]
  need(vcs[0]['errors']==vcs[1]['errors']==0,'ordinary semantic proof results')
  need(vcs[0]['verified']-len(before)==vcs[1]['verified']-len(after),'all non-BV checks retained')
  comps=[json.loads((r/('stable-'+kind)/'comparison.json').read_text()) for r in [A,R]]
  result['comparisons'][kind]=dict(before=vcs[0],after=vcs[1],original_BV=len(before),retained_BV=len(after),removed_BV=len(before)-len(after),ordered_retained_exact=True,queries_before=before,queries_after=after,typed=ts[1],candidate_id_preserving_function_hashes=[typed(d,raw=True) for d in ds],typed_log_raw_sha256=[sha(d/'vir/crate.vir') for d in ds],typed_before_after_equal=True,query_counts=[v['query_counts'] for v in comps],wall_seconds={name:[(ars if name=='disabled' else rs)['stable-'+kind+'/'+ep]['seconds'] for ep in ['cold','warm']] for name in ['disabled','enabled']})
  for r in [A,R]:
   cold=r/('stable-'+kind)/'cold';warm=cold.parent/'warm'
   for ep in [cold,warm]:
    j=json.loads((ep/'stdout').read_text());need(j['verification-results']['errors']==0,'C/W no failures')
   need(typed(cold,cw=True)==typed(warm,cw=True),'typed C/W with only the exact known Eq ensure diagnostic relocation')
   if kind=='full':
    fixture=next(iter(json.loads((cold/'freeze.json').read_text())['fixtures']))
    tokens=lambda p:re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',p.read_text())
    x=tokens(cold/'vir/crate.vir');y=tokens(warm/'vir/crate.vir')
    need(len(x)==len(y),'C/W typed token lengths')
    differences=[(a,b) for a,b in zip(x,y) if a!=b]
    need(differences==[('"'+fixture+':5:19: 5:31 (#18)"','"'+fixture+':4:29: 4:38 (#18)"')],'EXACTLY one known diagnostic token differs in complete C/W typed log')
  if kind=='full':
   need(len(ts[1]['functions'])==38,'full typed inventory')
   result['full_source_sha256']=sha(R/'full.rs');need(result['full_source_sha256']=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','frozen full generator')
   result['fidelity']=json.loads((R/'fidelity.json').read_text())
   eq=[]
   for ep in ['cold','warm']:
    d=R/'stable-full'/ep;j=json.loads((d/'stdout').read_text());h=j['verus-infer']['functions'];need(len(h)==1,'one Eq candidate');h=h[0]
    need(h['contract']['requires']=='true','full Eq domain')
    need(all(h['proofs'][k]['required'] and h['proofs'][k]['status']=='proved' for k in ['soundness','maximality','exactness']),'fresh Eq3')
    need(h['metrics']['cache']==('miss' if ep=='cold' else 'hit'),'real miss/hit')
    cert=metadata(d/'component.vir')[1];air=(d/'vir/root-final.air').read_text();a=air.index(';; Function-Def component::checked_foreign_bool_to_u64%\n');b=air.find('\n;; Function-',a+1);helper=air[a:b if b>=0 else len(air)];need('(check-valid' in helper,'normal checked foreign helper')
    eq.append(dict(candidate_id=h['candidate_id'],cache=h['metrics']['cache'],proofs=h['proofs'],helper_air_sha256=hashlib.sha256(helper.encode()).hexdigest(),foreign_certificate_sha256=hashlib.sha256(json.dumps(cert['foreign_dependencies'],sort_keys=True).encode()).hexdigest(),API_summary_sha256=hashlib.sha256(json.dumps(cert['summaries'],sort_keys=True).encode()).hexdigest()))
   need(eq[0]['candidate_id']==eq[1]['candidate_id'] and eq[0]['helper_air_sha256']==eq[1]['helper_air_sha256'] and eq[0]['foreign_certificate_sha256']==eq[1]['foreign_certificate_sha256'],'full helper/cert identities')
   result['Eq']=eq
   result['Eq_diagnostic_relocation']=dict(function='component::impl&%5::eq',field='ensure',cold='stable-full.rs:5:19: 5:31 (#18)',warm='stable-full.rs:4:29: 4:38 (#18)',scope='one diagnostic span only; raw typed logs and API summaries preserved')
   result['ablation_Eq_candidate_id']=json.loads((A/'stable-full/cold/stdout').read_text())['verus-infer']['functions'][0]['candidate_id']
 packet=(ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text()
 for kind,expected in [('packet',packet),('packet-public-api',packet.replace('fn packet(','pub fn packet(').replace('const CHANNEL_MASK','pub const CHANNEL_MASK'))]:
  f=json.loads((R/('stable-'+kind)/'cold/freeze.json').read_text());need(len(f['fixtures'])==1,'one packet source');need(Path(next(iter(f['fixtures']))).read_text()==expected,'packet source/visibility only '+kind)
 for ep in ['cold','warm']:
  d=R/('observer-'+ep);t=(d/'vir/crate.vir').read_text()
  for m in re.finditer(r'\(Function\s',t):
   form=t[m.start():end(t,m.start())];name=field(form,'name')
   if 'component::' in name and name.endswith(('::new)','::perm)','::protected)','::word)','::value)','::permission)')):need(field(form,'body')=='None','imported hidden body pruned '+name)
 from tests import TARGETS
 for t,count in {'vir':149,'rust_verify-unit':6,**TARGETS}.items():
  label='test-'+t;need(rs[label]['exit']==0,'test exit '+t);out=(R/label/'stdout').read_text();m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',out);need(m and int(m[1])==count and int(m[2])==0 and int(m[3])==0,'test count '+t)
 for label,expected in {'bool-provider':(2,0),'bool-client':(1,0),'false-client':(0,1),'observer-cold':(5,0),'observer-warm':(5,0),'observer-false':(4,1),'constructor-protector-limit':(0,1),'packet-client':(1,0)}.items():
  v=rs[label]['verification'];need((v['verified'],v['errors'])==expected,'exact semantic control '+label)
 need('verification results:: 2250 verified, 0 errors' in (R/'vstd/stdout').read_text(),'matching vstd2250/0')
 need('verification results:: 2250 verified, 0 errors' in (A/'vstd/stdout').read_text(),'ablation matching vstd2250/0')
 need(rs['vstd']['exit']==0 and ars['vstd']['exit']==0,'matching vstd exits')
 need(json.loads((R/'observer-false/stdout').read_text())['verus-infer']['failed_functions']==['observer_false::protector'],'named false protector goal')
 for entry in ['explicit','cargo']:
  for control in ['failed-bv-auxiliary','erased-type-failure-v2']:
   d=R/(entry+'-'+control);need(rs[d.name]['exit']!=0,'failure expected '+d.name)
   need(not (d/('libprovider.vir' if entry=='cargo' else 'provider.vir')).exists(),'stale withhold '+d.name)
   j=json.loads((d/'stdout').read_text())
   if control=='failed-bv-auxiliary':
    need(j['verus-infer']['failed_functions']==[] and j['verification-results']['errors']==1,'global auxiliary rejection')
    rows=queries(d/'vir');isolated=[p for p in (d/'vir').glob('*-final.air') if 'bitvector' in p.read_text()]
    # Query inventory, NOT a success threshold. Exactly one isolated false check.
    need(sum(p.read_text().count('(check-valid') for p in isolated)==len(rows),'all isolated queries inventoried')
    total=sum(p.read_text().count('(check-valid') for p in (d/'vir').glob('*-final.air'))
    root_air=(d/'vir/root-final.air').read_text()
    chunks=re.split(r'(?m)(?=^;; Function-)',root_air)
    recommends=sum(c.count('(check-valid') for c in chunks if c.startswith(';; Function-Recommends '))
    v=j['verification-results'];need(total-recommends==v['verified']+v['errors'],'auxiliary genuine result inventory, separating diagnostic recommends queries')
    result.setdefault('auxiliary',{})[entry]=dict(queries=rows,verification=j['verification-results'],genuine_queries=total-recommends,diagnostic_recommends_queries=recommends,failed_functions=[],stale_withheld=True)
   else:need(j['verification-results']['verified']==2 and 'E0308' in (d/'stderr').read_text(),'erased-only type rejection')
 result['source_manifest_sha256']=hashlib.sha256(json.dumps(current,sort_keys=True).encode()).hexdigest()
 out=HERE/'RESULTS.json'
 if '--write' in sys.argv:write(out,result)
 else:need(json.loads(out.read_text())==result,'deterministic final evidence')
 print('PASS',json.dumps({k:{n:v[n] for n in ['original_BV','retained_BV','removed_BV','wall_seconds']} for k,v in result['comparisons'].items()}))
if __name__=='__main__':main()
