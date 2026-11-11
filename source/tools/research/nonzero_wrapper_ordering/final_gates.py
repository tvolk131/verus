"""Final-runtime global failures, Cargo prior and all-selected refusal controls."""
import finalize as f
from controls import *
need(EPOCH=='sealed','sealed runtime')
prior=(R/'final-bool-prior/wrapper.vir').read_bytes();rows={}
bit='assert((0u64 | (1u64 << 1u64)) == 0u64) by(bit_vector);'
expect('proof-wrapper-auxiliary',CAND+'verus!{pub fn auxiliary(){'+bit+'}}',ok=False)
for kind,src in [('false_wrapper',CAND+'verus!{fn false_client(){assert(false);}}'),('auxiliary',BOOL.replace('{u64::from(b)}','{let r=u64::from(b);'+bit+'r}')),('erased',BOOL+'\n#[cfg(not(verus_keep_ghost_body))] fn erased_error()->u64{()}')]:
 for route in ['explicit','cargo','both','equal']:
  label='proof-export-'+kind+'-'+route;d=R/label;d.mkdir(exist_ok=True);c=d/'libwrapper.vir';a=c if route=='equal' else d/'selected.vir';selected=[a] if route=='explicit' else [c] if route=='cargo' else list(dict.fromkeys([a,c]))
  for p in selected:p.write_bytes(prior)
  j=expect(label,src,ok=False,cargo=route!='explicit',export=a if route!='cargo' else None);need(all(not p.exists() for p in selected),'all selected paths removed')
  if kind=='false_wrapper':need('local-only' in (d/'stderr').read_text(),'early local-only gate')
  elif kind=='auxiliary':need(j['inference']['failed_functions']==[] and j['verification']['errors']==1,'auxiliary classification')
  else:need(j['verification']['verified']==2 and j['verification']['errors']==0 and 'E0308' in (d/'stderr').read_text(),'late erasure classification')
  rows[label]=dict(prior_sha256=hashlib.sha256(prior).hexdigest(),removed=list(map(str,selected)))
for kind,opts in [('function',['--verify-root','--verify-function','total']),('module',['--verify-module','sub'])]:
 src=CAND if kind=='function' else 'mod sub{'+CAND+'}'
 for route in ['explicit','both']:
  label='proof-filter-'+kind+'-'+route;d=R/label;d.mkdir(exist_ok=True);a=d/'selected.vir';c=d/'libwrapper.vir';selected=[a,c] if route=='both' else [a]
  for p in selected:p.write_bytes(prior)
  expect(label,src,ok=False,cargo=route=='both',export=a,options=opts);need(all(not p.exists() for p in selected),'filtered selected outputs removed')
  rows[label]=dict(prior_sha256=hashlib.sha256(prior).hexdigest(),removed=list(map(str,selected)))
write(R/'supplement-export-invalidation.json',rows)
expect('proof-cargo-bool-prior',BOOL,n=2,cargo=True)
p=R/'proof-cargo-bool-prior/libwrapper.vir';raw=p.read_bytes();i=raw.rfind(b'{"schema":');cert=json.loads(raw[i:-8]);need(cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v73','Cargo-driver Bool prior')
rows={}
for route in ['explicit','cargo','both','equal']:
 label='proof-cargo-prior-refusal-'+route;d=R/label;d.mkdir(exist_ok=True);c=d/'libwrapper.vir';a=c if route=='equal' else d/'selected.vir';selected=[a] if route=='explicit' else [c] if route=='cargo' else list(dict.fromkeys([a,c]))
 for q in selected:q.write_bytes(raw)
 expect(label,CAND,ok=False,cargo=route!='explicit',export=a if route!='cargo' else None);need('local-only' in (d/'stderr').read_text() and all(not q.exists() for q in selected),'Cargo-driver prior all-selected invalidation')
 rows[label]=dict(prior_sha256=sha(p),prior_schema=cert['schema'],prior_policy=cert['policy'],removed=list(map(str,selected)))
write(R/'cargo-prior-invalidation.json',rows)
