#!/usr/bin/env python3
"""Independent source-freeze review. Writes only fresh private output trees.

No shared extraction, implementation, baseline or toolchain is changed.
Linux inotify schedules a single persistent private binding update; controller
and helper bytes stay exact. There is no byte restoration or interpreter hook.
"""
import argparse
import copy
import ctypes
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import struct
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
BRIDGE = Path('source/tools/research/native_semantic_bridge')
RAW = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')
REQUIRED = [BRIDGE/'source_map.py', BRIDGE/'audit.py', BRIDGE/'source-bindings.json',
    Path('source/tools/research/native_heap_lowering/lower.py'),
    Path('source/tools/research/native_heap_lowering/fixtures/drop.rs'),
    Path('source/tools/infer_eval_support.py'), Path('source/tools/infer_experiment.py')]
TOOLS = ('source_map.py', 'audit.py')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2)+'\n')


def prepare(directory):
    directory.mkdir(parents=True)
    for name in REQUIRED:
        target = directory/name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT/name, target)
    raw = directory/'extraction'
    raw.mkdir()
    for name in ('drop-native.json', 'drop-borrowck.json', 'receipt.json'):
        shutil.copyfile(RAW/name, raw/name)
    frozen = dict(before_first_trial=True, files={str(n): digest(directory/n) for n in REQUIRED})
    return frozen


def command(directory, tool, optimized):
    return [sys.executable, '-B', *(['-O'] if optimized else []), str(directory/BRIDGE/tool),
        '--freeze', str(directory/'freeze.json'), '--extraction', str(directory/'extraction'),
        '--output', str(directory/'output')]


def finish(directory, argv, result, expected):
    (directory/'stdout').write_text(result.stdout)
    (directory/'stderr').write_text(result.stderr)
    receipt = json.loads((directory/'output/receipt.json').read_text())
    require(receipt['publication'] is False and receipt['native_bridge_proved'] is False, 'native claim')
    if expected is None:
        require(result.returncode == 0 and receipt['success'] is True, result.stderr)
        require(all(str((directory/n).resolve()) in receipt['artifacts'] for n in REQUIRED), 'guard coverage')
    else:
        require(result.returncode != 0 and receipt['success'] is False, 'false success')
        require(expected in receipt['failure']['message'], receipt['failure']['message'])
    return dict(argv=argv, returncode=result.returncode, expected_failure=expected,
        receipt_sha256=digest(directory/'output/receipt.json'), receipt=receipt,
        stdout_sha256=digest(directory/'stdout'), stderr_sha256=digest(directory/'stderr'))


def run_case(directory, tool, optimized, frozen, expected=None):
    if not (directory/'freeze.json').exists():
        write(directory/'freeze.json', frozen)
    argv = command(directory, tool, optimized)
    result = subprocess.run(argv, capture_output=True, text=True, timeout=60)
    return finish(directory, argv, result, expected)


def synthetic(directory):
    raw = directory/'extraction/receipt.json'
    raw.write_text('{"success":true}\n')
    binding = directory/BRIDGE/'source-bindings.json'
    pin = json.loads(binding.read_text())
    pin['receipt_sha256'] = digest(raw)
    write(binding, pin)


def inotify_open(path):
    libc = ctypes.CDLL(None, use_errno=True)
    fd = libc.inotify_init1(os.O_CLOEXEC | os.O_NONBLOCK)
    require(fd >= 0, 'inotify_init1 failed')
    wd = libc.inotify_add_watch(fd, os.fsencode(path), 0x20)  # IN_OPEN
    require(wd >= 0, 'inotify_add_watch failed')
    return fd


def await_open(fd, count=1):
    deadline = time.monotonic()+20
    observed = 0
    while observed < count:
        require(time.monotonic() < deadline, 'no expected controller open event')
        readable, _, _ = select.select([fd], [], [], max(0, deadline-time.monotonic()))
        require(readable, 'inotify timeout')
        data = os.read(fd, 65536)
        offset = 0
        while offset < len(data):
            _, mask, _, length = struct.unpack_from('iIII', data, offset)
            observed += bool(mask & 0x20)
            offset += 16+length


