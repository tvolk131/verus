"""Regressions for actual review counterexamples; no native semantics claimed."""
from pathlib import Path
import copy
import json
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from proof_checks import names, parse_axioms, parse_inventory, parse_kernel_summary
from source_map import build, authenticated_extraction

EXTRACTION = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')
SUMMARY = '''CONTEXT SUMMARY
===============
* Theory: Set is predicative
* Theory: Rewrite rules are not allowed
* Axioms:
    Coq.Logic.FunctionalExtensionality.functional_extensionality_dep
    Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq
* Constants/Inductives relying on type-in-type: <none>
* Constants/Inductives relying on unsafe (co)fixpoints: <none>
* Inductives whose positivity is assumed: <none>
'''


class Assumptions(unittest.TestCase):
    def test_single_line_and_qualified_extra_axioms_reject(self):
        for axiom in ('classic', 'Coq.Logic.Classical_Prop.classic',
                      'Other.functional_extensionality_dep'):
            with self.subTest(axiom=axiom), self.assertRaises(ValueError):
                parse_axioms('Axioms:\n' + axiom + ' : forall P : Prop, P \\/ ~ P')

    def test_indented_extra_axiom_rejects(self):
        with self.assertRaises(ValueError):
            parse_axioms('Axioms:\nfunctional_extensionality_dep :\n  forall x, x = x\n  classic : forall P, P \\/ ~ P')

    def test_closed_and_multiline_allowed(self):
        self.assertEqual(parse_axioms('Closed under the global context\n'), [])
        self.assertEqual(len(parse_axioms('Axioms:\nfunctional_extensionality_dep :\n  forall (A : Type), A = A')), 1)

    def test_truncated_or_ambiguous_assumptions_reject(self):
        for text in ('', 'Axioms:\n', 'Closed under the global context\nAxioms:',
                     'Axioms:\n  forall x, x = x'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_axioms(text)

    def test_exact_inventory_and_missing_duplicate_mislabelled_blocks(self):
        text = ''.join(f'BEGIN {n}\nClosed under the global context\nEND {n}\n' for n in names())
        self.assertEqual(len(parse_inventory(text)), 25)
        first = names()[0]
        for corrupt in (text.replace('BEGIN ' + first, 'BEGIN missing', 1),
                        text.replace('END ' + first, 'END missing', 1), text + text,
                        text[text.index('BEGIN ' + names()[1]):]):
            with self.assertRaises(ValueError):
                parse_inventory(corrupt)

    def test_kernel_summary_rejects_extra_axiom_and_unsafe_options(self):
        self.assertEqual(len(parse_kernel_summary(SUMMARY)), 2)
        corruptions = [SUMMARY.replace('* Constants/Inductives', '    Coq.Logic.Classical_Prop.classic\n* Constants/Inductives', 1),
                       SUMMARY.replace('not allowed', 'allowed'), SUMMARY.replace('predicative', 'impredicative'),
                       SUMMARY.replace('type-in-type: <none>', 'type-in-type: Bad.constant'),
                       SUMMARY.replace('unsafe (co)fixpoints: <none>', 'unsafe (co)fixpoints: Bad.constant'),
                       SUMMARY.replace('positivity is assumed: <none>', 'positivity is assumed: Bad.inductive'),
                       SUMMARY + SUMMARY, '']
        for text in corruptions:
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_kernel_summary(text)


class SourceBindings(unittest.TestCase):
    def setUp(self):
        self.native = json.loads((EXTRACTION/'drop-native.json').read_text())
        self.borrowck = json.loads((EXTRACTION/'drop-borrowck.json').read_text())
        self.source = HERE.parent/'native_heap_lowering/fixtures/drop.rs'

    def test_real_chain_and_review_mutants(self):
        self.assertEqual(build(self.native, self.borrowck, self.source)['operation_count'], 44)
        for kind in ('unsize', 'call', 'reborrow'):
            b = copy.deepcopy(self.borrowck)
            if kind == 'unsize':
                b['blocks'][0]['statements'][4]['ir']['value']['operand']['place']['local'] = 1
            elif kind == 'call':
                b['blocks'][0]['terminator']['arguments'][0]['place']['local'] = 1
            else:
                b['blocks'][0]['statements'][3]['ir']['value']['place']['local'] = 4
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                build(self.native, b, self.source)

    def test_authentication_rejects_changed_output_and_forged_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            for name in ('drop-native.json', 'drop-borrowck.json', 'receipt.json'):
                (out/name).write_bytes((EXTRACTION/name).read_bytes())
            authenticated_extraction(out)
            p = out/'drop-borrowck.json';b = json.loads(p.read_text())
            b['blocks'][0]['terminator']['arguments'][0]['place']['local'] = 1
            p.write_text(json.dumps(b))
            with self.assertRaises(ValueError):
                authenticated_extraction(out)
            (out/'drop-borrowck.json').write_bytes((EXTRACTION/'drop-borrowck.json').read_bytes())
            (out/'receipt.json').write_text(json.dumps({'success': True}))
            with self.assertRaises(ValueError):
                authenticated_extraction(out)


if __name__ == '__main__':
    unittest.main()
