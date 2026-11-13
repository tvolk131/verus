#!/usr/bin/env python3
"""Run maintained Infer suites; logs and a compact receipt stay outside the source tree."""
import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile

from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require

ROOT = Path(__file__).resolve().parents[2]
TOOLS = ROOT / 'source/tools'
TESTS = ROOT / 'source/rust_verify_test/tests'


def positive_int(value):
    result = int(value)
    if result <= 0:
        raise argparse.ArgumentTypeError('must be positive')
    return result


def integration_targets(suite):
    if suite == 'applications':
        return ['infer_application_regressions']
    names = {p.stem for p in TESTS.glob('infer*.rs')}
    names.update(p.stem for p in TESTS.glob('local_checked*.rs'))
    names.update(['constructor_function_values', 'fndef_types', 'certified_foreign_bool'])
    names.discard('infer_application_regressions')
    return sorted(names)


def select_targets(suites, requested):
    targets = {s: integration_targets(s) for s in suites if s in ('compiler', 'applications')}
    available = {name for names in targets.values() for name in names}
    require(set(requested) <= available, 'unknown target or target outside selected suites')
    return {suite: [name for name in names if not requested or name in requested]
            for suite, names in targets.items()}


def artifacts(text, expected):
    result = {}
    for line in text.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get('reason') != 'compiler-artifact' or not message.get('executable'):
            continue
        name = message['target']['name']
        if name in expected and message.get('profile', {}).get('test') is True:
            executable = Path(message['executable'])
            require(name not in result or result[name] == executable, 'ambiguous test executable')
            result[name] = executable
    require(set(result) == set(expected), 'incomplete compiled test inventory')
    return result


def test_names(text):
    names = [line[:-6] for line in text.splitlines() if line.endswith(': test')]
    require(names and len(names) == len(set(names)), 'empty or duplicate Rust test inventory')
    return names


