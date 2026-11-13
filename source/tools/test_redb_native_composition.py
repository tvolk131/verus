import copy
import json
import unittest
from pathlib import Path

import check_redb_native_composition as guard


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (ROOT / 'source/rust_verify_test/fixtures/infer/transfer_composition/redb_native.rs').read_text()
SCOPE = json.loads((ROOT / 'source/tools/fixtures/transfers/redb-native-composition-cycle1/scope.json').read_text())


def positive_report():
    suffixes = ('BtreeHeader::serialized_size', 'PageNumber::serialized_size', 'TransactionId::eq',
                'TransactionId::new', 'get_u64')
    checked = ('DatabaseError::from', 'DatabaseHeader::primary_slot', 'DatabaseHeader::secondary_slot',
        'DatabaseHeader::swap_primary_slot', 'TransactionId::new', 'TransactionId::cmp',
        'TransactionId::partial_cmp', 'TransactionId::eq', 'TransactionHeader::from_id_tail',
        'UnrepairedDatabaseHeader::assemble_physical_slots', 'UnrepairedDatabaseHeader::select_primary_slot',
        'UnrepairedDatabaseHeader::select_after_layout_context', 'get_u64', 'decoded_id_selection_client',
        'BtreeHeader::clone', 'DatabaseHeader::clone', 'PageNumber::clone', 'TransactionHeader::clone',
        'TransactionId::clone')
    return {'crate_success': True, 'functions': [{'function': 'guard_fixture::' + suffix,
        'mode': 'exact', 'required_checks_passed': True,
        'proofs': {name: {'required': True, 'status': 'proved'}
            for name in ('soundness', 'maximality', 'exactness')}} for suffix in suffixes],
        'imported_inference_dependencies': [], 'interface_written': True,
        'verification_scope': {'native_collection_completed': True, 'complete_selected_scope_verified': True,
            'checked_exec_bodies': 19, 'checked_proof_bodies': 0,
            'native_functions': [{'function': 'guard_fixture::' + suffix, 'body_status': 'checked_exec_body',
                'selected': True} for suffix in checked]}}


