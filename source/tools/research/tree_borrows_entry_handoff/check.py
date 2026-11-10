"""Private normal-check runner; immutable delivered baseline, exact trial receipts."""
import datetime, hashlib, json, os, pathlib, shutil, subprocess, sys

OWN = pathlib.Path(__file__).resolve().parent
BASE = OWN.parent / 'tree_borrows_checked_foundation'
READY = pathlib.Path('/tmp/tree-borrows-checked-foundation-20261110')
OUT = pathlib.Path('/tmp/tree-borrows-entry-handoff-20261110')
ROCQ = READY / 'opamroot/tb/bin/rocq'

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def identity(p):
    return 'symlink:' + str(p.readlink()) if p.is_symlink() else sha(p)

def write(p, x):
    p.write_text(json.dumps(x, indent=2) + '\n')

def check(root, m):
    for n, h in m.items():
        if identity(root / n) != h:
            raise RuntimeError('identity mismatch: ' + str(root / n))
    return len(m)

def auth():
    if sha(READY / 'baseline-ready.json') != 'f56490bb8441dbd73c37fe46fe970dc008bb8b2b67ee2b651a04c34d2b1ca523':
        raise RuntimeError('ready receipt mismatch')
    ready = json.loads((READY / 'baseline-ready.json').read_text())
    if not all(ready[k] is True for k in ['ready', 'normal_compilation', 'full_recursive_kernel_check']):
        raise RuntimeError('baseline not ready')
    receipts = {}
    for name, x in ready['receipts'].items():
        p = pathlib.Path(x['receipt'])
        if sha(p) != x['sha256']:
            raise RuntimeError('receipt mismatch: ' + name)
        m = json.loads(p.read_text())
        if m['exit_code'] != 0 or sha(pathlib.Path(m['log'])) != m['log_sha256']:
            raise RuntimeError('unsuccessful/mismatched baseline log: ' + name)
        receipts[name] = {'sha256': sha(p), 'exit_code': m['exit_code'], 'log_sha256': m['log_sha256']}
    # Bind each map to the delivered ready receipt before using it.
    for n, h in ready['hashes'].items():
        p = READY / n
        if not p.exists():
            p = BASE / n
        if sha(p) != h:
            raise RuntimeError('ready-bound map mismatch: ' + n)
    pre = json.loads((READY / 'model-pretrial.json').read_text())
    check(BASE, pre['support_files'])
    check(READY, pre['private_runner_files'])  # Bytes only; NEVER execute old runner.
    tools = json.loads((BASE / 'TOOLS.json').read_text())
    for n, x in tools['tools'].items():
        p = pathlib.Path(n)
        if 'resolved_sha256' in x and (identity(p) != x['snapshot_identity'] or sha(p.resolve()) != x['resolved_sha256']):
            raise RuntimeError('tool mismatch: ' + n)
    for n, x in tools['host_tools'].items():
        if sha(pathlib.Path(x['path'])) != x['sha256']:
            raise RuntimeError('host-tool mismatch: ' + n)
    counts = {'source': check(READY / 'simuliris', json.loads((BASE / 'upstream-source.json').read_text())['files']),
              'model_objects': check(READY / 'simuliris', json.loads((READY / 'model-objects.json').read_text())),
              'loaded_objects': check(pathlib.Path('/'), json.loads((READY / 'loaded-objects.json').read_text()))}
    expected_v = {n for n in json.loads((BASE / 'upstream-source.json').read_text())['files'] if n.endswith('.v')}
    actual_v = {str(p.relative_to(READY / 'simuliris')) for p in (READY / 'simuliris').rglob('*.v')}
    if actual_v != expected_v:
        raise RuntimeError('upstream proof-source set mismatch')
    counts['installed'] = {root: check(pathlib.Path(root), m) for root, m in json.loads((READY / 'installed-pretrial.json').read_text()).items()}
    buildlog = (READY / 'jobs/model-build.log').read_text()
    if 'MODEL_OBJECTS_SHA256 ' + sha(READY / 'model-objects.json') not in buildlog:
        raise RuntimeError('build did not bind object map')
    return {'baseline_ready_sha256': sha(READY / 'baseline-ready.json'), 'counts': counts, 'receipts': receipts,
            'tools_manifest_sha256': sha(BASE / 'TOOLS.json'), 'rocq_sha256': sha(ROCQ),
            'model_pretrial_sha256': sha(READY / 'model-pretrial.json'),
            'installed_pretrial_sha256': sha(READY / 'installed-pretrial.json'),
            'model_object_map_sha256': sha(READY / 'model-objects.json'),
            'loaded_object_map_sha256': sha(READY / 'loaded-objects.json')}

