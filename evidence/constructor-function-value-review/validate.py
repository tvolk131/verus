"""Validate completed review evidence with explicit -O-safe gates. No verifier execution."""
import json,pathlib,re,sys
import epoch as e
HERE=pathlib.Path(__file__).resolve().parent
P=e.PRIVATE
EXCLUDED={'build','build-2','test-constructor_function_values'}
def data(label):
 p=P/'epochs'/label/'stdout'
 return json.loads(p.read_text())
def proof(label,verified,errors=0):
 d=data(label);v=d['verification-results']
 e.require(v.get('verified')==verified and v.get('errors')==errors and v.get('encountered-vir-error') is False and v.get('is-verifying-entire-crate') is True and v['success']==(errors==0),label+' exact proof result')
 return d

def main():
 receipts={}
 for p in sorted((P/'epochs').glob('*/receipt.json')):
  r=json.loads(p.read_text());label=p.parent.name
  e.require(e.digest(p.parent/'stdout')==r['stdout_sha256'] and e.digest(p.parent/'stderr')==r['stderr_sha256'],label+' logs')
  b=json.loads((p.parent/'before.json').read_text())
  e.require(all(b[k]==r[k] for k in ('argv','cwd','environment','frozen')),label+' before binding')
  if label not in EXCLUDED:
   e.require(r['final_guards'] is True,label+' epoch guard')
   for n,h in r['frozen'].items():e.require(e.digest(n)==h,label+' final input '+n)
   for n,h in r['products'].items():e.require(e.digest(n)==h,label+' final product '+n)
  out=(p.parent/'stdout').read_text();err=(p.parent/'stderr').read_text()
  item={k:r[k] for k in ('argv','cwd','exit','seconds','final_guards','stdout_sha256','stderr_sha256','products')}
  item.update(receipt_sha256=e.digest(p),before_sha256=e.digest(p.parent/'before.json'),frozen_manifest_sha256=__import__('hashlib').sha256(json.dumps(r['frozen'],sort_keys=True).encode()).hexdigest(),frozen_count=len(r['frozen']),accepted_for_review=label not in EXCLUDED)
  item['environment']={k:v for k,v in r['environment'].items() if k.startswith(('CARGO','RUST','VERUS','VARGO','GIT_')) or k in ('PATH','LD_LIBRARY_PATH','TMPDIR')}
  if label.startswith(('original-','delivered-','probe-','interface-','stale-')):
   e.require('panicked at' not in err and 'internal compiler error' not in err,label+' verifier health')
   d=json.loads(out);item['verification']=d['verification-results'];infer=d['verus-infer']
   crate=next((pathlib.Path(a).stem for a in r['argv'] if a.endswith('.rs')),'')
   if '--crate-name' in r['argv']:crate=r['argv'][r['argv'].index('--crate-name')+1]
   item['local_function_details']=sorted(n for n in d['func-details'] if n.startswith(crate+'::'))
   item['inference']={k:infer[k] for k in ('status','generated_functions','functions','interface_written')}
   item['assistance']={k:v for k,v in infer.items() if k.endswith('_assistance')}
   item['source_sha256']={a:r['frozen'][str(pathlib.Path(a).resolve())] for a in r['argv'] if a.endswith('.rs')}
   if item['verification'].get('encountered-vir-error'):item['classification']='frontend_unsupported'
   elif 'error[E0' in err:item['classification']='rust_rejection'
   elif r['exit']==0:item['classification']='proved'
   elif label=='probe-opaque_callback':item['classification']='unproved_valid_contract_outside_delivery_claim'
   else:item['classification']='proof_rejection'
  if label.startswith('test-') and label!='test-build-2':
   matches=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed;',out)
   e.require(len(matches)==1,label+' test summary');item['tests']={'passed':int(matches[0][0]),'failed':int(matches[0][1])}
  receipts[label]=item
 e.require(len(receipts)==60,'complete epoch inventory')
 for label in ('original-off','original-on'):
  d=proof(label,2);infer=d['verus-infer']
  e.require(receipts[label]['local_function_details']==['bortag::BorTag::ctor%','bortag::BorTag::new'],label+' local body inventory')
  e.require(infer['generated_functions']==0 and infer['functions']==[] and all(v==[] for k,v in infer.items() if k.endswith('_assistance')),label+' no inference helpers')
 expected={'original_nonzero':2,'generic_move_lifetime_variant_and_order':8,'higher_order_checked':3,'payload_get_roundtrip':4,'self_public_bound_const_generic_and_zero_arity':9}
 for n,count in expected.items():proof('delivered-'+n,count)
 proof('delivered-higher_order_requires',0,1);proof('delivered-wrong_variant_field_order_and_false_post',3,4)
 e.require(receipts['test-constructor_function_values-2']['tests']=={'passed':13,'failed':0},'focused 13')
 for n in ('fn_pointer_rejected','invariant_rejected','mixed_mode_rejected'):
  e.require(receipts['delivered-'+n]['classification']=='frontend_unsupported','unsupported '+n)
 for n in ('lifetime_checked','move_checked','visibility_checked'):
  e.require(receipts['delivered-'+n]['classification']=='rust_rejection','Rust '+n)
 for n in ('direct_mut_ref','direct_unsized','local_import','mutable_callback','spec_constructor_use','unused_associated_bound','unused_hrtb','unused_lifetime_bounds','unused_mut_ref','unused_shared_ref','unused_tracked_field','unused_unsized'):proof('probe-'+n,2)
 proof('probe-unused_spec_field',1);proof('probe-fn_def_in_field',3)
 proof('probe-mutable_callback_false',1,1);proof('probe-opaque_callback',2,1);proof('probe-opaque_ordinary_control',4)
 for n in ('stale-zero-arity','stale-zero-arity-self'):proof(n,2)
 for n in ('test-fndef_types-unit_struct_as_fn_not_supported','test-fndef_types-unit_struct_as_fn_not_supported_self_ctor'):
  e.require(receipts[n]['tests']=={'passed':0,'failed':1} and receipts[n]['exit']==101,'stale expectation '+n)
 for n,r in receipts.items():
  if n.startswith('test-') and n not in EXCLUDED and 'tests' in r and 'unit_struct_as_fn_not_supported' not in n:e.require(r['tests']['failed']==0,n+' compatibility')
 for n,count in [('ordinary_base',1),('ordinary_user',1),('infer_base',6),('infer_middle',3),('infer_app',4),('ctor_provider',1),('ctor_user_direct',1)]:proof('interface-'+n,count)
 e.require(receipts['interface-ctor_user_value']['classification']=='frontend_unsupported','imported constructor value conservative restriction')
 e.require(not (P/'exports/failed_export.vir').exists() and not (P/'exports/libfailed_export.rlib').exists(),'failed proof publication guard')
 for name,hash in json.loads((P/'source-copy.json').read_text()).items():e.require(e.digest(e.WORK/name)==hash==e.digest(e.ROOT/name),'final source copy '+name)
 e.require('2250 verified, 0 errors' in (P/'epochs/vstd/stdout').read_text(),'normal vstd')
 import run as driver
 result=dict(review_base='868b6a19',verdict='request_changes',blocking_findings=['P2: update two obsolete zero-arity FnDef rejection tests at fndef_types.rs:1416,1427'],eager_regression_observed=False,scope='bounded constructor-function-item frontend only',compiler_commit='58cf8f9c35b355199727627d07ae34a4e2846d67',private_root=str(P),tools={str(x):e.digest(x) for x in driver.tools()},controllers={str(x):e.digest(x) for x in [HERE/'epoch.py',HERE/'run.py',HERE/'validate.py',P/'batch.py',P/'finish.py']},source_copy_manifest_sha256=e.digest(P/'source-copy.json'),epochs=receipts,final_guards=True,publication=False,goal_complete=False)
 result['environment_profiles']={};result['assistance_profiles']={}
 for receipt in receipts.values():
  for field,profiles in [('environment',result['environment_profiles']),('assistance',result['assistance_profiles'])]:
   if field in receipt:
    value=receipt.pop(field);key=__import__('hashlib').sha256(json.dumps(value,sort_keys=True).encode()).hexdigest()
    profiles[key]=value;receipt[field+'_profile']=key
 result['compiler_and_solver_tools']={str(x):e.digest(x) for x in [e.TC/'bin/rustc',e.TC/'bin/cargo',*e.TC.glob('lib/librustc_driver*'),e.BUNDLE/'z3',pathlib.Path('/home/dev/.cargo/bin/rustup')]}
 e.dump(HERE/'evidence.json',result)
 print(json.dumps({'epochs':len(receipts),'verdict':result['verdict'],'final_guards':True,'focused':'13/13','original':'2/0 off/on','normal_vstd':'2250/0'}))
if __name__=='__main__':main()
