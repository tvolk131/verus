"""Offline replay of a bounded authored lifecycle and one checked Infer bridge."""

import argparse
from copy import deepcopy
import json
import os
from pathlib import Path
import re
import shutil
import sys
import time

sys.dont_write_bytecode = True

import workflow_runtime as shared
from bitmap_stateful_acceptance import unique_functions, validate_positive_report
from check_redb_stateful import native_binding, source_manifest
from infer_transfer_identity import INTERFACE_POLICY, INTERFACE_SCHEMA
from infer_transfer_receipts import metadata


DATA = Path(__file__).resolve().with_name('stateful_bitmap_allocation93')
QUALIFICATION_SHA256 = 'e5b06609ce3f7f56a4675f86832bf8e8b3bdf06c4cb879c1d08ab59a30d42482'
BRIDGE = 'assert((1u64 << bit) == 1u64) by(bit_vector) requires bit == 0;'
CASES = (
    ('manual', 'manual.rs', 'initialized93', False, 'host'),
    ('on', 'reduced.rs', 'initialized93', True, 'host'),
    ('off', 'reduced.rs', 'initialized93', False, 'host'),
    ('import', 'consumer.rs', 'consumer_initialized93', False, 'host'),
    ('seeded_off', 'reduced.rs', 'initialized93', False, 'host'),
    ('missing_import', 'consumer.rs', 'consumer_initialized93', False, 'host'),
    ('negative', 'negative.rs', 'negative93', False, 'host'),
    ('termination', 'termination.rs', 'initialized93', False, 'host'),
    ('arch_on', 'arch32.rs', 'architecture32_qualified93', True, 'target32'),
    ('arch_off', 'arch32.rs', 'architecture32_qualified93', False, 'target32'),
    ('arch_manual', 'arch32_manual.rs', 'architecture32_qualified93', False, 'target32'),
)
FORBIDDEN = ('panicked at', 'internal compiler error', 'timed out', 'resource limit',
             'not supported', 'type mismatch', 'cannot find', 'aborting due to 0')


def qualification():
    shared.require(shared.digest(DATA / 'qualification.json') == QUALIFICATION_SHA256,
                   'qualification drift')
    return shared.read_json(DATA / 'qualification.json')


def tree_manifest(root):
    root = Path(root)
    return {str(path.relative_to(root)): dict(link=str(path.readlink())) if path.is_symlink()
            else dict(sha256=shared.digest(path)) for path in sorted(root.rglob('*'))
            if path.is_file() or path.is_symlink()}


def preflight(repo, native, runtimes, sysroots, maximum, expected):
    shared.require(type(maximum) is int and maximum >= len(CASES), 'insufficient launch allowance')
    shared.require([row[0] for row in CASES] == expected['plan'], 'plan drift')
    shared.require(shared.identical(source_manifest(repo), expected['compiler_source']),
                   'compiler source drift')
    shared.require(shared.identity(expected['compiler_source']) == expected['compiler_identity'],
                   'compiler identity')
    binding = native_binding(native)
    for kind in ('host', 'target32'):
        shared.require(shared.identical(tree_manifest(sysroots[kind]), expected['sysroots'][kind]),
                       'sysroot drift: ' + kind)
        actual = {name: shared.digest(Path(runtimes[kind]) / name)
                  for name in expected['runtime'][kind]}
        shared.require(shared.identical(actual, expected['runtime'][kind]), 'runtime drift: ' + kind)
    actual = {name: shared.digest(DATA / 'fixtures' / name) for name in expected['fixtures']}
    shared.require(shared.identical(actual, expected['fixtures']), 'fixture drift')
    methods = expected['rust_methods']
    shared.require(all(type(methods[key]) is int for key in ('unit', 'integration', 'total', 'cap'))
                   and methods['unit'] + methods['integration'] == methods['total'] == 32
                   and methods['total'] <= methods['cap'] == 48, 'aggregate Rust method allowance')
    manual = (DATA / 'fixtures/manual.rs').read_text()
    shared.require(manual.count(BRIDGE) == 1 and manual.replace(' ' + BRIDGE, '', 1)
                   == (DATA / 'fixtures/reduced.rs').read_text(), 'one-bridge ablation drift')
    return dict(native=binding, compiler_identity=expected['compiler_identity'],
                planned_applications=len(CASES), launch_allowance=maximum,
                rust_methods=methods, theorem_credit=False)


