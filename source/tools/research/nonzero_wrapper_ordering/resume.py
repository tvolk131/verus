"""Append-only continuation receipts. Original checkpoint controllers stay unchanged."""
import hashlib,json,os,re,shutil,subprocess,sys,time
from pathlib import Path
import build as b
import fixtures
R=b.root();H=b.HERE;ROOT=b.ROOT;TC=b.TC
EPOCH=os.environ.get('WRAPPER_EPOCH','baseline')
B=R/('bundle' if EPOCH=='baseline' else 'bundle-'+EPOCH)
def write(p,j):b.write(p,j)
def need(x,m):b.need(x,m)
def sha(p):return b.sha(p)
def environment():
    return b.env()|dict(VERUS_ROOT=str(B),VERUS_Z3_PATH=str(B/'z3'),VERUS_INFER_CACHE=str(R/('cache-'+EPOCH)))
def run(label,args,extra=None,outputs=(),dependencies=()):
    d=R/label;d.mkdir(exist_ok=True);need(not (d/'freeze.json').exists(),'immutable receipt label');env=environment()|dict(extra or {})
    f=dict(argv=list(map(str,args)),cwd=str(ROOT/'source'),environment=env,source=b.manifest(),controllers=b.controllers(),tools={n:sha(B/n) for n in b.TOOLS if (B/n).exists() and n not in outputs},fixtures={str(p):sha(p) for p in args if str(p).endswith('.rs') and Path(p).is_file()},dependencies={str(p):sha(p) for p in dependencies})
    archive=R/'controller-snapshots';archive.mkdir(exist_ok=True)
    for n,h in f['controllers'].items():
        q=archive/(h+'.py')
        if not q.exists():shutil.copyfile(H/n,q)
    write(d/'freeze.json',f);fs=sha(d/'freeze.json');start=time.monotonic()
    with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:p=subprocess.run(f['argv'],cwd=f['cwd'],env=env,stdout=out,stderr=err)
    guards=f['source']==b.manifest() and f['controllers']==b.controllers() and all(sha(B/n)==h for n,h in f['tools'].items()) and all(sha(p)==h for p,h in (f['fixtures']|f['dependencies']).items()) and sha(d/'freeze.json')==fs
    j=dict(exit=p.returncode,guards=guards,seconds=time.monotonic()-start,freeze_sha256=fs,stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),outputs={n:sha(B/n) for n in outputs if (B/n).exists()})
    try:
        data=json.loads((d/'stdout').read_text());j['verification']=data['verification-results'];j['inference']=data.get('verus-infer');j['native']=data.get('verus-native')
    except (ValueError,KeyError):pass
    write(d/'receipt.json',j);print(label,json.dumps({k:v for k,v in j.items() if k not in ['inference','native','outputs']}),flush=True);need(guards,'input drift');return j

def freeze():
    path=R/(EPOCH+'-eight-tools.json');need(not path.exists(),'immutable epoch freeze')
    tools={n:sha(B/n) for n in b.TOOLS};write(path,dict(source=b.manifest(),tools=tools,controllers=b.controllers(),before_proofs=True))

def app(label,source,goals=True,options=(),cargo=False,export=None,name='wrapper',paths_label=None,deps=(),compile=True):
    frozen=json.loads((R/(EPOCH+'-eight-tools.json')).read_text());need(frozen['source']==b.manifest(),'frozen source drift');need(all(sha(B/n)==h for n,h in frozen['tools'].items()),'tool drift')
    p=R/('source-'+hashlib.sha256(source.encode()).hexdigest()+'.rs')
    if p.exists():need(p.read_text()==source,'immutable source')
    else:p.write_text(source)
    d=R/(paths_label or label);d.mkdir(exist_ok=True)
    args=[B/'verus',p,'--crate-name',name,'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','30','--log-all','--log-dir',d/'vir']
    extra=None
    if cargo:
        args=[B/'rust_verify',TC/'bin/rustc',p,'--crate-name',name,'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','30','--log-all','--log-dir',d/'vir','--VIA-CARGO','compile-when-primary-package','--emit=metadata','--out-dir',d,'--extern','verus_builtin='+str(B/'libverus_builtin.rlib'),'--extern','verus_builtin_macros='+str(B/'libverus_builtin_macros.so'),'--extern','vstd='+str(B/'libvstd.rlib'),'--import','vstd='+str(B/'vstd.vir'),'-L','dependency='+str(B)]
        project=R/'cargo-project';project.mkdir(exist_ok=True);package='wrapper-0.0.0-'+hashlib.sha256(str(project/'Cargo.toml').encode()).hexdigest()[:12]
        extra=dict(__VERUS_DRIVER_VIA_CARGO__='1',CARGO_PKG_NAME='wrapper',CARGO_PKG_VERSION='0.0.0',CARGO_MANIFEST_DIR=str(project),CARGO_PRIMARY_PACKAGE='1');extra['__VERUS_DRIVER_VERIFY_'+package]='1'
    elif compile and '--no-erasure-check' not in options:args+=['--compile','-o',d/('lib'+name+'.rlib')]
    if goals:args+=['--infer-goals']
    if export is not None:args+=['--export',export]
    dependencies=[]
    for n,path in deps:
        args+=['--extern',n+'='+str(path/('lib'+n+'.rlib')),'--import',n+'='+str(path/(n+'.vir')),'-L',path];dependencies.extend([path/('lib'+n+'.rlib'),path/(n+'.vir')])
    args+=list(options);rec=run(label,args,extra,dependencies=dependencies)
    err=(R/label/'stderr').read_text();need(not any(t in err.lower() for t in ['panicked at','internal compiler error','resource limit','timed out']),'excluded compiler/resource failure')
    return rec

def build():
    need(not B.exists(),'new bundle epoch');B.mkdir()
    rec=run('build-'+EPOCH,[TC/'bin/cargo','build','--offline','--locked','-j','2',*sum((['-p',p] for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']),[])])
    need(rec['exit']==0,'build failed')
    for n in b.TOOLS:
        if n in ['libvstd.rlib','vstd.vir']:continue
        shutil.copy2(R/'bundle/z3' if n=='z3' else R/'target/debug'/n,B/n)
    (B/'verus-root').touch();shutil.copytree(ROOT/'source/vstd',B/'vstd')

def vstd():
    rec=run('vstd-'+EPOCH,[R/'target/debug/vstd_build',B],outputs=['libvstd.rlib','vstd.vir']);need(rec['exit']==0,'vstd failed')
    need('2250 verified, 0 errors' in (R/('vstd-'+EPOCH)/'stdout').read_text(),'matching vstd count');freeze()

if __name__=='__main__':
    need(sys.flags.dont_write_bytecode,'Python -B required')
    mode=sys.argv[1]
    if mode=='freeze':freeze()
    elif mode=='build':build()
    elif mode=='vstd':vstd()
    elif mode=='baseline':
        for m in ['original','request','authored']:
            for g in [False,True]:app('baseline-'+m+('-on' if g else '-off'),fixtures.source(m),goals=g)
