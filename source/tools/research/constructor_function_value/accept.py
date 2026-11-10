"""Fresh local acceptance, not independent review or native-runtime permission."""
import json, pathlib
import epoch

BUNDLE = pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
RECEIPT = pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
TC = pathlib.Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
TARGET = epoch.ROOT/'source/target-verus/release'
HERE = pathlib.Path(__file__).resolve().parent

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def main():
    require(epoch.digest(RECEIPT) == '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a', 'canonical receipt hash')
    canonical = json.loads(RECEIPT.read_text())
    require(canonical['success'] is True, 'canonical success')
    baseline = json.loads((HERE/'baseline.json').read_text())
    require(len(baseline['bundle']) == 8, 'bundle cardinality')
    for name, h in baseline['bundle'].items():
        require(epoch.digest(BUNDLE/name) == h == canonical['artifacts'][str(BUNDLE/name)]['sha256'] == canonical['artifacts_after'][str(BUNDLE/name)]['sha256'], 'bundle identity '+name)
    for path, h in baseline['rebound_engine_sources'].items():
        require(epoch.digest(epoch.ROOT/path) == h, 'unchanged inference engine '+path)
    pins = json.loads((epoch.ROOT/'source/tools/research/native_miri_policy_pin/pins.json').read_text())
    native_paths = []
    for entry in pins['sources'].values():
        require(epoch.digest(entry['path']) == entry['sha256'], 'native source pin')
        native_paths.append(pathlib.Path(entry['path']))
    native = pathlib.Path(pins['sources']['src/tools/miri/src/borrow_tracker/mod.rs']['path']).read_bytes()
    source = (HERE/'bortag.rs').read_bytes()
    def body(data, anchor):
        start = data.index(b'{', data.index(anchor))
        return data[start:data.index(b'}', start)+1]
    native_body = body(native, b'pub fn new(i: u64) -> Option<Self>')
    require(body(source, b'fn new(i: u64)') == native_body, 'literal original native new body')
    tools = [TARGET/n for n in ['rust_verify','verus','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib']]
    extra = tools + [TC/'bin/rustc', *TC.glob('lib/librustc_driver*'), BUNDLE/'z3', RECEIPT, *native_paths]
    env = {'PATH':'/home/dev/.cargo/bin:/usr/bin:/bin', 'RUSTUP_TOOLCHAIN':'1.99.1',
           'LD_LIBRARY_PATH':str(TC/'lib'), 'VERUS_ROOT':str(TARGET), 'CARGO_BUILD_JOBS':'2',
           'VERUS_Z3_PATH':str(BUNDLE/'z3')}
    result = {'native_new_body_sha256':__import__('hashlib').sha256(native_body).hexdigest(),
              'native_mod_sha256':pins['sources']['src/tools/miri/src/borrow_tracker/mod.rs']['sha256'],
              'native_pin_count':len(native_paths), 'proofs':{}}
    for label, flags in [('final-off', []), ('final-on', ['--infer-goals'])]:
        record = epoch.run(label, [str(TARGET/'verus'), str(HERE/'bortag.rs'), '--crate-type=lib', '--no-cheating', '--output-json', '--multiple-errors', '20']+flags,
                           extra=extra, env={**env, 'VERUS_INFER_CACHE':str(epoch.PRIVATE/(label+'-cache'))})
        require(record['exit'] == 0, label+' process')
        out = json.loads((epoch.PRIVATE/label/'stdout').read_text())
        verification = out['verification-results']; infer = out['verus-infer']
        require(verification['success'] is True and verification['verified'] == 2 and verification['errors'] == 0 and verification['is-verifying-entire-crate'] is True, label+' proof')
        require(infer['generated_functions'] == 0 and infer['functions'] == [], label+' inference gain')
        assists = {k:v for k,v in infer.items() if k.endswith('_assistance')}
        require(all(v == [] for v in assists.values()), label+' helpers')
        result['proofs'][label] = {'verification':verification, 'status':infer['status'], 'generated_functions':0,
            'inference_functions':infer['functions'], 'assistance':assists,
            'trusted_models_sha256':__import__('hashlib').sha256(json.dumps(infer['trusted_models'], sort_keys=True).encode()).hexdigest(),
            'func_details':out['func-details'], 'receipt_sha256':epoch.digest(epoch.PRIVATE/label/'receipt.json')}
    result['tools'] = {str(p):epoch.digest(p) for p in tools}
    result['fixture_sha256'] = epoch.digest(HERE/'bortag.rs')
    result['compiler_commit'] = pins['compiler_commit']
    result['independent_review'] = 'pending_parent'
    result['publication'] = False
    result['goal_complete'] = False
    (epoch.PRIVATE/'acceptance.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__ == '__main__':
    main()
