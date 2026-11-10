"""Specific false-certificate and honestly rebound source controls; no asserts."""
from copy import deepcopy
import hashlib
import json
from check import inspect, validate_certificate, walk
from infer_experiment import require

def run_controls(original, source_hash):
    results = []
    def expect(label, action, contains):
        try:
            action()
        except (ValueError, KeyError) as error:
            require(contains in str(error), f'{label}: wrong rejection: {error}')
            results.append(dict(control=label, rejected=True, reason=str(error)))
        else:
            raise ValueError(f'{label}: false input accepted')
    def check_mutation(label, mutation, contains):
        data = deepcopy(original)
        mutation(data)
        expect(label, lambda: inspect(data, source_hash), contains)
    check_mutation('forged-source', lambda d: d.update(input_sha256='0'*64), 'forged_source_binding')
    check_mutation('wrong-domain', lambda d: d['instances'][0]['locals'][1].update(ty='&mut [u8; 64]'), 'writer_type_domain_mismatch')
    check_mutation('wrong-layout', lambda d: d['instances'][0]['locals'][1]['layout'].update(size=16), 'argument_layout_mismatch')
    def remove_callee(d):
        callee = d['instances'][0]['blocks'][0]['call']['resolved']
        d['instances'] = [i for i in d['instances'] if i['id'] != callee]
    check_mutation('unexamined-helper', remove_callee, 'unexamined_helper')
    def remove_drop(d):
        glue = next(l['drop_query']['instance'] for l in d['instances'][0]['locals'] if l.get('drop_query'))
        d['instances'] = [i for i in d['instances'] if i['id'] != glue]
    check_mutation('unexamined-drop', remove_drop, 'unexamined_drop_query')

    def summary_mutation(label, mutate):
        d = deepcopy(original)
        mutate(d)
        # The whole mutated typed IR gets its own updated binding. Debug strings
        # deliberately remain unchanged; effects must come from typed nodes.
        updated_ir_sha256 = hashlib.sha256(json.dumps(d,sort_keys=True).encode()).hexdigest()
        inspection = inspect(d, source_hash)
        require(any(b['kind'] == 'native_value_summary' for b in inspection['blockers']), label + ': semantics not rejected')
        results.append(dict(control=label, rejected=True, updated_ir_sha256=updated_ir_sha256,
                            reason=[b for b in inspection['blockers'] if b['kind'] == 'native_value_summary']))
    def wrong_index(d):
        getter = next(i for i in d['instances'] if i.get('arg_count') == 2 and i['locals'][0]['ty'] == '&mut u8')
        offset = next(n for n in walk(getter) if n.get('op') == 'binary' and n['kind'] == 'Offset')
        offset['right'] = dict(op='constant', ty='usize', bits='1', zero_sized=False, evaluation_ok=True, evaluated='Ok(1)', literal='1_usize')
    summary_mutation('native-wrong-index', wrong_index)
    def wrong_origin(d):
        getter = next(i for i in d['instances'] if i.get('arg_count') == 2 and i['locals'][0]['ty'] == '&mut u8')
        offset = next(n for n in walk(getter) if n.get('op') == 'binary' and n['kind'] == 'Offset')
        offset['left'] = deepcopy(offset['right'])  # usize is not a pointer origin.
    summary_mutation('native-wrong-origin', wrong_origin)
    def wrong_zip(d):
        for i in d['instances']:
            for n in walk(i):
                if n.get('op') == 'assign' and n['value'].get('kind') == 'Add' and n['destination']['projection']:
                    n['destination']['projection'][-1]['field'] = 3
                    return
        raise ValueError('missing native increment')
    summary_mutation('native-wrong-Zip-field', wrong_zip)
    d = deepcopy(original)
    constructor = next(i for i in d['instances'] if i.get('arg_count') == 1 and i['locals'][1]['ty'] == '&mut [u8]'
        and any(n.get('op') == 'binary' and n['kind'] == 'Offset' for n in walk(i)))
    offset = next(n for n in walk(constructor) if n.get('op') == 'binary' and n['kind'] == 'Offset')
    offset['right'] = dict(op='constant', ty='usize', bits='127', zero_sized=False, evaluation_ok=True, evaluated='Ok(127)', literal='127_usize')
    inspection = inspect(d, source_hash)
    require(any(b['kind'] == 'native_constructor_summary' for b in inspection['blockers']), 'constructor extent mutation accepted')
    results.append(dict(control='native-constructor-wrong-extent', rejected=True, reason='native_constructor_summary'))

    for label, op in [('unsupported-rvalue','unsupported_rvalue'), ('unsupported-terminator','unsupported_terminator')]:
        d = deepcopy(original)
        block = d['instances'][0]['blocks'][0]
        if label == 'unsupported-rvalue':
            assignment = next(s['ir'] for s in block['statements'] if s['ir']['op'] == 'assign')
            assignment['value'] = dict(op=op, debug='UnknownOperation')
        else: block['term_ir'] = dict(op=op, debug='UnknownCallback')
        inspection = inspect(d, source_hash)
        require(any(b.get('op') == op for b in inspection['blockers']), 'unsupported operation accepted')
        results.append(dict(control=label, rejected=True, reason=op))
    inspected = inspect(original, source_hash)
    certificate = dict(evidence_sha256=hashlib.sha256(json.dumps(original,sort_keys=True).encode()).hexdigest(),
        source_sha256=source_hash, logical_zip_index='native_zip_index', owned_slots=list(range(128)), proof_status='proved')
    for label, change, reason in [
        ('lost-permission',dict(owned_slots=list(range(127))),'lost_permission'),
        ('duplicate-permission',dict(owned_slots=list(range(127))+[126]),'lost_permission'),
        ('incorrect-logical-Zip-projection',dict(logical_zip_index='physical_inner_cursor'),'incorrect_logical_Zip_projection'),
        *[(f'proof-{s}',dict(proof_status=s),'failed_incomplete_or_resource_limited_proof') for s in ('failed','incomplete','timeout','resource_limited')],
    ]:
        c = dict(certificate, **change)
        expect(label, lambda c=c: validate_certificate(original, inspected, c), reason)
    expect('unproved-original', lambda: validate_certificate(original, inspected, certificate), 'unsupported_effects_or_writer')
    forged = dict(inspected, blockers=[], writer_diagnostics=[], all_obligations_proved=True, missing_refinement=[])
    expect('forged-all-proofs', lambda: validate_certificate(original, forged, certificate), 'native_refinement_proof_checker_not_implemented')
    return results