def check_rust_summary(text, selected):
    matches = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', text)
    require(matches, 'missing successful Rust test summary')
    passed, failed, ignored = map(int, matches[-1])
    require(failed == 0 and passed + ignored == selected, 'selected Rust tests were not all reported')
    return dict(passed=passed, ignored=ignored, selected=selected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite', choices=['tooling', 'compiler', 'applications', 'nightly', 'all'], default='tooling')
    parser.add_argument('--verus-root', type=Path, default=ROOT / 'source/target-verus/release')
    parser.add_argument('--solver', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--cargo', default='cargo')
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--jobs', type=positive_int, default=2)
    parser.add_argument('--batch-size', type=positive_int, default=8)
    parser.add_argument('--test-threads', type=positive_int, default=2)
    parser.add_argument('--target', action='append', default=[], help='select integration targets (repeatable); compiler units still run')
    parser.add_argument('--timeout', type=positive_timeout, default=600)
    parser.add_argument('--list', action='store_true', help='list suites/targets without running or building')
    args = parser.parse_args()
    suites = ['tooling', 'compiler', 'applications'] if args.suite == 'all' else [args.suite]
    targets_by_suite = select_targets(suites, args.target)
    if args.list:
        print(json.dumps({s: targets_by_suite.get(s, s) for s in suites}, indent=2))
        return
    output = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix='verus-infer-tests-'))
    if args.output:
        output.mkdir(parents=True, exist_ok=False)
    receipt = dict(suites=suites, targets=targets_by_suite, commands=[], rust_tests=[], application_theorems_claimed=False)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    with replay_receipt(output, receipt):
        guard = ArtifactGuard([Path(__file__), TOOLS / 'infer_experiment.py', TOOLS / 'infer_eval_support.py'])
        run = LoggedCommands(output, ROOT / 'source', receipt['commands'], timeout=args.timeout, guard=guard)
        tool_guard = None
        def invoke(label, command, cwd=None):
            if tool_guard is not None: tool_guard.check()
            command_runner = run if cwd is None else LoggedCommands(output, cwd, receipt['commands'], timeout=args.timeout, guard=guard)
            require(command_runner(label, list(map(str, command)), env) == 0, label + ' failed; logs retained')
            if tool_guard is not None: tool_guard.check()
            return (output / (label + '.stdout')).read_text()
        if 'tooling' in suites:
            for optimized in [False, True]:
                for pattern in ['test_*.py']:
                    label = ('optimized-' if optimized else 'normal-') + pattern.replace('*', 'all')
                    command = [sys.executable, '-B'] + (['-O'] if optimized else [])
                    invoke(label, command + ['-m', 'unittest', 'discover', '-s', TOOLS, '-p', pattern])
                    text = (output / (label + '.stderr')).read_text()
                    count = re.search(r'Ran (\d+) tests?', text)
                    require(count and int(count[1]) > 0, 'empty Python test selection')
        if any(s != 'tooling' for s in suites):
            bundle = args.verus_root.resolve()
            solver = (args.solver or bundle / ('z3.exe' if os.name == 'nt' else 'z3')).resolve()
            for name in ['rust_verify' + ('.exe' if os.name == 'nt' else ''), 'vstd.vir', 'libvstd.rlib']:
                require((bundle / name).is_file(), 'missing matching bundle input: ' + name)
            require(solver.is_file(), 'missing solver')
            inputs = tool_artifacts(bundle / ('verus.exe' if os.name == 'nt' else 'verus'), solver)
            tool_guard = ArtifactGuard(inputs)
            receipt['tool_sha256'] = {str(p): digest(p) for p in inputs}
            env.update(VERUS_IN_VARGO='1', VERUS_TEST_ROOT=str(bundle), VERUS_ROOT=str(bundle),
                       VERUS_Z3_PATH=str(solver))
            sysroot = Path(invoke('rust-sysroot', ['rustc', '--print', 'sysroot']).strip())
            variable = 'DYLD_LIBRARY_PATH' if sys.platform == 'darwin' else 'LD_LIBRARY_PATH'
            env[variable] = str(sysroot / 'lib') + os.pathsep + env.get(variable, '')
        cargo = [args.cargo, 'test', '--locked', '--jobs', str(args.jobs)]
        if args.offline: cargo.append('--offline')
        if args.release: cargo.append('--release')
        for suite in [s for s in suites if s in ('compiler', 'applications')]:
            builds = []
            if suite == 'compiler':
                builds.append(('units', ['air', 'vir', 'rust_verify'], ['--lib'], ['air', 'vir', 'rust_verify']))
            targets = targets_by_suite[suite]
            if targets:
                builds.append((suite, ['rust_verify_test'], sum((['--test', t] for t in targets), []), targets))
            for group, packages, selection, expected in builds:
                command = cargo + sum((['-p', p] for p in packages), []) + selection + ['--no-run', '--message-format=json']
                binaries = artifacts(invoke(group + '-build', command), expected)
                for name, binary in sorted(binaries.items()):
                    cwd = ROOT / 'source' / (name if group == 'units' else 'rust_verify_test')
                    names = test_names(invoke(name + '-list', [binary, '--list'], cwd))
                    for offset in range(0, len(names), args.batch_size):
                        batch = names[offset:offset + args.batch_size]
                        label = name + '-' + str(offset // args.batch_size)
                        selected = test_names(invoke(label + '-selection', [binary, '--list', '--exact', *batch], cwd))
                        require(set(selected) == set(batch), 'Rust exact batch selection changed')
                        threads = 1 if group == 'units' else args.test_threads
                        text = invoke(label, [binary, '--exact', *batch, '--test-threads=' + str(threads)], cwd)
                        receipt['rust_tests'].append(dict(target=name, **check_rust_summary(text, len(batch))))
        if 'nightly' in suites:
            invoke('assurance-mutations', [sys.executable, '-B', TOOLS / 'test_infer_assurance_mutations.py'])
            for name in ['eval_verus_infer.py', 'eval_verus_infer_cargo.py']:
                invoke(name, [sys.executable, '-B', TOOLS / name, '--verus', bundle / 'verus',
                              '--output-dir', output / name.removesuffix('.py')])
        guard.check()
    print(json.dumps(dict(output=str(output), success=receipt['success'],
                          rust_passed=sum(x['passed'] for x in receipt['rust_tests']),
                          rust_ignored=sum(x['ignored'] for x in receipt['rust_tests']))))


if __name__ == '__main__':
    main()
