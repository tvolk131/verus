"""Additional rustc-selected cargo metadata paths, using actual wrapper entry."""
import hashlib,json
import review as t
from proofs import source
R=t.R;B=R/'bundle';project=R/'cargo-project'
package='review_bool-0.0.0-'+hashlib.sha256(str(project/'Cargo.toml').encode()).hexdigest()[:12]
extra={'__VERUS_DRIVER_VIA_CARGO__':'1','__VERUS_DRIVER_VERIFY_'+package:'1','CARGO_PKG_NAME':'review_bool','CARGO_PKG_VERSION':'0.0.0','CARGO_MANIFEST_DIR':str(project),'CARGO_PRIMARY_PACKAGE':'1'}
for shape in ['extra-filename','explicit-emit','single-output']:
 for stage in ['complete','parse','type']:
  label='cargo-path-'+shape+'-'+stage;d=R/label;d.mkdir();src=d/'component.rs';src.write_text(source('actual')+('fn broken( {' if stage=='parse' else 'fn bad()->u64{()}' if stage=='type' else ''))
  if shape=='extra-filename':opts=['--emit=metadata','--out-dir',d,'-C','extra-filename=-review'];selected=d/'libcomponent-review.vir'
  elif shape=='explicit-emit':opts=['--emit=metadata='+str(d/'chosen.rmeta'),'--out-dir',d];selected=d/'chosen.vir'
  else:opts=['--emit=metadata','-o',d/'chosen.rmeta'];selected=d/'chosen.vir'
  selected.write_bytes((R/'ordinary.vir').read_bytes())
  args=[B/'rust_verify',t.TC/'bin/rustc',src,'--crate-name','component','--crate-type=lib','--no-cheating','--output-json','--infer-goals','--VIA-CARGO','compile-when-primary-package',*opts,'--extern','verus_builtin='+str(B/'libverus_builtin.rlib'),'--extern','verus_builtin_macros='+str(B/'libverus_builtin_macros.so'),'--extern','vstd='+str(B/'libvstd.rlib'),'--import','vstd='+str(B/'vstd.vir'),'-L','dependency='+str(B)]
  r=t.run(label,args,extra);t.need(r['exit']!=0 and not selected.exists(),'selected stale path survived '+label)
  if stage=='complete':t.need('local-only' in (d/'stderr').read_text(),'wrong refusal '+label)
