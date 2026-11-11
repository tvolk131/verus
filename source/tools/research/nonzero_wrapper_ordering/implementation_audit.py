"""Read-only final audit, explicit --write for compact result generation. Use -B/-O -B."""
import sys
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'source/tools/research/nonzero_wrapper_ordering'))
from resume import *
BUILD_ROOT=R
R=BUILD_ROOT/'sealed-validation'
need(R.is_dir(),'completed final receipt namespace required')
def load(p):return json.loads(Path(p).read_text())
def receipt(label,matching=True):
 d=R/label;j=load(d/'receipt.json');f=load(d/'freeze.json');need(j['guards'],'last guard '+label)
 for n,k in [('freeze.json','freeze_sha256'),('stdout','stdout_sha256'),('stderr','stderr_sha256')]:need(sha(d/n)==j[k],'receipt drift '+label+'/'+n)
 if matching:need(f['tools']==load(R/'sealed-eight-tools.json')['tools'],'same matching eight tools '+label)
 for p,h in f.get('fixtures',{}).items():need(sha(p)==h,'immutable fixture '+p)
 for p,h in f.get('dependencies',{}).items():need(sha(p)==h,'immutable dependency '+p)
 for n,h in f.get('controllers',{}).items():
  p=R/'controller-snapshots'/(h+'.py');need(p.exists() and sha(p)==h,'captured controller '+n)
 need(not any(s in (d/'stderr').read_text().lower() for s in ['panicked at','internal compiler error','timed out','resource limit']),'excluded failure '+label)
 return j

def logs(label):
 d=R/label;return d/('retained-vir' if (d/'retained-vir').exists() else 'vir')
def actual_function(d,name):
 raw_name=name
 if name.startswith('wrapper::BorTag::'):
  method=name.rsplit('::',1)[1];trait={'cmp':'core::cmp::Ord','partial_cmp':'core::cmp::PartialOrd','eq':'core::cmp::PartialEq','clone':'core::clone::Clone'}[method]
  records=[list(map(str.strip,line.split('###'))) for line in (d/'crate.impl_names').read_text().splitlines()]
  owner='wrapper::BorTag, wrapper::BorTag' if method in ['eq','partial_cmp'] else 'wrapper::BorTag'
  impls=[r[0] for r in records if len(r)>=3 and r[1]==trait and r[2]==owner]
  need(len(impls)==1,'unique actual trait/owner impl');raw_name=impls[0]+'::'+method
 text=(d/'crate.vir').read_text();key=':name (Fun :path '+raw_name+')';need(key in text,'actual Function '+name);i=text.index(key);start=text.rfind('\n(@',0,i)+1;end=text.find('\n(@',i);f=text[start:end if end>=0 else len(text)]
 return dict(actual_Function_name=raw_name,actual_Function_sha256=hashlib.sha256(f.encode()).hexdigest())
def query_section(text,name):
 key=';; Function-Def '+name+'\n';need(key in text,'normal query '+name);i=text.index(key);k=text.find('\n;; Function-',i+1);q=text[i:k if k>=0 else len(text)];need(q.count('(check-valid')==1,'single normal body query '+name);return hashlib.sha256(q.encode()).hexdigest()
def normal(label,name):
 d=logs(label);return actual_function(d,name)|dict(query_sha256=query_section((d/'root-final.air').read_text(),name))
def healthy_candidates(j,count):
 fs=j['inference']['functions'];need(len(fs)==count,'candidate inventory')
 for f in fs:
  need(f['required_checks_passed'] and f['contract']['requires']=='true','full-domain exact candidate')
  for p in ['soundness','maximality','exactness']:need(f['proofs'][p]['required'] and f['proofs'][p]['status']=='proved','fresh mandatory '+p)
 return {f['function']:dict(cache=f['metrics']['cache'],contract=f['contract'],proofs=f['proofs']) for f in fs}

def final_vstd():
 d=BUILD_ROOT/'vstd-sealed';j=load(d/'receipt.json');need(j['exit']==0 and j['guards'],'final normal vstd build')
 for n,k in [('freeze.json','freeze_sha256'),('stdout','stdout_sha256'),('stderr','stderr_sha256')]:need(sha(d/n)==j[k],'matching vstd receipt')
 return j

