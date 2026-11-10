"""Exact direct verifier receipts for completed focused test sources."""
import json, pathlib, shutil
import epoch
import accept

CASES = {
    'original_nonzero': ('proved', None),
    'generic_move_lifetime_variant_and_order': ('proved', None),
    'higher_order_checked': ('proved', None),
    'payload_get_roundtrip': ('proved', None),
    'self_public_bound_const_generic_and_zero_arity': ('proved', None),
    'higher_order_requires': ('proof_rejection', 'fails to satisfy `callee.requires(args)`'),
    'wrong_variant_field_order_and_false_post': ('proof_rejection', 'postcondition not satisfied'),
    'fn_pointer_rejected': ('frontend_unsupported', 'casting a pointer'),
    'invariant_rejected': ('frontend_unsupported', 'constructor function values require'),
    'mixed_mode_rejected': ('frontend_unsupported', 'constructor function values require'),
    'lifetime_checked': ('rust_lifetime_rejection', 'error[E0515]'),
    'move_checked': ('rust_move_rejection', 'error[E0382]'),
    'visibility_checked': ('rust_visibility_rejection', 'error[E0603]'),
}

def main():
    source_dir = epoch.PRIVATE/'direct-sources-2'
    source_dir.mkdir(exist_ok=True)
    inputs = accept.TARGET/'test_inputs'
    sources = {}
    for name in CASES:
        found = list(inputs.glob('constructor_function_values-*-'+name+'/test.rs'))
        accept.require(len(found) == 1, 'completed source selection '+name)
        source = source_dir/(name+'.rs')
        if source.exists():
            accept.require(epoch.digest(source) == epoch.digest(found[0]), 'unchanged completed source '+name)
        else:
            shutil.copyfile(found[0], source)
        sources[name] = source
    tools = [accept.TARGET/n for n in ['rust_verify','verus','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib']]
    extra = tools + [accept.TC/'bin/rustc', *accept.TC.glob('lib/librustc_driver*'), accept.BUNDLE/'z3']
    report = {}
    for name, (classification, expected) in CASES.items():
        source = sources[name]
        label = 'direct2-'+name
        receipt = epoch.PRIVATE/label/'receipt.json'
        if receipt.is_file():
            r = json.loads(receipt.read_text())
            accept.require(r['final_guards'] is True, name+' recorded guard')
            for item in extra+[source]:
                accept.require(epoch.digest(item) == r['frozen'][str(item)], name+' recorded input')
            accept.require(epoch.digest(epoch.PRIVATE/label/'stdout') == r['stdout_sha256'] and
                           epoch.digest(epoch.PRIVATE/label/'stderr') == r['stderr_sha256'], name+' recorded logs')
        else:
            label = 'direct3-'+name
            r = epoch.run(label, [str(accept.TARGET/'verus'), str(source), '--crate-type=lib', '--no-cheating', '--output-json', '--multiple-errors', '20'],
                      extra=extra+[source], env={'PATH':'/home/dev/.cargo/bin:/usr/bin:/bin', 'RUSTUP_TOOLCHAIN':'1.99.1',
                      'LD_LIBRARY_PATH':str(accept.TC/'lib'), 'VERUS_ROOT':str(accept.TARGET),
                      'VERUS_Z3_PATH':str(accept.BUNDLE/'z3'), 'VERUS_INFER_CACHE':str(epoch.PRIVATE/(label+'-cache'))})
        stderr = (epoch.PRIVATE/label/'stderr').read_text()
        accept.require('panicked at' not in stderr and 'internal compiler error' not in stderr, name+' panic')
        stdout = (epoch.PRIVATE/label/'stdout').read_text()
        data = json.loads(stdout) if stdout.strip() else {}
        v = data.get('verification-results')
        if classification == 'proved':
            accept.require(r['exit'] == 0 and v and v['success'] and v['errors'] == 0 and v['is-verifying-entire-crate'], name+' proof')
        else:
            accept.require(r['exit'] == 1 and expected in stderr, name+' expected diagnostic')
            if classification == 'proof_rejection':
                accept.require(v and not v['encountered-vir-error'] and v['errors'] == (4 if name.startswith('wrong_') else 1), name+' semantic rejection')
        report[name] = {'classification':classification, 'source_sha256':epoch.digest(source), 'exit':r['exit'],
                        'verification':v, 'receipt_sha256':epoch.digest(epoch.PRIVATE/label/'receipt.json')}
    (epoch.PRIVATE/'direct-cases.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__ == '__main__':
    main()
