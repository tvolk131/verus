"""Owned bounded epochs only. No old controller execution or shared cache writes."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
TC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined69-bundle-3aa7f49a')
BUNDLE_RECEIPT = '2e3fcc58f6fca9364061329104710937ec2aac6aab95a5c48d068c1ca6ee9bb7'
TOOLS = {'verus', 'rust_verify', 'z3', 'libverus_builtin.rlib',
         'libverus_builtin_macros.so', 'libverus_state_machines_macros.so',
         'vstd.vir', 'libvstd.rlib'}


def need(ok, message):
    if not ok:
        raise RuntimeError(message)


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for b in iter(lambda: f.read(1024 * 1024), b''):
            h.update(b)
    return h.hexdigest()


def write(path, obj):
    Path(path).write_text(json.dumps(obj, indent=2, sort_keys=True) + '\n')


def authenticate():
    need(sha(BUNDLE / 'receipt.json') == BUNDLE_RECEIPT, 'baseline receipt hash')
    r = json.loads((BUNDLE / 'receipt.json').read_text())
    need(r['success'] is True and r['all_final_guards_passed'] is True
         and r['matching_vstd_verified'] == 2250, 'baseline accepted success')
    need(r['schema_version'] == 38 and r['policy'] == 'candidate-bound-evidence-v69'
         and r['cache'].startswith('verus-infer-v72:'), 'baseline versions')
    need(set(r['tools']) == TOOLS, 'exact eight tools')
    for n, h in r['tools'].items():
        need(sha(BUNDLE / n) == h, 'baseline tool drift: ' + n)
    parent = Path('/tmp/verus-infer-buffer-cycle-20261109/item-tag-parent-v2/receipt.json')
    need(sha(parent) == 'ba75225dcf12c21a8f7c17fcfdef493d1083aa3ed2c865d74740b0a66de476aa',
         'parent tag receipt hash')
    pr = json.loads(parent.read_text())
    need(pr['success'] is True and pr['all_final_guards_passed'] is True,
         'parent tag acceptance')
    return r


def frozen_inputs(extra=()):
    authenticate()
    tracked = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).decode().split('\0')
    paths = [ROOT / p for p in tracked if p and (ROOT / p).is_file()]
    paths += [p for p in HERE.iterdir() if p.is_file() and p.name not in
              {'attempts.jsonl', 'RESULTS.json', 'REPORT.md'}]
    paths += [BUNDLE / n for n in TOOLS] + [BUNDLE / 'receipt.json', TC / 'bin/rustc']
    paths += list((TC / 'lib').glob('librustc_driver*'))
    paths += list((TC / 'lib/rustlib/x86_64-unknown-linux-gnu/lib').glob('libcore-*'))
    paths += [TC / 'lib/rustlib/src/rust/library/core/src/convert/num.rs',
              Path(sys.executable).resolve()]
    paths += list(map(Path, extra))
    return {str(p): sha(p) for p in sorted(set(paths))}


def epoch(label, argv, plan, extra=(), env_extra=None, timeout=180, source=None):
    d = Path(tempfile.mkdtemp(prefix='checked-bool-' + label + '-', dir='/tmp'))
    d.chmod(0o700)
    argv = [str(a).replace('@EPOCH@', str(d)) for a in argv]
    env = {**os.environ, 'PATH': '/home/dev/.cargo/bin:/usr/bin:/bin',
           'RUSTUP_TOOLCHAIN': '1.99.1', 'RUSTC_BOOTSTRAP': '1',
           'LD_LIBRARY_PATH': str(TC / 'lib'), 'TMPDIR': str(d),
           'CARGO_BUILD_JOBS': '2', 'CARGO_NET_OFFLINE': 'true',
           'CARGO_HOME': str(d / 'cargo-home'), 'CARGO_TARGET_DIR': str(d / 'target'),
           'CARGO_BUILD_BUILD_DIR': str(d / 'target'),
           'VERUS_ROOT': str(BUNDLE), 'VERUS_Z3_PATH': str(BUNDLE / 'z3'),
           'VERUS_INFER_CACHE': str(d / 'cache'), 'XDG_CACHE_HOME': str(d / 'cache')}
    env.update(env_extra or {})
    if source is not None:
        (d / 'component.rs').write_text(source)
        extra = [*extra, d / 'component.rs']
    write(d / 'plan.json', plan)
    paths = frozen_inputs([*extra, d / 'plan.json'])
    env_record = {k: env[k] for k in ['PATH', 'RUSTUP_TOOLCHAIN', 'RUSTC_BOOTSTRAP',
                 'LD_LIBRARY_PATH', 'TMPDIR', 'CARGO_BUILD_JOBS', 'CARGO_NET_OFFLINE',
                 'CARGO_HOME', 'CARGO_TARGET_DIR', 'VERUS_ROOT', 'VERUS_Z3_PATH',
                 'CARGO_BUILD_BUILD_DIR',
                 'VERUS_INFER_CACHE', 'XDG_CACHE_HOME']}
    record = dict(label=label, argv=argv, cwd=str(ROOT / 'source'),
                  environment=env_record, plan=plan, inputs=paths)
    write(d / 'freeze.json', record)
    frozen = sha(d / 'freeze.json')
    for p in HERE.iterdir():
        if p.is_file() and p.name not in {'attempts.jsonl', 'RESULTS.json', 'REPORT.md'}:
            shutil.copyfile(p, d / ('controller-' + p.name))
    start = time.monotonic()
    try:
        with (d / 'stdout').open('wb') as out, (d / 'stderr').open('wb') as err:
            r = subprocess.run(argv, cwd=ROOT / 'source', env=env, stdout=out,
                               stderr=err, timeout=timeout)
        record['exit'] = r.returncode
    except subprocess.TimeoutExpired:
        record['exit'] = None
        record['failure_class'] = 'resource_timeout'
    except OSError as e:
        record['exit'] = None
        record['failure_class'] = 'setup_launch_failure'
        (d / 'launch-error').write_text(repr(e))
    record['seconds'] = time.monotonic() - start
    record['final_guards_passed'] = sha(d / 'freeze.json') == frozen and all(
        Path(p).is_file() and sha(p) == h for p, h in paths.items())
    if 'private_root' in plan:
        target = Path(plan['private_root']) / 'target'
        record['external_build_products'] = {str(p): sha(p) for p in target.rglob('*')
            if p.is_file() and (p.suffix in {'.rlib', '.rmeta', '.so', '.d'} or p.name == 'libvir.rlib')}
    record['products'] = {str(p.relative_to(d)): sha(p) for p in d.rglob('*') if p.is_file()}
    write(d / 'receipt.json', record)
    with (HERE / 'attempts.jsonl').open('a') as f:
        f.write(json.dumps(dict(label=label, path=str(d), exit=record['exit'],
                               receipt_sha256=sha(d / 'receipt.json'),
                               guards=record['final_guards_passed'])) + '\n')
    need(record['final_guards_passed'], 'epoch input/freeze drift')
    print(json.dumps(dict(label=label, path=str(d), exit=record['exit'], guards=True)), flush=True)
    return d


def private_cache():
    """Only locked archives/index entries and immutable getopts Git database."""
    src = Path('/tmp/constructor-function-value-review-t3-755cb0d1/cargo-home')
    d = Path(tempfile.mkdtemp(prefix='checked-bool-build-', dir='/tmp'))
    d.chmod(0o700)
    home = d / 'cargo-home'
    lock = tomllib.loads((ROOT / 'source/Cargo.lock').read_text())
    packages = lock['package']
    by_key = {(p['name'], p['version']): p for p in packages}
    selected_keys = set()

    def visit(p):
        key = (p['name'], p['version'])
        if key in selected_keys:
            return
        selected_keys.add(key)
        for dep in p.get('dependencies', []):
            words = dep.split()
            found = [v for k, v in by_key.items() if k[0] == words[0]
                     and (len(words) < 2 or k[1] == words[1])]
            need(len(found) == 1, 'unambiguous locked dependency ' + dep)
            visit(found[0])

    visit(next(p for p in packages if p['name'] == 'vir'))
    selected = {}
    missing = []
    for p in packages:
        if not p.get('source', '').startswith('registry+'):
            continue
        name = p['name']
        filename = f"{name}-{p['version']}.crate"
        candidates = list((src / 'registry/cache').glob('*/' + filename)) if (p['name'], p['version']) in selected_keys else []
        candidates = [a for a in candidates if sha(a) == p['checksum']]
        if candidates:
            archive = candidates[0]
            dest = home / 'registry/cache/index.crates.io-1949cf8c6b5b557f' / filename
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(archive, dest)
            selected[str(archive)] = sha(archive)
        elif (p['name'], p['version']) in selected_keys:
            missing.append(dict(name=filename, checksum=p['checksum']))
        key = ('1/' + name if len(name) == 1 else '2/' + name if len(name) == 2
               else '3/' + name[0] + '/' + name if len(name) == 3
               else name[:2] + '/' + name[2:4] + '/' + name)
        indices = list((src / 'registry/index').glob('*/.cache/' + key))
        if not indices:
            missing.append(dict(index=key))
            continue
        dest = home / 'registry/index/index.crates.io-1949cf8c6b5b557f/.cache' / key
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(indices[0], dest)
        selected[str(indices[0])] = sha(indices[0])
    write(home / 'registry/index/index.crates.io-1949cf8c6b5b557f/config.json',
          {'dl': 'https://static.crates.io/crates', 'api': 'https://crates.io'})
    git_packages = [p for p in lock['package'] if p.get('source', '').startswith('git+')]
    need(len(git_packages) == 1 and git_packages[0]['name'] == 'getopts', 'bounded git source')
    commit = git_packages[0]['source'].split('#')[1]
    db = src / 'git/db/getopts-a5b2265512d62342'
    subprocess.run(['git', '--git-dir=' + str(db), 'cat-file', '-e', commit + '^{commit}'],
                   check=True, stdout=subprocess.DEVNULL)
    # clone --bare copies only the authorized Git object database, never a checkout/config execution.
    dest = home / 'git/db/getopts-a5b2265512d62342'
    dest.parent.mkdir(parents=True)
    subprocess.run(['git', '-c', 'core.hooksPath=/dev/null', 'clone', '--bare', '--no-hardlinks',
                    str(db), str(dest)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    write(d / 'cache-inputs.json', dict(lock_sha256=sha(ROOT / 'source/Cargo.lock'),
                                      selected=selected, missing=missing, git_commit=commit))
    return d


if __name__ == '__main__':
    need(sys.flags.dont_write_bytecode, 'run with Python -B')
    if sys.argv[1] == 'build-vir':
        d = private_cache()
        epoch('build-vir', [TC / 'bin/cargo', 'build', '--offline', '--locked', '-p', 'vir'],
              dict(kind='setup_build_ordinary_vir_only', private_root=str(d),
                   tool_bundle=authenticate()['tools'], no_foreign_contract=True),
              extra=[d / 'cache-inputs.json'],
              env_extra={'CARGO_HOME': str(d / 'cargo-home'), 'CARGO_TARGET_DIR': str(d / 'target'),
                         'CARGO_BUILD_BUILD_DIR': str(d / 'target')},
              timeout=900)
    elif sys.argv[1] == 'compile-inspector':
        lib = Path(sys.argv[2])
        epoch('compile-inspector', [TC / 'bin/rustc', HERE / 'inspect.rs',
              '--edition=2024', '-o', '@EPOCH@/inspect', '-L', TC / 'lib',
              '--extern', 'vir=' + str(lib), '-L', 'dependency=' + str(lib.parent)],
              dict(kind='diagnostic_build_no_contract', ordinary_VIR=True,
                   selected_body='actual resolved core From<bool> for u64',
                   primitive_observation='ReturnBoolToU64(Copy(_1), _0)'),
              extra=[HERE / 'inspect.rs', *lib.parent.glob('*.rlib'), *lib.parent.glob('*.so')])
    elif sys.argv[1] == 'inspect':
        binary = Path(sys.argv[2])
        epoch('typed-inspection', [binary, HERE / 'call_shapes.rs', '--crate-type=lib',
              '--edition=2024', '--sysroot', TC, '--emit=metadata', '-o', '@EPOCH@/caller.rmeta'],
              dict(kind='typed_identity_and_shape_diagnostics',
                   selected='probe_actual: actual core bool/u64 From instance',
                   excluded=['probe_other_output', 'probe_generic', 'probe_user_trait',
                             'probe_function_pointer'],
                   proposed_contract=None, contract_available=False,
                   expected_control_count=12, ordinary_VIR=True,
                   structural_VIR_controls=8, proof=False),
              extra=[binary, HERE / 'call_shapes.rs'])
    elif sys.argv[1] == 'proof':
        kind = sys.argv[2]
        need(kind in {'actual', 'mirror', 'full-help', 'full-helpers'}, 'bounded proof case')
        origin = HERE.parent / 'item_tag_constructor_composition'
        sys.path.insert(0, str(origin))
        from source import generate, fidelity, load
        pin = HERE.parent / 'native_miri_policy_pin/pins.json'
        pins = json.loads(pin.read_text())
        upstream = [Path(pins['sources'][n]['path']) for n in [
            'src/tools/miri/src/borrow_tracker/mod.rs',
            'src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']]
        if kind in {'actual', 'mirror'}:
            expression = 'u64::from(b)' if kind == 'actual' else 'b as u64'
            source = ('use vstd::prelude::*;\nverus! {\n'
                      'pub fn all_bool(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64},'
                      ' { ' + expression + ' }\n}\n')
            expected = dict(query_result=[0, 1] if kind == 'actual' else [1, 0],
                            no_helpers=True, selected_ordinary_exec=['component::all_bool'],
                            excluded_native=[], body_origin='actual std call' if kind == 'actual'
                            else 'local cast diagnostic; not foreign-body proof')
        else:
            source = generate(kind)
            expected = dict(fidelity=fidelity(source),
                            original_scope=json.loads((origin / 'identities.json').read_text())[kind],
                            query_result=[36, 1] if kind == 'full-help' else [36, 0],
                            failed_function='component::Item::new' if kind == 'full-help' else None,
                            manual_packing_support='unchanged original adapter inventory')
        epoch(kind, [BUNDLE / 'verus', '@EPOCH@/component.rs', '--crate-name=component',
              '--crate-type=lib', '--no-cheating', '--infer-goals', '--output-json',
              '--multiple-errors', '20', '--log-all', '--log-dir', '@EPOCH@/vir',
              '--compile', '-o', '@EPOCH@/libcomponent.rlib', '--export', '@EPOCH@/component.vir'],
              dict(kind='ordinary_proof_gap' if kind in {'actual', 'full-help'} else
                   'independent_diagnostic_support', case=kind, expected=expected,
                   foreign_body_registered=False, checked_foreign_instance=False,
                   publication=False), source=source,
              extra=[origin / 'source.py', origin / 'identities.json', pin, *upstream])
    else:
        raise RuntimeError('unknown operation')
