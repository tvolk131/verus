"""Ordinary stale output control using a genuine fresh same-runtime Bool artifact."""
from matrix import *
provider='use vstd::prelude::*; verus! {pub fn old_api(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}'
label='review-dual-valid-prior';d=R/label
expect(label,provider,ok=True,verified=2,errors=0,cargo=True)
cargo=d/'liblocal_order.vir';need(cargo.exists(),'ordinary Cargo provider artifact missing')
old=sha(cargo);raw=cargo.read_bytes();i=raw.rfind(b'{"schema":');cert=json.loads(raw[i:-8])
need(cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v73','fresh certificate epoch')
explicit=d/'explicit.vir';explicit.write_bytes(raw)
rec=app('review-dual-valid-refusal',DIRECT,cargo=True,paths_label=label,options=['--export',str(explicit)])
need(rec['exit']!=0 and 'local-only' in (R/'review-dual-valid-refusal/stderr').read_text(),'must reach local-only gate')
result=dict(before_sha256=old,cargo_exists=cargo.exists(),explicit_exists=explicit.exists(),after_sha256=sha(cargo) if cargo.exists() else None,prior_provider_receipt_sha256=sha(d/'receipt.json'),refusal_receipt_sha256=sha(R/'review-dual-valid-refusal/receipt.json'),prior_schema=cert['schema'],prior_policy=cert['policy'],classification='fresh valid Bool Cargo interface survives rejected new Ord verification with both paths selected; not an accepted new ordering artifact')
write(R/'dual-valid-result.json',result);print(json.dumps(result),flush=True)
# Keep only sealed evidence of the failed invalidation, remove the selected path.
if cargo.exists():shutil.copy2(cargo,R/'dual-valid-retained-prior.vir');cargo.unlink()
need(not cargo.exists() and not explicit.exists(),'control cleanup')
