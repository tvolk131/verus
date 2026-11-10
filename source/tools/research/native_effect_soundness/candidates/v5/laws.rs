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


pub open spec fn owner_shape(m: Map<int,PointsTo<u8>>, e: Entry) -> bool {
    &&& forall|i:int| #[trigger] m.dom().contains(i) <==> 0 <= i < 128
    &&& forall|i:int| m.dom().contains(i) ==> slot(#[trigger] m[i],e,i)
}

pub proof fn resolve(tracked c: &mut Carrier, tracked loan: Loan)
    requires old(c).wf(), loan.wf(), loan.entry == old(c).entry,
        old(c).pending.contains(loan.index),
    ensures final(c).wf(), final(c).entry == old(c).entry,
        final(c).physical == old(c).physical, final(c).retained == old(c).retained,
        final(c).pending == old(c).pending.remove(loan.index),
        final(c).returned == old(c).returned.insert(loan.index,loan.token),
{
    c.pending = c.pending.remove(loan.index);
    c.returned.tracked_insert(loan.index,loan.token);
}

// Works on early scope exit as well as at None; no postcondition about XOR.
// This reconstructs the permission map, NOT the native executable array borrow.
pub proof fn reconstruct(tracked c: Carrier) -> (tracked all: Map<int,PointsTo<u8>>)
    requires c.wf(), c.pending.is_empty(),
    ensures owner_shape(all,c.entry),
        all == c.retained.union_prefer_right(c.returned),
        forall|i:int| c.physical.index <= i < 128 ==> (#[trigger] all[i]).value() == c.entry.before[i],
{
    let tracked mut all = c.retained;
    all.tracked_union_prefer_right(c.returned);
    all
}

pub proof fn partition(tracked c: &Carrier, i:int)
    requires c.wf(), 0 <= i < 128,
    ensures
        c.retained.dom().contains(i) || c.pending.contains(i) || c.returned.dom().contains(i),
        !(c.retained.dom().contains(i) && c.pending.contains(i)),
        !(c.retained.dom().contains(i) && c.returned.dom().contains(i)),
        !(c.pending.contains(i) && c.returned.dom().contains(i)),
{}

pub proof fn pointer_separation(e:Entry,i:int,j:int)
    requires e.valid(), 0 <= i < 128, 0 <= j < 128, i != j,
    ensures e.pad[i]@.addr != e.pad[j]@.addr,
{}

pub proof fn retained_tail(tracked c:&Carrier,i:int)
    requires c.wf(), c.physical.index <= i < 128,
    ensures c.retained.dom().contains(i), slot(c.retained[i],c.entry,i),
        c.retained[i].value() == c.entry.before[i],
{}

pub proof fn empty_key_preserves_all(e:Entry,tracked pad:Map<int,PointsTo<u8>>)
    -> (tracked all:Map<int,PointsTo<u8>>)
    requires e.valid(), full_owner(pad,e), e.key.len() == 0,
    ensures full_owner(all,e), all == pad,
{
    let tracked mut c = accept_owner(e,pad);
    none(&mut c);
    let tracked all = reconstruct(c);
    assert(all =~= pad);
    all
}

pub open spec fn projection(e:Entry,p:Physical) -> Seq<(*mut u8,*mut u8)> {
    Seq::new((p.length-p.index) as nat,|j:int| (e.pad[p.index+j],e.reads[p.index+j]))
}

pub proof fn projection_step(e:Entry,p:Physical)
    requires e.valid(), physical_ok(p,e), p.index < p.length,
    ensures projection(e,p)[0] == (e.pad[p.index],e.reads[p.index]),
        projection(e,Physical {index:p.index+1,..p}) == projection(e,p).drop_first(),
{
    assert(projection(e,Physical {index:p.index+1,..p}) =~= projection(e,p).drop_first());
}

pub proof fn constructor_extent(e:Entry,tracked pad:Map<int,PointsTo<u8>>)
    -> (tracked c:Carrier)
    requires e.valid(), full_owner(pad,e),
    ensures c.wf(), c.retained == pad, c.entry == e,
        c.physical.index == 0, c.physical.length == e.count(),
        c.physical.pad_end == e.pad_base@.addr as int + 128,
        c.physical.key_end == e.key_base@.addr as int + e.key.len(),
        c.physical.pad_end <= usize::MAX, c.physical.key_end <= usize::MAX,
{
    accept_owner(e,pad)
}

// These execute only under ALREADY SUPPLIED tokens. They are NOT native getter
// contracts. The actual native call has no permission argument and is unlowered.
pub fn observe_key(p:*const u8,Ghost(e):Ghost<Entry>,Ghost(i):Ghost<int>,
    Tracked(reads):Tracked<&Map<int,PointsTo<u8>>>) -> (b:u8)
    requires e.valid(), key_owner(*reads,e), 0 <= i < e.count(), p == e.reads[i],
    ensures b == e.key[i],
{
    let tracked token = reads.tracked_borrow(i);
    let r = ptr_ref(p,Tracked(token));
    *r
}

pub fn write_loan(p:*mut u8,Tracked(loan):Tracked<&mut Loan>,b:u8)
    requires old(loan).wf(), p == old(loan).token.ptr(),
    ensures final(loan).wf(), final(loan).entry == old(loan).entry,
        final(loan).index == old(loan).index,
        final(loan).token.ptr() == old(loan).token.ptr(), final(loan).token.value() == b,
{
    let r = ptr_mut_ref(p,Tracked(&mut loan.token));
    *r = b;
}

// Unlike a pending receipt, this wrapper physically owns every outstanding token.
pub tracked struct World {
    pub c:Carrier,
    pub loans:Map<int,Loan>,
}
impl World {
    pub open spec fn wf(self) -> bool {
        &&& self.c.wf()
        &&& self.loans.dom() == self.c.pending
        &&& forall|i:int| self.loans.dom().contains(i) ==> (#[trigger] self.loans[i]).wf()
            && self.loans[i].entry == self.c.entry && self.loans[i].index == i
    }
    pub open spec fn inventory(self) -> Map<int,PointsTo<u8>> {
        Map::new(self.c.retained.dom().union(self.c.returned.dom()).union(self.loans.dom()),
            |i:int| if self.c.retained.dom().contains(i) { self.c.retained[i] }
            else if self.c.returned.dom().contains(i) { self.c.returned[i] }
            else { self.loans[i].token })
    }
}

pub proof fn world_owner(tracked w:&World)
    requires w.wf(),
    ensures owner_shape(w.inventory(),w.c.entry),
        forall|i:int| w.c.physical.index <= i < 128 ==> (#[trigger] w.inventory()[i]).value() == w.c.entry.before[i],
{
    assert forall|i:int| w.inventory().dom().contains(i) <==> 0 <= i < 128 by {
        if 0 <= i < 128 { partition(&w.c,i); }
    }
}

pub proof fn world_issue(tracked w:&mut World)
    requires old(w).wf(), old(w).c.physical.index < old(w).c.physical.length,
    ensures final(w).wf(), final(w).inventory() == old(w).inventory(),
        final(w).c.physical.index == old(w).c.physical.index+1,
        final(w).c.entry == old(w).c.entry,
{
    let ghost before = *w;
    let ghost i = w.c.physical.index;
    let tracked loan = issue(&mut w.c);
    w.loans.tracked_insert(i,loan);
    assert(w.inventory() =~= before.inventory());
}

pub proof fn world_resolve(tracked w:&mut World,i:int)
    requires old(w).wf(), old(w).loans.dom().contains(i),
    ensures final(w).wf(), final(w).inventory() == old(w).inventory(),
        final(w).c.physical == old(w).c.physical, final(w).c.entry == old(w).c.entry,
{
    let ghost before = *w;
    let tracked loan = w.loans.tracked_remove(i);
    resolve(&mut w.c,loan);
    assert(w.inventory() =~= before.inventory());
}

} // verus!
