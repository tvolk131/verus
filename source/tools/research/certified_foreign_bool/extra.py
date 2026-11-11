from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
mode=sys.argv[1]
if mode=='transitive':
 src='use vstd::prelude::*; verus! {pub fn unrelated()->(r:u64) ensures r==3u64, {3}}'
 require('unused-reexport',app('unused-reexport','unused',src,[('provider',R/'bool-provider')]),1)
 c='use vstd::prelude::*; verus! {pub fn get()->(r:u64) ensures r==3u64, {unused::unrelated()}}'
 require('unused-consumer',app('unused-consumer','unused_client',c,[('unused',R/'unused-reexport'),('provider',R/'bool-provider')]),1)
 for label,source,deps in [('missing-transitive-typed', 'use vstd::prelude::*; verus! {pub fn get(b:bool)->u64 {bridge::via(b)}}',[('bridge',R/'reexport')]),('missing-unused-typed',c,[('unused',R/'unused-reexport')])]:
  require(label,app(label,label.replace('-','_'),source,deps,options=['-L',R/'bool-provider']),None,success=False)
  need('dependency fingerprint' in (R/label/'stderr').read_text(),'wrong transitive rejection')
 for label in ['unused-reexport','unused-consumer']:
  j=metadata(next((R/label).glob('*.vir')))[1];need(len(j['foreign_dependencies'])==1,'unused required evidence')
 # A real compiler-option mismatch with entirely unchanged certificate bytes.
 require('actual-target-mismatch',app('actual-target-mismatch','target_mismatch',BOOL.replace('u64::from(b)','provider::all_bool(b)'),[('provider',R/'bool-provider')],options=['-C','target-cpu=x86-64']),None,success=False)
 need('checked foreign identity mismatch' in (R/'actual-target-mismatch/stderr').read_text(),'fresh target inspection')
elif mode=='cargo':
 require('cargo-provider',app('cargo-provider','provider',BOOL,goals=True,cargo=True),2)
 require('cargo-client',app('cargo-client','cargo_client',BOOL.replace('u64::from(b)','provider::all_bool(b)'),[('provider',R/'cargo-provider')],cargo=True),1)
 j=metadata(R/'cargo-client/libcargo_client.vir')[1];need(len(j['foreign_dependencies'])==1,'cargo propagation')
 # Extra filename, explicit metadata file, and single -o output selection.
 for shape in ['extra-filename','explicit-emit','single-output']:
  for stage in ['complete','parse','type']:
   label='cargo-path-'+shape+'-'+stage;d=R/label;d.mkdir()
   src=BOOL+('fn broken( {' if stage=='parse' else 'fn bad()->u64{()}' if stage=='type' else '')
   f=fixture(label+'.rs',src)
   if shape=='extra-filename':opts=['--emit=metadata','--out-dir',d,'-C','extra-filename=-proof'];selected=d/'libcomponent-proof.vir'
   elif shape=='explicit-emit':opts=['--emit=metadata='+str(d/'chosen.rmeta'),'--out-dir',d];selected=d/'chosen.vir'
   else:opts=['--emit=metadata','-o',d/'chosen.rmeta'];selected=d/'chosen.vir'
   selected.write_bytes((R/'explicit-ordinary/seed.vir').read_bytes())
   project=R/'cargo-project';package='certified_bool-0.0.0-'+hashlib.sha256(str(project/'Cargo.toml').encode()).hexdigest()[:12]
   extra={'__VERUS_DRIVER_VIA_CARGO__':'1','__VERUS_DRIVER_VERIFY_'+package:'1','CARGO_PKG_NAME':'certified_bool','CARGO_PKG_VERSION':'0.0.0','CARGO_MANIFEST_DIR':str(project),'CARGO_PRIMARY_PACKAGE':'1'}
   args=[R/'bundle/rust_verify',TC/'bin/rustc',f,'--crate-name','component','--crate-type=lib','--no-cheating','--output-json','--infer-goals','--VIA-CARGO','compile-when-primary-package',*opts,'--extern','verus_builtin='+str(R/'bundle/libverus_builtin.rlib'),'--extern','verus_builtin_macros='+str(R/'bundle/libverus_builtin_macros.so'),'--extern','vstd='+str(R/'bundle/libvstd.rlib'),'--import','vstd='+str(R/'bundle/vstd.vir'),'-L','dependency='+str(R/'bundle')]
   rec=run(label,args,extra);require(label,rec,2 if stage=='complete' else None,success=stage=='complete')
   need(selected.exists()==(stage=='complete'),'selected cargo output '+label)
elif mode=='boundaries':
 for name,expr,ok,count in [('qualified','<u64 as From<bool>>::from(b)',True,2),('alias','Wide::from(b)',True,2),('function-value','{let f=u64::from; f(b)}',False,None),('coercion','{let f:fn(bool)->u64=u64::from; f(b)}',False,None)]:
  s=BOOL.replace('u64::from(b)',expr)
  if name=='alias':s='type Wide=u64;\n'+s
  label='boundary-'+name;r=app(label,name.replace('-','_'),s,goals=True,export=False);require(label,r,count,success=ok)
  if not ok:
   txt=(R/label/'vir/crate.vir').read_text() if (R/label/'vir/crate.vir').exists() else ''
   need('checked_foreign_bool_to_u64%' not in txt,'value/coercion eligibility expanded')
elif mode=='ordinary-controls':
 src=BOOL.replace('u64::from(b)','provider::all_bool(b)')
 for cargo in [False,True]:
  for name,options,source in [('no-verify',['--no-verify'],src),('no-lifetime',['--no-lifetime'],src),('no-trait',['--no-trait-conflicts'],src),('no-erasure',['--no-erasure-check'],src),('filter',['--verify-root'],src),('parse',[],'fn broken( {'),('type',[],src.replace('provider::all_bool(b)','unknown_variable'))]:
   label=('cargo-consumer-' if cargo else 'explicit-consumer-')+name;d=R/label;d.mkdir();selected=d/('libconsumer.vir' if cargo else 'consumer.vir')
   shutil.copy2(R/('cargo-ordinary/libseed.vir' if cargo else 'explicit-ordinary/seed.vir'),selected)
   require(label,app(label,'consumer',source,[('provider',R/('cargo-provider' if cargo else 'bool-provider'))],options=options,cargo=cargo),None,success=False)
   need(not selected.exists(),'stale ordinary consumer output '+label)
