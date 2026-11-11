"""Bind focused replay's native identities/source scopes, supplementing count gates."""
import json,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2];E=Path(__file__).resolve().parent;O=Path('/tmp/practical-set-review-902041b3');C=R/'source/tools/research/practical_redb_fedimint'
sys.path.insert(0,str(R/'source/tools'));from infer_eval_support import digest
r=json.loads((O/'fresh-replay/receipt.json').read_text());assert r['success'] and len(r['outcomes'])==25
re=[o+'::'+n for o,ns in [('BtreeBitmap',['get_level','get_height','set','clear','get_level_mut','update_to_root']),('U64GroupedBitmap',['data_index_of','select_mask','len','get','set','clear'])] for n in ns]
rp=['word_bits','word_frame','layer_edit','leaf_edit','clear_not_full','finish_mutation','propagation','ancestor_next']
fe=['<NumPeers as std::clone::Clone>::clone','<NumPeers as std::convert::From<usize>>::from']+['NumPeers::'+n for n in ['total','max_evil','one_honest','threshold','degree']]+['policy_client']
fp=['quorum_intersection','quorum_has_honest','one_honest_margin']
inventories={}
for label,ex,pr,src,pairs in [
 ('r2_cold',re,rp,C/'common_set/r2-reduced.rs',{'finish_mutation':20}),('r2_repeat',re,rp,C/'common_set/r2-reduced.rs',{'finish_mutation':20}),
 ('f2_cold',fe,fp,C/'common_set/f2-reduced.rs',dict(zip(fp,[20,6,6]))),('f2_repeat',fe,fp,C/'common_set/f2-reduced.rs',dict(zip(fp,[20,6,6]))),
 ('r2_export',re+['CheckedBitmap::mark_allocated','CheckedBitmap::mark_free'],rp,O/'fresh-replay/r2_api.rs',{'finish_mutation':20}),
 ('f2_export',fe,fp,O/'fresh-replay/f2_api.rs',dict(zip(fp,[20,6,6]))),
 *[(f'r2_import_{g}',['free_membership','record_alloc_membership','repeat_alloc_membership'],[],C/'redb_tree/callers.rs',{}) for g in [False,True]],
 *[(f'f2_import_{g}',['admission_observation','boundary_observations'],[],C/'fedimint_quorum/client.rs',{}) for g in [False,True]],
 ('generic_relations',[],['subset_size','extensional','inclusion','unsupported_mutable'],C/'common_set/compiler.rs',{'subset_size':2,'extensional':2,'inclusion':12})]:
 d=json.loads((O/'fresh-replay'/(label+'.stdout')).read_text());inf=d['verus-infer'];ns=inf['verification_scope']['native_functions'];assert inf['functions']==[]
 for status,names in [('checked_exec_body',ex),('checked_proof_body',pr)]:
  found=[x for x in ns if x['body_status']==status];assert sorted(x['rust_function'] for x in found)==sorted(names),(label,status)
  assert all(x['source'].startswith(str(src)+':') and x['selected'] for x in found),label
 found={x['function'].rsplit('::',1)[-1]:x['pairs'] for x in inf['set_pair_assistance']};assert found==pairs,label
 inventories[label]={'native_functions':[x for x in ns if x['body_status'].startswith('checked_')],'pair_scopes':inf['set_pair_assistance'],'source_sha256':digest(src)}
b=json.loads((O/'receipt.json').read_text());assert b['success']
for n,h in b['source'].items():assert digest(R/n)==h,n
for n,h in b['tools'].items():assert digest(O/'bundle'/n)==h,n
text=(O/'units.stdout').read_text();assert '145 passed; 0 failed' in text and '163 passed; 0 failed' in text
assert 'verification results:: 2251 verified, 0 errors' in (O/'vstd.stdout').read_text()
(E/'FRESH_INVENTORIES.json').write_text(json.dumps(inventories,indent=2)+'\n')
print('Bound fresh native names, source scopes, pair identities, source/tool hashes, units and NORMAL vstd.')
i=json.loads((O/'independent/receipt.json').read_text());assert i['success'] and len(i['outcomes'])==13
out={x['label']:x for x in i['outcomes']}
for x in out.values():
 d=x['data'];assert not d['verification-results']['encountered-vir-error'];assert d['verus-infer'].get('functions',[])==[]
 if x['label'] not in ['termination','bound_false']:assert d['verification-results']['success'] and x['exit_code']==0
for label in ['compatibility_false','compatibility_true']:
 d=out[label]['data'];assert d['verification-results']['verified']==7 and not (O/'independent'/(label+'.stderr')).read_text();ns=d['verus-infer']['verification_scope']['native_functions'];assert sorted(x['rust_function'] for x in ns if x['body_status']=='checked_proof_body')==sorted(['guarded','entry_guard','guarded_index','nested','references','over_budget','local_scope'])
 pairs={x['function'].rsplit('::',1)[-1]:x['pairs'] for x in d['verus-infer']['set_pair_assistance']};assert pairs==({'guarded':6,'entry_guard':6,'guarded_index':6,'nested':2} if label.endswith('true') else {})
for label in ['recommends_false','recommends_true']:
 assert 'warning: recommendation not met' in (O/'independent'/(label+'.stderr')).read_text();assert not out[label]['data']['verus-infer']['set_pair_assistance']
for label,target,diagnostic,pairs in [('termination','recursive','could not prove termination',2),('bound_false','bound_false','postcondition not satisfied',0)]:
 x=out[label];d=x['data'];assert x['exit_code']==1 and not d['verification-results']['success'] and d['verification-results']['errors']==1;assert [n.rsplit('::',1)[-1] for n in d['verus-infer']['failed_functions']]==[target];err=(O/'independent'/(label+'.stderr')).read_text();assert diagnostic in err and 'error[E' not in err
 assert sum(p['pairs'] for p in d['verus-infer']['set_pair_assistance'])==pairs
print('Independent positive/negative/recommendation diagnostic controls all match their exact expectations.')
