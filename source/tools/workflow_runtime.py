"""Small offline execution/binding helpers; specialized proof oracles stay local."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1048576), b''):
            result.update(chunk)
    return result.hexdigest()


def identity(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def record(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def artifact(path):
    path = Path(path).absolute()
    require(path.is_file() and not path.is_symlink(), 'missing regular artifact: ' + str(path))
    return dict(path=str(path), sha256=digest(path))


def identical(actual, expected):
    if type(actual) is not type(expected):
        return False
    if type(actual) is dict:
        return actual.keys() == expected.keys() and all(identical(actual[key], expected[key]) for key in actual)
    if type(actual) is list:
        return len(actual) == len(expected) and all(identical(left, right) for left, right in zip(actual, expected))
    return actual == expected


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


def read_json(path):
    return json.loads(Path(path).read_bytes(), object_pairs_hook=unique_object,
                      parse_constant=lambda value: require(False, 'nonfinite JSON: ' + value))


def environment(updates, *, base=None):
    excluded = {'RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER',
                'RUSTC_WORKSPACE_WRAPPER', 'VERUS_EXTRA_ARGS', 'VERUS_INFER_CACHE'}
    inherited = os.environ if base is None else base
    result = {key: value for key, value in inherited.items()
              if not key.startswith('GIT_') and key not in excluded}
    result.update(updates)
    require(all(type(key) is str and type(value) is str for key, value in result.items()), 'invalid environment')
    return result


def runtime_artifacts(runtime, names, expected=None):
    actual = {name: artifact(Path(runtime) / name) for name in names}
    if expected is not None:
        require(set(expected) == set(actual), 'incomplete runtime binding')
        require(identical(actual, expected), 'runtime binding')
    return actual


def stage_runtime(runtime, destination, names):
    started = time.monotonic()
    sources = runtime_artifacts(runtime, names)
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    for name in names:
        target = destination / name
        require(not target.is_symlink(), 'runtime destination symlink')
        if (Path(runtime) / name).resolve() != target.resolve():
            shutil.copy2(Path(runtime) / name, target)
    staged = runtime_artifacts(destination, names)
    require(all(staged[name]['sha256'] == sources[name]['sha256'] for name in names), 'runtime stage mismatch')
    return staged, time.monotonic() - started


def validate_command(actual, expected, message='noncanonical command'):
    require(type(actual) is list and actual and all(type(value) is str and value for value in actual), message)
    require(identical(actual, expected), message)


def execute(directory, argv, cwd, environment, timeout, *, metadata=None, inputs=()):
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=False)
    argv = list(map(str, argv))
    validate_command(argv, argv)
    invocation = dict(metadata or {})
    invocation.update(argv=argv, cwd=str(cwd), environment=environment, timeout_seconds=timeout)
    if inputs:
        invocation['inputs'] = [artifact(path) for path in inputs]
    record(directory / 'invocation.json', invocation)
    started = time.monotonic()
    result = dict(returncode=None, timeout=False)
    with (directory / 'stdout').open('wb') as stdout, (directory / 'stderr').open('wb') as stderr:
        try:
            process = subprocess.Popen(argv, cwd=cwd, env=environment, stdout=stdout, stderr=stderr,
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
    if inputs:
        require(invocation['inputs'] == [artifact(path) for path in inputs], 'execution input drift')
    return result


def bind_execution(directory, argv, cwd, environment, *, inputs=()):
    directory = Path(directory)
    invocation = read_json(directory / 'invocation.json')
    validate_command(invocation['argv'], argv, 'noncanonical command')
    require(invocation['cwd'] == str(cwd), 'invocation cwd binding')
    require(identical(invocation['environment'], environment), 'environment binding')
    if inputs:
        require(identical(invocation.get('inputs'), [artifact(path) for path in inputs]), 'execution input drift')
    result = read_json(directory / 'result.json')
    require(result.get('spawn_error') is None and result['timeout'] is False, 'incomplete execution')
    require(type(result['returncode']) is int, 'invalid exit status')
    for stream in ('stdout', 'stderr'):
        require(identical(result[stream], artifact(directory / stream)), 'unbound ' + stream)
    return result


def report_sections(report):
    require(type(report) is dict, 'invalid report schema')
    result, inference = report.get('verification-results'), report.get('verus-infer')
    require(type(result) is dict and type(inference) is dict, 'invalid report schema')
    scope = inference.get('verification_scope')
    require(type(scope) is dict, 'invalid report schema')
    return result, inference, scope


def validate_outcome(report, code, positive, *, verified=None, errors=None):
    require(type(positive) is bool, 'invalid expectation')
    require(type(code) is int and code == (0 if positive else 1), 'positive exit code' if positive else 'negative exit code')
    result, inference, scope = report_sections(report)
    for key, expected in (('success', positive), ('encountered-error', not positive),
                          ('encountered-vir-error', False), ('is-verifying-entire-crate', True)):
        require(result.get(key) is expected, 'positive result flag: ' + key if positive else 'negative result flag: ' + key)
    for key, expected in (('errors', errors), ('verified', verified)):
        message = 'positive errors' if key == 'errors' else 'verified count'
        require(type(result.get(key)) is int and result[key] >= 0, message)
        if expected is not None:
            require(result[key] == expected, message)
    require(not positive or result['errors'] == 0, 'positive errors')
    require(inference.get('crate_success') is positive, 'positive inference status: contradictory outcome' if positive else 'contradictory inference outcome')
    return result, inference, scope


def validate_timed(timings, validator, *arguments, **keywords):
    started = time.monotonic()
    try:
        return validator(*arguments, **keywords)
    finally:
        timings['validation_seconds'] = timings.get('validation_seconds', 0.0) + time.monotonic() - started
