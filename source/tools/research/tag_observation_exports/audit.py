"""Read only named completed epochs; write compact owned RESULTS, no verification run."""
import json,re,sys
from pathlib import Path
import replay as e
from source import need, sha, HERE, original

def functions(d):
    text=(d/'vir/crate.vir').read_text();out={}
    for m in re.finditer(r'\(Function\s',text):
        f=text[m.start():e.inv.end(text,m.start())];name=e.inv.field(f,'name')[len('(Fun :path '):-1]
        if name.startswith('component::'):
            out[name]=dict(body_present=e.inv.field(f,'body')!='None',mode=e.inv.field(f,'mode'),body_visibility=e.inv.field(f,'body_visibility'),body_sha256=__import__('hashlib').sha256(e.inv.field(f,'body').encode()).hexdigest())
    return out

def eqhealthy(i):
    hs=i['helpers'];need(len(hs)==1 and hs[0]['function']=='component::BorTag::eq','exact inferred helper set')
    h=hs[0];need(h['mode']=='exact' and h['contract']['requires']=='true' and h['required_checks_passed'] is True,'full-domain exact Eq')
    need(set(h['proofs'])=={'soundness','maximality','exactness'},'three Eq checks')
    for p in h['proofs'].values():need(p['required'] is True and p['status']=='proved','mandatory Eq proof')

