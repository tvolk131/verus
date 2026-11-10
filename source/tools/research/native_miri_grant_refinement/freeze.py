"""Authenticate delivered immutable inputs; select literal component before trials."""
import hashlib, json, pathlib, tempfile
ROOT = pathlib.Path(__file__).resolve().parent
RESEARCH = ROOT.parent
BUNDLE = pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
RECEIPT = pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
def require(ok):
    if not ok: raise ValueError("immutable input authentication failed")
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def function(text, name):
    import re
    m = re.search(r'(?:pub )?fn '+name+r'\b', text)
    start = text.index('{', m.start()); depth = 1; end = start + 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}'); end += 1
    return text[m.start():end]
def main():
    if (ROOT/'FREEZE.json').exists(): raise SystemExit('already frozen')
    require(sha(RECEIPT) == '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a')
    receipt = json.loads(RECEIPT.read_text()); pins = json.loads((RESEARCH/'native_miri_policy_pin/pins.json').read_text())
    private = pathlib.Path(tempfile.mkdtemp(prefix='native-miri-grant-refinement-'))
    (private/'upstream').mkdir()
    sources = {}
    for name in ['mod.rs', 'stack.rs', 'item.rs']:
        entry = pins['sources']['src/tools/miri/src/borrow_tracker/stacked_borrows/'+name]
        p = pathlib.Path(entry['path']); require(sha(p) == entry['sha256'])
        (private/'upstream'/name).write_bytes(p.read_bytes()); sources[name] = entry
    tools = {}
    for name in ['verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','z3']:
        p = BUNDLE/name; expected = receipt['artifacts'][str(p)]['sha256']; require(sha(p) == expected)
        tools[str(p)] = {'sha256':expected, 'size':p.stat().st_size}
    it = (private/'upstream/item.rs').read_text(); st = (private/'upstream/stack.rs').read_text(); mo = (private/'upstream/mod.rs').read_text()
    funcs = {'find_first_write_incompatible':function(mo,'find_first_write_incompatible'), 'get':function(st,'get'), 'perm':function(it,'perm'), 'from_bits':function(it,'from_bits'), 'to_bits':function(it,'to_bits'), 'set_permission':function(it,'set_permission'), 'protected':function(it,'protected')}
    (ROOT/'selected.json').write_text(json.dumps(funcs,indent=2)+'\n')
    # Exact selected executable bodies. Only carrier projection, diagnostic macro,
    # unused impl lifetime, and derive inventory are adapted here.
    s = '''use vstd::prelude::*;
macro_rules! bug { ($($arg:tt)*) => { panic!($($arg)*) }; }
verus! {
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Item(u64);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Permission { Unique, SharedReadWrite, SharedReadOnly, Disabled }
const PERM_MASK: u64 = 0x3 << 61;
const PROTECTED_MASK: u64 = 0x1 << 63;
const PERM_SHIFT: u64 = 61;
// Explicit projection adapter: only borrows is observed by this component.
pub struct Stack { borrows: Vec<Item> }
impl Permission {
    const UNIQUE: u64 = 0;
    const SHARED_READ_WRITE: u64 = 1;
    const SHARED_READ_ONLY: u64 = 2;
    const DISABLED: u64 = 3;
'''+funcs['to_bits']+'\n'+funcs['from_bits']+'''\n}
impl Item {
'''+ '\n'.join(funcs[k] for k in ['perm','protected','set_permission'])+'''\n}
impl Stack {
'''+funcs['get']+'\n'+funcs['find_first_write_incompatible']+'\n}\n}\n'
    (ROOT/'literal.rs').write_text(s)
    freeze = {'schema':1,'private_output':str(private),'compiler_commit':pins['compiler_commit'],'sources':sources,'tools':tools,'receipt_sha256':sha(RECEIPT),'selected_sha256':sha(ROOT/'selected.json'),'literal_sha256':sha(ROOT/'literal.rs'),'native_correspondences':0,'publication':False,'goal_complete':False}
    (ROOT/'FREEZE.json').write_text(json.dumps(freeze,indent=2)+'\n')
    print(private)
if __name__ == '__main__': main()
