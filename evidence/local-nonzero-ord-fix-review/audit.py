"""Read-only acceptance with explicit exceptions; same behavior under -O."""
from build import *
from authenticate import audit as authenticate
from structural import audit as structural
R=root()
def end(t,s):
 depth=0;quote=False;escape=False
 for i in range(s,len(t)):
  c=t[i]
  if quote:
   if escape:escape=False
   elif c=='\\':escape=True
   elif c=='"':quote=False
  elif c=='"':quote=True
  elif c=='(':depth+=1
  elif c==')':
   depth-=1
   if depth==0:return i+1
 raise RuntimeError('unbalanced log')
def function(text,name):
 for m in re.finditer(r'\(Function\s',text):
  f=text[m.start():end(text,m.start())]
  if re.search(r':name \(Fun :path '+re.escape(name)+r'\)',f):return f
 raise RuntimeError('missing Function '+name)
def helper(label):
 d=R/label;v=d/('retained-vir' if (d/'retained-vir').exists() else 'vir')
 text=(v/'root-final.air').read_text();name='local_order::checked_nonzero_u64_ord%'
 a=text.index(';; Function-Def '+name+'\n');b=text.find('\n;; Function-',a+1);q=text[a:b if b>=0 else len(text)]
 need(q.count('(check-valid')==1,'one normal helper query '+label)
 f=function((v/'crate.vir').read_text(),name)
 return dict(actual_Function_sha256=hashlib.sha256(f.encode()).hexdigest(),normal_query_sha256=hashlib.sha256(q.encode()).hexdigest(),VIR_sha256=sha(v/'crate.vir'),AIR_sha256=sha(v/'root-final.air'))
