"""Fresh bounded review; raw commands/products stay in the private epoch root.

No delivered acceptance predicates are imported by the independent audit.
Delivered drivers are also exercised, with identical source bytes and only
RESUMPTION output metadata / freeze path labels remapped.
"""
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RESEARCH = ROOT / 'source/tools/research'
PRIVATE = Path('/tmp/miri-grant-component-review-t3-8295fb02')
A = RESEARCH / 'native_miri_grant_refinement'
B = RESEARCH / 'native_miri_permission_grants'
C = RESEARCH / 'native_entry_type_binding'
CLONE = Path('/tmp/native-entry-type-binding-20261110-delivered')
CANON = Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')

def need(ok, message):
    if not ok:
        raise ValueError(message)

def digest(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def read(p):
    return json.loads(Path(p).read_text())

def write(p, value):
    Path(p).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

def snapshot(paths):
    return {str(Path(p).resolve()): digest(p) for p in sorted(set(map(str, paths)))}

def guard(frozen):
    for p, h in frozen.items():
        need(digest(p) == h, 'input drift: ' + p)

def prepare():
    PRIVATE.mkdir(exist_ok=False)
    (PRIVATE/'tmp').mkdir()
    copied = PRIVATE/'owned/source/tools/research/native_miri_grant_refinement'
    copied.mkdir(parents=True)
    mapping = {}
    for src in A.iterdir():
        if src.is_file():
            dst = copied/src.name
            shutil.copyfile(src, dst)
            mapping[str(src)] = str(dst)
    for name in ['infer_eval_support.py', 'infer_experiment.py']:
        src = ROOT/'source/tools'/name
        dst = copied.parents[1]/name
        shutil.copyfile(src, dst)
        mapping[str(src)] = str(dst)
    # Output metadata is the only changed file in the owned driver copy.
    write(copied/'RESUMPTION.json', {'private_output': str(PRIVATE/'grant')})
    (PRIVATE/'grant').mkdir()
    all_inputs = list(A.glob('*')) + [HERE/'review.py']
    all_inputs = [p for p in all_inputs if p.is_file()]
    all_inputs += [Path(p) for p in mapping.values()]
    remaps = {}
    for label, directory, filename in [('permission', B, 'freeze004.json'), ('carrier', C, 'trial004-freeze.json')]:
        original = read(directory/filename)
        mapped = dict(original, inputs={})
        rows = {}
        for old, h in original['inputs'].items():
            if old.startswith('/home/dev/projects/verus/source/target-verus/release/'):
                new = CLONE/Path(old).name
            elif '/source/tools/' in old and old.startswith('/home/dev/') and not old.startswith('/home/dev/.rustup/'):
                new = ROOT/('source/tools/' + old.split('/source/tools/', 1)[1])
            else:
                new = Path(old)
            need(digest(new) == h, 'same-byte remap mismatch: ' + str(new))
            mapped['inputs'][str(new)] = h
            rows[old] = str(new)
        if label == 'carrier':
            mapped['verus_bundle'] = str(CLONE)
        dst = PRIVATE/(label+'-freeze.json')
        write(dst, mapped)
        remaps[label] = {'original_sha256': digest(directory/filename), 'freeze': str(dst), 'sha256': digest(dst), 'mapping': rows}
        all_inputs += [directory/filename, dst] + list(map(Path, mapped['inputs']))
    need(digest(CLONE/'bundle.json') == 'aae1f6b375c4b6f558d6cc62d4268925f959105a79bde920dc102884d72a3012', 'clone manifest')
    for name, h in read(CLONE/'bundle.json')['files'].items():
        need(digest(CLONE/name) == h, 'clone bytes')
    all_inputs += [CLONE/'bundle.json']
    f = read(A/'FREEZE.json')
    all_inputs += list(map(Path, f['tools'])) + [Path(x['path']) for x in f['sources'].values()]
    need(digest(CANON) == '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a', 'canonical receipt')
    canonical = read(CANON)
    need(canonical['success'] is True and len(f['tools']) == 8, 'canonical success/inventory')
    for p, info in f['tools'].items():
        need(digest(p) == info['sha256'] == canonical['artifacts'][p]['sha256'], 'bundle hash')
    all_inputs += [CANON]
    historical = {
        '/tmp/native-miri-grant-resumed-pa16x1fp/suite-mz5vtu97/receipt.json': '20da6694b768c6a8c654af159818443b65adbeb09de98ac7a97863e5be36729f',
        '/tmp/native-miri-grant-resumed-pa16x1fp/coq-f8wu9d37/receipt.json': '5d7b097981ff8900d525d3ee483898dd7d7f95607a86ab2990f58d2389f1556d',
        '/tmp/native-miri-permission-grants-parent004/receipt.json': 'ed662790278e99f40f0890226cc9d2596485c4d64ee208a222f5c809902935f6',
        '/tmp/native-entry-carrier-binding-parent004/receipt.json': 'a8e18bb44878408d42ba3ba79c9765587bd9c458a0deddd4e2f379b608760670',
    }
    for p, h in historical.items():
        need(digest(p) == h and read(p)['success'] is True, 'named receipt authentication')
    all_inputs += list(map(Path, historical))
    # Bytecode guard inspection is static only; no poisoned-output experiments.
    python_files = [HERE/'review.py'] + list(A.glob('*.py')) + [B/'replay.py', C/'check.py']
    ast_rows = {}
    for p in python_files:
        tree = ast.parse(p.read_text())
        need(not any(isinstance(n, ast.Assert) for n in ast.walk(tree)), 'assert acceptance gate: '+str(p))
        for opt in (0, 1, 2):
            compile(tree, str(p), 'exec', optimize=opt)
        ast_rows[str(p.relative_to(ROOT))] = {'sha256': digest(p), 'asserts': 0, 'compiled_optimization_levels': [0,1,2]}
    state = {'schema':1, 'status':'frozen_before_fresh_epochs', 'base': subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip(),
             'private':str(PRIVATE), 'owned_driver':str(copied), 'copy_mapping':mapping,
             'remaps':remaps, 'historical_receipts':historical, 'python_static':ast_rows,
             'inputs':snapshot(all_inputs), 'publication':False, 'goal_complete':False}
    write(HERE/'pretrial.json', state)
    print('Frozen', len(state['inputs']), 'inputs')

def run_epoch(label):
    state = read(HERE/'pretrial.json')
    guard(state['inputs'])
    copied = Path(state['owned_driver'])
    cmds = {
        'grant-suite': [sys.executable,'-B','-O',str(copied/'replay.py')],
        'grant-mutations': [sys.executable,'-B','-O',str(copied/'mutations.py')],
        'coq': [sys.executable,'-B','-O',str(copied/'coq_trial.py')],
        'permission': [sys.executable,'-B','-O',str(B/'replay.py'),'--freeze',str(PRIVATE/'permission-freeze.json'),'--output',str(PRIVATE/'permission')],
        'carrier': [sys.executable,'-B','-O',str(C/'check.py'),'--freeze',str(PRIVATE/'carrier-freeze.json'),'--output',str(PRIVATE/'carrier')],
    }
    rec = {'label':label,'argv':cmds[label], 'cwd':str(PRIVATE), 'before':digest(HERE/'pretrial.json'), 'status':'running'}
    dst = PRIVATE/(label+'-terminal.json')
    need(not dst.exists(), 'epoch already attempted')
    write(dst,rec)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', PYTHONOPTIMIZE='1', RUSTUP_AUTO_INSTALL='0', TMPDIR=str(PRIVATE/'tmp'), XDG_CACHE_HOME=str(PRIVATE/'cache'))
    start = time.time()
    try:
        with (PRIVATE/(label+'.stdout')).open('w') as out, (PRIVATE/(label+'.stderr')).open('w') as err:
            proc = subprocess.Popen(cmds[label], cwd=PRIVATE, env=env, stdout=out, stderr=err)
            rec['pid'] = proc.pid
            write(dst,rec)
            rec['returncode'] = proc.wait(timeout=900)
        guard(state['inputs'])
        rec['final_guards'] = True
        need(rec['returncode'] == 0, 'delivered epoch failed: '+label)
        rec['status'] = 'finished'
    except BaseException as e:
        rec['status'] = 'error'
        rec['failure'] = {'type':type(e).__name__,'message':str(e)}
        raise
    finally:
        rec['seconds'] = time.time()-start
        rec['logs'] = {n:digest(PRIVATE/(label+'.'+n)) for n in ['stdout','stderr']}
        write(dst,rec)
    print(label, rec['returncode'])

if __name__ == '__main__':
    if sys.argv[1] == 'prepare':
        prepare()
    else:
        run_epoch(sys.argv[1])
