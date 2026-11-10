"""Pinned completed graph binding. Identity/shape only, no source effect laws."""
import copy
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
EXTRACTION = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def selected(native, borrowck, coverage):
    need(native['pointer_bits'] == 64 and len(native['instances']) == 6,
         'wrong target or instance count')
    need(not native['extraction_blockers'] and not native['pending_instances'],
         'unfinished extraction')
    root = native['instances'][0]
    need(root['id'] == native['root'], 'wrong root')
    incoming = next(x for x in root['locals'] if x['local'] == 1)
    need(incoming['ty'] == '&mut [u8; 128]', 'wrong incoming reference')
    first = root['blocks'][0]
    ref = first['statements'][3]['ir']
    need(ref == dict(destination=dict(local=4, projection=[], ty='&mut [u8; 128]'),
                    op='assign', value=dict(fake=False, kind='Mut { kind: TwoPhaseBorrow }',
                    mutable=True, op='ref', place=dict(local=1, projection=[dict(op='deref')],
                                                     ty='[u8; 128]'))), 'root reborrow changed')
    cast = first['statements'][4]['ir']
    need(cast['value']['kind'] == 'Unsize' and cast['destination']['local'] == 3 and
         cast['value']['operand']['place']['local'] == 4, 'root unsize changed')
    need(first['term_ir']['arguments'][0]['place']['local'] == 3 and
         first['term_ir']['destination']['local'] == 2, 'constructor operand changed')
    loan = borrowck['loans'][0]
    need(loan['loan'] == 0 and loan['assigned']['local'] == 4 and
         loan['borrowed']['local'] == 1 and loan['activation'] == 'ActivatedAt(bb0[4])',
         'loan0 is not the incoming source loan')
    rows = []
    for i, inst in enumerate(native['instances']):
        for block in inst['blocks']:
            for index, op in [*enumerate(block['statements']), (None, dict(ir=block['term_ir']))]:
                c = coverage['coverage'][len(rows)]
                need((c['instance'], c['bb'], c['index']) ==
                     (inst['id'], block['bb'], index if index is not None else 'term'),
                     'coverage/site mismatch')
                rows.append(dict(instance=i, instance_identity=inst['id'], bb=block['bb'],
                                 index=index, cleanup=block['cleanup'], ir=op['ir'],
                                 descriptor_status=c['status']))
    need(len(rows) == 44 == native['operations'], 'incomplete graph')
    phases = []
    for block in borrowck['blocks']:
        for statement in block['statements']:
            if statement['ir']['op'] == 'unsupported_statement':
                need('FakeRead' in statement['ir']['debug'], 'unknown pre-erasure phase')
                phases.append(dict(kind='FakeReadPhase', location=statement['location'],
                                   ir=statement['ir']))
        if block['cleanup']:
            need(block['terminator']['op'] == 'unwind_resume', 'unknown cleanup')
            phases.append(dict(kind='CleanupResumePhase', bb=block['bb'], ir=block['terminator']))
        if block['terminator']['op'] == 'call':
            phases.append(dict(kind='CallUnwindPhase', bb=block['bb'],
                               unwind=block['terminator']['unwind']))
    need(len(phases) == 3, 'incomplete phase graph')
    return dict(schema=1, classification='source_identity_only_not_effect_theorem',
                operations=rows, phases=phases, loan=loan,
                incoming_dynamic_tag='not supplied by static extraction',
                actual_source_semantics_instance=False, source_certificate=False,
                native_correspondences=0, publication=False)


def load_graph():
    pins = json.loads((HERE.parent/'native_mir_lrust_refinement/dependency-pins.json').read_text())
    bindings = json.loads((HERE.parent/'native_semantic_bridge/source-bindings.json').read_text())
    for p, expected in pins['source_files'].items():
        need(sha(p) == expected, 'completed extraction identity changed: '+p)
    need(sha(EXTRACTION/'receipt.json') == bindings['receipt_sha256'], 'source receipt mismatch')
    for n, expected in bindings['outputs'].items():
        need(sha(EXTRACTION/n) == expected, 'source binding changed: '+n)
    return (json.loads((EXTRACTION/'drop-native.json').read_text()),
            json.loads((EXTRACTION/'drop-borrowck.json').read_text()),
            json.loads((HERE.parent/'native_mir_lrust_refinement/source-coverage.json').read_text()))


def coq_graph(graph):
    source = ('From NativeConstructorErasure Require Import Interface.\n'
              'From Coq Require Import List.\nImport ListNotations.\n'
              '(* Site inventory, not a semantic interpretation of these sites. *)\n'
              'Definition graph_events : list event := [\n')
    events = []
    for row in graph['operations']:
        idx = 'None' if row['index'] is None else '(Some '+str(row['index'])+'%nat)'
        events.append(f"MirOperation {row['instance']}%nat {row['bb']}%nat {idx}")
    events += ['PhaseEvent '+row['kind'] for row in graph['phases']]
    source += ';\n'.join('  '+e for e in events)+'].\n'
    source += ('Lemma graph_events_count : List.length graph_events = 47%nat.\n'
               'Proof. reflexivity. Qed.\n'
               'Lemma root_reservation_listed :\n'
               '  In (MirOperation 0%nat 0%nat (Some 3%nat)) graph_events.\n'
               'Proof. simpl. auto. Qed.\n'
               'Lemma phase_events_listed :\n'
               '  In (PhaseEvent FakeReadPhase) graph_events /\\\n'
               '  In (PhaseEvent CallUnwindPhase) graph_events /\\\n'
               '  In (PhaseEvent CleanupResumePhase) graph_events.\n'
               'Proof. simpl. intuition congruence. Qed.\n')
    return source


def controls(data):
    results = {}
    for label in ['wrong-original-operand', 'activation', 'wrong-unsize', 'wrong-constructor-operand']:
        n, b, c = copy.deepcopy(data)
        if label == 'wrong-original-operand':
            n['instances'][0]['blocks'][0]['statements'][3]['ir']['value']['place']['local'] = 9
        elif label == 'activation':
            b['loans'][0]['activation'] = 'NotActivated'
        elif label == 'wrong-unsize':
            n['instances'][0]['blocks'][0]['statements'][4]['ir']['value']['operand']['place']['local'] = 1
        else:
            n['instances'][0]['blocks'][0]['term_ir']['arguments'][0]['place']['local'] = 4
        try:
            selected(n, b, c)
        except ValueError as error:
            results[label] = str(error)
        else:
            raise ValueError('mutated structured source accepted: '+label)
    return results
