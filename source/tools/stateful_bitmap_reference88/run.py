"""Private offline reference88 execution ledger; no acceptance inferred from exit codes."""

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_redb_stateful import (
    BUILD_PACKAGES, RUNTIME_FILES, TOOLCHAIN, artifact, digest, full_source_manifest,
    git_read, identity, native_binding, private_environment, record, seed_cache, source_manifest,
)

REPO = Path(__file__).resolve().parents[3]
CAMPAIGN = Path('/private/tmp/verus-native-reference88.w3Ws2g')
OUTPUT = CAMPAIGN / 'implementation'
RUSTUP = Path('/Users/tommyvolk/.cargo/bin/rustup')
SYSROOT = Path('/Users/tommyvolk/.rustup/toolchains') / TOOLCHAIN
SEED = Path('/private/tmp/verus-consolidated-resize87.govPpCSX/final-replay/build/cargo-home')
SOLVER = SEED.parents[1] / 'runtime/z3'
GUARDS = Path('/private/tmp/verus-bitmap-stateful-goal.IR9WYXR2/baseline82')


def preservation():
    immutable = json.loads((GUARDS / 'immutable-hashes.json').read_text())
    native = json.loads((GUARDS / 'native-hashes.json').read_text())
    result = dict(immutable_count=len(immutable), immutable_mismatches=[], native={})
    for name, expected in immutable.items():
        if not Path(name).is_file() or digest(name) != expected:
            result['immutable_mismatches'].append(name)
    for name, original in native.items():
        root = Path(original['root'])
        result['native'][name] = dict(
            head=git_read(root, 'rev-parse', 'HEAD'),
            status=git_read(root, 'status', '--porcelain', '--untracked-files=all'),
            mismatches=[relative for relative, expected in original['sources'].items()
                        if not (root / relative).is_file() or digest(root / relative) != expected])
    baseline = json.loads((CAMPAIGN / 'git-baseline.json').read_text())
    refs = git_read(REPO, 'show-ref')
    index = Path(git_read(REPO, 'rev-parse', '--git-path', 'index'))
    if not index.is_absolute():
        index = REPO / index
    result['git'] = dict(head=git_read(REPO, 'rev-parse', 'HEAD'),
                         branch=git_read(REPO, 'symbolic-ref', 'HEAD'),
                         index_sha256=digest(index), refs=refs,
                         refs_drift=refs != baseline['refs'])
    result['scope_sha256'] = digest(REPO / 'evidence/native-reference88/SCOPE.md')
    return result


def execute(name, argv, category, environment, timeout):
    counts_path = OUTPUT / 'launch-counts.json'
    directory = OUTPUT / 'runs' / name
    with (OUTPUT / 'launch-counts.lock').open('a+b') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        counts = json.loads(counts_path.read_text())
        limit = dict(build=1, vstd=1, application=64, test=100)[category]
        if counts[category] >= limit:
            raise RuntimeError('budget exhausted: ' + category)
        directory.mkdir(parents=True)
        counts[category] += 1
        record(counts_path, counts)
    invocation = dict(argv=list(map(str, argv)), cwd=str(REPO / 'source'),
                      environment=environment, timeout_seconds=timeout, category=category,
                      compiler_source_sha256=identity(source_manifest(REPO)),
                      source_sha256=identity(full_source_manifest(REPO)))
    record(directory / 'invocation.json', invocation)
    started = time.monotonic()
    result = dict(returncode=None, timeout=False)
    with (directory / 'stdout').open('wb') as stdout, (directory / 'stderr').open('wb') as stderr:
        try:
            process = subprocess.Popen(invocation['argv'], cwd=invocation['cwd'],
                                       env=environment, stdout=stdout, stderr=stderr,
                                       start_new_session=True)
            try:
                result['returncode'] = process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                result['timeout'] = True
                os.killpg(process.pid, signal.SIGKILL)
                result['returncode'] = process.wait()
        except OSError as error:
            result['spawn_error'] = str(error)
    result.update(seconds=time.monotonic() - started,
                  stdout=artifact(directory / 'stdout'), stderr=artifact(directory / 'stderr'))
    record(directory / 'result.json', result)
    print(json.dumps(dict(name=name, **result)), flush=True)
    return result


