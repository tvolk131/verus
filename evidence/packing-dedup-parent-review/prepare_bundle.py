"""Finalize own bundle before verification: remove debug information only."""
from review import *
need(sys.flags.dont_write_bytecode,'python -B')
r=root();p=r/'bundle/rust_verify';receipt=r/'bundle-preparation.json'
need(not receipt.exists(),'immutable bundle preparation')
before=dict(sha256=sha(p),bytes=p.stat().st_size)
subprocess.run(['/usr/bin/strip','--strip-debug',str(p)],check=True)
write(receipt,dict(command=['/usr/bin/strip','--strip-debug',str(p)],before=before,after=dict(sha256=sha(p),bytes=p.stat().st_size),scope='Remove debug sections before matching vstd and all provider/consumer proofs; unoptimized executable code. No performance inference.'))
print(receipt.read_text())
