#!/usr/bin/env python3
"""Check Aleph's data-bearing validation errors and original noninitial ordering.

The error enum, own-parent lookup and noninitial orchestration retain pinned
source bodies. Count/search pipelines use explicit checked collection queries.
"""
import argparse
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import item, FIXTURES, ROOT, NODE, CORRECTED, prepare, validate_native
from eval_verus_infer_alephbft_traits import UNITS
from eval_verus_infer_alephbft_units import real_api, CONTROL, HASHER, CODEC_MODEL
from eval_verus_infer_alephbft_native_collector import COMPARISONS, QUERY_MODELS
from eval_verus_infer_alephbft_private import validate
from infer_eval_support import ArtifactGuard, host_info, measured, tool_artifacts

MODELS = FIXTURES / 'validation_models.rs'
RUNTIME = FIXTURES / 'validation_runtime.rs'
NAMES = ('unit_creator_is_descendant_of_previous_unit',
         'previous_round_have_enough_parents',
         'check_if_parents_greater_than_previous_round', 'validate_non_initial_round')


def validation_api(control):
    error = item(control, 'pub enum Error')[2]
    implementation = item(control, 'impl<H: Hasher> ControlHash<H>')[2]
    methods = []
    for name in NAMES:
        method = item(implementation, 'fn '+name+'(')[2]
        if name == 'previous_round_have_enough_parents':
            old = '''self
            .parents()
            .filter(|&parent| parent.round == round - 1)
            .count()'''
            if method.count(old) != 1:
                raise ValueError('native filtered-count pipeline changed')
            method = method.replace(old, 'self.parents.rounds_in_range_checked(round - 1, round - 1)')
        elif name == 'check_if_parents_greater_than_previous_round':
            old = '''self
            .parents()
            .any(|unit_coord| unit_coord.round > round - 1)'''
            if method.count(old) != 1:
                raise ValueError('native future-parent pipeline changed')
            method = method.replace(old, 'self.parents.contains_after_checked(round - 1)')
        methods.append('#[verifier::infer]\n'+method)
    return MODELS.read_text().replace('// NATIVE_ERROR_ENUM', error).replace(
        '// NATIVE_VALIDATION_METHODS', '\n'.join(methods))


