"""Read-only by default. Exception gates also run under python -O -B.
Raw source, VIR, MIR, AIR, symbols and logs stay private; output is compact hashes.
"""
from build import *
R=root();B=R/'bundle-final';P='bundle-final-'
def receipt(label):
 d=R/label;j=json.loads((d/'receipt.json').read_text())
 need(j['guards'],'failed immutable run guard '+label)
 for n in ['stdout','stderr','freeze.json']:need(sha(d/n)==j[n.replace('.json','')+'_sha256'],'retained run drift '+label+'/'+n)
 return j
def section(text,name):
 start=text.index(';; Function-Def '+name+'\n');end=text.find('\n;; Function-',start+1)
 return text[start:end if end>=0 else len(text)]
def helper(label):
 d=R/label;v=d/('retained-vir' if (d/'retained-vir').exists() else 'vir')
 air=(v/'root-final.air').read_text();s=section(air,'local_order::checked_nonzero_u64_ord%')
 need(s.count('(check-valid')==1,'actual normal helper query missing '+label)
 text=(v/'crate.vir').read_text();pos=text.index(':name (Fun :path local_order::checked_nonzero_u64_ord%)')
 a=text.rfind('\n(@',0,pos)+1;b=text.find('\n(@',pos)
 full=text[a:b if b>=0 else len(text)]
 return dict(normal_query_sha256=hashlib.sha256(s.encode()).hexdigest(),actual_Function_log_sha256=hashlib.sha256(full.encode()).hexdigest(),full_VIR_sha256=sha(v/'crate.vir'),full_AIR_sha256=sha(v/'root-final.air'))
