use vstd::prelude::*;
verus! {

// MODEL only: none of these definitions are installed as native backend summaries.
pub struct Write { pub offset: int, pub data: Seq<u8> }
pub struct Storage {
    pub volatile: Seq<u8>, pub persistent: Seq<u8>,
    pub base: Seq<u8>, pub pending: Seq<Write>,
}
pub struct Candidate { pub transaction: nat, pub required: Map<int, u8> }
pub enum Phase { Preparing, FirstBarrier, Publishing, SecondBarrier, Acknowledged, Failed, Crashed }
pub enum Event {
    Buffer(Write), Submit(Write), Persist(Seq<u8>), SyncSuccess,
    SyncError(Seq<u8>), Crash(Seq<u8>), Ack, ReturnError,
}
pub struct Protocol {
    pub storage: Storage, pub buffered: Seq<Write>, pub phase: Phase,
    pub previous_two_phase: bool, pub last_ack: Option<nat>,
}

pub open spec fn valid_write(w: Write, n: nat) -> bool {
    0 <= w.offset && w.offset + w.data.len() <= n
}
pub open spec fn covers(w: Write, i: int) -> bool {
    w.offset <= i < w.offset + w.data.len()
}
pub open spec fn apply_write(v: Seq<u8>, w: Write) -> Seq<u8> {
    Seq::new(v.len(), |i: int| if covers(w, i) { w.data[i-w.offset] } else { v[i] })
}
pub open spec fn allowed(s: Storage, p: Seq<u8>) -> bool {
    p.len() == s.base.len() && forall|i: int| 0 <= i < p.len() ==>
        p[i] == s.base[i] || exists|k: int| 0 <= k < s.pending.len()
            && covers(s.pending[k], i) && p[i] == s.pending[k].data[i-s.pending[k].offset]
}
pub open spec fn wf(s: Storage) -> bool {
    s.volatile.len() == s.base.len() && s.persistent.len() == s.base.len()
    && (forall|k: int| 0 <= k < s.pending.len() ==> valid_write(s.pending[k], s.base.len()))
    && allowed(s, s.persistent)
}
pub open spec fn matches(v: Seq<u8>, c: Candidate) -> bool {
    forall|i: int| #[trigger] c.required.dom().contains(i) ==> 0 <= i < v.len() && v[i] == c.required[i]
}
pub open spec fn frames(w: Write, c: Candidate) -> bool {
    forall|i: int| #[trigger] c.required.dom().contains(i) && covers(w, i) ==>
        w.data[i-w.offset] == c.required[i]
}
pub open spec fn retained(s: Storage, c: Candidate) -> bool {
    matches(s.base, c) && forall|k: int| 0 <= k < s.pending.len() ==> frames(s.pending[k], c)
}
pub open spec fn storage_step(s: Storage, t: Storage, e: Event) -> bool {
    match e {
        Event::Submit(w) => valid_write(w, s.base.len()) && t == (Storage {
            volatile: apply_write(s.volatile,w), persistent: s.persistent,
            base: s.base, pending: s.pending.push(w) }),
        Event::Persist(p) | Event::SyncError(p) => allowed(s,p) && t == (Storage {
            persistent: p, ..s }),
        Event::SyncSuccess => t == (Storage { persistent: s.volatile,
            base: s.volatile, volatile: s.volatile, pending: Seq::empty() }),
        Event::Crash(p) => allowed(s,p) && t == (Storage { persistent: p,
            base: p, volatile: p, pending: Seq::empty() }),
        _ => t == s,
    }
}
// Buffer draining is explicit: the source must prove that a successful flush's
// preceding submissions cover all required bytes. SyncSuccess alone cannot do so.
pub open spec fn protocol_step(p: Protocol, q: Protocol, e: Event, tx: nat) -> bool {
    storage_step(p.storage, q.storage, e) && q.previous_two_phase == p.previous_two_phase
    && match e {
        Event::Buffer(w) => p.phase == Phase::Preparing && q.buffered == p.buffered.push(w)
            && q.phase == p.phase && q.last_ack == p.last_ack,
        Event::Submit(w) => (p.phase == Phase::Preparing || p.phase == Phase::Publishing)
            && p.buffered.len() > 0 && w == p.buffered[0]
            && q.buffered == p.buffered.drop_first() && q.phase == p.phase && q.last_ack == p.last_ack,
        Event::SyncSuccess => p.buffered.len() == 0 && q.buffered == p.buffered
            && q.last_ack == p.last_ack && (
                (p.phase == Phase::Preparing && q.phase == Phase::FirstBarrier)
                || (p.phase == Phase::Publishing && q.phase == Phase::SecondBarrier)),
        Event::Ack => p.phase == Phase::SecondBarrier && q.phase == Phase::Acknowledged
            && q.buffered == p.buffered && q.last_ack == Some(tx),
        Event::ReturnError | Event::SyncError(_) => q.phase == Phase::Failed
            && q.buffered == p.buffered && q.last_ack == p.last_ack,
        Event::Crash(_) => q.phase == Phase::Crashed && q.buffered.len() == 0 && q.last_ack == p.last_ack,
        Event::Persist(_) => q.phase == p.phase && q.buffered == p.buffered && q.last_ack == p.last_ack,
    }
}
// The control publication is a separate explicit transition, not an assumption
// that all header bytes were atomic. It can queue a full header write, byte-torn.
pub open spec fn publication(p: Protocol, q: Protocol, header: Write) -> bool {
    p.phase == Phase::FirstBarrier && p.buffered.len() == 0
    && q == (Protocol { phase: Phase::Publishing, buffered: seq![header], ..p })
}

pub proof fn outside_writes_unchanged(s: Storage, p: Seq<u8>, i: int)
    requires allowed(s,p), 0 <= i < p.len(),
        forall|k: int| 0 <= k < s.pending.len() ==> !covers(s.pending[k],i),
    ensures p[i] == s.base[i],
{}
pub proof fn publication_requires_first(p: Protocol, q: Protocol, header: Write)
    requires publication(p,q,header),
    ensures p.phase == Phase::FirstBarrier,
{}
pub proof fn retained_image(s: Storage, p: Seq<u8>, c: Candidate)
    requires retained(s,c), allowed(s,p),
    ensures matches(p,c),
{
    assert forall|i: int| #[trigger] c.required.dom().contains(i) implies
        0 <= i < p.len() && p[i] == c.required[i] by {
        assert(0 <= i < s.base.len());
        assert(s.base[i] == c.required[i]);
        if p[i] != s.base[i] {
            let k = choose|k: int| 0 <= k < s.pending.len() && covers(s.pending[k],i)
                && p[i] == s.pending[k].data[i-s.pending[k].offset];
            assert(frames(s.pending[k],c));
        }
    }
}
pub proof fn submit_frame(v: Seq<u8>, w: Write, c: Candidate)
    requires matches(v,c), frames(w,c),
    ensures matches(apply_write(v,w),c),
{
    assert forall|i: int| #[trigger] c.required.dom().contains(i) implies
        0 <= i < apply_write(v,w).len() && apply_write(v,w)[i] == c.required[i] by {
        assert(0 <= i < v.len());
        assert(v[i] == c.required[i]);
        if covers(w,i) { assert(w.data[i-w.offset] == c.required[i]); }
    }
}
pub proof fn step_retention(s: Storage, t: Storage, e: Event, c: Candidate)
    requires wf(s), retained(s,c), matches(s.volatile,c), storage_step(s,t,e),
        match e { Event::Submit(w) => frames(w,c), _ => true },
    ensures retained(t,c), matches(t.volatile,c), matches(t.persistent,c),
{
    match e {
        Event::Submit(w) => {
            submit_frame(s.volatile,w,c);
            retained_image(s,s.persistent,c);
            assert forall|k: int| 0 <= k < t.pending.len() implies frames(t.pending[k],c) by {
                if k < s.pending.len() { assert(t.pending[k] == s.pending[k]); }
                else { assert(k == s.pending.len()); assert(t.pending[k] == w); }
            }
        },
        Event::Persist(p) | Event::SyncError(p) => { retained_image(s,p,c); },
        Event::Crash(p) => { retained_image(s,p,c); },
        Event::SyncSuccess => {},
        _ => { retained_image(s,s.persistent,c); },
    }
}
pub open spec fn retained_trace(ss: Seq<Storage>, es: Seq<Event>, c: Candidate) -> bool {
    ss.len() == es.len()+1 && forall|k: int| 0 <= k < es.len() ==>
        wf(ss[k]) && storage_step(ss[k],ss[k+1],es[k])
        && match es[k] { Event::Submit(w) => frames(w,c), _ => true }
}
pub proof fn arbitrary_trace_retains(ss: Seq<Storage>, es: Seq<Event>, c: Candidate)
    requires retained_trace(ss,es,c), retained(ss[0],c), matches(ss[0].volatile,c),
    ensures retained(ss.last(),c), matches(ss.last().volatile,c),
        es.len() > 0 ==> matches(ss.last().persistent,c),
    decreases es.len(),
{
    if es.len() > 0 {
        let pre = ss.drop_last();
        let events = es.drop_last();
        assert(retained_trace(pre,events,c));
        arbitrary_trace_retains(pre,events,c);
        step_retention(ss[ss.len()-2],ss.last(),es.last(),c);
    }
}
pub proof fn successful_barrier_covers(s: Storage, t: Storage, c: Candidate)
    requires storage_step(s,t,Event::SyncSuccess), matches(s.volatile,c),
    ensures matches(t.persistent,c), retained(t,c), matches(t.volatile,c),
{}
pub proof fn ack_is_after_second(p: Protocol, q: Protocol, tx: nat)
    requires protocol_step(p,q,Event::Ack,tx),
    ensures p.phase == Phase::SecondBarrier, q.last_ack == Some(tx), q.storage == p.storage,
{}
pub proof fn required_two_barriers_ack_and_crash(
    p: Protocol, first: Protocol, published: Protocol, submitted: Protocol,
    second: Protocol, acked: Protocol, crashed: Storage,
    header: Write, c: Candidate, image: Seq<u8>)
    requires
        p.phase == Phase::Preparing, wf(p.storage), matches(p.storage.volatile,c),
        protocol_step(p,first,Event::SyncSuccess,c.transaction),
        publication(first,published,header), frames(header,c),
        protocol_step(published,submitted,Event::Submit(header),c.transaction),
        protocol_step(submitted,second,Event::SyncSuccess,c.transaction),
        protocol_step(second,acked,Event::Ack,c.transaction),
        storage_step(acked.storage,crashed,Event::Crash(image)),
    ensures matches(crashed.persistent,c), acked.last_ack == Some(c.transaction),
        acked.previous_two_phase == p.previous_two_phase,
{
    successful_barrier_covers(p.storage,first.storage,c);
    submit_frame(published.storage.volatile,header,c);
    successful_barrier_covers(submitted.storage,second.storage,c);
    ack_is_after_second(second,acked,c.transaction);
    retained_image(acked.storage,image,c);
}

pub open spec fn write_covers(w: Write, c: Candidate) -> bool {
    forall|i: int| #[trigger] c.required.dom().contains(i) ==>
        covers(w,i) && w.data[i-w.offset] == c.required[i]
}
pub proof fn write_establishes(v: Seq<u8>, w: Write, c: Candidate)
    requires valid_write(w,v.len()), write_covers(w,c),
    ensures matches(apply_write(v,w),c),
{}
// Separate pages and serialized publication footprints: barrier1 covers pages;
// the later header submission establishes its own bytes before barrier2.
pub proof fn acknowledged_header_and_pages(
    p: Protocol, first: Protocol, published: Protocol, submitted: Protocol,
    second: Protocol, acked: Protocol, crashed: Storage,
    header: Write, pages: Candidate, publication_bytes: Candidate, image: Seq<u8>)
    requires
        p.phase == Phase::Preparing, wf(p.storage), matches(p.storage.volatile,pages),
        protocol_step(p,first,Event::SyncSuccess,pages.transaction),
        publication(first,published,header), frames(header,pages), write_covers(header,publication_bytes),
        protocol_step(published,submitted,Event::Submit(header),pages.transaction),
        protocol_step(submitted,second,Event::SyncSuccess,pages.transaction),
        protocol_step(second,acked,Event::Ack,pages.transaction),
        storage_step(acked.storage,crashed,Event::Crash(image)),
    ensures matches(crashed.persistent,pages), matches(crashed.persistent,publication_bytes),
        acked.last_ack == Some(pages.transaction),
{
    required_two_barriers_ack_and_crash(p,first,published,submitted,second,acked,
        crashed,header,pages,image);
    write_establishes(published.storage.volatile,header,publication_bytes);
    successful_barrier_covers(submitted.storage,second.storage,publication_bytes);
    retained_image(acked.storage,image,publication_bytes);
}

pub proof fn control_byte_from_one_write(s: Storage, p: Seq<u8>, control: int)
    requires allowed(s,p), 0 <= control < p.len(),
    ensures p[control] == s.base[control] || exists|k: int|
        0 <= k < s.pending.len() && covers(s.pending[k],control)
            && p[control] == s.pending[k].data[control-s.pending[k].offset],
{}

pub proof fn covered_barrier_then_crash(s: Storage, durable: Storage, crashed: Storage, c: Candidate, bytes: Seq<u8>)
    requires matches(s.volatile,c), storage_step(s,durable,Event::SyncSuccess),
        storage_step(durable,crashed,Event::Crash(bytes)),
    ensures matches(crashed.persistent,c),
{
    successful_barrier_covers(s,durable,c);
    retained_image(durable,bytes,c);
}

// Header observations are separate from the storage-byte interpretation. The
// source lane must prove that interpretation, including actual checksum code.
pub struct Slot {
    pub transaction: nat, pub checksum_valid: bool, pub tree_complete: bool,
    pub raw: Seq<u8>, pub detected_corrupt: bool,
}
pub struct Control {
    pub primary: bool, pub two_phase: bool,
    pub recovery_required: bool, pub primary_unflushed: bool,
}
pub struct Header { pub control: Control, pub zero: Slot, pub one: Slot }
pub enum Selection { Reject, Zero, One }
pub open spec fn slot(h: Header, which: bool) -> Slot { if which { h.one } else { h.zero } }
pub open spec fn selection(which: bool) -> Selection { if which { Selection::One } else { Selection::Zero } }
pub open spec fn header_select(h: Header) -> Selection {
    let p = slot(h,h.control.primary);
    let s = slot(h,!h.control.primary);
    if h.control.two_phase {
        if p.checksum_valid { selection(h.control.primary) } else { Selection::Reject }
    } else if !p.checksum_valid {
        if s.checksum_valid { selection(!h.control.primary) } else { Selection::Reject }
    } else if s.checksum_valid && s.transaction > p.transaction {
        selection(!h.control.primary)
    } else { selection(h.control.primary) }
}
pub open spec fn good(s: Slot) -> bool { s.checksum_valid && s.tree_complete && !s.detected_corrupt }
pub open spec fn selected_good(h: Header, r: Selection) -> bool {
    match r { Selection::Zero => good(h.zero), Selection::One => good(h.one), Selection::Reject => false }
}
// Target recovery SPECIFICATION, not an implementation of REDB repair. A real
// two-phase corrupt primary may reject before this layer is reached.
pub open spec fn published_choice(h: Header) -> Selection {
    selection(if h.control.primary_unflushed { !h.control.primary } else { h.control.primary })
}
pub open spec fn served_recovery(h: Header) -> Selection {
    let first = header_select(h);
    if selected_good(h,first) { first }
    else if good(h.zero) { Selection::Zero }
    else if good(h.one) { Selection::One }
    else { Selection::Reject }
}
pub proof fn coherent_nonvacuous_target(h: Header)
    ensures (served_recovery(h) != Selection::Reject ==> selected_good(h,served_recovery(h))),
        (good(h.zero) || good(h.one)) ==> served_recovery(h) != Selection::Reject,
{}
pub proof fn header_selection_with_complete_candidates(h: Header)
    requires h.zero.checksum_valid ==> good(h.zero), h.one.checksum_valid ==> good(h.one),
    ensures header_select(h) != Selection::Reject ==> selected_good(h,header_select(h)),
{}
pub open spec fn repair_serialize(s: Slot, canonical: Seq<u8>) -> Seq<u8> {
    if s.detected_corrupt { s.raw } else { canonical }
}
// Observation is a mathematical deterministic function, not XXH3 certification.
pub proof fn detected_corrupt_preserved(s: Slot, canonical: Seq<u8>, detects: spec_fn(Seq<u8>) -> bool)
    requires s.detected_corrupt, detects(s.raw),
    ensures repair_serialize(s,canonical) == s.raw, detects(repair_serialize(s,canonical)),
{}
pub proof fn both_prior_modes_remain(prior: bool, p: Protocol, q: Protocol, tx: nat)
    requires p.previous_two_phase == prior, protocol_step(p,q,Event::Ack,tx),
    ensures q.previous_two_phase == prior,
{}

// Concrete *witnesses*, not exhaustive crash schedule enumeration.
pub proof fn torn_header_witness()
{
    let w = Write { offset: 0, data: seq![1u8, 1u8] };
    let s = Storage { volatile: seq![1u8,1u8], persistent: seq![0u8,0u8],
        base: seq![0u8,0u8], pending: seq![w] };
    assert(wf(s));
    assert(allowed(s,seq![1u8,0u8])) by {
        assert forall|i: int| 0 <= i < 2 implies
            seq![1u8,0u8][i] == s.base[i] || exists|k: int| 0 <= k < s.pending.len()
            && covers(s.pending[k],i) && seq![1u8,0u8][i] == s.pending[k].data[i-s.pending[k].offset] by {
            if i == 0 { assert(covers(s.pending[0],i)); }
        }
    }
    assert(seq![1u8,0u8] != s.base && seq![1u8,0u8] != s.volatile);
}
pub proof fn error_can_be_fully_durable()
{
    let w = Write { offset: 0, data: seq![1u8] };
    let s = Storage { volatile: seq![1u8], persistent: seq![0u8], base: seq![0u8], pending: seq![w] };
    let t = Storage { persistent: seq![1u8], ..s };
    assert(allowed(s,seq![1u8])) by { assert(covers(s.pending[0],0)); }
    assert(storage_step(s,t,Event::SyncError(seq![1u8])));
    assert(t.persistent =~= s.volatile);
    assert(t.persistent[0] != s.base[0]);
    assert(t.persistent == s.volatile && t.persistent != s.base);
    assert(storage_step(s,s,Event::SyncError(seq![0u8])));
}
pub proof fn prior_false_incomplete_secondary_witness()
{
    let old = Slot { transaction: 1, checksum_valid: true, tree_complete: true,
        raw: seq![1u8], detected_corrupt: false };
    let new = Slot { transaction: 2, checksum_valid: true, tree_complete: false,
        raw: seq![2u8], detected_corrupt: false };
    let h = Header { control: Control { primary: false, two_phase: false,
        recovery_required: true, primary_unflushed: false }, zero: old, one: new };
    assert(header_select(h) == Selection::One);
    assert(!selected_good(h,header_select(h)));
    assert(served_recovery(h) == Selection::Zero);
    let previous_two = Header { control: Control { two_phase: true, ..h.control }, ..h };
    assert(header_select(previous_two) == Selection::Zero);
}
// Satisfiable nonempty candidate/old-root footprints and BOTH previous mode flags.
pub proof fn concrete_two_barrier_success(prior: bool)
{
    let data = Write { offset: 2, data: seq![9u8] };
    let header = Write { offset: 0, data: seq![3u8] };
    let c = Candidate { transaction: 2, required: Map::empty().insert(2,9u8) };
    let old = Candidate { transaction: 1, required: Map::empty().insert(1,7u8) };
    let s = Storage { base: seq![0u8,7u8,0u8], persistent: seq![0u8,7u8,0u8],
        volatile: seq![0u8,7u8,9u8], pending: seq![data] };
    let p = Protocol { storage: s, buffered: Seq::empty(), phase: Phase::Preparing,
        previous_two_phase: prior, last_ack: None };
    let first = Protocol { storage: Storage { base: s.volatile, persistent: s.volatile,
        volatile: s.volatile, pending: Seq::empty() }, phase: Phase::FirstBarrier, ..p };
    let published = Protocol { phase: Phase::Publishing, buffered: seq![header], ..first };
    let submitted = Protocol { storage: Storage { volatile: apply_write(first.storage.volatile,header),
        pending: seq![header], ..first.storage }, buffered: Seq::empty(), ..published };
    let second = Protocol { storage: Storage { base: submitted.storage.volatile,
        persistent: submitted.storage.volatile, volatile: submitted.storage.volatile,
        pending: Seq::empty() }, phase: Phase::SecondBarrier, ..submitted };
    let acked = Protocol { phase: Phase::Acknowledged, last_ack: Some(2), ..second };
    let crashed = Storage { volatile: acked.storage.persistent, base: acked.storage.persistent,
        persistent: acked.storage.persistent, pending: Seq::empty() };
    assert(wf(s));
    assert(matches(s.volatile,c));
    assert(retained(s,old));
    assert(protocol_step(p,first,Event::SyncSuccess,2));
    assert(publication(first,published,header));
    assert(frames(header,c));
    assert(submitted.storage.pending =~= first.storage.pending.push(header));
    assert(submitted.buffered =~= published.buffered.drop_first());
    assert(protocol_step(published,submitted,Event::Submit(header),2));
    assert(protocol_step(submitted,second,Event::SyncSuccess,2));
    assert(protocol_step(second,acked,Event::Ack,2));
    assert(storage_step(acked.storage,crashed,Event::Crash(acked.storage.persistent)));
    required_two_barriers_ack_and_crash(p,first,published,submitted,second,acked,
        crashed,header,c,acked.storage.persistent);
    let publication_bytes = Candidate { transaction: 2, required: Map::empty().insert(0,3u8) };
    assert(write_covers(header,publication_bytes));
    acknowledged_header_and_pages(p,first,published,submitted,second,acked,
        crashed,header,c,publication_bytes,acked.storage.persistent);
    assert(matches(crashed.persistent,c));
    assert(matches(crashed.persistent,publication_bytes));
    assert(matches(crashed.persistent,old));
    assert(acked.previous_two_phase == prior);
}

pub proof fn false_sync_without_coverage(s: Storage, t: Storage, c: Candidate)
    requires wf(s), storage_step(s,t,Event::SyncSuccess),
    ensures matches(t.persistent,c),
{}

}
