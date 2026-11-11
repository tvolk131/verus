"""Read-only acceptance audit; the same checks run under -B and -O -B."""
from trials import *
sys.path.insert(0,str(ROOT/'source/tools/research/item_tag_constructor_composition'))
from inventory import inventory,signature

def receipt(label):
 d=R/label;j=json.loads((d/'receipt.json').read_text());f=json.loads((d/'freeze.json').read_text())
 need(j['guards'],'failed freeze '+label)
 for n in ['stdout','stderr']:need(sha(d/n)==j[n+'_sha256'],'raw log drift '+label)
 need(sha(d/'freeze.json')==j['freeze_sha256'],'freeze drift '+label)
 return d,j,f

def compact(label):
 d,j,f=receipt(label)
 return dict(path=str(d),receipt_sha256=sha(d/'receipt.json'),freeze_sha256=j['freeze_sha256'],stdout_sha256=j['stdout_sha256'],stderr_sha256=j['stderr_sha256'],exit=j['exit'],verification=j.get('verification'),source_fixtures=f['fixtures'],artifact_outputs={p.name:sha(p) for suffix in ['*.vir','*.rlib','*.rmeta'] for p in sorted(d.glob(suffix))})

def inventory_case(label,crate):
 x=inventory(R/label,crate)
 return {k:v for k,v in x.items() if k!='local_function_details'} | {'function_detail_names':sorted(x['local_function_details'])}

