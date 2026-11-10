"""Bounded private epochs. No imports of historical writing controllers."""
import hashlib, importlib.util, json, os, re, subprocess, sys, tempfile, time
from pathlib import Path
from source import HERE, PREVIOUS, original, need, sha, provider, client, identity
ROOT=HERE.parents[3]
B=Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined69-bundle-3aa7f49a')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
RECEIPT='2e3fcc58f6fca9364061329104710937ec2aac6aab95a5c48d068c1ca6ee9bb7'
NAMES={'verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib'}
spec=importlib.util.spec_from_file_location('pure_tag_inventory',PREVIOUS/'inventory.py')
inv=importlib.util.module_from_spec(spec);spec.loader.exec_module(inv)

def write(p,data): p.write_text(json.dumps(data,indent=2)+'\n')
def authenticate():
    need(sha(B/'receipt.json')==RECEIPT,'bundle receipt identity')
    r=json.loads((B/'receipt.json').read_text())
    need(r['success'] is True and r['all_final_guards_passed'] is True and r['matching_vstd_verified']==2250,'accepted bundle')
    need(r['schema_version']==38 and r['policy']=='candidate-bound-evidence-v69' and r['cache'].startswith('verus-infer-v72:'),'versions')
    need(set(r['tools'])==NAMES,'all eight tools')
    for n,h in r['tools'].items():need(sha(B/n)==h,'tool '+n)
    original.load()
    return r

def signature(d, role):
    i=inv.inventory(d, 'component' if role=='provider' else 'client')
    return dict(ordinary={k:{f:v[f] for f in ['mode','body_present','normalized_body_sha256']} for k,v in i['ordinary_original_functions'].items()},function_details=sorted(i['local_function_details']),native_functions=[{k:v for k,v in b.items() if k not in ['source','body_status']} for b in i['native_scope']['native_functions']],helpers=[{k:h[k] for k in ['function','mode','summary_kind','parameters','result_type','contract']} for h in i['helpers']])

def baseline():
    """Freeze completed baseline source/tools/failure before experiments; no rerun."""
    authenticate();r=json.loads((PREVIOUS/'RESULTS.json').read_text())
    labels=['tag-provider-final','ordinary-import-final','lookup-import']
    out={}
    for label in labels:
        row=next(x for x in r['attempt_ledger'] if x['label']==label);d=Path(row['path'])
        need(sha(d/'receipt.json')==row['receipt_sha256'],'baseline receipt '+label)
        receipt=json.loads((d/'receipt.json').read_text());need(receipt['final_guards_passed'],'baseline guards')
        for n,h in receipt['products'].items():need(sha(d/n)==h,'baseline product '+label+' '+n)
        out[label]=dict(path=str(d),receipt_sha256=row['receipt_sha256'],verification=json.loads((d/'stdout').read_text())['verification-results'],stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),classification=row['classification'],inventory=inv.inventory(d,'client' if 'import' in label else 'component'))
    need(sha(Path(out[labels[0]]['path'])/'component.rs')==identity(provider('closed'),True)['source_sha256'],'exact baseline provider source')
    parent=Path('/tmp/verus-infer-buffer-cycle-20261109/item-tag-parent-v2/receipt.json')
    need(sha(parent)=='ba75225dcf12c21a8f7c17fcfdef493d1083aa3ed2c865d74740b0a66de476aa','independent parent replay')
    write(HERE/'BASELINE.json',dict(bundle_receipt_sha256=RECEIPT,results_sha256=sha(PREVIOUS/'RESULTS.json'),independent_parent_receipt_sha256=sha(parent),observations=out,source_identity=identity(provider('closed'),True)))
    print('baseline frozen: provider19/0, imported Eq/value1/1, lookup1/0')

