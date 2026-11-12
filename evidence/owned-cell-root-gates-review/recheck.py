#!/usr/bin/env python3
"""Scoped parent-artifact replay, no builds or harness/tool transformations."""
import datetime, json, os, re, shutil, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
PIN = '9761b5f55405fdcb2901f6599b52973cb1e31c43'
PARENT = Path('/tmp/owned-cell-root-gate-fixes-d18c0f62-parent-snapshot')
RECEIPT = 'fb56f75c515ab64a6c87b8ac465a1bb282b819f0d5aa10a9069eb102cf8c8f81'
OUT = Path(os.environ.get('ROOT_GATES_REVIEW_OUTPUT','/tmp/owned-cell-root-gates-review-9761b5f5'))
HERE = Path(__file__).resolve().parent

def authenticate():
    require(subprocess.run(['git','merge-base','--is-ancestor',PIN,'HEAD'], cwd=ROOT).returncode==0,'prepared root ancestor')
    changed=subprocess.check_output(['git','diff','--name-only',PIN,'HEAD'],cwd=ROOT,text=True).splitlines()
    require(all(n.startswith('evidence/owned-cell-root-gates-review/') for n in changed),'only review-owned descendant changes')
    require(digest(PARENT/'receipt.json') == RECEIPT, 'receipt pin')
    manifest = json.loads((PARENT/'receipt.json').read_text())
    paths = {n:h for n,h in manifest['tools_sha256'].items()}
    paths.update({'checks/'+n:v['sha256'] for n,v in manifest['checks'].items()})
    require(len(manifest['tools_sha256']) == 8 and len(manifest['checks']) == 4, 'exact tool inventory')
    require((PARENT/'verus-root').read_bytes() == b'', 'root marker')
    for n,h in paths.items():
        require(digest(PARENT/n) == h, 'parent authentication '+n)
    return manifest, paths

def env():
    e = dict(os.environ)
    tc = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
    e.update(RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1', VERUS_IN_VARGO='1',
             VERUS_ROOT=str(OUT/'snapshot'), VERUS_Z3_PATH=str(OUT/'snapshot/z3'),
             RUST_MIN_STACK='20971520', LD_LIBRARY_PATH=f'{tc}/lib:{tc}/lib/rustlib/x86_64-unknown-linux-gnu/lib',
             PATH=f'{tc}/bin:/home/dev/.cargo/bin:/usr/local/bin:/usr/bin:/bin', TMPDIR=str(OUT/'tmp'),
             VERUS_INFER_CACHE=str(OUT/'cache'), CARGO_TARGET_DIR=str(OUT/'unused-target'),
             PYTHONDONTWRITEBYTECODE='1')
    return e

def prepare():
    manifest, paths = authenticate()  # Must precede private copying.
    OUT.mkdir(exist_ok=False)
    for d in ['snapshot/checks','tmp','cache','logs','cases']:
        (OUT/d).mkdir(parents=True, exist_ok=True)
    for n in [*paths, 'verus-root', 'receipt.json']:
        shutil.copy2(PARENT/n, OUT/'snapshot'/n)
        require(digest(PARENT/n) == digest(OUT/'snapshot'/n), 'private copy '+n)
        (OUT/'snapshot'/n).chmod(0o555 if os.access(PARENT/n, os.X_OK) else 0o444)
    relatives = [
        'source/tools/research/redb_crash_protocol/GOAL.md',
        'source/tools/research/redb_crash_protocol/CURRENT.md',
        'source/tools/research/redb_crash_protocol/WORKER_BRIEFS.md',
        'source/tools/research/redb_crash_protocol/native_state_link/ACCEPTANCE_REFINEMENTS.md',
    ]
    relatives += [str(p.relative_to(ROOT)) for d in ['source/vir/src','source/rust_verify/src','source/rust_verify_test/tests']
                  for p in sorted((ROOT/d).rglob('*.rs')) if '/common/' in str(p) or d != 'source/rust_verify_test/tests' or p.name in ['local_checked_owned_cell.rs','local_checked_owned_cell_admission.rs']]
    relatives += [str(p.relative_to(ROOT)) for p in support_artifacts()]
    relatives += ['source/tools/research/redb_crash_protocol/native_state_link/implementation/'+n for n in ['ROOT_GATE_FIXES.md','ROOT_GATE_FIXES_REPORT.md','ROOT_GATE_FIXES_RESULTS.json','root_gate_fixes.py']]
    hashes = {}
    import hashlib
    for n in relatives:
        h = digest(ROOT/n)
        blob = subprocess.check_output(['git','show',PIN+':'+n], cwd=ROOT)
        require(hashlib.sha256(blob).hexdigest() == h, 'frozen source '+n)
        hashes[n] = h
    binding = dict(prepared_root=PIN, source_build=manifest['source_commit'], parent_receipt_sha256=RECEIPT,
                   parent_snapshot=str(PARENT), private_output=str(OUT), authenticated_files=paths,
                   source_sha256=hashes, started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                   replay_not_independent_rebuild=True, cache_copies=False, source_transformations=0,
                   tool_transformations=0, vstd_inherited='2257/0; not reverified', exact_environment=env())
    (HERE/'BINDING.json').write_text(json.dumps(binding, indent=2)+'\n')
    (OUT/'binding.json').write_text(json.dumps(binding, indent=2)+'\n')
    print(json.dumps(dict(authenticated_tools=8, authenticated_original_tests=4, sources=len(hashes), private_output=str(OUT))))

