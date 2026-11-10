"""Independent bounded review; no historical driver imports or execution. Python -B -O."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
DEL = REPO / 'source/tools/research/native_miri_tag_representation'
BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
CANON = Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
AUTH = {
    CANON: '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a',
    Path('/tmp/native-miri-tag-delivery-wwm_pq9c/receipt.json'): '855f7ca4dae41e7ac7df00fb543d4ba96f1cf2dad8f6ce9c8ad3344520ec3721',
    Path('/tmp/native-miri-tag-parent-audit-9myohdjy/receipt.json'): 'e12f684689d47dc61cb2ac688bdf59f6829c0499c4f735390d06d1fc1c7d38f5',
    DEL/'RESULTS.json': 'cc6ee8aee77c3e55a34f23eafc3b7a67aceab687cf00099036da784918dc7cef',
}
TOOLS = ['verus', 'rust_verify', 'vstd.vir', 'libvstd.rlib', 'libverus_builtin.rlib',
         'libverus_builtin_macros.so', 'libverus_state_machines_macros.so', 'z3']
PROOFS = ['packed_fields', 'updated_fields', 'tag_range', 'packed_fields_raw']
EXEC = ['BorTag::get', 'BorTag::inner', 'Item::perm', 'Item::protected',
        'Item::set_permission', 'Permission::to_bits', 'Permission::from_bits']
EXEC += ['<'+t+' as std::clone::Clone>::clone' for t in ['BorTag', 'Item', 'Permission']]
EXACT = EXEC + PROOFS + ['tag_identity']
ADAPTER = EXACT + ['BorTag::new', 'BorTag::one', 'Item::tag', '<BorTag as std::default::Default>::default']
SKIP = ['specification_definition_or_declaration', 'selected_item_without_translated_body']


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for chunk in iter(lambda: f.read(1048576), b''):
            h.update(chunk)
    return h.hexdigest()


def read(path):
    return json.loads(Path(path).read_bytes())


def write(path, data):
    Path(path).write_text(json.dumps(data, indent=2, sort_keys=True)+'\n')


def identities(paths):
    result = {}
    for path in sorted(set(map(str, paths))):
        p = Path(path)
        s = p.stat()
        result[path] = dict(sha256=sha(p), bytes=s.st_size, inode=s.st_ino, mtime_ns=s.st_mtime_ns)
    return result


def guard(snapshot):
    require(identities(snapshot) == snapshot, 'frozen input changed')


def body(text, start):
    pos = text.index('{', start)
    depth = 1
    end = pos+1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[pos:end]


def erased_body(text, marker):
    require(text.count(marker) == 1, 'ambiguous body '+marker)
    start = text.index(marker)
    start = text.index('\n    {', start)
    result = body(text, start)
    while 'proof {' in result:
        start = result.index('proof {')
        b = body(result, start)
        result = result[:start] + result[start+len('proof ')+len(b):]
    return result


def tokens(text):
    return re.findall(r'\w+|[^\s]', re.sub(r'//[^\n]*', '', text))


def static_checks(rows):
    selection = read(DEL/'selection.json')
    primary = {k: Path(v['path']).read_text() for k, v in selection['sources'].items()}
    for k, entry in selection['sources'].items():
        require(sha(entry['path']) == entry['sha256'], 'primary drift '+k)
    bor = primary['src/tools/miri/src/borrow_tracker/mod.rs']
    item = primary['src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']
    exact = Path(rows['exact-final-on']['output'])/'component.rs'
    adapter = Path(rows['subset-compiled-on']['output'])/'component.rs'
    full = Path(rows['full-fifth']['output'])/'component.rs'
    markers = ['pub fn get(', 'pub fn inner(', 'pub fn perm(', 'pub fn protected(',
               'pub fn set_permission(', 'fn to_bits(', 'fn from_bits(']
    fidelity = []
    for label, p, extra in [('exact', exact, []), ('adapter', adapter, ['pub fn one(', 'fn default(', 'pub fn tag(']),
                            ('excluded-full', full, ['pub fn new(tag:'])]:
        text = p.read_text()
        require('pub struct BorTag(pub NonZero<u64>);' in text and 'pub struct Item(pub u64);' in text, 'carrier drift')
        require(not re.search(r'\b(assume|external_body|admit|assume_specification)\b', text), 'new trust escape')
        for marker in markers+extra:
            original = bor if marker in ['pub fn get(', 'pub fn inner(', 'pub fn one(', 'fn default('] else item
            old = body(original, original.index(marker))
            new = erased_body(text, marker)
            require(tokens(old) == tokens(new), 'runtime body drift '+label+' '+marker)
            fidelity.append(dict(source=label, marker=marker, token_sha256=hashlib.sha256(json.dumps(tokens(old)).encode()).hexdigest()))
    adapted = erased_body(adapter.read_text(), 'pub fn new(i:')
    old = body(bor, bor.index('pub fn new(i:'))
    require('.map(BorTag)' in old, 'original constructor changed')
    require('ensures r.0 == n' in adapted, 'closure contract missing')
    require(tokens(adapted.replace('|n: NonZero<u64>| -> (r: BorTag) ensures r.0 == n, { BorTag(n) }', 'BorTag')) == tokens(old), 'unrecorded constructor adapter')
    text = exact.read_text()
    manual = dict(proof_bodies=re.findall(r'pub proof fn (\w+)', text),
                  inline_proof_blocks=len(re.findall(r'\bproof\s*\{', text)),
                  bitvector_assertions=text.count('by(bit_vector)'),
                  inverse_axiom_calls=text.count('axiom_view_nonzero_from_primitive_eq('),
                  local_updated_fields_calls=text.count('proof { updated_fields('))
    require(set(manual['proof_bodies']) == set(PROOFS+['tag_identity']), 'manual proof inventory')
    require([manual[k] for k in ['inline_proof_blocks','bitvector_assertions','inverse_axiom_calls','local_updated_fields_calls']] == [2,21,2,1], 'manual count')
    # Every false control must be a pure appended query, preserving ALL base helper bytes.
    prefix_checks = []
    for name, row in rows.items():
        if row['category'] != 'semantic_control':
            continue
        p = Path(row['output'])/'component.rs'
        raw = p.read_text()
        base = text if name.startswith('actual-') else (DEL/'bitfields.rs').read_text()
        require(base.endswith('}\n') and raw.startswith(base[:-2]), 'control helper prefix differs '+name)
        tail = raw[len(base[:-2]):]
        require(len(re.findall(r'pub (?:proof )?fn ', tail)) == 1, 'not singleton appended control')
        prefix_checks.append(name)
    require(len(prefix_checks) == 8, 'eight controls expected')
    return dict(fidelity=fidelity, manual=manual, controls_with_identical_helper_prefix=prefix_checks,
                adapted_constructor_is_exact=False, native_equivalence_proved=False)


def authenticate():
    for p, expected in AUTH.items():
        require(sha(p) == expected, 'receipt/result authentication '+str(p))
    canonical = read(CANON)
    require(canonical['success'] is True, 'canonical not successful')
    for name in TOOLS:
        p = BUNDLE/name
        require(sha(p) == canonical['artifacts'][str(p)]['sha256'], 'tool drift '+name)
    delivery = read(Path('/tmp/native-miri-tag-delivery-wwm_pq9c/receipt.json'))
    rebound = {}
    for label, v in delivery['artifacts_after'].items():
        if label.endswith('/usr/bin/python3.12') or label == '/usr/bin/python3.12':
            p = Path(label)
        else:
            require('/source/tools/' in label, 'unknown delivery key')
            p = REPO/'source/tools'/label.split('/source/tools/', 1)[1]
        require(sha(p) == v['sha256'], 'own delivery identity differs '+str(p))
        rebound[label] = str(p)
    product_dir = Path('/tmp/native-miri-tag-delivery-wwm_pq9c')
    for rel, v in delivery['products'].items():
        require(sha(product_dir/rel) == v['sha256'], 'delivery product drift '+rel)
    result = read(DEL/'RESULTS.json')
    require(len(result['epochs']) == 34, 'retained epoch count')
    retained = []
    for row in result['epochs']:
        out = Path(row['output'])
        require(out == Path('/tmp')/('native-miri-tag-'+next(n for n in read(DEL/'epochs.json') if 'native-miri-tag-'+n == out.name)), 'unnamed retained artifact')
        require(sha(out/'receipt.json') == row['receipt_sha256'], 'retained receipt drift')
        r, f = read(out/'receipt.json'), read(out/'input-freeze.json')
        require(r['all_final_guards_passed'] is True and f['before_first_trial'] is True, 'retained guards')
        require(sha(out/'input-freeze.json') == r['freeze_sha256'] == row['input_freeze_sha256'], 'freeze drift')
        require(sha(out/'component.rs') == r['source_sha256'] == row['source_sha256'], 'source drift')
        require(r['commands'][0]['argv'] == f['argv'] and r['terminal_exit'] == r['commands'][0]['exit_code'], 'retained command/exit')
        for rel, v in r['products'].items():
            p = out/rel
            require(sha(p) == v['sha256'] and p.stat().st_size == v['bytes'], 'retained product drift '+str(p))
        for p in out.glob('*.py'):
            require(sha(p) == f['inputs'][str(p)]['sha256'], 'retained helper copy drift')
        if r.get('verification'):
            data = read(out/'check.stdout')
            require(data['verification-results'] == r['verification'] and data['verus-infer'] == r['inference'], 'retained parsing mismatch')
        retained.append(dict(name=row['name'], category=row['category'], expectation_met=row['expectation_met'],
                             terminal_exit=row['terminal_exit'], product_count=len(r['products']),
                             receipt_sha256=row['receipt_sha256']))
    require(sum(x['product_count'] for x in retained) == 301, 'retained product total')
    return {r['name']:r for r in result['epochs']}, dict(receipt_authentication={str(k):v for k,v in AUTH.items()},
            rebound_delivery_labels=rebound, delivery_products=len(delivery['products']), retained=retained,
            retained_product_count=301)


def check(data, rc, stderr, expected):
    vr, inf = data['verification-results'], data['verus-infer']
    require(vr.get('is-verifying-entire-crate') is True and not vr.get('encountered-vir-error'), 'frontend/partial run')
    require(inf['interface_written'] is False and inf['functions'] == [] and inf['requested_functions'] == inf['generated_functions'] == 0, 'inference/publication')
    require(all(not v for k,v in inf.items() if k.endswith('_assistance') and isinstance(v,list)), 'assistance payload')
    scope = inf['verification_scope']
    require(scope['native_collection_completed'] is True, 'incomplete body inventory')
    selected = [b for b in scope['native_functions'] if b['selected'] and b['body_status'] not in SKIP]
    require(sorted(b['rust_function'] for b in selected) == sorted(expected['bodies']), 'selected body set differs')
    require(not any(b['body_status'] in ['external_specification','unverified_body_with_contract','interface_without_checked_body'] for b in selected), 'unchecked selected contract')
    require(not any(s in stderr.lower() for s in ['panicked at','timed out','resource limit','internal compiler error']), 'compiler/resource failure')
    failure = expected['negative']
    require((vr['verified'],vr['errors']) == tuple(expected['counts']), 'verification count differs')
    if failure is None:
        require(rc == 0 and vr['success'] is True and inf['crate_success'] is True, 'positive proof failed')
        require(scope['complete_selected_scope_verified'] is True and inf['failed_functions'] == [], 'incomplete positive')
        require(all(b['body_status'] in ['checked_exec_body','checked_proof_body'] for b in selected), 'positive body not checked')
        for kind, count in zip(['exec','proof'], expected['body_counts']):
            require(scope['checked_'+kind+'_bodies'] == count == sum(b['body_status']=='checked_'+kind+'_body' for b in selected), 'body count differs')
    else:
        require(rc != 0 and vr['success'] is False and inf['crate_success'] is False, 'control accepted')
        failed = [b['function'] for b in selected if b['rust_function'] == failure]
        require(len(failed) == 1 and inf['failed_functions'] == failed, 'false-query singleton differs')
        require('postcondition not satisfied' in stderr or 'assertion failed' in stderr, 'no semantic diagnostic')
        require(not any(x in stderr for x in ['precondition not satisfied','possible arithmetic','invariant not satisfied','recommendation not met','error[E']), 'unhealthy control helper')
    inventory = [{k:b[k] for k in ['rust_function','selected','body_status','automatically_derived']} for b in scope['native_functions']]
    return dict(verification=vr, failed_functions=inf['failed_functions'], inventory=inventory,
                checked_exec_bodies=scope['checked_exec_bodies'], checked_proof_bodies=scope['checked_proof_bodies'],
                inferred_helpers=[], interface_written=False)


def prepare():
    require(sys.flags.optimize > 0 and sys.dont_write_bytecode, 'run with -B -O')
    rows, auth = authenticate()
    out = Path(tempfile.mkdtemp(prefix='miri-tag-component-review-', dir='/tmp'))
    out.chmod(0o700)
    write(out/'authentication.json', auth)
    write(out/'static.json', static_checks(rows))
    plans = []
    for family, origin, names, counts, body_counts in [
        ('bitfields','bitfields-final-on',PROOFS,[26,0],[0,4]),
        ('exact','exact-final-on',EXACT,[45,0],[10,5]),
        ('adapter','subset-compiled-on',ADAPTER,[49,0],[14,5])]:
        for on in [True,False]:
            plans.append(dict(name=family+('-on' if on else '-off'), origin=origin, bodies=names,
                              counts=counts, body_counts=body_counts, negative=None, on=on, compile=family=='adapter' and on))
    for family, names, count, fnprefix in [('control-',PROOFS,26,'control_'),('actual-control-',EXACT,45,'actual_')]:
        for case in ['nonzero','range','protector','bitfield']:
            fn = fnprefix+case
            plans.append(dict(name=family+case,origin=family+case,bodies=names+[fn],counts=[count,1],negative=fn,on=True,compile=False))
    inputs = list(AUTH) + [BUNDLE/n for n in TOOLS] + list(DEL.iterdir()) + [HERE/'CHECKPOINT.md', Path(__file__),
              REPO/'source/tools/infer_eval_support.py', REPO/'source/tools/infer_experiment.py',
              REPO/'source/tools/research/mutable_buffer_goal/parent-miri-tag-component.json',
              REPO/'source/tools/research/native_miri_policy_pin/pins.json', Path(sys.executable).resolve(),
              Path('/home/dev/.cargo/bin/rustc').resolve(),
              Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc')]
    inputs += [Path(e['path']) for e in read(DEL/'selection.json')['sources'].values()]
    for plan in plans:
        origin = Path(rows[plan['origin']]['output'])/'component.rs'
        d = out/plan['name']
        d.mkdir(mode=0o700)
        src = d/'component.rs'
        src.write_bytes(origin.read_bytes())
        plan.update(output=str(d), source_sha256=sha(src), original_source=str(origin))
        inputs.extend([origin,src])
    (out/'review.py').write_bytes(Path(__file__).read_bytes())
    write(out/'plan.json', plans)
    inputs += [out/'review.py',out/'authentication.json',out/'static.json',out/'plan.json']
    write(out/'initial.json', dict(inputs=identities(inputs), before_first_epoch=True,
                                 controller_argv=[sys.executable,'-B','-O',str(out/'review.py'),'run',str(out)]))
    print(out, flush=True)


def run(root):
    require(sys.flags.optimize > 0 and sys.dont_write_bytecode, 'run with -B -O')
    initial = read(root/'initial.json')
    frozen = initial['inputs']
    frozen.update(identities([root/'initial.json']))
    summaries = []
    for plan in read(root/'plan.json'):
        d = Path(plan['output'])
        require(not (d/'receipt.json').exists(), 'epoch already completed')
        env = dict(PATH='/home/dev/.cargo/bin:/usr/bin:/bin',HOME=str(d/'home'),RUSTUP_HOME='/home/dev/.rustup',
                   RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(BUNDLE/'z3'),VERUS_INFER_CACHE=str(d/'cache'),
                   TMPDIR=str(d),XDG_CACHE_HOME=str(d/'cache'),CARGO_TARGET_DIR=str(d/'target'))
        (d/'home').mkdir(mode=0o700)
        argv = [str(BUNDLE/'verus'),str(d/'component.rs'),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20']
        if plan['on']:
            argv += ['--infer-goals']
        if plan['compile']:
            argv += ['--compile','-o',str(d/'checked.rlib')]
        guard(frozen)
        write(d/'input-freeze.json',dict(inputs=frozen,argv=argv,environment=env,cwd=str(d),expectation=plan,before_first_trial=True))
        epoch = identities([d/'input-freeze.json'])
        receipt = dict(argv=argv,environment=env,cwd=str(d),source_sha256=sha(d/'component.rs'),
                       freeze_sha256=sha(d/'input-freeze.json'),success=False,all_final_guards_passed=False)
        started = time.monotonic()
        try:
            with (d/'check.stdout').open('wb') as stdout, (d/'check.stderr').open('wb') as stderr:
                result = subprocess.run(argv,cwd=d,env=env,stdout=stdout,stderr=stderr,timeout=120)
            receipt['terminal_exit'] = result.returncode
            receipt['seconds'] = time.monotonic()-started
            receipt.update(check(read(d/'check.stdout'),result.returncode,(d/'check.stderr').read_text(),plan))
            if plan['compile']:
                require((d/'checked.rlib').is_file(), 'missing compiled adapter product')
            receipt['success'] = True
        except subprocess.TimeoutExpired as error:
            receipt.update(classification='resource_timeout_not_semantic',error=str(error))
        except Exception as error:
            receipt.update(classification='expectation_or_proof_failure_not_accepted',error=str(error))
        finally:
            guard(frozen)
            guard(epoch)
            receipt['all_final_guards_passed'] = True
            receipt['products'] = identities([p for p in d.rglob('*') if p.is_file() and p.name!='receipt.json'])
            write(d/'receipt.json',receipt)
        summary = dict(name=plan['name'],output=str(d),receipt_sha256=sha(d/'receipt.json'),
                       input_freeze_sha256=receipt['freeze_sha256'],source_sha256=receipt['source_sha256'],
                       argv=argv,terminal_exit=receipt.get('terminal_exit'),success=receipt['success'],
                       verification=receipt.get('verification'),failed_functions=receipt.get('failed_functions'),
                       all_final_guards_passed=True,product_manifest_sha256=hashlib.sha256(json.dumps(receipt['products'],sort_keys=True).encode()).hexdigest())
        summaries.append(summary)
        write(root/'progress.json',summaries)
        print(json.dumps({k:summary[k] for k in ['name','success','terminal_exit','verification','failed_functions']}),flush=True)
    guard(frozen)
    write(root/'summary.json',dict(epochs=summaries,all_final_guards_passed=True,
                                  initial_sha256=sha(root/'initial.json'),controller_sha256=sha(root/'review.py'),
                                  authentication_sha256=sha(root/'authentication.json'),static_sha256=sha(root/'static.json'),
                                  success=all(s['success'] for s in summaries),native_correspondences=0,
                                  publication=False,goal_complete=False))
    require(all(s['success'] for s in summaries), 'one or more fresh outcomes failed expectation')


if __name__ == '__main__':
    if sys.argv[1:] == ['prepare']:
        prepare()
    elif len(sys.argv) == 3 and sys.argv[1] == 'run':
        run(Path(sys.argv[2]))
    else:
        raise ValueError('usage: python3 -B -O review.py prepare | run PRIVATE_ROOT')
