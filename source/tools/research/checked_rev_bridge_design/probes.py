"""Small whole-crate probes of existing semantics, with unchanged runtime chain."""
PRE='''use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
verus! {
broadcast use {vstd::std_specs::iter::group_iter_axioms, vstd::std_specs::iter_adapters::group_native_adapter_axioms};
'''
BODIES={
'available':'''let e=borrows.iter().enumerate();
assert(e.obeys_prophetic_iter_laws()); assert(e.decrease().is_some());
let ghost before=e; let r=e.rev();
assert(r.remaining()==before.remaining().reverse());
assert(r.will_return_none()==before.will_return_none());
assert(r.decrease().is_some());''',
'lawful':'''let r=borrows.iter().enumerate().rev(); assert(r.obeys_prophetic_iter_laws());''',
'inner':'''let e=borrows.iter().enumerate(); let ghost before=e; let r=e.rev();
assert(vstd::std_specs::iter::rev_iter(r)==before);''',
'loop':'''for (_,_) in borrows.iter().enumerate().rev() {}''',
}
def source(name): return PRE+'pub fn probe_'+name+'(borrows: &Vec<u64>) {\n'+BODIES[name]+'\n}\n}\n'
