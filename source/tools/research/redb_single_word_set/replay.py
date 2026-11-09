#!/usr/bin/env python3
"""Full-crate original-body research replay; raw evidence is never committed."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
PIN = 'c82ab865bb362bc4af067d332adc44978ff64813'
def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def need(ok, why):
    if not ok: raise ValueError(why)
def body(text):
    # Contracts may contain braced conditionals: match the final exact executable
    # body against its frozen text instead of parsing the first opening brace.
    return text[text.index('{'):].strip()
def erase_marked(text):
    return re.sub(r'\s*// BEGIN ERASED\n.*?// END ERASED\n', '\n', text, flags=re.S)
def fidelity(fixture):
    source = fixture.read_text()
    text = erase_marked(source)
    for name in ('data_index_of', 'select_mask', 'get', 'set'):
        original = (HERE/'original'/f'{name}.rs').read_text()
        need(body(original) in text, f'original executable body lost: {fixture.name}/{name}')
    need((HERE/'original/struct.rs').read_text().strip() in text, 'struct changed')
    for forbidden in ('assume(', 'admit(', 'external_body', 'external_fn_specification', 'assume_specification', 'verify=false'):
        need(forbidden not in source, f'forbidden trust: {forbidden}')

def same_contracts(baseline, assisted):
    canonical = lambda text: re.sub(r'\s+', '', text)
    actual = erase_marked(assisted.read_text()).replace('mod word_laws;\n', '')
    need(canonical(actual) == canonical(baseline.read_text()), 'assistance changed executable code or semantic contracts')

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--redb', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--verus', type=Path, default=ROOT/'source/target-verus/release/verus')
    p.add_argument('--solver', type=Path, default=ROOT/'source/z3')
    p.add_argument('--baseline-only', action='store_true')
    a = p.parse_args()
    out = Path(tempfile.mkdtemp(prefix='single-word-set-', dir=a.output.resolve()))
    os.chmod(out, 0o700)
    if (HERE/'word_laws.rs').exists():
        (out/'word_laws.rs').write_bytes((HERE/'word_laws.rs').read_bytes())
    receipt = {'application_proof_proved': False, 'runs': [], 'output':str(out), 'started':time.time()}
    env = dict(os.environ, RUSTUP_TOOLCHAIN='1.99.1', PATH='/home/dev/.cargo/bin:'+os.environ['PATH'], VERUS_Z3_PATH=str(a.solver.resolve()))
    def run(label, command, expected=None):
        command = list(map(str, command)); started = time.time()
        try:
            r = subprocess.run(command, cwd=ROOT/'source', env=env, text=True, capture_output=True, timeout=240)
        except subprocess.TimeoutExpired as error:
            for suffix, value in (('stdout', error.stdout), ('stderr', error.stderr)):
                (out/f'{label}.{suffix}').write_bytes(value if isinstance(value, bytes) else (value or '').encode())
            receipt['runs'].append({'label':label, 'command':command, 'timed_out':True, 'seconds':time.time()-started})
            raise
        (out/f'{label}.stdout').write_text(r.stdout); (out/f'{label}.stderr').write_text(r.stderr)
        record = {'label':label, 'command':command, 'returncode':r.returncode, 'seconds':time.time()-started,
                  'stdout_sha256':sha(out/f'{label}.stdout'), 'stderr_sha256':sha(out/f'{label}.stderr')}
        receipt['runs'].append(record)
        if expected is not None: need(r.returncode == expected, f'{label}: unexpected exit {r.returncode}')
        return r, record
    def verify(label, path, expected):
        frozen = out/f'{label}.rs'
        frozen.write_bytes(path.read_bytes())
        r, rec = run(label, [a.verus.resolve(), frozen, '--infer-goals', '--no-cheating', '--output-json', '--multiple-errors', '20'])
        rec['fixture_sha256'] = sha(frozen)
        d = json.loads(r.stdout); v = d['verification-results']
        rec['verification_results'] = v
        need(not v.get('encountered-vir-error') and 'verified' in v, f'{label}: frontend error')
        need(v.get('is-verifying-entire-crate') is True, f'{label}: partial verification')
        need(not any(x in r.stderr.lower() for x in ('resource limit','timed out','panicked at')), f'{label}: resource/internal failure')
        need((r.returncode == 0) == expected and v['success'] == expected, f'{label}: unexpected proof result')
        if not expected:
            need('postcondition not satisfied' in r.stderr, f'{label}: not an authored postcondition failure')
            need('precondition not satisfied' not in r.stderr and 'assertion failed' not in r.stderr, f'{label}: mixed failure kinds')
        else:
            scope = d['verus-infer']['verification_scope']
            expected_bodies = {'assisted': (4, 4), 'unrelated_positive': (2, 4)}[label]
            need(scope.get('complete_selected_scope_verified') is True, f'{label}: incomplete scope')
            need((scope.get('checked_exec_bodies'), scope.get('checked_proof_bodies')) == expected_bodies, f'{label}: missing bodies')
        rec['verification_results'] = v
        return r
    try:
        need(subprocess.check_output(['git','-C',str(a.redb),'rev-parse','HEAD'],text=True).strip()==PIN, 'pin mismatch')
        need(not subprocess.check_output(['git','-C',str(a.redb),'status','--porcelain'],text=True), 'dirty upstream')
        freeze=json.loads((HERE/'freeze.json').read_text())
        need(sha(a.redb/freeze['source_path'])==freeze['source_sha256'], 'upstream source changed')
        for name, item in freeze['items'].items():
            original = HERE/'original'/f'{name}.rs'
            need(sha(original)==item['sha256'], 'frozen item changed')
            upstream = (a.redb/freeze['source_path']).read_bytes()
            need(original.read_bytes() in upstream, 'item is not an exact pinned source extraction')
            need(upstream[:upstream.index(original.read_bytes())].count(b'\n') + 1 == item['line'], 'source location changed')
        receipt['freeze'] = freeze
        receipt['git_head'] = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
        receipt['source_hashes'] = {str(f.relative_to(ROOT)):sha(f) for f in HERE.rglob('*') if f.is_file() and '__pycache__' not in str(f) and f.name not in ('REPORT.md', 'RESULTS.json')}
        bundle=a.verus.resolve().parent
        receipt['tool_hashes'] = {str(f):sha(f) for f in [a.solver.resolve(), *[bundle/n for n in ('verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so')]]}
        receipt['solver_version']=subprocess.check_output([a.solver.resolve(),'-version'],text=True)
        run('rustc_version', ['rustc','--version','--verbose'], 0)
        fidelity(HERE/'baseline.rs')
        baseline=verify('baseline', HERE/'baseline.rs', False)
        receipt['baseline_diagnostics'] = baseline.stderr
        if not a.baseline_only:
            fidelity(HERE/'assisted.rs')
            same_contracts(HERE/'baseline.rs', HERE/'assisted.rs')
            verify('assisted', HERE/'assisted.rs', True)
            receipt['application_proof_proved'] = True
            assisted=(HERE/'assisted.rs').read_text()
            need(assisted.count('group == u64::MAX')==1, 'mutation anchor ambiguous')
            mutant=out/'return_mutant.rs'; mutant.write_text(assisted.replace('group == u64::MAX','group != u64::MAX'))
            verify('semantic_return_mutant', mutant, False)
            verify('unrelated_positive', HERE/'unrelated_positive.rs', True)
            verify('unrelated_negative', HERE/'unrelated_negative.rs', False)
            native=(HERE/'original/struct.rs').read_text()+'\nimpl U64GroupedBitmap {\n'+''.join((HERE/'original'/f'{n}.rs').read_text() for n in ('data_index_of','select_mask','get','set'))+'}\n'+(HERE/'native_harness.rs').read_text()
            raw=out/'original_native.rs'; raw.write_text(native)
            erased=out/'erased_native.rs'; erased.write_text(assisted.replace('fn main() {}',(HERE/'native_harness.rs').read_text()))
            results=[]
            for profile in ('debug','release'):
                for kind,path in (('original',raw),('erased',erased)):
                    exe=out/f'{kind}_{profile}'
                    opts=['-C',f'debug-assertions={"yes" if profile=="debug" else "no"}']
                    if profile=='release': opts+=['-O']
                    if kind=='original': run(f'{kind}_{profile}_compile',['rustc',path,'--edition=2021',*opts,'-o',exe],0)
                    else: run(f'{kind}_{profile}_compile',[a.verus.resolve(),path,'--infer-goals','--no-cheating','--compile',*opts,'-o',exe],0)
                    r,_=run(f'{kind}_{profile}_run',[exe],0); results.append(r.stdout)
            need(len(set(results))==1,'native/erased behavior differs')
            receipt['native_behavior']=results[0]
        receipt['source_hashes_after'] = {name:sha(ROOT/name) for name in receipt['source_hashes']}
        receipt['tool_hashes_after'] = {name:sha(name) for name in receipt['tool_hashes']}
        need(receipt['source_hashes'] == receipt['source_hashes_after'], 'research inputs changed during replay')
        need(receipt['tool_hashes'] == receipt['tool_hashes_after'], 'tool bundle changed during replay')
        need(not subprocess.check_output(['git','-C',str(a.redb),'status','--porcelain'],text=True), 'upstream dirtied during replay')
        need(sha(a.redb/freeze['source_path'])==freeze['source_sha256'], 'upstream source changed during replay')
        receipt['completed_expected_checks'] = True
    except BaseException as error:
        receipt['failure']={'type':type(error).__name__,'message':str(error)}
        raise
    finally:
        receipt['finished']=time.time()
        (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        print(out)
if __name__=='__main__': main()
