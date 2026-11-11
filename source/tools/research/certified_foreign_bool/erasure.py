from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
for cargo in [False,True]:
 label=('cargo' if cargo else 'explicit')+'-erased-type-failure-v2';d=R/label;d.mkdir();selected=d/('libprovider.vir' if cargo else 'provider.vir')
 shutil.copy2(R/'bool-provider/provider.vir',selected)
 source=BOOL+'\n#[cfg(not(verus_keep_ghost_body))] fn erased_type_error()->u64 { () }\n'
 rec=app(label,'provider',source,goals=True,cargo=cargo)
 require(label,rec,None,success=False)
 text=(d/'stderr').read_text();need('erased_type_error' in text and 'E0308' in text,'expected erased Rust type failure')
 need(not selected.exists(),'publication occurred before erased compilation completed')
 j=json.loads((d/'stdout').read_text());need(j['verification-results'].get('verified')==2,'foreign/provider ordinary proof must have run')