def initialize():
    OUTPUT.mkdir()
    manifest = source_manifest(REPO)
    producer = 'source-' + identity(manifest)
    environment = private_environment(OUTPUT, RUSTUP, SYSROOT, producer)
    record(OUTPUT / 'launch-counts.json', dict(build=0, vstd=0, application=0, test=0))
    record(OUTPUT / 'source-manifest.json', manifest)
    record(OUTPUT / 'full-source-start.json', full_source_manifest(REPO))
    record(OUTPUT / 'preservation-start.json', preservation())
    record(OUTPUT / 'native.json', native_binding('/private/tmp/verus-transfer-redb-miuzm2w3/redb'))
    record(OUTPUT / 'setup.json', dict(environment=environment, producer=producer,
                                     cache_commands=seed_cache(SEED, Path(environment['CARGO_HOME']))))
    runtime = OUTPUT / 'runtime'
    runtime.mkdir()
    shutil.copy2(SOLVER, runtime / 'z3')


def build(environment):
    runtime = OUTPUT / 'runtime'
    argv = [RUSTUP, 'run', TOOLCHAIN, SYSROOT / 'bin/cargo', 'build', '--offline', '--locked',
            '--release', '--message-format=json']
    for package in BUILD_PACKAGES:
        argv += ['-p', package]
    result = execute('compiler-build', argv, 'build', environment, 1800)
    if result['returncode'] != 0 or result['timeout']:
        raise RuntimeError('compiler build failed; no second build authorized')
    matched_vstd(environment)


def matched_vstd(environment):
    runtime = OUTPUT / 'runtime'
    result = json.loads((OUTPUT / 'runs/compiler-build/result.json').read_text())
    if result['returncode'] != 0 or result['timeout']:
        raise RuntimeError('no successful compiler build')
    if source_manifest(REPO) != json.loads((OUTPUT / 'source-manifest.json').read_text()):
        raise RuntimeError('compiler source changed during build')
    for name in RUNTIME_FILES:
        if name not in ('libvstd.rlib', 'vstd.vir', 'z3'):
            shutil.copy2(Path(environment['CARGO_TARGET_DIR']) / 'release' / name, runtime / name)
    vstd_environment = dict(environment, VSTD_KIND='IsVstd')
    vstd_environment.pop('VARGO_BUILD_SHA')
    vstd_environment.pop('VARGO_TOOLCHAIN')
    argv = [RUSTUP, 'run', TOOLCHAIN, runtime / 'rust_verify', '--internal-test-mode',
            '--sysroot', SYSROOT, '--crate-type=lib', '--export', runtime / 'vstd.vir',
            '--out-dir', runtime, '--multiple-errors', '2', '--is-vstd', '--compile',
            '-C', 'opt-level=3']
    for name, filename in [('verus_builtin', 'libverus_builtin.rlib'),
                           ('verus_builtin_macros', 'libverus_builtin_macros.dylib'),
                           ('verus_state_machines_macros', 'libverus_state_machines_macros.dylib')]:
        argv += ['--extern', name + '=' + str(runtime / filename)]
    for feature in ['std', 'alloc', 'nonzero_internals']:
        argv += ['--cfg', 'feature="' + feature + '"']
    argv += ['vstd/vstd.rs']
    result = execute('matched-vstd', argv, 'vstd', vstd_environment, 900)
    if result['returncode'] != 0 or result['timeout']:
        raise RuntimeError('matched vstd failed; no second vstd authorized')
    record(OUTPUT / 'runtime-manifest.json', {name: artifact(runtime / name) for name in RUNTIME_FILES})


