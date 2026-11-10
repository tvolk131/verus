"""False semantic sequents, with the identical healthy bitfield helper bodies."""
import json
from pathlib import Path
import tempfile
from source_selection import require
HERE=Path(__file__).resolve().parent
CONTROLS={
 'nonzero': ('w: u64', '', 'item_word_wf(w)', ''),
 'range': ('t: u64', 'requires t != 0,', 't <= TAG_MASK', 'tag_range(t);'),
 'protector': ('t: u64, p: u64, protected: bool', 'requires 0 < t <= TAG_MASK, p <= 3,',
               '(packed(t,p,protected) & PROTECTED_MASK > 0) == !protected', 'packed_fields(t,p,protected);'),
 'bitfield': ('w: u64, p: u64', 'requires p <= 3,',
              '(updated(w,p) & TAG_MASK) == ((w & TAG_MASK) ^ 1u64)', 'updated_fields(w,p);'),
}
if __name__=='__main__':
 out=Path(tempfile.mkdtemp(prefix='native-miri-tag-controls-',dir='/tmp'));out.chmod(0o700)
 base=(HERE/'bitfields.rs').read_text();require(base.endswith('}\n'),'wrapper mismatch')
 names=json.loads((HERE/'bitfields.inventory.json').read_bytes())
 for name,(args,req,post,body) in CONTROLS.items():
  (out/(name+'.rs')).write_text(base[:-2]+f'pub proof fn control_{name}({args})\n {req}\n ensures {post},\n {{ {body} }}\n}}\n')
  (out/(name+'.inventory.json')).write_text(json.dumps(names+['control_'+name])+'\n')
 print(out)
