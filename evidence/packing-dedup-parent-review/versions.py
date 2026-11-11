"""Ordinary stale-policy and candidate-cache compatibility controls; own files only."""
from trials import *

def versions():
 original=R/'bool-provider'; prefix,cert=metadata(original/'provider.vir')
 need(cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v72','current export version')
 stale=R/'stale-policy-provider';stale.mkdir()
 shutil.copy2(original/'libprovider.rlib',stale/'libprovider.rlib')
 cert['policy']='candidate-bound-evidence-v71'
 (stale/'provider.vir').write_bytes(prefix+json.dumps(cert,separators=(',',':')).encode()+b'VRINFER1')
 source=BOOL.replace('u64::from(b)','provider::all_bool(b)')
 rec=app('stale-policy-client','stale_policy_client',source,[('provider',stale)])
 require('stale-policy-client',rec,None,success=False)
 need('incompatible inference schema, safety policy, or producer' in (R/'stale-policy-client/stderr').read_text(),'policy rejection diagnostic')
 need(not (R/'stale-policy-client/stale_policy_client.vir').exists(),'stale policy export withheld')
 base=R/'stable-full';cache=R/'cache74-compatibility';shutil.copytree(base/'candidate-cache',cache)
 entries=list(cache.glob('*.json'));need(len(entries)==1,'one Eq candidate cache entry')
 entry=json.loads(entries[0].read_text());need(entry['version']=='verus-infer-v75:checked-buffer-statements:4096','cache75 active')
 entry['version']='verus-infer-v74:checked-buffer-statements:4096';write(entries[0],entry)
 stale_sha=sha(entries[0]);freeze=json.loads((base/'cold/freeze.json').read_text());args=freeze['argv']
 # Keep the exact invocation/source/log/export paths; the completed cold/warm
 # receipts were moved and remain immutable. This creates a fresh run directory.
 rec=run('cache74-rejected',args,{'VERUS_INFER_CACHE':str(cache)})
 require('cache74-rejected',rec,None)
 d=base/'run';j=json.loads((R/'cache74-rejected/stdout').read_text());h=j['verus-infer']['functions'];need(len(h)==1,'same Eq candidate')
 h=h[0];need(h['candidate_id']==json.loads((base/'cold/stdout').read_text())['verus-infer']['functions'][0]['candidate_id'],'stable candidate identity')
 need(h['metrics']['cache']=='miss','cache74 must miss')
 need(h['contract']['requires']=='true' and all(h['proofs'][k]['required'] and h['proofs'][k]['status']=='proved' for k in ['soundness','maximality','exactness']),'fresh full-domain three precision proofs')
 need(json.loads(entries[0].read_text())['version']=='verus-infer-v75:checked-buffer-statements:4096','cache75 regenerated')
 write(R/'version-controls.json',dict(schema=39,policy=72,cache=75,stale_policy_refused=True,cache74_miss=True,fresh_three_precision_proofs=True,stale_cache_sha256=stale_sha,current_cache_sha256=sha(entries[0]),candidate_id=h['candidate_id']))
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'python -B');versions()
