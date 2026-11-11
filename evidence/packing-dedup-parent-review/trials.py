"""No assumptions hidden in Python assert; usable with -B and -O -B."""
from review import *
R=root()
BOOL='use vstd::prelude::*; verus! {pub fn all_bool(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}\n'

def fixture(name,source):
 p=R/'fixtures'/name;p.parent.mkdir(exist_ok=True)
 if p.exists():need(p.read_text()==source,'fixture drift '+name)
 else:p.write_text(source)
 return p

def app(label,name,source,deps=(),options=(),goals=False,export=True,cargo=False):
 d=R/label;d.mkdir(exist_ok=True)
 args=[R/'bundle/verus',fixture(name+'-'+hashlib.sha256(source.encode()).hexdigest()[:12]+'.rs',source),'--crate-name='+name,'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir','--compile']
 if cargo:
  args=[R/'bundle/rust_verify',TC/'bin/rustc',*args[1:]]
  args+=['--VIA-CARGO','compile-when-primary-package','--emit=metadata','--out-dir',d,
         '--extern','verus_builtin='+str(R/'bundle/libverus_builtin.rlib'),
         '--extern','verus_builtin_macros='+str(R/'bundle/libverus_builtin_macros.so'),
         '--extern','vstd='+str(R/'bundle/libvstd.rlib'),
         '--import','vstd='+str(R/'bundle/vstd.vir'),'-L','dependency='+str(R/'bundle')]
 else:args+=['-o',d/('lib'+name+'.rlib')]
 if export and not cargo:args+=['--export',d/(name+'.vir')]
 if goals:args+=['--infer-goals']
 for dep,depdir in deps:
  native=depdir/('lib'+dep+'.rlib')
  if not native.exists():native=depdir/('lib'+dep+'.rmeta')
  interface=depdir/(dep+'.vir')
  if not interface.exists():interface=depdir/('lib'+dep+'.vir')
  args+=['--extern',dep+'='+str(native),'--import',dep+'='+str(interface),'-L',depdir]
 if '--no-erasure-check' in options and '--compile' in args:args.remove('--compile')
 args+=list(options)
 extra=None
 if cargo:
  project=R/'cargo-project';project.mkdir(exist_ok=True)
  package='certified_bool-0.0.0-'+hashlib.sha256(str(project/'Cargo.toml').encode()).hexdigest()[:12]
  extra={'__VERUS_DRIVER_VIA_CARGO__':'1','__VERUS_DRIVER_VERIFY_'+package:'1','CARGO_PKG_NAME':'certified_bool','CARGO_PKG_VERSION':'0.0.0','CARGO_MANIFEST_DIR':str(project),'CARGO_PRIMARY_PACKAGE':'1'}
 r=run(label,args,extra);return r

def require(label,r,verified=None,errors=0,success=True):
 need((r['exit']==0)==success,'unexpected exit '+label+': '+(R/label/'stderr').read_text()[-2500:])
 v=r.get('verification') or {}
 if verified is not None:need(v.get('verified')==verified and v.get('errors')==errors,'proof inventory '+label+' '+str(v))
 return r

def metadata(path):
 data=Path(path).read_bytes();need(data.endswith(b'VRINFER1'),'required trailer')
 # Trailer's unique first schema field follows the ordinary bincode payload.
 i=data.rfind(b'{"schema":');need(i>=0,'metadata start')
 return data[:i],json.loads(data[i:-8])
