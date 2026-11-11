"""Own matching-build review audit. Exception gates remain active under Python -O."""
from trials import *
from baseline import queries
from worker_inventory import typed, field, end
from tests import TARGETS
E=ROOT/'source/tools/research/packing_dedup';W=Path('/tmp/packing-dedup-hd1n_rpr');A=Path('/tmp/packing-dedup-cb7lqob8')
def check():
 final=json.loads((R/'FINAL_TOOLS.json').read_text());tools=final['tools'];sources=manifest(ROOT)
 need(final['source']==sources==manifest(R/'src'),'shipping source equals tested source')
 need(tools=={n:sha(R/'bundle'/n) for n in TOOLS},'final tools immutable')
 need(sha(ROOT/'source/rust_verify/src/checked_foreign_packing.rs')=='d384bdd6091b89a83612529e6dc9bfba1573dd7c35018941f6befacb3b536f72','reviewed module')
 changed=subprocess.check_output(['git','diff','--name-only','21eeabbb','5d79f711','--','source/rust_verify','source/vir'],cwd=ROOT).decode().splitlines()
 need(set(changed)=={'source/rust_verify/src/checked_foreign_packing.rs','source/rust_verify/src/import_export.rs','source/vir/src/infer/registry.rs'},'review scope')
 old=subprocess.check_output(['git','show','21eeabbb:source/rust_verify/src/checked_foreign_packing.rs'],cwd=ROOT).decode();new=(ROOT/'source/rust_verify/src/checked_foreign_packing.rs').read_text()
 need(old[old.index('fn number('):old.index('    for fact in proposed {')]==new[new.index('fn number('):new.index('    append_checked(&mut stms, proposed);')],'unchanged discovery/formulas/bounds')
 emission=old[old.index('    for fact in proposed {'):old.index('    stms.push(check.body.clone());')]
 need(emission.replace('    for fact in proposed {','    for fact in proposed {\n        if closed_key(&fact).is_some_and(|key| !seen.insert(key)) {\n            continue;\n        }') in new,'unchanged original Arc emitter')
 rs={}
 for p in sorted(R.rglob('receipt.json')):
  if not (p.parent/'freeze.json').exists():continue
  label=str(p.parent.relative_to(R));j=json.loads(p.read_text());f=json.loads((p.parent/'freeze.json').read_text())
  need(j['guards'],'guards '+label)
  for name in ['freeze','stdout','stderr']:need(sha(p.parent/('freeze.json' if name=='freeze' else name))==j[name+'_sha256'],'raw '+label+'/'+name)
  need(f['source']==sources,'trial source '+label)
  if label=='vstd':need(all(tools[n]==h for n,h in f['tools'].items()),'matching vstd prerequisites')
  elif label not in ['build']:need(f['tools']==tools,'same frozen tool bundle '+label)
  for n,h in f['fixtures'].items():need(sha(n)==h,'fixture '+label)
  rs[label]=dict(receipt_sha256=sha(p),freeze_sha256=j['freeze_sha256'],stdout_sha256=j['stdout_sha256'],stderr_sha256=j['stderr_sha256'],exit=j['exit'],verification=j.get('verification'),seconds=j['seconds'],argv=f['argv'])
 need(rs['build']['exit']==rs['vstd']['exit']==0,'build/vstd pass')
 need('verification results:: 2250 verified, 0 errors' in (R/'vstd/stdout').read_text(),'vstd all2250')
 toolchain=json.loads((R/'toolchain.json').read_text())
 for path,h in toolchain['files'].items():need(sha(path)==h,'installed Rust input unchanged')
 need('rustc 1.99.1' in toolchain['rustc'],'required Rust toolchain')
 preparation=json.loads((R/'bundle-preparation.json').read_text());need(preparation['after']['sha256']==tools['rust_verify'],'finalized before vstd')
 for label in ['build','test-build','unit-build']:
  args=rs[label]['argv'];need('--offline' in args and '--locked' in args and '-j' in args and args[args.index('-j')+1]=='2','locked offline jobs2')
 test_artifacts=json.loads((R/'test-artifacts.json').read_text())
 for row in test_artifacts.values():need(sha(row['path'])==row['sha256'],'built test executable unchanged')
 for t,count in {'vir':149,'rust_verify-unit':6,**TARGETS}.items():
  label='test-'+t;need(rs[label]['exit']==0,'test pass '+t)
  m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',(R/label/'stdout').read_text());need(m and tuple(map(int,m.groups()))==(count,0,0),'complete test count '+t)
 comps={}
 for kind in ['full','packet']:
  before=queries(A/('stable-'+kind)/'cold/vir');after=queries(R/('stable-'+kind)/'cold/vir');worker=queries(W/('stable-'+kind)/'cold/vir')
  sig=lambda rows:[(x['owner'],x['formula_sha256']) for x in rows]
  need(sig(after)==sig(worker)==sig([x for x in before if x['keep']]),'ordered exact retained inventory '+kind)
  need(typed(R/('stable-'+kind)/'cold')==typed(W/('stable-'+kind)/'cold')==typed(A/('stable-'+kind)/'cold'),'typed entire function/domain/contract/visibility/native inventory '+kind)
  c=json.loads((R/('stable-'+kind)/'comparison.json').read_text())
  for ep in ['cold','warm']:
   d=R/('stable-'+kind)/ep
   need({p.name:sha(p) for p in (d/'vir').glob('*.air')}==c['air_sha256'],'raw AIR C/W identity '+kind)
   need(typed(R/('stable-'+kind)/'cold',cw=True)==typed(R/('stable-'+kind)/'warm',cw=True),'C/W complete typed function inventories '+kind)
   need(rs['stable-'+kind+'/'+ep]['exit']==0 and rs['stable-'+kind+'/'+ep]['verification']['errors']==0,'successful proof '+kind+'/'+ep)
  v=rs['stable-'+kind+'/cold']['verification'];oldv=json.loads((A/('stable-'+kind)/'cold/receipt.json').read_text())['verification']
  need(oldv['verified']-len(before)==v['verified']-len(after),'all ordinary+Eq queries retained')
  ts=typed(R/('stable-'+kind)/'cold')
  comps[kind]=dict(before=oldv,after=v,original_BV=len(before),kept_BV=len(after),removed_BV=len(before)-len(after),queries_before=before,queries=after,removed=[x for x in before if not x['keep']],typed=ts,air_sha256=c['air_sha256'],typed_log_sha256=sha(R/('stable-'+kind)/'cold/vir/crate.vir'))
 need(sha(R/'full.rs')=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','frozen full source')
 eq=[]
 for ep in ['cold','warm']:
  d=R/'stable-full'/ep;j=json.loads((d/'stdout').read_text());h=j['verus-infer']['functions'];need(len(h)==1,'Eq only');h=h[0]
  need(h['contract']['requires']=='true' and all(h['proofs'][k]['required'] and h['proofs'][k]['status']=='proved' for k in ['soundness','maximality','exactness']),'fresh full-domain Eq3')
  need(h['metrics']['cache']==('miss' if ep=='cold' else 'hit'),'real miss/hit')
  cert=metadata(d/'component.vir')[1];need(cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v72','policy72')
  air=(d/'vir/root-final.air').read_text();a=air.index(';; Function-Def component::checked_foreign_bool_to_u64%\n');b=air.find('\n;; Function-',a+1);helper=air[a:b if b>=0 else len(air)]
  need('(check-valid' in helper,'ordinary helper actual query')
  eq.append(dict(candidate_id=h['candidate_id'],cache=h['metrics']['cache'],proofs=h['proofs'],helper_air_sha256=hashlib.sha256(helper.encode()).hexdigest(),foreign_dependencies_sha256=hashlib.sha256(json.dumps(cert['foreign_dependencies'],sort_keys=True).encode()).hexdigest(),certificate_summary_sha256=hashlib.sha256(json.dumps(cert['summaries'],sort_keys=True).encode()).hexdigest()))
 fixture_path=next(iter(json.loads((R/'stable-full/cold/freeze.json').read_text())['fixtures']))
 tokens=lambda ep:re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',(R/'stable-full'/ep/'vir/crate.vir').read_text())
 x=tokens('cold');y=tokens('warm');need(len(x)==len(y),'C/W full typed token lengths')
 differences=[(a,b) for a,b in zip(x,y) if a!=b]
 need(differences==[('"'+fixture_path+':5:19: 5:31 (#18)"','"'+fixture_path+':4:29: 4:38 (#18)"')],'only known Eq ensure diagnostic span relocates in complete C/W typed log')
 need(eq[0]['candidate_id']==eq[1]['candidate_id'] and eq[0]['helper_air_sha256']==eq[1]['helper_air_sha256'] and eq[0]['foreign_dependencies_sha256']==eq[1]['foreign_dependencies_sha256'],'C/W identities')
 for ep in ['cold','warm']:
  text=(R/('observer-'+ep)/'vir/crate.vir').read_text()
  for m in re.finditer(r'\(Function\s',text):
   form=text[m.start():end(text,m.start())];name=field(form,'name')
   if 'component::' in name and name.endswith(('::new)','::perm)','::protected)','::word)','::value)','::permission)')):need(field(form,'body')=='None','imported hidden body pruned '+name)
 packet=(ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text()
 for kind,want in [('packet',packet),('packet-public-api',packet.replace('fn packet(','pub fn packet(').replace('const CHANNEL_MASK','pub const CHANNEL_MASK'))]:
  f=json.loads((R/('stable-'+kind)/'cold/freeze.json').read_text());need(len(f['fixtures'])==1 and Path(next(iter(f['fixtures']))).read_text()==want,'natural/public adapter source '+kind)
 controls={}
 for label,counts in {'bool-provider':(2,0),'bool-client':(1,0),'false-client':(0,1),'goals-off':(0,1),'observer-cold':(5,0),'observer-warm':(5,0),'observer-false':(4,1),'constructor-protector-limit':(0,1),'constructor-permission-limit':(0,1),'packet-client':(1,0),'reexport':(1,0),'transitive-client':(1,0),'private-provider':(3,0),'private-client':(1,0)}.items():
  v=rs[label]['verification'];need((v['verified'],v['errors'])==counts,'semantic result '+label);need((rs[label]['exit']==0)==(counts[1]==0),'semantic exit '+label)
  controls[label]=v
 need(json.loads((R/'observer-false/stdout').read_text())['verus-infer']['failed_functions']==['observer_false::protector'],'named false protector goal')
 for label in ['bool-client','false-client','observer-cold','observer-warm','observer-false','constructor-protector-limit','packet-client','transitive-client']:
  need('--infer-goals' not in rs[label]['argv'],'ordinary consumer '+label)
 aux={}
 for mode in ['explicit','cargo']:
  for control in ['failed-bv-auxiliary','erased-type-failure-v2']:
   label=mode+'-'+control;d=R/label;need(rs[label]['exit']!=0,'failure expected '+label)
   need(not (d/('libprovider.vir' if mode=='cargo' else 'provider.vir')).exists(),'stale selected export withheld '+label)
   j=json.loads((d/'stdout').read_text());v=j['verification-results'];err=(d/'stderr').read_text()
   if control=='failed-bv-auxiliary':
    need(v['errors']==1 and j['verus-infer']['failed_functions']==[],'auxiliary-only global failure')
    rows=queries(d/'vir');total=sum(p.read_text().count('(check-valid') for p in (d/'vir').glob('*-final.air'))
    air=(d/'vir/root-final.air').read_text();chunks=re.split(r'(?m)(?=^;; Function-)',air);recommends=sum(c.count('(check-valid') for c in chunks if c.startswith(';; Function-Recommends '))
    helper=[c for c in chunks if c.startswith(';; Function-Def provider::checked_foreign_bool_to_u64%\n')];ordinary=[c for c in chunks if c.startswith(';; Function-Def provider::all_bool\n')]
    need(len(helper)==len(ordinary)==1 and '(check-valid' in helper[0] and '(check-valid' in ordinary[0],'helper+ordinary checks present')
    need(total-recommends==v['verified']+v['errors'],'all genuine queries accounted')
    reference=queries(W/label/'vir');need([(x['owner'],x['formula_sha256']) for x in rows]==[(x['owner'],x['formula_sha256']) for x in reference],'exact auxiliary generated/false query inventories')
    aux[label]=dict(verification=v,failed_functions=[],stale_withheld=True,queries=rows,genuine_queries=total-recommends,diagnostic_recommends_queries=recommends,helper_air_sha256=hashlib.sha256(helper[0].encode()).hexdigest(),ordinary_air_sha256=hashlib.sha256(ordinary[0].encode()).hexdigest())
   else:need(v['verified']==2 and v['errors']==0 and 'E0308' in err and 'erased_type_error' in err,'erased-only failure after successful proofs')
 artifacts={}
 for label in rs:
  d=R/label
  artifacts[label]={p.name:sha(p) for ext in ['*.vir','*.rlib','*.rmeta'] for p in d.glob(ext)}
 artifacts['cache74-output']={p.name:sha(p) for ext in ['*.vir','*.rlib','*.rmeta'] for p in (R/'stable-full/run').glob(ext)}
 versions=json.loads((R/'version-controls.json').read_text());need(versions['stale_policy_refused'] and versions['cache74_miss'] and versions['fresh_three_precision_proofs'],'version controls')
 return dict(prepared_commit='5d79f711adc331d917e3b55271f44533c01903fb',findings=[],scope='typed per-function packing identity selection and policy72/cache75 compatibility',tools=tools,toolchain=toolchain,offline_inputs_receipt_sha256=sha(R/'bootstrap.json'),source_manifest_sha256=hashlib.sha256(json.dumps(sources,sort_keys=True).encode()).hexdigest(),source_files=len(sources),receipts=rs,bundle_preparation=preparation,tests=dict(VIR=149,compiler_library=6,focused_compiler=40),test_artifacts=test_artifacts,comparisons=comps,Eq=eq,controls=controls,auxiliary=aux,exported_artifacts=artifacts,versions=versions,fidelity=json.loads((R/'fidelity.json').read_text()),full_source_sha256=sha(R/'full.rs'),trust='unchanged compiler/backend/BV/Z3/model/vstd/imported-artifact trust; certificates are compiler-produced metadata, not independent proof objects',native_correspondences=0,publication=False,goal_complete=False)
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'python -B required');j=check();p=HERE/'RESULTS.json'
 if '--write' in sys.argv:write(p,j)
 else:need(json.loads(p.read_text())==j,'deterministic evidence')
 print('PASS matching review audit')
