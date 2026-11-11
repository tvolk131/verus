"""Final read-only receipt audit and compact owned report data."""
import hashlib,json,re,subprocess,sys
from pathlib import Path
import review as t
sys.path.insert(0,str(t.ROOT/'source/tools/research/item_tag_constructor_composition'))
from source import fidelity,generate
R=t.R;E=t.E
from functools import lru_cache
uncached_sha=t.sha
t.sha=lru_cache(maxsize=None)(t.sha)
CURRENT_SOURCE=t.manifest()

def load(p):return json.loads(Path(p).read_text())
def checked(d):
 d=Path(d);r=load(d/'receipt.json');f=load(d/'freeze.json');t.need(r['guards'],'guard '+str(d))
 t.need(f['runner_sha256'] in [t.sha(R/'review.executed.py'),t.sha(t.E/'review.py')],'executed runner identity')
 for n in ['freeze','stdout','stderr']:t.need(t.sha(d/(n+'.json' if n=='freeze' else n))==r[n+'_sha256'],'receipt '+str(d))
 for p,h in f['fixtures'].items():t.need(t.sha(p)==h,'fixture '+p)
 t.need(f['source']==CURRENT_SOURCE,'production source drift')
 for n,h in f['tools'].items():t.need(t.sha(R/'bundle'/n)==h,'shipping tool drift')
 return dict(path=str(d.relative_to(R)),receipt_sha256=t.sha(d/'receipt.json'),**r)

