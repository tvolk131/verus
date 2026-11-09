#!/usr/bin/env python3
"""Fresh, full-crate replay. Never builds or writes application checkouts."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
HISTORY = HERE.parent / 'redb_single_word_set'

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def fidelity():
    for name, digest in json.loads((HERE/'freeze.json').read_text())['files'].items():
        assert sha(HERE/name) == digest, ('frozen fixture changed', name)
    import sys
    sys.path.insert(0, str(HISTORY))
    from replay import fidelity as historical_fidelity
    historical_fidelity(HERE / 'automatic.rs')
    original = (HISTORY / 'baseline.rs').read_text()
    adapted = original.replace('(word & (1u64 << (j as u64))) != 0', 'vstd::word::member(word, j)')
    adapted = adapted.replace('Seq::new(len as nat, |j: int| member(word, j))', 'vstd::word::view(word, len as nat)')
    assert adapted == (HERE / 'automatic.rs').read_text(), 'executable/semantic drift'
    for name in ('automatic.rs', 'permission.rs', 'consumer.rs'):
        s = (HERE / name).read_text()
        assert not any(x in s for x in ('proof {', 'let ghost', 'word_laws', 'lemma_', 'assume(', 'admit(', 'external_body'))

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--verus', type=Path, default=ROOT / 'source/target-verus/release/verus')
    a = p.parse_args()
    fidelity()
    out = Path(tempfile.mkdtemp(prefix='word-goals-', dir=a.output.resolve()))
    os.chmod(out, 0o700)
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
               RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(ROOT / 'source/z3'),
               VERUS_INFER_CACHE=str(out/'cache'))
    receipt = {'output': str(out), 'runs': [], 'sources': {}, 'tools': {}}
    for folder in (HERE, ROOT / 'source/vir/src/infer', ROOT / 'source/vstd'):
        for f in folder.glob('*.rs'):
            receipt['sources'][str(f.relative_to(ROOT))] = sha(f)
    for name in ('verus', 'rust_verify', 'vstd.vir', 'libvstd.rlib', 'libverus_builtin_macros.so'):
        f = a.verus.resolve().parent / name
        receipt['tools'][str(f)] = sha(f)
    def run(label, text, success, opts=(), expected_error=None, source_name=None):
        source = out / ((source_name or label) + '.rs')
        source.write_text(text)
        command = [str(a.verus.resolve()), str(source), '--no-cheating', '--output-json',
                   '--multiple-errors', '20', *map(str, opts)]
        r = subprocess.run(command, cwd=out, env=env, text=True, capture_output=True, timeout=240)
        (out / (label + '.stdout')).write_text(r.stdout)
        (out / (label + '.stderr')).write_text(r.stderr)
        d = json.loads(r.stdout)
        v = d['verification-results']
        info = d.get('verus-infer', {})
        rec = dict(label=label, command=command, returncode=r.returncode,
                   source_sha256=sha(source), stdout_sha256=sha(out/(label+'.stdout')),
                   stderr_sha256=sha(out/(label+'.stderr')), verification=v,
                   assistance=info.get('goal_assistance'), functions=info.get('functions'),
                   interface_written=info.get('interface_written'),
                   imported_inference_dependencies=info.get('imported_inference_dependencies'))
        receipt['runs'].append(rec)
        (out/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
        assert (r.returncode == 0) == success and v.get('success', False) == success, (label, r.stderr)
        assert not any(t in r.stderr.lower() for t in ('resource limit', 'timed out', 'panicked at')), label
        if expected_error:
            assert expected_error in r.stderr, (label, r.stderr)
        else:
            assert not v.get('encountered-vir-error') and v.get('is-verifying-entire-crate'), (label, v)
        print(label, v.get('verified'), v.get('errors'), flush=True)
        return d
    automatic = (HERE/'automatic.rs').read_text()
    permission = (HERE/'permission.rs').read_text()
    goals = ['--infer-goals']
    run('historical_baseline', (HISTORY/'baseline.rs').read_text(), False, goals, 'postcondition not satisfied')
    d = run('automatic', automatic, True, goals)
    assert any(x['strategy'] == 'word_view_broadcast' for x in d['verus-infer']['goal_assistance'])
    run('disabled', automatic, False, (), 'postcondition not satisfied')
    run('permission', permission, True, goals)
    run('xor', permission.replace('rights | (1u64', 'rights ^ (1u64'), False, goals, 'postcondition not satisfied')
    run('return_negative', automatic.replace('group == u64::MAX', 'group != u64::MAX'), False, goals, 'postcondition not satisfied')
    run('count_negative', permission.replace('{ 0nat } else { 1nat }', '{ 1nat } else { 0nat }'), False, goals, 'postcondition not satisfied')
    for helper in ('helper_tuple', 'helper_shift'):
        run(helper, (HERE/(helper+'.rs')).read_text(), False, goals, 'Verus Infer')
    provider = (HERE/'provider.rs').read_text()
    for label in ('provider_cold', 'provider_warm'):
        run(label, provider, True, [*goals, '--crate-type=lib', '--crate-name=provider', '--compile',
                                  '--export', out/'provider.vir', '-o', out/'libprovider.rlib'], source_name='provider')
    run('consumer', (HERE/'consumer.rs').read_text(), True,
        [*goals, '--extern', 'provider='+str(out/'libprovider.rlib'), '--import', 'provider='+str(out/'provider.vir')])
    # A failed original goal must prevent publishing an otherwise valid inferred interface.
    bad = provider + '\nverus! { fn false_goal() { assert(false); } }\n'
    failed_export = out/'failed.vir'
    run('publication_negative', bad, False, [*goals, '--crate-type=lib', '--crate-name=failed',
        '--export', failed_export], 'assertion failed')
    assert not failed_export.exists(), 'failed proof published an interface'
    print(out/'receipt.json')

if __name__ == '__main__':
    main()