def rowaudit(row):
    d=Path(row['path']);need(sha(d/'receipt.json')==row['receipt_sha256'],'receipt identity')
    r=json.loads((d/'receipt.json').read_text());p=r['plan'];need(r['final_guards_passed'] is True and row['guards'] is True,'final guards')
    for n,h in r['products'].items():need(sha(d/n)==h,'retained product '+n)
    for n,h in r['inputs'].items():
        f=Path(n)
        if f.parent==HERE and f.suffix in ['.py','.md']:
            f=d/('controller-'+f.name)
        need(sha(f)==h,'frozen input '+n)
    need('--no-cheating' in r['argv'] and '--log-all' in r['argv'],'full ordinary checking flags')
    need(not any(a.startswith(('--verify-function','--verify-module','--no-verify','--rlimit')) for a in r['argv']),'no narrowed body scope')
    data=json.loads((d/'stdout').read_text());i=e.inv.inventory(d,'component' if p['role']=='provider' else 'client');v=i['verification'];err=(d/'stderr').read_text()
    need(v['is-verifying-entire-crate'] is True and not v['encountered-vir-error'],'full ordinary frontend')
    need('panicked at' not in err and 'internal compiler error' not in err,'no compiler failure')
    src=d/('component.rs' if p['role']=='provider' else 'client.rs');text=src.read_text()
    observed=e.identity(text,p['role']=='provider')
    need(all(observed[k]==v for k,v in p['source_identity'].items()),'exact frozen source/body identity fields')
    if p['expected_identity'] is not None:need(observed==p['source_identity'],'final complete identity schema')
    if p['expected_identity'] is not None:need(e.signature(d,p['role'])==p['expected_identity'],'pre-frozen exact VIR/helper/native/detail identity')
    selected=[]
    if p['role']=='provider':
        eqhealthy(i)
        need(sorted(i['ordinary_original_functions'])==p['expected_provider_ordinary_names'],'all ordinary provider signatures including generated constructors')
        need(sorted(i['local_function_details'])==p['expected_provider_function_detail_names'],'all ordinary function details')
        actual=[{k:v for k,v in b.items() if k not in ['source','body_status']} for b in i['native_scope']['native_functions']]
        need(sorted(actual,key=lambda b:b['function'])==sorted(p['expected_provider_native'],key=lambda b:b['function']),'complete HIR selected/excluded identity')
        hs=[{k:h[k] for k in ['function','mode','summary_kind','parameters','result_type','contract']} for h in i['helpers']]
        need(hs==p['expected_helpers'],'frozen helper contracts')
        execs={k for k,x in i['ordinary_original_functions'].items() if x['mode']=='Exec' and x['body_present']}
        need(len(execs)==12,'all 12 ordinary exec bodies')
        for ctor in ['component::BorTag::ctor%','component::Item::ctor%']:
            need(ctor in execs and ctor in i['local_function_details'],'generated constructor ordinary checks')
        original.fidelity(text)
        if p['adapter']=='zero':
            expected=(16,3);failed={'component::BorTag::new','component::BorTag::get','component::compare_equal'};classification='invalid_model_rejected_by_original_new_get_and_comparison_contracts'
        else:
            expected=(20 if p['adapter']=='compat' else 19,0);failed=set();classification='ordinary_component_positive'
    else:
        names={'client::'+x for x in p['expected_client_names']}
        need(set(i['ordinary_original_functions'])==names and set(i['local_function_details'])==names,'exact client ordinary bodies/details')
        need(all(x['mode']=='Exec' and x['body_present'] for x in i['ordinary_original_functions'].values()),'no client body omitted')
        native=i['native_scope']['native_functions']
        expected_hir=names | ({'_::__VERUS_REVEAL_INTERNAL__'} if p['adapter'].startswith('compat-') else set())
        need({b['function'] for b in native}==expected_hir and all(b['selected'] for b in native),'exact HIR client scope including broadcast-use expansion')
        for b in native:
            if b['function']=='_::__VERUS_REVEAL_INTERNAL__':
                need(b['body_status']=='selected_item_without_translated_body' and not b['ordinary_contract'] and not b['inference_candidate'],'exact generated broadcast-use item')
        need(i['helpers']==[] and data['verus-infer']['imported_inference_dependencies'],'certified imported dependency with no local helper')
        dep=Path(p['provider']['path']);receipt=json.loads((dep/'receipt.json').read_text());eqhealthy(e.inv.inventory(dep))
        need(sha(dep/'receipt.json')==p['provider']['receipt_sha256'] and receipt['final_guards_passed'] and receipt['exit']==0,'healthy immutable Eq provider')
        provider_names=e.inv.inventory(dep)['ordinary_original_functions'];fs=functions(d)
        imported={k:(fs[k] if k in fs else dict(entire_function_absent_after_pruning=True)) for k in provider_names}
        for k,x in fs.items():
            if any(k.endswith('::'+s) for s in ['eq','eq_spec','__verus_infer_nonzero_u64_view','value','word','new','get','tag','eq_value_compatibility']):
                # The excluded alias probe deliberately exports an open value alias.
                if k.endswith('::value') and receipt['plan']['adapter']=='alias':continue
                need(not x['body_present'],'closed/exec/proof body leaked at import '+k)
        # Record signatures as well as every unused pruned symbol, without relaxing scope.
        i['all_imported_provider_symbols']=imported
        i['imported_inference_dependencies']=data['verus-infer']['imported_inference_dependencies']
        mode=p['adapter']
        if mode=='compat-false':expected=(1,1);failed={'client::equal'};classification='healthy_false_returned_relation_control'
        elif mode=='equal':expected=(1,1);failed={'client::equal'};classification='opaque_trait_observation_composition_gap'
        else:expected=({'compat-equal':2,'lookup':1,'boundaries':3}[mode],0);failed=set();classification='ordinary_import_positive'
    need((v['verified'],v['errors'])==expected,'exact verification results '+row['label'])
    need(set(i['failed_functions'])==failed,'exact failed-function set')
    need(v['success']==(not failed) and r['exit']==(1 if failed else 0),'verification exits')
    if failed:
        need(not i['inference_interface_written'] and not (d/'component.vir').exists(),'failed crate publication barrier')
        need(not i['native_scope']['complete_selected_scope_verified'],'failed crate not certified')
        need('postcondition not satisfied' in err and 'precondition not satisfied' not in err and 'error[E' not in err,'postcondition-only diagnostic')
    else:
        need(i['native_scope']['complete_selected_scope_verified'],'full selected scope healthy')
        need(all(not x['failed_proof_notes'] for x in i['local_function_details'].values()),'ordinary function details healthy')
        selected=[b['function'] for b in i['native_scope']['native_functions'] if b['body_status'] in ['checked_exec_body','checked_proof_body']]
        if p['role']=='provider':
            need(i['inference_interface_written'] and (d/'component.vir').is_file() and (d/'libcomponent.rlib').is_file(),'private certified export')
            need(len(selected)==(11 if p['adapter']=='compat' else 10),'HIR checked executable/proof inventory')
        else:
            need(not i['inference_interface_written'],'client has no inference export')
            need(set(selected)==names,'all client bodies ordinarily checked')
    return dict(**row,classification=classification,identity_replayed=p['expected_identity'] is not None,source_sha256=sha(src),source_identity=p['source_identity'],freeze_sha256=sha(d/'freeze.json'),stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),product_count=len(r['products']),inventory=i)

