"""One bounded offline workflow, reusing reference88 execution and preservation."""

import argparse
import fcntl
import importlib.util
import json
import os
from pathlib import Path
import shlex
import shutil
import sys
import time
from types import SimpleNamespace

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from check_redb_stateful import NATIVE_FILE, RUNTIME_FILES, TOOLCHAIN, full_source_manifest, native_binding, source_manifest
from stateful_bitmap_workflow90.audit import HASHES, audit, require
from workflow_runtime import (artifact, bind_execution, digest, environment as shared_environment,
    execute, identity, read_json, record, report_sections, runtime_artifacts, stage_runtime,
    validate_command, validate_outcome, validate_timed)

REPO = HERE.parents[2]
SYSROOT = Path('/Users/tommyvolk/.rustup/toolchains') / TOOLCHAIN
RUSTUP = Path('/Users/tommyvolk/.cargo/bin/rustup')
TEST_ONLY_SOURCE = 'source/rust_verify_test/tests/infer_loop_points.rs'

def compiler_identity():
    return identity({name: value for name, value in source_manifest(REPO).items()
                     if name != TEST_ONLY_SOURCE})

def loader_binding():
    directories = [SYSROOT / 'lib', SYSROOT / 'lib/rustlib/aarch64-apple-darwin/lib']
    files = [artifact(path) for directory in directories for path in sorted(directory.glob('*.dylib'))]
    require(files, 'missing pinned loader libraries')
    return dict(DYLD_LIBRARY_PATH=':'.join(map(str, directories)), files=files)

def canonical_environment(output, manifest):
    build = output / 'build'
    return shared_environment(dict(
        HOME=str(build / 'home'), TMPDIR=str(build / 'tmp'), XDG_CACHE_HOME=str(build / 'cache'),
        CARGO_HOME=str(build / 'cargo-home'), CARGO_TARGET_DIR=str(build / 'target'),
        CARGO_BUILD_BUILD_DIR=str(build / 'target'), CARGO_BUILD_JOBS='2', CARGO_NET_OFFLINE='true',
        RUSTUP_HOME=str(build / 'rustup-home'), RUSTUP_TOOLCHAIN=TOOLCHAIN,
        RUSTC=str(SYSROOT / 'bin/rustc'), RUSTDOC=str(SYSROOT / 'bin/rustdoc'),
        RUSTC_BOOTSTRAP='1', RUST_MIN_STACK='20971520', VERUS_IN_VARGO='1',
        VARGO_BUILD_SHA='source-' + manifest, VARGO_BUILD_VERSION='stateful-source', VARGO_TOOLCHAIN=TOOLCHAIN,
        RUSTFLAGS='--sysroot ' + shlex.quote(str(SYSROOT)) +
                  ' --cfg proc_macro_span --cfg procmacro2_semver_exempt --cfg span_locations --cfg verus_keep_ghost',
        PATH=str(RUSTUP.parent) + ':/usr/bin:/bin:/usr/sbin:/sbin', GIT_OPTIONAL_LOCKS='0',
        GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM='1', PYTHONDONTWRITEBYTECODE='1',
        VERUS_Z3_PATH=str(output / 'runtime/z3'),
        VERUS_EXTRA_ARGS='--sysroot ' + shlex.quote(str(SYSROOT))), base={})

def qualify_runtime(runtime):
    qualification = json.loads((HERE / 'qualification.json').read_text())
    require(qualification['compiler_sources_sha256'] == compiler_identity(), 'compiler source drift')
    require(qualification['rustup'] == artifact(RUSTUP), 'rustup drift')
    require(qualification['rustc'] == artifact(SYSROOT / 'bin/rustc'), 'rustc drift')
    require(qualification['loader'] == loader_binding(), 'qualified loader drift')
    require(set(qualification['runtime']) == set(RUNTIME_FILES), 'incomplete qualified runtime')
    for name in RUNTIME_FILES:
        require(digest(runtime / name) == qualification['runtime'][name], 'unqualified runtime: ' + name)
    return qualification