def audit():
 need(sys.flags.dont_write_bytecode,'use -B')
 f=json.loads((R/'bundle-final-final-tools.json').read_text())
 need(f['before_proofs'] and set(f['tools'])==set(TOOLS),'complete before-proof freeze')
 need(manifest(ROOT)==f['source']==manifest(R/'src'),'final own-source drift')
 for n,h in f['tools'].items():need(sha(B/n)==h,'final tool drift '+n)
 versions=dict(schema=39,policy=72,cache=75)
 sealed=json.loads(Path('/tmp/nonzero-ordering-mir-x1sbsnfo/inspect/freeze.json').read_text())['inputs']
 compiler_core={n:h for n,h in sealed.items() if 'libcore-' in n or n.endswith('/bin/rustc') or 'librustc_driver' in n}
 need(len(compiler_core)==3,'exact compiler/core inputs')
 for n,h in compiler_core.items():need(sha(n)==h,'sealed compiler/core changed '+n)
 for name in ['source/vstd/std_specs/nonzero.rs','source/vstd/std_specs/num.rs','source/vstd/std_specs/cmp.rs','source/rust_verify/src/checked_foreign.rs','source/rust_verify/src/import_export.rs']:
  need((ROOT/name).read_bytes()==subprocess.check_output(['git','show','fc0479a6:'+name],cwd=ROOT),'closed baseline changed '+name)
 expected={'cold':(0,2,0),'warm':(0,2,0),'exact-cold':(0,2,0),'exact-warm':(0,2,0),'off':(1,0,1),'partial':(0,5,0),'swapped':(1,1,1),'false':(1,1,1),'weak':(0,2,0),'alias':(0,2,0),'observations':(0,11,0),'three-relations':(0,2,0),'bool-provider':(0,2,0),'bool-ordinary':(0,1,0),'bool-false':(1,0,1)}
 runs={};helpers={};native={}
 for suffix,(code,n,errors) in expected.items():
  label=P+suffix;j=receipt(label);v=j['verification']
  need((j['exit'],v.get('verified'),v.get('errors'))==(code,n,errors),'ordinary proof inventory '+label)
  need(not v['encountered-vir-error'],'frontend failure counted as semantic '+label)
  data=json.loads((R/label/'stdout').read_text())
  runs[suffix]=dict(receipt_sha256=sha(R/label/'receipt.json'),verification=v,failed_functions=data['verus-infer']['failed_functions'])
  if suffix in ['swapped','false']:need(data['verus-infer']['failed_functions']==['local_order::total'],'wrong named semantic failure')
  if suffix not in ['off','partial','bool-provider','bool-ordinary','bool-false']:
   helpers[suffix]=helper(label)
   native[suffix]=data['verus-infer']['verification_scope']
   need(not any('checked_nonzero' in n['function'] for n in native[suffix]['native_functions']),'synthetic executable helper')
 a=json.loads((R/(P+'exact-cold')/'freeze.json').read_text());b=json.loads((R/(P+'exact-warm')/'freeze.json').read_text());need(a==b,'not same invocation/source/tools/environment')
 need(helpers['exact-cold']==helpers['exact-warm'],'actual Function/query changed on warm run')
 # Symbol inventory of original compiled source, not a native refinement claim.
 symbols=subprocess.check_output(['/usr/bin/nm','-C','--defined-only',str(R/(P+'same-invocation')/'liblocal_order.rlib')],stderr=subprocess.DEVNULL).decode()
 need('local_order::total' in symbols and 'checked_nonzero' not in symbols,'native executable helper appeared')
 refuses={}
 for d in sorted(R.iterdir()):
  if d.name != P+'export---no-erasure-check-explicit' and d.is_dir() and (d.name.startswith(P+'export-') or d.name.startswith(P+'filter-v2-') or d.name.startswith(P+'global-')):
   j=receipt(d.name);need(j['exit']!=0,'selected interface accepted '+d.name)
   need(not (d/'local_order.vir').exists() and not (d/'liblocal_order.vir').exists(),'selected stale output survived '+d.name)
   if d.name.startswith(P+'global-'):
    data=json.loads((d/'stdout').read_text());v=j['verification']
    need(not v['encountered-vir-error'],'global gate frontend misclassified')
    if 'auxiliary' in d.name:
     need(data['verus-infer']['failed_functions']==[] and v['errors']==1,'auxiliary failed_functions classification')
     query=section((d/'vir/root-final.air').read_text(),'local_order::checked_foreign_bool_to_u64%')
     need('(check-valid' in query,'auxiliary missing normal bool helper query')
    else:need(v['verified']==2 and v['errors']==0 and 'E0308' in (d/'stderr').read_text(),'late Rust classification')
   refuses[d.name]=dict(receipt_sha256=sha(d/'receipt.json'),stderr_sha256=j['stderr_sha256'],verification=j.get('verification'))
 need(len(refuses)==24,'export/filter/global matrix incomplete')
 unsupported={}
 for suffix in ['function-value','unsupported-generic','unsupported-lookalike','unsupported-overloaded-deref']:
  label=P+suffix;j=receipt(label);d=R/label
  need('checked_nonzero_u64_ord%' not in (d/'stdout').read_text(),'unsupported concrete helper acquired')
  for p in (d/'vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'unsupported helper in VIR')
  unsupported[suffix]=dict(receipt_sha256=sha(d/'receipt.json'),exit=j['exit'],verification=j.get('verification'),classification='frontend unsupported' if j.get('verification',{}).get('encountered-vir-error') else 'ordinary property only; no concrete helper')
 tests={}
 for suite,count in {'vir':149,'rust_verify-unit':7,'certified_foreign_bool':3,'local_checked_foreign':5,'local_checked_packing':4,'infer_goals':12,'infer_nonzero_derive':3,'constructor_function_values':13}.items():
  label='final-test-'+suite;j=receipt(label);need(j['exit']==0,'failed suite '+suite)
  out=(R/label/'stdout').read_text();need(f'test result: ok. {count} passed; 0 failed; 0 ignored' in out,'wrong suite count '+suite)
  tests[suite]=dict(passed=count,receipt_sha256=sha(R/label/'receipt.json'))
 vstd=receipt('vstd-final');need(vstd['exit']==0 and '2250 verified, 0 errors' in (R/'vstd-final/stdout').read_text(),'matching vstd')
 # Existing bool certificates keep exactly their existing single kind/schema.
 raw=(R/(P+'bool-provider')/'provider.vir').read_bytes();i=raw.rfind(b'{"schema":');need(i>=0 and raw.endswith(b'VRINFER1'),'bool metadata missing')
 cert=json.loads(raw[i:-8]);need(cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v72' and len(cert['foreign_dependencies'])==1,'bool certificate changed')
 excluded={}
 for label in ['build','build-02','unit-build-01','direct-epoch01','bundle-epoch02-direct','bundle-final-filter-function-explicit','bundle-final-export---no-erasure-check-explicit']:
  j=receipt(label);need(j['exit']!=0,'excluded failure drift '+label);excluded[label]=dict(receipt_sha256=sha(R/label/'receipt.json'),exit=j['exit'])
 return dict(scope='Stage1 checked local genuine NonZero<u64> carrier only; independent parent review required',versions=versions,compiler_core=compiler_core,tools=f['tools'],source_inventory_sha256=hashlib.sha256(json.dumps(f['source'],sort_keys=True).encode()).hexdigest(),final_freeze_sha256=sha(R/'bundle-final-final-tools.json'),engine_sources={n:sha(ROOT/n) for n in subprocess.check_output(['git','diff','--name-only','fc0479a6','--','source/rust_verify/src'],cwd=ROOT).decode().splitlines()},runs=runs,ordinary_helpers=helpers,native_source_inventory=native,original_native_symbols_sha256=hashlib.sha256(symbols.encode()).hexdigest(),refusals=refuses,unsupported=unsupported,tests=tests,matching_vstd=2250,vstd_receipt_sha256=sha(R/'vstd-final/receipt.json'),excluded_failed_runs=excluded,ordering_exports=0,ordering_imports=0,new_StdModel_admissions=0,wrapper_ordering_proved=False,native_correspondences=0,publication=False,goal_complete=False)
if __name__=='__main__':
 result=audit()
 if '--write' in sys.argv:write(HERE/'RESULTS.json',result)
 else:
  need(json.loads((HERE/'RESULTS.json').read_text())==result,'committed results differ')
  print(json.dumps(dict(success=True,results_sha256=sha(HERE/'RESULTS.json'),ordinary_cases=len(result['runs']),export_refusals=len(result['refusals']),tests=result['tests'],matching_vstd=2250,publication=False,goal_complete=False),sort_keys=True))
