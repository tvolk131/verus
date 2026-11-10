#!/usr/bin/env python3
"""Bounded independent review. All acceptance gates remain active under -O.

Only completed named artifacts and this checkout are read. Historical paths are
labels, mapped explicitly before any read; source bytes are never patched.
"""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
DELIVERED = ROOT / 'source/tools/research/native_miri_find_granting'
EPOCH = Path('/tmp/native-miri-find-granting-20261110-impl-final008')
BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
CANON = Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
CANON_SHA = '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a'
RECEIPT_SHA = '297c67eae1dc1242ad0c1f08175548fc16b2167f6c267fdf8b0a5e15ef1e0075'
FREEZE_SHA = '9c75f4c548c20d8783591ba41cf383ac1878e48e8f6924f28d15994f747aa3f3'
OLD = '/home/dev/.t3/worktrees/verus/t3-4855bc31/'
NAMES = ['verus', 'rust_verify', 'vstd.vir', 'libvstd.rlib', 'libverus_builtin.rlib', 'libverus_builtin_macros.so', 'libverus_state_machines_macros.so', 'z3']
sys.path.insert(0, str(ROOT / 'source/tools'))
sys.path.insert(0, str(DELIVERED))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt
from fidelity import check as fidelity


def need(value, message):
    if not value:
        raise ValueError(message)


def readj(path):
    return json.loads(Path(path).read_text())


def writej(path, data):
    Path(path).write_text(json.dumps(data, indent=2) + '\n')


def rebind(path):
    text = str(path)
    if text.startswith(OLD):
        return ROOT / text[len(OLD):]
    if Path(path).is_relative_to(ROOT):
        return Path(path)
    need(not text.startswith('/home/dev/.t3/worktrees/'), 'unmapped historical checkout')
    return Path(path)


def bundle_auth():
    need(digest(CANON) == CANON_SHA, 'canonical receipt hash')
    canonical = readj(CANON)
    need(canonical['success'] is True, 'canonical receipt success')
    result = {}
    for name in NAMES:
        path = BUNDLE / name
        result[str(path)] = digest(path)
        need(result[str(path)] == canonical['artifacts'][str(path)]['sha256'], 'bundle identity ' + name)
    return result


