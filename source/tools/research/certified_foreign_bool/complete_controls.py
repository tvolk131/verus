from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
ordinary='use vstd::prelude::*; verus! {pub fn seed()->u64 {0}}'
for cargo in [False,True]:
 for goals in [False,True]:
  label=('cargo' if cargo else 'explicit')+('-infer' if goals else '-ordinary')
  if not (R/label/'receipt.json').exists():require(label,app(label,'seed',ordinary,goals=goals,cargo=cargo),1)
 controls={'proof':BOOL.replace('r==if b {1u64}else{0u64}','r==0u64'),'parse':'use vstd::prelude::*; fn broken( {','type':BOOL.replace('u64::from(b)','unknown_variable')}
 for opt in ['--no-verify','--no-lifetime','--no-trait-conflicts','--no-erasure-check','--verify-root','--verify-module=other']:controls[opt]=BOOL
 for control,src in controls.items():
  original=('cargo-' if cargo else 'explicit-')+control.removeprefix('--').replace('=','-')
  if (R/original/'receipt.json').exists() and control!='--no-erasure-check':continue
  label=original+'-valid';d=R/label;d.mkdir();selected=d/('libprovider.vir' if cargo else 'provider.vir')
  shutil.copy2(R/('cargo-ordinary/libseed.vir' if cargo else 'explicit-ordinary/seed.vir'),selected)
  require(label,app(label,'provider',src,goals=True,options=[control] if control.startswith('--') else [],cargo=cargo),None,success=False)
  need(not selected.exists(),'valid disabled/failed export survived '+label)