def reference_runner(output, seed):
    specification = importlib.util.spec_from_file_location('workflow90_reference_runner', HERE.parent / 'stateful_bitmap_reference88/run.py')
    runner = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(runner)
    runner.OUTPUT = output
    runner.SEED = seed
    runner.SOLVER = seed.parents[1] / 'runtime/z3'
    def shared_execute(name, argv, category, environment, timeout):
        counts_path = output / 'launch-counts.json'
        with (output / 'launch-counts.lock').open('a+b') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            counts = read_json(counts_path)
            require(counts[category] < dict(build=1, vstd=1, application=64, test=100)[category],
                    'budget exhausted: ' + category)
            counts[category] += 1
            record(counts_path, counts)
        result = execute(output / 'runs' / name, argv, REPO / 'source', environment, timeout,
            metadata=dict(category=category, compiler_source_sha256=identity(source_manifest(REPO)),
                          source_sha256=identity(full_source_manifest(REPO))))
        print(json.dumps(dict(name=name, **result)), flush=True)
        return result
    runner.execute = shared_execute
    return runner

def stage(runtime, executable, source_solver, receipt):
    started = time.monotonic()
    executable = executable.resolve(strict=True)
    require(os.access(executable, os.X_OK), 'test executable not executable')
    profile = executable.parent.parent
    common = profile.parent.parent / 'target-verus' / profile.name
    require(digest(source_solver) == digest(runtime / 'z3'), 'InferCrates source/z3 mismatch')
    result = dict(executable=artifact(executable), source_solver=artifact(source_solver), destinations={},
                  test_loader=loader_binding())
    for destination in (profile, common):
        files, seconds = stage_runtime(runtime, destination, RUNTIME_FILES)
        result['destinations'][str(destination)] = files
    result['staging_seconds'] = time.monotonic() - started
    record(receipt, result)
    return result

def preflight(receipt, runtime, executable, expected_sha256):
    require(isinstance(receipt, dict), 'invalid preflight receipt')
    require(receipt['executable'] == artifact(executable), 'test executable drift')
    require(receipt['executable']['sha256'] == expected_sha256, 'unqualified test executable')
    profile = Path(executable).parent.parent
    expected_destinations = {str(profile), str(profile.parent.parent / 'target-verus' / profile.name)}
    require(set(receipt['destinations']) == expected_destinations, 'missing runtime destinations')
    for destination, files in receipt['destinations'].items():
        require(set(files) == set(RUNTIME_FILES), 'incomplete runtime bundle')
        for name, expected in files.items():
            require(expected['path'] == str(Path(destination) / name), 'runtime destination binding')
            require(artifact(expected['path']) == expected, 'staged runtime drift')
            require(expected['sha256'] == digest(runtime / name), 'source runtime drift')
    require(artifact(receipt['source_solver']['path']) == receipt['source_solver'], 'source solver drift')
    require(receipt['source_solver']['sha256'] == digest(runtime / 'z3'), 'source solver binding')
    require(receipt['test_loader'] == loader_binding(), 'pinned test loader library drift')

def preflight_all(receipts, runtime, executables, expected_hashes):
    labels = {'infer_entry_goals', 'infer_loop_points'}
    require(set(receipts) == set(executables) == set(expected_hashes) == labels,
            'incomplete test executable inventory')
    for label in sorted(labels):
        preflight(receipts[label], runtime, executables[label], expected_hashes[label])

def bound_run(output, name, positive):
    require(type(positive) is bool, 'invalid expectation')
    fixture, separator, mode = name.rpartition('_')
    require(separator and fixture in HASHES and mode in ('on', 'off'), 'unknown matrix case')
    require(positive == ('manual' in fixture or mode == 'on'), 'contradictory expectation')
    runtime = output / 'runtime'
    qualification = qualify_runtime(runtime)
    application = output / 'applications' / name
    directory = output / 'runs' / name
    source = application / 'input.rs'
    interface = application / (fixture + '.vir')
    library = application / ('lib' + fixture + '.rlib')
    frozen = json.loads((application / 'frozen-expectation.json').read_text())
    require(frozen['input'] == artifact(source) and digest(source) == HASHES[fixture], 'input binding')
    require(frozen['original'] == artifact(HERE / 'fixtures' / (fixture + '.rs')), 'fixture binding')
    require(frozen['imports'] == [], 'unexpected imports')
    runtime_artifacts(runtime, RUNTIME_FILES, frozen['runtime'])
    invocation = read_json(directory / 'invocation.json')
    expected_argv = list(map(str, [RUSTUP, 'run', TOOLCHAIN, runtime / 'verus', source,
        '--sysroot', SYSROOT, '--crate-type=lib', '--edition=2021', '--crate-name', fixture,
        '--no-cheating', '--compile', '--output-json', '--num-threads', '1',
        '--export', interface, '-o', library]))
    if mode == 'on':
        expected_argv.append('--infer-goals')
    validate_command(invocation['argv'], expected_argv)
    require(invocation['cwd'] == str(REPO / 'source') and invocation['category'] == 'application', 'invocation context')
    manifest = identity(source_manifest(REPO))
    require(invocation['compiler_source_sha256'] == frozen['compiler_source_sha256'] == manifest, 'compiler manifest binding')
    setup = json.loads((output / 'setup.json').read_text())
    environment = invocation['environment']
    require(environment == setup['environment'] == canonical_environment(output, manifest), 'environment binding')
    require(environment['VERUS_Z3_PATH'] == str(runtime / 'z3'), 'solver selection')
    require(environment['RUSTUP_TOOLCHAIN'] == TOOLCHAIN and environment['RUSTC'] == str(SYSROOT / 'bin/rustc'), 'toolchain selection')
    require(not any(key.startswith('DYLD') or key == 'VSTD_KIND' for key in environment), 'unexpected loader override')
    result = bind_execution(directory, expected_argv, REPO / 'source', canonical_environment(output, manifest))
    require(type(result['returncode']) is int and result['returncode'] == (0 if positive else 1), 'contradictory exit status')
    return fixture, source, interface, library, result, qualification

