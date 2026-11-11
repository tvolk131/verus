"""Read only permitted completed epochs and rebind historical source to own Git."""
import hashlib,json,pathlib,subprocess
import build
r=pathlib.Path((build.HERE/'PRIVATE_ROOT').read_text().strip())
w=pathlib.Path('/tmp/rev-stage-a-05c25_w6')
j=json.loads((build.ROOT/'source/tools/research/rev_nominal_definition/stage_a/RESULTS.json').read_text())
records={}
for name,record in j['epochs'].items():
 d=w/'epochs'/name
 build.need(build.sha(d/'receipt.json')==record['receipt_sha256'],'receipt '+name)
 receipt=json.loads((d/'receipt.json').read_text())
 build.need(build.sha(d/'freeze.json')==receipt['freeze_sha256'],'freeze '+name)
 for n in ['stdout','stderr']:build.need(build.sha(d/n)==receipt[n+'_sha256'],'log '+name)
 records[name]=record
source=json.loads((w/'epochs/build-04/freeze.json').read_text())['source']
for p,h in source.items():
 data=subprocess.check_output(['git','show','38be1482:'+p],cwd=build.ROOT)
 build.need(hashlib.sha256(data).hexdigest()==h,'own Git38be1482 '+p)
# Do not compare worker source to changed parent epochs; record the exact delta.
current=build.shipping()
delta={p:{'worker':h,'parent':current.get(p)} for p,h in source.items() if current.get(p)!=h}
build.write(r/'worker-authentication.json',dict(worker='38be1482',epochs=records,source_files=len(source),source_delta=delta,no_historical_live_checkout_read=True))
print('authenticated worker epochs and own Git source; parent delta',len(delta))