def race_case(directory, tool, optimized, frozen, stage):
    if stage == 'between_validation_and_guard':
        # A regular, immutable optional freeze member, last in validation order.
        # Its first IN_OPEN proves all seven mandatory digest checks completed.
        # Sparse zeros only increase hash time, with no modified controller code.
        target = directory/'ballast.bin'
        with target.open('wb') as stream:
            stream.truncate(128 * 1024 * 1024)
        frozen['files']['ballast.bin'] = digest(target)
        trigger, count = target, 1
        expected = None  # Demonstrates the remaining false provenance acceptance.
    else:
        # receipt is opened for the first time by the initial guard and second
        # by authenticated_extraction. Change already-loaded support persistently.
        trigger, count = directory/'extraction/receipt.json', 2
        expected = 'tool inputs changed during evaluation'
    write(directory/'freeze.json', frozen)
    freeze_sha = digest(directory/'freeze.json')
    fd = inotify_open(trigger)
    argv = command(directory, tool, optimized)
    process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        await_open(fd, count)
        if stage == 'between_validation_and_guard':
            synthetic(directory)
        else:
            with (directory/'source/tools/infer_experiment.py').open('a') as stream:
                stream.write('\n# persistent review mutation after initial guard\n')
        stdout, stderr = process.communicate(timeout=60)
    finally:
        os.close(fd)
        if process.poll() is None:
            process.kill()
            process.communicate()
    require(digest(directory/'freeze.json') == freeze_sha, 'changed pretrial freeze')
    result = finish(directory, argv,
        subprocess.CompletedProcess(argv, process.returncode, stdout, stderr), expected)
    result.update(stage=stage, freeze_sha256=freeze_sha,
        persistent_frozen_mismatches=[n for n, h in frozen['files'].items() if digest(directory/n) != h])
    if stage == 'between_validation_and_guard':
        binding = str(BRIDGE/'source-bindings.json')
        require(result['persistent_frozen_mismatches'] == [binding], 'not exact binding mutation')
        require(result['receipt']['artifacts'][str(directory/binding)]['sha256'] == digest(directory/binding),
            'not changed binding snapshot')
        require(digest(directory/'extraction/receipt.json') != digest(RAW/'receipt.json'), 'original receipt')
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--race-only', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    results = dict(base=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        required={str(n): digest(ROOT/n) for n in REQUIRED}, classification='provenance/eligibility, not semantic proofs',
        cases=[])
    for optimized in (False, True):
        for tool in TOOLS:
            prefix = ('optimized' if optimized else 'normal')+'/'+tool
            if not args.race_only:
                cases = [('healthy', None, None)]
                cases += [('omission-'+str(i), n, 'omit') for i, n in enumerate(REQUIRED)]
                cases += [('mutation-'+str(i), n, 'mutate') for i, n in enumerate(REQUIRED)]
                cases += [('synthetic-omitted', BRIDGE/'source-bindings.json', 'synthetic-omit'),
                          ('synthetic-complete', BRIDGE/'source-bindings.json', 'synthetic-complete'),
                          ('synthetic-receipt-only', None, 'synthetic-receipt-only')]
                for label, name, action in cases:
                    directory = output/prefix/label
                    frozen = prepare(directory)
                    expected = None
                    if action in ('omit', 'synthetic-omit'):
                        del frozen['files'][str(name)]
                        expected = 'incomplete frozen source-audit inputs'
                    write(directory/'freeze.json', frozen)  # precedes every mutation
                    if action == 'mutate':
                        with (directory/name).open('a') as stream:
                            stream.write('\n')
                        expected = 'frozen source changed'
                    if action in ('synthetic-omit', 'synthetic-complete'):
                        synthetic(directory)
                        if action == 'synthetic-complete':
                            expected = 'frozen source changed'
                    if action == 'synthetic-receipt-only':
                        (directory/'extraction/receipt.json').write_text('{"success":true}\n')
                        expected = 'unbound extraction receipt'
                    item = run_case(directory, tool, optimized, frozen, expected)
                    item.update(name=prefix+'/'+label, input=str(name) if name else None)
                    results['cases'].append(item)
            for stage in ('between_validation_and_guard', 'after_initial_guard'):
                directory = output/prefix/stage
                frozen = prepare(directory)
                item = race_case(directory, tool, optimized, frozen, stage)
                item['name'] = prefix+'/'+stage
                results['cases'].append(item)
                print(item['name'], item['returncode'], 'success='+str(item['receipt']['success']), flush=True)
    write(output/'results.json', results)
    print(json.dumps(dict(results=str(output/'results.json'), sha256=digest(output/'results.json'),
        cases=len(results['cases']))))


if __name__ == '__main__':
    main()
