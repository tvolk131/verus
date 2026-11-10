"""Unchanged genuine policy68 provider: logical import gate, no substitutions."""
import json
from pathlib import Path as P
import review as e
W=P(e.read(e.HERE/'work.json')['work']);B=W/'final-bundle'
old=P('/tmp/nonzero-owned-macro-4okwhwsz');tools=P('/tmp/nonzero-derive-frozen-tools-8hnei2sw')
d=e.private('old-policy-input');source=d/'consumer.rs';source.write_text('use vstd::prelude::*;\nverus! {fn goal(){}}\n')
argv=[B/'verus',source,'--crate-type=lib','--crate-name=consumer','--no-cheating','--infer-goals','--output-json','--extern','component='+str(old/'libcomponent.rlib'),'--import','component='+str(old/'component.vir'),'-L',old,'-L',tools]
out,r=e.epoch(W,'old-policy-interface',argv,extra=[source,old/'receipt.json',old/'component.vir',old/'libcomponent.rlib']+[B/n for n in e.NAMES]+[tools/n for n in e.NAMES],cwd=d,timeout=120)
err=(out/'stderr').read_text();j=e.read(out/'stdout')
e.need(r['exit']!=0 and 'incompatible inference schema, safety policy, or producer' in err,'not unchanged certificate rejection')
e.need(j['verus-infer']['interface_written'] is False and not j['verus-infer']['functions'],'unexpected publication/helper')
v=dict(classification='unchanged_old_policy_certificate_rejection',acceptance_checks_passed=True,verification=j['verification-results'],provider_sha256=e.sha(old/'component.vir'),provider_unchanged=True,source_sha256=e.sha(source),client_uses_provider_symbols=False,policy_or_producer_gate=True)
e.write(out/'validation.json',v)
with (W/'validations.jsonl').open('a') as f:f.write(json.dumps(dict(path=str(out),validation_sha256=e.sha(out/'validation.json')))+'\n')
print(v)