def execution_environment(output, runtime, sysroot, case):
    return shared.environment(dict(
        PATH=str(sysroot / 'bin') + ':/usr/bin:/bin:/usr/sbin:/sbin',
        HOME=str(output / 'home'), TMPDIR=str(output / 'tmp'),
        CARGO_HOME=str(output / 'cargo-home'), CARGO_TARGET_DIR=str(output / 'target'),
        CARGO_NET_OFFLINE='true', RUSTUP_HOME=str(output / 'rustup-home'),
        RUSTUP_NO_UPDATE_CHECK='1', RUSTUP_TOOLCHAIN='1.99.1-aarch64-apple-darwin',
        RUSTC=str(sysroot / 'bin/rustc'), RUSTDOC=str(sysroot / 'bin/rustdoc'),
        RUSTC_BOOTSTRAP='1', RUST_MIN_STACK='20971520', PYTHONDONTWRITEBYTECODE='1',
        DYLD_LIBRARY_PATH=str(sysroot / 'lib') + ':' + str(sysroot / 'lib/rustlib/aarch64-apple-darwin/lib'),
        VERUS_USE_RUSTUP='0', VERUS_IN_VARGO='1', VERUS_Z3_PATH=str(runtime / 'z3'),
        VERUS_EXTRA_ARGS='--sysroot ' + str(sysroot),
        VERUS_INFER_CACHE=str(output / 'cache' / case), XDG_CACHE_HOME=str(output / 'cache'),
        GIT_OPTIONAL_LOCKS='0', GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM='1'), base={})


def command(output, case, runtime, sysroot):
    label, fixture, crate, enabled, kind = case
    directory = output / label
    arguments = [str(runtime / 'verus'), '--sysroot', str(sysroot)]
    if kind == 'target32':
        arguments += ['--target', 'i686-unknown-linux-gnu']
    arguments += ['--edition=2024', '--crate-type=lib', '--crate-name', crate]
    if kind == 'host':
        arguments += ['--no-cheating']
    arguments += ['--triggers-mode', 'silent', '--multiple-errors', '16', '--rlimit', '20',
                  '--num-threads', '1', '--output-json', str(directory / 'input.rs')]
    if label in ('import', 'negative', 'missing_import'):
        provider = output / ('seeded_off' if label == 'missing_import' else 'on')
        arguments += ['--import', 'initialized93=' + str(provider / 'initialized93.vir'),
                      '--extern', 'initialized93=' + str(provider / 'libinitialized93.rlib')]
    arguments += ['--compile', '--export', str(directory / (crate + '.vir')),
                  '-o', str(directory / ('lib' + crate + '.rlib'))]
    if enabled:
        arguments += ['--infer-goals']
    return arguments


def diagnostic_rows(stderr, source):
    text = stderr.decode('utf-8')
    shared.require(not any(marker in text.lower() for marker in FORBIDDEN), 'incidental failure')
    headers = re.findall(r'(?m)^[^\S\n]*error\b[^\n]*', text)
    shared.require(headers and all(header.startswith('error: ') for header in headers),
                   'noncanonical error diagnostic')
    blocks = re.split(r'(?m)^error: ', text)
    shared.require(not blocks[0].strip(), 'unclassified diagnostic preamble')
    found = []
    summary_seen = False
    for index, block in enumerate(blocks[1:], 1):
        shared.require(bool(block.strip()), 'empty error diagnostic')
        heading = block.splitlines()[0]
        summary = re.fullmatch(r'aborting due to (\d+) previous errors?\n*', block)
        if summary is not None:
            shared.require(index == len(blocks) - 1 and int(summary[1]) == len(found),
                           'diagnostic abort summary')
            summary_seen = True
            continue
        location = re.search(r'(?m)^\s*--> (.*):(\d+):(\d+)\s*$', block)
        shared.require(location is not None and location[1] == str(source), 'diagnostic source')
        found.append([heading, int(location[2]), int(location[3])])
    shared.require(found and summary_seen, 'missing intended backend diagnostics or abort summary')
    return found


