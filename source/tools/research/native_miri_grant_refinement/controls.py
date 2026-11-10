"""Generate only the three semantic controls in fresh private outputs."""
import json, pathlib, tempfile
HERE=pathlib.Path(__file__).resolve().parent
CONTROLS={'wrong_permission': 'fn wrong_permission(w: u64) {\n    let p = Item(w).perm(); assert(p == Permission::Unique);\n}\n', 'wrong_boundary': 'fn wrong_boundary(s: &Stack, g: usize)\n    requires g + 1 < s.borrows@.len(),\n        s.borrows@[g as int].permission() == Permission::SharedReadWrite,\n        s.borrows@[g as int + 1].permission() == Permission::SharedReadWrite,\n{ let r = s.find_first_write_incompatible(g); assert(r == g + 1); }\n', 'wrong_bitfield': 'fn wrong_bitfield(w: u64, p: Permission) {\n    let mut item = Item(w); item.set_permission(p);\n    assert(item.0 & PERM_MASK == w & PERM_MASK);\n}\n'}
def main():
    private=pathlib.Path(json.loads((HERE/'FREEZE.json').read_text())['private_output'])
    out=pathlib.Path(tempfile.mkdtemp(prefix='controls-',dir=private))
    base=(HERE/'component.rs').read_text().rstrip()[:-1]
    for name,body in CONTROLS.items():
        path=out/(name+'.rs');path.write_text(base+body+'}\n');print(path)
if __name__=='__main__':main()