def main():
 need(sys.flags.dont_write_bytecode,'use -B')
 preflight='--preflight' in sys.argv
 need(sha(P/'receipt.json')=='f24584b8ccfde3ac92e84c146d87317b04231b2b165ef759411a0c53ea3199e3','accepted parent bundle')
 tools=json.loads((R/'final-tool-identities.json').read_text())
 for n,h in tools.items():
  need(sha(R/'bundle'/n)==h and sha(R/'frozen-final-bundle'/n)==h,'frozen final tool '+n)
 positives={'bool-provider':2,'bool-provider-warm':2,'bool-client':1,'reexport':1,'transitive-client':1,'private-provider':3,'private-client':1,'full-provider':52,'full-provider-warm':52,'full-observer':5,'unused-reexport':1,'unused-consumer':1,'explicit-ordinary':1,'explicit-infer':1,'cargo-ordinary':1,'cargo-infer':1,'cargo-provider':2,'cargo-client':1,'cpu-provider':2,'cpu-matching-client':1,'boundary-qualified':2,'boundary-alias':2,'native-call-identity':2}
 selected=set(positives)
 for label,count in positives.items():
  d,j,f=receipt(label);need(j['exit']==0 and j['verification']['verified']==count and j['verification']['errors']==0,'positive proof '+label)
  if label in ['bool-client','reexport','transitive-client','private-client','full-observer','unused-reexport','unused-consumer','cargo-client']:
   need('--infer-goals' not in f['argv'],'ordinary consumer gained goals '+label)
 for label,v,e in [('false-client',0,1),('full-observer-false',4,1),('goals-off',0,1),('explicit-proof',1,1),('cargo-proof-valid',1,1),('explicit-failed-bv-auxiliary',4,1),('cargo-failed-bv-auxiliary',4,1)]:
  d,j,f=receipt(label);selected.add(label);need(j['exit']!=0 and (j['verification']['verified'],j['verification']['errors'])==(v,e),'healthy proof failure '+label)
  if label=='goals-off':
   need(not any(p.read_bytes().endswith(b'VRINFER1') for p in d.glob('*.vir')),'goals-off acquired a certificate')
   need(not json.loads((d/'stdout').read_text())['verus-infer']['interface_written'],'goals-off certified interface')
  else:need(not list(d.glob('*.vir')),'failed dependent proof published '+label)
 for label in ['explicit-failed-bv-auxiliary','cargo-failed-bv-auxiliary']:
  data=json.loads((R/label/'stdout').read_text());need(data['verus-infer']['failed_functions']==[],'auxiliary must be distinguished from main-body failure')
 for label in ['explicit-erased-type-failure-v2','cargo-erased-type-failure-v2']:
  d,j,f=receipt(label);selected.add(label);need(j['exit']!=0 and j['verification']['verified']==2 and j['verification']['errors']==0,'erased failure must follow healthy proof')
  need('E0308' in (d/'stderr').read_text() and not list(d.glob('*.vir')),'early publication before erased checks')
 for prefix in ['refuse-','explicit-consumer-','cargo-consumer-','foreign-conflict-','missing-']:
  for p in sorted(R.glob(prefix+'*/receipt.json')):
   label=p.parent.name;d,j,f=receipt(label);selected.add(label);need(j['exit']!=0,'negative accepted '+label)
   need(not list(d.glob('*.vir')),'refusal published '+label)
 for label in ['transitive-missing','actual-target-mismatch','conflicting-completed-artifacts','boundary-function-value','boundary-coercion','legacy-consumer-v2','new-consumer-old-v2']:
  d,j,f=receipt(label);selected.add(label);need(j['exit']!=0,'negative accepted '+label)
 for prefix in ['explicit','cargo']:
  for control in ['proof','parse','type','no-verify','no-lifetime','no-trait-conflicts','no-erasure-check','verify-root','verify-module-other']:
   labels=[prefix+'-'+control+'-valid',prefix+'-'+control]
   label=next(x for x in labels if (R/x/'receipt.json').exists());d,j,f=receipt(label);selected.add(label)
   need(j['exit']!=0 and not list(d.glob('*.vir')),'stale selected output '+label)
   if control=='no-erasure-check':need('--compile' not in f['argv'],'invalid no-erasure control')
 for shape in ['extra-filename','explicit-emit','single-output']:
  for stage in ['complete','parse','type']:
   label='cargo-path-'+shape+'-'+stage;d,j,f=receipt(label);selected.add(label)
   need((j['exit']==0)==(stage=='complete'),'cargo selected path '+label)
   need(bool(list(d.glob('*.vir')))==(stage=='complete'),'cargo output existence '+label)
 for kind in ['bool','full']:
  base=R/('stable-'+kind);cmp=json.loads((base/'comparison.json').read_text())
  need(cmp['identical_inputs'],'identical run inputs')
  fs=[]
  for epoch in ['cold','warm']:
   label='stable-'+kind+'/'+epoch;d,j,f=receipt(label);selected.add(label);fs.append(f)
   need(j['exit']==0 and j['verification']['errors']==0,'stable proof '+label)
  need(fs[0]==fs[1],'actual cold/warm freeze identity')
  need(signature(base/'cold')==signature(base/'warm'),'actual ordinary typed identity')
  certs=[metadata(base/e/'component.vir')[1] for e in ['cold','warm']]
  need(certs[0]['foreign_dependencies']==certs[1]['foreign_dependencies'],'actual foreign certificate identity')
  queries=[]
  for epoch in ['cold','warm']:
   text=(base/epoch/'vir/root-final.air').read_text();a=text.index(';; Function-Def component::checked_foreign_bool_to_u64%\n');b=text.find('\n;; Function-',a+1);queries.append(text[a:b if b>=0 else len(text)])
  need(queries[0]==queries[1] and hashlib.sha256(queries[0].encode()).hexdigest()==cmp['helper_query_sha256'],'actual helper query identity')
  if kind=='full':
   a,b=cmp['Eq'];need(a['candidate_id']==b['candidate_id'] and a['metrics']['cache']=='miss' and b['metrics']['cache']=='hit','fresh Eq miss/hit identity')
   need(cmp['Eq_air'][0]==cmp['Eq_air'][1] and cmp['Eq_air'][0]['check_valid_count']==3,'three fresh identical Eq queries')
   for h in [a,b]:
    need(h['contract']['requires']=='true','Eq full domain')
    for k in ['soundness','maximality','exactness']:need(h['proofs'][k]['required'] and h['proofs'][k]['status']=='proved','fresh required Eq '+k)
 # Required vectors are independent of helper/public API pruning.
 interfaces={}
 for label,name in [('bool-provider','provider'),('private-provider','private'),('reexport','bridge'),('transitive-client','third'),('unused-reexport','unused'),('unused-consumer','unused_client'),('cargo-provider','libprovider'),('cargo-client','libcargo_client'),('full-provider','component'),('cpu-provider','provider')]:
  _,m=metadata(R/label/(name+'.vir'));need(m['schema']==39 and m['policy']=='candidate-bound-evidence-v71','reserved certificate format')
  need(len(m['foreign_dependencies'])==1,'required vector lost '+label)
  interfaces[label]=m
 for label in ['reexport','transitive-client','unused-reexport','unused-consumer']:
  need(interfaces[label]['foreign_dependencies']==interfaces['bool-provider']['foreign_dependencies'],'foreign propagation '+label)
 # Imported fingerprints reported by the verifier bind the actual immutable
 # producer certificates. This supplements fixed input paths with content.
 def fingerprint(m):return hashlib.sha256(json.dumps(m,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
 for consumer,providers in [('bool-client',['bool-provider']),('reexport',['bool-provider']),('transitive-client',['bool-provider','reexport']),('private-client',['private-provider']),('full-observer',['full-provider']),('unused-reexport',['bool-provider']),('unused-consumer',['bool-provider','unused-reexport']),('cargo-client',['cargo-provider']),('cpu-matching-client',['cpu-provider'])]:
  j=json.loads((R/consumer/'stdout').read_text());actual={d['fingerprint'] for d in j['verus-infer']['imported_inference_dependencies']}
  need(actual=={fingerprint(interfaces[x]) for x in providers},'imported content binding '+consumer)
 # Exact own/foreign-body inventories and opacity; counts alone are not evidence.
 inv={label:inventory_case(label,crate) for label,crate in [('bool-provider','provider'),('full-provider','component'),('bool-client','bool_client'),('full-observer','observer'),('private-provider','private')]}
 need(len(inv['bool-provider']['ordinary_original_functions'])==2,'bool ordinary inventory')
 need(len(inv['full-provider']['ordinary_original_functions'])==39,'full ordinary inventory')
 need(inv['full-provider']['native_scope']['checked_exec_bodies']==16 and inv['full-provider']['native_scope']['checked_proof_bodies']==1,'full native inventory')
 opacity={}
 for label,crate in [('bool-client','provider'),('full-observer','component'),('private-client','private')]:
  x=inventory(R/label,crate)['ordinary_original_functions'];need(not any('checked_foreign_bool' in n for n in x),'private helper leaked')
  for n,v in x.items():
   if v['mode']=='Exec' or n.endswith(('::value','::word','::permission')):need(not v['body_present'],'opaque imported body leaked '+n)
  opacity[label]=x
 # Every ordinary body, auxiliary and precision query remains in the raw index.
 query_index={}
 for label in ['bool-provider','full-provider','false-client','full-observer','explicit-failed-bv-auxiliary']:
  query_index[label]={p.name:dict(sha256=sha(p),check_valid_count=p.read_text().count('(check-valid')) for p in sorted((R/label/'vir').glob('*final.air'))}
 need(sum(x['check_valid_count'] for x in query_index['full-provider'].values())==52,'exact isolated baseline query inventory')
 # Focused regressions, not the inherited full710/canonical/observer suites.
 regressions={}
 for name in ['certified_foreign_bool','local_checked_foreign','infer_goals','infer_nonzero_derive','constructor_function_values','infer','vir']:
  label='test-'+name
  if preflight and not (R/label/'receipt.json').exists():continue
  d,j,f=receipt(label);need(j['exit']==0,'focused regression '+name)
  lines=[s for s in (d/'stdout').read_text().splitlines() if s.startswith('test result:')];need(lines and all('FAILED' not in s for s in lines),'test result '+name)
  regressions[name]=compact(label)|dict(result_lines=lines)
 # Authentication includes source-matching build inputs and loaded core artifacts.
 engine_files=['source/rust_verify/src/'+x for x in ['checked_foreign.rs','driver.rs','import_export.rs','verifier.rs','commands.rs','checked_foreign_packing.rs']]+['source/vir/src/infer/registry.rs']
 for label in selected:
  _,_,f=receipt(label)
  if label not in ['legacy-consumer-v2']:
   for n,h in tools.items():need(f['tools'].get(n)==h,'tool mismatch '+label+' '+n)
  for p in engine_files:need(f['source'][p]==sha(ROOT/p),'engine source mismatch '+label+' '+p)
 core=TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib';identity=interfaces['bool-provider']['foreign_dependencies'][0]['identity']
 candidates={kind:next(core.glob('libcore-*.'+ext)) for kind,ext in [('rlib','rlib'),('rmeta','rmeta')]}
 used=[]
 for kinds in [('rlib',),('rmeta',),('rlib','rmeta')]:
  entries=[(k,sha(candidates[k])) for k in kinds]
  if hashlib.sha256(json.dumps(entries,separators=(',',':')).encode()).hexdigest()==identity['core_artifact']:used=[dict(kind=k,path=str(candidates[k]),sha256=h) for k,h in entries]
 need(used,'actual loaded core artifact identity')
 source=Path('/tmp/checked-bool-build-klx7abqu/source-full-help.rs').read_text();need(next(iter(json.loads((R/'full-provider/freeze.json').read_text())['fixtures'].values()))==hashlib.sha256(source.encode()).hexdigest(),'unchanged full-help source')
 cold_warm={}
 for kind in ['bool','full']:
  cmp=json.loads((R/('stable-'+kind)/'comparison.json').read_text())
  cold_warm[kind]={k:v for k,v in cmp.items() if k not in ['inventories','Eq']}
  cold_warm[kind]['comparison_sha256']=sha(R/('stable-'+kind)/'comparison.json')
  if 'Eq' in cmp:cold_warm[kind]['Eq']=[{k:h[k] for k in ['function','candidate_id','proofs','metrics','parameters','result_type','contract']} for h in cmp['Eq']]
 result=dict(schema=1,scope='certified actual direct core bool-to-u64 publication/import in isolated accepted-base engine',engine_commit='3a23cdd5',test_commit='905dd062',versions=dict(schema=39,policy=71,cache=74),parent_bundle_receipt_sha256=sha(P/'receipt.json'),tools=tools,loaded_core=used,foreign_interfaces={k:{x:v[x] for x in ['producer','foreign_dependencies','dependencies','summaries']} for k,v in interfaces.items()},legacy_provider=compact('legacy-provider-v2'),trusted_models=interfaces['full-provider']['trusted_models'],cases={label:compact(label) for label in sorted(selected)},inventories=inv,import_opacity=opacity,query_index=query_index,runtime_inspection=json.loads((R/'native-call-identity/inspection.json').read_text()),cold_warm=cold_warm,controllers={p.name:sha(p) for p in sorted(E.glob('*.py'))},regressions=regressions,inputs=dict(legacy_tool_manifest_sha256=sha(R/'legacy-tool-identities.json'),archives_sha256=sha(R/'cache-inputs.json'),index_git_sha256=sha(R/'index-git-authentication.json'),toolchain_sha256=sha(R/'toolchain.json'),build_receipt_sha256=sha(R/'certificate-build-4/receipt.json'),strip_receipt_sha256=sha(R/'certificate-build-4/strip.json')),manual=dict(full_help_inline_proof_blocks=source.count('proof {'),full_help_proof_functions=source.count('proof fn '),full_help_explicit_bv_assertions=source.count('by(bit_vector)'),unchanged_full_help_sha256=hashlib.sha256(source.encode()).hexdigest(),bool_application_proof_glue=0,ordinary_observer_proof_glue=0),all_selected_guards_passed=not preflight,certified_bool_test_publication=True,parent_independent_review_required=True,publication=False,goal_complete=False,native_correspondences=0,HMAC='1/1 UNPROVED; inherited, not rerun',canonical='6/0 inherited, not rerun',open_native_obligations='all44+3/101',arithmetic_routing='Parent reported isolated arithmetic commits a2aa379f + f711aa51; not read/adopted. No fixed query total is a certificate criterion.')
 print(json.dumps(result,indent=2,sort_keys=True))
if __name__=='__main__':main()
