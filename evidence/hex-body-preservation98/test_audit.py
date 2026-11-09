"""Synthetic AUDIT/METADATA tests; never native semantic negatives."""

import copy
import json
import unittest

import audit


class MetadataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.original, cls.final = audit.source_trees(audit.ARCHIVE, audit.INPUT / 'native')
        cls.historical = audit.read_json(audit.INPUT / 'historical-v3-baseline.json')['inference']['verification_scope']
        cls.current = audit.read_json(audit.INPUT / 'invocation-03/stdout')['verus-infer']['verification_scope']

    def setUp(self):
        self.old = copy.deepcopy(self.historical)
        self.new = copy.deepcopy(self.current)

    def compare(self, original=None, final=None):
        return audit.compare_scopes(self.old, self.new,
                                    self.original if original is None else original,
                                    self.final if final is None else final,
                                    str(audit.INPUT / 'native'))

    def overload_indices(self):
        return [index for index, row in enumerate(self.new['native_functions'])
                if row['body_status'] == 'checked_exec_body' and row['rust_function'] in audit.OVERLOADS]

    def test_AUDIT_METADATA_frozen_positive_all_32_both_overloads(self):
        result = self.compare()
        self.assertEqual(result['matched_original_rows'], 32)
        self.assertEqual(len(result['both_overloads']), 2)
        self.assertEqual({row['original_span'][0] for row in result['both_overloads']}, {150, 157})

    def test_AUDIT_METADATA_remove_overload_duplicate_sibling_same_count_and_labels(self):
        for reverse in (False, True):
            with self.subTest(reverse=reverse):
                self.new = copy.deepcopy(self.current)
                first, second = self.overload_indices()[::(-1 if reverse else 1)]
                before = len(self.new['native_functions'])
                labels = {row['function'] for row in self.new['native_functions'] if row['body_status'] == 'checked_exec_body'}
                self.new['native_functions'][second] = copy.deepcopy(self.new['native_functions'][first])
                self.assertEqual(len(self.new['native_functions']), before)
                self.assertEqual({row['function'] for row in self.new['native_functions']
                                  if row['body_status'] == 'checked_exec_body'}, labels)
                with self.assertRaisesRegex(audit.AuditError, 'duplicate identity/multiplicity'):
                    self.compare()

    def test_AUDIT_METADATA_reordered_rows_positive_multiset(self):
        self.old['native_functions'].reverse()
        self.new['native_functions'].reverse()
        self.assertEqual(self.compare()['matched_original_rows'], 32)

    def test_AUDIT_METADATA_swap_overload_spans_same_counts_rejected(self):
        first, second = [self.new['native_functions'][index] for index in self.overload_indices()]
        first['source'], second['source'] = second['source'], first['source']
        with self.assertRaisesRegex(audit.AuditError, 'missing historical identity'):
            self.compare()

    def test_AUDIT_METADATA_changed_implementation_same_function_label(self):
        row = self.new['native_functions'][self.overload_indices()[0]]
        row['rust_function'] = row['rust_function'].replace('InvalidCharError', 'AlienError')
        with self.assertRaisesRegex(audit.AuditError, 'missing historical identity'):
            self.compare()

    def test_AUDIT_METADATA_duplicate_implementation_with_distinct_span(self):
        first, second = self.overload_indices()
        self.new['native_functions'][second]['rust_function'] = self.new['native_functions'][first]['rust_function']
        with self.assertRaisesRegex(audit.AuditError, 'duplicate implementation/multiplicity'):
            self.compare()

    def test_AUDIT_METADATA_duplicate_historical_identity(self):
        indices = [index for index, row in enumerate(self.old['native_functions']) if row['rust_function'] in audit.OVERLOADS]
        self.old['native_functions'][indices[1]] = copy.deepcopy(self.old['native_functions'][indices[0]])
        with self.assertRaisesRegex(audit.AuditError, 'duplicate identity/multiplicity'):
            self.compare()

    def test_AUDIT_METADATA_altered_span(self):
        row = self.new['native_functions'][self.overload_indices()[0]]
        row['source'] = row['source'].replace(':150:5: 150:41', ':150:6: 150:41')
        with self.assertRaisesRegex(audit.AuditError, 'missing historical identity'):
            self.compare()

    def test_AUDIT_METADATA_altered_file(self):
        row = self.new['native_functions'][self.overload_indices()[0]]
        row['source'] = row['source'].replace('src/error.rs', 'src/iter.rs')
        with self.assertRaises(audit.AuditError):
            self.compare()

    def test_AUDIT_METADATA_altered_current_root(self):
        row = self.new['native_functions'][self.overload_indices()[0]]
        row['source'] = row['source'].replace(str(audit.INPUT / 'native'), '/alien/native')
        with self.assertRaisesRegex(audit.AuditError, 'source root mismatch'):
            self.compare()

    def test_AUDIT_METADATA_altered_historical_root(self):
        row = next(row for row in self.old['native_functions'] if row['rust_function'] in audit.OVERLOADS)
        row['source'] = row['source'].replace(audit.HISTORICAL_ROOT, audit.HISTORICAL_ROOT + '-sibling')
        with self.assertRaisesRegex(audit.AuditError, 'source root mismatch'):
            self.compare()

    def test_AUDIT_METADATA_noncanonical_path(self):
        row = self.new['native_functions'][self.overload_indices()[0]]
        row['source'] = row['source'].replace('/src/error.rs', '/src/../src/error.rs')
        with self.assertRaisesRegex(audit.AuditError, 'noncanonical relative'):
            self.compare()

    def test_AUDIT_METADATA_missing_identity_fields(self):
        for field in ('source', 'rust_function', 'function', 'body_status', 'selected'):
            with self.subTest(field=field):
                self.new = copy.deepcopy(self.current)
                del self.new['native_functions'][self.overload_indices()[0]][field]
                with self.assertRaisesRegex(audit.AuditError, 'missing'):
                    self.compare()

    def test_AUDIT_METADATA_wrong_scope_counts(self):
        for target, field, value in [('old', 'checked_exec_bodies', 31), ('new', 'checked_exec_bodies', 35),
                                     ('new', 'checked_proof_bodies', 0)]:
            with self.subTest(target=target, field=field):
                self.old = copy.deepcopy(self.historical)
                self.new = copy.deepcopy(self.current)
                getattr(self, target)[field] = value
                with self.assertRaisesRegex(audit.AuditError, 'wrong declared'):
                    self.compare()

    def test_AUDIT_METADATA_extra_row_wrong_actual_count(self):
        self.new['native_functions'].append(copy.deepcopy(self.new['native_functions'][self.overload_indices()[0]]))
        with self.assertRaisesRegex(audit.AuditError, 'wrong actual scope counts'):
            self.compare()

    def test_AUDIT_METADATA_mapping_failure_edited_lib(self):
        final = dict(self.final)
        final['src/lib.rs'] = final['src/lib.rs'].replace(b'pub enum Case {', b'pub enum AlteredCase {', 1)
        with self.assertRaisesRegex(audit.AuditError, 'unsupported edit'):
            self.compare(final=final)

    def test_AUDIT_METADATA_other_archived_file_modified(self):
        final = dict(self.final)
        final['src/error.rs'] += b'\n'
        with self.assertRaisesRegex(audit.AuditError, 'archived file modified'):
            self.compare(final=final)

    def test_AUDIT_METADATA_unexpected_source_file(self):
        final = dict(self.final)
        final['src/unreviewed.rs'] = b''
        with self.assertRaisesRegex(audit.AuditError, 'unexpected added/removed'):
            self.compare(final=final)

    def test_AUDIT_METADATA_insertion_mapping_positive(self):
        mapping = audit.LineMap(b'header\nfn original() {}\nend\n', b'header\ninserted\nfn original() {}\nend\n')
        self.assertEqual(mapping.map_span((2, 1, 2, 17)), (3, 1, 3, 17))
        frozen = self.compare()
        changed = [row for row in frozen['matches'] if row['source_relative_file'] == 'src/lib.rs']
        self.assertTrue(changed)
        self.assertTrue(all(row['normalized_current_span'][0] > row['original_span'][0] for row in changed))

    def test_AUDIT_METADATA_ambiguous_mapping_rejected(self):
        mapping = audit.LineMap(b'same\nend\n', b'same\nsame\nend\n')
        with self.assertRaisesRegex(audit.AuditError, 'ambiguous span identity'):
            mapping.map_span((1, 1, 1, 5))

    def test_AUDIT_METADATA_insertion_inside_multiline_span_rejected(self):
        mapping = audit.LineMap(b'first\nlast\n', b'first\nnew\nlast\n')
        with self.assertRaisesRegex(audit.AuditError, 'unmappable span'):
            mapping.map_span((1, 1, 2, 5))

    def test_AUDIT_METADATA_out_of_bounds_span(self):
        row = self.new['native_functions'][self.overload_indices()[0]]
        row['source'] = row['source'].replace(':150:5: 150:41', ':99999:5: 99999:41')
        with self.assertRaisesRegex(audit.AuditError, 'out of bounds'):
            self.compare()

    def test_AUDIT_METADATA_duplicate_json_keys(self):
        with self.assertRaisesRegex(audit.AuditError, 'duplicate JSON key'):
            json.loads('{"rust_function":"first","rust_function":"second"}', object_pairs_hook=audit.unique_object)


if __name__ == '__main__':
    unittest.main(verbosity=2)
