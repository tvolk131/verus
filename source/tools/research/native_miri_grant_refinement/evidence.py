"""Compact final evidence from terminal frozen epochs; no proof runner is restarted."""
import argparse, collections, json, pathlib, re
from replay_support import *
from fidelity import strict_check

def save(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def receipt(path):
    p=pathlib.Path(path)/'receipt.json';r=json.loads(p.read_text());require(r['success'] is True,'controller did not finish: '+str(p))
    require(all(type(c.get('returncode')) is int and not c.get('timed_out') and not c.get('interrupted') for c in r['commands']),'nonterminal command')
    return p,r

def summary(p,r):
    result={'path':str(p),'sha256':digest(p),'commands':r['commands']}
    for key in ['verification_results','assistance_summary','observation','build_exits','run_exits','outputs_equal','compile_returncode','check_returncode','products_sha256']:
        if key in r:result[key]=r[key]
    logs={}
    for q in p.parent.iterdir():
        if q.suffix in ['.stdout','.stderr']:logs[q.name]=digest(q)
    result['logs_sha256']=logs
    if 'stdout_sha256' in r:require(r['stdout_sha256']==logs['verify.stdout'] and r['stderr_sha256']==logs['verify.stderr'])
    if 'products_sha256' in r:
        for name,sha in r['products_sha256'].items():require(digest(p.parent/name)==sha,'product drift')
    for k in ['artifacts','inventory_inputs']:
        if k not in r:continue
        after=r[k+'_after'] if k=='inventory_inputs' else r['artifacts_after']
        require(r[k]==after,'epoch guard drift')
        for name,entry in after.items():require(digest(name)==entry['sha256'],'final input guard drift: '+name)
        result[k+'_final_matched']=len(after)
    return result

def main():
    a=argparse.ArgumentParser();a.add_argument('suite',type=pathlib.Path);a.add_argument('mutations',type=pathlib.Path);a.add_argument('coq',type=pathlib.Path);args=a.parse_args()
    strict_check(HERE/'component.rs');p,s=receipt(args.suite);results={'schema':1,'classification':'component_evidence_for_independent_review','suite':summary(p,s),'epochs':{}}
    for name,path in s['epochs'].items():
        q,r=receipt(path);results['epochs'][name]=summary(q,r)
        require(digest(q.parent/'component.rs')==r['input']['sha256'],'frozen trial source drift')
    q,r=receipt(s['native_compare']);results['native_macro_erasure_support']=summary(q,r)
    require(r['build_exits']==[0,0] and r['run_exits']==[0,0] and r['outputs_equal'] is True)
    q,r=receipt(args.mutations);results['executable_mutations']=summary(q,r)
    results['executable_mutations']['rows']={n:{k:v for k,v in row.items() if k!='scope'} for n,row in r['mutations'].items()}
    q,c=receipt(args.coq);results['coq']=summary(q,c)
    require(c['compile_returncode']==0 and c['check_returncode']==0)
    glob=(q.parent/'GrantBridge.glob').read_text();names=re.findall(r'^(def|prf|ind|constr|proj) \d+:\d+ <> ([^\s]+)',glob,re.M)
    require(len(names)==7 and {n for _,n in names}==set(c['declarations']),'compiler source declaration mismatch')
    require(not any(k in ['ind','constr','proj'] for k,n in names),'unexpected own constructors/projections')
    inventory={'scope':'All authored definitions/theorems; no own inductives, constructors, projections, generated declarations or assumed helper callbacks. Full unabridged elaborated proof terms are the authoritative helper-expression inventory, including lia-generated terms; the recursive kernel closure is a superset, not an exact used-helper set.',
               'coq':c['declarations'],'compiler_glob_declarations':names,
               'recursive_kernel_closure':{'libraries':c['kernel_libraries'],'constants_count':len(c['kernel_constants']),'inductive_blocks_count':len(c['kernel_inductive_blocks']),'raw_inventory_receipt':str(q),'receipt_sha256':digest(q)},
               'verus':{},'source_fidelity':strict_check(HERE/'component.rs')}
    for name in ['component-on','component-off']:
        _,v=receipt(s['epochs'][name]);inventory['verus'][name]=v['scope']
    for entry in c['declarations'].values():
        require(entry['assumptions_text'].strip()=='Closed under the global context')
        for kind in ['type','term','assumptions']:require(digest(entry[kind+'_path'])==entry[kind+'_sha256'])
        require('...' not in pathlib.Path(entry['term_path']).read_text(),'abridged proof term')
    invpath=private_root()/'complete-inventory.json';save(invpath,inventory)
    # Compact committed inventory retains exact local body rows and closed types;
    # full raw helper terms and 41k inherited kernel names remain private.
    compact={k:v for k,v in inventory.items() if k!='recursive_kernel_closure'}
    compact['recursive_kernel_closure']={k:v for k,v in inventory['recursive_kernel_closure'].items() if k!='libraries'}
    compact['recursive_kernel_closure']['libraries_count']=len(c['kernel_libraries'])
    compact['private_complete_inventory']={'path':str(invpath),'sha256':digest(invpath)}
    save(HERE/'INVENTORY.json',compact)
    results['inventory']={'path':str(HERE/'INVENTORY.json'),'sha256':digest(HERE/'INVENTORY.json'),'private_path':str(invpath),'private_sha256':digest(invpath)}
    results.update(native_correspondences=0,minimal_operations_open=44,phase_obligations_open=3,writer_effects_open=101,publication=False,goal_complete=False,independent_acceptance=False,
                   canonical_inherited='6/0; exact full domain unchanged, not rerun',HMAC_inherited='1/1 UNPROVED; arbitrary initialized [u8;128], every valid key length including 0 and >128; XOR prefix and unchanged suffix',
                   parent_permission_grants='User reports separate immutable 8aa9fa72; not read/imported/rebuilt; independent review pending')
    results['own_attempt_catalog']=[]
    for q in sorted(private_root().glob('*/receipt.json')):
        r=json.loads(q.read_text())
        results['own_attempt_catalog'].append({'path':str(q),'sha256':digest(q),'controller_success':r.get('success'),'input':r.get('input'), 'commands':[{k:c[k] for k in ['label','returncode'] if k in c} for c in r['commands']],
            'current_evidence':str(q) in {results['suite']['path'],results['coq']['path'],results['executable_mutations']['path'],results['native_macro_erasure_support']['path']}|{x['path'] for x in results['epochs'].values()}})
    results['recovery_manifest_sha256']=digest(HERE/'RECOVERY.json')
    save(HERE/'RESULTS.json',results)
    print(json.dumps({'results':str(HERE/'RESULTS.json'),'coq_libraries':len(c['kernel_libraries']),'coq_constants':len(c['kernel_constants']),'guarded_coq_inputs':len(c['artifacts']),'suite':str(p)},indent=2))
if __name__=='__main__':main()
