#!/usr/bin/env python3
"""Complete typed-source obligation inventory, NOT a MIR semantics/lowering rule.

Every actual instruction has a binding and an unresolved correspondence goal.
Even an ordinary checked conditional carrier law does not discharge those goals.
"""
from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from infer_experiment import require
sys.path.insert(0,str(HERE.parent/'native_iterator_certificate'))
from check import inspect, Unsupported


def binding(x):
    return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def obligations(evidence,source_sha256):
    inspection=inspect(evidence,source_sha256)
    instances={i['id']:i for i in evidence['instances']}
    require(not evidence['pending_instances'],'incomplete native graph')
    entries=[]
    locations=set()
    calls=[]
    drops=[]
    leaves=[]
    for instance in evidence['instances']:
        iid=instance['id']
        if instance['kind']!='mir':
            leaves.append(dict(instance=iid,kind=instance['kind'],correspondence='unproved'))
            continue
        require([b['bb'] for b in instance['blocks']]==list(range(len(instance['blocks']))),'block numbering')
        for local in instance['locals']:
            if local.get('drop_query'):
                did=local['drop_query']['instance']
                require(did in instances,'missing native drop helper')
                drops.append(dict(instance=iid,local=local['local'],callee=did,
                    empty_return=instances[did]['kind']=='mir' and all(not b['statements'] and b['term_ir']=={'op':'return'} for b in instances[did]['blocks']),
                    ownership_release='unproved'))
        for block in instance['blocks']:
            seq=[(f's{n}',s['ir']) for n,s in enumerate(block['statements'])]+[('term',block['term_ir'])]
            for position,op in seq:
                loc=(iid,block['bb'],position)
                require(loc not in locations,'duplicate native location')
                locations.add(loc)
                # A binding to actual typed MIR is included for every operation,
                # including local moves, storage, branches, returns and cleanup.
                entries.append(dict(instance=iid,bb=block['bb'],position=position,
                    operation=op['op'],typed_ir_sha256=binding(op),
                    source=instance['definition_source'],
                    required='typed-MIR-to-linear-carrier-simulation',status='unproved'))
            term=block['term_ir']
            if term['op'] in ('call','drop'):
                edge=block['call']
                require(edge is not None and edge['resolved'] in instances,'unresolved native helper')
                calls.append(dict(instance=iid,bb=block['bb'],callee=edge['resolved'],
                    unwind=term['unwind'],correspondence='unproved'))
    require(len(entries)==evidence['operations'],'operation count does not cover extraction')
    return dict(schema=1,native_bridge_proved=False,evidence_sha256=binding(evidence),
        source_sha256=source_sha256,root=evidence['root'],target=evidence['target'],pointer_bits=evidence['pointer_bits'],
        instances=len(instances),operations=len(entries),instruction_obligations=entries,
        call_edges=calls,drop_queries=drops,unavailable_or_intrinsic=leaves,
        inherited_effect_obligations=inspection['generated_obligations'],
        inherited_blockers=inspection['blockers'],writer_diagnostics=inspection['writer_diagnostics'],
        conditional_carrier_laws='separate ordinary proofs; no source obligations discharged',
        checked_native_source_correspondences=[],
        missing=['initial executable borrow entry (sibling)',
            'independently checked interpretation of every typed MIR operation',
            'pointer offset/metadata/retag/validity and alias simulation',
            'every helper/callback/drop/unwind/panic exit simulation',
            'entry and return linked to the same executable original owner',
            'actual native induction and unchanged full writer theorem'])


def validate_inventory(evidence,source_sha256,ledger):
    require(ledger==obligations(evidence,source_sha256),'typed-source ledger altered or incomplete')


def publication(evidence,source_sha256,ledger,proof):
    validate_inventory(evidence,source_sha256,ledger)
    require(proof.get('status')=='proved','failed/incomplete/resource proof')
    # No user-supplied label, binding or collection of conditional proofs is an
    # ordinary proof of source refinement. This intentionally cannot succeed.
    raise Unsupported('no_machine_checked_native_lowering_soundness')


def controls(evidence,source_sha256):
    ledger=obligations(evidence,source_sha256)
    out=[]
    def reject(name,fn,classification):
        try: fn()
        except (ValueError,Unsupported) as error:
            out.append(dict(id=name,rejected=True,classification=classification,reason=str(error)))
        else: raise ValueError('false acceptance: '+name)
    for name,edit in [
        ('omitted-operation',lambda x:x['instruction_obligations'].pop()),
        ('duplicate-operation',lambda x:x['instruction_obligations'].append(x['instruction_obligations'][0])),
        ('missing-helper',lambda x:x['call_edges'].pop()),
        ('missing-drop',lambda x:x['drop_queries'].pop()),
        ('omitted-panic',lambda x:x['unavailable_or_intrinsic'].clear()),
        ('forged-source',lambda x:x.update(source_sha256='0'*64)),
        ('forged-operation-proof',lambda x:x['instruction_obligations'][0].update(status='proved')),
        ('forged-native-fact',lambda x:x.update(native_bridge_proved=True)),
    ]:
        changed=deepcopy(ledger);edit(changed)
        reject(name,lambda:validate_inventory(evidence,source_sha256,changed),'artifact/inventory-rejection')
    for status in ['failed','incomplete','timeout','resource','proved']:
        reject('publication-'+status,lambda:publication(evidence,source_sha256,ledger,dict(status=status)),
            'publication-barrier' if status=='proved' else 'proof-status-rejection')
    # Honesty of a rehash never discharges an unknown effect.
    altered=deepcopy(evidence)
    root=next(i for i in altered['instances'] if i['id']==altered['root'])
    statement=next(s for b in root['blocks'] for s in b['statements'])
    statement['ir']={'op':'unsupported_native_callback'}
    changed=obligations(altered,source_sha256)
    require(changed['evidence_sha256']!=ledger['evidence_sha256'],'rebound evidence')
    require(any(b['kind']=='unsupported_operation' for b in changed['inherited_blockers']),'unsupported effect not retained')
    reject('rebound-unknown-effect',lambda:publication(altered,source_sha256,changed,dict(status='proved')),'unsupported-source-effect')
    return out
