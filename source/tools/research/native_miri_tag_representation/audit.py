"""Recheck explicitly named completed own artifacts; compact evidence only, Python -O safe."""
import hashlib
import json
from pathlib import Path
import sys
from trial import authenticate, check_data
from source_selection import require
from fidelity import fidelity
HERE=Path(__file__).resolve().parent

def digest(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def main():
 authenticate() # success and every one of eight accepted tool hashes; own support keys only
 rows=[]
 for name in json.loads((HERE/'epochs.json').read_bytes()):
  out=Path('/tmp')/('native-miri-tag-'+name)
  r=json.loads((out/'receipt.json').read_bytes())
  require(r['all_final_guards_passed'] is True,'epoch final guard failed: '+name)
  require(r['freeze_sha256']==digest(out/'input-freeze.json'),'epoch freeze drift')
  f=json.loads((out/'input-freeze.json').read_bytes())
  require(f['before_first_trial'] is True,'missing pre-epoch freeze')
  require(r['source_sha256']==f['source_sha256']==digest(out/'component.rs'),'body drift')
  require(r['commands'][0]['argv']==f['argv'],'argv differs')
  require(r['commands'][0]['exit_code']==r['terminal_exit'],'terminal exit differs')
  require(r['products'],'missing log/product hashes')
  for rel,p in r['products'].items():
   path=out/rel
   require(path.is_file() and digest(path)==p['sha256'] and path.stat().st_size==p['bytes'],'private product drift: '+str(path))
  # Later controller revisions live only in the own checkout; old identities are
  # retained by copied, hash-bound helper bytes in each private epoch.
  for p in out.glob('*.py'):
   require(str(p) in f['inputs'] and digest(p)==f['inputs'][str(p)]['sha256'],'helper snapshot drift')
  vr=r.get('verification')
  if vr:
   data=json.loads((out/'check.stdout').read_bytes())
   require(data['verification-results']==vr and data['verus-infer']==r['inference'],'parsed log mismatch')
   if r['success']:
    check_data(data,r['terminal_exit'],(out/'check.stderr').read_text(),r['expected'],f['body_inventory'],f['negative_function'])
   scope=r['inference'].get('verification_scope',{})
  else:scope={}
  # Harness success means its expected outcome was observed, not semantic truth.
  if name.startswith('actual-control-conversion-'):
   category='excluded_misclassified_true_property_standard_spec_gap'
  elif not r['success']:
   category='excluded_frontend' if vr and (vr.get('encountered-vir-error') or not r.get('inference',{}).get('verification_scope',{}).get('native_collection_completed')) else 'excluded_ordinary_proof_failure' if vr and not vr.get('success') else 'excluded_harness_expectation_mismatch'
  else:category=r['classification']
  rows.append(dict(name=r['name'],output=str(out),receipt_sha256=digest(out/'receipt.json'),
    input_freeze_sha256=r['freeze_sha256'],source_sha256=r['source_sha256'],
    controller_sha256=r['products']['trial.py']['sha256'],terminal_exit=r['terminal_exit'],
    verified=vr.get('verified') if vr else None,errors=vr.get('errors') if vr else None,
    category=category,expectation_met=r['success'],assistance=f['assistance'],
    checked_exec_bodies=scope.get('checked_exec_bodies'),checked_proof_bodies=scope.get('checked_proof_bodies'),
    failed_functions=r.get('inference',{}).get('failed_functions'),
    product_count=len(r['products']),product_manifest_sha256=hashlib.sha256(json.dumps(r['products'],sort_keys=True).encode()).hexdigest()))
 positive=json.loads((Path('/tmp/native-miri-tag-exact-final-on-48_484rs')/'receipt.json').read_bytes())
 inventory=positive['inference']['verification_scope']['native_functions']
 result=dict(schema=1,classification='partial_source_component_only_exact_constructor_closure_UNPROVED',
    scope={'native_correspondences':0,'minimal_open':44,'phase_open':3,'writer_effects_open':101,
           'full_hmac':'UNPROVED','publication':False,'goal_complete':False,'goal_status':'active'},
    epochs=rows,all_private_products_rechecked=True,epoch_count=len(rows),
    exact_original_body_subset_inventory=inventory,fidelity=fidelity(),
    final_evidence={'bitfields':'26/0 on and off; four manually authored proof bodies',
                    'original_body_subset':'45/0 on and off; ten exec and five proof bodies',
                    'explicit_closure_adapter_subset':'49/0 on and off; fourteen exec and five proof bodies',
                    'false_semantic_controls':'4 bitfield + 4 actual-carrier; each singleton failed query',
                    'true_property_spec_gaps':'runtime Eq/Lt and bool conversion failures are NOT semantic negatives',
                    'native_comparison':'616 finite observations; 5929 ordered input pairs; supporting only'},
    setup_exclusions=[{'tool_record':'cafb49','kind':'sandbox_startup','terminal_exit':1,'detail':'bwrap loopback RTM_NEWADDR Operation not permitted; exact read/proof escalations succeeded'},
                      {'tool_record':'6f3bec','kind':'controller_import','terminal_exit':1,'detail':'local select.py collided with Python standard select; renamed source_selection.py before trials'}])
 (HERE/'RESULTS.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({'epochs':len(rows),'private_products':'all hash-checked','python_optimization':sys.flags.optimize,
                   'results_sha256':digest(HERE/'RESULTS.json')}))

if __name__=='__main__':main()