def retained_accept(receipt):
    """Delivered acceptance, with only its SHA reader explicitly rebound.

    This supplements the independent checks below. Imports originate in own
    identical frozen helpers; HERE/ROOT refer to this checkout, never OLD.
    """
    spec = importlib.util.spec_from_file_location('delivered_cache_accept', DELIVERED / 'accept.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.sha = lambda p: digest(rebind(p))
    argv = sys.argv
    try:
        sys.argv = [str(DELIVERED / 'accept.py'), str(receipt)]
        with contextlib.redirect_stdout(io.StringIO()) as output:
            module.main()
        return json.loads(output.getvalue())
    finally:
        sys.argv = argv


def authenticate(dest):
    dest.mkdir(exist_ok=False)
    need(digest(EPOCH / 'output/receipt.json') == RECEIPT_SHA, 'delivered receipt identity')
    need(digest(EPOCH / 'freeze.json') == FREEZE_SHA, 'delivered freeze identity')
    receipt = readj(EPOCH / 'output/receipt.json')
    frozen = readj(EPOCH / 'freeze.json')
    need(receipt['frozen_inputs'] == frozen and receipt['frozen_file_sha256'] == FREEZE_SHA, 'embedded freeze identity')
    need(frozen['before_first_trial'] is True, 'pretrial designation')
    inputs, mapping, snapshots = [], {}, []
    for label, record in frozen['inputs'].items():
        path = rebind(label)
        need(digest(path) == record['sha256'] and path.stat().st_size == record['size'], 'frozen byte identity ' + label)
        inputs.append(dict(label=label, path=str(path), sha256=record['sha256'], bytes=record['size']))
        if label.startswith(OLD):
            mapping[label] = str(path)
            if path.parent == DELIVERED:
                snap = EPOCH / 'completed-own-helper-snapshot' / path.name
                need(digest(snap) == record['sha256'], 'completed helper snapshot ' + path.name)
                snapshots.append(dict(path=str(snap), sha256=record['sha256']))
    need(len(inputs) == 45 and len(mapping) == 12 and len(snapshots) == 9, 'frozen inventory counts')
    bundle = bundle_auth()
    products = []
    for name, sha in receipt['products'].items():
        path = EPOCH / 'output' / name
        need(digest(path) == sha, 'product identity ' + name)
        products.append(dict(path=str(path), sha256=sha))
    need(len(products) == 22, 'product count')
    accepted = retained_accept(EPOCH / 'output/receipt.json')
    attempts = readj(DELIVERED / 'ATTEMPTS.json')
    history = []
    for epoch in attempts['epochs']:
        path = Path(epoch['path']) / 'output/receipt.json'
        need(digest(path) == epoch['receipt_sha256'], 'retained epoch hash ' + epoch['epoch'])
        r = readj(path)
        need(len(r['runs']) == epoch['verifier_runs'], 'retained run count')
        rows = []
        for run, listed in zip(r['runs'], epoch['runs']):
            need(run['name'] == listed['name'] and run['exit_code'] == listed['exit_code'], 'retained run identity')
            for suffix, sha in listed['log_hashes'].items():
                need(digest(path.parent / (run['name'] + '.' + suffix)) == sha, 'retained log hash')
            rows.append(dict(name=run['name'], exit_code=run['exit_code'], recorded=run['classification'], corrected=listed['final_classification']))
        history.append(dict(epoch=epoch['epoch'], receipt=str(path), sha256=digest(path), accepted_final=epoch['accepted_final_evidence'], success=r['success'], failure=r.get('failure'), runs=rows))
    need(len(history) == 8 and sum(len(x['runs']) for x in history) == 45, 'attempt retention')
    result = dict(success=True, inputs=inputs, explicit_identical_byte_mapping=mapping, completed_helper_snapshots=snapshots, bundle=bundle, canonical_receipt_sha256=CANON_SHA, products=products, delivered_acceptance=accepted, retained_attempts=history)
    writej(dest / 'authentication.json', result)
    print(json.dumps(dict(success=True, inputs=len(inputs), rebindings=len(mapping), helper_snapshots=len(snapshots), products=len(products), retained_epochs=len(history), retained_runs=45)))


def prepare(dest, authentication):
    dest.mkdir(exist_ok=False)
    auth = readj(authentication)
    need(auth['success'] is True, 'authentication prerequisite')
    cases = []
    old_plan = readj(EPOCH / 'plan.json')
    for case in old_plan['cases']:
        c = dict(case)
        source = Path(c['source'])
        need(source.parent == EPOCH and digest(source) == c['source_sha256'], 'completed case source')
        own = dest / source.name
        own.write_bytes(source.read_bytes())
        c['source'] = str(own)
        cases.append(c)
    # Exact candidate bytes, not happy-proof annotation changes.
    for c in cases[:3]:
        need(Path(c['source']).read_bytes() == (DELIVERED / 'cache_component.rs').read_bytes(), 'positive candidate bytes')
    writej(dest / 'plan.json', dict(schema=1, cases=cases))
    artifacts = [Path(x['path']) for x in auth['inputs']]
    artifacts += [Path(x['path']) for x in auth['completed_helper_snapshots']]
    artifacts += [Path(x['path']) for x in auth['products']]
    artifacts += [Path(authentication), EPOCH / 'freeze.json', EPOCH / 'output/receipt.json']
    artifacts += [p for p in HERE.iterdir() if p.is_file()]
    artifacts += [p for p in DELIVERED.iterdir() if p.is_file()]
    artifacts += [dest / 'plan.json', *[Path(c['source']) for c in cases]]
    guard = ArtifactGuard(artifacts)
    writej(dest / 'freeze.json', dict(schema=1, before_first_trial=True, inputs=guard.initial))
    print(str(dest / 'freeze.json'))


def run(dest):
    frozen = readj(dest / 'freeze.json')
    need(frozen['before_first_trial'] is True, 'not pretrial')
    guard = ArtifactGuard([Path(p) for p in frozen['inputs']] + [dest / 'freeze.json'])
    need(all(guard.initial[p] == record for p, record in frozen['inputs'].items()), 'pretrial drift')
    bundle_auth()
    out = dest / 'output'
    out.mkdir(exist_ok=False)
    r = dict(schema=1, commands=[], runs=[], frozen_inputs=frozen, frozen_file_sha256=digest(dest / 'freeze.json'), native_correspondences=0, publication=False, goal_complete=False, authenticated_bundle_names=NAMES, canonical_receipt_sha256=CANON_SHA)
    with replay_receipt(out, r):
        for n in ['tmp', 'cache', 'target', 'xdg']:
            (out / n).mkdir()
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'], RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(BUNDLE / 'z3'), PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(out / 'tmp'), XDG_CACHE_HOME=str(out / 'xdg'), CARGO_TARGET_DIR=str(out / 'target'))
        runner = LoggedCommands(out, out, r['commands'], timeout=90, guard=guard)
        for c in readj(dest / 'plan.json')['cases']:
            need(digest(c['source']) == c['source_sha256'], 'planned source changed')
            env['VERUS_INFER_CACHE'] = str(out / 'cache' / c['cache'])
            argv = [str(BUNDLE / 'verus'), c['source'], '--crate-type=lib', '--no-cheating', '--output-json', '--multiple-errors', '20'] + c['extra']
            if c['infer']:
                argv.append('--infer-goals')
            code = runner(c['name'], argv, env)
            data = readj(out / (c['name'] + '.stdout'))
            v = data.get('verification-results', {})
            stderr = (out / (c['name'] + '.stderr')).read_text()
            if any(x in stderr.lower() for x in ['panicked at', 'timed out', 'resource limit']):
                kind = 'resource_or_internal_failure'
            elif v.get('encountered-vir-error'):
                kind = 'frontend_failure'
            elif v.get('errors', 0) > 0:
                kind = 'proof_failure'
            elif code == 0 and v.get('success') is True:
                kind = 'ordinary_success'
            else:
                kind = 'setup_or_unclassified_failure'
            r['runs'].append(dict(name=c['name'], source_sha256=c['source_sha256'], exit_code=code, classification=kind, verification=v))
            writej(out / 'progress.json', r)
            print(json.dumps(dict(name=c['name'], exit_code=code, classification=kind, verification=v)), flush=True)
        guard.check()
        r['all_final_guards_passed'] = True
        r['products'] = {str(p.relative_to(out)): digest(p) for p in out.rglob('*') if p.is_file() and p.name not in ['progress.json', 'receipt.json']}
    print('receipt', str(out / 'receipt.json'), digest(out / 'receipt.json'))


def accept(dest):
    out = dest / 'output'
    r = readj(out / 'receipt.json')
    need(r['success'] is True and r['all_final_guards_passed'] is True, 'incomplete replay')
    guard = ArtifactGuard([Path(p) for p in r['frozen_inputs']['inputs']] + [dest / 'freeze.json'])
    need(all(guard.initial[p] == v for p, v in r['frozen_inputs']['inputs'].items()), 'postrun drift')
    result = retained_accept(out / 'receipt.json')
    inventory = readj(DELIVERED / 'INVENTORY.json')
    runs = []
    for c in r['commands']:
        n = c['label']
        d = readj(out / (n + '.stdout'))
        inf = d.get('verus-infer', {})
        if n.startswith('cache-'):
            positive = n in ['cache-enabled-cold', 'cache-enabled-warm', 'cache-assistance-off']
            need(inf['status'] == ('no_inference' if positive else 'caller_proof_failed'), 'unexpected inference status')
            need(inf['functions'] == [] and inf['imported_inference_dependencies'] == [], 'unexpected helper/dependency')
            scope = inf['verification_scope']
            native = scope['native_functions']
            # Independently bind selected and explicitly excluded names, including
            # ignored Rust derive bodies; query count is never body count.
            for selected, key in [(True, 'selected'), (False, 'excluded')]:
                observed = {f['rust_function'] for f in native if f['selected'] is selected}
                need(observed == {f['rust_function'] for f in inventory[key]}, 'inventory names ' + key)
            if n in ['cache-enabled-cold', 'cache-enabled-warm', 'cache-assistance-off']:
                observed = {(f['rust_function'], f['body_status']) for f in native}
                need(observed == {(f['rust_function'], f['body_status']) for f in inventory['selected'] + inventory['excluded']}, 'exact positive body statuses')
        runs.append(dict(name=n, argv=c['argv'], cwd=c['cwd'], exit_code=c['exit_code'], environment=c['environment'], verification=d['verification-results'], log_hashes={suffix: digest(out / (n + '.' + suffix)) for suffix in ['stdout', 'stderr']}))
    guard.check()
    result.update(independent_inventory_check=True, receipt=str(out / 'receipt.json'), freeze=str(dest / 'freeze.json'), freeze_sha256=digest(dest / 'freeze.json'), final_guards=True, inputs=len(r['frozen_inputs']['inputs']), products=r['products'], commands=runs, selected=inventory['selected'], excluded=inventory['excluded'], manual_proof_blocks=1, authored_proof_functions=1, causal_assistance_gain=False)
    writej(dest / 'acceptance.json', result)
    print(json.dumps(dict(success=True, runs=len(runs), inputs=result['inputs'], products=len(result['products']), receipt_sha256=result['receipt_sha256'], acceptance_sha256=digest(dest / 'acceptance.json'))))


if __name__ == '__main__':
    mode, path = sys.argv[1], Path(sys.argv[2]).resolve()
    if mode == 'authenticate':
        authenticate(path)
    elif mode == 'prepare':
        prepare(path, Path(sys.argv[3]).resolve())
    elif mode == 'run':
        run(path)
    elif mode == 'accept':
        accept(path)
    else:
        raise ValueError('unknown mode')
