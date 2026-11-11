"""Independent read-only acceptance audit. Explicit requirements survive -O."""
from resume import *
def load(p):return json.loads(Path(p).read_text())
def receipt(label,matching=True):
 d=R/label;j=load(d/'receipt.json');f=load(d/'freeze.json');need(j['guards'],'last guard '+label)
 for n,k in [('freeze.json','freeze_sha256'),('stdout','stdout_sha256'),('stderr','stderr_sha256')]:need(sha(d/n)==j[k],'receipt drift '+label+'/'+n)
 if matching:need(f['tools']==load(R/'proofs-eight-tools.json')['tools'],'matching eight tools '+label)
 else:
  ft=load(R/'proofs-eight-tools.json')['tools']
  need(all(ft[n]==h for n,h in f['tools'].items()),'build tool subset identity '+label)
 need(f['source']==b.manifest(),'exact prepared parent source '+label)
 for p,h in (f.get('fixtures',{})|f.get('dependencies',{})).items():need(sha(p)==h,'immutable input '+p)
 for n,h in f.get('controllers',{}).items():need(sha(R/'controller-snapshots'/(h+'.py'))==h,'captured controller '+n)
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
 return {f['function']:dict(cache=f['metrics']['cache'],candidate_id=f['candidate_id'],dependencies=f['dependencies'],mode=f['mode'],contract=f['contract'],proofs=f['proofs']) for f in fs}


