#!/usr/bin/env python3
"""Independent read-only acceptance for final bounded trials. Valid under python -O."""
import hashlib,json,re,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
from fidelity import check

def need(b,m):
 if not b:raise ValueError(m)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
EXEC={'<BorTag as std::clone::Clone>::clone','<Item as std::clone::Clone>::clone','<Permission as std::clone::Clone>::clone','Item::perm','Permission::from_bits','Stack::new','StackCache::add'}
PROOF={'add_effect_preserves_consistency'}
SPEC={'Item::permission','add_effect','cache_consistent','decode'}
AUX={'Permission::VERUS_UNERASED_PROXY__DISABLED','Permission::VERUS_UNERASED_PROXY__SHARED_READ_ONLY','Permission::VERUS_UNERASED_PROXY__SHARED_READ_WRITE','Permission::VERUS_UNERASED_PROXY__UNIQUE','VERUS_UNERASED_PROXY__CACHE_LEN','VERUS_UNERASED_PROXY__PERM_MASK','VERUS_UNERASED_PROXY__PERM_SHIFT','_::<impl vstd::prelude::Structural for Permission>::assert_receiver_is_structural'}
POS={'cache-enabled-cold','cache-enabled-warm','cache-assistance-off'}
NEG={'cache-wrong-index-head':'StackCache::add','cache-short-index-shift':'StackCache::add','cache-invalid-initial-index':'Stack::new','cache-false-preservation-property':'add_effect_preserves_consistency'}
LIT={'literal-enabled-on','literal-enabled-off'}
DIAG={'reverse-laws-diagnostic':'assertion failed','reverse-for-diagnostic':'loop invariant not satisfied'}

