from matrix import *
p=Path('/tmp/local-nonzero-ord-09t1tshb/source-1adb8b6c2ee64cc1.rs')
h='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d'
f=json.loads(Path('/tmp/local-nonzero-ord-09t1tshb/review-full-proofless-export/freeze.json').read_text())
need(f['fixtures'][str(p)]==h and sha(p)==h,'exact approved Item132 input')
expect('review-full-proofless-export',p.read_text(),ok=True,verified=132,errors=0,export=True)
for p in (R/'review-full-proofless-export/vir').glob('*.vir'):
 need('checked_nonzero_u64_ord%' not in p.read_text(),'excluded wrapper derive activated')