def targets(f):
 out=[]
 for m in re.finditer(r'\(CallTarget\s',f):
  s=f[m.start():end(f,m.start())]
  if '::get)' in s or 'core::cmp::Ord::cmp)' in s:out.append(re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',s))
 return out

def audit():
 need(sys.flags.dont_write_bytecode,'use -B')
 src=authenticate();need(src==json.loads((HERE/'SOURCE_AUTHENTICATION.json').read_text()),'source identity drift')
 f=json.loads((R/'bundle-final-tools.json').read_text());need(f['before_proofs'] and set(f['tools'])==set(TOOLS),'eight tool pre-proof freeze')
 need(f['source']==manifest(ROOT)==manifest(R/'src'),'source guards')
 for n,h in f['tools'].items():need(sha(bundle()/n)==h,'final tool guard '+n)
 for p,h in json.loads((R/'toolchain.json').read_text())['files'].items():need(sha(p)==h,'toolchain guard')
 runs={}
 for d in sorted(R.iterdir()):
  if not d.is_dir() or not (d/'receipt.json').exists():continue
  j=json.loads((d/'receipt.json').read_text());fr=json.loads((d/'freeze.json').read_text())
  need(j['guards'] and fr['source']==f['source'],'receipt/source guards '+d.name)
  for n in ['stdout','stderr','freeze.json']:need(sha(d/n)==j[n.split('.')[0]+'_sha256'],'raw retention '+d.name+'/'+n)
  need(fr['runner_sha256']==sha(HERE/'build.py'),'runner guard')
  for p,h in fr['fixtures'].items():need(sha(p)==h,'fixture guard')
  dep=d/'dependency-inputs.json'
  if dep.exists():
   for p,h in json.loads(dep.read_text()).items():need(sha(p)==h,'import dependency guard')
  for n,h in fr['tools'].items():need(h==f['tools'][n],'matching runtime '+d.name+'/'+n)
  need(not any(s in (d/'stderr').read_text().lower() for s in ['panicked at','internal compiler error','resource limit','timed out']),'excluded panic/resource evidence '+d.name)
  runs[d.name]=dict(exit=j['exit'],verification=j.get('verification'),receipt_sha256=sha(d/'receipt.json'),freeze_sha256=j['freeze_sha256'],stdout_sha256=j['stdout_sha256'],stderr_sha256=j['stderr_sha256'],argv=fr['argv'])
 expected={'review-three-relations':(0,2,0),'bundle-exact-cold':(0,2,0),'bundle-exact-warm':(0,2,0),'bundle-cold':(0,2,0),'bundle-warm':(0,2,0),'bundle-off':(1,0,1),'bundle-partial':(0,5,0),'bundle-swapped':(1,1,1),'bundle-false':(1,1,1),'bundle-weak':(0,2,0),'bundle-alias':(0,2,0),'review-contexts':(0,6,0),'review-effects':(0,2,0),'review-effects-false':(1,1,1),'review-mirror':(0,1,0),'review-ord-auxiliary':(1,10,1),'review-ignored-derive-export':(0,6,0),'review-wrapper-unproved':(1,2,6),'bundle-bool-provider':(0,2,0),'bundle-bool-ordinary':(0,1,0),'bundle-bool-false':(1,0,1),'review-full-proofless-export':(0,132,0),'review-packet-export':(0,31,0),'dual-distinct-prior':(0,2,0),'dual-same-prior':(0,2,0),'structural-boundaries':(0,2,0)}
 for label,(exit,v,e) in expected.items():
  j=runs[label];a=j['verification'];need((j['exit'],a['verified'],a['errors'])==(exit,v,e) and not a['encountered-vir-error'] and a['is-verifying-entire-crate'],'proof inventory '+label)
 for label in expected:
  argv=runs[label]['argv']
  need('--no-cheating' in argv and ('--compile' in argv or '--VIA-CARGO' in argv),'full application/erasure route '+label)
  need(not any(flag in argv for flag in ['--no-verify','--no-lifetime','--no-trait-conflicts','--no-erasure-check','--verify-function','--verify-module']),'disabled or partial proof '+label)
 helpers={n:helper(n) for n in ['review-three-relations','bundle-exact-cold','bundle-exact-warm','bundle-cold','bundle-warm','bundle-swapped','bundle-false','bundle-weak','bundle-alias','review-contexts','review-effects','review-effects-false','review-ord-auxiliary']}
 need(helpers['bundle-exact-cold']==helpers['bundle-exact-warm'],'cold/warm Function/query identities')
 need((R/'bundle-exact-cold/freeze.json').read_bytes()==(R/'bundle-exact-warm/freeze.json').read_bytes(),'same invocation cold/warm')
 for label in ['bundle-false','bundle-swapped']:
  need(json.loads((R/label/'stdout').read_text())['verus-infer']['failed_functions']==['local_order::total'],'only intended caller fails')
 for label in ['review-ord-auxiliary','bundle-global-auxiliary-explicit','bundle-global-auxiliary-cargo']:
  data=json.loads((R/label/'stdout').read_text())
  need(data['verus-infer']['failed_functions']==[] and data['verification-results']['errors']==1,'auxiliary failure classification '+label)
 no_helper=['review-mirror','review-ignored-derive-export','review-wrapper-unproved','review-full-proofless-export']
 for label in no_helper:
  for p in (R/label/'vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'unintended helper '+label)
 refused={}
 for label,j in runs.items():
  if label.startswith(('bundle-export-','bundle-filter-v2-','bundle-global-')):
   need(j['exit']!=0 and not any((R/label/n).exists() for n in ['local_order.vir','liblocal_order.vir']),'single-route refusal '+label)
   if '-erased-' in label:need(j['verification']['verified']==2 and j['verification']['errors']==0 and 'E0308' in (R/label/'stderr').read_text(),'late erasure failure')
   refused[label]=j['receipt_sha256']
 need(len(refused)==24,'single-route refusal count')
 dual=json.loads((R/'dual-result.json').read_text())
 need(set(dual)=={'dual-distinct','dual-same'},'distinct and equal selected paths')
 for mode,j in dual.items():
  need(j['prior_receipt_sha256']==runs[mode+'-prior']['receipt_sha256'] and j['refusal_receipt_sha256']==runs[mode+'-refusal']['receipt_sha256'],'dual receipt binding')
  need(j['schema']==39 and j['policy']=='candidate-bound-evidence-v73','dual same-runtime epoch')
  need(j['both_removed'] and not Path(j['cargo_path']).exists() and not Path(j['explicit_path']).exists(),'dual removal')
  need(runs[mode+'-refusal']['exit']!=0 and 'local-only' in (R/(mode+'-refusal')/'stderr').read_text(),'dual classification')
 item_freeze=json.loads((R/'review-full-proofless-export/freeze.json').read_text())
 need(list(item_freeze['fixtures'].values())==[src['item_fixture_sha256']],'exact unchanged Item132 fixture')
 for label in ['review-full-proofless-export','review-packet-export']:
  need((R/label/'local_order.vir').is_file(),'ordinary compatibility export retained '+label)
 structural_record=structural();need(structural_record==json.loads((R/'structural-result.json').read_text()),'independent witness controls')
 tests={}
 for n,count in {'vir':149,'rust_verify-unit':8,'certified_foreign_bool':3,'local_checked_foreign':5,'local_checked_packing':4,'infer_goals':12,'infer_nonzero_derive':3,'constructor_function_values':13}.items():
  label='final-test-'+n;need(runs[label]['exit']==0 and f'test result: ok. {count} passed; 0 failed; 0 ignored' in (R/label/'stdout').read_text(),'suite '+n)
  tests[n]=dict(passed=count,receipt_sha256=runs[label]['receipt_sha256'])
 need(runs['vstd']['exit']==0 and '2250 verified, 0 errors' in (R/'vstd/stdout').read_text(),'full normal matching vstd')
 actual=function((R/'bundle-exact-cold/retained-vir/crate.vir').read_text(),'local_order::checked_nonzero_u64_ord%')
 mirror=function((R/'review-mirror/vir/crate.vir').read_text(),'local_order::total')
 ts,ms=targets(actual),targets(mirror);need(len(ts)==len(ms)==3 and ts==ms,'ordinary get/get/u64cmp model targets')
 cert=(R/'bundle-bool-provider/provider.vir').read_bytes();cj=json.loads(cert[cert.rfind(b'{"schema":'):-8]);need(cj['schema']==39 and cj['policy']=='candidate-bound-evidence-v73' and len(cj['foreign_dependencies'])==1,'fresh certified Bool compatibility')
 freeze=dict(source_snapshot_git_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R/'src').decode().strip(),tools=f['tools'],source_inventory_sha256=hashlib.sha256(json.dumps(f['source'],sort_keys=True).encode()).hexdigest(),source_count=len(f['source']),private_freeze_sha256=sha(R/'bundle-final-tools.json'),before_proofs=True)
 return dict(status='ACCEPT_F1_ACCEPT_F2',source=src,freeze=freeze,controllers={p.name:sha(p) for p in sorted(HERE.glob('*.py'))},runs=runs,helpers=helpers,dual=dual,structural=structural_record,single_route_refusals=refused,tests=tests,matching_vstd=2250,model_targets_sha256=hashlib.sha256(json.dumps(ts).encode()).hexdigest(),ordinary_model_targets_identical=True,new_std_contracts=0,ordering_imports=0,wrapper_ordering_proved=False,native=0,publication=False,goal_complete=False)
if __name__=='__main__':
 j=audit()
 if '--write' in sys.argv:write(HERE/'RESULTS.json',j);write(HERE/'TOOL_FREEZE.json',j['freeze'])
 else:need(j==json.loads((HERE/'RESULTS.json').read_text()),'results differ');print(json.dumps(dict(accept=True,status=j['status'],runs=len(j['runs']),results_sha256=sha(HERE/'RESULTS.json'),matching_vstd=2250)))
