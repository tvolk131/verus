#!/usr/bin/env python3
"""Authenticate historical Git objects and collect compact independent evidence."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
import review as r

DELIVERY={'constructor':'3d668d29','pointer':'2c0b8f23','exit':'173d169e'}

def history():
    r.guard()
    freeze=r.read(r.HERE/'finalize-freeze.json')
    r.require(freeze['sha256']==r.sha(Path(__file__)), 'finalizer changed')
    output={}
    for short,(lane,_,_,_) in r.LANES.items():
        directory=r.RESEARCH/lane
        historical=r.read(directory/'results.json')
        f=directory/'final-freeze.json'
        expected=(historical.get('freeze_sha256') or historical.get('final_freeze_sha256') or historical.get('freeze',{}).get('sha256'))
        r.require(r.sha(f)==expected,'historical freeze index mismatch')
        frozen=r.read(f)
        sources={}
        for name,wanted in frozen['files'].items():
            # Read immutable Git object bytes, never a historical checkout path.
            p=subprocess.run(['git','show',DELIVERY[short]+':'+name],cwd=r.ROOT,capture_output=True,check=True)
            actual=hashlib.sha256(p.stdout).hexdigest()
            r.require(actual==wanted, 'historical Git source does not match frozen source '+name)
            sources[name]=actual
        receipt=r.read(r.read(directory/'integration-results.json')['delivered_receipt']['path'])
        delivered_source_pins={}
        for name,facts in receipt['inputs'].items():
            if name.startswith('/tmp/'): continue
            for relative,wanted in sources.items():
                if name.endswith('/'+relative):
                    r.require(facts['sha256']==wanted,'receipt disagrees with historical Git source')
                    delivered_source_pins[relative]=wanted
        r.require(set(delivered_source_pins)==set(sources),'receipt lacks historical frozen source pins')
        output[short]={'delivery_commit':DELIVERY[short], 'historical_freeze':r.pin(f),
                       'all_frozen_files_authenticated_against_git_and_receipt':sources}
    # Audit the predecessor review at the delivered review commit as immutable Git bytes.
    old={}
    for name in ['REPORT.md','results.json','review.py']:
        rel='evidence/native-components-model-review/'+name
        p=subprocess.run(['git','show','170e24b6:'+rel],cwd=r.ROOT,capture_output=True,check=True)
        actual=hashlib.sha256(p.stdout).hexdigest()
        r.require(actual==r.sha(r.ROOT/rel),'historical independent review changed')
        old[rel]=actual
    output['review170e24b6']=old
    # Authenticate every explicitly indexed completed development receipt, without
    # replaying superseded proof prototypes or treating failures as semantics.
    trials=[]
    for t in r.read(r.RESEARCH/'native_original_reference_exit/development.json')['trials']:
        r.require(r.sha(t['receipt'])==t['sha256'],'development receipt digest mismatch')
        d=r.read(t['receipt'])
        r.require(d['success']==t['success'],'development success mismatch')
        trials.append({k:t[k] for k in ['trial','receipt','sha256','success','classification','semantic_negative']})
    output['exit_development_receipts']=trials
    r.guard()
    r.write(r.RAW/'history-authentication.json',output)
    print('historical sources, frozen controllers and completed receipts authenticated')


def final():
    r.guard()
    r.require(r.read(r.HERE/'finalize-freeze.json')['sha256']==r.sha(Path(__file__)),'finalizer changed')
    result={'schema':1,'base':'bc2a0b37','raw':str(r.RAW), 'pretrial':r.pin(r.RAW/'pretrial.json'),
            'history_authentication':r.pin(r.RAW/'history-authentication.json'),
            'decisions':[
                {'subject':'F1','decision':'accept','severity':'none','historical_severity':'P2',
                 'scope':'constructor/exit current replay stable goal requirements AND guarded full TARGET snapshot'},
                {'subject':'F2','decision':'accept','severity':'none','historical_severity':'P2',
                 'scope':'77 mathematical declarations accepted separately from complete inventory acceptance of 231 declarations/203 constants'},
                {'subject':'exit_scope','decision':'accept','severity':'none',
                 'scope':'17 conditional/algebra/MODEL source declarations plus 68 closed generated helpers only'}],
            'native_correspondences':0,'minimal_operations_open':44,'phase_obligations_open':3,
            'writer_effects_open':101,'publication':False,'goal_complete':False,
            'canonical':'inherited 6/0; not rerun','full_HMAC':'inherited 1/1 unproved; not rerun',
            'same_actual_checked_automation_for_both_full_goals_established':False,
            'actual_source_instance_proved':False,'source_certificate_proved':False,
            'OriginalOperandCF_proved':False,'PendingExitCF_proved':False,
            'all_fresh_math_source_object_hashes_match_delivery':True,
            'normal_compilation_and_recursive_kernel_replay':{},'complete_assumptions_inventory':'assumptions.json',
            'process_deviation':{'before_proof_trials':True,
                'historical_TARGET_opened':'/home/dev/projects/verus/source/tools/research/mutable_buffer_goal/TARGET.md',
                'reason':'initial harness recognized only .t3/worktrees historical labels',
                'corrected_in':'afb6c2fc','proof_or_semantic_evidence_from_wrong_checkout':False,
                'overbroad_ancestor_find':'interrupted before any output; no files opened by that find'},
            'nonsemantic_review_failures':['initial preparation TARGET path remapping mismatch; no proof commands ran',
                'independent exit parser rejected short printed name of inherited extensionality; successful compile outputs retained; corrected independently'],
            'exit_inventory_parser_failure':r.pin(r.RAW/'exit-inventory-parser-failure.json'),
            'goal_scope_tests':{},'commands':[]}
    assumptions={};hashes={}
    pre=r.read(r.RAW/'pretrial.json')
    result['delivered_and_integration_authentication']=pre['authentication']
    for short,(lane,_,_,_) in r.LANES.items():
        receipt=r.read(r.RAW/short/'receipt.json')
        independent=r.read(r.RAW/(short+'-independent.json'))
        require_set={**receipt['assumptions'],**receipt.get('helper_assumptions',{})}
        r.require(independent['assumptions']==require_set,'independent/controller individual assumptions differ')
        result['normal_compilation_and_recursive_kernel_replay'][short]={
            'receipt':r.pin(r.RAW/short/'receipt.json'),'independent_receipt':r.pin(r.RAW/(short+'-independent.json')),
            'source_count':receipt['constant_count'],'complete_declarations':independent['declaration_count'],
            'kernel_constants':independent['constant_count'],'kernel_axioms':receipt['kernel_axioms'],
            'module_discovery':independent['discovered_modules']}
        assumptions[short]={'individual':independent['assumptions'],'recursive_kernel_axioms':receipt['kernel_axioms'],
                            'source_constructors_eliminators_projections_and_generated_helpers_all_included':True,
                            'review_modules_declaration_free':True}
        original=pre['authentication'][short]['unchanged_math']
        fresh={Path(p).name:f['sha256'] for p,f in receipt['proof_artifacts'].items()}
        r.require(all(fresh[n]==h for n,h in original.items()),'fresh objects differ')
        hashes[short]={'math_sources_objects':original,'fresh_products':fresh,
                      'delivery_same_as_fresh':{n:True for n in original}}
        if short=='exit':
            old=r.read(pre['authentication'][short]['receipts']['delivered_receipt']['path'])
            r.require(receipt['proof_terms']==old['proof_terms'],'exit printed proof terms changed')
            hashes[short]['printed_proof_terms']=receipt['proof_terms']
    for mode in ['normal','O']:
        command=r.read(r.RAW/('goal-scope-'+mode+'-command.json'))
        text=Path(command['stderr']['path']).read_text()
        r.require('Ran 7 tests' in text and text.rstrip().endswith('OK'),'goal scope tests incomplete')
        result['goal_scope_tests'][mode]=command
    for p in sorted(r.RAW.glob('*-command.json')): result['commands'].append(r.read(p))
    r.guard()
    for name,value in [('assumptions.json',assumptions),('hashes.json',hashes),('results.json',result)]:
        r.write(r.HERE/name,value)
    r.write(r.RAW/'final-receipt.json',{'success':True,'all_final_input_and_product_guards_passed':True,
            'publication':False,'goal_complete':False,'products':{str(r.HERE/n):r.sha(r.HERE/n)
            for n in ['results.json','hashes.json','assumptions.json']}})
    print('independent review finalized; conditional/MODEL only')


if __name__=='__main__':
    if sys.argv[1]=='history': history()
    elif sys.argv[1]=='final': final()
    else: raise ValueError('unknown action')
