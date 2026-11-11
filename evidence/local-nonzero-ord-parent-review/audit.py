"""Read-only exception-gated review audit, valid with Python -B and -O -B."""
from build import *
from source_audit import audit as source_audit
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
 raise RuntimeError('unbalanced diagnostic')
def function(text,name):
 for m in re.finditer(r'\(Function\s',text):
  f=text[m.start():end(text,m.start())]
  if re.search(r':name \(Fun :path '+re.escape(name)+r'\)',f):return f
 raise RuntimeError('missing actual Function '+name)
def section(text,name):
 start=text.index(';; Function-Def '+name+'\n');stop=text.find('\n;; Function-',start+1)
 return text[start:stop if stop>=0 else len(text)]
def helper(label):
 d=R/label;v=d/('retained-vir' if (d/'retained-vir').exists() else 'vir')
 s=section((v/'root-final.air').read_text(),'local_order::checked_nonzero_u64_ord%')
 need(s.count('(check-valid')==1,'one normal helper query '+label)
 f=function((v/'crate.vir').read_text(),'local_order::checked_nonzero_u64_ord%')
 return dict(normal_query_sha256=hashlib.sha256(s.encode()).hexdigest(),actual_full_Function_log_sha256=hashlib.sha256(f.encode()).hexdigest(),full_VIR_sha256=sha(v/'crate.vir'),full_AIR_sha256=sha(v/'root-final.air'))
