#!/usr/bin/env python3
"""Diagnostic typed-IR checks and conditional value interpretation.

Nothing in this module certifies Rust ownership, retags, or correspondence to
Verus observations. Unknown operations fail closed. No certificate can pass the
final publication barrier until that missing proof implementation exists.
"""
from collections import Counter
from copy import deepcopy
from dataclasses import dataclass, replace
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from infer_experiment import require

class Unsupported(ValueError):
    pass

@dataclass(frozen=True)
class Expr:
    op: str
    args: tuple

@dataclass(frozen=True)
class Address:
    root: str
    fields: tuple = ()

@dataclass(frozen=True)
class Pointer:
    origin: str
    offset: object = 0

@dataclass(frozen=True)
class WidePointer:
    data: Pointer
    length: object

@dataclass(frozen=True)
class Reference:
    address: object
    mutable: bool

@dataclass(frozen=True)
class Record:
    fields: tuple
    variant: int = 0

@dataclass(frozen=True)
class Uninitialized:
    pass

def expr(op, a, b):
    if op == 'Add' and b == 0:
        return a
    if op == 'Add' and a == 0:
        return b
    if type(a) is int and type(b) is int:
        if op == 'Add': return a + b
        if op == 'Lt': return a < b
        if op == 'BitXor': return a ^ b
    if op not in ('Add', 'Lt', 'BitXor'):
        raise Unsupported('symbolic_binary:' + op)
    return Expr(op, (a, b))

def scalar_json(value):
    if type(value) is int: return value
    if isinstance(value, Expr): return dict(op=value.op, args=[scalar_json(a) for a in value.args])
    raise Unsupported('non_scalar_VC_value')

