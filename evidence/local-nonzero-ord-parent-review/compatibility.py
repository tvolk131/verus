from matrix import *
import importlib.util
spec=importlib.util.spec_from_file_location('source_only',ROOT/'source/tools/research/item_tag_constructor_composition/source.py')
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
# This generator reads ONLY these two exact selected authenticated policy pins.
pins=json.loads(s.PIN.read_text())
selected=['src/tools/miri/src/borrow_tracker/mod.rs','src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']
for name in selected:
 p=pins['sources'][name];need(sha(p['path'])==p['sha256'],'selected source pin')
full=s.generate('full');fidelity=s.fidelity(full)
need(sha_text(full)=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','exact full proofless source')
write(R/'full-fidelity.json',fidelity)
expect('review-full-proofless-export',full,ok=True,verified=132,errors=0,export=True)
for p in (R/'review-full-proofless-export/vir').glob('*.vir'):
 need('checked_nonzero_u64_ord%' not in p.read_text(),'excluded wrapper derive activated ordering path')
