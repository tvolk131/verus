import hashlib,json,pathlib,subprocess
R=pathlib.Path.cwd(); O=R/'NEW/evidence/published-entry-source-review'; out=[]
def check(path,sha,kind):
 b=pathlib.Path(path).read_bytes(); h=hashlib.sha256(b).hexdigest()
 if h!=sha: raise ValueError(str(path))
 out.append(dict(path=str(path),sha256=h,bytes=len(b),kind=kind))
def tracked(rel,commit='b008f187'):
 b=subprocess.check_output(['git','show',commit+':'+rel]);check(R/rel,hashlib.sha256(b).hexdigest(),'git '+commit)
 return json.loads(b) if rel.endswith('.json') else b
f=tracked('source/tools/research/stbor_entry_handoff/candidate003-freeze.json')
check(R/'source/tools/research/stbor_entry_handoff/candidate.v',f['candidate_sha256'],'candidate freeze')
for n,h in f['upstream_sources'].items():
 p=pathlib.Path('/tmp/native-borrow-foundation-inspection-20261110/stacked-borrows')/n
 check(p,h,'candidate upstream freeze')
 b=subprocess.check_output(['git','-C',str(p.parents[2]),'show',f['upstream_commit']+':'+n])
 if hashlib.sha256(b).hexdigest()!=h: raise ValueError(n+' git')
# Additional definition used for model locations/tags/state; authenticate by the same named Git object.
p=pathlib.Path('/tmp/native-borrow-foundation-inspection-20261110/stacked-borrows')
n='theories/lang/lang_base.v'
b=subprocess.check_output(['git','-C',str(p),'show',f['upstream_commit']+':'+n])
check(p/n,hashlib.sha256(b).hexdigest(),'named upstream Git object')
for rel in ['source/tools/research/native_source_storage/pins.json','source/tools/research/native_source_unsize/pins.json']:
 p=tracked(rel)
 for s in p['rust_sources']:
  # Only explicitly delivered primary files; no historical paths in receipts.
  if pathlib.Path(s['path']).name in ['step.rs','machine.rs','place.rs','stack.rs','syntax.rs','operand.rs','cast.rs','validity.rs']:
   if s['url']!='https://raw.githubusercontent.com/rust-lang/rust/'+p['compiler_commit']+'/'+s['upstream']:raise ValueError('url')
   check(s['path'],s['sha256'],'committed exact compiler pin')
p=tracked('source/tools/research/native_miri_policy_pin/pins.json')
for n,s in p['sources'].items():check(s['path'],s['sha256'],'committed exact Miri pin')
r=tracked('source/tools/research/native_miri_policy_pin/results.json')
check(r['receipt']['path'],r['receipt']['sha256'],'completed source identity receipt')
for rel in ['source/tools/research/stbor_entry_handoff/README.md','source/tools/research/stbor_entry_handoff/status.json','source/tools/research/native_miri_policy_pin/source-freeze.json','source/tools/research/native_miri_policy_pin/REPORT.md','source/tools/research/native_miri_policy_pin/check_sources.py','source/tools/research/native_constructor_erasure/graph.json']:
 tracked(rel)
# Named delivered checkpoint bytes must match final candidate.
for commit in ['136d8f82']:
 b=subprocess.check_output(['git','show',commit+':source/tools/research/stbor_entry_handoff/candidate.v'])
 if hashlib.sha256(b).hexdigest()!=f['candidate_sha256']:raise ValueError('checkpoint')
(O/'authenticated-inputs.json').write_text(json.dumps(dict(classification='source_identity_only',files=out),indent=2)+'\n')
print('authenticated',len(out),'inputs; no semantic acceptance')