class NativeCompositionGuards(unittest.TestCase):
    def test_original_fixture_passes_each_source_layer_independently(self):
        guard.validate_bodies(SOURCE, SCOPE)
        guard.validate_constants(SOURCE, SCOPE)
        guard.validate_domains(SOURCE, SCOPE)
        guard.validate_records(SOURCE, SCOPE)
        guard.validate_scaffold(SOURCE, SCOPE)

    def test_named_native_body_guard_without_outer_digest_masking(self):
        changed = SOURCE.replace('u64::from_le_bytes(data[..size_of::<u64>()].try_into().unwrap())', '0')
        with self.assertRaisesRegex(guard.GuardError, '^native body: header::get_u64$'):
            guard.validate_bodies(changed, SCOPE)

    def test_named_exact_block_guard_without_outer_digest_masking(self):
        changed = SOURCE.replace('kept_primary && !layout_stale', 'kept_primary && layout_stale')
        with self.assertRaisesRegex(guard.GuardError, '^native body: finalize-selection-tail$'):
            guard.validate_bodies(changed, SCOPE)

    def test_named_const_dag_guard_without_body_guard_masking(self):
        changed = SOURCE.replace('const TRANSACTION_ID_OFFSET: usize = _UNUSED2_OFFSET + BtreeHeader::serialized_size();',
            'const TRANSACTION_ID_OFFSET: usize = 104;')
        with self.assertRaisesRegex(guard.GuardError, '^native constant: const TRANSACTION_ID_OFFSET$'):
            guard.validate_constants(changed, SCOPE)

    def test_named_original_domain_guard_without_scaffold_masking(self):
        changed = SOURCE.replace('requires data.len() >= TRANSACTION_LAST_FIELD,', 'requires data.len() >= TRANSACTION_SIZE,')
        with self.assertRaisesRegex(guard.GuardError, '^original domain: tail last-field length$'):
            guard.validate_domains(changed, SCOPE)

    def test_named_error_payload_guard_without_native_source_masking(self):
        changed = SOURCE.replace('Corrupted(String)', 'Corrupted(u8)')
        with self.assertRaisesRegex(guard.GuardError, '^native record/error payloads: StorageError$'):
            guard.validate_records(changed, SCOPE)

    def test_named_scaffold_guard_without_inner_guard_masking(self):
        with self.assertRaisesRegex(guard.GuardError, '^full fixture/scaffold identity$'):
            guard.validate_scaffold(SOURCE + '\n', SCOPE)

    def test_named_zero_import_guard_joined_separated_and_cross_wired(self):
        guard.validate_arguments(['--infer-goals', '--compile'])
        for arguments in (['--import', 'alien=x.vir'], ['--import=alien=x.vir'],
            ['--extern', 'alien=x.rlib'], ['--extern=alien=x.rlib'],
            ['--import=alien=x.vir', '--extern', 'other=x.rlib'],
            ['--extern=alien=x.rlib', '--import', 'other=x.vir']):
            with self.subTest(arguments=arguments):
                with self.assertRaisesRegex(guard.GuardError, '^zero application import cardinality$'):
                    guard.validate_arguments(arguments)

    def test_named_receipt_guards_have_no_incidental_error_masking(self):
        report = positive_report()
        guard.validate_receipt(report, True, True)
        changes = (
            ('crate verification result', lambda value: value.update(crate_success=False)),
            ('exact helper cardinality', lambda value: value['functions'].pop()),
            ('exact helper identity: BtreeHeader::serialized_size', lambda value: value['functions'][0].update(function='wrong')),
            ('fresh exact helper checks', lambda value: value['functions'][0].update(required_checks_passed=False)),
            ('fresh soundness', lambda value: value['functions'][0]['proofs']['soundness'].update(status='skipped')),
            ('zero imported inference dependencies', lambda value: value.update(imported_inference_dependencies=['alien'])),
            ('Infer-aware publication', lambda value: value.update(interface_written=False)),
            ('whole selected scope', lambda value: value['verification_scope'].update(complete_selected_scope_verified=False)),
            ('selected caller/body: DatabaseError::from', lambda value: value['verification_scope']['native_functions'][0].update(selected=False)),
        )
        for name, change in changes:
            with self.subTest(guard=name):
                modified = copy.deepcopy(report)
                change(modified)
                with self.assertRaises(guard.GuardError) as error:
                    guard.validate_receipt(modified, True, True)
                self.assertEqual(str(error.exception), name)

    def test_every_native_body_is_required_even_with_consistent_reduced_counts(self):
        report = positive_report()
        for row in report['verification_scope']['native_functions']:
            with self.subTest(function=row['function']):
                changed = copy.deepcopy(report)
                scope = changed['verification_scope']
                scope['native_functions'].remove(row)
                scope['checked_exec_bodies'] -= 1
                suffix = row['function'].removeprefix('guard_fixture::')
                with self.assertRaisesRegex(guard.GuardError, '^selected caller/body: ' + suffix + '$'):
                    guard.validate_receipt(changed, True, True)

    def test_body_counts_are_typed_present_and_recomputed(self):
        for field, valid in (('checked_exec_bodies', 19), ('checked_proof_bodies', 0)):
            for value in (None, False, True, str(valid), float(valid), -1, valid + 1):
                with self.subTest(field=field, value=value):
                    changed = positive_report()
                    changed['verification_scope'][field] = value
                    with self.assertRaisesRegex(guard.GuardError, '^checked body count: ' + field + '$'):
                        guard.validate_receipt(changed, True, True)
            changed = positive_report()
            del changed['verification_scope'][field]
            with self.assertRaisesRegex(guard.GuardError, '^checked body count: ' + field + '$'):
                guard.validate_receipt(changed, True, True)

    def test_original_contradictory_counts_are_rejected(self):
        changed = positive_report()
        changed['verification_scope'].update(checked_exec_bodies=0, checked_proof_bodies=True)
        with self.assertRaisesRegex(guard.GuardError, '^checked body count: checked_exec_bodies$'):
            guard.validate_receipt(changed, True, True)

    def test_native_body_classification_and_selection_are_bound(self):
        for field, values, diagnostic in (
                ('selected', (None, 0, 1, 'true'), 'native body metadata'),
                ('selected', (False,), 'selected caller/body: TransactionHeader::clone'),
                ('body_status', ('outside_selected_scope', 'checked_proof_body'),
                    'selected caller/body: TransactionHeader::clone'),
                ('function', ('alien::TransactionHeader::clone',),
                    'selected caller/body: TransactionHeader::clone')):
            for value in values:
                with self.subTest(field=field, value=value):
                    changed = positive_report()
                    row = next(row for row in changed['verification_scope']['native_functions']
                        if row['function'].endswith('TransactionHeader::clone'))
                    row[field] = value
                    with self.assertRaisesRegex(guard.GuardError, '^' + diagnostic + '$'):
                        guard.validate_receipt(changed, True, True)

    def test_native_body_duplicates_and_undeclared_checked_bodies_reject(self):
        changed = positive_report()
        changed['verification_scope']['native_functions'].append(
            copy.deepcopy(changed['verification_scope']['native_functions'][0]))
        changed['verification_scope']['checked_exec_bodies'] += 1
        with self.assertRaisesRegex(guard.GuardError, '^unique native body identities$'):
            guard.validate_receipt(changed, True, True)
        for status, counter in (('checked_exec_body', 'checked_exec_bodies'),
                                ('checked_proof_body', 'checked_proof_bodies')):
            changed = positive_report()
            changed['verification_scope']['native_functions'].append(
                {'function': 'guard_fixture::unexpected', 'body_status': status, 'selected': True})
            changed['verification_scope'][counter] += 1
            with self.assertRaisesRegex(guard.GuardError, '^complete checked body set$'):
                guard.validate_receipt(changed, True, True)

    def test_boundary_bodies_are_explicit_not_inferred_from_the_report(self):
        boundary_bodies = ('original_leaf_domain', 'original_clear_short_domain', 'original_corrupt_long_domain',
            'invalid_index_two_phase', 'invalid_index_corrupt_early', 'total_swap_domain')
        report = positive_report()
        scope = report['verification_scope']
        scope['native_functions'] += [{'function': 'guard_fixture::' + suffix,
            'body_status': 'checked_exec_body', 'selected': True} for suffix in boundary_bodies]
        scope['checked_exec_bodies'] = 25
        guard.validate_receipt(report, True, True, additional_bodies=boundary_bodies)
        with self.assertRaisesRegex(guard.GuardError, '^complete checked body set$'):
            guard.validate_receipt(report, True, True)
        with self.assertRaisesRegex(guard.GuardError, '^declared additional body scope$'):
            guard.validate_receipt(report, True, True, additional_bodies=('unexpected',))
        undeclared = positive_report()
        undeclared['verification_scope']['native_functions'].append(
            {'function': 'guard_fixture::unexpected', 'body_status': 'checked_exec_body', 'selected': True})
        undeclared['verification_scope']['checked_exec_bodies'] = 20
        with self.assertRaisesRegex(guard.GuardError, '^declared additional body scope$'):
            guard.validate_receipt(undeclared, True, True, additional_bodies=('unexpected',))
        for suffix in boundary_bodies:
            changed = copy.deepcopy(report)
            changed['verification_scope']['native_functions'] = [row for row in scope['native_functions']
                if row['function'] != 'guard_fixture::' + suffix]
            changed['verification_scope']['checked_exec_bodies'] = 24
            with self.assertRaisesRegex(guard.GuardError, '^selected caller/body: ' + suffix + '$'):
                guard.validate_receipt(changed, True, True, additional_bodies=boundary_bodies)

    def test_unchecked_spec_entries_do_not_inflate_checked_counts(self):
        report = positive_report()
        report['verification_scope']['native_functions'] += [
            {'function': 'guard_fixture::specification', 'selected': True,
                'body_status': 'specification_definition_or_declaration'},
            {'function': 'guard_fixture::outside', 'selected': False, 'body_status': 'outside_selected_scope'}]
        guard.validate_receipt(report, True, True)
        report['verification_scope']['native_functions'].reverse()
        guard.validate_receipt(report, True, True)

    def test_unchecked_entries_still_have_typed_metadata(self):
        for field, value in (('selected', 1), ('function', None), ('body_status', None)):
            changed = positive_report()
            row = {'function': 'guard_fixture::outside', 'selected': False, 'body_status': 'outside_selected_scope'}
            row[field] = value
            changed['verification_scope']['native_functions'].append(row)
            with self.assertRaisesRegex(guard.GuardError, '^native body metadata$'):
                guard.validate_receipt(changed, True, True)


if __name__ == '__main__':
    unittest.main()