class Interpreter:
    """Bounded, normal-return, value-only fragment. Inputs are hypothetical states.

    Copies/moves/retags are NOT interpreted as linear permission operations.
    Unwind behavior and raw-to-reference validity remain explicit obligations.
    """
    def __init__(self, evidence, budget=2048):
        self.instances = {i['id']: i for i in evidence['instances']}
        self.budget = budget
        self.fresh = 0
        self.conditions = []

    def tick(self):
        self.budget -= 1
        if self.budget < 0: raise Unsupported('symbolic_operation_budget')

    def read(self, memory, a):
        if not isinstance(a, Address) or a.root not in memory:
            raise Unsupported('uninitialized_or_external_read')
        v = memory[a.root]
        for f in a.fields:
            if not isinstance(v, Record): raise Unsupported('non_record_field')
            v = v.fields[f]
        if isinstance(v, Uninitialized): raise Unsupported('read_after_move_or_storage_live')
        return v

    def write(self, memory, a, value):
        if not isinstance(a, Address): raise Unsupported('unmodeled_byte_store')
        def update(v, fs):
            if not fs: return value
            if not isinstance(v, Record): raise Unsupported('non_record_store')
            fields = list(v.fields)
            fields[fs[0]] = update(fields[fs[0]], fs[1:])
            return replace(v, fields=tuple(fields))
        memory[a.root] = update(memory.get(a.root), a.fields)

    def address(self, p, frame, memory):
        a = Address(f'{frame}:{p["local"]}')
        for projection in p['projection']:
            op = projection['op']
            if op == 'deref':
                ref = self.read(memory, a)
                if isinstance(ref, Reference): a = ref.address
                elif isinstance(ref, Pointer): a = ref
                else: raise Unsupported('unmodeled_deref')
            elif op == 'field' and isinstance(a, Address):
                a = replace(a, fields=a.fields + (projection['field'],))
            elif op == 'downcast':
                v = self.read(memory, a)
                if not isinstance(v, Record) or v.variant != projection['variant']:
                    raise Unsupported('invalid_downcast')
            else:
                raise Unsupported('symbolic_projection:' + op)
        return a

    def operand(self, o, frame, memory):
        op = o['op']
        if op in ('copy', 'move'):
            a = self.address(o['place'], frame, memory)
            value = self.read(memory, a)
            if op == 'move': self.write(memory, a, Uninitialized())
            return value
        if op == 'constant' and o['evaluation_ok']:
            if o['bits'] is not None: return int(o['bits'])
            if o['zero_sized']: return Record(())
        raise Unsupported('symbolic_operand:' + op)

    def value(self, r, frame, memory, locals_):
        op = r['op']
        if op == 'use': return self.operand(r['operand'], frame, memory)
        if op == 'ref':
            if r['fake']: raise Unsupported('fake_borrow')
            a = self.address(r['place'], frame, memory)
            self.conditions.append(('borrow_validity_and_retag', repr(a)))
            return Reference(a, r['mutable'])
        if op == 'raw_ptr':
            self.conditions.append(('raw_pointer_validity', repr(r['place'])))
            return self.address(r['place'], frame, memory)
        if op == 'unary' and r['kind'] == 'PtrMetadata':
            v = self.operand(r['operand'], frame, memory)
            if isinstance(v, Reference): v = v.address
            if not isinstance(v, WidePointer): raise Unsupported('pointer_metadata_origin')
            return v.length
        if op == 'binary':
            a, b = self.operand(r['left'], frame, memory), self.operand(r['right'], frame, memory)
            if r['kind'] == 'Offset':
                if not isinstance(a, Pointer): raise Unsupported('offset_wrong_origin')
                self.conditions.append(('pointer_bounds_stride_and_provenance', repr((a, b))))
                return replace(a, offset=expr('Add', a.offset, b))
            return expr(r['kind'], a, b)
        if op == 'cast' and r['kind'] in ('Transmute','PtrToPtr'):
            o = r['operand']
            v = self.operand(o, frame, memory)
            if o['op'] not in ('copy', 'move'): raise Unsupported('transmute_operand')
            source_ty = o['place']['ty']
            source = next((l for l in locals_ if l['ty'] == source_ty), None)
            target = next((l for l in locals_ if l['ty'] == r['to']), None)
            if source is None or target is None: raise Unsupported('transmute_layout_missing')
            if r['kind'] == 'PtrToPtr':
                if source['shape']['kind'] != 'raw_ptr' or target['shape']['kind'] != 'raw_ptr':
                    raise Unsupported('pointer_cast_type')
                if target['shape']['pointee'] == 'u8' and isinstance(v, (Pointer,WidePointer)):
                    return v.data if isinstance(v, WidePointer) else v
                raise Unsupported('unmodeled_pointer_cast')
            if source['layout']['size'] != target['layout']['size'] or source['layout']['align'] != target['layout']['align']:
                raise Unsupported('transmute_layout_mismatch')
            s = source['shape']
            if (isinstance(v, Record) and len(v.fields) == 1 and s.get('repr_transparent') is True
                    and target['shape']['kind'] == 'raw_ptr' and isinstance(v.fields[0], (Pointer,WidePointer))):
                self.conditions.append(('transparent_pointer_validity', source_ty))
                return v.fields[0]
            if (isinstance(v, (Pointer,WidePointer)) and source['shape']['kind'] == 'raw_ptr'
                    and target['shape'].get('repr_transparent') is True
                    and len(target['shape']['variants']) == 1 and len(target['shape']['variants'][0]['fields']) == 1):
                self.conditions.append(('transparent_pointer_validity', r['to']))
                return Record((v,))
            raise Unsupported('unmodeled_transmute')
        if op == 'aggregate' and r['kind']['op'] in ('tuple', 'adt'):
            return Record(tuple(self.operand(o, frame, memory) for o in r['operands']), r['kind'].get('variant', 0))
        raise Unsupported('symbolic_rvalue:' + op)

    def run(self, id_, args, memory, depth=0):
        if depth > 12: raise Unsupported('symbolic_call_depth')
        i = self.instances.get(id_)
        if i is None or i['kind'] != 'mir': raise Unsupported('unexamined_callee')
        if len(args) != i['arg_count']: raise Unsupported('symbolic_arity')
        self.fresh += 1
        frame = str(self.fresh)
        memory = deepcopy(memory)
        for n, arg in enumerate(args, 1): memory[f'{frame}:{n}'] = arg

        def block(bb, memory, guards, visited):
            self.tick()
            if bb in visited: raise Unsupported('loop_requires_induction')
            b = i['blocks'][bb]
            for s in b['statements']:
                self.tick()
                r = s['ir']
                if r['op'] == 'assign':
                    value = self.value(r['value'], frame, memory, i['locals'])
                    self.write(memory, self.address(r['destination'], frame, memory), value)
                elif r['op'] in ('storage_dead', 'storage_live'): memory.pop(f'{frame}:{r["local"]}', None)
                elif r['op'] != 'nop': raise Unsupported('symbolic_statement:' + r['op'])
            t = b['term_ir']
            op = t['op']
            if op == 'return':
                yield guards, self.read(memory, Address(f'{frame}:0')), memory
            elif op == 'goto':
                yield from block(t['target'], memory, guards, visited | {bb})
            elif op == 'switch':
                value = self.operand(t['operand'], frame, memory)
                if type(value) in (int, bool):
                    target = next((c['target'] for c in t['cases'] if int(c['value']) == value), t['otherwise'])
                    yield from block(target, memory, guards, visited | {bb})
                elif len(t['cases']) == 1 and t['cases'][0]['value'] == '0':
                    for condition, target in ((False, t['cases'][0]['target']), (True, t['otherwise'])):
                        yield from block(target, deepcopy(memory), guards + ((value, condition),), visited | {bb})
                else: raise Unsupported('symbolic_switch')
            elif op == 'call':
                callee = b['call'].get('resolved') if b['call'] else None
                if callee is None: raise Unsupported('unexamined_call')
                self.conditions.append(('callee_nonunwind_or_checked_cleanup', callee))
                arguments = [self.operand(a, frame, memory) for a in t['arguments']]
                for gs, value, mem in self.run(callee, arguments, memory, depth+1):
                    self.write(mem, self.address(t['destination'], frame, mem), value)
                    if t['target'] is None: raise Unsupported('diverging_call')
                    yield from block(t['target'], mem, guards + gs, visited | {bb})
            else:
                raise Unsupported('symbolic_terminator:' + op)
        return list(block(0, memory, (), set()))

