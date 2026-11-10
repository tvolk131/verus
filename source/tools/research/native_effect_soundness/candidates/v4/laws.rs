use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo, ptr_ref, ptr_mut_ref};

verus! {

pub ghost struct Entry {
    pub before: Seq<u8>,
    pub key: Seq<u8>,
    pub pad: Seq<*mut u8>,
    pub reads: Seq<*mut u8>,
    pub pad_base: *mut u8,
    pub key_base: *mut u8,
}

impl Entry {
    pub open spec fn count(self) -> int {
        if self.key.len() < 128 { self.key.len() as int } else { 128 }
    }
    pub open spec fn valid(self) -> bool {
        &&& self.before.len() == 128
        &&& self.pad.len() == 128
        &&& self.reads.len() == self.count()
        &&& self.key.len() <= isize::MAX
        &&& self.pad_base@.addr as int + 128 <= usize::MAX
        &&& self.key_base@.addr as int + self.key.len() <= usize::MAX
        &&& forall|i:int| 0 <= i < 128 ==> (
            (#[trigger] self.pad[i])@.addr == self.pad_base@.addr as int + i
            && self.pad[i]@.provenance == self.pad_base@.provenance)
        &&& forall|i:int| 0 <= i < self.count() ==> (
            (#[trigger] self.reads[i])@.addr == self.key_base@.addr as int + i
            && self.reads[i]@.provenance == self.key_base@.provenance)
        &&& forall|i:int,j:int| 0 <= i < 128 && 0 <= j < self.count()
            ==> (#[trigger] self.pad[i])@.addr != (#[trigger] self.reads[j])@.addr
    }
}

pub ghost struct Physical {
    pub pad_base: *mut u8,
    pub key_base: *mut u8,
    pub pad_end: int,
    pub key_end: int,
    pub index: int,
    pub length: int,
}

pub open spec fn physical_ok(p: Physical, e: Entry) -> bool {
    &&& p.pad_base == e.pad_base
    &&& p.key_base == e.key_base
    &&& p.pad_end == e.pad_base@.addr as int + 128
    &&& p.key_end == e.key_base@.addr as int + e.key.len()
    &&& p.length == e.count()
    &&& 0 <= p.index <= p.length
}

pub open spec fn slot(t: PointsTo<u8>, e: Entry, i: int) -> bool {
    &&& 0 <= i < 128
    &&& t.ptr() == e.pad[i]
    &&& t.is_init()
}

pub open spec fn full_owner(m: Map<int,PointsTo<u8>>, e: Entry) -> bool {
    &&& forall|i:int| #[trigger] m.dom().contains(i) <==> 0 <= i < 128
    &&& forall|i:int| m.dom().contains(i) ==> slot(#[trigger] m[i],e,i)
        && m[i].value() == e.before[i]
}

pub open spec fn key_owner(m: Map<int,PointsTo<u8>>, e: Entry) -> bool {
    &&& forall|i:int| #[trigger] m.dom().contains(i) <==> 0 <= i < e.count()
    &&& forall|i:int| m.dom().contains(i) ==> (#[trigger] m[i]).ptr() == e.reads[i]
        && m[i].is_init() && m[i].value() == e.key[i]
}

pub tracked struct Carrier {
    pub ghost entry: Entry,
    pub ghost physical: Physical,
    pub retained: Map<int,PointsTo<u8>>,
    pub returned: Map<int,PointsTo<u8>>,
    pub ghost pending: Set<int>,
}

pub tracked struct Loan {
    pub ghost entry: Entry,
    pub ghost index: int,
    pub token: PointsTo<u8>,
}

impl Loan {
    pub open spec fn wf(self) -> bool { self.entry.valid() && slot(self.token,self.entry,self.index) }
}

impl Carrier {
    pub open spec fn wf(self) -> bool {
        &&& self.entry.valid()
        &&& physical_ok(self.physical,self.entry)
        &&& forall|i:int| #[trigger] self.retained.dom().contains(i) <==> self.physical.index <= i < 128
        &&& forall|i:int| self.retained.dom().contains(i) ==> slot(#[trigger] self.retained[i],self.entry,i)
            && self.retained[i].value() == self.entry.before[i]
        &&& forall|i:int| self.returned.dom().contains(i) ==> slot(#[trigger] self.returned[i],self.entry,i)
        &&& forall|i:int| #[trigger] self.pending.contains(i) ==> 0 <= i < self.physical.index
        &&& forall|i:int| #[trigger] self.returned.dom().contains(i) <==> 0 <= i < self.physical.index && !self.pending.contains(i)
    }
}

pub proof fn accept_owner(e: Entry, tracked pad: Map<int,PointsTo<u8>>) -> (tracked c: Carrier)
    requires e.valid(), full_owner(pad,e),
    ensures c.wf(), c.entry == e, c.physical.index == 0, c.retained == pad,
        c.returned.dom().is_empty(), c.pending.is_empty(),
{
    Carrier { entry:e, physical:Physical { pad_base:e.pad_base, key_base:e.key_base,
        pad_end:e.pad_base@.addr as int + 128, key_end:e.key_base@.addr as int + e.key.len(),
        index:0, length:e.count() }, retained:pad, returned:Map::tracked_empty(), pending:Set::empty() }
}

pub proof fn issue(tracked c: &mut Carrier) -> (tracked loan: Loan)
    requires old(c).wf(), old(c).physical.index < old(c).physical.length,
    ensures final(c).wf(), final(c).entry == old(c).entry,
        final(c).physical == (Physical { index: old(c).physical.index + 1, ..old(c).physical }),
        loan.wf(), loan.entry == old(c).entry, loan.index == old(c).physical.index,
        loan.token == old(c).retained[loan.index],
        final(c).retained == old(c).retained.remove(loan.index), final(c).returned == old(c).returned,
        final(c).pending == old(c).pending.insert(loan.index),
{
    let ghost i = c.physical.index;
    let tracked token = c.retained.tracked_remove(i);
    c.pending = c.pending.insert(i);
    c.physical = Physical { index:i+1, ..c.physical };
    Loan { entry:c.entry, index:i, token }
}

pub proof fn none(tracked c: &mut Carrier)
    requires old(c).wf(), old(c).physical.index == old(c).physical.length,
    ensures *final(c) == *old(c), final(c).wf(),
{}

} // verus!
