from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
need(sha(P/'receipt.json')=='f24584b8ccfde3ac92e84c146d87317b04231b2b165ef759411a0c53ea3199e3','accepted legacy bundle receipt')
legacy=json.loads((P/'receipt.json').read_text())
for n,h in legacy['tools'].items():need(sha(P/n)==h,'legacy tool '+n)
write(R/'legacy-tool-identities.json',legacy)
# Legacy consumer, current native artifacts, current certified provider. The
# actual legacy importer must reject policy/schema before any proof is queried.
d=R/'legacy-consumer-v2';d.mkdir();s=fixture('legacy-consumer-v2.rs',BOOL.replace('u64::from(b)','provider::all_bool(b)'))
args=[P/'rust_verify','--internal-test-mode',s,'--crate-name=old_consumer','--crate-type=lib','--no-cheating','--output-json','--compile','-o',d/'libold_consumer.rlib','--extern','verus_builtin='+str(R/'bundle/libverus_builtin.rlib'),'--extern','verus_builtin_macros='+str(R/'bundle/libverus_builtin_macros.so'),'--extern','vstd='+str(R/'bundle/libvstd.rlib'),'--import','vstd='+str(P/'vstd.vir'),'--extern','provider='+str(R/'bool-provider/libprovider.rlib'),'--import','provider='+str(R/'bool-provider/provider.vir'),'-L','dependency='+str(R/'bundle'),'-L',R/'bool-provider']
r=run('legacy-consumer-v2',args,{'VERUS_ROOT':str(P)});require('legacy-consumer-v2',r,None,success=False)
need('incompatible inference schema' in (d/'stderr').read_text(),'actual legacy consumer schema refusal')
# Legacy inference-aware artifact without any foreign capability, then a new
# ordinary consumer. Missing required v39 fields/schema must still refuse.
d=R/'legacy-provider-v2';d.mkdir();s=fixture('legacy-provider-v2.rs','use verus_builtin::*; use verus_builtin_macros::*; verus! {pub fn value(x:u64)->u64 {x}}')
base=['--cfg=verus_no_vstd','--crate-type=lib','--no-cheating','--output-json','--compile','--extern','verus_builtin='+str(P/'libverus_builtin.rlib'),'--extern','verus_builtin_macros='+str(P/'libverus_builtin_macros.so'),'-L','dependency='+str(P)]
r=run('legacy-provider-v2',[P/'rust_verify','--internal-test-mode',s,*base,'--crate-name=legacy','--export',d/'legacy.vir','-o',d/'liblegacy.rlib','--infer-goals'],{'VERUS_ROOT':str(P)})
require('legacy-provider-v2',r,1)
need(metadata(d/'legacy.vir')[1]['schema']==38,'real legacy certificate')
f=R/'new-consumer-old-v2';f.mkdir();s=fixture('new-consumer-old-v2.rs','use verus_builtin::*; use verus_builtin_macros::*; verus! {pub fn read(x:u64)->u64 {legacy::value(x)}}')
r=run('new-consumer-old-v2',[R/'bundle/rust_verify','--internal-test-mode',s,*base,'--crate-name=new_old','--export',f/'new_old.vir','-o',f/'libnew_old.rlib','--extern','legacy='+str(d/'liblegacy.rlib'),'--import','legacy='+str(d/'legacy.vir'),'-L',d])
require('new-consumer-old-v2',r,None,success=False)
need('inference proof metadata' in (f/'stderr').read_text(),'actual new consumer old-format refusal')
