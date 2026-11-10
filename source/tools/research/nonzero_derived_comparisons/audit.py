"""Audit only explicitly named completed epochs; never search neighboring outputs."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
from trial import ROOT,authenticate
sys.path.insert(0,str(HERE.parents[1]))
from infer_eval_support import digest
from infer_experiment import require,require_exact

FINAL={
 'build':'/tmp/nonzero-derive-build-mmnfzmog',
 'vstd':'/tmp/nonzero-owned-vstd-2jzgualu',
 'bundle':'/tmp/nonzero-derive-frozen-tools-8hnei2sw',
 'macro':'/tmp/nonzero-owned-macro-4okwhwsz',
 'off':'/tmp/nonzero-owned-macro-2v1jk5mu',
 'width':'/tmp/nonzero-owned-width-i9b_eutu',
 'false':'/tmp/nonzero-owned-false-3d2f3nda',
 'weak':'/tmp/nonzero-owned-weak-1votr_u0',
 'wrong_model':'/tmp/nonzero-owned-wrong_model-jvt_y4vu',
 'unsupported':'/tmp/nonzero-owned-unsupported-zy9cov17',
 'order':'/tmp/nonzero-owned-order-z94tcgkv',
 'standard_order':'/tmp/nonzero-owned-standard_order-thitm5nh',
 'import':'/tmp/nonzero-derive-import-1qa6phni',
 'tests':'/tmp/nonzero-derive-tests-ycbn7tvv',
 'body':'/tmp/nonzero-derive-body-ny_0y7oi',
}
PRODUCTION=['source/builtin_macros/src/infer_derive_ops.rs','source/vir/src/infer.rs',
 'source/vir/src/infer/newtypes.rs','source/vir/src/infer/registry.rs',
 'source/vir/src/infer/wrapper_binary.rs','source/vir/src/infer/wrapper_compare.rs']

def read(name):return json.loads((Path(FINAL[name])/'receipt.json').read_bytes())
def data(name,label='check'):return json.loads((Path(FINAL[name])/(label+'.stdout')).read_text())
def positive(d,helper,execs):
 v=d['verification-results'];i=d['verus-infer'];scope=i['verification_scope']
 require(v['success'] is True and v['errors']==0 and v['is-verifying-entire-crate'] is True,'not full positive')
 require(i['crate_success'] is True and i['failed_functions']==[],'failed function')
 require([f['function'] for f in i['functions']]==[helper],'helper identity mismatch')
 require_exact(i['functions'])
 require(scope['native_collection_completed'] is True and scope['complete_selected_scope_verified'] is True,'scope incomplete')
 selected=[b for b in scope['native_functions'] if b['selected'] and b['body_status'] not in ['specification_definition_or_declaration','selected_item_without_translated_body']]
 require(len(selected)==len(execs) and {b['rust_function'] for b in selected}==set(execs),'body identity mismatch')
 require(all(b['body_status']=='checked_exec_body' for b in selected),'unchecked body')
 return i

def main():
 authenticate()
 ledger=json.loads((HERE/'epochs.json').read_bytes());rows=[]
 for entry in ledger:
    p=Path(entry['output']);rp=p/'receipt.json'
    require(digest(rp)==entry['receipt_sha256'],'receipt drift: '+str(p))
    r=json.loads(rp.read_bytes())
    if 'freeze_sha256' in r:require(digest(p/'input-freeze.json')==r['freeze_sha256'],'freeze drift')
    for name,h in r.get('products',{}).items():
        # Compiler builds retain their durable products in frozen tool bundles;
        # overwritten intermediate mutable build targets are diagnostic only.
        if Path(name).is_absolute() or name.startswith('target/') or name.startswith('retained_inputs/'):continue
        require(digest(p/name)==h,'private product drift: '+str(p/name))
    if 'tools' in r:
        for name,h in r['tools'].items():require(digest(p/name)==h,'frozen tool drift')
    rows.append(dict(output=str(p),receipt_sha256=entry['receipt_sha256'],
                     controller_success=r['success'],terminal_exit=r.get('terminal_exit'),
                     verification=r.get('verification'),failure=r.get('failure')))
 for name in FINAL:
    r=read(name);require(r['success'] is True and r['final_guards_passed'] is True,'final controller/guard failed: '+name)
 build_freeze=json.loads((Path(FINAL['build'])/'input-freeze.json').read_bytes())
 for name in PRODUCTION:
    require(digest(ROOT/name)==build_freeze['inputs'][str(ROOT/name)]['sha256'],'final compiler source drift')
 execs={'<BorTag as std::clone::Clone>::clone','<BorTag as std::cmp::PartialEq>::eq','compare_equal'}
 on=positive(data('macro'),'component::BorTag::eq',execs)
 warm=positive(data('macro','warm'),'component::BorTag::eq',execs)
 off=positive(data('off'),'component::BorTag::eq',execs)
 width=positive(data('width'),'component::BorTag::eq',execs)
 require(on['interface_written'] is True and warm['interface_written'] is True,'export absent')
 require(on['functions'][0]['metrics']['cache']=='miss' and warm['functions'][0]['metrics']['cache']=='hit','not cold/warm')
 require(off['interface_written'] is False and width['interface_written'] is False,'unexpected export')
 imported=positive(data('import'),'client::equal',{'equal','goal'})
 require(imported['imported_inference_dependencies'] and imported['interface_written'] is False,'import metadata absent')
 false=data('false');fi=false['verus-infer'];fv=false['verification-results']
 require(fv['is-verifying-entire-crate'] is True and fv['verified']==4 and fv['errors']==1,'not singleton semantic negative')
 require_exact(fi['functions']);require([f['function'] for f in fi['functions']]==['component::BorTag::eq'],'negative helper set')
 require(fi['failed_functions']==['component::compare_equal'] and fi['interface_written'] is False,'wrong failed function/publication')
 for name in ['weak','wrong_model']:
    d=data(name);i=d['verus-infer'];v=d['verification-results'];f=i['functions'][0]
    require(v['is-verifying-entire-crate'] is True and v['errors']==2 and i['failed_functions']==['component::BorTag::eq'],'not derive-body/model rejection')
    require(f['required_checks_passed'] is False and f['proofs']['soundness']['status']=='not_proved' and f['proofs']['exactness']['status']=='not_proved' and f['proofs']['maximality']['status']=='proved','bad model obligation state')
    require(i['interface_written'] is False,'failed model published')
 for name in ['unsupported','order']:
    require(read(name)['terminal_exit']!=0 and data(name)['verus-infer']['functions']==[],'unsupported profile accepted')
    require(('infer_derive' if name=='unsupported' else 'support only PartialEq') in (Path(FINAL[name])/'check.stderr').read_text(),'missing shape diagnostic')
 so=data('standard_order');require(so['verification-results']['errors']==1 and so['verus-infer']['functions']==[],'standard order gap changed')
 require('postcondition not satisfied' in (Path(FINAL['standard_order'])/'check.stderr').read_text(),'not ordinary order gap')
 testlog=(Path(FINAL['tests'])/'tests.stdout').read_text()
 require('test result: ok. 7 passed' in testlog and 'test result: ok. 3 passed' in testlog,'focused suite not green')
 require('verification results:: 2226 verified, 0 errors' in (Path(FINAL['vstd'])/'check.stdout').read_text(),'matching vstd failed')
 body=read('body');require(body['rust_generated_eq_body']=='{ self.0 == other.0 }','generated body differs')
 result=dict(schema=1,classification='checked_original_PartialEq_only',
    final=FINAL,epochs=rows,actual_helper=warm['functions'][0],native_scope=warm['verification_scope'],
    imported_helpers=imported['functions'],body_identity={k:v for k,v in body.items() if k.endswith('sha256') or k=='rust_generated_eq_body'},
    production_source_hashes={n:digest(ROOT/n) for n in PRODUCTION},
    equality_full_nonzero_u64_domain=True,ordering='UNPROVED',
    native_correspondences=0,publication=False,goal_complete=False)
 (HERE/'RESULTS.json').write_text(json.dumps(result,indent=2)+'\n')
 print('audited',len(rows),'completed epochs; final equality 5/0, import 4/0; tests 10/0; ordering UNPROVED')
if __name__=='__main__':main()
