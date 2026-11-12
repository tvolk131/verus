#!/usr/bin/env python3
"""Fidelity and receipt audit; no new verification or source assumptions."""
import json, re, sys
sys.dont_write_bytecode=True
from pathlib import Path
from fixtures import HERE, p, baseline, contracted, eligible, closeout_cases, NOMINAL
import importlib.util
_spec=importlib.util.spec_from_file_location('eligibility_replay',HERE/'replay.py')
_replay=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(_replay)
EXEC,scope=_replay.EXEC,_replay.scope
from infer_eval_support import digest
from infer_experiment import require, validate_run

def sha(text):
    import hashlib
    return hashlib.sha256(text.encode()).hexdigest()
def erase_visibility(text):return re.sub(r'\bpub(?:\([^)]*\))?\s+', '',text)
def audit(out):
    frozen=json.loads((HERE/'FROZEN.json').read_text())
    for n,h in frozen['inputs'].items():require(digest(HERE.parent/n)==h,n)
    require((out/'eligible.rs').read_text()==eligible(),'eligible regeneration')
    body_inventory={}
    for name,expected in frozen['bodies'].items():
        original=p.item(p.H,'fn',name);body=original[original.index('{'):]
        require(sha(body)==expected['body_sha256'],name+' frozen body')
        for fixture in closeout_cases().values():require(fixture.count(body)==1,name+' full body')
        require(contracted().count(body)==1,name+' proposed full body')
        body_inventory[name]=dict(**expected,complete_original_body=True)
    field_inventory={}
    from freeze import span
    for rel,name in [('src/tree_store/page_store/base.rs','PageNumber'),('src/transaction_tracker.rs','TransactionId'),
        ('src/tree_store/btree_base.rs','BtreeHeader'),(p.H,'TransactionHeader'),(p.H,'DatabaseHeader'),(p.H,'UnrepairedDatabaseHeader')]:
        original=erase_visibility(p.item(rel,'struct',name));text=eligible()
        m=re.search(r'(?m)^pub struct '+name+r'\b',text);require(m is not None,name)
        end=text.index(';',m.start())+1 if name=='TransactionId' else span(text,m.start())
        actual=erase_visibility(text[m.start():end]);require(actual==original,name+' fields')
        field_inventory[name]=dict(visibility_erased_sha256=sha(original),fields_and_types_preserved=True)
    error=p.item('src/error.rs','enum','StorageError');require(eligible().count(error)==1,'actual error variants')
    text=eligible();require(text.count(NOMINAL)==1,'nominal freeze')
    require(text.count('#[verifier::external_type_specification]')==2 and text.count('#[verifier::external_body]')==2,'only nominal attributes')
    require(not re.search(r'\b(assume|admit|assume_specification|external_fn_specification|axiom|uninterp)\b',text),'new behavioral trust')
    r=json.loads((out/'receipt.json').read_text());require(r['success'] and r['guard_passed'],'receipt success')
    require(len(r['outcomes'])==14,'closeout denominator')
    inventory={}
    for o in r['outcomes']:
        label=o['label'];err=(out/(label+'.stderr')).read_text();data=o['data']
        require(digest(out/(label+'.stderr'))==o['stderr_sha256'],'diagnostic hash')
        if '_client_' not in label:
            case=label.rsplit('_',1)[0]
            positive=case=='eligible'
            validate_run(dict(returncode=o['returncode'],stdout=json.dumps(data),stderr=err),positive)
            scope(data,positive,EXEC)
            if not positive:
                kind='postcondition not satisfied' if case=='finalize_wrong' else 'precondition not satisfied'
                require(kind in err and data['verification-results']['errors']==1,'strict negative')
            if label.endswith('_infer'):
                inf=data['verus-infer'];require(inf['interface_written']==positive and o['export_exists']==positive,'Infer publication')
        elif not label.endswith('failed_import'):
            positive=label.endswith('healthy')
            validate_run(dict(returncode=o['returncode'],stdout=json.dumps(data),stderr=err),positive)
            scope(data,positive,{'imported_finalize','imported_selection'})
            require(data['verus-infer']['interface_written']==positive and o['export_exists']==positive,'import publication')
            if not positive:
                kind='postcondition not satisfied' if label.endswith('wrong') else 'precondition not satisfied'
                require(kind in err,'import negative kind')
        else:
            require(o['returncode']!=0 and ('No such file' in err or 'failed to read' in err),'failed import rejection')
        if data:
            inf=data.get('verus-infer',{});inventory[label]=dict(helpers=inf.get('functions',[]),scope=inf.get('verification_scope'))
        require(not any(x in err.lower() for x in ['panicked at','resource limit','timed out']),'resource failure')
    for command in r['commands']:
        args=command['argv'];require('--no-cheating' in args,'normal checking')
        require(not any(a.startswith(('--no-verify','--no-lifetime','--no-erasure','--verify-function','--verify-module')) for a in args),'disabled check')
    # Preserve and classify the earlier failures rather than treating them as semantic controls.
    evidence=HERE/'evidence'
    manifest=json.loads((evidence/'MANIFEST.json').read_text())
    for n,h in manifest.items():require(digest(evidence/n)==h,'evidence '+n)
    for attempt,expected in [('nominal01',4),('contract02',2),('contract03',2)]:
        folder=evidence/('redb-eligibility-'+attempt)
        previous=json.loads((folder/'receipt.json').read_text());require(previous['success'] and len(previous['outcomes'])==expected,'attempt inventory')
        for o in previous['outcomes']:
            err=(folder/(o['label']+'.stderr')).read_text();v=o['data']['verification-results']
            require(o['returncode']!=0,'expected earlier rejection')
            if attempt=='nominal01' and o['label'].startswith('without'):
                require(v['encountered-vir-error'] and all(t in err for t in ['core::io::error::Error','core::panic::location::Location']),'original payload barrier')
            elif attempt=='nominal01':
                require(not v['encountered-vir-error'] and 'precondition not satisfied' in err,'nominal admission')
            elif attempt=='contract02':require(v['encountered-vir-error'] and 'non-visible datatype' in err,'visibility setup failure')
            else:
                require(not v['encountered-vir-error'] and err.count('error: postcondition not satisfied')==4 and 'precondition not satisfied' not in err,'numeric ordering blocker')
            native=o['data'].get('verus-infer',{});require(native.get('functions',[])==[],'earlier helper inventory')
        for fixture in folder.glob('*.rs'):
            text=fixture.read_text()
            for name in frozen['bodies']:
                original=p.item(p.H,'fn',name);body=original[original.index('{'):]
                require(text.count(body)==1,attempt+' body '+name)
    return dict(stage='NOMINAL_ELIGIBLE_FULL_NUMERIC_SELECTION_CONTRACT_UNPROVED',source_coupling_proved=False,
        complete_selection_contract_proved=False,finalize_contract_proved=True,selection_domain_only=True,
        body_inventory=body_inventory,field_inventory=field_inventory,storage_error_sha256=sha(error),
        nominal_declarations_sha256=sha(NOMINAL),inventory=inventory,
        strict_closeout_receipt_sha256=digest(out/'receipt.json'),
        first_unproved_sequent='For original TransactionId a,b: the boolean returned by <TransactionId as PartialOrd>::gt(&a,&b) iff a.0 > b.0. Necessary when !two_phase_commit && !primary_corrupted && !secondary_corrupted. Derived Ord/PartialOrd bodies are outside selected scope; no law added.',
        caller_establishment='UNPROVED: primary_slot < 2 and redundant xor bounds/distinctness at actual selection callers',
        proof_bodies=[],inferred_helpers=[],infer_annotation_reduction=0,
        HMAC_native='DEFERRED AND UNACHIEVED; native0/44+3/101 unchanged')
if __name__=='__main__':print(json.dumps(audit(Path(sys.argv[1])),indent=2))