def expected_diagnostics(label, source):
    if label in ('off', 'seeded_off'):
        return [['possible arithmetic underflow/overflow', 76, 20], ['assertion failed', 77, 24]]
    if label == 'arch_off':
        return [['postcondition not satisfied', 4, 67], ['possible arithmetic underflow/overflow', 4, 79]]
    if label == 'termination':
        line = next(index for index, text in enumerate(source.read_text().splitlines(), 1)
                    if 'for index in 0..pages' in text)
        return [['decreases not satisfied at end of loop', line, 5]]
    return [['precondition not satisfied' if line == 32 else 'assertion failed', line,
             1 if line == 32 else 12 if line == 42 else 8]
            for line in (7, 11, 15, 20, 24, 28, 32, 36, 42)]


def report_expectation(expected, label, output):
    wanted = deepcopy(expected['expectations']['off' if label == 'seeded_off' else label])
    if wanted['dependencies'] == 'fresh_provider_trailer':
        certificate, fingerprint = metadata((output / 'on/initialized93.vir').read_bytes())
        shared.require(certificate['schema'] == INTERFACE_SCHEMA and certificate['policy'] == INTERFACE_POLICY
                       and all(certificate[key] is True for key in ('soundness', 'maximality', 'exactness')),
                       'provider trailer')
        wanted['dependencies'] = {5885436244799145436: fingerprint}
    return wanted


def validate_negative(document, code, wanted, stderr, source, label):
    results, inference, scope = shared.validate_outcome(document, code, False,
        verified=wanted['verified'], errors=wanted['errors'])
    shared.require(inference.get('interface_written') is False
                   and 'generation_failure' in inference and inference['generation_failure'] is None
                   and inference.get('failed_functions') == wanted['failed_functions']
                   and inference.get('functions') == []
                   and type(inference.get('requested_functions')) is int and inference['requested_functions'] == 0
                   and type(inference.get('generated_functions')) is int and inference['generated_functions'] == 0,
                   'negative inference status')
    shared.require(scope.get('native_collection_completed') is True
                   and scope.get('complete_selected_scope_verified') is False
                   and type(scope.get('checked_exec_bodies')) is int and scope['checked_exec_bodies'] == 0
                   and type(scope.get('checked_proof_bodies')) is int and scope['checked_proof_bodies'] == 0,
                   'negative complete-body status')
    bodies = unique_functions(scope.get('native_functions'), 'negative body rows')
    actual = {name: {key: row.get(key) for key in wanted['bodies'][name]}
              for name, row in bodies.items() if name in wanted['bodies']}
    shared.require(set(bodies) == set(wanted['bodies']) and shared.identical(actual, wanted['bodies']),
                   'negative body scope')
    shared.require(inference.get('trusted_models') == wanted['trusted_models']
                   and inference.get('imported_inference_dependencies') == [
                       dict(crate_id=key, fingerprint=value) for key, value in wanted['dependencies'].items()],
                   'negative import/model binding')
    shared.require(diagnostic_rows(stderr, source) == expected_diagnostics(label, source),
                   'negative intended diagnostics')
    return results


def validate_missing_import(document, code, stderr, missing):
    results, inference, scope = shared.report_sections(document)
    shared.require(type(code) is int and code == 1 and results.get('success') is False
                   and results.get('encountered-error') is True
                   and results.get('encountered-vir-error') is True
                   and 'verified' not in results and 'errors' not in results
                   and inference.get('crate_success') is False
                   and inference.get('interface_written') is False
                   and scope.get('native_collection_completed') is False
                   and scope.get('complete_selected_scope_verified') is False
                   and type(scope.get('checked_exec_bodies')) is int and scope['checked_exec_bodies'] == 0
                   and type(scope.get('checked_proof_bodies')) is int and scope['checked_proof_bodies'] == 0
                   and scope.get('native_functions') == [], 'import gate status')
    text = stderr.decode('utf-8')
    expected = ('error: could not open imported library file `' + str(missing)
                + '`: No such file or directory (os error 2)\n\n'
                + 'error: aborting due to 1 previous error\n\n')
    shared.require(text == expected, 'import gate diagnostic')


