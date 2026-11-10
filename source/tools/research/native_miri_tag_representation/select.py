"""Freeze exact primary-source closure; produce private literal Rust, never a model."""
import hashlib
import json
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
PINS = HERE.parent / 'native_miri_policy_pin/pins.json'
NAMES = ['src/tools/miri/src/borrow_tracker/mod.rs',
         'src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']

def require(ok, message):
    if not ok:
        raise ValueError(message)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def block(text, marker):
    require(text.count(marker) == 1, 'ambiguous marker: ' + marker)
    start = text.index(marker)
    brace = text.index('{', start)
    depth = 0
    for i in range(brace, len(text)):
        depth += (text[i] == '{') - (text[i] == '}')
        if depth == 0:
            return text[start:i+1]
    raise ValueError('unclosed block')

def selection():
    pins = json.loads(PINS.read_bytes())
    require(pins['compiler_commit'] == '58cf8f9c35b355199727627d07ae34a4e2846d67', 'wrong compiler')
    texts = []
    sources = {}
    for name in NAMES:
        entry = pins['sources'][name]
        data = Path(entry['path']).read_bytes()
        require(sha(data) == entry['sha256'], 'upstream drift')
        texts.append(data.decode())
        sources[name] = entry
    bor, item = texts
    pieces = {
        'BorTag': '#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\n' + re.search(r'pub struct BorTag\([^;]+;', bor)[0],
        'BorTag_impl': block(bor, 'impl BorTag {'),
        'Default': block(bor, 'impl std::default::Default for BorTag {'),
        'Item': '#[derive(Copy, Clone, Hash, PartialEq, Eq)]\n' + re.search(r'pub struct Item\([^;]+;', item)[0],
        'constants': item[item.index('const TAG_MASK:'):item.index('impl Item {')].strip(),
        'Item_impl': block(item, 'impl Item {'),
        'Permission': '#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]\n' + block(item, 'pub enum Permission {'),
        'Permission_impl': block(item, 'impl Permission {'),
    }
    literal = 'use std::num::NonZero;\n' + '\n\n'.join(pieces.values()) + '\n'
    meta = dict(schema=1, compiler_commit=pins['compiler_commit'], sources=sources,
                selected={k:dict(sha256=sha(v.encode()), bytes=len(v.encode())) for k,v in pieces.items()},
                literal_sha256=sha(literal.encode()),
                omitted=['BorTag fmt::Debug', 'Item fmt::Debug', 'unrelated module bodies'],
                domains={'BorTag.new':'all u64', 'BorTag':'all genuine NonZero<u64>',
                         'BorTag.succ':'all NonZero<u64>, MAX returns None',
                         'Item.new':'all BorTag/Permission/bool; existing assert for tag > TAG_MASK',
                         'Item.tag':'normal return only when low 61 bits nonzero',
                         'Item.set_permission':'all u64 words/all Permission'},
                native_correspondences=0, publication=False, goal_complete=False)
    return meta, literal

if __name__ == '__main__':
    meta, literal = selection()
    if sys.argv[1:] == ['--freeze']:
        target = HERE / 'selection.json'
        require(not target.exists(), 'selection already frozen')
        target.write_text(json.dumps(meta, indent=2)+'\n')
    else:
        require(meta == json.loads((HERE/'selection.json').read_bytes()), 'selection changed')
        target = Path(sys.argv[1])
        require(not target.exists(), 'refuse overwrite')
        target.write_text(literal)
