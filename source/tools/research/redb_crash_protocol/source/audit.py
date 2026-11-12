#!/usr/bin/env python3
"""Audit exact input fields/body fidelity and the final result denominator."""
import hashlib, json, re, sys
sys.dont_write_bytecode=True
from pathlib import Path
from probes import item, H, supported, publication
from freeze import HERE

def sha_bytes(b):return hashlib.sha256(b).hexdigest()
def body(text):return text[text.index('{'):]
def strip_visibility(s):return re.sub(r'\bpub(?:\([^)]*\))?\s+', '',s)
def audit(out):
    frozen=json.loads((HERE/'FROZEN.json').read_text())
    hashes=dict(frozen['source_sha256']);hashes.update(json.loads((HERE/'DEPENDENCIES02.json').read_text())['additional_source_sha256'])
    for n,h in hashes.items():assert sha_bytes((HERE/'original'/n).read_bytes())==h,n
    for i in frozen['items']:
        data=(HERE/'original'/i['file']).read_bytes()
        start=sum(len(s) for s in data.splitlines(keepends=True)[:i['start_line']-1])
        assert sha_bytes(data[start:start+i['bytes']])==i['sha256'],i
    text=(out/'supported.rs').read_text()
    assert text==supported(),'generated provider differs'
    body_inventory={}
    for name in ['primary_slot','secondary_slot','published_slot','swap_primary_slot','write_secondary_slot']:
        b=body(item(H,'fn',name));assert text.count(b)==1,name
        body_inventory['DatabaseHeader::'+name]=dict(body_sha256=sha_bytes(b.encode()),full_original_body=True)
    field_inventory={}
    for rel,name in [('src/tree_store/page_store/base.rs','PageNumber'),('src/transaction_tracker.rs','TransactionId'),
        ('src/tree_store/btree_base.rs','BtreeHeader'),(H,'TransactionHeader'),(H,'DatabaseHeader')]:
        native=strip_visibility(item(rel,'struct',name))
        m=re.search(r'(?m)^pub struct '+name+r'\b',text);assert m,name
        if name=='TransactionId': actual=text[m.start():text.index(';',m.start())+1]
        else:
            from freeze import span
            actual=text[m.start():span(text,m.start())]
        assert native==strip_visibility(actual),name
        field_inventory[name]=dict(visibility_erased_original_sha256=sha_bytes(native.encode()),field_and_type_changes=False)
    assert not re.search(r'\b(assume|admit|external_body|external_type_specification|axiom)\b',text),'new trust marker'
    r=json.loads((out/'receipt.json').read_text());assert r['success']
    assert len(r['outcomes'])==22,len(r['outcomes'])
    return dict(stage='PARTIAL_SOURCE_BOUNDARY_NOT_CRASH_THEOREM',source_coupling_proved=False,
        body_inventory=body_inventory,field_inventory=field_inventory,
        prior_mode_cases=[False,True],initial_application_cases_completed=0,initial_application_cases_total=2,
        CP1_requested_two_phase_previous_modes_completed=0,CP1_previous_modes_total=2,
        CP2_coherent_non_vacuous_recovery_proved=False,detected_corruption_roundtrip_proved=False,
        typed_minimum_need='CheckedBackend::sync_data actual error types are ineligible; successful CachedFile::flush page coverage before successful backend sync remains the smallest effect obligation, with candidate-page/retention linkage separate.',
        checked_artifacts={n:sha_bytes((out/n).read_bytes()) for n in ['supported_ordinary.vir','supported_infer.vir','lib_supported_ordinary.rlib','lib_supported_infer.rlib']},
        accepted_runtime=frozen['runtime'],validated_receipt_sha256=sha_bytes((out/'receipt.json').read_bytes()),
        infer_helpers=[],infer_annotation_reduction=0,HMAC_native='DEFERRED AND UNACHIEVED; native0/44+3/101 unchanged')
if __name__=='__main__':
    out=Path(sys.argv[1]);print(json.dumps(audit(out),indent=2))