if __name__ == '__main__':
    if sys.argv[1:] == ['prepare']: prepare()

def replay():
    b = json.loads((OUT/'binding.json').read_text())
    _, files = authenticate_snapshot_only()
    guard = ArtifactGuard([ROOT/n for n in b['source_sha256']] +
                          [OUT/'snapshot'/n for n in [*files,'receipt.json','verus-root']] +
                          [PARENT/n for n in [*files,'receipt.json','verus-root']])
    records = dict(kind='ORIGINAL_PARENT_BUILT_BINARY_REPLAY_NOT_REBUILD', commands=[], suites={},
                   source_build=b['source_build'], binding_sha256=digest(HERE/'BINDING.json'))
    if (OUT/'receipt.json').exists():
        shutil.copy2(OUT/'receipt.json', OUT/('receipt-controller-attempt-'+str(len(list(OUT.glob('receipt-controller-attempt-*.json'))))+'.json'))
        records = json.loads((OUT/'receipt.json').read_text())
        records.pop('failure',None)
    runner = LoggedCommands(OUT/'logs', ROOT, records['commands'], timeout=600, guard=guard)
    selections = {
        'refusal-tests': (5, []),
        'admission-tests': (7, []),
        'vir-unit-tests': (172, []),
        'native-control-tests': (1, ['--exact','checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions']),
    }
    with replay_receipt(OUT, records):
        for name,(count,filters) in selections.items():
            e = env()
            for var,suffix in [('TMPDIR','tmp'),('VERUS_INFER_CACHE','cache')]:
                p = OUT/name/suffix; p.mkdir(parents=True,exist_ok=True); e[var] = str(p)
            executable = str(OUT/'snapshot/checks'/name)
            if not any(r['label']==name+'-list' and r['exit_code']==0 for r in records['commands']):
                require(runner(name+'-list', [executable,*filters,'--list'], e)==0, 'list '+name)
            listing = (OUT/'logs'/f'{name}-list.stdout').read_text()
            inventory = re.findall(r'^(.+): test$', listing, re.M)
            require(len(inventory)==count and len(set(inventory))==count, 'exact nonempty inventory '+name)
            if name in ['admission-tests','refusal-tests']:
                f = ROOT/'source/rust_verify_test/tests'/('local_checked_owned_cell'+('_admission' if name=='admission-tests' else '')+'.rs')
                expected = re.findall(r'#\[test\]\s*fn (\w+)', f.read_text())
                require(sorted(expected)==sorted(inventory), 'source/test inventory '+name)
            if name=='vir-unit-tests':
                metadata = [n for n in inventory if 'internal_representation' in n]
                require(len(metadata)==9, 'nine internal representation tests')
            command = [executable,*filters,'--nocapture','--test-threads=1']
            trace = OUT/'logs'/f'{name}.execve'
            if name in ['admission-tests','refusal-tests']:
                command = ['/usr/bin/strace','-f','-qq','-e','trace=execve','-s','65535','-o',str(trace),*command]
            if not any(r['label']==name and r['exit_code']==0 for r in records['commands']):
                require(runner(name,command,e)==0, 'replay '+name)
            stdout = (OUT/'logs'/f'{name}.stdout').read_text()
            require(f'test result: ok. {count} passed; 0 failed;' in stdout,'pass count '+name)
            row = dict(tests=count, inventory=inventory, original_executable_sha256=digest(executable))
            if trace.exists():
                text = trace.read_text()
                calls = [s for s in text.splitlines() if f'execve("{OUT}/snapshot/rust_verify"' in s and '= 0' in s]
                drivers = [s for s in calls if '--internal-trait-conflict-driver' in s]
                calls = [s for s in calls if '--internal-trait-conflict-driver' not in s]
                missing = [s for s in calls if '--no-cheating' not in s]
                require(calls and len(missing)==(2 if name=='refusal-tests' else 0), 'exact intentional no-cheating omissions')
                require(all('--experimental-owned-unsafe-cell' in s for s in missing), 'missing option is requested refusal control')
                row.update(internal_trait_conflict_driver_invocations=len(drivers), compiler_invocations=len(calls), infer_invocations=sum('--infer-goals' in s for s in calls),
                           trace_sha256=digest(trace), intentional_no_cheating_omissions=len(missing))
            records['suites'][name] = row
        require(sum(records['suites'][n]['compiler_invocations'] for n in ['refusal-tests','admission-tests'])==226, '226 actual compiler execs')
        guard.check()
        records.update(guards_passed=True, independent_builds=0, tool_transformations=0, harness_transformations=0,
                       native_body_proofs=0,state_proofs=0, field_authorities=0)
    print(json.dumps(dict(success=records['success'], suites={n:{k:v for k,v in r.items() if k not in ['inventory']} for n,r in records['suites'].items()})))

def authenticate_snapshot_only():
    require(digest(PARENT/'receipt.json')==RECEIPT,'parent receipt pin')
    manifest=json.loads((PARENT/'receipt.json').read_text())
    files={**manifest['tools_sha256'], **{'checks/'+n:v['sha256'] for n,v in manifest['checks'].items()}}
    for n,h in files.items():
        require(digest(PARENT/n)==h and digest(OUT/'snapshot'/n)==h,'original/private tools '+n)
    return manifest,files

if __name__=='__main__' and sys.argv[1:]==['replay']: replay()
