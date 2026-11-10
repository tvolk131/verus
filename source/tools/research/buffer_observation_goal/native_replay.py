#!/usr/bin/env python3
"""Finite actual macro-erasure/source correspondence checks, not formal proof."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix='native-observations-', dir=a.output.resolve()))
    transfer = HERE.parent / 'buffer_transfer_goal'
    selected = transfer / 'original-selected.rs'
    frozen = json.loads((transfer / 'freeze.json').read_text())
    require(digest(selected) == frozen['files']['original-selected.rs'], 'changed real selected loop')
    require(selected.read_text() in (transfer / 'upstream/src/hmac.rs').read_text(), 'upstream selection mismatch')
    for name in ['prefix-zip.rs', 'full-zip.rs']:
        (out / name).write_bytes((HERE / name).read_bytes())
    (out / 'original.rs').write_text('pub fn xor_key(ipad: &mut [u8;128], key: &[u8]) {\n' + selected.read_text() + '\n}\n')
    native = out / 'main.rs'
    native.write_text('''#[path="prefix-zip.rs"] mod prefix;
#[path="full-zip.rs"] mod full;
#[path="original.rs"] mod original;
fn main() {
    let mut cases=0usize;
    for seed in 0..256usize {
        for n in [0,1,2,63,64,65,127,128,129,255,1024] {
            let before: [u8;128]=std::array::from_fn(|i|(seed.wrapping_mul(37).wrapping_add(i*19)) as u8);
            let key: Vec<u8>=(0..n).map(|i|seed.wrapping_add(i*53) as u8).collect();
            let key_before=key.clone();
            let mut expected=before;original::xor_key(&mut expected,&key);
            for writer in [prefix::xor_key,full::xor_key] {
                let mut actual=before;writer(&mut actual,&key);
                assert_eq!(actual,expected);assert_eq!(key,key_before);
                for j in 0..128 {
                    assert_eq!(actual[j],if j<n {before[j]^key[j]} else {before[j]});
                }
            }
            cases+=1;
        }
    }
    println!("{cases} cases, both actual macro-erased fixtures match original source including every suffix byte");
}
''')
    r = dict(commands=[], goal_complete=False, formal_proof=False,
             source_freeze={p.name: digest(p) for p in out.glob('*.rs')})
    (out / 'input-freeze.json').write_text(json.dumps(r['source_freeze'], indent=2) + '\n')
    with replay_receipt(out, r):
        guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(),
            extra=[Path(__file__), selected, transfer / 'freeze.json', *out.glob('*.rs')]))
        r['inputs'] = guard.initial
        call = LoggedCommands(out, ROOT / 'source', r['commands'], timeout=180, guard=guard)
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'])
        for profile, flags in [('debug', []), ('release', ['-O'])]:
            binary = out / profile
            command = ['rustc', '+1.99.1', '--edition=2021', str(native), '--extern',
                'vstd=' + str(a.verus.resolve().parent / 'libvstd.rlib'), '-L',
                'dependency=' + str(a.verus.resolve().parent), '-o', str(binary), *flags]
            require(call('compile-' + profile, command, env) == 0, 'actual macro-erasure compile failed')
            require(call('run-' + profile, [str(binary)], env) == 0, 'native source comparison failed')
            r[profile] = (out / ('run-' + profile + '.stdout')).read_text()
        guard.check()
    print(out / 'receipt.json')

if __name__ == '__main__':
    main()
