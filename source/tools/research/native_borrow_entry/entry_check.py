"""Typed local reborrow eligibility, not a source-to-ownership proof checker."""
from copy import deepcopy
import hashlib, json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from infer_experiment import require

def inspect(d, source_sha256):
    require(d['schema']==1 and d['source_sha256']==source_sha256,'source_binding')
    require(d['normal_analysis_completed'] is True,'unchecked_Rust_body')
    require(d['native_bridge_proved'] is False and d['publication'] is False,'forged_publication')
    require(d['arg_count']==2 and d['input_shapes']==[
        dict(kind='array_borrow',exclusive=True,element='u8',length=128,size=128,align=1),
        dict(kind='slice_borrow',exclusive=False,element='u8')], 'input_domain_or_layout')
    entry=d['blocks'][0]
    require(entry['bb']==0 and entry['cleanup'] is False,'entry_block')
    active=[s for s in entry['statements'] if s['ir']['op'] not in ('storage_live','storage_dead')]
    require(len(active)==2,'unsupported_entry_effect')
    ref,unsize=[s['ir'] for s in active]
    arg=dict(local=1,projection=[dict(op='deref')],ty='[u8; 128]')
    require(ref['op']=='assign' and ref['value']['op']=='ref' and ref['value']['mutable'] is True
        and ref['value']['fake'] is False and ref['value']['place']==arg,'original_array_reborrow')
    rb=ref['destination']; require(rb['projection']==[] and rb['ty']=='&mut [u8; 128]','reborrow_type')
    require(unsize['op']=='assign' and unsize['value']==dict(op='unsize',operand=dict(op='move',place=rb),to='&mut [u8]'),'unsize_input')
    us=unsize['destination'];require(us['projection']==[] and us['ty']=='&mut [u8]','unsize_output')
    term=entry['terminator']
    require(term['op']=='call' and term['arguments']==[dict(op='move',place=us)]
        and term['destination']['ty']=="std::slice::IterMut<'_, u8>", 'constructor_operand')
    require(entry['call']['definition']=='core::slice::<impl [T]>::iter_mut','constructor_identity')
    candidates=[b for b in d['loans'] if b['borrowed']==arg and b['assigned']==rb and b['reserve']==active[0]['location']]
    require(len(candidates)==1 and candidates[0]['kind']=='exclusive','borrowck_loan_witness')
    loan=candidates[0]
    require(any(r==loan['region'] and l==loan['loan'] for r,l,p in d['borrowck_facts']['loan_issued_at']),'borrowck_issue_fact')
    # Whole local writer effect guard, not a proof of the loop or native callees.
    stores=[s['ir'] for b in d['blocks'] for s in b['statements']
        if s['ir']['op']=='assign' and s['ir']['destination']['projection']]
    require(len(stores)==1 and stores[0]['value'].get('op')=='binary' and stores[0]['value']['kind']=='BitXor','unexpected_local_write')
    calls=[b['call']['definition'] for b in d['blocks'] if b['call'] is not None]
    require(calls==['core::slice::<impl [T]>::iter_mut','std::iter::Iterator::zip','std::iter::IntoIterator::into_iter','std::iter::Iterator::next'],'unexamined_local_call')
    return dict(typed_entry_eligibility=True, original_array_local=1,extent=128,
        reborrow_local=rb['local'],slice_local=us['local'],iterator_local=term['destination']['local'],
        loan=loan, conditional_heap_entry_premise_verified=False, native_bridge_proved=False,publication=False,
        missing=['Rust-reference-validity-to-explicit-heap-refinement',
            'actual-input-value-and-future-array-interpretation',
            'native-constructor-and-getter-ownership-refinement',
            'retag-provenance-alias-and-unwind-semantics'])

def validate_publication(evidence, proof):
    require(proof.get('status')=='proved' and proof.get('complete') is True,'failed_incomplete_resource_proof')
    require(proof.get('source_sha256')==evidence['source_sha256'],'proof_source_binding')
    # No data, including a forged success receipt, is a checked refinement theorem.
    raise ValueError('source_to_ownership_refinement_checker_missing')

def controls(d):
    results=[]
    def reject(name,mutate,reason):
        x=deepcopy(d);mutate(x)
        try: inspect(x,x['source_sha256'])
        except ValueError as e:
            require(str(e)==reason,name+': wrong rejection '+str(e))
            results.append(dict(case=name,reason=reason,semantic_negative=False))
        else: raise ValueError('false eligibility accepted: '+name)
    reject('forged-borrow-witness',lambda x:x['loans'][0].update(kind='shared'),'borrowck_loan_witness')
    reject('wrong-borrow-origin',lambda x:x['loans'][0]['borrowed'].update(local=2),'borrowck_loan_witness')
    reject('missing-issue-fact',lambda x:x['borrowck_facts'].update(loan_issued_at=[]),'borrowck_issue_fact')
    reject('wrong-layout',lambda x:x['input_shapes'][0].update(size=127),'input_domain_or_layout')
    reject('unchecked-body',lambda x:x.update(normal_analysis_completed=False),'unchecked_Rust_body')
    reject('forged-publication',lambda x:x.update(publication=True),'forged_publication')
    for status,complete in [('failed',True),('incomplete',False),('timeout',False),('resource_limit',False),('proved',False),('proved',True)]:
        proof=dict(status=status,complete=complete,source_sha256=d['source_sha256'])
        try:validate_publication(d,proof)
        except ValueError as e:results.append(dict(case='publication-'+status+'-'+str(complete),reason=str(e),semantic_negative=False))
        else:raise ValueError('publication barrier bypass')
    return results
