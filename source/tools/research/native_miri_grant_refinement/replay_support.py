"""Local strict evidence gates; shared checking utilities remain unchanged."""
import json, os, pathlib, sys
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run
EXEC = {'Item::perm', 'Item::protected', 'Item::set_permission', 'Permission::from_bits',
        'Permission::to_bits', 'Stack::get', 'Stack::find_first_write_incompatible',
        '<Item as std::clone::Clone>::clone', '<Permission as std::clone::Clone>::clone'}
SPEC = {'Item::permission', 'bits', 'decode'}
def private_root():
    return pathlib.Path(json.loads((HERE/'RESUMPTION.json').read_text())['private_output'])
def authenticate():
    f = json.loads((HERE/'FREEZE.json').read_text())
    receipt = pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
    require(digest(receipt) == f['receipt_sha256'], 'accepted bundle receipt drift')
    r = json.loads(receipt.read_text()); require(r['success'] is True)
    files = [receipt, HERE/'FREEZE.json', HERE/'RESUMPTION.json', HERE/'selected.json', HERE/'literal.rs']
    for p, entry in f['tools'].items():
        require(digest(p) == entry['sha256'] == r['artifacts'][p]['sha256'], 'tool drift: '+p); files.append(pathlib.Path(p))
    for entry in f['sources'].values():
        require(digest(entry['path']) == entry['sha256'], 'source drift'); files.append(pathlib.Path(entry['path']))
    require(digest(HERE/'selected.json') == f['selected_sha256'])
    require(digest(HERE/'literal.rs') == f['literal_sha256'])
    bundle = pathlib.Path(next(iter(f['tools']))).parent
    files += list(HERE.glob('*.py'))
    files += [pathlib.Path(__file__).resolve().parents[2]/n for n in ['infer_experiment.py','infer_eval_support.py']]
    rustc = pathlib.Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc')
    files += [rustc, pathlib.Path('/home/dev/.cargo/bin/rustc').resolve()]
    return f, bundle, files

def environment(bundle, out):
    return dict(os.environ, PATH='/home/dev/.cargo/bin:'+os.environ['PATH'], RUSTUP_TOOLCHAIN='1.99.1',
                VERUS_Z3_PATH=str(bundle/'z3'), VERUS_INFER_CACHE=str(out/'cache'),
                TMPDIR=str(out), XDG_CACHE_HOME=str(out/'cache'))

def validate(data, rc, stderr, expected, *, extra=None, frontend=False, diagnostic=False):
    validate_run({'returncode':rc, 'stdout':json.dumps(data), 'stderr':stderr}, expected, allow_frontend=frontend)
    v = data['verification-results']; inf = data['verus-infer']; scope = inf['verification_scope']
    require(inf['functions'] == [] and inf['requested_functions'] == 0 and inf['generated_functions'] == 0)
    require(inf['interface_written'] is False, 'unexpected interface publication')
    require(all(not entries for key, entries in inf.items() if key.endswith('_assistance') and isinstance(entries,list)), 'unexpected assistance')
    if frontend:
        require(v.get('encountered-vir-error') is True and 'loop must have a decreases clause' in stderr)
        return
    require(v['is-verifying-entire-crate'] is True and scope['native_collection_completed'] is True)
    require(v['errors'] == 0 if expected else v['errors'] > 0)
    bodies = scope['native_functions']
    expected_exec = EXEC | ({extra} if extra else set())
    selected_exec = [b for b in bodies if b['rust_function'] in expected_exec]
    require(len(selected_exec)==len(expected_exec) and {b['rust_function'] for b in selected_exec}==expected_exec, 'body coverage')
    require(all(b['selected'] for b in selected_exec), 'body not selected')
    require({b['rust_function'] for b in bodies if b['body_status']=='specification_definition_or_declaration'}==SPEC, 'spec inventory')
    require(not any(b['selected'] and b['body_status'] in ['external_specification','unverified_body_with_contract','interface_without_checked_body'] for b in bodies))
    if expected:
        require(scope['complete_selected_scope_verified'] is True and scope['checked_exec_bodies']==len(expected_exec) and scope['checked_proof_bodies']==0)
        require(all(b['body_status']=='checked_exec_body' for b in selected_exec))
    elif not diagnostic:
        require(v['errors']==1 and 'assertion failed' in stderr, 'single property assertion required')
        require(not any(s in stderr for s in ['precondition not satisfied','postcondition not satisfied','possible arithmetic','invariant not satisfied','error[E','recommendation not met']))
        # Negative crate body statuses are globally uncertified. Health is established
        # by the identical full positive companion plus the sole error's control span.
        import re
        m=re.search(r'error: assertion failed\n\s*--> .*component.rs:(\d+):',stderr)
        require(m is not None, 'missing diagnostic span')
        return int(m[1])
