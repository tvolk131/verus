"""Audit explicitly named own completed epochs. Never executes an old proof driver."""
import hashlib,json,re,sys
from functools import lru_cache
from pathlib import Path
import epoch as e
from source import need,sha as raw_sha,fidelity
sha=lru_cache(maxsize=None)(raw_sha)
from inventory import inventory,signature,field,end

def classify(label,r,d):
    err=(d/'stderr').read_text();out=(d/'stdout').read_text()
    if r['plan'].get('frozen_expected_identity') and signature(d)!=r['plan']['frozen_expected_identity']:return 'controller_identity_rejection_after_verifier_success'
    if r['exit'] is None:return 'resource_or_launch_failure'
    if r['exit']==128:return 'setup_missing_rustup_PATH'
    if 'panicked at' in err or 'internal compiler error' in err:return 'compiler_failure'
    try:v=json.loads(out)['verification-results']
    except (ValueError,KeyError):return 'source_inspection_or_tool_build' if r['exit']==0 else 'setup_or_frontend_failure'
    if v.get('encountered-vir-error') or not v.get('is-verifying-entire-crate'):return 'frontend_failure'
    if v['success']:return 'ordinary_component_positive'
    if label in ['false-final','false-frozen-v2']:return 'healthy_semantic_control'
    if label in ['invalid-final','invalid-frozen-v2']:return 'invalid_representation_precondition_diagnostic'
    if label.startswith('bool-actual'):return 'library_contract_proof_gap'
    if label.startswith('ordinary-import'):return 'import_frontend_failure' if 'error[E' in err else 'closed_observer_equality_import_gap'
    if label.startswith('full'):return 'true_property_proof_gap'
    return 'excluded_boundary_or_mixed_control_proof_failure'

def eqhealthy(i):
    hs=i['helpers'];need(len(hs)==1 and hs[0]['function']=='component::BorTag::eq','exact Eq helper')
    h=hs[0];need(h['mode']=='exact' and h['contract']['requires']=='true' and h['required_checks_passed'] is True,'Eq exact domain')
    need(set(h['proofs'])=={'soundness','maximality','exactness'},'three obligations')
    for p in h['proofs'].values():need(p['required'] is True and p['status']=='proved','Eq proof missing')

def ordinary(d,label,expected,failed=None):
    r=json.loads((d/'receipt.json').read_text());p=r['plan'];i=inventory(d);v=i['verification'];eqhealthy(i)
    need(v.get('is-verifying-entire-crate') is True and not v.get('encountered-vir-error'),'full ordinary scope '+label)
    need((v['verified'],v['errors'])==expected,'query inventory '+label)
    need(p.get('frozen_expected_identity')==signature(d),'pre-epoch body/helper/details/native freeze '+label)
    need(i['failed_functions']==([] if failed is None else [failed]),'failed singleton '+label)
    if failed is None:
        need(r['exit']==0 and v['success'] and i['native_scope']['complete_selected_scope_verified'],'full success '+label)
        selected=[b for b in i['native_scope']['native_functions'] if b['body_status'] in ['checked_exec_body','checked_proof_body']]
        need({b['function'] for b in selected}==set(p['expected_native_checked']),'native checked set '+label)
        need(all(not v['failed_proof_notes'] for v in i['local_function_details'].values()),'function details failure')
        for ctor in ['component::BorTag::ctor%','component::Item::ctor%']:
            need(ctor in i['local_function_details'],'separate generated constructor detail')
        need(i['inference_interface_written'] and (d/'component.vir').is_file(),'private export missing')
    else:
        need(r['exit']==1 and not v['success'] and not i['inference_interface_written'] and not (d/'component.vir').exists(),'failure published')
    fidelity((d/'component.rs').read_text())
    i.update(path=str(d),source_sha256=sha(d/'component.rs'),receipt_sha256=sha(d/'receipt.json'),manual_proof_blocks=p['manual_proof_blocks'],manual_proof_functions=p.get('manual_proof_functions',[]))
    return i

