//! Bounded ownership bridge for a fresh global-allocator Vec<u8> spare buffer.
//! Primitive boundaries are allocation/acquisition, in-bounds pointer arithmetic,
//! and transfer into Vec. Writes use the existing tracked raw-pointer primitive.
use crate::prelude::*;
use crate::raw_ptr::{PointsTo, PointsToRaw, PtrData, ptr_mut_from_data};
use alloc::vec::Vec;

verus! {
pub uninterp spec fn allocation_id(v:&Vec<u8>)->int;
pub uninterp spec fn allocation_base(v:&Vec<u8>)->*mut u8;
pub uninterp spec fn allocation_capacity(v:&Vec<u8>)->usize;
// Ghost protocol generation, incremented by each exclusive acquisition.
pub uninterp spec fn allocation_epoch(v:&Vec<u8>)->int;

// Only the allocation boundary can produce this noncopyable lease.
#[verifier::external_body]
pub tracked struct AllocationLease { no_copy: NoCopy }
pub uninterp spec fn lease_id(lease:AllocationLease)->int;

pub tracked struct ByteSpare<'a> {
    borrow: core::marker::PhantomData<&'a mut Vec<u8>>,
    ghost epoch: int,
    lease: AllocationLease,
    ghost base: *mut u8,
    ghost capacity: usize,
    ghost contents: Seq<u8>,
    raw: PointsToRaw,
    cells: Map<int,PointsTo<u8>>,
}
impl<'a> ByteSpare<'a> {
    pub closed spec fn base(&self)->*mut u8 {self.base}
    pub closed spec fn capacity(&self)->usize {self.capacity}
    pub closed spec fn contents(&self)->Seq<u8> {self.contents}
    pub closed spec fn matches(&self,v:&Vec<u8>)->bool {
        allocation_id(v)==lease_id(self.lease) && allocation_base(v)==self.base &&
        allocation_capacity(v)==self.capacity && allocation_epoch(v)==self.epoch && v@.len()==0
    }
    pub open spec fn pointer(&self,p:*mut u8,i:int)->bool {
        0<=i<=self.capacity() && p@.addr as int==self.base()@.addr as int+i &&
        p@.provenance==self.base()@.provenance && p@.metadata==()
    }
    pub closed spec fn wf(&self)->bool {
        &&& self.base@.addr>0
        &&& self.base@.addr as int+self.capacity<=usize::MAX
        &&& self.capacity<=isize::MAX
        &&& self.contents.len()<=self.capacity
        &&& self.raw.provenance()==self.base@.provenance
        &&& self.raw.is_range(self.base@.addr as int+self.contents.len(),self.capacity as int-self.contents.len())
        &&& self.cells.dom()==Set::range(0,self.contents.len() as int)
        &&& forall|i:int| 0<=i<self.contents.len() ==>
            #[trigger] self.cells[i].ptr()==ptr_mut_from_data::<u8>(PtrData {addr:(self.base@.addr+i) as usize,provenance:self.base@.provenance,metadata:()}) &&
            self.cells[i].is_init() && self.cells[i].value()==self.contents[i]
    }
}

/// Trusted fresh Vec allocation boundary. Capacity may exceed the request.
#[verifier::external_body]
#[cfg_attr(verus_keep_ghost, rustc_diagnostic_item = "verus::vstd::byte_vec::allocate")]
pub fn allocate(request:usize)->(v:Vec<u8>)
    ensures v@==Seq::<u8>::empty(),allocation_capacity(&v)>=request,
        request==0 ==> allocation_capacity(&v)==0,
{Vec::with_capacity(request)}

