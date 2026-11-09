import hashlib
import copy
import json
from pathlib import Path
import tempfile
import unittest
import sys
from unittest.mock import patch

from stateful_bitmap_workflow90 import audit
from stateful_bitmap_workflow90 import run
from check_redb_stateful import RUNTIME_FILES

class Workflow90(unittest.TestCase):
    def test_reference_orchestration_uses_shared_logged_executor_and_budgets(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            run.record(root / 'launch-counts.json', dict(build=0, vstd=0, application=0, test=0))
            runner = run.reference_runner(root, root / 'seed/cargo-home')
            with patch.object(run, 'source_manifest', return_value={'compiler': 'frozen'}), \
                    patch.object(run, 'full_source_manifest', return_value={'all': 'frozen'}):
                result = runner.execute('synthetic', [sys.executable, '-B', '-c', 'print("smoke")'],
                                        'test', {'PYTHONDONTWRITEBYTECODE': '1'}, 10)
            self.assertEqual(result['returncode'], 0)
            self.assertGreaterEqual(result['seconds'], 0)
            invocation = json.loads((root / 'runs/synthetic/invocation.json').read_text())
            self.assertEqual(invocation['category'], 'test')
            self.assertEqual(invocation['source_sha256'], run.identity({'all': 'frozen'}))
            self.assertEqual((root / 'runs/synthetic/stdout').read_text(), 'smoke\n')
            run.record(root / 'launch-counts.json', dict(build=1, vstd=1, application=64, test=100))
            with self.assertRaisesRegex(ValueError, 'budget exhausted'):
                runner.execute('not-launched', [], 'test', {}, 10)
            self.assertFalse((root / 'runs/not-launched').exists())

    def test_exact_frozen_inputs(self):
        for name, expected in audit.HASHES.items():
            path = audit.HERE / 'fixtures' / (name + '.rs')
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), expected)

    def test_only_frozen_point_instances_are_removed(self):
        native = (audit.HERE / 'fixtures/native_manual.rs').read_text()
        reduced = (audit.HERE / 'fixtures/native_ablated.rs').read_text()
        removed = '''                    assert(live(before, query) ==
                        (if old_len <= query < new_len && query < 64 * word as int { full }
                         else if query < 64 * entry.len() { live(entry, query) } else { full }));
'''
        self.assertIn(removed, native)
        self.assertEqual(native.replace(removed, '', 1), reduced)
        tree = (audit.HERE / 'fixtures/tree_manual.rs').read_text()
        tree_reduced = (audit.HERE / 'fixtures/tree_ablated.rs').read_text()
        removed = '''                        assert(layer(before, query) ==
                            expected_layer(entry_pages as int, entry_capacity as int, query));
'''
        self.assertIn(removed, tree)
        self.assertEqual(tree.replace(removed, '', 1), tree_reduced)

    def test_unknown_loop_erasure_is_rejected(self):
        with self.assertRaises(ValueError):
            audit.erase_tree(['loop', 'invariant_except_break', 'true', '{', '}'])

    def test_guarded_fixture_drift_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            for name in audit.HASHES:
                source = audit.HERE / 'fixtures' / (name + '.rs')
                (Path(directory) / source.name).write_bytes(source.read_bytes() + b'\n')
            with self.assertRaisesRegex(ValueError, 'frozen fixture drift'):
                audit.audit(__file__, Path(directory))

    def test_both_current_executable_runtime_paths_and_solver_are_preflighted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            runtime = root / 'runtime'
            runtime.mkdir()
            for name in RUNTIME_FILES:
                (runtime / name).write_text(name)
            executable = root / 'build/target/release/deps/test-binary'
            executable.parent.mkdir(parents=True)
            executable.write_text('test executable')
            executable.chmod(0o700)
            solver = root / 'source-z3'
            solver.write_text('z3')
            receipt = run.stage(runtime, executable, solver, root / 'receipt.json')
            executable_hash = run.digest(executable)
            self.assertEqual(set(receipt['destinations']), {
                str(root / 'build/target/release'), str(root / 'build/target-verus/release')})
            run.preflight(receipt, runtime, executable, executable_hash)
            (root / 'build/target-verus/release/libverus_builtin.rlib').write_text('stale')
            with self.assertRaisesRegex(ValueError, 'staged runtime drift'):
                run.preflight(receipt, runtime, executable, executable_hash)
            solver.write_text('other solver')
            with self.assertRaisesRegex(ValueError, 'source/z3 mismatch'):
                run.stage(runtime, executable, solver, root / 'other-receipt.json')

    def bound_case(self, root, positive, fixture='tree_ablated'):
        root = root.resolve()
        name = fixture + ('_on' if positive else '_off')
        application = root / 'applications' / name
        directory = root / 'runs' / name
        application.mkdir(parents=True)
        directory.mkdir(parents=True)
        runtime = root / 'runtime'
        runtime.mkdir()
        for filename in RUNTIME_FILES:
            (runtime / filename).write_text(filename)
        source = application / 'input.rs'
        original = audit.HERE / 'fixtures' / (fixture + '.rs')
        source.write_bytes(original.read_bytes())
        interface = application / (fixture + '.vir')
        library = application / ('lib' + fixture + '.rlib')
        manifest = run.identity({'compiler': 'frozen'})
        environment = run.canonical_environment(root, manifest)
        run.record(root / 'setup.json', dict(environment=environment))
        run.record(application / 'frozen-expectation.json', dict(input=run.artifact(source),
            original=run.artifact(original), imports=[], compiler_source_sha256=manifest,
            runtime={filename: run.artifact(runtime / filename) for filename in RUNTIME_FILES}))
        argv = list(map(str, [run.RUSTUP, 'run', run.TOOLCHAIN, runtime / 'verus', source,
            '--sysroot', run.SYSROOT, '--crate-type=lib', '--edition=2021', '--crate-name', fixture,
            '--no-cheating', '--compile', '--output-json', '--num-threads', '1', '--export',
            interface, '-o', library]))
        if positive:
            argv.append('--infer-goals')
        invocation = dict(argv=argv, cwd=str(run.REPO / 'source'), category='application',
                          compiler_source_sha256=manifest, environment=environment)
        run.record(directory / 'invocation.json', invocation)
        expected = json.loads((run.HERE / 'expected.json').read_text())['native' if fixture.startswith('native') else 'tree']
        case = json.loads((run.HERE / 'expected.json').read_text())['cases'][fixture]
        items = expected['selected_items'].copy()
        if not positive:
            items.update({function: 'selected_body_not_certified_by_successful_full_run'
                          for function in expected['checked_bodies']})
        report = {'verification-results': dict(success=positive, errors=0 if positive else 1,
            verified=expected['baseline_manual_verified' if positive else 'baseline_reduced_verified'],
            **{'encountered-error': not positive, 'encountered-vir-error': False,
               'is-verifying-entire-crate': True}),
            'verus-infer': dict(schema_version=1, crate_success=positive,
                status='no_inference' if positive else 'caller_proof_failed',
                functions=[], generation_failure=None,
                failed_functions=[] if positive else case['failed_functions'],
                interface_written=positive, requested_functions=0, generated_functions=0,
                trusted_models=[], imported_inference_dependencies=[],
                verification_scope=dict(native_collection_completed=True,
                    complete_selected_scope_verified=positive,
                    checked_exec_bodies=expected['checked_exec_bodies'] if positive else 0,
                    checked_proof_bodies=expected['checked_proof_bodies'] if positive else 0,
                    native_functions=[dict(rust_function=function, body_status=status, selected=True,
                        **json.loads(json.dumps(case['selected_identities'][function]).replace('{input}', str(source))))
                        for function, status in items.items()]))}
        run.record(directory / 'stdout', report)
        (directory / 'stderr').write_text('' if positive else expected['negative_stderr'].replace('{input}', str(source)))
        result = dict(returncode=0 if positive else 1, timeout=False,
            stdout=run.artifact(directory / 'stdout'), stderr=run.artifact(directory / 'stderr'))
        run.record(directory / 'result.json', result)
        if positive:
            interface.write_bytes(b'checked interface')
            library.write_bytes(b'compiled library')
            run.record(application / 'published.json', dict(interface=run.artifact(interface),
                                                           library=run.artifact(library)))
        return name, directory, report

    def classify_fixture(self, root, name, positive):
        qualification = {'trusted_models_sha256': {'tree': run.identity([]), 'native': 'qualified_on_only'}}
        with patch.object(run, 'qualify_runtime', return_value=qualification), \
                patch.object(run, 'source_manifest', return_value={'compiler': 'frozen'}):
            return run.classify(root, name, positive)

    def rewrite_report(self, directory, report):
        run.record(directory / 'stdout', report)
        result = json.loads((directory / 'result.json').read_text())
        result['stdout'] = run.artifact(directory / 'stdout')
        run.record(directory / 'result.json', result)

    def test_duplicate_report_keys_reject_even_with_coherent_artifact_hash(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            name, directory, report = self.bound_case(root, True)
            self.assertTrue(self.classify_fixture(root, name, True)['matches_expected'])
            original = (directory / 'stdout').read_text()
            (directory / 'stdout').write_text('{"verification-results":{},' + original[1:])
            result = json.loads((directory / 'result.json').read_text())
            result['stdout'] = run.artifact(directory / 'stdout')
            run.record(directory / 'result.json', result)
            accepted = self.classify_fixture(root, name, True)
            self.assertFalse(accepted['matches_expected'])
            self.assertIn('duplicate JSON key', accepted['reason'])
            self.assertGreaterEqual(accepted['validation_seconds'], 0)

    def test_bound_positive_and_negative_controls(self):
        for positive in [True, False]:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, positive)
                self.assertTrue(self.classify_fixture(root, name, positive)['matches_expected'])

    def test_unbound_commands_sources_and_artifacts_reject(self):
        for mutation in ['missing', 'empty', 'cwd', 'environment', 'joined', 'source', 'stdout', 'library', 'compiler', 'runtime']:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, True)
                path = directory / 'invocation.json'
                invocation = json.loads(path.read_text())
                if mutation == 'missing':
                    path.unlink()
                elif mutation == 'source':
                    (root / 'applications' / name / 'input.rs').write_text('assert(false);')
                elif mutation == 'stdout':
                    report['unbound_payload'] = 'changed report bytes'
                    run.record(directory / 'stdout', report)
                elif mutation == 'library':
                    (root / 'applications' / name / 'libtree_ablated.rlib').unlink()
                elif mutation == 'runtime':
                    frozen_path = root / 'applications' / name / 'frozen-expectation.json'
                    frozen = json.loads(frozen_path.read_text())
                    frozen['runtime']['verus']['sha256'] = '0' * 64
                    run.record(frozen_path, frozen)
                else:
                    if mutation == 'empty':
                        invocation['argv'] = []
                    elif mutation == 'cwd':
                        invocation['cwd'] = '/unrelated'
                    elif mutation == 'environment':
                        invocation['environment']['RUSTFLAGS'] = 'unrecorded flags'
                    elif mutation == 'compiler':
                        invocation['compiler_source_sha256'] = '0' * 64
                    else:
                        invocation['argv'].append('--import=unrecorded=missing.vir')
                    run.record(path, invocation)
                self.assertFalse(self.classify_fixture(root, name, True)['matches_expected'])

    def test_complete_selected_inventory_and_counts_are_required(self):
        for mutation in ['extra', 'missing', 'duplicate', 'scope', 'body_status', 'exec_count', 'proof_count']:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, True)
                scope = report['verus-infer']['verification_scope']
                if mutation == 'extra':
                    scope['native_functions'].append(dict(rust_function='unchecked', selected=True,
                        body_status='selected_body_not_certified_by_successful_full_run'))
                elif mutation == 'missing':
                    scope['native_functions'].pop()
                elif mutation == 'duplicate':
                    scope['native_functions'].append(copy.deepcopy(scope['native_functions'][0]))
                elif mutation == 'scope':
                    del report['verus-infer']['verification_scope']
                elif mutation == 'body_status':
                    scope['native_functions'][0]['body_status'] = 'selected_body_not_certified_by_successful_full_run'
                else:
                    scope['checked_exec_bodies' if mutation == 'exec_count' else 'checked_proof_bodies'] += 1
                self.rewrite_report(directory, report)
                self.assertFalse(self.classify_fixture(root, name, True)['matches_expected'])

    def test_contradictory_reports_and_malformed_schema_reject(self):
        for positive in [True, False]:
            for mutation in ['success', 'vir_error', 'entire_crate', 'errors', 'verified', 'malformed']:
                with self.subTest(positive=positive, mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary).resolve()
                    name, directory, report = self.bound_case(root, positive)
                    if mutation == 'malformed':
                        report = []
                    else:
                        field = {'success': 'success', 'vir_error': 'encountered-vir-error',
                                 'entire_crate': 'is-verifying-entire-crate', 'errors': 'errors',
                                 'verified': 'verified'}[mutation]
                        report['verification-results'][field] = not report['verification-results'][field]
                    self.rewrite_report(directory, report)
                    self.assertFalse(self.classify_fixture(root, name, positive)['matches_expected'])

    def test_negative_diagnostic_must_bind_exact_source_expression(self):
        for diagnostic in ['error: assertion failed\n --> /not-the-input/input.rs:93:1\n',
                           'error: assertion failed\nassert(false);\n']:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, False)
                (directory / 'stderr').write_text(diagnostic)
                result = json.loads((directory / 'result.json').read_text())
                result['stderr'] = run.artifact(directory / 'stderr')
                run.record(directory / 'result.json', result)
                self.assertFalse(self.classify_fixture(root, name, False)['matches_expected'])

    def test_preflight_binds_executable_destinations_and_loader(self):
        for mutation in ['executable', 'destinations', 'partial_bundle', 'loader']:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                runtime = root / 'runtime'
                runtime.mkdir()
                for filename in RUNTIME_FILES:
                    (runtime / filename).write_text(filename)
                executable = root / 'build/target/release/deps/test'
                executable.parent.mkdir(parents=True)
                executable.write_text('executable')
                executable.chmod(0o700)
                receipt = run.stage(runtime, executable, runtime / 'z3', root / 'receipt.json')
                executable_hash = run.digest(executable)
                if mutation == 'executable':
                    executable.write_text('changed executable')
                elif mutation == 'destinations':
                    receipt['destinations'] = {}
                elif mutation == 'partial_bundle':
                    next(iter(receipt['destinations'].values())).pop('vstd.vir')
                else:
                    receipt['test_loader']['files'] = []
                with self.assertRaises(ValueError):
                    run.preflight(receipt, runtime, executable, executable_hash)

    def test_inference_outcome_and_inventory_must_be_coherent(self):
        for positive in (True, False):
            for field in ('schema_version', 'crate_success', 'status', 'functions',
                          'generation_failure', 'failed_functions', 'interface_written'):
                with self.subTest(positive=positive, field=field), tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary).resolve()
                    name, directory, report = self.bound_case(root, positive)
                    inference = report['verus-infer']
                    replacements = dict(schema_version=999, crate_success=not positive,
                        status='rejected', functions=[dict(function='unproved')],
                        generation_failure='unexpected generation error',
                        failed_functions=['unrelated::failed'], interface_written=not positive)
                    inference[field] = replacements[field]
                    self.rewrite_report(directory, report)
                    self.assertFalse(self.classify_fixture(root, name, positive)['matches_expected'])

    def test_selected_body_qualified_identity_and_source_are_bound(self):
        for field, value in [('function', 'unrelated::body'), ('source', '/not-the-input.rs:1:1'),
                             ('automatically_derived', True), ('inference_candidate', 0),
                             ('selected', False)]:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, True)
                report['verus-infer']['verification_scope']['native_functions'][0][field] = value
                self.rewrite_report(directory, report)
                self.assertFalse(self.classify_fixture(root, name, True)['matches_expected'])

    def test_positive_error_diagnostics_are_not_credited(self):
        for diagnostic in ['error: assertion failed\n', 'error: unsupported source operation\n',
                           'warning: unexpected output\n']:
            with self.subTest(diagnostic=diagnostic), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, True)
                (directory / 'stderr').write_text(diagnostic)
                result = json.loads((directory / 'result.json').read_text())
                result['stderr'] = run.artifact(directory / 'stderr')
                run.record(directory / 'result.json', result)
                self.assertFalse(self.classify_fixture(root, name, True)['matches_expected'])

    def test_coherently_changed_environments_are_not_self_authenticating(self):
        for field, value in [('PATH', '/unrelated/bin'), ('VERUS_EXTRA_ARGS', '--no-verify'),
                             ('UNEXPECTED_OPTION', '1'), ('VARGO_BUILD_SHA', 'unrelated-source')]:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, True)
                setup = json.loads((root / 'setup.json').read_text())
                invocation = json.loads((directory / 'invocation.json').read_text())
                for record in (setup, invocation):
                    record['environment'][field] = value
                run.record(root / 'setup.json', setup)
                run.record(directory / 'invocation.json', invocation)
                self.assertFalse(self.classify_fixture(root, name, True)['matches_expected'])

    def test_preflight_binds_both_binaries_to_independent_hashes(self):
        for label in ('infer_entry_goals', 'infer_loop_points'):
            for rehash in (False, True):
                with self.subTest(label=label, rehash=rehash), tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary).resolve()
                    runtime = root / 'runtime'
                    runtime.mkdir()
                    for filename in RUNTIME_FILES:
                        (runtime / filename).write_text(filename)
                    receipts, executables, hashes = {}, {}, {}
                    for binary in ('infer_entry_goals', 'infer_loop_points'):
                        executable = root / 'build/target/release/deps' / binary
                        executable.parent.mkdir(parents=True, exist_ok=True)
                        executable.write_text(binary)
                        executable.chmod(0o700)
                        executables[binary] = executable
                        hashes[binary] = run.digest(executable)
                        receipts[binary] = run.stage(runtime, executable, runtime / 'z3', root / (binary + '.json'))
                    run.preflight_all(receipts, runtime, executables, hashes)
                    executable = executables[label]
                    executable.write_text('changed binary')
                    if rehash:
                        receipts[label]['executable'] = run.artifact(executable)
                    with self.assertRaises(ValueError):
                        run.preflight_all(receipts, runtime, executables, hashes)

    def test_preflight_requires_both_harnesses(self):
        for missing in ('receipts', 'executables', 'expected_hashes'):
            with self.subTest(missing=missing):
                values = {name: dict(infer_entry_goals=None, infer_loop_points=None)
                          for name in ('receipts', 'executables', 'expected_hashes')}
                values[missing].pop('infer_loop_points')
                with self.assertRaisesRegex(ValueError, 'incomplete test executable inventory'):
                    run.preflight_all(values['receipts'], Path('/not-used'), values['executables'],
                                      values['expected_hashes'])

    def test_model_and_import_oracles_are_not_learned_from_reports(self):
        for mutation in ['model', 'dependency', 'boolean_count']:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                name, directory, report = self.bound_case(root, True)
                inference = report['verus-infer']
                if mutation == 'model':
                    inference['trusted_models'] = [dict(function='unapproved', fingerprint='0' * 64)]
                elif mutation == 'dependency':
                    inference['imported_inference_dependencies'] = [dict(crate_id=1, fingerprint='0' * 64)]
                else:
                    inference['requested_functions'] = False
                self.rewrite_report(directory, report)
                self.assertFalse(self.classify_fixture(root, name, True)['matches_expected'])

    def test_negative_success_exit_status_is_not_accepted(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            name, directory, report = self.bound_case(root, False)
            result = json.loads((directory / 'result.json').read_text())
            result['returncode'] = 0
            run.record(directory / 'result.json', result)
            self.assertFalse(self.classify_fixture(root, name, False)['matches_expected'])

    def test_original_model_oracle_is_retained_and_requalification_is_exact(self):
        expected = json.loads((run.HERE / 'expected.json').read_text())
        qualification = json.loads((run.HERE / 'qualification.json').read_text())
        self.assertEqual(expected['native']['trusted_models_sha256'],
                         '62713b5aaac6741035e5b8097d136853c9f44c759921a9e22d3fa58d89efa326')
        self.assertEqual(qualification['original_trusted_models_sha256'],
                         expected['native']['trusted_models_sha256'])
        self.assertEqual(qualification['candidate_vstd_sha256'], qualification['runtime']['vstd.vir'])
        self.assertEqual(qualification['identical_logical_payload_bytes'], 33347277)
        with patch.object(run, 'compiler_identity', return_value='changed compiler'):
            with self.assertRaisesRegex(ValueError, 'compiler source drift'):
                run.qualify_runtime(Path('/not-used'))

    def test_feature_off_requires_empty_goal_model_inventory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            name, directory, report = self.bound_case(root, False, 'native_ablated')
            self.assertTrue(self.classify_fixture(root, name, False)['matches_expected'])
            report['verus-infer']['trusted_models'] = [dict(function='unrequested', fingerprint='0' * 64)]
            self.rewrite_report(directory, report)
            self.assertFalse(self.classify_fixture(root, name, False)['matches_expected'])

if __name__ == '__main__':
    unittest.main()
