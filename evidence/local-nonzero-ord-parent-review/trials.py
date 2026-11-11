"""Local receipts only; no historical VIR positives. Use python -B, also valid -O."""
from build import *
R=root()
def freeze():
 need(json.loads((R/('vstd-'+os.environ.get('LOCAL_NONZERO_ORD_BUNDLE','bundle').replace('bundle-','')+'/receipt.json')).read_text())['exit']==0,'matching vstd failed')
 need(not (R/(os.environ.get('LOCAL_NONZERO_ORD_BUNDLE','bundle')+'-final-tools.json')).exists(),'immutable tools already frozen')
 need(manifest(ROOT)==manifest(R/'src'),'own source snapshot differs')
 write(R/(os.environ.get('LOCAL_NONZERO_ORD_BUNDLE','bundle')+'-final-tools.json'),dict(tools={n:sha(bundle()/n) for n in TOOLS},source=manifest(ROOT),before_proofs=True))
def app(label,source,goals=True,options=(),export=False,cargo=False,name="local_order",deps=(),paths_label=None):
 need((R/(os.environ.get('LOCAL_NONZERO_ORD_BUNDLE','bundle')+'-final-tools.json')).exists(),'freeze eight tools before proofs')
 f=json.loads((R/(os.environ.get('LOCAL_NONZERO_ORD_BUNDLE','bundle')+'-final-tools.json')).read_text())
 need(all(sha(bundle()/n)==h for n,h in f['tools'].items()),'tool drift')
 need(manifest(ROOT)==f['source']==manifest(R/'src'),'source drift')
 receipt_dir=R/label;receipt_dir.mkdir(exist_ok=True)
 d=R/(paths_label or label);d.mkdir(exist_ok=True)
 p=R/('source-'+sha_text(source)[:16]+'.rs')
 if p.exists():need(p.read_text()==source,'fixture changed')
 else:p.write_text(source)
 args=[bundle()/'verus',p,'--crate-name',name,'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir','--compile','-o',d/('lib'+name+'.rlib')]
 extra=None
 if cargo:
  args=[bundle()/'rust_verify',TC/'bin/rustc',p,'--crate-name',name,'--crate-type=lib',
        '--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir',
        '--VIA-CARGO','compile-when-primary-package','--emit=metadata','--out-dir',d,
        '--extern','verus_builtin='+str(bundle()/'libverus_builtin.rlib'),
        '--extern','verus_builtin_macros='+str(bundle()/'libverus_builtin_macros.so'),
        '--extern','vstd='+str(bundle()/'libvstd.rlib'),
        '--import','vstd='+str(bundle()/'vstd.vir'),'-L','dependency='+str(bundle())]
  project=R/'cargo-project';project.mkdir(exist_ok=True)
  package='local_order-0.0.0-'+hashlib.sha256(str(project/'Cargo.toml').encode()).hexdigest()[:12]
  extra={'__VERUS_DRIVER_VIA_CARGO__':'1','__VERUS_DRIVER_VERIFY_'+package:'1',
         'CARGO_PKG_NAME':'local_order','CARGO_PKG_VERSION':'0.0.0','CARGO_MANIFEST_DIR':str(project),
         'CARGO_PRIMARY_PACKAGE':'1'}
 if goals:args+=['--infer-goals']
 if export and not cargo:args+=['--export',d/(name+'.vir')]
 for dep,path in deps:
  args+=['--extern',dep+'='+str(path/('lib'+dep+'.rlib')),'--import',dep+'='+str(path/(dep+'.vir')),'-L',path]
 dependencies={str(path/(prefix+dep+ext)):sha(path/(prefix+dep+ext)) for dep,path in deps for prefix,ext in [('', '.vir'),('lib','.rlib')]}
 if '--no-erasure-check' in options and '--compile' in args:args.remove('--compile')
 args+=list(options)
 rec=run(label,args,extra)
 need(all(sha(p)==h for p,h in dependencies.items()),'dependency drift')
 write(receipt_dir/'dependency-inputs.json',dependencies)
 err=(receipt_dir/'stderr').read_text()
 need(not any(s in err.lower() for s in ['panicked at','internal compiler error','resource limit','timed out']),'nonsemantic compiler/resource failure')
 return rec
def sha_text(s):return hashlib.sha256(s.encode()).hexdigest()
ORD=(ROOT/'source/tools/research/nonzero_ordering_goal/run.py').read_text()
# Load only the pure source generator, never its historical controller main.
spec=__import__('importlib.util',fromlist=['']).spec_from_file_location('baseline_source',ROOT/'source/tools/research/nonzero_ordering_goal/run.py')
base=__import__('importlib.util',fromlist=['']).module_from_spec(spec);spec.loader.exec_module(base)
DIRECT=base.source('carrier_ord')
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'Python -B required')
 if sys.argv[1]=='freeze':freeze()
 elif sys.argv[1]=='direct':app(os.environ.get('LOCAL_NONZERO_ORD_BUNDLE','bundle')+'-direct',DIRECT)
 elif sys.argv[1]=='off':app('goals-off-epoch01',DIRECT,goals=False)
