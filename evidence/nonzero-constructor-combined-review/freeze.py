"""Bind final eight acceptance tools to own guarded build and matching vstd."""
import shutil
from pathlib import Path as P
import review as e
W=P(e.read(e.HERE/'work.json')['work']);rows=e.read_epochs(W)
b=next(x for x in reversed(rows) if x['label']=='build');v=next(x for x in rows if x['label']=='vstd')
br=e.read(P(b['path'])/'receipt.json');vr=e.read(P(v['path'])/'receipt.json')
for r in [br,vr]:e.need(r['success'] is True and r['exit']==0 and r['final_guards_passed'] is True,'unaccepted build/vstd')
d=W/'target/release';f=e.read(P(v['path'])/'freeze.json')['inputs']
for n in e.NAMES:
    p=e.BASE/n if n=='z3' else d/n
    expected=(e.sha(e.BASE/n) if n=='z3' else vr['products'][str(p)] if n in ['vstd.vir','libvstd.rlib'] else br['products'][str(p)])
    e.need(e.sha(p)==expected,'build product drift '+n)
    if str(p) in f:e.need(f[str(p)]==expected,'vstd/compiler mismatch '+n)
inputs=e.hashes([d/n for n in e.NAMES if n!='z3']+[e.BASE/'z3',P(b['path'])/'receipt.json',P(v['path'])/'receipt.json',P(__file__),e.HERE/'review.py'])
bundle=W/'final-bundle';bundle.mkdir()
for n in e.NAMES:
    shutil.copyfile(e.BASE/n if n=='z3' else d/n,bundle/n);(bundle/n).chmod(0o500 if n in ['verus','rust_verify','z3'] else 0o400)
e.guard(inputs)
e.write(bundle/'receipt.json',dict(success=True,final_guards_passed=True,inputs=inputs,tools={n:e.sha(bundle/n) for n in e.NAMES},compiler=b,vstd=v,publication=False,goal_complete=False))
print(bundle)