def targets(f):
 out=[]
 for m in re.finditer(r'\(CallTarget\s',f):
  s=f[m.start():end(f,m.start())]
  if '::get)' in s or 'core::cmp::Ord::cmp)' in s:
   out.append(re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',s))
 return out

def audit():
 need(sys.flags.dont_write_bytecode,'use -B')
 source=source_audit();need(source==json.loads((HERE/'SOURCE_AUTHENTICATION.json').read_text()),'source record drift')
 freeze=json.loads((R/'bundle-final-tools.json').read_text())
 need(freeze['before_proofs'] and set(freeze['tools'])==set(TOOLS),'eight pre-proof tools')
 need(freeze['source']==manifest(ROOT)==manifest(R/'src'),'production source drift')
 for n,h in freeze['tools'].items():need(sha(bundle()/n)==h,'tool drift '+n)
 runs={}
 for d in sorted(R.iterdir()):
  if not d.is_dir() or not (d/'receipt.json').exists() or not (d/'freeze.json').exists():continue
  j=json.loads((d/'receipt.json').read_text());f=json.loads((d/'freeze.json').read_text())
  need(j['guards'],'immutable guards '+d.name)
  for n in ['stdout','stderr','freeze.json']:need(sha(d/n)==j[n.split('.')[0]+'_sha256'],'retained '+d.name+'/'+n)
  need(f['source']==freeze['source'],'source epoch '+d.name)
  need(f['runner_sha256']==sha(HERE/'build.py'),'runner source drift '+d.name)
  for p,h in f['fixtures'].items():need(sha(p)==h,'fixture drift')
  for n,h in f['tools'].items():need(h==freeze['tools'][n],'different runtime '+d.name+'/'+n)
  err=(d/'stderr').read_text().lower()
  need(not any(s in err for s in ['panicked at','internal compiler error','resource limit','timed out']),'excluded panic/resource evidence '+d.name)
  runs[d.name]=dict(source_kind='separate cfg(test)-only control variant; see structural_variant' if d.name.startswith('review-type-dependency-') else 'original prepared source',receipt_sha256=sha(d/'receipt.json'),exit=j['exit'],verification=j.get('verification'),stdout_sha256=j['stdout_sha256'],stderr_sha256=j['stderr_sha256'],freeze_sha256=j['freeze_sha256'])
 def expected(label,exit,v,e):
  j=runs[label];a=j['verification'];need((j['exit'],a.get('verified'),a.get('errors'))==(exit,v,e),'case inventory '+label)
  need(not a['encountered-vir-error'],'frontend counted as semantic '+label)
 expected_cases={'bundle-exact-cold':(0,2,0),'bundle-exact-warm':(0,2,0),'bundle-cold':(0,2,0),'bundle-warm':(0,2,0),'bundle-off':(1,0,1),'bundle-partial':(0,5,0),'bundle-swapped':(1,1,1),'bundle-false':(1,1,1),'bundle-weak':(0,2,0),'bundle-alias':(0,2,0),'bundle-observations':(0,11,0),'review-three-relations':(0,2,0),'review-contexts':(0,6,0),'review-effects':(0,2,0),'review-effects-false':(1,1,1),'review-mirror':(0,1,0),'review-ord-auxiliary':(1,10,1),'review-ignored-derive-export':(0,6,0),'review-wrapper-unproved':(1,2,6),'bundle-bool-provider':(0,2,0),'bundle-bool-ordinary':(0,1,0),'bundle-bool-false':(1,0,1),'review-dual-valid-prior':(0,2,0),'review-full-proofless-export':(0,132,0),'review-packet-export':(0,31,0)}
 for label,w in expected_cases.items():expected(label,*w)
 healthy_labels=[n for n in expected_cases if n not in ['bundle-off','bundle-partial','review-mirror','review-ignored-derive-export','review-wrapper-unproved','bundle-bool-provider','bundle-bool-ordinary','bundle-bool-false','review-dual-valid-prior','review-full-proofless-export','review-packet-export']]
 helpers={n:helper(n) for n in healthy_labels}
 need(helpers['bundle-exact-cold']==helpers['bundle-exact-warm'],'cold/warm actual query/Function mismatch')
 need((R/'bundle-exact-cold/freeze.json').read_bytes()==(R/'bundle-exact-warm/freeze.json').read_bytes(),'same invocation freeze')
 for label in ['bundle-swapped','bundle-false']:
  data=json.loads((R/label/'stdout').read_text());need(data['verus-infer']['failed_functions']==['local_order::total'],'only named caller failure')
 for label in ['review-ord-auxiliary','bundle-global-auxiliary-explicit','bundle-global-auxiliary-cargo']:
  data=json.loads((R/label/'stdout').read_text());need(data['verus-infer']['failed_functions']==[] and data['verification-results']['errors']==1,'auxiliary-only failure')
 for label in ['review-function-value','review-old-policy-import']:
  need(runs[label]['exit']!=0 and runs[label]['verification']['encountered-vir-error'],'frontend/compat refusal')
 need('policy' in (R/'review-old-policy-import/stderr').read_text().lower(),'old-policy diagnostic')
 no_helper=['review-function-value','bundle-unsupported-generic','bundle-unsupported-lookalike','bundle-unsupported-overloaded-deref','review-mirror','review-ignored-derive-export','review-wrapper-unproved','review-full-proofless-export']
 for label in no_helper:
  for p in (R/label/'vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'unexpected concrete helper '+label)
 refused={}
 for label,j in runs.items():
  if label.startswith(('bundle-export-','bundle-filter-v2-','bundle-global-')):
   need(j['exit']!=0,'selected output accepted '+label)
   need(not any((R/label/n).exists() for n in ['local_order.vir','liblocal_order.vir']),'selected stale output '+label)
   if label.startswith('bundle-global-erased-'):
    expected(label,1,2,0);need('E0308' in (R/label/'stderr').read_text(),'late erased error')
   if label.startswith('bundle-global-auxiliary-'):expected(label,1,10,1)
   refused[label]=j
 need(len(refused)==24,'complete single-path export/global matrix')
 dual=json.loads((R/'dual-valid-result.json').read_text())
 need(dual['cargo_exists'] and not dual['explicit_exists'] and dual['before_sha256']==dual['after_sha256'],'dual-path finding did not reproduce')
 need(sha(R/'dual-valid-retained-prior.vir')==dual['before_sha256'],'valid prior retention')
 need(not (R/'review-dual-valid-prior/liblocal_order.vir').exists(),'selected control path cleanup')
 f=function((R/'bundle-exact-cold/retained-vir/crate.vir').read_text(),'local_order::checked_nonzero_u64_ord%')
 mirror=function((R/'review-mirror/vir/crate.vir').read_text(),'local_order::total')
 ts,ms=targets(f),targets(mirror)
 need(len(ts)==len(ms)==3 and ts==ms,'get/get/u64cmp target mismatch against ordinary mirror')
 model_text=(R/'bundle-exact-cold/retained-vir/crate.vir').read_text()
 model_names=re.findall(r'\(Function\s+:name \(Fun :path ([^)]*)\)',model_text)
 model_names=[n for n in model_names if n in ['core::cmp::Ord::cmp','vstd::std_specs::nonzero::nonzero_spec_get','vstd::std_specs::nonzero::impl&%13::view','vstd::view::View::view','core::num::nonzero::impl&%16::get','core::cmp::impls::impl&%67::cmp'] or n.startswith('vstd::std_specs::num::u64_specs_tmp::impl&%3::')]
 need(len(model_names)==8,'complete observed get/View/u64 model closure')
 models=dict(observed_model_Functions={n:hashlib.sha256(function(model_text,n).encode()).hexdigest() for n in model_names},helper_model_targets_sha256=hashlib.sha256(json.dumps(ts).encode()).hexdigest(),ordinary_mirror_targets_identical=True,mirror_is_diagnostic_only=True)
 tests={}
 for n,count in {'vir':149,'rust_verify-unit':7,'certified_foreign_bool':3,'local_checked_foreign':5,'local_checked_packing':4,'infer_goals':12,'infer_nonzero_derive':3,'constructor_function_values':13}.items():
  label='final-test-'+n;need(runs[label]['exit']==0,'test failed')
  need(f'test result: ok. {count} passed; 0 failed; 0 ignored' in (R/label/'stdout').read_text(),'test count '+n)
  tests[n]=dict(passed=count,receipt_sha256=runs[label]['receipt_sha256'])
 need('2250 verified, 0 errors' in (R/'vstd/stdout').read_text() and runs['vstd']['exit']==0,'vstd')
 cert=(R/'bundle-bool-provider/provider.vir').read_bytes();start=cert.rfind(b'{"schema":');j=json.loads(cert[start:-8]);need(cert.endswith(b'VRINFER1') and j['schema']==39 and j['policy']=='candidate-bound-evidence-v73' and len(j['foreign_dependencies'])==1,'fresh bool metadata')
 erased=json.loads((R/'erased-observation.json').read_text())
 need(erased['std_cmp_kept'] and not erased['synthetic_helper_present'] and erased['erased_source_sha256']==sha(R/'review-erased-expansion/stdout'),'erased source observation')
 symbols=subprocess.check_output(['/usr/bin/nm','-C','--defined-only',str(R/'bundle-same-invocation/liblocal_order.rlib')],stderr=subprocess.DEVNULL).decode()
 need('local_order::total' in symbols and 'checked_nonzero' not in symbols,'synthetic native helper')
 scope={}
 for label in healthy_labels:
  s=json.loads((R/label/'stdout').read_text())['verus-infer']['verification_scope'];scope[label]=s
  need(not any('checked_nonzero' in n['function'] for n in s['native_functions']),'synthetic helper counted native')
 structural=json.loads((R/'type-dependency-control-result.json').read_text())
 control_inputs=json.loads((R/'type-dependency-control-inputs.json').read_text())
 need(structural['accepted_cleared_param_embedded_impl_paths'] and structural['production_tools_unchanged'] and structural['only_cfg_test_changed'],'actual typed dependency coverage finding')
 need(control_inputs['base_source']==freeze['source'] and manifest(R/'type-dependency-control-src')==control_inputs['variant_source'],'structural variant identity')
 need({n for n in freeze['source'] if freeze['source'][n]!=control_inputs['variant_source'][n]}=={'source/rust_verify/src/checked_nonzero_ord_tests.rs'},'cfg(test)-only variant')
 need('REVIEW_EMBEDDED_TYPE_DEPENDENCY_CONTROL accepted=true' in (R/'review-type-dependency-actual-function/stdout').read_text() and runs['review-type-dependency-actual-function']['exit']==0,'structural result')
 return dict(structural_variant=structural,structural_variant_inputs_sha256=sha(R/'type-dependency-control-inputs.json'),controllers={p.name:sha(p) for p in sorted(HERE.glob('*.py'))},status='REQUEST_CHANGES_F1_DUAL_OUTPUT_F2_TYPE_DEPENDENCIES',review_base=source['source_base'],preregistration_commit='56210c34',private_root=str(R),source_inventory_sha256=hashlib.sha256(json.dumps(freeze['source'],sort_keys=True).encode()).hexdigest(),source_count=len(freeze['source']),tools=freeze['tools'],freeze_sha256=sha(R/'bundle-final-tools.json'),versions=source['versions'],runs=runs,helpers=helpers,models=models,erased_source_observation=erased,native_source_scope=scope,original_native_symbols_sha256=hashlib.sha256(symbols.encode()).hexdigest(),single_path_refusals=refused,dual_selected_output_finding=dual,tests=tests,matching_vstd=2250,compiler_core=source['compiler_core'],inherited_models=['opaque genuine NonZero View','NonZero get/spec_get','actual u64 Ord/obeys_cmp_spec/numeric cmp_spec'],new_std_contracts=0,ordering_certificates=0,ordering_imports=0,wrapper_ordering_proved=False,native_correspondences=0,publication=False,goal_complete=False)
if __name__=='__main__':
 result=audit()
 if '--write' in sys.argv:write(HERE/'RESULTS.json',result)
 else:need(result==json.loads((HERE/'RESULTS.json').read_text()),'results differ');print(json.dumps(dict(success=True,results_sha256=sha(HERE/'RESULTS.json'),status=result['status'],runs=len(result['runs']),tests=result['tests'],matching_vstd=2250,publication=False,goal_complete=False),sort_keys=True))
