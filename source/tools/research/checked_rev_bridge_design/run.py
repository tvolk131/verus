"""Private read-only inputs; no compiler edits/build, network, caches or import mixing."""
import hashlib,json,os,pathlib,shutil,subprocess,tempfile,sys
import probes
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[3]
TC=pathlib.Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
B=pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined74-bundle-d3b2aa6e')
def need(b,m):
    if not b: raise RuntimeError(m)
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def write(p,j): p.write_text(json.dumps(j,indent=2)+'\n')
def run(argv,d,env):
    d.mkdir(); argv=list(map(str,argv))
    with (d/'stdout').open('wb') as o,(d/'stderr').open('wb') as e:
        r=subprocess.run(argv,cwd=ROOT,env=env,stdout=o,stderr=e)
    j={'argv':argv,'exit':r.returncode,'stdout':{'path':str(d/'stdout'),'sha256':sha(d/'stdout')},'stderr':{'path':str(d/'stderr'),'sha256':sha(d/'stderr')}}
    write(d/'receipt.json',j); print(d.name,r.returncode,flush=True); return j

def main():
    need(sys.flags.dont_write_bytecode,'run -B')
    r=pathlib.Path(tempfile.mkdtemp(prefix='checked-rev-bridge-')); print(r,flush=True)
    receipt=B/'receipt.json'; need(sha(receipt)=='5e7e79cf6b6cef0509226cf6134c8fce0a8c47ed608fc4c5a7520b5023aa4c45','receipt hash')
    j=json.loads(receipt.read_text()); need(j['source_commit']=='d3b2aa6e63cbc440e09cd5764d99df94e315e959','engine')
    need(set(j['tools'])==set(['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir']),'eight tools')
    bundle=r/'bundle'; bundle.mkdir()
    for n,h in j['tools'].items():
        need(sha(B/n)==h,'shared hash '+n); shutil.copy2(B/n,bundle/n); need(sha(bundle/n)==h,'copy hash '+n)
    env={k:v for k,v in os.environ.items() if k in ['PATH','LANG','LC_ALL','USER']}
    for n in ['home','cargo','target','tmp','cache']:(r/n).mkdir()
    env.update(HOME=str(r/'home'),CARGO_HOME=str(r/'cargo'),CARGO_TARGET_DIR=str(r/'target'),TMPDIR=str(r/'tmp'),XDG_CACHE_HOME=str(r/'cache'),RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1',RUSTC_BOOTSTRAP='1',VERUS_Z3_PATH=str(bundle/'z3'),LD_LIBRARY_PATH=str(TC/'lib'),PATH=str(TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin')
    engine=subprocess.check_output(['git','diff','--name-only',j['source_commit'],'--','source'],cwd=ROOT,text=True).splitlines()
    need(all(p.startswith(('source/tools/research/','source/docs/')) for p in engine),'production differs from engine')
    engine_files=subprocess.check_output(['git','ls-tree','-r','--name-only',j['source_commit'],'source'],cwd=ROOT,text=True).splitlines()
    engine_files=[p for p in engine_files if not p.startswith(('source/tools/research/','source/docs/'))]
    production={p:sha(ROOT/p) for p in engine_files}
    tpaths=[TC/'bin/rustc',pathlib.Path('/home/dev/.cargo/bin/rustup')]+list((TC/'lib').glob('librustc_driver*.so'))
    for pat in ['libcore-*.rlib','libstd-*.rlib','liballoc-*.rlib','libcompiler_builtins-*.rlib']:
        tpaths+=list((TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib').glob(pat))
    tpaths+= [TC/'lib/rustlib/src/rust/library/core/src/iter'/p for p in ['adapters/rev.rs','adapters/enumerate.rs','traits/iterator.rs']]
    toolchain={str(p):sha(p) for p in tpaths}; need(len(toolchain)>=9,'toolchain files')
    inputs={str(HERE/n):sha(HERE/n) for n in ['inspect.rs','native.rs','probes.py','run.py','observations.smt2']}
    write(r/'freeze.json',{'tools':j['tools'],'toolchain':toolchain,'production':production,'controllers':inputs,'environment':env,'receipt_sha256':sha(receipt)})
    trials={}
    trials['rustc_version']=run([TC/'bin/rustc','-vV'],r/'rustc_version',env)
    trials['driver_build']=run([TC/'bin/rustc',HERE/'inspect.rs','--edition=2024','-o',r/'inspect','-L',TC/'lib','-C','rpath=yes'],r/'driver_build',env)
    if trials['driver_build']['exit']!=0: raise RuntimeError((r/'driver_build/stderr').read_text())
    shutil.copyfile(HERE/'native.rs',r/'native.rs')
    trials['mir']=run([r/'inspect',r/'native.rs','--crate-name','rev_witness','--crate-type=lib','--edition=2024','--sysroot',TC,'-C','opt-level=0','-Z','mir-opt-level=0','--emit=metadata','-o',r/'witness.rmeta'],r/'mir',env)
    if trials['mir']['exit']!=0:raise RuntimeError((r/'mir/stderr').read_text())
    # Actual original lifetime/type/body/erased Rust compile, independently of diagnostic callback.
    trials['native_compile']=run([TC/'bin/rustc',r/'native.rs','--crate-name','rev_witness','--crate-type=lib','--edition=2024','-o',r/'libwitness.rlib'],r/'native_compile',env)
    for name in probes.BODIES:
        p=r/(name+'.rs');p.write_text(probes.source(name))
        for infer in [False,True]:
            label=name+('-on' if infer else '-off');d=r/label
            trials[label]=run([bundle/'verus',p,'--crate-name','rev_probe','--crate-type=lib','--no-cheating','--compile','-o',r/(label+'.rlib'),'--output-json','--multiple-errors','10',*(['--infer-goals'] if infer else [])],d,env)
            try: trials[label]['verification']=json.loads((d/'stdout').read_text())['verification-results']
            except (ValueError,KeyError):raise RuntimeError('probe frontend/setup failure '+label)
            need(trials[label]['verification']['success'] or trials[label]['verification']['errors']>0,'nonsemantic failure '+label)
            print(trials[label]['verification'],flush=True)
    shutil.copyfile(HERE/'observations.smt2',r/'observations.smt2')
    trials['observations']=run([bundle/'z3',r/'observations.smt2'],r/'observations',env)
    need(trials['observations']['exit']==0,'solver diagnostic failure')
    lines=(r/'observations/stdout').read_text().splitlines()
    need(lines[0]=='sat' and lines[-1]=='unsat','observation diagnostic outcomes')
    need(all(sha(bundle/n)==h for n,h in j['tools'].items()),'tool drift')
    need(all(sha(p)==h for p,h in toolchain.items()),'toolchain drift')
    need(all(sha(ROOT/p)==h for p,h in production.items()),'production drift')
    need(all(sha(p)==h for p,h in inputs.items()),'controller drift')
    products={str(p):sha(p) for p in [r/'freeze.json',r/'inspect',r/'native.rs',r/'libwitness.rlib',r/'observations.smt2',*r.glob('*.rs')]}
    result={'schema':1,'private_root':str(r),'trials':trials,'products':products,'tools':j['tools'],'toolchain':toolchain,'goal_complete':False,'publication':False,'native_correspondences':0,'diagnostic_only':True}
    write(r/'RESULTS.json',result);write(HERE/'RESULTS.json',result)
if __name__=='__main__':main()
