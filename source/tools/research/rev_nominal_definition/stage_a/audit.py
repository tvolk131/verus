"""Read-only Stage A evidence acceptance, identical with Python -O; no asserts.
This authenticates the dormant compiler slice, not Stage B or a native theorem.
"""
import hashlib,json,pathlib,subprocess,sys
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[4]
def need(b,m):
 if not b:raise RuntimeError(m)
def sha(p):
 with pathlib.Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def load(p):return json.loads(pathlib.Path(p).read_text())
def main():
 need(sys.flags.dont_write_bytecode,'use -B')
 j=load(HERE/'RESULTS.json');r=pathlib.Path(j['private_root'])
 need(j['stage_a']=='IMPLEMENTED_INACTIVE' and j['stage_b']=='UNIMPLEMENTED','stage boundary')
 for p,h in j['current_controllers'].items():need(sha(ROOT/p)==h,'current controller '+p)
 for p,h in j['archived_controllers'].items():need(sha(p)==h,'archived controller '+p)
 archived={h for h in j['archived_controllers'].values()}
 for name,t in j['epochs'].items():
  d=r/'epochs'/name;receipt=load(d/'receipt.json');f=load(d/'freeze.json')
  need(sha(d/'receipt.json')==t['receipt_sha256'],'immutable receipt '+name)
  need(sha(d/'freeze.json')==receipt['freeze_sha256'],'freeze '+name)
  need(sha(d/'stdout')==receipt['stdout_sha256'] and sha(d/'stderr')==receipt['stderr_sha256'],'log '+name)
  need(receipt['guards'] is True,'input guards '+name)
  need(f['controller'] in archived or f['controller'] in j['current_controllers'].values(),'controller version '+name)
  need(f['environment']['RUSTUP_HOME']=='/home/dev/.rustup' and f['environment']['CARGO_NET_OFFLINE']=='true' and f['environment']['CARGO_BUILD_JOBS']=='2','offline env '+name)
  if t['classification']=='accepted':need(receipt['exit']==t['exit'],'accepted status '+name)
 for name in ['build-04','vir-final-01','vstd-normal-01','focused-01-driver-build','focused-01-metadata','focused-01-ordinary','item-original-01']:
  f=load(r/'epochs'/name/'freeze.json')
  need(f['source']==load(r/'epochs/build-04/freeze.json')['source'],'matching final source '+name)
 source=load(r/'epochs/build-04/freeze.json')['source']
 for p,h in source.items():need(sha(ROOT/p)==h and sha(r/'src'/p)==h,'final source '+p)
 for n,h in j['eight_tools'].items():need(sha(r/'bundle'/n)==h,'eight-tool final guard '+n)
 need(len(j['eight_tools'])==8,'eight files')
 need(j['eight_tools']['z3']=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','solver pin')
 need((r/'bundle/z3').stat().st_mode & 0o111,'executable solver')
 boot=load(r/'bootstrap.json')
 for p,h in boot['toolchain'].items():need(sha(p)==h,'pinned toolchain '+p)
 for rel,h in boot['archives'].items():need(sha(r/rel)==h,'private archive '+rel)
 for rel,h in boot['index_git'].items():need(sha(r/'cargo-home'/rel)==h,'private index/git '+rel)
 for p,h in j['additional_toolchain_sources'].items():need(sha(p)==h,'pinned compiler source '+p)
 finalv=load(r/'epochs/vstd-normal-01/freeze.json')
 for n,h in finalv['tools'].items():need(j['eight_tools'][n]==h,'matching vstd producer '+n)
 need('verification results:: 2250 verified, 0 errors' in (r/'epochs/vstd-normal-01/stdout').read_text(),'NORMAL vstd 2250/0')
 need('--no-verify' not in ' '.join(finalv['argv']),'vstd normal checks')
 text=(r/'epochs/vir-final-01/stdout').read_text()
 need('157 passed; 0 failed; 0 ignored' in text,'VIR final suite')
 newtests=[l for l in text.splitlines() if 'test infer_checks::tests::internal_representation::' in l]
 need(len(newtests)==8 and all(l.endswith(' ... ok') for l in newtests),'eight focused tests')
 meta=(r/'epochs/focused-01-metadata/stdout').read_text()
 need('METADATA_PASS true WRONG_ADT_REFUSED true MUTATED_DESCRIPTOR_REFUSED true INSTALLED false' in meta,'actual loaded metadata controls')
 need('14147836729157185704' in meta and '5029879297925419223' in meta and 'Item' in meta,'actual semantic ADT/field/Item instance')
 for name,expected in [('focused-01-ordinary',(3,0)),('item-original-01',(132,0))]:
  f=load(r/'epochs'/name/'freeze.json');need(f['tools']==j['eight_tools'],'matching app eight tools '+name);raw=load(r/'epochs'/name/'stdout');v=raw['verification-results'];s=raw['verus-infer']['verification_scope']
  need(v['success'] and v['is-verifying-entire-crate'] and not v['encountered-vir-error'] and (v['verified'],v['errors'])==expected,'ordinary checks '+name)
  need(s['native_collection_completed'] and s['complete_selected_scope_verified'],'whole-crate scope '+name)
  need('--compile' in f['argv'] and '--no-cheating' in f['argv'],'Rust/body/erasure checks '+name)
  need(not any(a.startswith(('--verify-','--no-lifetime','--no-erasure','--no-verify')) for a in f['argv']),'no bypass '+name)
  need(load(j['body_inventories'][name]['path'])==raw['verus-infer'],'exact helper/model/body inventory '+name)
 need('E0616' in (r/'epochs/focused-01-privacy/stderr').read_text(),'source privacy negative')
 attr=(r/'epochs/focused-01-attribute/stderr').read_text();need('internal_representation' in attr and ('unrecognized' in attr or 'unknown' in attr),'unforgeable source attribute')
 for p,h in load(r/'focused-01/inputs.json').items():
  need(sha(p)==h or h in archived,'focused source/controller '+p)
 need((r/'focused-01/item.rs').read_bytes().startswith((r/'original_item.rs').read_bytes()),'byte preserved actual Item metadata witness')
 need(sha(r/'original_item.rs')=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','exact Item fixture')
 for p,h in j['products'].items():need(sha(p)==h,'product '+p)
 need(j['publication'] is False and j['goal_complete'] is False and j['native_correspondences']==0,'nonclaims')
 # Checkpoint files must be byte-identical to f90a6679; new stage files are added.
 checkpoint=subprocess.check_output(['git','ls-tree','-r','--name-only','f90a6679','source/tools/research/rev_nominal_definition'],cwd=ROOT,text=True).splitlines()
 for p in checkpoint:
  old=subprocess.check_output(['git','show','f90a6679:'+p],cwd=ROOT)
  need(hashlib.sha256(old).hexdigest()==sha(ROOT/p),'preserved checkpoint '+p)
 changed=subprocess.check_output(['git','diff','--name-only','f90a6679'],cwd=ROOT,text=True).splitlines()
 need(all(p.startswith('source/tools/research/rev_nominal_definition/') or p in j['compiler_changes'] for p in changed),'narrow change scope')
 print(json.dumps({'stage_a':'implemented_inactive','stage_b':'unimplemented','VIR_tests':157,'new_focused_tests':8,'matching_NORMAL_vstd':'2250/0','original_Item':'132/0','ordinary_borrow_smoke':'3/0','actual_core_metadata_controls':True,'source_privacy_and_attribute_controls':True,'new_descriptor_helpers':0,'native_correspondences':0,'publication':False,'goal_complete':False},sort_keys=True))
if __name__=='__main__':main()
