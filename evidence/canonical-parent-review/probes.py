#!/usr/bin/env python3
"""Independent matching-runtime probes; only writes to a new private output dir."""
import argparse
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
    require_exact, require_function_coverage, validate_goal_run, validate_run)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.bundle = a.bundle.resolve()
    a.output = a.output.resolve()
    a.output.mkdir(mode=0o700)
    fixture = ROOT / 'source/tools/research/varint_canonical_goal/candidate-008.rs'
    original = fixture.read_text()
    helper = '\nverus! { #[verifier::infer] pub fn identity(n:u64)->u64 {n} }\n'
    second = '''
#[verus_spec(encoded => ensures canonical(number, encoded@))]
pub fn second(number:u64, buf:&mut[u8;10])->&[u8] { encode!(number,buf) }
'''
    false_caller = '''
verus! { fn false_goal(n:u64,b:&mut[u8;10])->(r:&[u8])
    ensures r@.len()==0, { second(n,b) } }
'''
    cases = [
        ('two_decorated_functions', original + second, True, (), {'u64','second'}, set(), None),
        ('two_with_healthy_helper', original + second + helper, True, (), {'u64','second','identity'}, {'identity'}, None),
        ('two_false_caller', original + second + helper + false_caller, False, (),
            {'u64','second','identity','false_goal'}, {'identity'}, 'postcondition not satisfied'),
    ]
    for label, flags in [('no_lifetime',('--no-lifetime',)), ('no_traits',('--no-trait-conflicts',)),
                         ('no_verify',('--no-verify',)),
                         ('filtered',('--verify-root','--verify-function','u64'))]:
        cases.append((label, original, False, flags, {'u64'}, set(), 'policy'))
    # Freeze every variant and its expectations before any compiler trial.
    frozen = []
    for label, text, success, opts, execs, helpers, kind in cases:
        source = a.output / (label + '.rs')
        source.write_text(text)
        source.chmod(0o444)
        frozen.append(dict(label=label, source_sha256=digest(source), success=success,
            opts=list(opts), execs=sorted(execs), helpers=sorted(helpers), kind=kind))
    (a.output / 'freeze.json').write_text(json.dumps(frozen, indent=2) + '\n')
    paths = [fixture, Path(__file__), a.output/'freeze.json',
             *[a.output/(r['label']+'.rs') for r in frozen]]
    guard = ArtifactGuard(tool_artifacts(a.bundle/'verus', a.bundle/'z3', extra=paths))
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
        RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.bundle/'z3'),
        VERUS_INFER_CACHE=str(a.output/'cache'))
    receipt = dict(commands=[], runs=[], artifacts=guard.initial)
    with replay_receipt(a.output, receipt):
        invoke = LoggedCommands(a.output, a.output, receipt['commands'], timeout=180,
            guard=guard, status_key='returncode')
        for r in frozen:
            label = r['label']
            argv = [str(a.bundle/'verus'), str(a.output/(label+'.rs')), '--crate-type=lib',
                '--no-cheating', '--infer-goals', '--output-json', '--multiple-errors', '20', *r['opts']]
            code = invoke(label, argv, env)
            run = dict(returncode=code, stdout=(a.output/(label+'.stdout')).read_text(),
                       stderr=(a.output/(label+'.stderr')).read_text())
            if r['kind'] == 'policy':
                data = validate_run(run, False, allow_frontend=True)
                require('requires full-crate verification' in run['stderr'], 'policy gate missing')
                require(data['verus-infer']['interface_written'] is False, 'policy publication')
            else:
                data = validate_goal_run(run, r['success'], expected_functions=set(r['helpers']),
                    exec_bodies=set(r['execs']), proof_bodies=set(), failure_kind=r['kind'])
                decisions = data['verus-infer']['assistance_explanations']['decisions']
                applied = [d['function'].rsplit('::',1)[-1] for d in decisions
                    if d['profile']=='mutable_buffer_u64_radix128' and d['status']=='applied']
                require(len(applied)==2 and set(applied)=={'u64','second'}, 'per-function decoration coverage')
                if not r['success']:
                    require(data['verification-results']['errors']==1, 'mixed semantic errors')
                    require('invariant not satisfied' not in run['stderr'], 'invariant mixed with caller failure')
            require_function_coverage(data['verus-infer']['functions'], set(r['helpers']))
            require_exact(data['verus-infer']['functions'])
            receipt['runs'].append(dict(**r, verification=data['verification-results'],
                inference=data['verus-infer'], stdout_sha256=digest(a.output/(label+'.stdout')),
                stderr_sha256=digest(a.output/(label+'.stderr'))))
            print(label, data['verification-results'], flush=True)
        guard.check()
        receipt['artifacts_after'] = guard.snapshot()


if __name__ == '__main__':
    main()
