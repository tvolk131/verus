use vstd::prelude::*;

verus! {
struct Buffer {
    slots: Vec<Option<u16>>,
    epoch: u8,
}

impl Buffer {
    // No contract, invariant, or proof block is supplied for this worker.
    #[verifier::infer]
    fn fill_prefix(&mut self, n: usize, value: u16) {
        let mut i = 0usize;
        while i < n {
            self.slots[i] = Some(value);
            i += 1;
        }
    }

    // The author chooses the error policy. The ordinary verifier checks it
    // against the inferred worker, including complete preservation on Err.
    fn try_fill_prefix(&mut self, n: usize, value: u16) -> (result: Result<(), usize>)
        ensures
            result.is_ok() == (n <= old(self).slots@.len()),
            result.is_err() ==> result->Err_0 == n,
            result.is_err() ==> final(self).slots@ == old(self).slots@,
            final(self).slots@.len() == old(self).slots@.len(),
            final(self).epoch == old(self).epoch,
            result.is_ok() ==> forall|j: int| 0 <= j < old(self).slots@.len() ==>
                #[trigger] final(self).slots@[j] ==
                    if j < n { Some(value) } else { old(self).slots@[j] },
    {
        if n > self.slots.len() {
            return Err(n);
        }
        self.fill_prefix(n, value);
        Ok(())
    }
}

struct Counters { values: Vec<i8>, stamp: u16 }
impl Counters {
    // Inference also derives the per-visited-element overflow requirements.
    #[verifier::infer]
    fn bump(&mut self, n: usize) {
        let mut i = 0usize;
        while i < n {
            self.values[i] += 1;
            i += 1;
        }
    }
}

fn empty_prefix_accepts_every_state(c: &mut Counters)
    ensures final(c).values@ == old(c).values@, final(c).stamp == old(c).stamp,
{
    c.bump(0);
}
}

fn main() {}