def run(label,role,mode,dependency=None,off=False,expected=None):
    bundle=authenticate()
    rows=[json.loads(x) for x in (HERE/'attempts.jsonl').read_text().splitlines()] if (HERE/'attempts.jsonl').exists() else []
    need(label not in {r['label'] for r in rows},'fresh label')
    need((HERE/'BASELINE.json').is_file(),'baseline frozen before changes')
    d=Path(tempfile.mkdtemp(prefix='tag-observation-'+label+'-',dir='/tmp'));d.chmod(0o700)
    is_provider=role=='provider';need(is_provider or role=='client','role')
    text=provider(mode) if is_provider else client(mode)
    src=d/('component.rs' if is_provider else 'client.rs');src.write_text(text)
    base=json.loads((HERE/'BASELINE.json').read_text())['observations']['tag-provider-final']['inventory']
    p=dict(label=label,role=role,adapter=mode,source_identity=identity(text,is_provider),optional_goals=not off,
           inference_explicit_for_provider=True,publication=False,expected_identity=expected,
           expected_provider_native=[{k:v for k,v in b.items() if k not in ['source','body_status']} for b in base['native_scope']['native_functions']] if is_provider else None,
           expected_helpers=[{k:h[k] for k in ['function','mode','summary_kind','parameters','result_type','contract']} for h in base['helpers']] if is_provider else [],
           expected_provider_ordinary_names=sorted(base['ordinary_original_functions']) if is_provider else None,
           expected_provider_function_detail_names=sorted(base['local_function_details']) if is_provider else None,
           expected_client_names=sorted(re.findall(r'pub fn (\w+)\(',text)) if not is_provider else None)
    if is_provider and mode=='compat':
        p['expected_provider_native'].append(dict(function='component::eq_value_compatibility',rust_function='eq_value_compatibility',automatically_derived=False,selected=True,inference_candidate=False,ordinary_contract=True))
        p['expected_provider_ordinary_names'].append('component::eq_value_compatibility');p['expected_provider_ordinary_names'].sort()
        p['expected_provider_function_detail_names'].append('component::eq_value_compatibility');p['expected_provider_function_detail_names'].sort()
    argv=[B/'verus',src,'--crate-name='+('component' if is_provider else 'client'),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir']
    extra=[]
    if is_provider:
        argv+=['--compile','-o',d/'libcomponent.rlib','--export',d/'component.vir']
        if not off:argv+=['--infer-goals']
    else:
        dep=Path(dependency);need(dep.is_dir(),'explicit completed provider')
        rr=json.loads((dep/'receipt.json').read_text());need(rr['exit']==0 and rr['final_guards_passed'],'healthy provider')
        for n in ['libcomponent.rlib','component.vir']:
            need(sha(dep/n)==rr['products'][n],'provider product')
        argv+=['--extern','component='+str(dep/'libcomponent.rlib'),'--import','component='+str(dep/'component.vir'),'-L',dep]
        extra=[dep/'receipt.json',dep/'libcomponent.rlib',dep/'component.vir']
        p['provider']=dict(path=str(dep),receipt_sha256=sha(dep/'receipt.json'),products={n:sha(dep/n) for n in ['libcomponent.rlib','component.vir']})
    paths=[src,B/'receipt.json',*[B/n for n in NAMES],original.PIN,PREVIOUS/'source.py',PREVIOUS/'inventory.py',HERE/'BASELINE.json',Path(sys.executable).resolve(),TC/'bin/rustc',Path('/home/dev/.cargo/bin/rustup').resolve(),*extra]
    pins=json.loads(original.PIN.read_text());paths += [Path(pins['sources'][n]['path']) for n in ['src/tools/miri/src/borrow_tracker/mod.rs','src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']]
    paths+=list((TC/'lib').glob('librustc_driver*'))+list((TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib').glob('*.rlib'))
    for f in HERE.iterdir():
        if f.suffix in ['.py','.md']:
            copy=d/('controller-'+f.name);copy.write_bytes(f.read_bytes());paths.extend([f,copy])
    write(d/'plan.json',p);paths.append(d/'plan.json')
    env={**os.environ,'PATH':'/home/dev/.cargo/bin:/usr/bin:/bin','RUSTUP_TOOLCHAIN':'1.99.1','RUSTC_BOOTSTRAP':'1','LD_LIBRARY_PATH':str(TC/'lib'),'VERUS_ROOT':str(B),'VERUS_Z3_PATH':str(B/'z3'),'TMPDIR':str(d),'VERUS_INFER_CACHE':str(d/'cache'),'XDG_CACHE_HOME':str(d/'cache'),'CARGO_TARGET_DIR':str(d/'target'),'PYTHONDONTWRITEBYTECODE':'1'}
    frozen={str(f):sha(f) for f in sorted(set(paths))}
    record=dict(argv=list(map(str,argv)),cwd=str(d),environment={k:env[k] for k in ['PATH','RUSTUP_TOOLCHAIN','RUSTC_BOOTSTRAP','LD_LIBRARY_PATH','VERUS_ROOT','VERUS_Z3_PATH','TMPDIR','VERUS_INFER_CACHE','XDG_CACHE_HOME','CARGO_TARGET_DIR','PYTHONDONTWRITEBYTECODE']},inputs=frozen,plan=p)
    write(d/'freeze.json',record);fh=sha(d/'freeze.json');start=time.monotonic();rc=None
    try:
        with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
            rc=subprocess.run(record['argv'],cwd=d,env=env,stdout=out,stderr=err,timeout=180).returncode
        record['exit']=rc
    except subprocess.TimeoutExpired:record.update(exit=None,classification='resource_timeout')
    finally:
        record['seconds']=time.monotonic()-start
        record['final_guards_passed']=all(sha(f)==h for f,h in frozen.items()) and sha(d/'freeze.json')==fh
        record['products']={str(f.relative_to(d)):sha(f) for f in d.rglob('*') if f.is_file()}
        write(d/'receipt.json',record)
        row=dict(label=label,path=str(d),exit=rc,guards=record['final_guards_passed'],receipt_sha256=sha(d/'receipt.json'))
        with (HERE/'attempts.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
    need(record['final_guards_passed'],'final input guard')
    print(json.dumps(row),flush=True)
    if expected:need(signature(d,role) == expected,'exact frozen VIR/helper/native/detail identity')
    return d

if __name__=='__main__':
    if sys.argv[1]=='baseline':baseline()
    else:
        import argparse
        a=argparse.ArgumentParser();a.add_argument('label');a.add_argument('role');a.add_argument('mode');a.add_argument('--provider');a.add_argument('--off',action='store_true');a.add_argument('--identity')
        args=a.parse_args();expected=json.loads(Path(args.identity).read_text()) if args.identity else None
        run(args.label,args.role,args.mode,args.provider,args.off,expected)