def application(arguments, environment):
    if source_manifest(REPO) != json.loads((OUTPUT / 'source-manifest.json').read_text()):
        raise RuntimeError('source/runtime mismatch')
    runtime = OUTPUT / 'runtime'
    bindings = json.loads((OUTPUT / 'runtime-manifest.json').read_text())
    if any(artifact(runtime / name) != value for name, value in bindings.items()):
        raise RuntimeError('runtime drift')
    directory = OUTPUT / 'applications' / arguments.name
    directory.mkdir(parents=True)
    source = directory / 'input.rs'
    shutil.copyfile(arguments.fixture, source)
    interface = directory / (arguments.crate + '.vir')
    library = directory / ('lib' + arguments.crate + '.rlib')
    expectation = dict(input=artifact(source), expected=arguments.expect,
                       original=artifact(arguments.fixture), imports=arguments.imports,
                       compiler_source_sha256=identity(source_manifest(REPO)), runtime=bindings)
    record(directory / 'frozen-expectation.json', expectation)
    argv = [RUSTUP, 'run', TOOLCHAIN, runtime / 'verus', source, '--sysroot', SYSROOT,
            '--crate-type=lib', '--edition=2021', '--crate-name', arguments.crate,
            '--no-cheating', '--compile', '--output-json', '--num-threads', '1',
            '--export', interface, '-o', library]
    for value in arguments.imports:
        name, run = value.split('=', 1)
        dependency = OUTPUT / 'applications' / run
        previous = json.loads((dependency / 'published.json').read_text())
        for kind in ['interface', 'library']:
            if artifact(previous[kind]['path']) != previous[kind]:
                raise RuntimeError('import drift')
        argv += ['--import', name + '=' + previous['interface']['path'],
                 '--extern', name + '=' + previous['library']['path']]
    argv += arguments.extra
    result = execute(arguments.name, argv, 'application', environment, arguments.timeout)
    report = None
    try:
        report = json.loads((OUTPUT / 'runs' / arguments.name / 'stdout').read_text())
    except (ValueError, OSError):
        pass
    checked = report is not None and report.get('verification-results', {}).get('success') is True
    if result['returncode'] == 0 and not result['timeout'] and checked and not arguments.negative:
        record(directory / 'published.json', dict(interface=artifact(interface), library=artifact(library)))
    else:
        record(directory / 'withheld.json', dict(reason='failed verification; no provider credit',
                                                existing_outputs=[str(path) for path in [interface, library]
                                                                  if path.exists()]))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('operation', choices=['init', 'build', 'vstd', 'app', 'audit', 'test'])
    parser.add_argument('--name')
    parser.add_argument('--fixture', type=Path)
    parser.add_argument('--crate', default='reference88')
    parser.add_argument('--expect', default='exploratory; not acceptance')
    parser.add_argument('--imports', action='append', default=[])
    parser.add_argument('--negative', action='store_true')
    parser.add_argument('--timeout', type=int, default=240)
    arguments, extra = parser.parse_known_args()
    arguments.extra = extra
    if arguments.operation == 'init':
        initialize()
        return
    environment = json.loads((OUTPUT / 'setup.json').read_text())['environment']
    environment.pop('VSTD_KIND', None)
    if arguments.operation == 'build':
        build(environment)
    elif arguments.operation == 'vstd':
        matched_vstd(environment)
    elif arguments.operation == 'app':
        application(arguments, environment)
    elif arguments.operation == 'test':
        command = arguments.extra[1:] if arguments.extra[:1] == ['--'] else arguments.extra
        execute(arguments.name, command, 'test', environment, arguments.timeout)
    else:
        record(OUTPUT / 'preservation-final.json', preservation())
        record(OUTPUT / 'full-source-final.json', full_source_manifest(REPO))


if __name__ == '__main__':
    main()