def native_step(evidence):
    """Check selected real next-body's normal value paths at an arbitrary state.

    This is a conditional value diagnostic, not a constructor, ownership or
    all-path refinement theorem. Type names select candidates only; actual
    instructions determine every returned value and mutation below.
    """
    candidates = [i for i in evidence['instances'] if i['kind'] == 'mir' and i['arg_count'] == 1
        and i['locals'][1]['shape'].get('kind') == 'ref'
        and i['locals'][1]['shape']['pointee'].startswith('std::iter::Zip<')
        and i['locals'][0]['ty'] == 'std::option::Option<(&mut u8, &u8)>']
    # Wrapper and specialized implementation must independently agree.
    require(len(candidates) == 2, 'native_next_inventory')
    index, length = Expr('index', ()), Expr('length', ())
    mutable = Record((Record((Pointer('ipad'),)), Pointer('ipad', 128), Record(())))
    immutable = Record((Record((Pointer('key'),)), Pointer('key', Expr('key_length', ())), Record(())))
    initial = Record((mutable, immutable, index, length))
    result = []
    for candidate in candidates:
        interpreter = Interpreter(evidence)
        paths = interpreter.run(candidate['id'], [Reference(Address('zip'), True)], {'zip': initial})
        require(len(paths) == 2, 'native_next_path_inventory')
        expected_guard = Expr('Lt', (index, length))
        actual_index = None
        for guards, value, memory in paths:
            require(len(guards) == 1 and guards[0][0] == expected_guard, 'native_next_guard')
            successful = guards[0][1]
            expected_state = replace(initial, fields=(mutable, immutable, expr('Add', index, 1), length)) if successful else initial
            require(memory['zip'] == expected_state, 'native_next_state_or_tail_mutation')
            expected_value = Record((Record((Reference(Pointer('ipad', index), True), Reference(Pointer('key', index), False))),), 1) if successful else Record((), 0)
            require(value == expected_value, 'native_getter_origin_index_or_projection')
            if successful: actual_index = memory['zip'].fields[2]
        result.append(dict(instance=candidate['id'], normal_paths_checked=2,
            conditional_value_summary=True, conditions=sorted(set(interpreter.conditions)),
            new_index_expression=scalar_json(actual_index),
            ownership_proved=False, unwind_proved=False))
    return result

def native_constructors(evidence):
    candidates = [i for i in evidence['instances'] if i['kind'] == 'mir' and i['arg_count'] == 1
        and i['locals'][1]['ty'] in ('&mut [u8]', '&[u8]')
        and i['locals'][0]['ty'].startswith(('std::slice::IterMut<','std::slice::Iter<'))]
    require(len(candidates) == 3 and sum(i['locals'][1]['ty'] == '&mut [u8]' for i in candidates) == 2,
            'native_constructor_inventory')
    results = []
    for candidate in candidates:
        mutable = candidate['locals'][1]['ty'] == '&mut [u8]'
        origin, length = ('ipad',128) if mutable else ('key',Expr('key_length',()))
        initial = Reference(WidePointer(Pointer(origin),length),mutable)
        interpreter = Interpreter(evidence)
        paths = interpreter.run(candidate['id'],[initial],{})
        require(len(paths) == 1 and paths[0][0] == (), 'native_constructor_paths')
        expected = Record((Record((Pointer(origin),)),Pointer(origin,length),Record(())))
        require(paths[0][1] == expected, 'native_constructor_origin_extent')
        # Conditional physical value shape only; PhantomData is not interpreted
        # as a generated ownership permission. No array/PointsTo bridge exists.
        results.append(dict(instance=candidate['id'], normal_paths_checked=1,
            origin=origin, extent=scalar_json(length), conditional_value_summary=True,
            conditions=sorted(set(interpreter.conditions)), ownership_proved=False))
    return results