def env():
    e = os.environ.copy()
    removed = []
    for k in list(e):
        if k.startswith(('COQ', 'ROCQ', 'OCAML', 'CAML', 'OPAM')) or k in ['DUNE_CACHE_ROOT', 'DUNE_CACHE']:
            removed.append(k)
            del e[k]
    e.update(PATH=str(ROCQ.parent) + ':/usr/bin:/bin', TMPDIR=str(OUT / 'tmp'),
             XDG_CACHE_HOME=str(OUT / 'cache'), XDG_CONFIG_HOME=str(OUT / 'config'),
             LD_LIBRARY_PATH=str(READY / 'deps/lib'), OPAMJOBS='2', DUNE_CACHE='disabled')
    return e, removed

def run(name, argv, cwd):
    target = OUT / name
    target.mkdir()  # No receipt/log substitution or overwriting a trial.
    e, removed = env()
    freeze = {p.name: sha(p) for p in sorted(OWN.iterdir()) if p.is_file()}
    private_inputs = {str(p): sha(p) for p in sorted(cwd.iterdir()) if p.is_file()}
    for arg in argv:
        p = pathlib.Path(arg)
        if p.is_absolute() and p.is_file():
            private_inputs[str(p)] = sha(p)
    write(target / 'pretrial.json', {'own_sources': freeze, 'argv': argv, 'cwd': str(cwd),
                                  'private_inputs_and_executable': private_inputs,
                                  'environment': {k: e[k] for k in ['PATH','TMPDIR','XDG_CACHE_HOME','XDG_CONFIG_HOME','LD_LIBRARY_PATH','OPAMJOBS','DUNE_CACHE']},
                                  'removed_environment_names': removed, 'baseline_auth_sha256': sha(OUT / 'auth.json')})
    meta = {'argv': argv, 'cwd': str(cwd), 'start_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'pretrial_sha256': sha(target / 'pretrial.json')}
    with (target / 'stdout.log').open('wb') as log:
        p = subprocess.run(argv, cwd=cwd, env=e, stdout=log, stderr=subprocess.STDOUT)
    meta.update(exit_code=p.returncode, end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                log_sha256=sha(target / 'stdout.log'),
                products={str(x): sha(x) for x in sorted(cwd.iterdir()) if x.is_file()})
    write(target / 'receipt.json', meta)
    print(str(target), p.returncode, flush=True)
    return p.returncode

if __name__ == '__main__':
    OUT.mkdir(exist_ok=True)
    for n in ['tmp', 'cache', 'config']:
        (OUT / n).mkdir(exist_ok=True)
    action = sys.argv[1]
    if action == 'auth':
        destination = OUT / (sys.argv[2] if len(sys.argv) > 2 else 'auth.json')
        if destination.exists():
            raise RuntimeError('authentication destination already exists')
        write(destination, auth())
        print('authenticated')
    elif action == 'authenticate-final':
        sys.exit(run('authentication-final-01', [sys.executable, '-B', str(OWN / 'check.py'),
                                               'auth', 'auth-final.json'], OUT))
    elif action == 'compile':
        trial = sys.argv[2]
        src = OUT / (trial + '-source')
        src.mkdir()
        shutil.copyfile(OWN / 'Entry.v', src / 'Entry.v')
        sys.exit(run(trial, [str(ROCQ), 'compile', '-q', '-Q', str(READY / 'simuliris/theories'), 'simuliris',
                            '-Q', str(src), 'EntryHandoff', str(src / 'Entry.v')], src))
    elif action == 'kernel':
        src = OUT / (sys.argv[2] + '-source')
        sys.exit(run(sys.argv[3], [str(ROCQ), 'check', '-o', '-Q', str(READY / 'simuliris/theories'), 'simuliris',
                                  '-Q', str(src), 'EntryHandoff', 'EntryHandoff.Entry'], src))
    elif action == 'query':
        src = OUT / (sys.argv[2] + '-source')
        sys.exit(run(sys.argv[3], [str(ROCQ), 'repl', '-q', '-batch', '-Q', str(READY / 'simuliris/theories'), 'simuliris',
                                  '-Q', str(src), 'EntryHandoff', '-l', str(OUT / sys.argv[4])], src))
