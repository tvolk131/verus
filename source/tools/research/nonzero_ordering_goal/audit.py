"""Exception-based acceptance of retained baseline receipts; also run under -O -B."""
from run import *
import re

def original_bodies(text):
    out = {}
    for name in ['clone','hash','eq','assert_fields_are_eq','partial_cmp','cmp']:
        match = re.search(r'\bfn '+name+r'(?:<[^>]*>)?\(', text)
        need(match is not None,'missing original generated method '+name)
        start = text.index('{',match.start());depth=0
        for i in range(start,len(text)):
            depth += (text[i]=='{')-(text[i]=='}')
            if depth==0:
                body=text[start:i+1];break
        out[name] = dict(sha256=hashlib.sha256(body.encode()).hexdigest(),bytes=len(body.encode()))
    return out

def audit(out):
    authenticate()
    rr=json.loads((out/'receipt.json').read_text());results=rr['results'];summary={}
    for case,row in results.items():
        d=out/case
        need(row==json.loads((d/'receipt.json').read_text()),'case receipt differs')
        need(sha(d/'freeze.json')==row['freeze_sha256'],'freeze drift')
        f=json.loads((d/'freeze.json').read_text())
        for p,h in f['inputs'].items():need(sha(p)==h,'input drift '+p)
        for n in ['stdout','stderr']:need(sha(d/n)==row[n+'_sha256'],'output drift '+case)
        inf=row.get('inference',{});vr=row.get('verification',{})
        summary[case]=dict(exit=row['exit'],verification=vr,receipt_sha256=sha(d/'receipt.json'),
            interface_written=inf.get('interface_written'), failed_functions=inf.get('failed_functions'),
            helpers=inf.get('functions',[]),native_functions=[{k:x[k] for k in ['rust_function','selected','body_status','inference_candidate']} for x in inf.get('verification_scope',{}).get('native_functions',[]) if x['automatically_derived'] or x['body_status'] not in ['specification_definition_or_declaration']])
        if case in ['carrier_partial','eq','eq_warm','unrelated_eq','unrelated_eq_warm']:
            need(row['exit']==0 and vr['success'] and vr['errors']==0 and vr['is-verifying-entire-crate'],'positive failed')
            need(inf['verification_scope']['complete_selected_scope_verified'],'incomplete selected scope')
        if case in ['eq','eq_warm','unrelated_eq','unrelated_eq_warm']:
            fs=inf['functions'];need(len(fs)==1 and fs[0]['mode']=='exact','one exact Eq helper')
            need(fs[0]['contract']['requires']=='true','restricted Eq domain')
            need(set(fs[0]['proofs'])=={'soundness','maximality','exactness'},'missing exact proof')
            need(all(p['required'] and p['status']=='proved' for p in fs[0]['proofs'].values()),'fresh exact proof missing')
            need(fs[0]['metrics']['cache']==('hit' if case.endswith('warm') else 'miss'),'cache phase differs')
        if case in ['carrier_ord','original_calls','request','ordinary_derive']:
            need(row['exit']!=0 and not inf['interface_written'],'negative published')
            need(not (out/(case+'.vir')).exists() and not (out/('lib'+case+'.rlib')).exists(),'failed output retained')
            need(not inf['functions'],'ordering inference unexpectedly succeeded')
    need(results['carrier_partial']['verification']['verified']==5,'carrier partial count')
    need(results['carrier_ord']['verification']['errors']==1,'carrier Ord diagnostic')
    need(results['carrier_ord']['inference']['failed_functions']==['carrier_ord::total'],'carrier Ord wrong goal')
    need('postcondition not satisfied' in (out/'carrier_ord/stderr').read_text(),'carrier Ord not semantic failure')
    need(results['original_calls']['verification']['errors']==6,'original call count')
    need('currently support only PartialEq' in (out/'request/stderr').read_text(),'macro barrier changed')
    need(results['ordinary_derive']['verification']['encountered-vir-error'],'derive policy gate changed')
    native=summary['ordinary_derive']['native_functions']
    need(all(x['body_status']!='checked_exec_body' for x in native),'gate failure counted checked')
    text=(out/'expanded/stdout').read_text()
    need(DECL in (out/'native.rs').read_text(),'native declaration changed')
    need('pub struct BorTag(NonZero<u64>);' in text,'expanded carrier changed')
    need('::core::option::Option::Some(::core::cmp::Ord::cmp(self, other))' in text,'partial derive changed')
    need('::core::cmp::Ord::cmp(&self.0, &other.0)' in text,'Ord derive changed')
    result=dict(schema=1,private_root=str(out),receipt_sha256=sha(out/'receipt.json'),
       bundle_receipt_sha256=BUNDLE_SHA,declaration_sha256=DECL_SHA,
       generated_bodies=original_bodies(text),native_mir_sha256=sha(out/'native.mir'),
       cases=summary,engine_changed=False,versions=dict(schema=39,policy=72,cache=75),
       blocker='Original BorTag Ord delegates to core NonZero<u64> Ord; no existing OrdSpecImpl links its result to the NonZero view. Macro and typed derive profile also reject ordering. No ordering proof claimed.',
       native_correspondences=0,publication=False,goal_complete=False)
    return result

if __name__=='__main__':
    result=audit(Path(sys.argv[1]));write(HERE/'BASELINE.json',result)
    print('baseline gates passed; ordering remains unproved')