KNOWN = {'assign','use','copy','move','constant','runtime_checks','ref','raw_ptr','cast','binary','unary',
         'discriminant','aggregate','tuple','adt','deref','field','downcast','index','constant_index',
         'storage_live','storage_dead','nop','set_discriminant','assume',
         'goto','switch','return','unreachable','call','drop','assert','bounds_check','overflow','unwind_resume','unwind_terminate'}

def walk(value):
    if isinstance(value, dict):
        yield value
        for child in value.values(): yield from walk(child)
    elif isinstance(value, list):
        for child in value: yield from walk(child)

def inspect(evidence, source_sha256):
    require(evidence['schema'] == 2, 'IR schema mismatch')
    require(evidence['native_bridge_proved'] is False, 'forged native status')
    require(evidence['input_sha256'] == source_sha256, 'forged_source_binding')
    require(evidence['target'] == 'x86_64-unknown-linux-gnu' and evidence['pointer_bits'] == 64, 'target_layout_mismatch')
    instances = {i['id']:i for i in evidence['instances']}
    require(len(instances) == len(evidence['instances']), 'duplicate instance')
    root = instances[evidence['root']]
    require(root['arg_count'] == 2 and root['locals'][1]['ty'] == '&mut [u8; 128]'
            and root['locals'][2]['ty'] == '&[u8]', 'writer_type_domain_mismatch')
    require((root['locals'][1]['layout']['size'], root['locals'][1]['layout']['align']) == (8,8)
            and (root['locals'][2]['layout']['size'], root['locals'][2]['layout']['align']) == (16,8), 'argument_layout_mismatch')
    blockers = list(evidence['extraction_blockers'])
    obligations = []
    counts = Counter()
    for i in evidence['instances']:
        for l in i.get('locals', []):
            require('error' not in l['layout'], 'unavailable_layout')
            require(l['layout']['align'] > 0 and l['layout']['size'] >= 0, 'invalid_layout')
            if l['shape']['kind'] == 'unsupported_type': blockers.append(dict(kind='unsupported_type', instance=i['id'], local=l['local']))
            if l.get('drop_query'):
                drop = instances.get(l['drop_query']['instance'])
                require(drop is not None, 'unexamined_drop_query')
                if l['needs_drop'] or drop['kind'] != 'mir' or any(b['statements'] or b['term_ir']['op'] != 'return' for b in drop['blocks']):
                    blockers.append(dict(kind='drop_effect_requires_proof', instance=drop['id']))
        for b in i.get('blocks', []):
            require(b['bb'] < len(i['blocks']) and all(0 <= s < len(i['blocks']) for s in b['successors']), 'invalid_cfg')
            if b['call']: require(b['call']['resolved'] in instances, 'unexamined_helper')
            for instruction in [*[s['ir'] for s in b['statements']], b['term_ir']]:
                for n in walk(instruction):
                    if 'op' not in n: continue
                    op = n['op']; counts[op] += 1
                    if op not in KNOWN: blockers.append(dict(kind='unsupported_operation', op=op, instance=i['id'], bb=b['bb']))
                    if op in ('call','drop','assert'):
                        blockers.append(dict(kind='unwind_effect_requires_proof', instance=i['id'], bb=b['bb'], action=n['unwind']))
                    if op in ('unwind_resume','unwind_terminate'):
                        blockers.append(dict(kind='unwind_effect_requires_proof', instance=i['id'], bb=b['bb']))
                    if op in ('binary','cast') and n['kind'] == 'unsupported_' + op:
                        blockers.append(dict(kind='unsupported_' + op, instance=i['id'], bb=b['bb']))
                    if op in ('ref','raw_ptr','cast','binary','assume','call','drop','assert','unreachable','unwind_resume','unwind_terminate'):
                        obligations.append(dict(kind=op, instance=i['id'], bb=b['bb'], status='unproved'))
                    if op == 'assign' and n['destination']['projection']:
                        obligations.append(dict(kind='store_origin_value_permission', instance=i['id'], bb=b['bb'], status='unproved'))
                    if op == 'constant' and not n['evaluation_ok']: blockers.append(dict(kind='constant_evaluation', instance=i['id'], bb=b['bb']))
    stores = [s['ir'] for b in root['blocks'] for s in b['statements']
              if s['ir']['op'] == 'assign' and s['ir']['destination']['projection']]
    writer_diagnostics = []
    if sum(b['term_ir']['op'] == 'call' for b in root['blocks']) != 4:
        writer_diagnostics.append('writer_call_inventory')
    if len(stores) != 1: writer_diagnostics.append('writer_store_inventory')
    elif stores[0]['value'].get('op') != 'binary' or stores[0]['value'].get('kind') != 'BitXor': writer_diagnostics.append('writer_xor_operation')
    elif stores[0]['value']['left'].get('place') != stores[0]['destination']: writer_diagnostics.append('writer_xor_lhs')
    else:
        # Follow the actual local Use definitions in the store's block. This is
        # an eligibility diagnostic for the extracted writer, not a loop proof.
        store = stores[0]
        block = next(b for b in root['blocks'] if any(s['ir'] == store for s in b['statements']))
        definitions = {}
        for statement in block['statements']:
            r = statement['ir']
            if r == store: break
            if r['op'] == 'assign' and not r['destination']['projection']:
                definitions[r['destination']['local']] = r['value']
        def use_place(value):
            if value['op'] not in ('copy','move'): raise Unsupported('writer_pair_origin')
            return value['place']
        def pair_reference(local, member):
            r = definitions.get(local,{})
            if r.get('op') != 'use': raise Unsupported('writer_pair_origin')
            p = use_place(r['operand'])
            if [(p.get('op'), p.get('field',p.get('variant'))) for p in p['projection']] != [('downcast',1),('field',0),('field',member)]:
                raise Unsupported('writer_pair_origin')
            return p['local']
        try:
            p = use_place(store['value']['right'])
            for _ in range(16):
                if p['projection']: break
                r = definitions.get(p['local'],{})
                if r.get('op') != 'use': raise Unsupported('writer_pair_origin')
                p = use_place(r['operand'])
            require([p['op'] for p in p['projection']] == ['deref'], 'writer_pair_origin')
            require(pair_reference(store['destination']['local'],0) == pair_reference(p['local'],1), 'writer_pair_origin')
        except (ValueError,KeyError): writer_diagnostics.append('writer_pair_origin')
    try: summary = native_step(evidence)
    except (ValueError, KeyError, IndexError) as error:
        summary = []; blockers.append(dict(kind='native_value_summary', reason=str(error)))
    try: constructors = native_constructors(evidence)
    except (ValueError, KeyError, IndexError) as error:
        constructors = []; blockers.append(dict(kind='native_constructor_summary', reason=str(error)))
    return dict(schema=1, native_bridge_proved=False, conditional_native_value_summaries=summary,
        conditional_constructor_value_summaries=constructors,
        writer_diagnostics=writer_diagnostics, operation_counts=dict(sorted(counts.items())), blockers=blockers,
        generated_obligations=obligations, all_obligations_proved=False,
        missing_refinement=['exclusive-array-to-linear-permissions','constructor-retains-128-slots',
            'indexed-yield-loan-transfer','None-and-drop-retain-unissued-slots',
            'borrow-resolution-reconstructs-original-array','Zip-indexed-logical-projection',
            'MIR-value-and-ownership-lowering-soundness','full-frozen-writer-theorem'])

