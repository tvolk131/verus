#!/usr/bin/env python3
"""Six registered ordinary callee checks; existing replay/coverage helpers only."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run, require_exact, require_function_coverage, require_failure_kind, require_publication
from infer_eval_support import ArtifactGuard, digest


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--freeze', type=Path, default=HERE / 'freeze.json')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    receipt = {'kind': 'P0_CALLEE_PROBES_NOT_CALLER_ACCEPTANCE', 'commands': [], 'outcomes': [], 'application_goal_complete': False}
    with replay_receipt(args.output, receipt):
        freeze = json.loads(args.freeze.read_text())
        paths = [Path(p) if Path(p).is_absolute() else ROOT / p for p in freeze['inputs']]
        for name, path in zip(freeze['inputs'], paths):
            require(digest(path) == freeze['inputs'][name], 'input identity changed: ' + name)
        accepted = Path(freeze['runtime'])
        bundle = args.output / 'bundle'
        bundle.mkdir()
        manifest = json.loads((accepted / 'receipt.json').read_text())
        require(manifest['success'] is True, 'accepted runtime is not successful')
        for name, expected in manifest['tools'].items():
            require(digest(accepted / name) == expected, 'wrong runtime tool: ' + name)
            shutil.copy2(accepted / name, bundle / name)
        (bundle / 'verus-root').touch()
        tools = [bundle / name for name in manifest['tools']]
        spec = importlib.util.spec_from_file_location('previous_scope', ROOT / freeze['scope_utility'])
        previous = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(previous)
        original = (ROOT / freeze['r2_provider']).read_text()
        redb = original + '\n' + (HERE / 'redb-tail.rs').read_text()
        node = (HERE / 'node-slot.rs').read_text()
        texts = {
            'redb': redb,
            'node': node,
            'redb_wrong': redb.replace('final(self).len == old(self).len,', 'final(self).len as int == old(self).len as int + 1,'),
            'node_wrong': node.replace('Some(value)),', 'None),'),
        }
        require(texts['redb_wrong'] != redb and texts['node_wrong'] != node, 'negative control did not change a contract')
        fixtures = {}
        for label, text in texts.items():
            fixtures[label] = args.output / (label + '.rs')
            fixtures[label].write_text(text)
        guard = ArtifactGuard(paths + tools + list(fixtures.values()) + [args.freeze])
        receipt['inputs'] = {str(p): digest(p) for p in paths + tools + list(fixtures.values())}
        toolchain = '/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu'
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + toolchain + '/bin:/usr/bin:/bin',
                   RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',
                   VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu', LD_LIBRARY_PATH=toolchain + '/lib',
                   VERUS_Z3_PATH=str(bundle / 'z3'), VERUS_INFER_CACHE=str(args.output / 'cache'))
        run = LoggedCommands(args.output, args.output, receipt['commands'], timeout=180, guard=guard)
        cases = [('redb', False), ('redb', True), ('node', False), ('node', True), ('redb_wrong', True), ('node_wrong', True)]
        for case, goals in cases:
            label = case + ('_on' if goals else '_off')
            source = fixtures[case]
            artifact = args.output / (label + '.vir')
            positive = not case.endswith('_wrong')
            cmd = [str(bundle / 'verus'), str(source), '--crate-type=lib', '--edition=2024', '--crate-name', label, '--no-cheating', '--output-json', '--time', '--multiple-errors', '20']
            if goals:
                artifact.write_text('seeded stale interface\n')
                cmd += ['--infer-goals', '--export', str(artifact)]
            code = run(label, cmd, env)
            stderr = (args.output / (label + '.stderr')).read_text()
            data = validate_run({'returncode': code, 'stdout': (args.output / (label + '.stdout')).read_text(), 'stderr': stderr}, positive)
            inf = data['verus-infer']
            require_exact(inf['functions'])
            require_function_coverage(inf['functions'], [])
            target = 'BuddyAllocator::get_order_free_mut' if case.startswith('redb') else 'NodeMap::<T>::insert'
            executable = previous.R2_EXEC + [target] if case.startswith('redb') else [target]
            proofs = previous.R2_PROOF if case.startswith('redb') else []
            pairs = {'finish_mutation': 20} if goals and case.startswith('redb') else {}
            previous.require_native_scope(data, source, executable, proofs, [] if positive else [target], pairs)
            if not positive:
                require_failure_kind(stderr, 'postcondition not satisfied')
                require(not any(x in stderr for x in ('error[E', 'invariant not satisfied', 'possible arithmetic underflow/overflow', 'recommendation not met')), 'negative contains another failure')
            if goals:
                require_publication(data, positive, artifact)
            receipt['outcomes'].append({'label': label, 'verification': data['verification-results'], 'inferred_helpers': inf['functions'], 'failed_functions': inf['failed_functions'], 'scope': inf['verification_scope'], 'times': data['times-ms']})
            print(label, data['verification-results'], flush=True)
        guard.check()
        receipt['final_guards_passed'] = True


if __name__ == '__main__':
    main()
