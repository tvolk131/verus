"""Private local milestone trials. Run with Python -B (also valid with -O).
No prior writing/audit driver is executed; source.py/inventory.py are pure readers.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined69-bundle-3aa7f49a')
TC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
TOOLS = ['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so',
         'libverus_state_machines_macros.so','vstd.vir','libvstd.rlib']

def need(ok, message):
    if not ok:
        raise RuntimeError(message)

def sha(p):
    h = hashlib.sha256()
    with Path(p).open('rb') as f:
        for block in iter(lambda: f.read(1024*1024), b''):
            h.update(block)
    return h.hexdigest()

def write(p, x):
    Path(p).write_text(json.dumps(x, indent=2, sort_keys=True)+'\n')

def snapshot():
    paths = subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
    paths += ['source/rust_verify/src/checked_foreign.rs','source/rust_verify/src/checked_foreign_packing.rs']
    return {p:sha(ROOT/p) for p in sorted(set(paths)) if p and (ROOT/p).is_file()
            and not p.startswith('source/tools/research/local_checked_bool_from_mir/')}

def environment(d, bundle):
    return {**os.environ, 'PATH':'/home/dev/.cargo/bin:/usr/bin:/bin',
            'RUSTUP_TOOLCHAIN':'1.99.1','RUSTC_BOOTSTRAP':'1',
            'LD_LIBRARY_PATH':str(TC/'lib'),'CARGO_BUILD_JOBS':'2',
            'CARGO_NET_OFFLINE':'true','CARGO_HOME':str(d/'cargo-home'),
            'CARGO_TARGET_DIR':str(d/'target'),'CARGO_BUILD_BUILD_DIR':str(d/'target'),
            'VERUS_ROOT':str(bundle),'VERUS_Z3_PATH':str(bundle/'z3'),
            'VERUS_INFER_CACHE':str(d/'candidate-cache'),'XDG_CACHE_HOME':str(d/'cache')}

def run(root, label, argv, bundle, extra=None, outputs=()):
    d=root/label; d.mkdir(parents=True,exist_ok=True)
    before=snapshot()
    fixtures={str(Path(a)):sha(a) for a in argv if str(a).endswith('.rs') and Path(a).is_absolute() and Path(a).is_file()}
    tools={n:sha(bundle/n) for n in TOOLS if n not in outputs and (bundle/n).is_file()}
    env=environment(root,bundle); env.update(extra or {})
    frozen=dict(argv=list(map(str,argv)),cwd=str(ROOT/'source'),source=before,fixtures=fixtures,tools=tools,
                environment={k:env[k] for k in environment(root,bundle) if k not in os.environ
                             or env[k]!=os.environ.get(k)},controller_sha256=sha(__file__))
    write(d/'freeze.json',frozen); start=time.monotonic()
    with (d/'stdout').open('wb') as out, (d/'stderr').open('wb') as err:
        proc=subprocess.run(argv,cwd=ROOT/'source',env=env,stdout=out,stderr=err)
    guards=before==snapshot() and all(sha(p)==h for p,h in fixtures.items()) and tools=={n:sha(bundle/n) for n in tools} and sha(__file__)==frozen['controller_sha256']
    record=dict(exit=proc.returncode,seconds=time.monotonic()-start,guards=guards,
                freeze_sha256=sha(d/'freeze.json'),stdout_sha256=sha(d/'stdout'),
                stderr_sha256=sha(d/'stderr'),outputs={n:sha(bundle/n) for n in outputs if (bundle/n).is_file()})
    try:
        output=json.loads((d/'stdout').read_text())
        record['verification']=output.get('verification-results')
        record['failed_functions']=output.get('verus-infer',{}).get('failed_functions',[])
    except ValueError:
        pass
    write(d/'receipt.json',record)
    print(json.dumps(dict(label=label,path=str(d),**record)),flush=True)
    need(guards,'source/tool drift '+label)
    return d,record

def source(kind):
    if kind in {'actual','false','off'}:
        expected='0u64' if kind=='false' else 'if b {1u64}else{0u64}'
        return ('use vstd::prelude::*;\nverus! {\npub fn all_bool(b:bool)->(r:u64) '
                'ensures r=='+expected+', { u64::from(b) }\n}\n')
    folder=HERE.parent/'item_tag_constructor_composition'
    sys.path.insert(0,str(folder))
    from source import generate,fidelity
    text=generate(kind); fidelity(text)
    return text

def trial(root,bundle,label,kind='actual',options=(),export=None,cargo=False):
    # Stable source filename across cold/warm runs.
    src=root/('source-'+kind+'.rs'); text=source(kind)
    if src.exists(): need(src.read_text()==text,'stable trial source')
    else: src.write_text(text)
    d=root/label; d.mkdir(parents=True,exist_ok=True)
    args=[bundle/'verus',src,'--crate-name=component','--crate-type=lib','--no-cheating',
          '--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir',
          '--compile','-o',d/'libcomponent.rlib']
    if kind!='off': args+=['--infer-goals']
    args+=list(options)
    if export: args+=['--export',export]
    if cargo: args+=['--VIA-CARGO','compile-when-primary-package','--emit=metadata','--out-dir',d]
    return run(root,label,args,bundle)

if __name__=='__main__':
    need(sys.flags.dont_write_bytecode,'use Python -B')
    root=Path(sys.argv[2]); bundle=root/'bundle'
    if sys.argv[1]=='vstd':
        run(root,'vstd',[root/'target/debug/vstd_build',bundle],bundle,{'VERUS_IN_VARGO':'1'},outputs=('vstd.vir','libvstd.rlib'))
    elif sys.argv[1]=='trial':
        trial(root,bundle,sys.argv[3],sys.argv[4] if len(sys.argv)>4 else 'actual')