def collect():
 need(sys.flags.dont_write_bytecode,'use Python -B')
 frozen=load(R/'sealed-eight-tools.json');need(frozen['source']==b.manifest(),'final whole source inventory');need(set(frozen['tools'])==set(b.TOOLS),'all eight tools')
 for n,h in frozen['tools'].items():need(sha(B/n)==h,'executing tool '+n)
 for p,h in load(BUILD_ROOT/'toolchain.json')['files'].items():need(sha(p)==h,'compiler/core '+p)
 need(load(BUILD_ROOT/'solver-authorization-resume.json')['original_sha256']==frozen['tools']['z3']=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','authorized solver copy identity')
 # No broad old bundle reads: only own committed production bytes authenticate trust.
 unchanged=['source/rust_verify/src/'+n+'.rs' for n in ['checked_nonzero_ord','checked_nonzero_ord_tests','checked_foreign','import_export','query','context','fn_call_to_vir','rust_to_vir','external','automatic_derive']]+['source/vstd/std_specs/'+n+'.rs' for n in ['cmp','nonzero','num']]+['source/vir/src/infer/'+n+'.rs' for n in ['registry','newtypes','models']]
 for p in unchanged:need((ROOT/p).read_bytes()==subprocess.check_output(['git','show','913044c4:'+p],cwd=ROOT),'unchanged gate/model '+p)
 production=[p for p in subprocess.check_output(['git','diff','--name-only','913044c4','--','source'],cwd=ROOT).decode().splitlines() if not p.startswith('source/tools/research/nonzero_wrapper_ordering/')]
 need(set(production)==set(['source/builtin_macros/src/infer_derive_ops.rs','source/vir/src/infer/wrapper_binary.rs','source/vir/src/infer/wrapper_compare.rs','source/rust_verify_test/tests/infer_nonzero_derive.rs','source/rust_verify_test/tests/infer_nonzero_ordering.rs','source/vir/src/infer.rs','source/rust_verify/src/verifier.rs']),'bounded changed files')
 v=final_vstd();need(v['exit']==0 and '2250 verified, 0 errors' in (BUILD_ROOT/'vstd-sealed/stdout').read_text(),'normal matching vstd')
 for n in ['libvstd.rlib','vstd.vir']:need(v['outputs'][n]==frozen['tools'][n],'matching vstd output')
 rows={};queries={};scope={};candidates={}
 expected={'final-exact-v3-cold':(0,28,0),'final-exact-v3-warm':(0,28,0),'final-boundaries':(0,25,0),'proposal-correct':(0,25,0),'final-stage1-direct':(0,2,0),'final-stage1-off':(1,0,1),'final-stage1-partial':(0,5,0),'final-stage1-alias':(0,2,0),'final-proofless-item':(0,132,0),'final-packet':(0,31,0),'final-bool-prior':(0,2,0),'final-bool-ordinary':(0,1,0),'final-bool-false':(1,0,1),'proof-cargo-bool-prior':(0,2,0),'proof-wrapper-auxiliary':(1,37,1),'partial-only-local':(0,6,0)}
 expected.update({'final-false-'+n:(1,28,1) for n in ['Less','Equal','Greater','swapped']})
 for label,(code,n,e) in expected.items():
  j=receipt(label);v=j['verification'];need((j['exit'],v.get('verified'),v.get('errors'))==(code,n,e),'proof count '+label);need(not v['encountered-vir-error'] and v['is-verifying-entire-crate'],'ordinary full-scope '+label)
  rows[label]=dict(receipt_sha256=sha(R/label/'receipt.json'),verification=v,failed_functions=j['inference']['failed_functions'])
  if label.startswith('final-exact-v3') or label.startswith('final-false-') or label=='proof-wrapper-auxiliary':
   candidates[label]=healthy_candidates(j,5)
   queries[label]={name:normal(label,name) for name in ['wrapper::checked_nonzero_u64_ord%','wrapper::BorTag::cmp','wrapper::BorTag::partial_cmp']}
  if label.startswith('final-false-'):need(j['inference']['failed_functions']==['wrapper::wrong'],'only intended semantic failure')
  if label=='proof-wrapper-auxiliary':need(j['inference']['failed_functions']==[],'isolated auxiliary failure')
  if label.startswith('final-exact-v3'):
   scope[label]=j['inference']['verification_scope']
   for name in ['wrapper::BorTag::cmp','wrapper::BorTag::partial_cmp']:
    found=[f for f in scope[label]['native_functions'] if f['function']==name];need(len(found)==1 and found[0]['automatically_derived'] and found[0]['selected'] and found[0]['body_status']=='checked_exec_body','actual original native derive checked')
   need(not any('checked_nonzero' in f['function'] for f in scope[label]['native_functions']),'no synthetic native helper')
 for label in ['final-proofless-item','final-packet']:
  need('checked_nonzero_u64_ord%' not in (logs(label)/'crate.vir').read_text(),'original ignored ordering compatibility')
 a=load(R/'final-exact-v3-cold/freeze.json');c=load(R/'final-exact-v3-warm/freeze.json');need(a==c,'identical cold/warm command and environment');need(queries['final-exact-v3-cold']==queries['final-exact-v3-warm'],'same actual helpers/queries')
 for label,cache in [('final-exact-v3-cold','miss'),('final-exact-v3-warm','hit')]:need(all(f['cache']==cache for f in candidates[label].values()),'candidate cache '+cache)
 refusals={}
 all_rows={}
 for name in ['final-export-invalidation.json','supplement-export-invalidation.json','cargo-prior-invalidation.json','partial-only-invalidation.json']:all_rows.update(load(R/name))
 need(len(all_rows)==48,'forty-eight all-selected export/disabled/filter/global controls')
 for label,row in all_rows.items():
  j=receipt(label);need(j['exit']!=0,'export refusal '+label)
  for p in row.get('paths',row.get('removed',[])):need(not Path(p).exists(),'stale selected output '+p)
  if 'auxiliary' in label:need(j['verification'].get('errors')==1 and j['inference']['failed_functions']==[],'global auxiliary classification')
  if 'erased' in label:need(j['verification'].get('verified')==2 and j['verification'].get('errors')==0 and 'E0308' in (R/label/'stderr').read_text(),'late erasure classification')
  refusals[label]=dict(receipt_sha256=sha(R/label/'receipt.json'),selection=row,verification=j.get('verification'),local_only='local-only' in (R/label/'stderr').read_text())
 for label in ['final-bool-prior','proof-cargo-bool-prior']:
  raw=(R/label/('libwrapper.vir' if 'cargo' in label else 'wrapper.vir')).read_bytes();i=raw.rfind(b'{"schema":');cert=load_json= json.loads(raw[i:-8]);need(cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v73','unchanged Bool schema/policy')
 proposals={}
 for label in ['proposal-wrong_view','proposal-weak_view','proposal-swapped_view']:
  j=receipt(label);fs=j['inference']['functions'];failed=[f for f in fs if f['proofs']['soundness']['status']=='not_proved' and f['proofs']['exactness']['status']=='not_proved'];need(j['exit']!=0 and failed,'failed authored proposal obligations')
  need(any(f['proofs']['maximality']['status']=='proved' for f in failed),'independent maximality result')
  proposals[label]=dict(receipt_sha256=sha(R/label/'receipt.json'),proofs={f['function']:f['proofs'] for f in fs},classification='inference rejection; dependent normal helper checks not started; NOT healthy-helper semantic client evidence')
 shapes={}
 for name in ['generic','u32','multi','missing-ord','lookalike']:
  label='final-shape-'+name;j=receipt(label);need(j['exit']!=0,'fail-closed shape')
  for p in (R/label/'vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'unsupported helper')
  shapes[name]=dict(receipt_sha256=sha(R/label/'receipt.json'),classification='frontend/profile refusal, no semantic result')
 tests={}
 for name,count in {'vir':149,'rust_verify-unit':8,'certified_foreign_bool':3,'local_checked_foreign':5,'local_checked_packing':4,'infer_goals':12,'infer_nonzero_derive':3,'constructor_function_values':13,'infer_compare_derive':7,'infer_nonzero_ordering':5}.items():
  label='final-test-'+name;j=receipt(label);need(j['exit']==0,'test suite '+name)
  text=(R/label/'stdout').read_text();need(f'test result: ok. {count} passed; 0 failed; 0 ignored' in text,'test counts '+name);tests[name]=dict(passed=count,receipt_sha256=sha(R/label/'receipt.json'))
 mir=load(R/'loaded-mir-result.json');need(len(mir['seven_loaded_bodies'])==7 and mir['native_correspondences']==0,'complete loaded MIR')
 for p,h in mir['seven_loaded_bodies'].items():need(sha(R/'loaded-mir'/p)==h,'actual MIR '+p)
 for label in ['loaded-mir-build','loaded-mir-run','final-erased-expanded','final-erased-mir']:need(receipt(label)['exit']==0,'MIR/erasure diagnostic')
 need(sha(R/'loaded-mir/erased.mir')==mir['erased_mir_sha256'] and sha(R/'final-erased-expanded/stdout')==mir['erased_expansion_sha256'],'runtime body diagnostics')
 # Complete actual Function/query diagnostic inventories; hashes supply identity only.
 artifact_hashes={};native_bodies={};normal_inventory={}
 for label in ['final-exact-v3-cold','final-exact-v3-warm']:
  native_bodies[label]={f['function']:actual_function(logs(label),f['function']) for f in scope[label]['native_functions'] if f['body_status']=='checked_exec_body'}
  native_bodies[label]['wrapper::checked_nonzero_u64_ord%']=actual_function(logs(label),'wrapper::checked_nonzero_u64_ord%')
  air=(logs(label)/'root-final.air').read_text();names=re.findall(r'^;; Function-Def (.+)$',air,re.M);need(len(names)==13,'exact ordinary body inventory');normal_inventory[label]={name:query_section(air,name) for name in names}
  artifact_hashes[label]={p.name:sha(p) for p in logs(label).iterdir() if p.is_file() and (p.suffix in ['.air','.vir'] or p.name.endswith('smt_transcript') or p.name=='crate.impl_names')}
 counter=load(BUILD_ROOT/'partial-only-counterexample.json');need(counter['unexpected_output'] and not (BUILD_ROOT/'counter-partial-only/wrapper.vir').exists(),'historical unexpected ordering interface removed')
 return dict(historical_local_only_counterexample=counter,status='provisionally_implemented_local_source_stage',parent_validation_required=True,production_files={p:sha(ROOT/p) for p in production},tools=frozen['tools'],source_manifest_sha256=hashlib.sha256(json.dumps(frozen['source'],sort_keys=True).encode()).hexdigest(),vstd_verified=2250,versions=dict(schema=39,policy=73,cache=76),runs=rows,candidates=candidates,ordinary_body_queries=queries,native_source_scope=scope,actual_native_Functions=native_bodies,ordinary_query_inventory=normal_inventory,proof_artifact_hashes=artifact_hashes,export_refusals=refusals,proposal_controls=proposals,unsupported_shapes=shapes,tests=tests,actual_loaded_MIR=mir,new_std_laws=0,ordering_exports=0,ordering_imports=0,native_correspondences=0,publication=False,goal_complete=False)
if __name__=='__main__':
 result=collect();target=H/'IMPLEMENTATION_RESULTS.json'
 if '--write' in sys.argv:write(target,result)
 else:
  seal=load(H/'CURRENT_SEAL.json')
  for path,digest in seal.items():need(sha(H/path)==digest,'sealed continuation file '+path)
  need(load(target)==result,'committed continuation result differs')
  print(json.dumps(dict(audit='passed',optimized=bool(sys.flags.optimize),result_sha256=sha(target),exports=48,focused_tests=52,vir_tests=149,library_tests=8,publication=False,goal_complete=False),sort_keys=True))