def main():
 # All completed receipts, including failed setup and immutable cold/warm copies.
 ds=sorted(R.glob('*/receipt.json'))+sorted(R.glob('stable-*/*/receipt.json'))+sorted(R.glob('false/*/receipt.json'))+sorted(R.glob('off/*/receipt.json'))+sorted(R.glob('full-helpers/*/receipt.json'))
 receipts=[checked(p.parent) for p in ds]
 tools={n:t.sha(R/'bundle'/n) for n in t.TOOLS}
 t.need('2250 verified, 0 errors' in (R/'vstd/stdout').read_text(),'vstd')
 t.need(load(R/'vstd/receipt.json')['outputs']=={n:tools[n] for n in ['vstd.vir','libvstd.rlib']},'matching vstd')
 expected={'stable-actual/cold':(2,0),'stable-actual/warm':(2,0),'false/single':(1,1),'off/single':(0,1),'stable-full-help/cold':(52,0),'stable-full-help/warm':(52,0),'full-helpers/single':(36,0)}
 for p,count in expected.items():
  r=load(R/p/'receipt.json');v=r['verification'];t.need((v['verified'],v['errors'])==count and v['is-verifying-entire-crate'],'counts '+p)
 t.need(load(R/'false/single/receipt.json')['failed_functions']==['component::all_bool'],'healthy helper false caller')
 for kind in ['actual','full-help']:
  b=R/('stable-'+kind);a=load(b/'cold/freeze.json');z=load(b/'warm/freeze.json');t.need(a==z,'cold/warm frozen input')
 inv={k:load(R/k/'inventory.json') for k in ['stable-actual','stable-full-help','full-helpers']}
 full=inv['stable-full-help'];scope=full['native_scope'];t.need(scope['checked_exec_bodies']==16 and scope['checked_proof_bodies']==1 and scope['complete_selected_scope_verified'],'HIR scope')
 for n in ['component::checked_foreign_bool_to_u64%','component::BorTag::ctor%','component::Item::ctor%']:t.need(full['ordinary_original_functions'][n]['body_present'],'ordinary helper body')
 comparison=load(R/'stable-full-help/comparison.json');t.need(comparison['cold']['candidate_id']==comparison['warm']['candidate_id'],'Eq ID')
 eq={}
 for temp in ['cold','warm']:
  p=R/'stable-full-help'/temp/'vir/impl&%5__eq-final.air';s=p.read_text();t.need(s.count('(check-valid')==3,'fresh Eq queries')
  t.need('packing_low' not in s,'packing reached precision queries')
  eq[temp]=dict(air_sha256=t.sha(p),fresh_check_valid_count=3,proofs=comparison[temp]['proofs'])
  for k in ['soundness','maximality','exactness']:t.need(comparison[temp]['proofs'][k]['required'] and comparison[temp]['proofs'][k]['status']=='proved','mandatory proof '+k)
 t.need(comparison['cold']['contract']['requires']=='true' and comparison['warm']['contract']['requires']=='true','Eq full domain')
 for n in ['ordinary','infer-aware']:
  d=R/('cargo-capfree-'+n);rr=load(d/'receipt.json');t.need(rr['exit']==0 and rr['verification']['verified']==1 and rr['verification']['errors']==0 and (d/'libcomponent.vir').is_file(),'healthy capfree cargo '+n)
 exports=load(R/'exports.json');t.need(len(exports['controls'])==24,'24 export controls')
 for r in exports['controls']:
  p=R/r['label']/('libcomponent.vir' if r['label'].startswith('cargo-') else 'selected.vir');t.need(not p.exists(),'stale output')
 for n in ['no-verify','no-lifetime','no-erasure','no-traits','filter-function','filter-module','filter-only-module','filter-root']:t.need('complete source/body' in (R/('local-'+n)/'stderr').read_text(),'incomplete refused')
 for shape in ['extra-filename','explicit-emit','single-output']:
  for stage in ['complete','parse','type']:
   d=R/('cargo-path-'+shape+'-'+stage);p=d/('libcomponent-review.vir' if shape=='extra-filename' else 'chosen.vir');t.need(not p.exists() and load(d/'receipt.json')['exit']!=0,'cargo path stale')
 faults=load(R/'fault-controls.json');t.need(len(faults)==6,'all independent source controls')
 for f in faults:
  n=f['control'];d=R/('fault-'+n+'-complete');t.need(t.sha(d/'fixture-source.json')==f['source_sha256'],'control manifest');t.need(t.sha(d/'mutation.patch')==f['patch_sha256'],'patch')
  for name,h in f['actual_tools'].items():t.need(t.sha(d/'bundle'/name)==h,'actual control tools')
 t.need(faults[0]['verification']['verified']==1 and faults[0]['verification']['errors']==1 and faults[0]['failed_functions']==['component::checked_foreign_bool_to_u64%'],'false helper proof')
 t.need(faults[1]['verification']['errors']>0,'false BV failure accounting')
 tests={}
 for name,count in [('local_checked_foreign-corrected',5),('infer_goals',12),('infer_nonzero_derive',3),('constructor_function_values',13),('vir',149)]:
  d=R/('test-'+name);s=(d/'stdout').read_text();t.need(f'{count} passed; 0 failed' in s and load(d/'receipt.json')['exit']==0,'tests '+name);tests[name]=dict(passed=count,receipt_sha256=t.sha(d/'receipt.json'))
 t.need('1 passed; 0 failed' in (R/'extra-contract-retry/stdout').read_text(),'nested contract test')
 text=generate('full-help');t.need(text==(R/'stable-full-help/component.rs').read_text(),'pure fidelity')
 t.need(text==(t.W/'source-full-help.rs').read_text(),'worker source identity')
 native_mir=R/'runtime/component.mir';t.need('checked_foreign_bool_to_u64' not in native_mir.read_text(),'erasure')
 t.need(load(R/'runtime-build/receipt.json')['verification']['verified']==52 and load(R/'runtime-execution/receipt.json')['exit']==0,'runtime')
 semantic=[]
 for p in sorted(R.glob('semantic-*/receipt.json'))+sorted(R.glob('packing-*/receipt.json')):
  if p.parent.name=='semantic-qualified':continue
  r=load(p);semantic.append(dict(label=p.parent.name,exit=r['exit'],verification=r.get('verification'),failed_functions=r.get('failed_functions'),receipt_sha256=t.sha(p)))
 for w in [8,16,32,64]:
  t.need(load(R/f'packing-{w}-bounded/receipt.json')['exit']==0 and load(R/f'packing-{w}-unbounded/receipt.json')['verification']['errors']>0,'packing premise '+str(w))
 production_objects={}
 for p in subprocess.check_output(['git','diff','--name-only','15045979','35bc7b12','--','source/rust_verify','source/vir','source/rust_verify_test/tests/local_checked_foreign.rs'],cwd=t.ROOT).decode().splitlines():
  production_objects[p]=dict(sha256=t.sha(t.ROOT/p),git_blob=subprocess.check_output(['git','rev-parse','35bc7b12:'+p],cwd=t.ROOT).decode().strip())
 test_binaries={str(p.relative_to(R)):t.sha(p) for stem in ['local_checked_foreign','infer_goals','infer_nonzero_derive','constructor_function_values','vir'] for p in (R/'build/debug/deps').glob(stem+'-*') if p.is_file() and __import__('os').access(p,__import__('os').X_OK)}
 result=dict(full_typed_helper_identity=load(R/'helper-identities.json'),read_only_audit_note='Interrupted only own exact read-only auditor commands to memoize duplicate immutable digests; final pass freshly rehashes production source and tools before output.',controllers={str(p.relative_to(E)):t.sha(p) for p in E.iterdir() if p.suffix in ['.py','.rs']},executed_runner_sha256=t.sha(R/'review.executed.py'),production_objects=production_objects,test_binaries=test_binaries,review_base='35bc7b12fff34551da06a9b81e65f1f20d530992',status='independent_local_review_passed',findings=[{'priority':'informational','summary':'Zero-shift arithmetic automation remains incomplete','repro':'semantic-shift-zero/component.rs','on':'1/1, healthy bool helper','off':'0/1, same source','ordinary_BV':'3/0','scope':'Conservative k=0 omission; no false success; does not block LOCAL bool capability'}],private_root=str(R),tools=tools,toolchain=load(R/'toolchain.json'),git=load(R/'git-object-identities.json'),production_manifest_sha256=t.sha(R/'source-manifest.json'),cache_inputs_manifest_sha256=t.sha(R/'cache-manifest.json'),worker_authentication_sha256=t.sha(R/'worker-authentication.json'),worker_authenticated_receipt_count=len(load(R/'worker-authentication.json')['receipts']),worker_fully_available_proof_control_vstd_receipts=52,worker_prebuild_snapshot_limit={'tool':'rust_verify','sha256':'80aa23c06acab915474781da22d1f3c8e2d8d59718f0c6ef9d0c44bea6e9b5b3','bytes':'unavailable; superseded prebuild snapshot, not a final build input or accepted proof tool'},body_inventories=inv,cold_warm=comparison,Eq_fresh_queries=eq,actual_cold_warm=load(R/'stable-actual/comparison.json'),full_source_sha256=t.sha(R/'stable-full-help/component.rs'),fidelity=fidelity(text),runtime_mir_sha256=t.sha(native_mir),tests=tests,additional_contract_test=load(R/'extra-contract-identity.json'),export_controls=exports,healthy_capfree_cargo_exports={n:t.sha(R/('cargo-capfree-'+n)/'libcomponent.vir') for n in ['ordinary','infer-aware']},additional_cargo_paths=9,incomplete_local_refusals=8,semantic_controls=semantic,source_controls=faults,receipts=receipts,excluded_setup=['sandbox bwrap RTM_NEWADDR before execution','setup inventory demanded absent optional workspace archives; available archive checksums validated and selected packages built offline/locked','extra-contract-tests: reduced workspace required lockfile change; exact complete original workspace restored for locked retry','test-local_checked_foreign: separate build-dir lacked profile/verus, ENOENT; fixed private symlinks and all5 rerun','semantic-qualified: unsuffixed spec literals caused E0283; typed fixture rerun','fault-build-false-proposal: private archive lacked tools/common/consts.rs; complete own Git tools source restored before new guarded controls','private fault retry first rejected already mutated private fixture; reset to exact own production bytes before final controls','false-BV diagnostic classifier expected assertion failed; corrected to actual bitvector assertion not satisfied on the unchanged archived 45/7 proof; no negative-proof requirement weakened'],inherited={'HMAC':'1/1 unproved, not rerun','canonical_parent':'6/0 inherited, not rerun','open_obligations':'44 minimal +3 phase /101 writer effects OPEN'},native=0,publication=False,goal_complete=False)
 t.need(CURRENT_SOURCE=={p:uncached_sha(t.ROOT/p) for p in CURRENT_SOURCE},'source changed during audit')
 t.need(tools=={n:uncached_sha(R/'bundle'/n) for n in t.TOOLS},'tools changed during audit')
 t.write(E/'RESULTS.json',result)
 print(json.dumps(dict(review='passed',receipts=len(receipts),tests=tests,candidate_id=comparison['cold']['candidate_id'],publication=False,native=0,goal_complete=False)))
if __name__=='__main__':main()
