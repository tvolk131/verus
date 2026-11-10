"""Private exact-command epochs. Explicit gates remain valid under python -O."""
import hashlib,json,os,subprocess,sys,tempfile,time
from pathlib import Path
from source import HERE,ROOT,PIN,need,sha,generate,fidelity
B=Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined69-bundle-3aa7f49a')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
RECEIPT='2e3fcc58f6fca9364061329104710937ec2aac6aab95a5c48d068c1ca6ee9bb7'
NAMES={'verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib'}

def write(p,d):p.write_text(json.dumps(d,indent=2)+'\n')
def authenticate():
    need(sha(B/'receipt.json')==RECEIPT,'bundle receipt');r=json.loads((B/'receipt.json').read_text())
    need(r['success'] is True and r['all_final_guards_passed'] is True and r['matching_vstd_verified']==2250,'bundle acceptance')
    need(r['schema_version']==38 and r['policy']=='candidate-bound-evidence-v69' and r['cache'].startswith('verus-infer-v72:'),'versions')
    need(set(r['tools'])==NAMES,'eight tools')
    for n,h in r['tools'].items():need(sha(B/n)==h,'tool drift '+n)
    return r

def inputs(extra):
    r=authenticate();paths=[B/n for n in NAMES]+[B/'receipt.json',TC/'bin/rustc',Path('/home/dev/.cargo/bin/rustup').resolve(),Path(sys.executable).resolve(),PIN]
    paths+=list((TC/'lib').glob('librustc_driver*'))+list((TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib').glob('*.rlib'))
    paths+=[TC/'lib/rustlib/src/rust/library/core/src/convert/num.rs']
    pins=json.loads(PIN.read_text())
    paths += [Path(pins['sources'][n]['path']) for n in ['src/tools/miri/src/borrow_tracker/mod.rs','src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']]
    paths += list(HERE.glob('*.py'))+list(HERE.glob('*.md'))+list(HERE.glob('*.rs'))
    tracked=subprocess.check_output(['git','ls-files','-z','source','dependencies','rust-toolchain.toml','Cargo.lock'],cwd=ROOT).decode().split('\0')
    paths += [ROOT/p for p in tracked if p and (ROOT/p).is_file()]
    paths += list(map(Path,extra)); return {str(p):sha(p) for p in sorted(set(paths))}

def run(label,argv=None,extras=(),envextra=None,kind=None,infer=True,planextra=None,products=()):
    authenticate();d=Path(tempfile.mkdtemp(prefix='item-tag-'+label+'-',dir='/tmp'));d.chmod(0o700)
    plan=dict(label=label,kind=kind,eq_required=kind is not None,inference_enabled=infer,manual_proof_blocks=0,publication=False)
    if kind:
        src=d/'component.rs';src.write_text(generate(kind));plan['fidelity']=fidelity(src.read_text());extras=[*extras,src]
        argv=[str(B/'verus'),str(src),'--crate-type=lib','--crate-name=component','--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',str(d/'vir'),'--compile','-o',str(d/'libcomponent.rlib'),'--export',str(d/'component.vir')]
        if infer:argv+=['--infer-goals']
        plan['expected_native_checked']= ['component::BorTag::clone','component::BorTag::eq','component::Item::clone','component::BorTag::new','component::BorTag::get','component::Item::tag','component::roundtrip','component::compare_equal','component::lookup','component::boundaries']
        if kind=='false':plan['expected_native_checked']+=['component::false_relation']
        if kind=='invalid':plan['expected_native_checked']+=['component::invalid_zero_low']
        if kind.startswith('full'):plan['expected_native_checked']+=['component::Item::new','component::Item::perm','component::Item::protected','component::Permission::clone','component::Permission::to_bits','component::Permission::from_bits']
        if kind=='full-helpers':plan['expected_native_checked'].remove('component::Item::new')
        if kind=='full-help' or kind=='full-helpers':plan['expected_native_checked']+=['component::packed_fields_raw']
        plan['required_generated_bodies']=['component::BorTag::ctor%','component::Item::ctor%']
    if kind:
        identity_file=HERE/'identities.json'
        if identity_file.is_file():
            ids=json.loads(identity_file.read_text());plan['frozen_expected_identity']=ids.get(kind)
    if planextra:plan.update(planextra)
    env={**os.environ,'PATH':'/home/dev/.cargo/bin:/usr/bin:/bin','RUSTUP_TOOLCHAIN':'1.99.1','RUSTC_BOOTSTRAP':'1','LD_LIBRARY_PATH':str(TC/'lib'),'VERUS_ROOT':str(B),'VERUS_Z3_PATH':str(B/'z3'),'TMPDIR':str(d),'VERUS_INFER_CACHE':str(d/'cache'),'XDG_CACHE_HOME':str(d/'cache'),'CARGO_TARGET_DIR':str(d/'target')}
    env.update(envextra or {})
    argv=[str(a).replace('__EPOCH__',str(d)) for a in argv]
    if kind:
        plan['manual_proof_blocks']=src.read_text().count('proof {')
        plan['manual_proof_functions']=__import__('re').findall(r'proof fn (\w+)',src.read_text())
    write(d/'plan.json',plan);frozen=inputs([*extras,d/'plan.json']);record=dict(argv=list(map(str,argv)),cwd=str(d),environment={k:env[k] for k in ['PATH','RUSTUP_TOOLCHAIN','RUSTC_BOOTSTRAP','LD_LIBRARY_PATH','VERUS_ROOT','VERUS_Z3_PATH','TMPDIR','VERUS_INFER_CACHE','XDG_CACHE_HOME','CARGO_TARGET_DIR']},inputs=frozen,plan=plan)
    write(d/'freeze.json',record);fh=sha(d/'freeze.json');start=time.monotonic()
    for p in HERE.iterdir():
        if p.is_file() and p.suffix in ['.py','.md','.rs','.json']:(d/('controller-'+p.name)).write_bytes(p.read_bytes())
    rc=None
    try:
        with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
            p=subprocess.run(record['argv'],cwd=d,env=env,stdout=out,stderr=err,timeout=180);rc=p.returncode
        record.update(exit=rc,seconds=time.monotonic()-start)
    except subprocess.TimeoutExpired:
        record.update(exit=None,seconds=time.monotonic()-start,classification='resource_timeout')
    finally:
        record['final_guards_passed']=all(sha(p)==h for p,h in frozen.items()) and sha(d/'freeze.json')==fh
        record['external_products']={str(p):sha(p) for p in products if Path(p).is_file()}
        for idx,p in enumerate(products):
            if Path(p).is_file():(d/('retained-product-'+str(idx))).write_bytes(Path(p).read_bytes())
        record['products']={str(p.relative_to(d)):sha(p) for p in d.rglob('*') if p.is_file()}
        write(d/'receipt.json',record)
        with (HERE/'attempts.jsonl').open('a') as f:f.write(json.dumps(dict(label=label,path=str(d),receipt_sha256=sha(d/'receipt.json'),exit=rc,guards=record['final_guards_passed']))+'\n')
    need(record['final_guards_passed'],'epoch guard failed')
    if plan.get('frozen_expected_identity'):
        from inventory import signature
        actual=signature(d);write(d/'observed-identity.json',actual)
        need(actual==plan['frozen_expected_identity'],'frozen body/helper/details/native identity differs')
    print(json.dumps(dict(path=str(d),exit=rc)))
    return d
if __name__=='__main__':run(sys.argv[1],kind=sys.argv[2],infer='--off' not in sys.argv)