def main():
    bundle=e.authenticate()
    retention=json.loads((e.HERE/'retention.json').read_text());need(sha(retention['path'])==retention['sha256'],'supplemental retention receipt')
    for p,h in json.loads(Path(retention['path']).read_text())['files'].items():need(sha(p)==h,'retained supplemental product')
    rows=[json.loads(x) for x in (e.HERE/'attempts.jsonl').read_text().splitlines()];paths={x['label']:Path(x['path']) for x in rows};ledger=[]
    for row in rows:
        d=Path(row['path']);need(sha(d/'receipt.json')==row['receipt_sha256'],'receipt '+row['label']);r=json.loads((d/'receipt.json').read_text())
        need(r['final_guards_passed'] is True and row['guards'] is True,'final guard '+row['label'])
        for name,h in r['products'].items():need(sha(d/name)==h,'product '+row['label']+' '+name)
        for path,h in r['inputs'].items():
            p=Path(path)
            if p.parent==e.HERE:
                snapshot=d/('controller-'+p.name)
                if snapshot.exists():need(sha(snapshot)==h,'controller snapshot '+str(p))
                else:need(sha(p)==h,'historical owned input without snapshot '+str(p))
            else:need(sha(p)==h,'unchanged input '+str(p))
        c=classify(row['label'],r,d)
        ledger.append({**row,'classification':c,'selected_final_evidence':row['label'] in ['tag-provider-final','tag-off-delivery','false-frozen-v2','invalid-frozen-v2','full-delivery','full-helpers-frozen','lookup-import','bool-actual-retained','bool-mirror-retained','foreign-inspect','compiler-version','native-eq-expansion'],'stdout_sha256':sha(d/'stdout'),'stderr_sha256':sha(d/'stderr'),'freeze_sha256':sha(d/'freeze.json'),'product_count':len(r['products'])})
    maincase=ordinary(paths['tag-provider-final'],'tag-provider-final',(19,0))
    off=ordinary(paths['tag-off-delivery'],'tag-off-delivery',(19,0))
    neg=ordinary(paths['false-frozen-v2'],'false-frozen-v2',(19,1),'component::false_relation')
    err=(paths['false-frozen-v2']/'stderr').read_text();need('postcondition not satisfied' in err and not any(s in err for s in ['precondition not satisfied','possible arithmetic','error[E']),'healthy false relation')
    inv=ordinary(paths['invalid-frozen-v2'],'invalid-frozen-v2',(19,1),'component::invalid_zero_low');need('precondition not satisfied' in (paths['invalid-frozen-v2']/'stderr').read_text(),'invalid precondition diagnostic')
    full=ordinary(paths['full-delivery'],'full-delivery',(36,1),'component::Item::new')
    err=(paths['full-delivery']/'stderr').read_text();need('debug_assert!(new.protected() == protected)' in err and 'debug_assert!(new.tag() == tag)' not in err and 'debug_assert!(new.perm() == perm)' not in err,'protector-only gap')
    support=ordinary(paths['full-helpers-frozen'],'full-helpers-frozen',(36,0))
    # Whole-run failure does not grant the native report successful certification.
    need(neg['native_scope']['complete_selected_scope_verified'] is False,'negative scope certification')
    imp=paths['lookup-import'];data=json.loads((imp/'stdout').read_text());im=inventory(imp,'client');vr=data['verification-results']
    need(vr['success'] and vr['verified']==1 and vr['errors']==0 and vr['is-verifying-entire-crate'],'lookup import result')
    need(set(im['local_function_details'])=={'client::read'} and {k for k,v in im['ordinary_original_functions'].items() if v['mode']=='Exec' and v['body_present']}=={'client::read'},'lookup import exact body set')
    need(data['verus-infer']['functions']==[] and not data['verus-infer']['interface_written'],'lookup import helper/publication')
    text=(imp/'vir/crate.vir').read_text();closed=json.loads((imp/'plan.json').read_text())['imported_closed_bodies_must_be_absent'];seen={}
    for m in re.finditer(r'\(Function\s',text):
        f=text[m.start():end(text,m.start())];path=field(f,'name')[len('(Fun :path '):-1]
        if path in closed:seen[path]=field(f,'body')=='None'
    need(all(seen.values()),'closed imported body unexpectedly present')
    seen={name:('signature_present_body_absent' if name in seen else 'entire_function_absent_after_pruning') for name in closed}
    im['closed_imported_bodies_absent']=seen;im['path']=str(imp);im['receipt_sha256']=sha(imp/'receipt.json')
    bools={}
    for name,count in [('actual',0),('mirror',1)]:
        d=paths['bool-'+name+'-retained'];data=json.loads((d/'stdout').read_text());v=data['verification-results']
        need(v['verified']==count and v['errors']==(1-count) and v['is-verifying-entire-crate'] and not v['encountered-vir-error'],'bool result '+name)
        need(data['verus-infer']['functions']==[],'bool helper')
        bools[name]=dict(path=str(d),verification=v,receipt_sha256=sha(d/'receipt.json'),foreign_bridge_proved=False)
    d=paths['foreign-inspect'];out=(d/'stdout').read_text();need('EXACT_PRIMITIVE_SHAPE true' in out and 'CRATE core' in out and 'fn(bool) -> u64' in out,'actual foreign extraction')
    version=(paths['compiler-version']/'stdout').read_text();need('58cf8f9c35b355199727627d07ae34a4e2846d67' in version,'compiler commit')
    expansion=(paths['native-eq-expansion']/'stdout').read_text();start=expansion.index('{',expansion.index('fn eq('));body=expansion[start:expansion.index('}',start)+1];need(body=='{ self.0 == other.0 }','original derived Eq')
    need(sha('/tmp/native-miri-tag-delivery-wwm_pq9c/receipt.json')=='855f7ca4dae41e7ac7df00fb543d4ba96f1cf2dad8f6ce9c8ad3344520ec3721','tag delivery')
    need(sha(e.ROOT/'evidence/nonzero-constructor-combined-review/RESULTS.json')=='221d4846448770b63177ef0ab5a1f07be14ee661e78a73c4881ad178bc7e111b','combined review results')
    need(sha('/tmp/combined-review-work-s_wrxbdr/probes-v3/compose/component.rs')=='0e6129159dc9830244ec5d7acf206b7294f58e3227808e6fcc2f6023e8f54703','completed composition source')
    status=__import__('subprocess').check_output(['git','status','--porcelain','--untracked-files=all'],cwd=e.ROOT).decode().splitlines()
    need(all(x[3:].startswith('source/tools/research/item_tag_constructor_composition/') for x in status),'edit ownership')
    result=dict(decision='scoped_Item_tag_component_proof',engine='3aa7f49a04a5e93c2ede882f2b947aac6e39263d',docs_base='89deb6a7',bundle_receipt_sha256=e.RECEIPT,tools=bundle['tools'],versions={'schema':38,'policy':69,'cache':72,'matching_vstd':'2250/0'},tag=maincase,tag_off=off,false_relation=neg,invalid_representation=inv,full_constructor=full,constructor_helpers=support,lookup_import=im,bool_conversion=bools,foreign_mir=dict(path=str(d),receipt_sha256=sha(d/'receipt.json'),source_sha256=sha(e.TC/'lib/rustlib/src/rust/library/core/src/convert/num.rs'),primitive_shape=True,foreign_bridge_proved=False),native_eq_body_sha256=hashlib.sha256(body.encode()).hexdigest(),attempt_ledger=ledger,all_receipt_product_and_final_guards_passed=True,supplemental_retention=retention,controllers={p.name:sha(p) for p in e.HERE.glob('*.py')},publication=False,native_correspondences=0,goal_complete=False,full_Item_new='UNPROVED_bool_library_source_bridge',whole_goal='PENDING',HMAC='1/1_UNPROVED')
    e.write(e.HERE/'RESULTS.json',result)
    print(json.dumps({'attempts':len(ledger),'tag':'19/0 on/off','full':'36/1 UNPROVED','helpers':'36/0','import':'1/0','guards':True}))
if __name__=='__main__':main()
