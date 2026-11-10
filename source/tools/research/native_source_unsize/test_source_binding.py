"""Typed-source selection controls, without filesystem mutation experiments."""
from copy import deepcopy
import json
from pathlib import Path
import unittest

from source_binding import GRAPH, source_text


class SourceBindingTests(unittest.TestCase):
    def setUp(self):
        self.graph = json.loads(GRAPH.read_text())
        self.row = next(r for r in self.graph['operations']
                        if (r['instance'], r['bb'], r['index']) == (0, 0, 4))

    def refused(self):
        with self.assertRaises(ValueError):
            source_text(self.graph)

    def test_healthy_generated_source_matches(self):
        self.assertEqual(source_text(self.graph),
                         (Path(__file__).parent/'candidates/Root.v').read_text())

    def test_wrong_destination(self):
        self.row['ir']['destination']['local'] = 2
        self.refused()

    def test_copy_instead_of_move(self):
        self.row['ir']['value']['operand']['op'] = 'copy'
        self.refused()

    def test_different_array_length(self):
        self.row['ir']['value']['operand']['place']['ty'] = '&mut [u8; 10]'
        self.refused()

    def test_immutable_destination(self):
        self.row['ir']['value']['to'] = '&[u8]'
        self.refused()

    def test_projected_source(self):
        self.row['ir']['value']['operand']['place']['projection'] = [{'op': 'deref'}]
        self.refused()

    def test_unknown_operation_field(self):
        self.row['ir']['value']['unchecked'] = True
        self.refused()

    def test_duplicate_site(self):
        self.graph['operations'].append(deepcopy(self.row))
        self.refused()

    def test_missing_site(self):
        self.graph['operations'].remove(self.row)
        self.refused()


if __name__ == '__main__':
    unittest.main()
