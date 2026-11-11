from review import *
need(sys.flags.dont_write_bytecode,'use -B')
r=root()
# Historical freeze paths are labels; source is read exclusively via this checkout's Git.
f=json.loads((OLD/'certificate-build-4/freeze.json').read_text())
paths=['source/rust_verify/src/'+n+'.rs' for n in ['checked_foreign','checked_foreign_packing','commands','driver','import_export','verifier']]+['source/vir/src/infer/registry.rs']
rows={}
for p in paths:
 b=subprocess.check_output(['git','show','3a23cdd5:'+p],cwd=ROOT);h=hashlib.sha256(b).hexdigest();need(h==f['source'][p],'historical source '+p)
 rows[p]=dict(historical_sha256=h,current_sha256=sha(ROOT/p))
# Completed artifact authentication is provenance, never a positive new certificate input.
receipts={p.name:sha(p) for p in [OLD/'RECEIPT.json'] if p.exists()}
parent=Path('/tmp/verus-infer-buffer-cycle-20261109/certificate-parent-delivery-auth/receipt.json')
write(r/'historical-authentication.json',dict(source=rows,historical_freeze_sha256=sha(OLD/'certificate-build-4/freeze.json'),delivery_receipt_sha256=sha(parent),completed_build_receipt_sha256=sha('/tmp/verus-infer-buffer-cycle-20261109/combined71-build-21eeabbb/receipt.json'),arithmetic_acceptance_sha256=sha(ROOT/'evidence/item-packing-review/RESULTS.json'),note='Completed receipt provenance only. No old compiler or .vir used for positives.'))
print('historical source7 authenticated against Git3a23cdd5')
