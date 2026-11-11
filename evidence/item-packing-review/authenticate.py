"""Authenticate explicitly named completed worker records; not fresh proof acceptance."""
import review as t
import json
j=json.loads((t.ROOT/'source/tools/research/item_packing_goal/RESULTS.json').read_text());receipts={};fixtures={};limits=[]
for label,v in j['receipts'].items():
 d=t.OLD/label
 for n,key in [('receipt.json','receipt_sha256'),('freeze.json','freeze_sha256'),('stdout','stdout_sha256'),('stderr','stderr_sha256')]:t.need(t.sha(d/n)==v[key],label+' '+n)
 f=json.loads((d/'freeze.json').read_text());t.need(f['tools']==v['tools'],'recorded tool IDs '+label)
 for p,h in v.get('fixture_sha256',{}).items():
  t.need(str(p).startswith(str(t.OLD)+'/'),'completed worker fixture only');t.need(t.sha(p)==h,'fixture '+p);fixtures[p]=h
 receipts[label]=v['receipt_sha256']
for n,h in j['tools'].items():t.need(t.sha(t.OLD/'bundle'/n)==h,'final worker tool '+n)
p=t.PARENT/'receipt.json';t.need(t.sha(p)=='f24584b8ccfde3ac92e84c146d87317b04231b2b165ef759411a0c53ea3199e3','accepted parent')
t.write(t.HERE/'HISTORICAL_AUTH.json',dict(status='authenticated_provisional_records_only',receipts=receipts,fixtures=fixtures,final_worker_tools=j['tools'],limitations=['Superseded build/runtime binaries may be unretained; recorded hashes are provenance, not independent fresh compilation evidence.','No historical checkout followed and no worker compiled verifier is used for fresh proof acceptance.']))
print(len(receipts),'receipts,',len(fixtures),'unique fixtures authenticated')