/// Exclusive acquisition lends the Vec allocation for the lease's lifetime.
/// The epoch is ghost protocol state, not an additional native Vec field.
#[verifier::external_body]
#[cfg_attr(verus_keep_ghost, rustc_diagnostic_item = "verus::vstd::byte_vec::acquire")]
pub fn acquire<'a>(v:&'a mut Vec<u8>)->(r:(*mut u8,Tracked<ByteSpare<'a>>))
    requires old(v)@.len()==0,
    ensures final(v)@==old(v)@,r.1@.wf(),r.1@.matches(final(v)),
        r.1@.contents()==Seq::<u8>::empty(),r.1@.capacity()==allocation_capacity(old(v)),
        allocation_id(final(v))==allocation_id(old(v)),
        allocation_base(final(v))==allocation_base(old(v)),
        allocation_capacity(final(v))==allocation_capacity(old(v)),
        allocation_epoch(final(v))==allocation_epoch(old(v))+1,
        r.0==r.1@.base(),r.1@.pointer(r.0,0),
    no_unwind
{(v.as_mut_ptr(),Tracked::assume_new())}

/// Checked splitting, initialization and preservation of the exclusive prefix.
#[cfg_attr(verus_keep_ghost, rustc_diagnostic_item = "verus::vstd::byte_vec::write") ]
pub fn write<'a>(p:*mut u8,value:u8,Tracked(spare):Tracked<&mut ByteSpare<'a>>)
    requires old(spare).wf(),old(spare).pointer(p,old(spare).contents().len() as int),
        old(spare).contents().len()<old(spare).capacity(),
    ensures final(spare).wf(),final(spare).base()==old(spare).base(),
        final(spare).capacity()==old(spare).capacity(),
        forall|v:&Vec<u8>| old(spare).matches(v) ==> final(spare).matches(v),
        final(spare).contents()==old(spare).contents().push(value),
    no_unwind
{
    broadcast use crate::group_vstd_default;
    broadcast use crate::layout::group_layout_axioms;
    broadcast use crate::layout::align_of_u8;
    broadcast use crate::raw_ptr::group_raw_ptr_axioms;
    let ghost before=*spare;
    proof {assert(before.wf());}
    let ghost k=spare.contents.len() as int;
    let tracked mut cell={
        let tracked mut raw=PointsToRaw::empty(spare.base@.provenance);
        crate::modes::tracked_swap(&mut raw,&mut spare.raw);
        let ghost address=spare.base@.addr as int+k;
        assert(raw.is_range(address,spare.capacity as int-k));
        assert forall|a:int| Set::<int>::range(address,address+1).contains(a)
            implies raw.dom().contains(a) by {}
        let tracked (slot,rest)=raw.split(Set::range(address,address+1));
        spare.raw=rest;
        assert(crate::layout::align_of::<u8>()==1);
        slot.into_typed::<u8>(address as usize)
    };
    proof {
        assert(p==cell.ptr());
        assert(p==ptr_mut_from_data::<u8>(PtrData {addr:(spare.base@.addr+k) as usize,provenance:spare.base@.provenance,metadata:()}));
    }
    crate::raw_ptr::ptr_mut_write(p,Tracked(&mut cell),value);
    proof {
        spare.cells.tracked_insert(k,cell);
        spare.contents=spare.contents.push(value);
        assert(spare.raw.is_range(spare.base@.addr as int+spare.contents.len(),spare.capacity as int-spare.contents.len())) by {
            assert(spare.raw.dom() =~= Set::range(spare.base@.addr as int+spare.contents.len(),spare.base@.addr as int+spare.capacity));
        }
        assert(spare.cells.dom() =~= Set::range(0,spare.contents.len() as int));
        assert forall|i:int| 0<=i<spare.contents.len() implies
            #[trigger] spare.cells[i].ptr()==ptr_mut_from_data::<u8>(PtrData {addr:(spare.base@.addr+i) as usize,provenance:spare.base@.provenance,metadata:()}) &&
            spare.cells[i].is_init() && spare.cells[i].value()==spare.contents[i] by {
            if i<k {assert(before.cells.dom().contains(i));} else {
                assert(i==k);
                assert(spare.cells[i].ptr()==p);
            }
        }
        assert(spare.contents.len()<=spare.capacity);
        assert(spare.raw.provenance()==spare.base@.provenance);
        assert(spare.base==before.base);
        assert(spare.capacity==before.capacity);
        assert(spare.base@.addr>0);
        assert(spare.base@.addr as int+spare.capacity<=usize::MAX);
        assert(spare.capacity<=isize::MAX);
        reveal(ByteSpare::wf);
        assert(spare.wf());
    }
}

/// Trusted standard pointer arithmetic, bounded by this same live allocation.
#[verifier::external_body]
#[cfg_attr(verus_keep_ghost, rustc_diagnostic_item = "verus::vstd::byte_vec::advance") ]
pub fn advance<'a>(p:*mut u8,n:usize,Tracked(spare):Tracked<&ByteSpare<'a>>)->(q:*mut u8)
    requires spare.wf(),spare.pointer(p,p@.addr as int-spare.base()@.addr),
        p@.addr as int-spare.base()@.addr+n<=spare.capacity(),
    ensures q@==(PtrData {addr:(p@.addr+n) as usize,provenance:p@.provenance,metadata:()}),
        spare.pointer(q,p@.addr as int-spare.base()@.addr+n),
    no_unwind
{unsafe {p.add(n)}}

/// A sealed initialization receipt carries no dereference permission or Vec
/// borrow. Reacquisition changes the Vec epoch and invalidates old receipts.
pub tracked struct SealReceipt {
    ticket: AllocationLease,
    ghost base: *mut u8,
    ghost capacity: usize,
    ghost epoch: int,
    ghost contents: Seq<u8>,
}
impl SealReceipt {
    pub closed spec fn wf(&self)->bool {
        self.contents.len()<=self.capacity && self.capacity<=isize::MAX &&
        self.base@.addr>0 && self.base@.addr as int+self.capacity<=usize::MAX
    }
    pub closed spec fn contents(&self)->Seq<u8> {self.contents}
    pub closed spec fn matches(&self,v:&Vec<u8>)->bool {
        v@.len()==0 && allocation_id(v)==lease_id(self.ticket) &&
        allocation_base(v)==self.base && allocation_capacity(v)==self.capacity &&
        allocation_epoch(v)==self.epoch
    }
}