def main():
 receipt=Path(sys.argv[1]).resolve();out=receipt.parent;r=json.loads(receipt.read_text())
 need(r['success'] is True and r['all_final_guards_passed'] is True,'epoch not completed')
 need(r['native_correspondences']==0 and r['publication'] is False and r['goal_complete'] is False,'scope claim')
 need(len(r['authenticated_bundle_names'])==8,'tool authentication')
 need(set(x['name'] for x in r['runs'])==POS|set(NEG)|LIT|set(DIAG),'trial names')
 need(len(r['runs'])==len(POS|set(NEG)|LIT|set(DIAG)),'duplicate trials')
 for k,v in r['products'].items():need(sha(out/k)==v,'product changed '+k)
 for p,v in r['frozen_inputs']['inputs'].items():need(sha(p)==v['sha256'],'frozen input changed '+p)
 canonical=(HERE/'cache_component.rs').read_text();fidelity=check(HERE/'cache_component.rs')
 expected_neg={
 'cache-wrong-index-head':canonical.replace('self.idx[0] = idx;','self.idx[0] = 0;'),
 'cache-short-index-shift':canonical.replace('self.idx.copy_within(0..CACHE_LEN - 1, 1);','self.idx.copy_within(0..CACHE_LEN - 2, 1);'),
 'cache-invalid-initial-index':canonical.replace('idx: [0; CACHE_LEN], items:', 'idx: [1; CACHE_LEN], items:'),
 'cache-false-preservation-property':canonical.replace('ensures cache_consistent(after, borrows),','ensures cache_consistent(after, borrows), after.idx[0] as int == idx as int + 1,')}

 accepted=[]
 for x in r['runs']:
  n=x['name'];stdout=out/(n+'.stdout');stderr=(out/(n+'.stderr')).read_text();d=json.loads(stdout.read_text());v=d['verification-results']
  need(v==x['verification'],'reported verification differs')
  need(not any(z in stderr.lower() for z in ['panicked at','timed out','resource limit']),'nonsemantic failure '+n)
  need((x['exit_code']==0)==(n in POS),'terminal outcome '+n)
  need(len([c for c in r['commands'] if c['label']==n])==1,'command identity '+n)
  cmd=next(c for c in r['commands'] if c['label']==n)
  need(cmd['exit_code']==x['exit_code'],'command terminal mismatch')
  need(cmd['argv'][0].endswith('/parent-stable-key-bundle-a93b774a/verus'),'wrong tool')
  need('--no-cheating' in cmd['argv'] and '--output-json' in cmd['argv'],'missing normal checks')
  source=Path(cmd['argv'][1]);need(sha(source)==x['source_sha256'],'trial source')
  if n in POS|set(NEG):
   inf=d['verus-infer'];scope=inf['verification_scope'];selected=[b for b in scope['native_functions'] if b['selected']]
   need(inf['functions']==[] and inf['requested_functions']==0 and inf['generated_functions']==0,'unexpected assistance')
   need(scope['native_collection_completed'] is True,'partial collection')
   need({b['rust_function'] for b in selected}==EXEC|PROOF|SPEC|AUX and len(selected)==20,'selected body inventory')
   need(v['is-verifying-entire-crate'] is True and not v['encountered-vir-error'],'frontend control')
   need(v['verified']==(17 if n in POS else 16) and v['errors']==(0 if n in POS else 1),'query inventory')
   need(inf['crate_success'] is (n in POS),'inference crate outcome')
   if n in POS:
    need(source.read_text()==canonical,'positive source differs')
    need(scope['complete_selected_scope_verified'] is True and v['success'] is True,'incomplete positive')
    need(scope['checked_exec_bodies']==7 and scope['checked_proof_bodies']==1,'body counts')
    need({b['rust_function'] for b in selected if b['body_status']=='checked_exec_body'}==EXEC,'exec scope')
    need({b['rust_function'] for b in selected if b['body_status']=='checked_proof_body'}==PROOF,'proof scope')
    need({b['rust_function'] for b in selected if b['body_status']=='specification_definition_or_declaration'}==SPEC,'spec scope')
    need({b['rust_function'] for b in selected if b['body_status']=='selected_item_without_translated_body'}==AUX,'aux scope')
    need(inf['failed_functions']==[],'failed positive')
   else:
    target=[b['function'] for b in selected if b['rust_function']==NEG[n]]
    need(len(target)==1 and inf['failed_functions']==target,'failure must target exactly '+NEG[n])
    need('postcondition not satisfied' in stderr,'intended property rejection')
    need(not any(z in stderr for z in ['precondition not satisfied','assertion failed','possible arithmetic underflow/overflow','invariant not satisfied','recommendation not met','error[E']),'mixed semantic rejection')
    need(v['success'] is False,'negative accepted')
    need(source.read_text()==expected_neg[n],'negative source transform '+n)
    if n=='cache-false-preservation-property':check(source)
    else:
     try:check(source)
     except ValueError as e:need(str(e)=='executable drift '+('new' if n=='cache-invalid-initial-index' else 'add'),'wrong fidelity rejection')
     else:raise ValueError('source negative passed fidelity')
  elif n in LIT:
   need(v.get('encountered-vir-error') is True and v['success'] is False,'literal must be frontend')
   for marker in ['datatype constructor as a function value','FxBuildHasher','is_some_and','swap']:
    need(marker in stderr,'missing literal blocker '+marker)
  else:
   need(v.get('encountered-vir-error') is False and v['errors']==1 and DIAG[n] in stderr,'reverse-law diagnostic')
  accepted.append(dict(name=n,exit_code=x['exit_code'],classification=x['classification'],verified=v.get('verified'),errors=v.get('errors'),failed_target=NEG.get(n)))
 # Literal source is identical across assistance on/off; cold/warm use one cache.
 commands={c['label']:c for c in r['commands']}
 need(Path(commands['literal-enabled-on']['argv'][1]).read_bytes()==Path(commands['literal-enabled-off']['argv'][1]).read_bytes(),'literal causal source differs')
 need(commands['cache-enabled-cold']['environment']['VERUS_INFER_CACHE']==commands['cache-enabled-warm']['environment']['VERUS_INFER_CACHE'],'warm cache differs')
 need('--infer-goals' not in commands['cache-assistance-off']['argv'],'off still enabled')
 need('--infer-goals' in commands['cache-enabled-cold']['argv'],'on not enabled')
 print(json.dumps(dict(success=True,classification='bounded_cache_component_only',receipt_sha256=sha(receipt),fidelity=fidelity,trials=accepted,inferred_helpers=[],native_correspondences=0,publication=False,goal_complete=False),indent=2))
if __name__=='__main__':main()
