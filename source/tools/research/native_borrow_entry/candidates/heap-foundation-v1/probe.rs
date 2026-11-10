use vstd::prelude::*;
verus! {
// A small explicit footprint semantics, NOT a runtime permission representation.
// valid_input is a hypothesis about a source heap. No Rust client supplies it yet.
#[derive(PartialEq, Eq)]
pub ghost struct Origin { pub allocation: int, pub array: int, pub base: int }
#[derive(PartialEq, Eq)]
pub enum Holder { Source, Retained, Outstanding, Resolved }
pub ghost struct Heap {
    pub origin: Origin,
    pub source_loan: int,
    pub bytes: Seq<u8>,
    pub holders: Seq<Holder>,
}
pub ghost struct Borrow {
    pub origin: Origin,
    pub source_loan: int,
    pub initial: Seq<u8>,
}
pub ghost struct State {
    pub source: Borrow,
    pub heap: Heap,
    pub index: int,
    pub key_len: nat,
}
pub open spec fn valid_origin(o: Origin) -> bool {
    0 <= o.base && o.base + 128 <= 0xffff_ffff_ffff_ffffint
}
pub open spec fn valid_input(b: Borrow, h: Heap) -> bool {
    valid_origin(b.origin) && h.origin == b.origin && h.source_loan == b.source_loan
    && b.initial.len() == 128 && h.bytes == b.initial && h.holders.len() == 128
    && forall|i: int| 0 <= i < 128 ==> #[trigger] h.holders[i] == Holder::Source
}
pub open spec fn limit(k: nat) -> int { if k < 128 { k as int } else { 128 } }
pub open spec fn wf(s: State) -> bool {
    valid_origin(s.source.origin) && s.heap.origin == s.source.origin
    && s.heap.source_loan == s.source.source_loan
    && s.source.initial.len() == 128 && s.heap.bytes.len() == 128
    && s.heap.holders.len() == 128 && 0 <= s.index <= limit(s.key_len)
    && forall|i: int| 0 <= i < 128 ==> (
        if i < s.index {
            #[trigger] s.heap.holders[i] == Holder::Outstanding
                || s.heap.holders[i] == Holder::Resolved
        } else {
            s.heap.holders[i] == Holder::Retained
                && #[trigger] s.heap.bytes[i] == s.source.initial[i]
        })
}
pub open spec fn enter(b: Borrow, h: Heap, k: nat) -> State {
    State { source: b, heap: Heap { holders: Seq::new(128, |i: int| Holder::Retained), ..h }, index: 0, key_len: k }
}
pub open spec fn issue(s: State) -> State {
    State { index: s.index + 1, heap: Heap { holders: s.heap.holders.update(s.index, Holder::Outstanding), ..s.heap }, ..s }
}
pub open spec fn resolve(s: State, i: int, value: u8) -> State {
    State { heap: Heap { bytes: s.heap.bytes.update(i,value), holders: s.heap.holders.update(i,Holder::Resolved), ..s.heap }, ..s }
}
// finite instruction set: Entry, indexed Issue, Resolve, None, Release.
// No instruction can write a retained or resolved slot; no address casts/retags.
pub open spec fn next(s: State) -> State {
    if s.index < limit(s.key_len) { issue(s) } else { s }
}
pub open spec fn ready(s: State) -> bool {
    forall|i: int| 0 <= i < s.index ==> #[trigger] s.heap.holders[i] == Holder::Resolved
}
pub open spec fn release(s: State) -> Heap {
    Heap { holders: Seq::new(128, |i: int| Holder::Source), ..s.heap }
}
pub open spec fn reconstructs(b: Borrow, h: Heap) -> bool {
    b.origin == h.origin && b.source_loan == h.source_loan && h.bytes.len() == 128
    && forall|i: int| 0 <= i < 128 ==> #[trigger] h.holders[i] == Holder::Source
}
pub proof fn entry_sound(b: Borrow, h: Heap, k: nat)
    requires valid_input(b,h),
    ensures wf(enter(b,h,k)), enter(b,h,k).heap.bytes == b.initial,
        enter(b,h,k).source == b,
{ }
pub proof fn origin_stride_nonalias(b: Borrow, h: Heap, i: int, j: int)
    requires valid_input(b,h), 0 <= i < 128, 0 <= j < 128, i != j,
    ensures b.origin.base + i != b.origin.base + j,
        0 <= b.origin.base+i < 0xffff_ffff_ffff_ffffint,
{ }
pub proof fn issue_sound(s: State)
    requires wf(s), s.index < limit(s.key_len),
    ensures wf(issue(s)), issue(s).heap.bytes == s.heap.bytes,
        issue(s).heap.origin == s.heap.origin,
        issue(s).heap.holders[s.index] == Holder::Outstanding,
        forall|i: int| 0 <= i < 128 && i != s.index ==>
            #[trigger] issue(s).heap.holders[i] == s.heap.holders[i],
{ }
pub proof fn resolve_sound(s: State, i: int, value: u8)
    requires wf(s), 0 <= i < 128, s.heap.holders[i] == Holder::Outstanding,
    ensures wf(resolve(s,i,value)), resolve(s,i,value).heap.bytes[i] == value,
        resolve(s,i,value).heap.holders[i] == Holder::Resolved,
        forall|j: int| 0 <= j < 128 && j != i ==>
            #[trigger] resolve(s,i,value).heap.bytes[j] == s.heap.bytes[j],
{ assert(i < s.index); }
pub proof fn next_sound(s: State)
    requires wf(s),
    ensures wf(next(s)), next(s).heap.bytes == s.heap.bytes,
        s.index >= limit(s.key_len) ==> next(s) == s,
        next(s).heap.origin == s.heap.origin,
{ if s.index < limit(s.key_len) { issue_sound(s); } }
pub proof fn release_sound(s: State)
    requires wf(s), ready(s),
    ensures reconstructs(s.source,release(s)), release(s).bytes == s.heap.bytes,
        forall|i: int| s.index <= i < 128 ==>
            #[trigger] release(s).bytes[i] == s.source.initial[i],
        forall|i: int| 0 <= i < 128 ==> s.heap.holders[i] != Holder::Outstanding,
{ }
pub proof fn empty_key_frame(b: Borrow, h: Heap)
    requires valid_input(b,h),
    ensures next(enter(b,h,0)) == enter(b,h,0),
        release(next(enter(b,h,0))).bytes == b.initial,
        reconstructs(b,release(next(enter(b,h,0)))),
{ entry_sound(b,h,0); next_sound(enter(b,h,0)); release_sound(enter(b,h,0)); }
// Same numeric address does not establish allocation/array/loan identity.
pub proof fn address_is_not_origin(a: Origin, b: Origin)
    requires a.base == b.base, a.allocation != b.allocation || a.array != b.array,
    ensures a != b,
{ }
}
