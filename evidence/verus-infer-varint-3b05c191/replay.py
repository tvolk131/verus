#!/usr/bin/env python3
"""Run fresh checks; a reproduced blocker is not an application proof."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts


def source_guard():
    freeze = json.loads((HERE / 'freeze.json').read_text())
    for name, expected in freeze['pretrial_files_sha256'].items():
        if digest(HERE / name) != expected:
            raise RuntimeError('frozen input changed: ' + name)
    extra = json.loads((HERE / 'inference-caller-freeze.json').read_text())
    for name, expected in extra['files_sha256'].items():
        if digest(HERE / name) != expected:
            raise RuntimeError('frozen caller changed: ' + name)
    for name, expected in json.loads((HERE / 'layout-refinement.json').read_text())['files_sha256'].items():
        if digest(HERE / name) != expected:
            raise RuntimeError('layout-refinement input changed: ' + name)
    original = (HERE / 'original-selected.rs').read_text()
    upstream = (HERE / 'upstream/src/encode.rs').read_text()
    macro_start = upstream.index('macro_rules! encode {')
    upstream_macro = upstream[macro_start:upstream.index('\n\n\n/// Encode', macro_start)]
    fn_start = upstream.index('pub fn u64(')
    upstream_fn = upstream[fn_start:upstream.index('\n}', fn_start) + 2]
    const_start = upstream.index('const U64_LEN:')
    upstream_const = upstream[const_start:upstream.index(';', const_start) + 1]
    if upstream_macro + '\n\n' + upstream_fn + '\n\n' + upstream_const + '\n' != original:
        raise RuntimeError('selected items differ from pinned upstream source')
    macro, rest = original.split('\npub fn u64', 1)
    wrapper = 'pub fn u64' + rest.split('\n\nconst ', 1)[0]
    for name in ['ordinary', 'inferred', 'adapted-ordinary', 'adapted-inferred',
                 'inferred-caller', 'adapted-inferred-caller']:
        src = (HERE / 'fixtures' / (name + '.rs')).read_text()
        if name.startswith('adapted-'):
            src = src.replace('=> { verus_exec_expr! {{', '=> {{').replace('    }} }\n}', '    }}\n}')
        if macro not in src or wrapper not in src:
            raise RuntimeError('selected executable body drift: ' + name)
    return freeze


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-root', type=Path, required=True)
    parser.add_argument('--verus', type=Path, default=ROOT / 'source/target-verus/release/verus')
    parser.add_argument('--solver', type=Path, default=ROOT / 'source/z3')
    parser.add_argument('--archive', type=Path, help='optional published 0.8.0 .crate to check against frozen source')
    parser.add_argument('--stage', choices=['baseline', 'diagnostics', 'manual', 'native', 'publication', 'all'], default='all')
    parser.add_argument('--case', help='run only this diagnostic/manual fixture stem')
    args = parser.parse_args()
    freeze = source_guard()
    if args.archive:
        if digest(args.archive) != freeze['source']['archive_sha256']:
            raise RuntimeError('archive digest mismatch')
        with tarfile.open(args.archive, mode='r:gz') as archive:
            for name in ['src/encode.rs', 'LICENSE', '.cargo_vcs_info.json']:
                if archive.extractfile('unsigned-varint-0.8.0/' + name).read() != (HERE / 'upstream' / name).read_bytes():
                    raise RuntimeError('archive member differs: ' + name)
    args.output_root.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='varint-', dir=args.output_root)).resolve()
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'], RUSTC_BOOTSTRAP='1',
               VERUS_Z3_PATH=str(args.solver.resolve()))
    rustc = subprocess.check_output(['/home/dev/.cargo/bin/rustup', 'which', 'rustc'], cwd=ROOT, env=env, text=True).strip()
    driver = list((Path(rustc).parents[1] / 'lib').glob('librustc_driver*'))
    inputs = [HERE / 'replay.py', HERE / 'freeze.json', HERE / 'inference-caller-freeze.json', HERE / 'layout-refinement.json',
              *sorted(HERE.rglob('*.rs'))]
    guard = ArtifactGuard(tool_artifacts(args.verus, args.solver, [rustc, *driver, *inputs]))
    report = dict(schema=1, engine_baseline=freeze['engine_baseline'], host=host_info(),
                  stage=args.stage, output=str(output), application_proofs_proved=False,
                  freeze_sha256=digest(HERE / 'freeze.json'), artifacts=guard.initial, cases=[])

    def run(name, fixture, flags, phase):
        work = output / name
        work.mkdir(exist_ok=True)
        cache = work / 'cache'
        cache.mkdir(exist_ok=True)
        run_env = dict(env, VERUS_INFER_CACHE=str(cache))
        export = work / 'provider.vir'
        cmd = [str(args.verus.resolve()), str(fixture), '--crate-type=lib', '--crate-name', 'varint_trial',
               '--no-cheating', '--output-json', '--export', str(export), '--out-dir', str(work), *flags]
        guard.check()
        (work / (phase + '.source.rs')).write_bytes(fixture.read_bytes())
        result = measured(cmd, run_env, 180, ROOT)
        guard.check()
        for stream in ['stdout', 'stderr']:
            (work / (phase + '.' + stream)).write_text(result[stream])
        row = dict(name=name, phase=phase, fixture=str(fixture.relative_to(HERE)),
                   source_sha256=digest(fixture), command=cmd, returncode=result['returncode'],
                   wall_micros=result['wall_micros'], export_exists=export.exists(),
                   cache_files=len(list(cache.rglob('*.json'))),
                   stdout_sha256=digest(work / (phase + '.stdout')),
                   stderr_sha256=digest(work / (phase + '.stderr')))
        try:
            row['verifier_json'] = json.loads(result['stdout'])
        except json.JSONDecodeError:
            row['verifier_json'] = None
        row['error_lines'] = [line for line in result['stderr'].splitlines()
                              if line.startswith(('error', '  = help:', '  = note:'))]
        report['cases'].append(row)
        (output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
        print(name, phase, row['returncode'], row['error_lines'], flush=True)

    if args.stage in ['baseline', 'all']:
        for name in ['ordinary', 'inferred', 'adapted-ordinary', 'adapted-inferred',
                     'inferred-caller', 'adapted-inferred-caller']:
            for assisted in [False, True]:
                case = name + ('-goals' if assisted else '-plain')
                for phase in ['cold', 'warm']:
                    run(case, HERE / 'fixtures' / (name + '.rs'), ['--infer-goals'] if assisted else [], phase)
    if args.stage in ['diagnostics', 'all']:
        for fixture in sorted((HERE / 'diagnostics').glob('*.rs')):
            if args.case and fixture.stem != args.case:
                continue
            variants = [False, True] if fixture.stem.startswith('application-') else [True]
            for assisted in variants:
                for phase in ['cold', 'warm']:
                    case = fixture.stem + ('-goals' if assisted else '-plain')
                    flags = ['--multiple-errors', '20']
                    if assisted:
                        flags.append('--infer-goals')
                    run(case, fixture, flags, phase)
    if args.stage in ['native', 'all']:
        report['native_supporting_oracles'] = []
        bundle = args.verus.resolve().parent
        for mode, opt, debug in [('debug', '0', 'yes'), ('release', '3', 'no')]:
            binary = output / ('native-' + mode)
            command = [rustc, str(HERE / 'native.rs'), '--edition=2021', '--test',
                       '--extern', 'vstd=' + str(bundle / 'libvstd.rlib'),
                       '-L', 'dependency=' + str(bundle), '-o', str(binary),
                       '-C', 'opt-level=' + opt, '-C', 'debug-assertions=' + debug]
            for step in ['build', 'run']:
                guard.check()
                result = measured(command, env, 180, ROOT)
                guard.check()
                for stream in ['stdout', 'stderr']:
                    (output / f'native-{mode}-{step}.{stream}').write_text(result[stream])
                report['native_supporting_oracles'].append(dict(
                    mode=mode, step=step, command=command, returncode=result['returncode'],
                    wall_micros=result['wall_micros']))
                if result['returncode'] != 0:
                    raise RuntimeError('native ' + mode + ' ' + step + ' failed; see ' + str(output))
                command = [str(binary)]
    if args.stage in ['manual', 'all']:
        for fixture in sorted((HERE / 'manual').glob('*.rs')):
            if args.case and fixture.stem != args.case:
                continue
            for assisted in [False, True]:
                for phase in ['cold', 'warm']:
                    case = fixture.stem + ('-goals' if assisted else '-plain')
                    flags = ['--multiple-errors', '20']
                    if assisted:
                        flags.append('--infer-goals')
                    run(case, fixture, flags, phase)
    if args.stage in ['publication', 'all']:
        run('seeded-length', HERE / 'manual/length-attempt.rs', ['--infer-goals'], 'seed')
        run('seeded-length', HERE / 'manual/length-negative.rs', ['--infer-goals'], 'mutant')
        run('seeded-inference', HERE / 'diagnostics/engine-positive.rs', ['--infer-goals'], 'seed')
        run('seeded-inference', HERE / 'diagnostics/application-no-layout-inferred.rs', ['--infer-goals'], 'reject')
    source_guard()
    guard.check()
    report['artifacts_unchanged'] = True
    report['completed'] = True
    (output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(output, flush=True)


if __name__ == '__main__':
    main()