def infer_member_count(api, control):
    """The original count wrapper is already within bounded record inference."""
    start, end, _ = item(api, 'pub fn n_members(')
    original = item(control, 'pub fn n_members(')[2]
    return api[:start]+'#[verifier::infer]\n'+original+api[end:]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--alephbft', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--verus', type=Path, default=ROOT/'source/target-verus/release/verus')
    parser.add_argument('--z3', type=Path, default=ROOT/'source/z3')
    parser.add_argument('--native-tests', action='store_true')
    parser.add_argument('--native-target', type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error('output must be new or empty')
    output.mkdir(parents=True, exist_ok=True)
    verus, z3 = args.verus.resolve(), args.z3.resolve()
    extras = list(Path(__file__).parent.glob('eval_verus_infer_alephbft*.py'))
    extras += list(FIXTURES.glob('*.rs')) + [Path(__file__).with_name('infer_eval_support.py')]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extras))
    result = dict(host=host_info(), tool_artifacts=guard.initial, runs=[], native=[])
    def save():
        (output/'results.json').write_text(json.dumps(result, indent=2)+'\n')
    try:
        shared, origin = prepare(args.alephbft.resolve(), output/'checkout')
        checkout = output/'checkout'
        directory = checkout/'verification-infer'
        node, units, control, hasher = [git(checkout, 'show', f'{REVISION}:{path}')
            for path in (NODE, UNITS, CONTROL, HASHER)]
        shared = shared.replace('(*self * 2) / 3 + NodeCount(1)', CORRECTED)
        api = infer_member_count(real_api(node, shared, units, control, hasher), control)
        api += COMPARISONS + QUERY_MODELS.read_text() + validation_api(control)
        result['source'] = dict(**origin,
            bodies='original six-variant Error<H>, own-parent lookup, n_members, runtime assertion and three-stage ? orchestration',
            adaptations='two iterator pipelines replaced by ordinary checked finite collection queries; positive-round validation scope',
            supplied='direct collection view, query equations, eligible policy, exact error goals, state constructor',
            boundaries='existing explicit SCALE codec and verified-lock boundaries; corrected quorum arithmetic; round remains u16',
            not_claimed='native iterator-pipeline inference, initial-round hash validation, full crate/protocol correctness; isolated future-parent leaf at round zero')
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK='20971520',
                   VERUS_INFER_CACHE=str(output/'candidate-cache'))
        codec_lib, codec_vir = directory/'libcodec_model.rlib', directory/'codec_model.vir'
        guard.check()
        run = measured([str(verus), str(CODEC_MODEL), '--crate-name=codec_model', '--crate-type=lib',
            '--compile', '--export', str(codec_vir), '-o', str(codec_lib), '--output-json'], env, 90, ROOT)
        guard.check()
        (output/'codec.stdout.json').write_text(run['stdout'])
        (output/'codec.stderr.txt').write_text(run['stderr'])
        if run['returncode'] or not json.loads(run['stdout'])['verification-results']['success']:
            raise ValueError('declared codec model failed')
        codec_guard = ArtifactGuard([codec_lib, codec_vir])
        path, interface, library = directory/'validation.rs', directory/'validation.vir', directory/'libvalidation.rlib'
        command = [str(verus), str(path), '--crate-name=validation', '--crate-type=lib', '--compile',
            '--export', str(interface), '-o', str(library), '--infer-goals', '--no-cheating', '--output-json',
            '--extern', f'codec_model={codec_lib}', '--import', f'codec_model={codec_vir}',
            '-L', f'dependency={directory}']
        cases = [
            ('validation', api, True),
            ('wrong-own-payload', api.replace('Error::DescendantOfPreviousUnitHasWrongRound(parent_round)',
                'Error::DescendantOfPreviousUnitHasWrongRound(unit_coord.round)'), False),
            ('wrong-error-tag', api.replace('Error::NotEnoughParentsForRound(round - 1)',
                'Error::ParentsHigherThanRound(round - 1)'), False),
            ('ignored-error', api.replace('self.unit_creator_is_descendant_of_previous_unit(unit_coord)?;',
                'let _ = self.unit_creator_is_descendant_of_previous_unit(unit_coord);'), False),
            ('wrong-error-priority', api.replace(
                'self.previous_round_have_enough_parents(unit_coord.round)?;\n        self.check_if_parents_greater_than_previous_round(unit_coord.round)?;',
                'self.check_if_parents_greater_than_previous_round(unit_coord.round)?;\n        self.previous_round_have_enough_parents(unit_coord.round)?;'), False),
            ('wrong-query-callback', api.replace('lo <= *entry.1 && *entry.1 <= hi',
                'lo < *entry.1 && *entry.1 <= hi'), False),
            ('missing-creator-bound', api.replace('coord.parts().1 > 0, coord.parts().0.0 < self@.0.len(),',
                'coord.parts().1 > 0,'), False),
            ('missing-round-bound', api.replace('coord.parts().1 > 0, coord.parts().0.0 < self@.0.len(),',
                'coord.parts().0.0 < self@.0.len(),'), False),
            ('validation-warm', api, True),
        ]
        for case, text, expected in cases:
            if not expected and text == api:
                raise ValueError('ineffective mutation: '+case)
            path.write_text(text)
            (output/(case+'.rs')).write_text(text)
            guard.check(); codec_guard.check()
            run = measured(command, env, 120, ROOT)
            guard.check(); codec_guard.check()
            (output/(case+'.stdout.json')).write_text(run['stdout'])
            (output/(case+'.stderr.txt')).write_text(run['stderr'])
            data = validate(run, expected, 15)
            if not expected:
                if interface.exists():
                    raise ValueError('failed provider published interface')
                if not any(s in run['stderr'] for s in ('postcondition not satisfied', 'precondition not satisfied', 'assertion failed',
                        'unable to prove post-condition of closure')):
                    raise ValueError('negative control lacks semantic proof failure')
            result['runs'].append(dict(case=case, verification=data['verification-results'],
                report=data['verus-infer'], wall_micros=run['wall_micros']))
            save()
            print(case+': expected outcome', flush=True)
        runtime_source = api + '\n#[cfg(verus_keep_ghost_body)] fn main() {}\n' + (
            '#[cfg(not(verus_keep_ghost_body))] mod runtime { use super::*;\n'+RUNTIME.read_text()+
            '\npub fn run(){validation_corpus();}}\n#[cfg(not(verus_keep_ghost_body))] fn main(){runtime::run();}\n')
        path.write_text(runtime_source)
        runtime_command = [arg for arg in command if arg != '--crate-type=lib']
        runtime_command[runtime_command.index(str(library))] = str(directory/'validation-runtime')
        guard.check(); codec_guard.check()
        run = measured(runtime_command, env, 120, ROOT)
        guard.check(); codec_guard.check()
        (output/'runtime.stdout.json').write_text(run['stdout'])
        (output/'runtime.stderr.txt').write_text(run['stderr'])
        data = validate(run, True, 15)
        result['runs'].append(dict(case='runtime-build', verification=data['verification-results'],
            report=data['verus-infer'], wall_micros=run['wall_micros']))
        run = measured([str(directory/'validation-runtime')], env, 30, ROOT)
        (output/'runtime.txt').write_text(run['stdout']+run['stderr'])
        if run['returncode'] or 'INFER_VALIDATION_CASES=7704;ZERO_ROUND_PANIC=1' not in run['stdout']:
            raise ValueError('adapted runtime corpus did not complete')
        result['native'].append(dict(case='adapted-runtime', returncode=run['returncode'], wall_micros=run['wall_micros']))
        if args.native_tests:
            end = control.rfind('}')
            (checkout/CONTROL).write_text(control[:end]+RUNTIME.read_text()+
                '\n#[test] fn inferred_validation_corpus(){validation_corpus();}\n'+control[end:])
            native_env = dict(os.environ)
            native_env.pop('RUSTUP_TOOLCHAIN', None)
            native_env['CARGO_TARGET_DIR'] = str((args.native_target or output/'native-target').resolve())
            guard.check(); codec_guard.check()
            run = measured(['cargo', 'test', '--offline', '-p', 'aleph-bft', '--lib',
                'units::control_hash::tests::', '--', '--test-threads=1', '--nocapture'], native_env, 600, checkout)
            guard.check(); codec_guard.check()
            (output/'native.stdout.txt').write_text(run['stdout'])
            (output/'native.stderr.txt').write_text(run['stderr'])
            validate_native(run, True, 'native-validation')
            if 'INFER_VALIDATION_CASES=7704;ZERO_ROUND_PANIC=1' not in run['stdout']:
                raise ValueError('untouched native corpus did not complete')
            result['native'].append(dict(case='untouched-native', returncode=run['returncode'], wall_micros=run['wall_micros']))
        result['success'] = True
    except Exception as error:
        result['success'], result['error'] = False, str(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
