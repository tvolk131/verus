"""Adversarial tests for shared mechanics, not substitutes for proof replay."""

import copy
import inspect
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import workflow_runtime as runtime


def positive_report():
    return {'verification-results': dict(success=True, errors=0, verified=3,
        **{'encountered-error': False, 'encountered-vir-error': False, 'is-verifying-entire-crate': True}),
        'verus-infer': dict(crate_success=True, verification_scope={})}


class RuntimeTests(unittest.TestCase):
    def execution(self, root):
        source = root / 'input.txt'
        source.write_text('bound input')
        directory = root / 'run'
        argv = [sys.executable, '-B', '-c', 'import os; print(os.getcwd()); print(os.environ["TOKEN"])']
        environment = runtime.environment({'TOKEN': 'exact'}, base={})
        result = runtime.execute(directory, argv, root, environment, 10, inputs=[source])
        return directory, argv, environment, source, result

    def test_regular_artifacts_streaming_digest_and_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / 'bytes'
            path.write_bytes(b'bytes' * 300000)
            self.assertEqual(runtime.artifact(path)['sha256'], runtime.digest(path))
            alias = root / 'alias'
            alias.symlink_to(path)
            with self.assertRaisesRegex(ValueError, 'regular artifact'):
                runtime.artifact(alias)

    def test_typed_equality_and_command_options(self):
        self.assertFalse(runtime.identical({'value': [True]}, {'value': [1]}))
        expected = ['/verus', '/input.rs', '--no-cheating']
        for actual in ([], expected + ['--no-verify'], [*expected, '--import=x=y'],
                       ['/other', *expected[1:]], tuple(expected), [False, *expected[1:]]):
            with self.subTest(actual=actual), self.assertRaises(ValueError):
                runtime.validate_command(actual, expected)

    def test_environment_poison_is_removed_without_parent_mutation(self):
        inherited = dict(os.environ, GIT_CONFIG_COUNT='1', RUSTFLAGS='--cfg poison',
            CARGO_ENCODED_RUSTFLAGS='poison', VERUS_EXTRA_ARGS='--no-verify', VERUS_INFER_CACHE='old')
        before = inherited.copy()
        result = runtime.environment({'GIT_OPTIONAL_LOCKS': '0', 'CARGO_HOME': '/private/tmp/owned'}, base=inherited)
        self.assertEqual(inherited, before)
        self.assertFalse(any(key.startswith('GIT_') and key != 'GIT_OPTIONAL_LOCKS' for key in result))
        self.assertTrue(all(key not in result for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS',
                                                        'VERUS_EXTRA_ARGS', 'VERUS_INFER_CACHE')))
        self.assertEqual(runtime.environment({'ONLY': 'new'}, base={}), {'ONLY': 'new'})

    def test_duplicate_and_nonfinite_json_reject(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'report.json'
            for text in ('{"success":false,"success":true}', '{"value":NaN}', '{"value":Infinity}', '{'):
                with self.subTest(text=text), self.assertRaises(ValueError):
                    path.write_text(text)
                    runtime.read_json(path)

    def test_execution_binds_command_cwd_environment_inputs_and_streams(self):
        for mutation in ('command', 'cwd', 'environment', 'source', 'stdout', 'stderr', 'split', 'returncode', 'timeout'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                directory, argv, environment, source, result = self.execution(root)
                runtime.bind_execution(directory, argv, root, environment, inputs=[source])
                invocation = runtime.read_json(directory / 'invocation.json')
                if mutation == 'command':
                    invocation['argv'].append('--no-verify')
                elif mutation == 'cwd':
                    invocation['cwd'] = str(root / 'other')
                elif mutation == 'environment':
                    invocation['environment']['RUSTFLAGS'] = '--cfg poison'
                elif mutation == 'source':
                    source.write_text('different input')
                elif mutation in ('stdout', 'stderr'):
                    (directory / mutation).write_text('changed stream')
                elif mutation == 'split':
                    other = root / 'identical-but-unexecuted'
                    other.write_bytes((directory / 'stdout').read_bytes())
                    result['stdout'] = runtime.artifact(other)
                elif mutation == 'returncode':
                    result['returncode'] = False
                else:
                    result['timeout'] = True
                runtime.record(directory / 'invocation.json', invocation)
                runtime.record(directory / 'result.json', result)
                with self.assertRaises(ValueError):
                    runtime.bind_execution(directory, argv, root, environment, inputs=[source])

    def test_timeout_and_spawn_errors_are_logged_not_accepted(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name, argv, timeout in (('missing', [str(root / 'absent')], 1),
                    ('timeout', [sys.executable, '-B', '-c', 'import time; time.sleep(30)'], 0.01)):
                result = runtime.execute(root / name, argv, root, {}, timeout)
                self.assertTrue(result.get('spawn_error') or result['timeout'])
                with self.assertRaisesRegex(ValueError, 'incomplete execution'):
                    runtime.bind_execution(root / name, argv, root, {})

    def test_runtime_stage_requires_exact_files_and_no_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, destination = root / 'source', root / 'destination'
            source.mkdir()
            (source / 'verus').write_bytes(b'verus')
            (source / 'z3').write_bytes(b'z3')
            staged, seconds = runtime.stage_runtime(source, destination, ['verus', 'z3'])
            self.assertGreaterEqual(seconds, 0)
            runtime.runtime_artifacts(destination, ['verus', 'z3'], staged)
            (destination / 'verus').write_bytes(b'wrong')
            with self.assertRaisesRegex(ValueError, 'runtime binding'):
                runtime.runtime_artifacts(destination, ['verus', 'z3'], staged)
            (destination / 'verus').unlink()
            (destination / 'verus').symlink_to(source / 'verus')
            with self.assertRaisesRegex(ValueError, 'destination symlink'):
                runtime.stage_runtime(source, destination, ['verus', 'z3'])

    def test_outcome_schema_types_counts_flags_and_process_are_required(self):
        report = positive_report()
        runtime.validate_outcome(report, 0, True, verified=3, errors=0)
        for section, key, value in (('verification-results', 'success', 1),
                ('verification-results', 'errors', False), ('verification-results', 'verified', 2),
                ('verus-infer', 'crate_success', 1), ('verus-infer', 'verification_scope', [])):
            with self.subTest(key=key):
                changed = copy.deepcopy(report)
                changed[section][key] = value
                with self.assertRaises(ValueError):
                    runtime.validate_outcome(changed, 0, True, verified=3, errors=0)
        for code in (False, 1, None):
            with self.subTest(code=code), self.assertRaises(ValueError):
                runtime.validate_outcome(report, code, True)

    def test_positive_outcomes_require_zero_errors_without_expected_count(self):
        for options in ({}, {'errors': None}):
            runtime.validate_outcome(positive_report(), 0, True, **options)
            for count in (1, 2, 10):
                report = positive_report()
                report['verification-results']['errors'] = count
                with self.subTest(options=options, errors=count), self.assertRaisesRegex(ValueError, 'positive errors'):
                    runtime.validate_outcome(report, 0, True, **options)

    def test_positive_error_expectation_cannot_authorize_errors(self):
        report = positive_report()
        report['verification-results']['errors'] = 1
        with self.assertRaisesRegex(ValueError, 'positive errors'):
            runtime.validate_outcome(report, 0, True, errors=1)

    def test_monotonic_validation_timing_also_records_rejection(self):
        timings = {}
        with patch.object(runtime.time, 'monotonic', side_effect=[10.0, 10.25]):
            self.assertEqual(runtime.validate_timed(timings, lambda: 'accepted'), 'accepted')
        with patch.object(runtime.time, 'monotonic', side_effect=[30.0, 30.5]):
            with self.assertRaises(ValueError):
                runtime.validate_timed(timings, runtime.require, False, 'reject')
        self.assertEqual(timings, {'validation_seconds': 0.75})

    def test_isolated_guard_removals_are_killed_by_adversarial_tests(self):
        source = Path(runtime.__file__).read_text()
        cases = [
            ("require(identical(actual, expected), message)", 'pass',
             'test_typed_equality_and_command_options'),
            ("require(identical(result[stream], artifact(directory / stream)), 'unbound ' + stream)",
             'pass', 'test_execution_binds_command_cwd_environment_inputs_and_streams'),
            ("require(identical(invocation['environment'], environment), 'environment binding')",
             'pass', 'test_execution_binds_command_cwd_environment_inputs_and_streams'),
            ("require(identical(invocation.get('inputs'), [artifact(path) for path in inputs]), 'execution input drift')",
             'pass', 'test_execution_binds_command_cwd_environment_inputs_and_streams'),
            ("path.is_file() and not path.is_symlink()", 'path.is_file()',
             'test_regular_artifacts_streaming_digest_and_symlinks'),
            ("require(inference.get('crate_success') is positive, 'positive inference status: contradictory outcome' if positive else 'contradictory inference outcome')",
             'pass', 'test_outcome_schema_types_counts_flags_and_process_are_required'),
            ("require(not positive or result['errors'] == 0, 'positive errors')",
             'pass', 'test_positive_outcomes_require_zero_errors_without_expected_count'),
        ]
        for old, new, test in cases:
            with self.subTest(guard=old):
                self.assertEqual(source.count(old), 1)
                namespace = {'__file__': runtime.__file__}
                exec(compile(source.replace(old, new, 1), '<isolated-guard-removal>', 'exec'), namespace)
                replacements = {name: value for name, value in namespace.items()
                                if callable(value) and getattr(value, '__module__', None) is None}
                result = unittest.TestResult()
                with patch.multiple(runtime, **replacements):
                    RuntimeTests(test).run(result)
                self.assertFalse(result.errors, result.errors)
                self.assertTrue(result.failures, 'guard removal survived: ' + old)

    def test_consumers_kill_isolated_specialized_guard_removals(self):
        import check_fedimint_prefix_infer79 as fedimint
        from stateful_bitmap_workflow90 import run as stateful
        from test_fedimint_prefix_infer79 import PrefixInferAcceptanceTests
        from test_stateful_workflow90 import Workflow90

        cases = [
            (fedimint, 'scope', 'require(identical(actual, target), "full selected-body verification scope differs")',
             'pass', PrefixInferAcceptanceTests('test_all_scope_fields_have_exact_typed_identity')),
            (fedimint, 'failed', 'require(not artifact.exists(), "seeded inference-aware failed export retained")',
             'pass', PrefixInferAcceptanceTests('test_seeded_export_removal_is_independent_of_negative_reason')),
            (stateful, 'bound_run', "require(invocation['compiler_source_sha256'] == frozen['compiler_source_sha256'] == manifest, 'compiler manifest binding')",
             'pass', Workflow90('test_unbound_commands_sources_and_artifacts_reject')),
            (stateful, 'classify_bound', 'accepted = accepted and models_match',
             'accepted = accepted', Workflow90('test_model_and_import_oracles_are_not_learned_from_reports')),
        ]
        for module, function, old, new, test in cases:
            with self.subTest(module=module.__name__, guard=old):
                source = inspect.getsource(getattr(module, function))
                self.assertEqual(source.count(old), 1)
                replacements = {}
                exec(compile(source.replace(old, new, 1), '<isolated-consumer-guard-removal>', 'exec'),
                     module.__dict__, replacements)
                result = unittest.TestResult()
                with patch.object(module, function, replacements[function]):
                    test.run(result)
                self.assertFalse(result.errors, result.errors)
                self.assertTrue(result.failures, 'specialized guard removal survived: ' + old)


if __name__ == '__main__':
    unittest.main()
