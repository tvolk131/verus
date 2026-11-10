#!/usr/bin/env python3
"""Freeze candidates before trials; output must be new."""
import hashlib,json,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
if __name__=='__main__':
 out=Path(sys.argv[1]); files=[p for p in HERE.rglob('*') if p.is_file() and (p.suffix in ('.py','.rs','.toml','.lock','.md') or (p.suffix=='.json' and 'candidates' in p.parts))]
 for rel in ['native_iterator_certificate/src/main.rs','native_iterator_certificate/src/ir.rs','native_borrow_entry/src/main.rs','buffer_transfer_goal/full-manual-bounds.rs','mutable_buffer_goal/transfer-freeze.json','buffer_transfer_goal/freeze.json','buffer_transfer_goal/original-selected.rs']:
  files.append(HERE.parent/rel)
 root=HERE.parents[3]
 value=dict(schema=1,before_first_trial=True,files={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)},publication=False)
 with out.open('x') as f: json.dump(value,f,indent=2);f.write('\n')
