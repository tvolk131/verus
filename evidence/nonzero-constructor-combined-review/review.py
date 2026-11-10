"""Private independent combined replay. Explicit acceptance checks survive python -O."""
import hashlib, json, os, pathlib, shutil, subprocess, sys, tempfile, time, tomllib
sys.dont_write_bytecode = True
P = pathlib.Path
HERE = P(__file__).resolve().parent
ROOT = HERE.parents[1]
TC = P('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
OLD = P('/tmp/constructor-function-value-review-t3-755cb0d1')
BASE = P('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
CANON = P('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
UPSTREAM = P('/tmp/native-miri-policy-pin-20261110/sources/src/tools/miri/src/borrow_tracker/mod.rs')
NAMES = ['verus','rust_verify','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib','z3']
def need(c, m):
    if not c: raise RuntimeError(m)
def sha(p): return hashlib.file_digest(P(p).open('rb'),'sha256').hexdigest()
def read(p): return json.loads(P(p).read_text())
def write(p, x): P(p).write_text(json.dumps(x,indent=2)+'\n')
def source_files():
    ns = subprocess.check_output(['git','ls-files','source','dependencies','rust-toolchain.toml'],cwd=ROOT,text=True).splitlines()
    return [ROOT/n for n in ns if (ROOT/n).is_file()] + [p for p in (ROOT/'dependencies').rglob('*') if p.is_file() and '.git' not in p.parts]
def hashes(ps): return {str(p):sha(p) for p in sorted(set(ps))}
def guard(before): need(all(P(p).is_file() and sha(p)==h for p,h in before.items()),'immutable input drift')
def private(prefix):
    p=P(tempfile.mkdtemp(prefix='combined-review-'+prefix+'-',dir='/tmp'));p.chmod(0o700);return p
def epoch(work,label,argv,*,extra=(),envextra=None,cwd=None,timeout=2400,products=()):
    out=private(label);print(str(out),flush=True)
    inputs=source_files()+list(HERE.glob('*.py'))+[TC/'bin/rustc',TC/'bin/cargo',P(sys.executable).resolve(),BASE/'z3']+list(extra)
    env=dict(os.environ,PATH=str(TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_TOOLCHAIN='1.99.1',CARGO_HOME=str(work/'cargo-home'),CARGO_TARGET_DIR=str(work/'target'),CARGO_BUILD_BUILD_DIR=str(work/'target'),CARGO_BUILD_JOBS='2',LD_LIBRARY_PATH=str(TC/'lib'),VERUS_IN_VARGO='1',VERUS_Z3_PATH=str(BASE/'z3'),VERUS_INFER_CACHE=str(out/'cache'),XDG_CACHE_HOME=str(out/'cache'),TMPDIR=str(out))
    env.update(envextra or {})
    freeze=dict(inputs=hashes(inputs),argv=list(map(str,argv)),cwd=str(cwd or ROOT/'source'),environment={k:v for k,v in env.items() if k.startswith(('CARGO_','VERUS_','RUSTUP_','LD_LIBRARY','TMPDIR','XDG_','PATH'))},before_first_trial=True)
    write(out/'freeze.json',freeze);fg=sha(out/'freeze.json')
    r=dict(label=label,commands=[],success=False,classification='setup_failure',freeze_sha256=fg,publication=False,goal_complete=False)
    try:
        guard(freeze['inputs']);start=time.monotonic()
        try:
            with (out/'stdout').open('w') as stdout,(out/'stderr').open('w') as stderr:
                result=subprocess.run(list(map(str,argv)),cwd=freeze['cwd'],env=env,stdout=stdout,stderr=stderr,timeout=timeout)
            r['exit']=result.returncode
        except subprocess.TimeoutExpired:
            r['classification']='resource_failure';raise
        finally:
            r['commands'].append(dict(argv=freeze['argv'],cwd=freeze['cwd'],exit=r.get('exit'),seconds=time.monotonic()-start))
        err=(out/'stderr').read_text()
        need(not any(x in err.lower() for x in ['panicked at','internal compiler error','resource limit','timed out']),'compiler/resource failure')
        r['classification']='command_completed'
        r['success']=True
    except BaseException as e:
        r['failure']=dict(type=type(e).__name__,message=str(e));raise
    finally:
        guard(freeze['inputs']);need(sha(out/'freeze.json')==fg,'freeze drift');r['final_guards_passed']=True
        r['products']=hashes([p for p in out.rglob('*') if p.is_file() and p.name!='receipt.json']+[p for p in products if p.is_file()])
        write(out/'receipt.json',r)
        with (work/'epochs.jsonl').open('a') as f:f.write(json.dumps(dict(label=label,path=str(out),receipt_sha256=sha(out/'receipt.json')))+'\n')
    return out,r

def authenticate():
    need(sha(CANON)=='2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a','canonical receipt drift')
    a=read(CANON);need(a['success'] is True,'canonical failure')
    for n in NAMES:need(sha(BASE/n)==a['artifacts'][str(BASE/n)]['sha256'],'baseline tool drift')
    need(sha(UPSTREAM)=='8dec93f2a161caff23bbae3d52ee95eaf690d5d921d46847caf80ceb5d29a896','native source drift')
    need(sha('/tmp/constructor-parent-review-fix-v2-wzhxl8d2/receipt.json')=='fadf9d4aa1d7d53df6a54cf8bf0773d31dee21b35822f136a36fabbb23305388','constructor fix receipt drift')
    rows=[]
    for e in read(ROOT/'source/tools/research/nonzero_derived_comparisons/epochs.json'):
        p=P(e['output']);need(sha(p/'receipt.json')==e['receipt_sha256'],'delivered receipt drift')
        r=read(p/'receipt.json')
        if 'freeze_sha256' in r:need(sha(p/'input-freeze.json')==r['freeze_sha256'],'delivered freeze drift')
        count=0
        for n,h in r.get('products',{}).items():
            if P(n).is_absolute() or n.startswith(('target/','retained_inputs/')):continue
            need(sha(p/n)==h,'delivered relative product drift');count+=1
        if 'tools' in r:
            for n,h in r['tools'].items():need(sha(p/n)==h,'delivered frozen tool drift')
        rows.append(dict(path=str(p),receipt_sha256=e['receipt_sha256'],relative_products=count,success=r['success']))
    return rows

def setup():
    w=P(read(HERE/'checkpoint.json')['work']) if (HERE/'checkpoint.json').exists() else private('work');print('WORK='+str(w),flush=True)
    write(HERE/'checkpoint.json',dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),type='independent_review_checkpoint',source_hashes={str(p.relative_to(ROOT)):sha(p) for p in source_files()},delivered=authenticate(),work=str(w),policy=69,cache=72,schema=38,publication=False,goal_complete=False))
    home=w/'cargo-home';(home/'registry/cache').mkdir(parents=True,exist_ok=True)
    # Only archive names and checksums in own lock, plus immutable Git object databases.
    lock=tomllib.loads((ROOT/'source/Cargo.lock').read_text());copied={}
    for pkg in lock['package']:
        if not pkg.get('source','').startswith('registry+'):continue
        name=f"{pkg['name']}-{pkg['version']}.crate"
        matches=list((OLD/'cargo-home/registry/cache').glob('*/'+name))
        if not matches:
            copied[name]={'unavailable':True};continue
        for p in matches:need(sha(p)==pkg['checksum'],'locked archive mismatch')
        p=matches[0];dest=home/'registry/cache'/p.parent.name/name;dest.parent.mkdir(exist_ok=True);shutil.copyfile(p,dest);copied[name]=sha(dest)
    # Index is immutable dependency metadata; no inherited Cargo config or src/build outputs.
    shutil.copytree(OLD/'cargo-home/registry/index',home/'registry/index',dirs_exist_ok=True)
    shutil.copytree(OLD/'cargo-home/git/db',home/'git/db',dirs_exist_ok=True)
    write(w/'dependency-archives.json',copied)
    write(HERE/'work.json',dict(work=str(w)))

def build(w):
    extra=[p for p in (w/'cargo-home').rglob('*') if p.is_file()]
    args=[TC/'bin/cargo','build','--offline','--locked','--release']
    for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']:args+=['-p',p]
    out,r=epoch(w,'build',args,extra=extra,products=[w/'target/release'/n for n in NAMES+['vstd_build']]);need(r['exit']==0,'matching compiler failed')

def vstd(w):
    d=w/'target/release';out,r=epoch(w,'vstd',[d/'vstd_build',d,'--release','--verbose'],extra=[d/n for n in NAMES if n not in ['vstd.vir','libvstd.rlib','z3']]+[d/'vstd_build'],products=[d/'vstd.vir',d/'libvstd.rlib']);need(r['exit']==0,'normal matching vstd failed')
    bundle=w/'bundle';bundle.mkdir()
    for n in NAMES:shutil.copyfile(BASE/n if n=='z3' else d/n,bundle/n);(bundle/n).chmod(0o500 if n in ['verus','rust_verify','z3'] else 0o400)
    write(bundle/'receipt.json',dict(success=True,tools={n:sha(bundle/n) for n in NAMES},build_epochs=read_epochs(w)))
def read_epochs(w):return [json.loads(x) for x in (w/'epochs.jsonl').read_text().splitlines()]

if __name__=='__main__':
    if sys.argv[1]=='setup':setup()
    else:
        w=P(read(HERE/'work.json')['work'])
        {'build':build,'vstd':vstd}[sys.argv[1]](w)
