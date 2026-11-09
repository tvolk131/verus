use vstd::prelude::*;

verus! {

pub struct WriteBack<'a> {
    pub target: &'a mut u64,
    pub value: u64,
}

impl<'a> Drop for WriteBack<'a> {
    fn drop(&mut self)
        ensures *final(self).target == old(self).value,
        opens_invariants none
        no_unwind
    {
        *self.target = self.value;
    }
}

pub fn normal_scope_writeback(target: &mut u64)
    ensures *final(target) == 7,
{
    {
        let guard = WriteBack { target, value: 7 };
    }
}

}
