"""Closed model-proof inventory and fail-closed Rocq output checks."""
import re

NAMES = {
    'borrow_handoff': ['source_loan_access', 'source_loan_release', 'source_context_access'],
    'byte_array': ['byte_list_length', 'bytes_size', 'bytes_physical', 'bytes_initialized',
                   'byte_array128_size', 'byte_array128_release',
                   'byte_array128_scoped_handoff', 'byte_array128_context_handoff'],
    'raw_view': ['typed_source_raw_view', 'typed_source_raw_offset', 'typed_source_loan_release'],
    'model_limits': ['arbitrary_raw_pointer_typed', 'raw_read_empty_head_stuck',
                     'model_offset_unrestricted', 'empty_heap_offset_adequate'],
}
ALIASES = {
    'functional_extensionality_dep': 'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep',
    'FunctionalExtensionality.functional_extensionality_dep': 'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep',
    'Eqdep.Eq_rect_eq.eq_rect_eq': 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq',
}
ALLOWED = set(ALIASES.values())


def need(condition, message):
    if not condition:
        raise ValueError(message)


def names():
    return ([module + '.' + n for module, ns in NAMES.items() for n in ns]
            + ['byte_array.byte_ty_obligation_' + str(i) for i in range(1, 7)]
            + ['byte_array.byte_ty'])


def check_inventory(candidate, modules):
    need(modules == list(NAMES), 'exact four-module inventory required')
    need({p.stem for p in candidate.glob('*.v')} == set(NAMES),
         'unexpected proof source inventory')
    for module, expected in NAMES.items():
        actual = re.findall(r'^\s*(?:Lemma|Theorem)\s+(\w+)',
                            (candidate / (module + '.v')).read_text(), re.M)
        need(actual == expected, 'named theorem inventory mismatch: ' + module)


def inventory_source():
    source = 'From NativeSemanticBridge Require Import ' + ' '.join(NAMES) + '.\n'
    for name in names():
        source += f'Goal True. idtac "BEGIN {name}". exact I. Qed.\n'
        source += f'Print Assumptions NativeSemanticBridge.{name}.\n'
        source += f'Goal True. idtac "END {name}". exact I. Qed.\n'
    source += 'Fail Check NativeSemanticBridge.byte_array.byte_ty_obligation_7.\n'
    return source


def parse_axioms(text):
    text = text.strip()
    if text == 'Closed under the global context':
        return []
    need(text.startswith('Axioms:\n'), 'unclassified assumption output')
    lines = text.splitlines()[1:]
    axioms = []
    for line in lines:
        if not line.strip():
            continue
        found = re.match(r'^\s*(\S+)\s*:', line)
        if found:
            name = ALIASES.get(found[1], found[1])
            need(name in ALLOWED, 'unexpected logical axiom: ' + found[1])
            axioms.append(name)
        else:
            need(bool(axioms) and line[:1].isspace(), 'malformed axiom declaration')
    need(bool(axioms), 'empty or truncated axiom output')
    need(len(axioms) == len(set(axioms)), 'duplicate axiom declaration')
    return axioms


def parse_inventory(text):
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', text, re.M | re.S)
    expected = names()
    need([a for a, _, _ in sections] == expected, 'incomplete assumption coverage')
    need([b for _, _, b in sections] == expected, 'mismatched assumption boundaries')
    need(len(re.findall(r'^BEGIN ', text, re.M)) == len(expected)
         and len(re.findall(r'^END ', text, re.M)) == len(expected),
         'extra assumption boundaries')
    result = {name: parse_axioms(body) for name, body, _ in sections}
    for name, axioms in result.items():
        closed = (not name.startswith('byte_array.') or name == 'byte_array.byte_list_length'
                  or name == 'byte_array.byte_ty' or '.byte_ty_obligation_' in name)
        need(not axioms if closed else set(axioms) <= {ALIASES['functional_extensionality_dep']},
             'assumption policy changed for ' + name)
    return result


def parse_kernel_summary(text):
    need(text.count('CONTEXT SUMMARY') == 1, 'missing or duplicate kernel summary')
    summary = text.split('CONTEXT SUMMARY', 1)[1]
    expected = {
        'Theory: Set is predicative',
        'Theory: Rewrite rules are not allowed',
        'Constants/Inductives relying on type-in-type: <none>',
        'Constants/Inductives relying on unsafe (co)fixpoints: <none>',
        'Inductives whose positivity is assumed: <none>',
    }
    headings = re.findall(r'^\* (.+?)\s*$', summary, re.M)
    need(len(headings) == len(expected) + 1 and set(headings) == expected | {'Axioms:'},
         'unsafe or unrecognized kernel summary')
    block = re.search(r'^\* Axioms:\s*\n(.*?)(?=^\* |\Z)', summary, re.M | re.S)
    need(block is not None, 'missing kernel axiom inventory')
    axioms = [line.strip() for line in block[1].splitlines() if line.strip()]
    need(len(axioms) == len(set(axioms)) and set(axioms) <= ALLOWED,
         'unexpected kernel axioms: ' + repr(axioms))
    return axioms