def validate_certificate(evidence, inspection, certificate):
    """Fail closed. A string saying proved is not an ordinary checked proof."""
    require(certificate.get('evidence_sha256') == hashlib.sha256(json.dumps(evidence,sort_keys=True).encode()).hexdigest(), 'certificate_evidence_binding')
    require(certificate.get('source_sha256') == evidence['input_sha256'], 'certificate_source_binding')
    require(certificate.get('logical_zip_index') == 'native_zip_index', 'incorrect_logical_Zip_projection')
    require(certificate.get('owned_slots') == list(range(128)), 'lost_permission')
    require(certificate.get('proof_status') == 'proved', 'failed_incomplete_or_resource_limited_proof')
    require(not inspection['blockers'] and not inspection['writer_diagnostics'], 'unsupported_effects_or_writer')
    # Deliberate unconditional barrier: no trusted receipt format for the missing
    # native ownership/lowering proof has been implemented. Never accept user data.
    raise Unsupported('native_refinement_proof_checker_not_implemented')

if __name__ == '__main__':
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument('--evidence', type=Path, required=True)
    p.add_argument('--source', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    result = inspect(json.loads(a.evidence.read_text()), hashlib.sha256(a.source.read_bytes()).hexdigest())
    a.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(native_bridge_proved=False, value_summaries=len(result['conditional_native_value_summaries']),
        blockers=len(result['blockers']), obligations=len(result['generated_obligations']), writer_diagnostics=result['writer_diagnostics'])))
