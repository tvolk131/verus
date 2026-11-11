from matrix import *
provider='use vstd::prelude::*; verus! {pub fn old_api(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}'
results={}
for same in [False,True]:
 label='dual-same' if same else 'dual-distinct';prior=label+'-prior';d=R/prior
 expect(prior,provider,ok=True,verified=2,errors=0,cargo=True)
 cargo=d/'liblocal_order.vir';raw=cargo.read_bytes();old=sha(cargo)
 cert=json.loads(raw[raw.rfind(b'{"schema":'):-8])
 need(raw.endswith(b'VRINFER1') and cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v73','fresh same-runtime Bool cert')
 explicit=cargo if same else d/'explicit.vir';explicit.write_bytes(raw)
 rec=app(label+'-refusal',DIRECT,cargo=True,paths_label=prior,options=['--export',str(explicit)])
 need(rec['exit']!=0 and 'local-only' in (R/(label+'-refusal')/'stderr').read_text(),'local-only rejection')
 need(not cargo.exists() and not explicit.exists(),'both selected outputs removed')
 results[label]=dict(prior_artifact_sha256=old,schema=cert['schema'],policy=cert['policy'],same_path=same,explicit_path=str(explicit),cargo_path=str(cargo),both_removed=True,prior_receipt_sha256=sha(d/'receipt.json'),refusal_receipt_sha256=sha(R/(label+'-refusal')/'receipt.json'))
write(R/'dual-result.json',results)