def classify(output, name, positive):
    timings = {}
    try:
        result = validate_timed(timings, classify_bound, Path(output).resolve(), name, positive)
    except (ValueError, KeyError, TypeError, AttributeError, OSError) as error:
        result = dict(status='invalid_evidence', expected_positive=positive, matches_expected=False,
                      reason=str(error))
    return dict(result, **timings)

def classify_bound(output, name, positive):
    fixture, source, interface, library, result, qualification = bound_run(output, name, positive)
    directory = output / 'runs' / name
    stderr = (directory / 'stderr').read_text()
    report = read_json(directory / 'stdout')
    verification, inference, scope = report_sections(report)
    expectations = json.loads((HERE / 'expected.json').read_text())
    expected = expectations['native' if name.startswith('native') else 'tree']
    case = expectations['cases'][fixture]
    require(type(inference['schema_version']) is int and inference['schema_version'] == 1,
            'inference schema version')
    require(inference['crate_success'] is positive
            and inference['status'] == ('no_inference' if positive else 'caller_proof_failed'),
            'contradictory inference outcome')
    require(inference['functions'] == [] and inference['generation_failure'] is None
            and inference['failed_functions'] == ([] if positive else case['failed_functions']),
            'contradictory inference function inventory')
    require(inference['interface_written'] is (positive and name.endswith('_on')),
            'contradictory interface status')
    rows = scope['native_functions']
    require(isinstance(rows, list) and all(isinstance(row, dict) and type(row.get('selected')) is bool for row in rows), 'invalid body inventory')
    require(all(all(type(row[field]) is bool for field in
                    ('automatically_derived', 'inference_candidate', 'ordinary_contract')) for row in rows),
            'invalid body flags')
    selected = [row for row in rows if row['selected']]
    require(len(selected) == len(rows), 'unexpected unselected body')
    identities = {row['rust_function']: {field: row[field] for field in
                  ('function', 'source', 'automatically_derived', 'inference_candidate', 'ordinary_contract')}
                  for row in selected}
    expected_identities = json.loads(json.dumps(case['selected_identities']).replace('{input}', str(source)))
    require(identities == expected_identities, 'selected body identity binding')
    bodies = {row['rust_function']: row['body_status'] for row in selected}
    require(len(bodies) == len(selected), 'duplicate selected body')
    expected_bodies = expected['selected_items'].copy()
    if not positive:
        expected_bodies.update({function: 'selected_body_not_certified_by_successful_full_run'
                                for function in expected['checked_bodies']})
    require(bodies == expected_bodies, 'incomplete or contradictory selected scope')
    require(scope['native_collection_completed'] is True, 'incomplete body collection')
    require(scope['complete_selected_scope_verified'] is positive, 'contradictory complete scope')
    require(type(scope['checked_exec_bodies']) is int and scope['checked_exec_bodies'] == (expected['checked_exec_bodies'] if positive else 0), 'executable body count')
    require(type(scope['checked_proof_bodies']) is int and scope['checked_proof_bodies'] == (expected['checked_proof_bodies'] if positive else 0), 'proof body count')
    validate_outcome(report, result['returncode'], positive, errors=0 if positive else 1,
                     verified=expected['baseline_manual_verified' if positive else 'baseline_reduced_verified'])
    if result.get('spawn_error'):
        status = 'setup_failure'
    elif result['timeout']:
        status = 'timeout'
    elif 'panicked' in stderr:
        status = 'panic'
    elif result['returncode'] == 0 and verification.get('success'):
        status = 'complete_success'
    elif 'assertion failed' in stderr and verification.get('errors', 0) > 0:
        status = 'backend_rejection'
    else:
        status = 'frontend_or_unclassified_failure'
    accepted = status == ('complete_success' if positive else 'backend_rejection')
    if positive:
        require(stderr == '', 'positive diagnostic failure')
        accepted = (accepted and verification.get('is-verifying-entire-crate') is True
                    and verification.get('encountered-error') is False
                    and verification.get('encountered-vir-error') is False
                    and scope.get('complete_selected_scope_verified') is True
                    and inference.get('requested_functions') == 0
                    and inference.get('generated_functions') == 0)
        published = json.loads((output / 'applications' / name / 'published.json').read_text())
        require(published == dict(interface=artifact(interface), library=artifact(library)), 'positive publication binding')
        require(interface.stat().st_size > 0 and library.stat().st_size > 0, 'empty positive artifact')
    else:
        accepted = (accepted and verification.get('errors') == 1
                    and stderr == expected['negative_stderr'].replace('{input}', str(source)))
        require(not interface.exists() and not library.exists(), 'unexpected failed publication')
        require(not (output / 'applications' / name / 'published.json').exists(), 'failed case published')
    require(type(inference['requested_functions']) is int and inference['requested_functions'] == 0
            and type(inference['generated_functions']) is int and inference['generated_functions'] == 0,
            'unexpected inferred functions')
    model_oracle = (qualification['trusted_models_sha256']['native' if fixture.startswith('native') else 'tree']
                    if name.endswith('_on') else identity([]))
    models_match = (identity(inference.get('trusted_models')) == model_oracle
                    and inference.get('imported_inference_dependencies') == [])
    accepted = accepted and models_match
    return dict(status=status, expected_positive=positive, matches_expected=accepted,
                verification=verification, verification_scope=scope,
                loop_point_preparation=inference.get('loop_point_preparation', []),
                requested=inference.get('requested_functions'), generated=inference.get('generated_functions'),
                trusted_models_sha256=identity(inference.get('trusted_models')), models_match_baseline=models_match,
                runtime_outputs={path.name: artifact(path) for path in (output / 'applications' / name).glob('*') if path.suffix in ('.vir', '.rlib')})

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--cargo-cache', type=Path, required=True)
    parser.add_argument('--runtime', type=Path)
    parser.add_argument('--test-executable', action='append', default=[])
    parser.add_argument('--test-manifest', type=Path)
    arguments = parser.parse_args()
    output = arguments.output.absolute()
    require(str(output).startswith(('/private/tmp/', '/tmp/')), 'private tmp output required')
    require(not output.exists(), 'fresh output directory required')
    native_path = arguments.native / NATIVE_FILE if arguments.native.is_dir() else arguments.native
    native = native_binding(native_path.resolve(strict=True).parents[3])
    projections = audit(native_path)
    runner = reference_runner(output, arguments.cargo_cache.resolve())
    if arguments.runtime:
        runner.SOLVER = arguments.runtime.resolve(strict=True) / 'z3'
    runner.initialize()
    record(output / 'native-source.json', native)
    record(output / 'native-projections.json', projections)
    environment = json.loads((output / 'setup.json').read_text())['environment']
    if arguments.runtime:
        qualify_runtime(arguments.runtime.resolve(strict=True))
        files, seconds = stage_runtime(arguments.runtime, output / 'runtime', RUNTIME_FILES)
        record(output / 'runtime-staging-timing.json', dict(staging_seconds=seconds))
        record(output / 'runtime-manifest.json',
               {filename: artifact(output / 'runtime' / filename) for filename in RUNTIME_FILES})
        executables = {}
        entries = arguments.test_executable
        if arguments.test_manifest:
            require(not entries, 'choose --test-manifest or --test-executable')
            manifest = json.loads(arguments.test_manifest.read_text())
            require(isinstance(manifest, dict) and all(isinstance(value, str) for value in manifest.values()), 'invalid test manifest')
            entries = [label + '=' + filename for label, filename in manifest.items()]
        for item in entries:
            label, filename = item.split('=', 1)
            require(label not in executables, 'duplicate test executable')
            executables[label] = Path(filename).resolve(strict=True)
        require(set(executables) == {'infer_entry_goals', 'infer_loop_points'}, 'both test harnesses required')
        record(output / 'reused-runtime.json', dict(runtime=str(arguments.runtime),
               compiler_builds=0, vstd_runs=0, qualification=artifact(HERE / 'qualification.json')))
    else:
        require(not arguments.test_executable and not arguments.test_manifest, 'test executables require --runtime')
        helper_command = [runner.RUSTUP, 'run', runner.TOOLCHAIN, runner.SYSROOT / 'bin/cargo', 'test', '--offline', '--locked', '--release', '-p', 'vir', '--lib', 'infer::entry_goals']
        result = runner.execute('typed-helpers', helper_command, 'test', environment, 1800)
        require(result['returncode'] == 0 and not result['timeout'], 'typed helper tests failed')
        runner.build(environment)
        command = [runner.RUSTUP, 'run', runner.TOOLCHAIN, runner.SYSROOT / 'bin/cargo', 'test', '--offline', '--locked', '--release', '-p', 'rust_verify_test', '--test', 'infer_entry_goals', '--test', 'infer_loop_points', '--no-run', '--message-format=json']
        result = runner.execute('harness-build', command, 'test', environment, 1800)
        require(result['returncode'] == 0 and not result['timeout'], 'test harness compilation failed')
        artifacts = [json.loads(line) for line in (output / 'runs/harness-build/stdout').read_text().splitlines() if line.startswith('{')]
        executables = {item['target']['name']: Path(item['executable']) for item in artifacts if item.get('reason') == 'compiler-artifact' and item.get('executable')}
    qualification = qualify_runtime(output / 'runtime')
    receipts = {label: stage(output / 'runtime', executable, runner.REPO / 'source/z3',
                output / ('stage-' + label + '-preflight.json'))
                for label, executable in executables.items()}
    record(output / 'stage-preflight.json', receipts)
    test_environment = dict(environment, DYLD_LIBRARY_PATH=loader_binding()['DYLD_LIBRARY_PATH'])
    matrix = [('native_ablated', True), ('tree_ablated', True), ('native_manual', True), ('tree_manual', True), ('native_ablated', False), ('tree_ablated', False), ('native_manual', False), ('tree_manual', False)]
    results = {}
    for fixture, enabled in matrix:
        preflight_all(receipts, output / 'runtime', executables, qualification['test_executables'])
        positive = 'manual' in fixture or enabled
        name = fixture + ('_on' if enabled else '_off')
        options = SimpleNamespace(name=name, fixture=HERE / 'fixtures' / (fixture + '.rs'), crate=fixture, expect='complete ordinary checked body' if positive else 'original authored point assertion fails', imports=[], negative=not positive, extra=['--infer-goals'] if enabled else [], timeout=240)
        runner.application(options, environment)
        results[name] = classify(output, name, positive)
        record(output / 'native-results.json', results)
        require(results[name]['status'] not in ('panic', 'timeout', 'setup_failure'), 'native stop gate: ' + name)
    for name, binary, test in [('common-smoke', 'infer_loop_points', 'common_runtime_smoke'), ('infercrates-smoke', 'infer_entry_goals', 'authored_forall_uses_only_original_entry_instances')]:
        preflight_all(receipts, output / 'runtime', executables, qualification['test_executables'])
        result = runner.execute(name, [executables[binary], test, '--exact', '--nocapture'], 'test', test_environment, 240)
        require(result['returncode'] == 0 and not result['timeout'], 'runtime smoke failed: ' + name)
    require(all(result['matches_expected'] for result in results.values()), 'PARTIAL: exact native matrix failed; no second cycle')
    for binary, executable in executables.items():
        preflight_all(receipts, output / 'runtime', executables, qualification['test_executables'])
        result = runner.execute(binary, [executable, '--nocapture'], 'test', test_environment, 900)
        require(result['returncode'] == 0 and not result['timeout'], 'regression failed: ' + binary)
    record(output / 'workflow-result.json', dict(status='bounded_workflow_pass',
           native_cases=len(results), inferred_exports=0, qualification=artifact(HERE / 'qualification.json')))

if __name__ == '__main__':
    main()