def collect():
 need(sys.flags.dont_write_bytecode,'use -B')
 frozen=load(R/'proofs-eight-tools.json');need(frozen['source']==b.manifest(),'final source guard')
 need(set(frozen['tools'])==set(b.TOOLS),'eight tools')
 for n,h in frozen['tools'].items():need(sha(B/n)==h,'tool guard '+n)
 for p,h in load(R/'toolchain.json')['files'].items():need(sha(p)==h,'compiler/core guard')
 need(frozen['tools']['z3']=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','authorized solver')
 v=receipt('vstd-proofs',matching=False);need(v['exit']==0 and '2250 verified, 0 errors' in (R/'vstd-proofs/stdout').read_text(),'normal full vstd')
 for n in ['vstd.vir','libvstd.rlib']:need(v['outputs'][n]==frozen['tools'][n],'matching vstd output')
 expected={'final-exact-v3-cold':(0,28,0),'final-exact-v3-warm':(0,28,0),'final-boundaries':(0,25,0),'proposal-correct':(0,25,0),'final-stage1-direct':(0,2,0),'final-stage1-off':(1,0,1),'final-stage1-partial':(0,5,0),'final-stage1-alias':(0,2,0),'final-proofless-item':(0,132,0),'final-packet':(0,31,0),'final-bool-prior':(0,2,0),'final-bool-ordinary':(0,1,0),'final-bool-false':(1,0,1),'proof-cargo-bool-prior':(0,2,0),'proof-wrapper-auxiliary':(1,37,1),'partial-only-local':(0,6,0),'independent-alias-arguments':(0,29,0)}
 expected.update({'final-false-'+n:(1,28,1) for n in ['Less','Equal','Greater','swapped']})
 rows={};candidates={};queries={};native={};artifacts={};allqueries={};scopes={};candidatequeries={}
 for label,count in expected.items():
  j=receipt(label);v=j['verification'];need((j['exit'],v.get('verified'),v.get('errors'))==count,'proof count '+label)
  need(not v['encountered-vir-error'] and v['is-verifying-entire-crate'],'ordinary entire crate '+label)
  rows[label]=dict(receipt_sha256=sha(R/label/'receipt.json'),verification=v,failed_functions=j['inference']['failed_functions'])
  if label.startswith('final-exact-v3') or label.startswith('final-false-') or label in ['proof-wrapper-auxiliary','independent-alias-arguments']:
   candidates[label]=healthy_candidates(j,5)
   queries[label]={n:normal(label,n) for n in ['wrapper::checked_nonzero_u64_ord%','wrapper::BorTag::cmp','wrapper::BorTag::partial_cmp']}
  if label.startswith('final-false-'):need(j['inference']['failed_functions']==['wrapper::wrong'],'isolated semantic failure')
  if label=='proof-wrapper-auxiliary':need(j['inference']['failed_functions']==[],'global auxiliary failure')
  if label=='partial-only-local':need('checked_nonzero_u64_ord%' not in (logs(label)/'crate.vir').read_text(),'nohelper PartialOrd-only')
  if label in ['final-proofless-item','final-packet']:need('checked_nonzero_u64_ord%' not in (logs(label)/'crate.vir').read_text(),'ignored ordering compatibility')
  if label.startswith('final-exact-v3'):
   scope=j['inference']['verification_scope'];scopes[label]=scope
   for name in ['wrapper::BorTag::cmp','wrapper::BorTag::partial_cmp']:
    fs=[f for f in scope['native_functions'] if f['function']==name];need(len(fs)==1 and fs[0]['automatically_derived'] and fs[0]['selected'] and fs[0]['body_status']=='checked_exec_body','original native derive')
   need(not any('checked_nonzero' in f['function'] for f in scope['native_functions']),'logical helper not native body')
   native[label]={f['function']:actual_function(logs(label),f['function']) for f in scope['native_functions'] if f['body_status']=='checked_exec_body'}
   cq={}
   for p in logs(label).glob('*-final.air'):
    if p.name=='root-final.air':continue
    air=p.read_text();matches=list(re.finditer(r'^;; (Function-Def|Infer-safe-domain maximality|Infer-output exactness) (.+)$',air,re.M))
    need(len(matches)==3 and air.count('(check-valid')==3,'three fresh candidate queries '+p.name)
    nq={}
    for i,m in enumerate(matches):
     q=air[m.start():matches[i+1].start() if i+1<len(matches) else len(air)]
     need(q.count('(check-valid')==1,'single precision query')
     nq[m[1]]=hashlib.sha256(q.encode()).hexdigest()
    cq[matches[0][2]]=dict(air_sha256=sha(p),queries=nq)
   need(len(cq)==5,'fifteen independent candidate queries');candidatequeries[label]=cq
   names=re.findall(r'^;; Function-Def (.+)$',(logs(label)/'root-final.air').read_text(),re.M);need(len(names)==13,'normal body inventory')
   allqueries[label]={n:query_section((logs(label)/'root-final.air').read_text(),n) for n in names}
   artifacts[label]={p.name:sha(p) for p in logs(label).iterdir() if p.is_file() and (p.suffix in ['.vir','.air'] or p.name.endswith('smt_transcript') or p.name=='crate.impl_names')}
 need(load(R/'final-exact-v3-cold/freeze.json')==load(R/'final-exact-v3-warm/freeze.json'),'identical invocation')
 need(queries['final-exact-v3-cold']==queries['final-exact-v3-warm'],'query/body identities cold warm')
 need(candidatequeries['final-exact-v3-cold']==candidatequeries['final-exact-v3-warm'],'fifteen candidate query identities cold warm')
 need({n:c['candidate_id'] for n,c in candidates['final-exact-v3-cold'].items()}=={n:c['candidate_id'] for n,c in candidates['final-exact-v3-warm'].items()},'candidate IDs cold warm')
 for label,status in [('final-exact-v3-cold','miss'),('final-exact-v3-warm','hit')]:need(all(c['cache']==status for c in candidates[label].values()),'five cache '+status)
 refusals={}
 for filename in ['final-export-invalidation.json','supplement-export-invalidation.json','cargo-prior-invalidation.json','partial-only-invalidation.json','independent-unused-invalidation.json','independent-off-invalidation.json']:
  for label,row in load(R/filename).items():
   j=receipt(label);need(j['exit']!=0,'refusal '+label)
   for p in row.get('paths',row.get('removed',[])):need(not Path(p).exists(),'stale selected output')
   err=(R/label/'stderr').read_text()
   if 'auxiliary' in label:need(j['verification']['errors']==1 and j['inference']['failed_functions']==[],'auxiliary classification')
   if 'erased' in label:need(j['verification']['verified']==2 and j['verification']['errors']==0 and 'E0308' in err,'late erasure classification')
   if label.startswith(('partial-request-','independent-unused-','final-export-','proof-cargo-prior-','independent-off-')):need('local-only' in err,'request local-only gate')
   refusals[label]=dict(receipt_sha256=sha(R/label/'receipt.json'),selection=row,verification=j.get('verification'),local_only='local-only' in err,stderr_sha256=sha(R/label/'stderr'))
 need(len(refusals)==68,'68 selected-output controls')
 for label in ['final-bool-prior','proof-cargo-bool-prior']:
  raw=(R/label/('libwrapper.vir' if 'cargo' in label else 'wrapper.vir')).read_bytes();i=raw.rfind(b'{"schema":');meta=json.loads(raw[i:-8]);need(meta['schema']==39 and meta['policy']=='candidate-bound-evidence-v74','Bool schema/policy')
 proposals={}
 for label in ['proposal-wrong_view','proposal-weak_view','proposal-swapped_view']:
  j=receipt(label);fs=j['inference']['functions'];failed=[f for f in fs if f['proofs']['soundness']['status']=='not_proved' and f['proofs']['exactness']['status']=='not_proved'];need(j['exit']!=0 and failed,'inference rejection')
  need(any(f['proofs']['maximality']['status']=='proved' for f in failed),'maximality independent')
  proposals[label]=dict(receipt_sha256=sha(R/label/'receipt.json'),proofs={f['function']:f['proofs'] for f in fs},classification='inference rejection; dependent helper checks not credited')
 shapes={}
 for n in ['generic','u32','multi','missing-ord','lookalike']:
  label='final-shape-'+n;j=receipt(label);need(j['exit']!=0,'unsupported shape')
  for p in logs(label).glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'unintended concrete helper')
  shapes[n]=dict(receipt_sha256=sha(R/label/'receipt.json'),classification='frontend/profile refusal, not semantic negative')
 tests={}
 for n,count in {'vir':149,'rust_verify-unit':8,'certified_foreign_bool':3,'local_checked_foreign':5,'local_checked_packing':4,'infer_goals':12,'infer_nonzero_derive':3,'constructor_function_values':13,'infer_compare_derive':7,'infer_nonzero_ordering':5}.items():
  label='final-test-'+n;j=receipt(label);need(j['exit']==0 and f'test result: ok. {count} passed; 0 failed; 0 ignored' in (R/label/'stdout').read_text(),'suite '+n);tests[n]=dict(passed=count,receipt_sha256=sha(R/label/'receipt.json'))
 mir=load(R/'loaded-mir-result.json');need(len(mir['seven_loaded_bodies'])==7,'seven original MIRs')
 for p,h in mir['seven_loaded_bodies'].items():need(sha(R/'loaded-mir'/p)==h,'MIR guard')
 for label in ['loaded-mir-build','loaded-mir-run','final-erased-expanded','final-erased-mir']:need(receipt(label)['exit']==0,'MIR diagnostic')
 for d in R.iterdir():
  if (d/'receipt.json').exists() and (d/'freeze.json').exists():receipt(d.name,matching=d.name not in ['build-proofs','vstd-proofs','final-test-build-v4','final-unit-build-v2'])
 return dict(decision='accept_bounded_local_wrapper_source_stage',reviewed_parent='d3b2aa6e63cbc440e09cd5764d99df94e315e959',versions=dict(schema=39,policy=74,cache=77),tools=frozen['tools'],source_manifest_sha256=hashlib.sha256(json.dumps(frozen['source'],sort_keys=True).encode()).hexdigest(),toolchain=load(R/'toolchain.json'),source_review_sha256=sha(H/'SOURCE_REVIEW.md'),historical_authentication_sha256=sha(H/'HISTORICAL_AUTHENTICATION.json'),source_authentication_sha256=sha(H/'SOURCE_AUTHENTICATION.json'),tool_freeze_sha256=sha(H/'TOOL_FREEZE.json'),excluded_setup_sha256=sha(H/'EXCLUDED_SETUP.json'),controllers=b.controllers(),vstd_verified=2250,runs=rows,candidates=candidates,ordinary_body_queries=queries,actual_native_Functions=native,native_scope=scopes,ordinary_query_inventory=allqueries,candidate_query_inventory=candidatequeries,proof_artifacts=artifacts,refusals=refusals,proposals=proposals,unsupported_shapes=shapes,tests=tests,actual_loaded_MIR=mir,findings=[],new_standard_laws=0,native_correspondences=0,ordering_exports=0,ordering_imports=0,canonical='inherited6/0',HMAC='1/1 UNPROVED',open_obligations='44+3/101 OPEN',publication=False,goal_complete=False)
if __name__=='__main__':
 j=collect()
 if '--write' in sys.argv:write(H/'RESULTS.json',j)
 else:
  need(j==load(H/'RESULTS.json'),'result agreement')
  print(json.dumps(dict(audit='passed',optimized=bool(sys.flags.optimize),result_sha256=sha(H/'RESULTS.json'),refusals=68,focused=52,VIR=149,library=8,publication=False,goal_complete=False),sort_keys=True))
