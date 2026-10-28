#!/usr/bin/env python3
"""Infer all four original UnitsCollector bodies and check supplied application goals.

Keeps the pinned u16 round, early returns, guarded matches, and error policy.
The Unit/Hasher provider retains the explicit codec and verified-lock boundaries.
"""
import argparse
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import ROOT, FIXTURES, NODE, CORRECTED, item, prepare, validate_native
from eval_verus_infer_alephbft_traits import UNITS
from eval_verus_infer_alephbft_private import validate
from eval_verus_infer_alephbft_units import real_api, CONTROL, HASHER, CODEC_MODEL
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

COLLECTOR = 'consensus/src/creation/collector.rs'
GOALS = FIXTURES / 'native_collector_goals.rs'
COMPARISONS = '''
#[verus_infer]
impl PartialEq for NodeCount { fn eq(&self, rhs: &Self) -> bool { self.0 == rhs.0 } }
#[verus_infer]
impl PartialOrd for NodeCount {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> { self.0.partial_cmp(&rhs.0) }
    fn lt(&self, rhs: &Self) -> bool { self.0 < rhs.0 }
}
'''


def collector(original):
    declaration = item(original, 'pub struct UnitsCollector')[2]
    implementation = item(original, 'impl<H: Hasher> UnitsCollector<H>')[2]
    for name in ('new_initial', 'from_previous', 'add_unit', 'prospective_parents'):
        marker = 'pub fn ' + name
        implementation = implementation.replace(marker, '#[verifier::infer]\n    ' + marker, 1)
        # Check each executable body independently, including original early exits.
        old = item(original, marker)[2].split('{', 1)[1]
        new = item(implementation, marker)[2].split('{', 1)[1]
        if old != new:
            raise ValueError('native executable body changed: ' + name)
    return '''use vstd::prelude::*;
use units_api::{Unit, Hasher, NodeCount, NodeIndex, NodeMap, Round};
verus! {
broadcast use vstd::std_specs::clone::lemma_cloned_identity;
#[derive(Copy, Clone, Debug)]
pub enum ConstraintError { NotEnoughParents, MissingOwnParent }
#[verifier::allow(autoderive_clone_without_spec)]
#[derive(Clone)]
''' + declaration + '\n' + implementation + '\n}\n' + GOALS.read_text()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--alephbft', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--verus', type=Path, default=ROOT/'source/target-verus/release/verus')
    p.add_argument('--z3', type=Path, default=ROOT/'source/z3')
    p.add_argument('--native-tests', action='store_true')
    p.add_argument('--native-target', type=Path)
    args = p.parse_args()
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        p.error('output must be new or empty')
    output.mkdir(parents=True, exist_ok=True)
    verus, z3 = args.verus.resolve(), args.z3.resolve()
    extras = [Path(__file__), GOALS, CODEC_MODEL, FIXTURES/'full_unit_models.rs']
    extras += list(Path(__file__).parent.glob('eval_verus_infer_alephbft*.py'))
    extras += [Path(__file__).with_name('infer_eval_support.py')]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extras))
    result = dict(host=host_info(), tool_artifacts=guard.initial, runs=[], native=[])
    def save():
        (output/'results.json').write_text(json.dumps(result, indent=2)+'\n')
    try:
        shared, origin = prepare(args.alephbft.resolve(), output/'checkout')
        checkout = output/'checkout'
        directory = checkout/'verification-infer'
        node, units, control, hasher, original = [git(checkout, 'show', f'{REVISION}:{path}')
            for path in (NODE, UNITS, CONTROL, HASHER, COLLECTOR)]
        shared = shared.replace('(*self * 2) / 3 + NodeCount(1)', CORRECTED)
        api = real_api(node, shared, units, control, hasher) + COMPARISONS
        source = collector(original)
        result['source'] = dict(**origin, collector=COLLECTOR, goals_sha256=digest(GOALS),
            bodies='all four original UnitsCollector methods, byte-identical executable bodies',
            supplied='application invariant, exact slot/frame/selection goals, checked scalar comparison adapters',
            boundaries='same Unit/Hasher codec and verified-lock model as units experiment; corrected quorum arithmetic; round remains u16',
            not_claimed='whole-crate verification, arbitrary native Clone, SCALE algorithm, parking_lot RAII, terminal-round recovery')
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK='20971520',
                   VERUS_INFER_CACHE=str(output/'candidate-cache'))
        codec_lib, codec_vir = directory/'libcodec_model.rlib', directory/'codec_model.vir'
        command = [str(verus), str(CODEC_MODEL), '--crate-name=codec_model', '--crate-type=lib',
                   '--compile', '--export', str(codec_vir), '-o', str(codec_lib), '--output-json']
        guard.check()
        run = measured(command, env, 90, ROOT)
        guard.check()
        (output/'codec.stdout.json').write_text(run['stdout'])
        (output/'codec.stderr.txt').write_text(run['stderr'])
        if run['returncode'] or not json.loads(run['stdout'])['verification-results']['success']:
            raise ValueError('declared codec model failed')
        codec_guard = ArtifactGuard([codec_lib, codec_vir])
        codec_args = ['--extern', f'codec_model={codec_lib}', '--import', f'codec_model={codec_vir}',
                      '-L', f'dependency={directory}']
        apifile, target = directory/'units_api.rs', directory/'native_collector.rs'
        lib, interface = directory/'libunits_api.rlib', directory/'units_api.vir'
        def check(case, path, text, command, expected, count, goal=None):
            path.write_text(text)
            (output/(case+'.rs')).write_text(text)
            guard.check(); codec_guard.check()
            run = measured(command, env, 90, ROOT)
            guard.check(); codec_guard.check()
            (output/(case+'.stdout.json')).write_text(run['stdout'])
            (output/(case+'.stderr.txt')).write_text(run['stderr'])
            data = validate(run, expected, count)
            if goal:
                if not any(n.endswith('::'+goal) for n in data['verus-infer']['failed_functions']):
                    raise ValueError('expected named application goal failure: '+goal)
                if not any(s in run['stderr'] for s in ('postcondition not satisfied', 'precondition not satisfied')):
                    raise ValueError('negative control lacks semantic proof failure')
            result['runs'].append(dict(case=case, verification=data['verification-results'],
                report=data['verus-infer'], wall_micros=run['wall_micros']))
            save()
            print(case+': expected outcome', flush=True)
        provider = [str(verus), str(apifile), '--crate-name=units_api', '--crate-type=lib', '--compile',
                    '--export', str(interface), '-o', str(lib), '--infer-goals', '--no-cheating', '--output-json']+codec_args
        check('provider', apifile, api, provider, True, 10)
        consumer = [str(verus), str(target), '--crate-type=lib', '--infer-goals', '--no-cheating', '--output-json',
                    '--extern', f'units_api={lib}', '--import', f'units_api={interface}']+codec_args
        cases = [
            ('native', source, True, None),
            ('wrong-counter', source.replace('self.direct_parents += NodeCount(1)', 'self.direct_parents += NodeCount(0)'), False, 'check_add'),
            ('wrong-next-count', source.replace('direct_parents: NodeCount(0)', 'direct_parents: NodeCount(1)'), False, 'check_next'),
            ('wrong-error', source.replace('return Err(ConstraintError::NotEnoughParents)', 'return Err(ConstraintError::MissingOwnParent)'), False, 'check_selection'),
            ('missing-round-bound', source.replace('previous.round()<u16::MAX,', ''), False, 'check_next'),
            ('missing-node-bound', source.replace('unit.creator_value().0<old(self).parents().len(),', ''), False, 'check_add'),
            ('native-warm', source, True, None),
        ]
        for case, variant, expected, goal in cases:
            if not expected and variant == source:
                raise ValueError('mutation did not change source: '+case)
            check(case, target, variant, consumer, expected, 4, goal)
        if args.native_tests:
            native_env = dict(os.environ)
            native_env.pop('RUSTUP_TOOLCHAIN', None)
            native_env['CARGO_TARGET_DIR'] = str((args.native_target or output/'native-target').resolve())
            command = ['cargo', 'test', '--offline', '-p', 'aleph-bft', '--lib',
                       'creation::collector::tests::', '--', '--test-threads=1']
            guard.check(); codec_guard.check()
            run = measured(command, native_env, 600, checkout)
            guard.check(); codec_guard.check()
            (output/'native-tests.stdout.txt').write_text(run['stdout'])
            (output/'native-tests.stderr.txt').write_text(run['stderr'])
            validate_native(run, True, 'original-native-collector')
            result['native'].append(dict(returncode=run['returncode'], wall_micros=run['wall_micros']))
        result['success'] = True
    except Exception as error:
        result['success'], result['error'] = False, str(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