def compiled_i386(library):
    payload = library.read_bytes()
    shared.require(payload.startswith(b'!<arch>\n'), 'target archive')
    offset, names, objects = 8, b'', []
    while offset + 60 <= len(payload):
        header = payload[offset:offset + 60]
        shared.require(header[58:60] == b'`\n', 'target archive header')
        name, size = header[:16].decode().strip(), int(header[48:58])
        body = payload[offset + 60:offset + 60 + size]
        shared.require(len(body) == size, 'truncated target object')
        if name == '//':
            names = body
        if name.startswith('/') and name[1:].isdigit():
            name = names[int(name[1:]):].split(b'/\n', 1)[0].decode()
        if name.endswith('.o'):
            shared.require(body[:5] == b'\x7fELF\x01' and int.from_bytes(body[18:20], 'little') == 3,
                           'not compiled ELF32 i386')
            objects.append(name)
        offset += 60 + size + size % 2
    shared.require(objects, 'no compiled target object')
    return objects


def validate_case(output, case, runtimes, sysroots, expected):
    label, fixture, crate, enabled, kind = case
    directory = output / label
    runtime, sysroot = Path(runtimes[kind]), Path(sysroots[kind])
    shared.require(all(shared.digest(runtime / name) == value
                       for name, value in expected['runtime'][kind].items()), 'validation runtime drift')
    source = directory / 'input.rs'
    shared.require(shared.digest(source) == expected['fixtures'][fixture], 'consumed source drift')
    inputs = [source, *(runtime / name for name in expected['runtime'][kind])]
    if label in ('import', 'negative', 'missing_import'):
        provider = output / ('seeded_off' if label == 'missing_import' else 'on')
        inputs += [provider / 'libinitialized93.rlib']
        if label != 'missing_import':
            inputs += [provider / 'initialized93.vir']
    bound = shared.bind_execution(output / 'runs' / label, command(output, case, runtime, sysroot),
        directory, execution_environment(output, runtime, sysroot, label), inputs=inputs)
    document = shared.read_json(output / 'runs' / label / 'stdout')
    stderr = (output / 'runs' / label / 'stderr').read_bytes()
    library, interface = directory / ('lib' + crate + '.rlib'), directory / (crate + '.vir')
    if label == 'missing_import':
        validate_missing_import(document, bound['returncode'], stderr, output / 'seeded_off/initialized93.vir')
        positive = False
        result = dict(classification='IMPORT_GATE_REJECTION', semantic_credit=False)
    else:
        wanted = report_expectation(expected, label, output)
        positive = wanted['errors'] == 0
        if positive:
            shared.require(bound['returncode'] == 0 and not stderr.strip(), 'positive execution/diagnostics')
            validate_positive_report(document, wanted)
            shared.require(library.is_file() and not library.is_symlink() and library.stat().st_size > 0
                           and interface.is_file() and not interface.is_symlink() and interface.stat().st_size > 0,
                           'missing fresh positive export')
            result = dict(classification='CHECKED_COMPONENT', semantic_credit=True,
                          library=shared.artifact(library), interface=shared.artifact(interface))
            if kind == 'target32':
                result['compiled_i386_objects'] = compiled_i386(library)
        else:
            validate_negative(document, bound['returncode'], wanted, stderr, source, label)
            result = dict(classification='INTENDED_BACKEND_REJECTION', semantic_credit=True)
    if not positive:
        shared.require(not interface.exists() and not interface.is_symlink(), 'failed VIR publication')
        if label == 'seeded_off':
            seed = shared.read_json(output / 'seed.json')
            shared.require(shared.digest(library) == seed['library_sha256'], 'stale rlib hash changed')
            result.update(withdrawn_VIR=True, retained_stale_rlib=shared.artifact(library),
                          retained_rlib_is_certificate=False)
        else:
            shared.require(not library.exists() and not library.is_symlink(), 'unexpected failure rlib')
    result.update(label=label, returncode=bound['returncode'], seconds=bound['seconds'])
    return result


