"""Semantic controls over original executable component bodies, no helper poisoning."""
from pathlib import Path
import json
import tempfile
from annotate import generate, once, block
from source_selection import require
import re
HERE=Path(__file__).resolve().parent

def exact_subset():
 text=generate()
 text=once(text,block(text,'    pub fn new(tag:'),'')
 start=text.index('    pub fn new(i:');end=text.index('    pub fn get(',start)
 text=text[:start]+text[end:]
 for marker in ['    pub fn one(', '    pub fn tag(self)']:
  text=once(text,block(text,marker),'')
 text=once(text,block(text,'impl std::default::Default for BorTag {'),'')
 return re.sub(r'(?m)^[ \t]*///[^\n]*\n','',text)

CASES={
 'nonzero': '''pub fn actual_nonzero(i: Item) -> (r: Item)
     ensures item_word_wf(r.0),
     { i }''',
 'range': '''pub fn actual_range(t: BorTag) -> (r: u64)
     ensures r <= TAG_MASK,
     { t.get() }''',
 'protector': '''pub fn actual_protector(i: Item) -> (r: bool)
     ensures r == !(i.0 & PROTECTED_MASK > 0),
     { i.protected() }''',
 'bitfield': '''pub fn actual_bitfield(i: Item, p: Permission) -> (r: Item)
     ensures (r.0 & TAG_MASK) == ((i.0 & TAG_MASK) ^ 1u64),
     { let mut next = i; next.set_permission(p); next }''',
}
if __name__=='__main__':
 out=Path(tempfile.mkdtemp(prefix='native-miri-tag-actual-controls-',dir='/tmp'));out.chmod(0o700)
 base=exact_subset();require(base.endswith('}\n'),'unexpected wrapper')
 names=json.loads((HERE/'exact-subset.inventory.json').read_bytes())
 for name,body in CASES.items():
  (out/(name+'.rs')).write_text(base[:-2]+body+'\n}\n')
  (out/(name+'.inventory.json')).write_text(json.dumps(names+['actual_'+name])+'\n')
 # Separate missing standard conversion specification from Item constructor effects.
 (out/'conversion.rs').write_text(base[:-2]+'''pub fn bool_conversion(b: bool) -> (r: u64)
     ensures r == (if b { 1u64 } else { 0u64 }),
     { u64::from(b) }
 }\n''')
 (out/'conversion.inventory.json').write_text(json.dumps(names+['bool_conversion'])+'\n')
 print(out)
