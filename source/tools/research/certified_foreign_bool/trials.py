"""No assumptions hidden in Python assert; usable with -B and -O -B."""
from run import *
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

def main():
 mode=sys.argv[1]
 if mode=='vstd':
  require('matching-vstd',run('matching-vstd',[R/'target/debug/vstd_build',R/'bundle'],outputs=('vstd.vir','libvstd.rlib')),None)
 elif mode=='bool':
  for label in ['bool-provider','bool-provider-warm']:
   require(label,app(label,'provider',BOOL,goals=True),2)
  for label,text,n,err,ok in [('bool-client','r==if b {1u64}else{0u64}',1,0,True),('false-client','r==0u64',0,1,False)]:
   src='use vstd::prelude::*; verus! {pub fn observe(b:bool)->(r:u64) ensures '+text+', {provider::all_bool(b)}}'
   require(label,app(label,label.replace('-','_'),src,[('provider',R/'bool-provider')]),n,err,ok)
  re='use vstd::prelude::*; verus! {pub fn via(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {provider::all_bool(b)}}'
  require('reexport',app('reexport','bridge',re,[('provider',R/'bool-provider')]),1)
  consumer='use vstd::prelude::*; verus! {pub fn observe(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {bridge::via(b)}}'
  require('transitive-client',app('transitive-client','third',consumer,[('bridge',R/'reexport'),('provider',R/'bool-provider')]),1)
  require('transitive-missing',app('transitive-missing','missing',consumer,[('bridge',R/'reexport')],options=['-L',R/'bool-provider']),None,success=False)
  private=BOOL.replace('pub fn all_bool','fn all_bool')+'verus! {pub fn unrelated()->(r:u64) ensures r==9u64, {9}}'
  require('private-provider',app('private-provider','private',private,goals=True),3)
  require('private-client',app('private-client','private_client','use vstd::prelude::*; verus! {pub fn read()->(r:u64) ensures r==9u64, {private::unrelated()}}',[('private',R/'private-provider')]),1)
  for label in ['bool-provider','private-provider','reexport','transitive-client']:
   p=next((R/label).glob('*.vir'));_,j=metadata(p);need(len(j['foreign_dependencies'])==1,'foreign evidence lost '+label)
 elif mode=='full':
  # Pure source generator only; old output paths are never consulted.
  sys.path.insert(0,str(ROOT/'source/tools/research/item_tag_constructor_composition'))
  from source import generate,fidelity
  src=generate('full-help');fidelity(src)
  need(src==Path('/tmp/checked-bool-build-klx7abqu/source-full-help.rs').read_text(),'unchanged full-help baseline')
  for label in ['full-provider','full-provider-warm']:
   require(label,app(label,'component',src,goals=True),52)
  obs='''use vstd::prelude::*; use component::*; verus! {
    pub fn tag(t:BorTag,p:Permission,b:bool)->(r:Item)
      requires t.value() <= (u64::MAX >> 3),
      ensures item_word_wf(r.word()), r.word() & (u64::MAX >> 3)==t.value(),
      {Item::new(t,p,b)}
    pub fn permission(x:Item)->(r:Permission) ensures r==x.permission(), {x.perm()}
    pub fn protector(x:Item)->(r:bool) ensures r==(x.word() & 0x8000000000000000u64 > 0), {x.protected()}
    pub fn eq(a:&BorTag,b:&BorTag)->(r:bool) ensures r==(a.value()==b.value()), {compare_equal(a,b)}
    pub fn lookup_word(w:u64)->(r:u64) requires item_word_wf(w), ensures r==(w & (u64::MAX >> 3)), {lookup(w)}
  }'''
  require('full-observer',app('full-observer','observer',obs,[('component',R/'full-provider')]),5)
  require('full-observer-false',app('full-observer-false','observer_false',obs.replace('r==(x.word() & 0x8000000000000000u64 > 0)','r==false'),[('component',R/'full-provider')]),4,1,False)
 elif mode=='controls':
  base=R/'bool-provider/provider.vir';prefix,cert=metadata(base)
  import copy
  mutations={'schema':lambda j:j.update(schema=38),'policy':lambda j:j.update(policy='candidate-bound-evidence-v70'),'producer':lambda j:j.update(producer='different'), 'model':lambda j:j['trusted_models'][0].__setitem__(1,'0'*64), 'foreign-missing':lambda j:j.pop('foreign_dependencies'), 'foreign-removed':lambda j:j.update(foreign_dependencies=[]), 'target':lambda j:j['foreign_dependencies'][0]['identity'].update(target='0'*64), 'mir':lambda j:j['foreign_dependencies'][0]['identity'].update(typed_mir='different'), 'core':lambda j:j['foreign_dependencies'][0]['identity'].update(core_artifact='0'*64), 'semantic':lambda j:j['foreign_dependencies'][0]['identity'].update(semantic_instance='spelling only')}
  client='use vstd::prelude::*; verus! {pub fn observe(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {provider::all_bool(b)}}'
  for name,mutate in mutations.items():
   d=R/('mutated-'+name);d.mkdir();shutil.copy2(R/'bool-provider/libprovider.rlib',d/'libprovider.rlib');j=copy.deepcopy(cert);mutate(j);(d/'provider.vir').write_bytes(prefix+json.dumps(j,separators=(',',':')).encode()+b'VRINFER1')
   require('refuse-'+name,app('refuse-'+name,'refused_'+name.replace('-','_'),client,[('provider',d)]),None,success=False)
  # Legacy and inference-aware ordinary export behavior, and stale selection.
  ordinary='use vstd::prelude::*; verus! {pub fn seed()->u64 {0}}'
  for cargo in [False,True]:
   for goals in [False,True]:
    label=('cargo' if cargo else 'explicit')+('-infer' if goals else '-ordinary')
    require(label,app(label,'seed',ordinary,goals=goals,cargo=cargo),1)
   controls={'proof':BOOL.replace('r==if b {1u64}else{0u64}','r==0u64'),'parse':'use vstd::prelude::*; fn broken( {','type':BOOL.replace('u64::from(b)','unknown_variable')}
   for opt in ['--no-verify','--no-lifetime','--no-trait-conflicts','--no-erasure-check','--verify-root','--verify-module=other']:
    controls[opt]=BOOL
   for control,src in controls.items():
    label=('cargo-' if cargo else 'explicit-')+control.removeprefix('--').replace('=','-');d=R/label;d.mkdir()
    selected=d/('libprovider.vir' if cargo else 'provider.vir')
    shutil.copy2(R/('cargo-ordinary/libseed.vir' if cargo else 'explicit-ordinary/seed.vir'),selected)
    opts=[control] if control.startswith('--') else []
    require(label,app(label,'provider',src,goals=True,options=opts,cargo=cargo),None,success=False)
    need(not selected.exists(),'stale selected output '+label)
 elif mode=='off':
  require('goals-off',app('goals-off','off',BOOL),0,1,False)
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B');main()
