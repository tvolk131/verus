#!/usr/bin/env python3
"""Focused F2 replay using existing execution/stability/validation utilities."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
TOOLS = HERE.parents[2]
sys.path.insert(0, str(TOOLS))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import (LoggedCommands, replay_receipt, require, validate_run,
                              require_exact, require_function_coverage, require_publication)
RUST = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
EXEC = ['total', 'max_evil', 'one_honest', 'degree', 'threshold', 'from', 'policy_client']
PROOFS = ['quorum_intersection', 'quorum_has_honest', 'one_honest_margin']


def method(text, name):
    found = re.search(r'    (?:pub )?fn ' + name + r'\(', text)
    require(found is not None, 'missing original method: ' + name)
    end = text.index('{', found.start()) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[found.start():end]


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path)
    p.add_argument('--scenario', choices=('baseline', 'corrected_baseline', 'policy', 'manual', 'intersection_negative', 'fault_negative', 'zero'), default='baseline')
    p.add_argument('--omit', choices=('none', 'union_lens', 'union_subset', 'honest_subset', 'margin_subset'), default='none')
    p.add_argument('--clients', action='store_true')
    p.add_argument('--warm', action='store_true')
    p.add_argument('--mode', choices=('ordinary', 'infer', 'both'), default='both')
    args = p.parse_args()
    out = args.output or Path(tempfile.mkdtemp(prefix='fedimint-f2-'))
    out.mkdir(parents=True, exist_ok=True)
    require(not (out / 'receipt.json').exists(), 'fresh output required')
    receipt = dict(case='F2', scenario=args.scenario, omit=args.omit, commands=[], outcomes=[])
    with replay_receipt(out, receipt):
        selection = json.loads((HERE.parent / 'SELECTION_R2_F2.json').read_text())
        runtime = selection['runtime']
        origin = Path(runtime['path'])
        require(digest(origin / 'receipt.json') == runtime['receipt_sha256'], 'runtime receipt pin')
        bundle = out / 'bundle'
        bundle.mkdir()
        for n, expected in runtime['tools'].items():
            require(digest(origin / n) == expected, 'runtime pin: ' + n)
            shutil.copyfile(origin / n, bundle / n)
            (bundle / n).chmod(0o700 if n in ('rust_verify', 'verus', 'z3') or n.endswith('.so') else 0o600)
            require(digest(bundle / n) == expected, 'private runtime pin: ' + n)
        (bundle / 'verus-root').touch()
        source = selection['sources']['fedimint']
        root = Path(source['root'])
        require(subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip() == source['commit'], 'source commit')
        for n, expected in source['files'].items():
            src = root / n
            require(src.stat().st_size == expected['bytes'] and digest(src) == expected['sha256'], 'source pin: ' + n)
            dst = out / 'source' / n
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dst)
        fidelity = json.loads((HERE / 'fidelity.json').read_text())
        require(digest(HERE / 'component.rs') == fidelity['fixture_sha256'], 'frozen fixture drift')
        fixture = (HERE / 'component.rs').read_text()
        original = (out / 'source' / fidelity['source']).read_text()
        for n, expected in fidelity['methods_sha256'].items():
            text = method(fixture, n)
            src = original.split('impl From<usize> for NumPeers {', 1)[1] if n == 'from' else original.split('impl NumPeers {', 1)[1]
            require(text == method(src, n) and hashlib.sha256(text.encode()).hexdigest() == expected, 'original body drift: ' + n)
        if args.scenario == 'policy':
            start = fixture.index('// Semantic BFT policy')
            end = fixture.index('// Numeric observation client')
            fixture = fixture[:start] + fixture[end:]
        if args.scenario in ('manual', 'intersection_negative', 'fault_negative'):
            manual = json.loads((HERE / 'manual.json').read_text())
            for key, text in manual.items():
                marker = '// MANUAL_' + key + ': unsolved baseline slot, never claimed as a proof.'
                require(fixture.count(marker) == 1, 'manual slot mismatch')
                fixture = fixture.replace(marker, text)
        omissions = dict(union_lens='vstd::set_lib::lemma_set_intersect_union_lens(a, b);', union_subset='vstd::set_lib::lemma_len_subset(a.union(b), u);', honest_subset='vstd::set_lib::lemma_len_subset(a, bad);', margin_subset='vstd::set_lib::lemma_len_subset(chosen, bad);')
        if args.omit != 'none':
            require(args.scenario == 'manual', 'ablations only change manual glue')
            require(fixture.count(omissions[args.omit]) == 1, 'ablation missing')
            fixture = fixture.replace(omissions[args.omit], '')
        if args.scenario == 'intersection_negative':
            fixture = fixture.replace('ensures a.intersect(b).len() > fault(n),', 'ensures a.intersect(b).len() > fault(n) + 1,')
        if args.scenario == 'fault_negative':
            fixture = fixture.replace('a.len() >= quorum(n), bad.len() <= fault(n),', 'a.len() >= quorum(n), bad.len() <= quorum(n),')
        if args.scenario == 'zero':
            # Remove only max_evil's positive-domain contract; preserve its exact body.
            fixture = fixture[:fixture.index('verus! {\n// Semantic BFT policy')]
            old = '#[verus_spec(r => requires self.count() > 0,\n        ensures r == fault(self.count()), 3 * r < self.count(), self.count() <= 3 * r + 3)]'
            require(fixture.count(old) == 1, 'zero adapter mismatch')
            fixture = fixture.replace(old, '#[verus_spec(r => ensures r == fault(self.count()))]')
        if args.scenario != 'baseline':
            fixture += '\n' + (HERE / 'adapter.rs').read_text()
        (out / 'component.rs').write_text(fixture)
        receipt.update(fixture_sha256=digest(out / 'component.rs'), runner_sha256=digest(Path(__file__)),
                       frozen_sha256=fidelity['fixture_sha256'], runtime=runtime, source=source)
        env = dict(os.environ, PATH=str(RUST / 'bin') + ':/home/dev/.cargo/bin:' + os.environ.get('PATH', ''),
                   RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=str(RUST),
                   VERUS_Z3_PATH=str(bundle / 'z3'), VERUS_INFER_CACHE=str(out / 'cache'),
                   LD_LIBRARY_PATH=str(RUST / 'lib') + ':' + os.environ.get('LD_LIBRARY_PATH', ''))
        guarded = [bundle / n for n in runtime['tools']] + [HERE / 'component.rs', HERE / 'fidelity.json', Path(__file__), HERE.parent / 'SELECTION_R2_F2.json', TOOLS / 'infer_experiment.py', TOOLS / 'infer_eval_support.py']
        if (HERE / 'manual.json').exists():
            guarded.append(HERE / 'manual.json')
        guarded += [HERE / 'adapter.rs']
        guarded += [HERE / 'client.rs', out / 'component.rs']
        guard = ArtifactGuard(guarded)
        run = LoggedCommands(out, out, receipt['commands'], timeout=120, guard=guard)
        modes = ('ordinary', 'infer') if args.mode == 'both' else (args.mode,)
        trials = [(m, m) for m in modes]
        if args.warm:
            require(args.scenario == 'manual', 'warm only after a positive manual proof')
            trials += [(m, m + '_warm') for m in modes]
        for mode, label in trials:
            artifact = out / (label + '.vir')
            cmd = [str(bundle / 'verus'), str(out / 'component.rs'), '--crate-type=lib', '--edition=2024',
                   '--crate-name', 'f2_' + mode, '--no-cheating', '--output-json', '--time', '--compile',
                   '--export', str(artifact), '-o', str(out / ('lib' + mode + '.rlib'))]
            if mode == 'infer':
                cmd += ['--infer-goals']
            code = run(label, cmd, env)
            stdout, stderr = [(out / (label + '.' + k)).read_text() for k in ('stdout', 'stderr')]
            data = json.loads(stdout)
            v = data['verification-results']
            expected = args.scenario in ('policy', 'manual') and args.omit == 'none'
            validate_run(dict(returncode=code, stdout=stdout, stderr=stderr), expected)
            require(v.get('is-verifying-entire-crate') is True and not v.get('encountered-vir-error'), 'full ordinary frontend/body checks required')
            if not expected:
                diagnostic = 'possible arithmetic underflow/overflow' if args.scenario == 'zero' else 'postcondition not satisfied'
                require(diagnostic in stderr, 'missing intended negative diagnostic')
                if args.scenario != 'zero':
                    require(not any(x in stderr for x in ('possible arithmetic underflow/overflow', 'precondition not satisfied', 'assertion failed', 'error[E')), 'unhealthy semantic negative')
            infer = data['verus-infer']
            require_function_coverage(infer.get('functions', []), [])
            require_exact(infer.get('functions', []))
            if mode == 'infer':
                require_publication(data, expected, artifact)
            else:
                require(infer['interface_written'] is False, 'ordinary run claims Infer publication')
            require(data['verus']['commit'] == 'f4415999876559504756ff9b4b9bfd245a09dd21', 'wrong engine')
            failures = set(infer['failed_functions'])
            targets = {'quorum_intersection', 'quorum_has_honest', 'one_honest_margin'}
            if args.scenario == 'baseline':
                targets.add('from')
            if args.scenario == 'intersection_negative' or args.omit in ('union_lens', 'union_subset'):
                targets = {'quorum_intersection'}
            if args.scenario == 'fault_negative' or args.omit == 'honest_subset':
                targets = {'quorum_has_honest'}
            if args.omit == 'margin_subset':
                targets = {'one_honest_margin'}
            if args.scenario == 'zero':
                targets = {'max_evil'}
            require({n.rsplit('::', 1)[-1] for n in failures} == (set() if expected else targets), 'wrong failing function identities')
            breakdown = [f for m in data['times-ms']['smt']['smt-run-module-times'] for f in m.get('function-breakdown', [])]
            require(all(f['success'] is True for f in breakdown if f['function'].rsplit('::', 1)[-1] not in targets), 'unhealthy dependency')
            scope = infer['verification_scope']
            require(scope['native_collection_completed'] is True, 'missing native collection')
            execs = EXEC + ['clone']
            proofs = [] if args.scenario in ('policy', 'zero') else PROOFS
            if args.scenario == 'zero':
                execs.remove('policy_client')
            for kind, names in (('exec', execs), ('proof', proofs)):
                actual = [b for b in scope['native_functions'] if b['body_status'] == ('checked_' + kind + '_body' if expected else 'selected_body_not_certified_by_successful_full_run')]
                if expected:
                    require_function_coverage(actual, names)
                    require(scope['checked_' + kind + '_bodies'] == len(names), 'checked-body count mismatch')
            if not expected:
                require_function_coverage([b for b in scope['native_functions'] if b['body_status'] == 'selected_body_not_certified_by_successful_full_run'], execs + proofs)
                require(scope['checked_exec_bodies'] == scope['checked_proof_bodies'] == 0, 'failed crate certified a body')
            require(scope['complete_selected_scope_verified'] is expected, 'incomplete full scope')
            receipt['outcomes'].append(dict(mode=mode, label=label, exit_code=code, verification=v, scope=scope,
                  total_verify_ms=data['times-ms']['total-verify'], helpers=infer.get('functions', []),
                  stderr_sha256=digest(out / (label + '.stderr')), stdout_sha256=digest(out / (label + '.stdout'))))
        if args.clients:
            require(args.scenario == 'manual' and 'ordinary' in modes and args.omit == 'none', 'clients need positive ordinary provider')
            shutil.copyfile(HERE / 'client.rs', out / 'client.rs')
            cmd = [str(bundle / 'verus'), str(out / 'client.rs'), '--crate-type=lib', '--edition=2024', '--crate-name=f2_client', '--no-cheating', '--output-json', '--time', '--extern', 'f2_ordinary=' + str(out / 'libordinary.rlib'), '--import', 'f2_ordinary=' + str(out / 'ordinary.vir'), '-L', str(out)]
            client_guard = ArtifactGuard([out / 'client.rs', out / 'ordinary.vir', out / 'libordinary.rlib'])
            code = run('client', cmd, env)
            client_guard.check()
            stdout, stderr = [(out / ('client.' + k)).read_text() for k in ('stdout', 'stderr')]
            data = validate_run(dict(returncode=code, stdout=stdout, stderr=stderr), True)
            scope = data['verus-infer']['verification_scope']
            require(scope['complete_selected_scope_verified'] and scope['native_collection_completed'], 'client body inventory incomplete')
            require_function_coverage([b for b in scope['native_functions'] if b['body_status'] == 'checked_exec_body'], ['admission_observation', 'boundary_observations'])
            require(scope['checked_exec_bodies'] == 2 and scope['checked_proof_bodies'] == 0, 'client count mismatch')
            require_function_coverage(data['verus-infer']['functions'], [])
            receipt['outcomes'].append(dict(mode='client', verification=data['verification-results'], scope=scope, total_verify_ms=data['times-ms']['total-verify'], stdout_sha256=digest(out / 'client.stdout'), stderr_sha256=digest(out / 'client.stderr')))
        guard.check()
    print(out)

if __name__ == '__main__':
    main()
