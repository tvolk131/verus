#!/usr/bin/env python3
"""Persistent-file race regressions; no semantic or native acceptance claims."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path[:0] = [str(ROOT/'source/tools'), str(HERE)]
from infer_eval_support import digest
from infer_experiment import require, replay_receipt
from source_map import guard_source_freeze

REVIEW = ROOT/'evidence/source-freeze-completeness-review/recheck.py'
spec = importlib.util.spec_from_file_location('frozen_race_review', REVIEW)
review = importlib.util.module_from_spec(spec)
spec.loader.exec_module(review)


def invoke(directory, argv, trigger, count, mutate, failure, frozen):
    review.write(directory/'freeze.json', frozen)
    before = digest(directory/'freeze.json')
    fd = review.inotify_open(trigger)
    process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        review.await_open(fd, count)
        mutate()
        stdout, stderr = process.communicate(timeout=90)
    finally:
        os.close(fd)
        if process.poll() is None:
            process.kill()
            process.communicate()
    require(digest(directory/'freeze.json') == before, 'pretrial freeze changed')
    record = review.finish(directory, argv,
        subprocess.CompletedProcess(argv, process.returncode, stdout, stderr), failure)
    record['persistent_mismatches'] = [n for n, h in frozen['files'].items()
                                      if digest(directory/n) != h]
    require(record['persistent_mismatches'], 'no persistent source mutation occurred')
    record['freeze_sha256'] = before
    return record


def ballast(directory, frozen):
    path = directory/'ballast.bin'
    with path.open('wb') as stream:
        stream.truncate(128*1024*1024)
    frozen['files']['ballast.bin'] = digest(path)
    return path


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    result = dict(schema=1, cases=[], publication=False, native_bridge_proved=False,
                  semantic_negative_count=0, classification='artifact/provenance refusals')
    with replay_receipt(a.output, result):
        f = json.loads(a.freeze.read_bytes())
        require({str(Path(__file__).relative_to(ROOT)), str(REVIEW.relative_to(ROOT))}
                <= set(f['files']), 'unfrozen race regression tools')
        guard = guard_source_freeze(a.freeze, review.RAW)
        for optimized in (False, True):
            mode = 'optimized' if optimized else 'normal'
            for tool in review.TOOLS:
                for stage in ('during_initial_snapshot', 'after_initial_snapshot'):
                    directory = a.output/mode/tool/stage
                    frozen = review.prepare(directory)
                    if stage == 'during_initial_snapshot':
                        trigger, count = ballast(directory, frozen), 1
                        mutate = lambda: review.synthetic(directory)
                        failure = 'frozen source changed'
                    else:
                        trigger, count = directory/'extraction/receipt.json', 2
                        def mutate():
                            with (directory/'source/tools/infer_experiment.py').open('a') as stream:
                                stream.write('\n# persistent post-snapshot control\n')
                        failure = 'tool inputs changed during evaluation'
                    record = invoke(directory, review.command(directory, tool, optimized),
                                    trigger, count, mutate, failure, frozen)
                    record.update(mode=mode, controller=tool, stage=stage)
                    result['cases'].append(record)

            # Preserve the former validation-to-snapshot window in the proof
            # controller. The changed candidate is only a comment; refusal must
            # precede every prover command, rather than rely on a failed proof.
            directory = a.output/mode/'proof-controller'
            directory.mkdir(parents=True)
            for name in f['files']:
                require(not Path(name).is_absolute(), 'nonlocal control input')
                target = directory/name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT/name, target)
            frozen = json.loads(json.dumps(f))
            trigger = ballast(directory, frozen)
            candidate = directory/frozen['candidate_directory']/'borrow_handoff.v'
            def mutate():
                with candidate.open('a') as stream:
                    stream.write('\n(* persistent pre-snapshot control *)\n')
            output = directory/'output'
            ready = json.loads((HERE.parent/'mutable_buffer_goal/foundation-baseline.json').read_text())['readiness']['path']
            argv = [sys.executable, '-B', *(['-O'] if optimized else []),
                    str(directory/HERE.relative_to(ROOT)/'check.py'), '--freeze',
                    str(directory/'freeze.json'), '--ready', ready, '--output', str(output)]
            record = invoke(directory, argv, trigger, 1, mutate, 'freeze mismatch', frozen)
            receipt = record['receipt']
            require(receipt['commands'] == [] and receipt['model_proofs_kernel_checked'] is False,
                    'prover ran or proof status published for changed input')
            record.update(mode=mode, controller='check.py', stage='before_initial_snapshot')
            result['cases'].append(record)
        require(len(result['cases']) == 10, 'race control coverage')
        result['artifacts'] = guard.initial
        guard.check()


if __name__ == '__main__':
    main()