def main():
    bundle=e.authenticate();base=json.loads((HERE/'BASELINE.json').read_text())
    for record in base['observations'].values():
        d=Path(record['path']);need(sha(d/'receipt.json')==record['receipt_sha256'],'baseline still frozen')
    rows=[json.loads(x) for x in (HERE/'attempts.jsonl').read_text().splitlines()]
    need(len({r['label'] for r in rows})==len(rows),'unique attempts')
    audited=[rowaudit(r) for r in rows]
    final={x['label']:x for x in audited if x['label'].endswith('-final')}
    required={'provider-final','provider-off-final','client-final','false-final','boundaries-final','no-broadcast-final','lookup-final','zero-final'}
    need(set(final)==required and all(x['identity_replayed'] for x in final.values()),'complete frozen final replay set')
    need(final['provider-final']['source_sha256']==final['provider-off-final']['source_sha256'],'same source optional goals on/off')
    old=Path(base['observations']['ordinary-import-final']['path'])
    receipt=json.loads((old/'receipt.json').read_text())
    source_hashes=[h for n,h in receipt['inputs'].items() if n.endswith('/client.rs')]
    reconstructed=__import__('hashlib').sha256(e.client('equal').encode()).hexdigest()
    need(source_hashes==[reconstructed],'exact original client source reproduced from authenticated receipt')
    accepted=(Path(final['client-final']['path'])/'client.rs').read_text()
    need(accepted.replace('broadcast use component::eq_value_compatibility;\n','',1)==e.client('equal'),'only change to original consumer is checked broadcast import')
    for label in ['client-final','false-final','boundaries-final','no-broadcast-final','lookup-final']:
        rr=json.loads((Path(final[label]['path'])/'plan.json').read_text());need(rr['provider']['path']==final['provider-final']['path'],'final clients use final healthy provider')
    # Client runtime bodies stay identical across the passing, no-broadcast and false-relation versions.
    def body_tokens(label,anchor):
        text=(Path(final[label]['path'])/'client.rs').read_text();body=original.block(text,anchor);return re.findall(r'\w+|[^\s]',body[body.index('{'):])
    for anchor in ['pub fn read(', 'pub fn equal(']:
        need(body_tokens('client-final',anchor)==body_tokens('no-broadcast-final',anchor)==body_tokens('false-final',anchor),'same ordinary runtime client bodies')
    status=__import__('subprocess').check_output(['git','status','--porcelain','--untracked-files=all'],cwd=e.ROOT).decode().splitlines()
    need(all(x[3:].startswith('source/tools/research/tag_observation_exports/') for x in status),'exclusive edit ownership')
    result=dict(decision='checked_source_compatibility_interface_closes_imported_Eq_value_gap',base='098a8be66e00734a010bf525ff832a81157621ea',bundle_receipt_sha256=e.RECEIPT,tools=bundle['tools'],versions=dict(schema=38,policy=69,cache=72,vstd='2250/0'),baseline_sha256=sha(HERE/'BASELINE.json'),final=final,attempt_ledger=[{k:v for k,v in x.items() if k!='inventory'} for x in audited],all_receipt_product_and_input_guards_passed=True,controllers={p.name:sha(p) for p in HERE.glob('*.py')},identities_sha256=sha(HERE/'identities.json'),manual=dict(provider_checked_broadcast_proof_bodies=1,provider_proof_body='empty, ordinarily checked',provider_local_proof_blocks=0,provider_theorem_calls=0,client_broadcast_imports=1,client_local_proof_blocks=0,client_explicit_theorem_calls=0,client_runtime_calls_added=0,client_snapshots=0),publication=False,native_correspondences=0,full_Item_new='UNPROVED',native_ordering='UNPROVED',HMAC='1/1_UNPROVED',whole_goal='PENDING',goal_complete=False)
    e.write(HERE/'RESULTS.json',result)
    print(json.dumps(dict(attempts=len(rows),provider='20/0 on/off',client='2/0',false='1/1 only intended postcondition',zero='16/3 invalid model',guards=True)))
if __name__=='__main__':main()
