// rust_verify/tests/example.rs expect-success --infer-goals
//! Run with: verus verus_infer_growth.rs --infer-goals --no-cheating
use vstd::prelude::*;

verus! {
struct Snapshot { items: Vec<u8>, round: u16 }

// A reusable, ordinary checked transition contract.
fn advance(previous: &Snapshot) -> (result: Snapshot)
    requires previous.round < u16::MAX,
    ensures result.round == previous.round + 1,
        result.items@ == previous.items@,
{
    Snapshot { items: previous.items.clone(), round: previous.round + 1 }
}

struct History { snapshots: Vec<Snapshot>, stamp: u8 }
impl History {
    // The application author chooses the meaning of a valid history.
    closed spec fn valid(&self) -> bool {
        &&& 0 < self.snapshots@.len() <= u16::MAX
        &&& forall|i: int| #![trigger self.snapshots@[i]]
            0 <= i < self.snapshots@.len() ==> self.snapshots@[i].round == i + 1
    }

    // The author supplies the behavior goal; --infer-goals supplies checked
    // loop invariants and termination. This is an ordinary application proof,
    // not an exact inferred contract for a state machine.
    fn ensure_round(&mut self, round: u16)
        requires old(self).valid(), round < u16::MAX,
        ensures final(self).valid(), final(self).stamp == old(self).stamp,
            final(self).snapshots@.len() ==
                if old(self).snapshots@.len() > round as int + 1 {
                    old(self).snapshots@.len() as int
                } else { round as int + 1 },
            final(self).snapshots@.take(old(self).snapshots@.len() as int)
                == old(self).snapshots@,
    {
        while self.snapshots.len() <= round as usize {
            let next = advance(self.snapshots.last().unwrap());
            self.snapshots.push(next);
        }
    }
}
}
fn main() {}
