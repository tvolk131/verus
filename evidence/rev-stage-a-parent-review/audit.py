"""Read-only explicit-exception acceptance. Identical checks with python -O."""
import hashlib,json,pathlib,subprocess,sys
H=pathlib.Path(__file__).resolve().parent;ROOT=H.parents[1]
def need(ok,message):
 if not ok:raise RuntimeError(message)
def sha(p):
 with pathlib.Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def load(p):return json.loads(pathlib.Path(p).read_text())
def main():
 need(sys.flags.dont_write_bytecode,'run with -B')
 j=load(H/'RESULTS.json');r=pathlib.Path(j['private_root']);source=load(r/'shipping.json')
 need(sha(r/'shipping.json')==j['source_manifest_sha256'],'source manifest binding')
 for p,h in source.items():need(sha(ROOT/p)==h and sha(r/'src'/p)==h,'owned matching source '+p)
 for p,h in j['compiler_source'].items():need(source[p]==h,'compiler source '+p)
 for p,h in j['controllers'].items():need(sha(ROOT/p)==h,'review controller '+p)
 for p,h in j['checkpoint_hashes'].items():
  need(sha(ROOT/p)==h,'checkpoint current '+p)
  need(hashlib.sha256(subprocess.check_output(['git','show','f90a6679:'+p],cwd=ROOT)).hexdigest()==h,'historical checkpoint '+p)
 for p,h in j['toolchain'].items():need(sha(p)==h,'pinned toolchain/core '+p)
 boot=load(r/'bootstrap.json')
 for rel,h in boot['archives'].items():need(sha(r/rel)==h,'offline archive '+rel)
 for rel,h in boot['index_git'].items():need(sha(r/'cargo-home'/rel)==h,'private index/git '+rel)
 for n,h in j['eight_tools'].items():need(sha(r/'bundle'/n)==h,'final tool '+n)
 need(len(j['eight_tools'])==8 and j['eight_tools']['z3']=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','eight tools/solver')
 need((r/'bundle/z3').stat().st_mode&0o111,'solver executable')
 for p,h in j['products'].items():need(sha(r/p)==h,'private product '+p)
 for name,record in j['epochs'].items():
  d=r/'epochs'/name;receipt=load(d/'receipt.json');f=load(d/'freeze.json')
  need(sha(d/'receipt.json')==record['receipt_sha256'],'receipt '+name)
  need(sha(d/'freeze.json')==receipt['freeze_sha256'],'freeze '+name)
  for n in ['stdout','stderr']:need(sha(d/n)==receipt[n+'_sha256'],'log '+name)
  need(receipt['exit']==record['exit'] and receipt['guards'],'exit/guards '+name)
  need(f['source']==source,'matching parent source '+name)
  env=f['environment'];need(env['RUSTUP_HOME']=='/home/dev/.rustup' and env['CARGO_BUILD_JOBS']=='2' and env['CARGO_NET_OFFLINE']=='true','owned offline environment '+name)
  need(env['CARGO_HOME']==str(r/'cargo-home') and env['TMPDIR']==str(r/'tmp'),'private cache/tmp '+name)
  if record['classification']!='accepted':continue
  for tool,h in f['tools'].items():need(j['eight_tools'][tool]==h,'matched tool '+name+'/'+tool)
  inputs=d/'inputs.json'
  if inputs.exists():
   for p,h in load(inputs).items():need(sha(p)==h,'extra command input '+name+'/'+p)
  text=(d/'stdout').read_text()
  expected={'vir-157-final':157,'rust-verify-lib8-final':8,'existing-constructors13-final':13,'independent-tests-wf2-final':6}
  if name in expected:need(f"{expected[name]} passed; 0 failed; 0 ignored" in text,'test outcomes '+name)
 need('2250 verified, 0 errors' in (r/'epochs/vstd-normal-01/stdout').read_text(),'normal vstd')
 for p,h in load(r/'model-source.json').items():need(source[p]==h,'model source '+p)
 need('--no-verify' not in ' '.join(load(r/'epochs/vstd-normal-01/freeze.json')['argv']),'normal vstd command')
 need(sha(H/'INVENTORY.json')==j['inventory_sha256'],'compact inventory')
 inventory=load(H/'INVENTORY.json')
 for name,expected in [('item132',132),('wrapper28-final',28),('packet31',31),('bool_provider',2),('bool_import',1),('ordinary_borrow3',3),('generic_nested',4)]:
  d=r/'epochs'/name;f=load(d/'freeze.json');raw=load(d/'stdout');v=raw['verification-results'];s=raw['verus-infer']['verification_scope'];inv=inventory['apps'][name]
  need(v['success'] and not v['encountered-vir-error'] and (v['verified'],v['errors'])==(expected,0) and v['is-verifying-entire-crate'],'normal source proof '+name)
  need(s['native_collection_completed'] and s['complete_selected_scope_verified'],'scope '+name)
  need('--no-cheating' in f['argv'] and '--compile' in f['argv'],'normal compilation '+name)
  need(not any(a.startswith(('--no-lifetime','--no-erasure','--no-verify','--no-trait','--verify-')) for a in f['argv']),'no bypass '+name)
  need(f['tools']==j['eight_tools'],'full eight tools '+name)
  need(sha(r/'apps'/name/'inventory.json')==inv['exact_inventory_sha256'],'raw body inventory '+name)
  lm=r/'apps'/name/'log-manifest.json';need(sha(lm)==inv['log_manifest_sha256'],'log inventory '+name)
  for p,h in load(lm).items():need(sha(r/'apps'/name/'logs'/p)==h,'AIR/VIR logs '+name+'/'+p)
 false=load(r/'epochs/bool_false/stdout')['verification-results']
 need(false['errors']==1 and not false['encountered-vir-error'] and not false['success'],'ordinary semantic negative')
 for label,needle in [('old_schema','incompatible inference schema'),('old_policy','incompatible inference schema, safety policy, or producer'),('truncated_base','failed to deserialize imported library'),('rust-privacy','E0616'),('forged-attribute','unrecognized verifier attribute')]:
  need(needle in (r/'epochs'/label/'stderr').read_text(),'diagnostic '+label)
 for label in ['private_pattern','private_constructor','private_contract']:
  text=(r/'epochs'/label/'stderr').read_text();need('private' in text or 'non-visible' in text,'privacy '+label)
 meta=(r/'epochs/actual-metadata-rerun/stdout').read_text()
 need('METADATA_PASS true WRONG_ADT_REFUSED true MUTATED_DESCRIPTOR_REFUSED true INSTALLED false' in meta and 'Item' in meta,'actual metadata')
 need('WIRE legacy old0/old1 compatible; Internal encode refused; invalid/truncated tags refused' in (r/'epochs/independent-tests-wf2-final/stdout').read_text(),'actual wire')
 for p,h in load(r/'diagnostic-source.json').items():need(sha(r/'diagnostic-vir-v8'/p)==h,'test-only source '+p)
 # Assert the diagnostic source differs ONLY by the committed review test module.
 base=r/'src/source/vir/src';testpath='infer_checks/tests/internal_representation.rs'
 for p,h in load(r/'diagnostic-source.json').items():
  if p!=testpath:need(sha(base/p)==h,'unchanged diagnostic compiler source '+p)
 need((r/'diagnostic-vir-v8'/testpath).read_bytes()==(base/testpath).read_bytes()+b'\n'+(H/'independent.rs').read_bytes(),'only test-only instrumentation')
 need(sha(r/'apps/item132/input.rs')=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','exact Item')
 for label in ['item132','ordinary_borrow3']:
  inv=inventory['apps'][label]['erased_runtime'];need(sha(r/'apps'/label/'runtime.mir')==inv['sha256'] and len(inv['bodies'])>0,'runtime inventory '+label)
 changed=subprocess.check_output(['git','diff','--name-only','f4415999'],cwd=ROOT,text=True).splitlines()
 need(all(p.startswith('evidence/rev-stage-a-parent-review/') for p in changed),'evidence-only review')
 need(j['acceptance']['production_installer'] is False and j['acceptance']['native_correspondences']==0 and j['acceptance']['publication'] is False and j['acceptance']['goal_complete'] is False,'StageA nonclaims')
 print(json.dumps(j['acceptance'],sort_keys=True))
if __name__=='__main__':main()
