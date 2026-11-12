"""Private, guarded D2 review replay and fresh VIR-only build. No production edits."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require

S = Path('/tmp/owned-cell-D2-cc9b088c5-parent-snapshot')
G = Path('/tmp/owned-cell-root-gate-fixes-d18c0f62-parent-snapshot')
TC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
O = Path(sys.argv[2]).resolve()
action = sys.argv[1]
binding = json.loads(Path(__file__).with_name('BINDING.json').read_text())

def authenticate():
    require(digest(S/'receipt.json') == binding['snapshot_receipt'], 'D2 receipt')
    s = json.loads((S/'receipt.json').read_text())
    for n, h in s['tools_sha256'].items():
        require(digest(S/n) == h, 'D2 tool '+n)
    require(digest(S/'checks/native-query-tests') == binding['native_test'], 'native executable')
    for p, h in binding['source'].items():
        require(digest(ROOT/p) == h, 'source '+p)
    return s

if action == 'replay':
    O.mkdir(exist_ok=False)
    receipt = dict(commands=[], builds=0, provenance=binding['build_provenance'])
    with replay_receipt(O, receipt):
        s = authenticate()
        require(digest(G/'receipt.json') == 'fb56f75c515ab64a6c87b8ac465a1bb282b819f0d5aa10a9069eb102cf8c8f81', 'CLI receipt')
        gs = json.loads((G/'receipt.json').read_text())
        original = [S/'receipt.json', *[S/n for n in s['tools_sha256']], S/'checks/native-query-tests', G/'receipt.json']
        for n in ['refusal-tests', 'admission-tests']:
            require(digest(G/'checks'/n) == gs['checks'][n]['sha256'], 'original CLI '+n)
            original.append(G/'checks'/n)
        guard = ArtifactGuard([*original, *[ROOT/p for p in binding['source']]])
        for d in ['snapshot/checks', 'tmp', 'cache', 'home']:
            (O/d).mkdir(parents=True, exist_ok=True)
        private = O/'snapshot'
        for n in s['tools_sha256']:
            shutil.copy2(S/n, private/n)
            require(digest(private/n) == s['tools_sha256'][n], 'private tool '+n)
        for n, src in [('native-query-tests', S), ('refusal-tests', G), ('admission-tests', G)]:
            shutil.copy2(src/'checks'/n, private/'checks'/n)
            require(digest(private/'checks'/n) == digest(src/'checks'/n), 'copy '+n)
        (private/'verus-root').touch()
        inputs = [*original, *[ROOT/p for p in binding['source']], *[p for p in private.rglob('*') if p.is_file()], Path(__file__), ROOT/'source/tools/infer_eval_support.py', ROOT/'source/tools/infer_experiment.py']
        guard.check()
        guard = ArtifactGuard(inputs)
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:/usr/bin:/bin', RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1', VERUS_IN_VARGO='1', VERUS_ROOT=str(private), VERUS_Z3_PATH=str(private/'z3'), RUST_MIN_STACK='20971520', LD_LIBRARY_PATH=str(TC/'lib')+':'+str(TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib'), TMPDIR=str(O/'tmp'), XDG_CACHE_HOME=str(O/'cache'), VERUS_INFER_CACHE=str(O/'infer-cache'), VERUS_TEST_LOG_DIR=str(O/'test-logs'))
        receipt['environment'] = {k:env[k] for k in ['PATH','RUSTUP_HOME','RUSTUP_TOOLCHAIN','VERUS_IN_VARGO','VERUS_ROOT','VERUS_Z3_PATH','RUST_MIN_STACK','LD_LIBRARY_PATH','TMPDIR','XDG_CACHE_HOME','VERUS_INFER_CACHE']}
        run = LoggedCommands(O, ROOT/'source', receipt['commands'], timeout=240, guard=guard)
        for label, args, expected in [
            ('native', ['--exact','checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions','--nocapture','--test-threads=1'], '1 passed; 0 failed'),
            ('refusal', ['--nocapture','--test-threads=1'], '5 passed; 0 failed'),
            ('admission', ['--nocapture','--test-threads=1'], '7 passed; 0 failed')]:
            name = {'native':'native-query-tests','refusal':'refusal-tests','admission':'admission-tests'}[label]
            require(run(label,[str(private/'checks'/name),*args],env)==0, label+' exit')
            stdout=(O/(label+'.stdout')).read_text()
            require(expected in stdout, label+' inventory')
        stderr=(O/'native.stderr').read_text()
        for negative in ['false','true']:
            require(stderr.count('D2 complete native body query inventory: 3 bodies, 3 queries, negative='+negative)==1, 'query inventory')
            require(stderr.count('D2 ordinary full native body result (negative='+negative+')')==3, 'function inventory')
        require('panicked at' not in stderr, 'native panic')
        guard.check()
        receipt.update(guard_passed=True, positive_body_queries=3, false_first_contract_queries=3, false_future_solver_control=False, hashes={str(p):digest(p) for p in inputs}, logs={p.name:digest(p) for p in O.glob('*.std*')})
elif action in ['vir', 'vir-lib']:
    authenticate()
    spec=importlib.util.spec_from_file_location('d2_review_setup',ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation/check.py')
    check=importlib.util.module_from_spec(spec); spec.loader.exec_module(check)
    if action == 'vir':
        check.setup(O)
    target = O if action == 'vir' else O/'unit-only'
    if action == 'vir-lib':
        target.mkdir(exist_ok=False)
    receipt=dict(commands=[], provenance='Fresh complete VIR tests from frozen local source; fresh authenticated offline dependencies, no copied build tree, no compiler/macros/vstd rebuild.')
    with replay_receipt(target,receipt):
        inputs=[p for base in ['source/vir','source/air','source/vir_macros'] for p in (ROOT/base).rglob('*') if p.is_file()]+[ROOT/'source/Cargo.lock',ROOT/'source/Cargo.toml',Path(__file__)]
        guard=ArtifactGuard(inputs)
        env=check.env(O)
        env.update(CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',CARGO_BUILD_JOBS='2')
        require(shutil.disk_usage(O).free >= 25*1024**3, '25GiB floor')
        run=LoggedCommands(target,ROOT/'source',receipt['commands'],timeout=1200,guard=guard)
        args = ['--lib'] if action == 'vir-lib' else []
        require(run('complete-vir',[str(TC/'bin/cargo'),'test','--offline','--locked','-p','vir',*args,'--','--test-threads=1'],env)==0,'fresh VIR')
        guard.check()
        receipt.update(guard_passed=True,source={str(p.relative_to(ROOT)):digest(p) for p in inputs},logs={p.name:digest(p) for p in target.glob('*.std*')},free_bytes=shutil.disk_usage(O).free)
else:
    raise ValueError(action)
print(json.dumps(dict(output=str(O),success=receipt['success'],receipt_sha256=digest((target if action.startswith('vir') else O)/'receipt.json'))))
