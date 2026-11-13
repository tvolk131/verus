"""Synthetic gate checks; these tests confer no verifier proof credit."""

from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_redb_allocation as replay
import workflow_runtime as shared
from bitmap_stateful_acceptance import validate_positive_report


class AllocationTests(unittest.TestCase):
    def setUp(self):
        self.expected = replay.qualification()
        self.wanted = deepcopy(self.expected['expectations']['on'])
        self.document = {
            'verification-results': dict(success=True, **{'encountered-error': False,
                'encountered-vir-error': False, 'is-verifying-entire-crate': True},
                verified=self.wanted['verified'], errors=0),
            'verus-infer': dict(crate_success=True, interface_written=True, generation_failure=None,
                failed_functions=[], functions=[], requested_functions=0, generated_functions=0,
                imported_inference_dependencies=[], trusted_models=[], verification_scope=dict(
                    native_collection_completed=True, complete_selected_scope_verified=True,
                    native_functions=[dict(function=name, **row) for name, row in self.wanted['bodies'].items()],
                    checked_exec_bodies=19, checked_proof_bodies=26)),
        }

    def rejects_positive(self, mutation):
        changed = deepcopy(self.document)
        mutation(changed)
        with self.assertRaises(ValueError):
            validate_positive_report(changed, self.wanted)

    def negative(self):
        wanted = deepcopy(self.expected['expectations']['off'])
        document = deepcopy(self.document)
        document['verification-results'].update(success=False, verified=wanted['verified'], errors=wanted['errors'])
        document['verification-results']['encountered-error'] = True
        inference = document['verus-infer']
        inference.update(crate_success=False, interface_written=False, failed_functions=wanted['failed_functions'])
        inference['verification_scope'].update(complete_selected_scope_verified=False,
            checked_exec_bodies=0, checked_proof_bodies=0,
            native_functions=[dict(function=name, **row) for name, row in wanted['bodies'].items()])
        stderr = b'error: possible arithmetic underflow/overflow\n --> /input.rs:76:20\n\nerror: assertion failed\n --> /input.rs:77:24\n\nerror: aborting due to 2 previous errors\n'
        return document, wanted, stderr

    def test_fixed_qualification(self):
        self.assertEqual(len(replay.CASES), 11)
        self.assertEqual(self.expected['rust_methods']['total'], 32)
        self.assertEqual(self.expected['rust_methods']['unit'] + self.expected['rust_methods']['integration'], 32)
        self.assertEqual(set(self.expected['runtime']), {'host', 'target32'})

    def test_complete_positive(self):
        validate_positive_report(self.document, self.wanted)

    def test_positive_errors(self):
        self.rejects_positive(lambda doc: doc['verification-results'].update(errors=1))

    def test_positive_boolean_counts(self):
        self.rejects_positive(lambda doc: doc['verification-results'].update(errors=False))

    def test_missing_body(self):
        self.rejects_positive(lambda doc: doc['verus-infer']['verification_scope']['native_functions'].pop())

    def test_duplicate_body(self):
        self.rejects_positive(lambda doc: doc['verus-infer']['verification_scope']['native_functions'].append(
            doc['verus-infer']['verification_scope']['native_functions'][0]))

    def test_incomplete_scope(self):
        self.rejects_positive(lambda doc: doc['verus-infer']['verification_scope'].update(
            complete_selected_scope_verified=False))

    def test_contradictory_body_count(self):
        self.rejects_positive(lambda doc: doc['verus-infer']['verification_scope'].update(checked_exec_bodies=18))

    def test_changed_body_status(self):
        self.rejects_positive(lambda doc: doc['verus-infer']['verification_scope']['native_functions'][0].update(
            body_status='unverified_body_with_contract'))

    def test_native_derived_flag(self):
        self.rejects_positive(lambda doc: doc['verus-infer']['verification_scope']['native_functions'][0].update(
            automatically_derived=True))

    def test_negative_positive_contradiction(self):
        document, wanted, stderr = self.negative()
        document['verification-results']['success'] = True
        with self.assertRaises(ValueError):
            replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')

    def test_negative_exact_diagnostics(self):
        document, wanted, stderr = self.negative()
        replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')
        for changed in (stderr.replace(b'76:20', b'75:20'), stderr.replace(b'assertion failed', b'type mismatch'),
                        stderr + b'error: unrelated failure\n --> /input.rs:10:1\n',
                        stderr.replace(b'/input.rs', b'/unbound.rs')):
            with self.assertRaises(ValueError):
                replay.validate_negative(document, 1, wanted, changed, Path('/input.rs'), 'off')
        coded = b'error[E0308]: mismatched types\n --> /unbound/incidental.rs:1:1\n'
        for changed in (coded + stderr, stderr + coded,
                        stderr.replace(b'error: assertion failed', coded + b'error: assertion failed'),
                        stderr + b'unclassified trailing text\n',
                        b'unclassified preamble\n' + stderr,
                        stderr.replace(b'2 previous errors', b'3 previous errors'),
                        stderr.replace(b'error: aborting due to 2 previous errors\n', b''),
                        stderr + b'error: aborting due to 2 previous errors\n',
                        stderr.replace(b'error: assertion failed', b'error[OTHER]: assertion failed')):
            with self.subTest(diagnostic=changed), self.assertRaises(ValueError):
                replay.validate_negative(document, 1, wanted, changed, Path('/input.rs'), 'off')
        for prefix in ('', ' ', '  ', '\t', '\t ', '\u00a0'):
            for spelling in ('error: unrelated compiler failure', 'error[E0308]: mismatched types',
                             'error[OTHER]: unrelated failure', 'error : malformed header'):
                extra = (prefix + spelling + '\n --> /unbound/incidental.rs:1:1\n').encode()
                for changed in (extra + stderr, stderr + extra,
                                stderr.replace(b'error: assertion failed', extra + b'error: assertion failed')):
                    with self.subTest(prefix=prefix, spelling=spelling, diagnostic=changed), self.assertRaises(ValueError):
                        replay.validate_negative(document, 1, wanted, changed, Path('/input.rs'), 'off')

    def test_negative_missing_body(self):
        document, wanted, stderr = self.negative()
        document['verus-infer']['verification_scope']['native_functions'].pop()
        with self.assertRaises(ValueError):
            replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')

    def test_negative_frontend_not_credit(self):
        document, wanted, stderr = self.negative()
        document['verification-results']['encountered-vir-error'] = True
        with self.assertRaises(ValueError):
            replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')

    def test_negative_error_count(self):
        document, wanted, stderr = self.negative()
        document['verification-results']['errors'] += 1
        with self.assertRaises(ValueError):
            replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')

    def test_negative_count_types_and_required_null(self):
        for field in ('requested_functions', 'generated_functions'):
            for value in (False, True, 0.0, '0', None):
                document, wanted, stderr = self.negative()
                document['verus-infer'][field] = value
                with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                    replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')
            document, wanted, stderr = self.negative()
            del document['verus-infer'][field]
            with self.subTest(field=field, omitted=True), self.assertRaises(ValueError):
                replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')
        document, wanted, stderr = self.negative()
        del document['verus-infer']['generation_failure']
        with self.assertRaises(ValueError):
            replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')

    def test_negative_no_published_certificate(self):
        document, wanted, stderr = self.negative()
        document['verus-infer']['interface_written'] = True
        with self.assertRaises(ValueError):
            replay.validate_negative(document, 1, wanted, stderr, Path('/input.rs'), 'off')

    def test_launch_preflight_before_source_reads(self):
        with patch.object(replay, 'source_manifest') as source:
            with self.assertRaisesRegex(ValueError, 'insufficient launch allowance'):
                replay.preflight(None, None, {}, {}, 10, self.expected)
            source.assert_not_called()

    def test_command_binding(self):
        case = replay.CASES[1]
        command = replay.command(Path('/output'), case, Path('/runtime'), Path('/sysroot'))
        self.assertIn('--no-cheating', command)
        self.assertIn('--infer-goals', command)
        self.assertNotIn('--infer-goals', replay.command(Path('/output'), replay.CASES[2], Path('/runtime'), Path('/sysroot')))
        for changed in ([], command + ['--no-verify'], [part.replace('/sysroot', '/other') for part in command]):
            with self.assertRaises(ValueError):
                shared.validate_command(changed, command)

    def test_environment_and_consumed_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'input'
            source.write_text('checked bytes')
            execution = root / 'run'
            execution.mkdir()
            environment = replay.execution_environment(root, Path('/runtime'), Path('/sysroot'), 'on')
            shared.record(execution / 'invocation.json', dict(argv=['/runtime/verus'], cwd=str(root),
                environment=environment, inputs=[shared.artifact(source)]))
            for name in ('stdout', 'stderr'):
                (execution / name).write_text('{}' if name == 'stdout' else '')
            shared.record(execution / 'result.json', dict(returncode=0, timeout=False,
                stdout=shared.artifact(execution / 'stdout'), stderr=shared.artifact(execution / 'stderr')))
            shared.bind_execution(execution, ['/runtime/verus'], root, environment, inputs=[source])
            with self.assertRaises(ValueError):
                shared.bind_execution(execution, ['/runtime/verus'], root,
                    dict(environment, VERUS_Z3_PATH='/other'), inputs=[source])
            source.write_text('unbound replacement')
            with self.assertRaises(ValueError):
                shared.bind_execution(execution, ['/runtime/verus'], root, environment, inputs=[source])

    def test_duplicate_json_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'report'
            for content in (b'{"success":true,"success":false}', b'{"bad":NaN}',
                            b'{"bad":Infinity}', b'{"bad":-Infinity}', b'{', b'\xff'):
                path.write_bytes(content)
                with self.subTest(content=content), self.assertRaises(ValueError):
                    shared.read_json(path)

    def test_i386_object_required(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'library'
            path.write_bytes(b'!<arch>\n')
            with self.assertRaises(ValueError):
                replay.compiled_i386(path)

    def test_import_gate_has_no_backend_credit(self):
        document = {'verification-results': dict(success=False, **{'encountered-error': True,
            'encountered-vir-error': True}), 'verus-infer': dict(crate_success=False,
                interface_written=False, verification_scope=dict(native_collection_completed=False,
                complete_selected_scope_verified=False, checked_exec_bodies=0,
                checked_proof_bodies=0, native_functions=[]))}
        stderr = (b'error: could not open imported library file `/missing.vir`: No such file or directory (os error 2)\n\n'
                  b'error: aborting due to 1 previous error\n\n')
        replay.validate_missing_import(document, 1, stderr, Path('/missing.vir'))
        for field in ('checked_exec_bodies', 'checked_proof_bodies'):
            for value in (False, True, 0.0, '0', None):
                changed = deepcopy(document)
                changed['verus-infer']['verification_scope'][field] = value
                with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                    replay.validate_missing_import(changed, 1, stderr, Path('/missing.vir'))
            changed = deepcopy(document)
            del changed['verus-infer']['verification_scope'][field]
            with self.subTest(field=field, omitted=True), self.assertRaises(ValueError):
                replay.validate_missing_import(changed, 1, stderr, Path('/missing.vir'))
        for changed in (stderr.replace(b'/missing.vir', b'/unbound.vir'), stderr + b'error: other\n'):
            with self.assertRaises(ValueError):
                replay.validate_missing_import(document, 1, changed, Path('/missing.vir'))
        document['verification-results']['verified'] = 0
        with self.assertRaises(ValueError):
            replay.validate_missing_import(document, 1, stderr, Path('/missing.vir'))

    def test_fixture_ablation_and_measure(self):
        manual = (replay.DATA / 'fixtures/manual.rs').read_text()
        reduced = (replay.DATA / 'fixtures/reduced.rs').read_text()
        self.assertEqual(manual.replace(' ' + replay.BRIDGE, '', 1), reduced)
        start = manual.index('pub fn clear_all')
        self.assertEqual(manual[:start] + manual[start:].replace('decreases pages - index,', 'decreases 0u32,', 1),
                         (replay.DATA / 'fixtures/termination.rs').read_text())


if __name__ == '__main__':
    unittest.main()