def replay(output, runtimes, sysroots, expected):
    for name in ('home', 'tmp', 'cargo-home', 'target', 'rustup-home', 'cache'):
        (output / name).mkdir()
    results = []
    for ordinal, case in enumerate(CASES, 1):
        label, fixture, crate, enabled, kind = case
        directory = output / label
        directory.mkdir()
        shutil.copy2(DATA / 'fixtures' / fixture, directory / 'input.rs')
        if label == 'seeded_off':
            for name in ('initialized93.vir', 'libinitialized93.rlib'):
                shutil.copy2(output / 'on' / name, directory / name)
            shared.record(output / 'seed.json', dict(
                library_sha256=shared.digest(directory / 'libinitialized93.rlib'),
                interface_sha256=shared.digest(directory / 'initialized93.vir')))
        runtime, sysroot = Path(runtimes[kind]), Path(sysroots[kind])
        inputs = [directory / 'input.rs', *(runtime / name for name in expected['runtime'][kind])]
        if label in ('import', 'negative', 'missing_import'):
            provider = output / ('seeded_off' if label == 'missing_import' else 'on')
            inputs += [provider / 'libinitialized93.rlib']
            if label != 'missing_import':
                inputs += [provider / 'initialized93.vir']
        shared.require(all(shared.digest(runtime / name) == value
                           for name, value in expected['runtime'][kind].items()), 'live runtime drift')
        shared.record(output / 'launches.json', dict(attempts=ordinal, cap=len(CASES),
            labels=[row[0] for row in CASES[:ordinal]], classification='IMPLEMENTATION_QUALIFICATION'))
        shared.execute(output / 'runs' / label, command(output, case, runtime, sysroot), directory,
            execution_environment(output, runtime, sysroot, label), 240, inputs=inputs)
        results.append(validate_case(output, case, runtimes, sysroots, expected))
        shared.record(output / 'completed-cases.json', results)
    return results


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('output', 'native', 'host-runtime', 'target32-runtime', 'host-sysroot', 'target32-sysroot'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--max-launches', type=int, default=11)
    parser.add_argument('--plan-only', action='store_true')
    arguments = parser.parse_args(argv)
    output = arguments.output.absolute()
    shared.require(not output.exists() and not output.is_symlink(), 'output must not exist')
    output.mkdir(parents=True)
    started = time.monotonic()
    expected = qualification()
    runtimes = dict(host=arguments.host_runtime.resolve(), target32=arguments.target32_runtime.resolve())
    sysroots = dict(host=arguments.host_sysroot.resolve(), target32=arguments.target32_sysroot.resolve())
    repo = Path(__file__).resolve().parents[2]
    try:
        before = preflight(repo, arguments.native, runtimes, sysroots, arguments.max_launches, expected)
        shared.record(output / 'preflight.json', dict(before, seconds=time.monotonic() - started))
        if arguments.plan_only:
            shared.record(output / 'summary.json', dict(status='PLAN_ONLY', theorem_credit=False, launches=0))
            return 2
        results = replay(output, runtimes, sysroots, expected)
        after = preflight(repo, arguments.native, runtimes, sysroots, arguments.max_launches, expected)
        shared.require(shared.identical(before, after), 'preflight/postflight mismatch')
        shared.record(output / 'summary.json', dict(status='REPLAY_PASSED_PENDING_INDEPENDENT_ACCEPTANCE',
            launches=len(results), cases=results, seconds=time.monotonic() - started,
            infer_gain='one original checked local-shift bridge removed; not semantic invariant inference',
            authored_proof=True, production_caller_established=False, physical32_execution=False,
            git_objectstore='NOT_CERTIFIED_AFTER_HISTORICAL_WRITE_TREE_ATTEMPT', old_goals_completed=False))
        return 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        shared.record(output / 'summary.json', dict(status='INCOMPLETE', error=str(error), theorem_credit=False))
        print(str(error), file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