/// Relinquish the raw/cell permissions and end the exclusive borrow. For u8,
/// discarding the unused initialized suffix requires no destructor obligations.
pub proof fn seal<'a>(tracked spare:ByteSpare<'a>)->(tracked receipt:SealReceipt)
    requires spare.wf(),
    ensures receipt.wf(),receipt.contents()==spare.contents(),
        forall|v:&Vec<u8>| spare.matches(v) ==> receipt.matches(v),
{
    SealReceipt {ticket:spare.lease,base:spare.base,capacity:spare.capacity,
        epoch:spare.epoch,contents:spare.contents}
}

/// Trusted set_len transfer. The receipt certifies initialized bytes in this
/// exact allocation/generation. It is consumed, and grants no later writes.
#[verifier::external_body]
#[cfg_attr(verus_keep_ghost, rustc_diagnostic_item = "verus::vstd::byte_vec::finish")]
pub fn finish(v:&mut Vec<u8>,n:usize,Tracked(receipt):Tracked<SealReceipt>)
    requires receipt.wf(),receipt.matches(old(v)),n<=receipt.contents().len(),
    ensures final(v)@==receipt.contents().take(n as int),
        allocation_id(final(v))==allocation_id(old(v)),
        allocation_base(final(v))==allocation_base(old(v)),
        allocation_capacity(final(v))==allocation_capacity(old(v)),
        allocation_epoch(final(v))==allocation_epoch(old(v)),
    no_unwind
{unsafe {v.set_len(n)}}

/// An abandoned byte lease returns the zero-length allocation to native Vec
/// drop. No pointer/cell permission survives; u8 has no destructor resources.
pub proof fn abandon<'a>(tracked spare:ByteSpare<'a>) requires spare.wf(), {}

// Impossible-domain carriers. Only a direct call with a compiler-validated
// original core/alloc function identity can lower to the resource-checked API.
// Indirect or forged calls retain requires false and grant no permission.
// Their erased bodies still execute the corresponding primitive, so even a
// direct validated carrier call preserves the checked primitive behavior.
#[verifier::external_body]
pub fn native_allocate<F>(_original:F,request:usize)->(r:Vec<u8>)
    requires false,
{Vec::with_capacity(request)}
#[verifier::external_body]
pub fn native_acquire<'a,F>(_original:F,v:&'a mut Vec<u8>)->(r:(*mut u8,Tracked<ByteSpare<'a>>))
    requires false,
{(v.as_mut_ptr(),Tracked::assume_new())}
#[verifier::external_body]
pub fn native_write<'a,F>(_original:F,p:*mut u8,value:u8,spare:Tracked<&mut ByteSpare<'a>>)
    requires false,
{unsafe {core::ptr::write(p,value)}}
#[verifier::external_body]
pub fn native_advance<'a,F>(_original:F,p:*mut u8,n:usize,spare:Tracked<&ByteSpare<'a>>)->(q:*mut u8)
    requires false,
{unsafe {p.add(n)}}
#[verifier::external_body]
pub fn native_finish<F>(_original:F,v:&mut Vec<u8>,n:usize,spare:Tracked<SealReceipt>)
    requires false,
{unsafe {v.set_len(n)}}
pub open spec fn successful_prefix<R,E>(results:Seq<Result<R,E>>,n:int)->bool {
    forall|i:int| 0<=i<n ==> #[trigger] results[i].is_ok()
}

/// Transfer a scalar validity predicate from a complete two-byte block to
/// either indexed byte. This is ordinary checked quantifier/division support,
/// independent of ASCII, any lookup table, any iterator, and any decoder.
pub proof fn whole_pair_validity<A,R,E>(bytes:Seq<A>,results:Seq<Result<R,E>>,byte_ok:spec_fn(A)->bool)
    requires results.len()==bytes.len()/2,
        forall|k:int| 0<=k<results.len() && #[trigger] results[k].is_ok() ==>
            byte_ok(bytes[2*k]) && byte_ok(bytes[2*k+1]),
    ensures forall|k:int| 0<=k<=results.len() && #[trigger] successful_prefix(results,k) ==>
        forall|j:int| 0<=j<2*k ==> byte_ok(#[trigger] bytes[j]),
{
    broadcast use crate::group_vstd_default;
    assert forall|k:int| 0<=k<=results.len() && #[trigger] successful_prefix(results,k)
        implies forall|j:int| 0<=j<2*k ==> byte_ok(#[trigger] bytes[j]) by {
        assert forall|j:int| 0<=j<2*k implies byte_ok(#[trigger] bytes[j]) by {
            let i=j/2;
            assert(0<=i<k);
            assert(results[i].is_ok());
            assert(2*i<=j<2*i+2);
            assert(byte_ok(bytes[2*i]) && byte_ok(bytes[2*i+1]));
            if j==2*i {} else {assert(j==2*i+1);}
        }
    }
}

}
