use vstd::prelude::*;
verus! {
// Mathematical observations only. None of these ghost values is a Rust permission.
pub ghost struct Ptr { pub origin: int, pub loan: int, pub offset: int }
pub ghost enum Val {
    Dead, Unit, Num(int), Ref { p: Ptr, len: int, slice: bool },
    Raw { p: Ptr, meta: Option<int> }, NonNull { p: Ptr, meta: Option<int> },
    Iter { start: Ptr, end: Ptr },
}
pub ghost struct State {
    pub bytes: Seq<u8>, pub origin: int, pub loan: int,
    pub regs: Map<int, Val>, pub custodian: int, pub stack: Seq<int>, pub ok: bool,
}
pub ghost enum Op {
    Clear(int), Unit(int), Num(int,int), Copy(int,int), Move(int,int),
    Reborrow(int,int), Unsize(int,int), Metadata(int,int), Raw(int,int),
    Wrap(int,int), Unwrap(int,int), Thin(int,int), Offset(int,int,int),
    Iter(int,int,int), Test(int,int), Enter(int,int), Leave(int,int),
    // Store is present so frame soundness isn't obtained by defining writes away.
    Store(int,u8),
}
pub open spec fn ptr_ok(p: Ptr, s: State) -> bool {
    p.origin == s.origin && p.loan == s.loan && 0 <= p.offset <= s.bytes.len()
        && s.bytes.len() <= 9223372036854775807int
}
pub open spec fn val_ok(v: Val, s: State) -> bool {
    match v {
        Val::Dead | Val::Unit => true,
        Val::Num(n) => 0 <= n <= 18446744073709551615int,
        Val::Ref{p,len,slice} => ptr_ok(p,s) && 0 <= len && p.offset + len <= s.bytes.len(),
        Val::Raw{p,meta} | Val::NonNull{p,meta} => ptr_ok(p,s)
            && (meta is Some ==> 0 <= meta->Some_0 && p.offset + meta->Some_0 <= s.bytes.len()),
        Val::Iter{start,end} => ptr_ok(start,s) && ptr_ok(end,s) && start.offset <= end.offset,
    }
}
pub open spec fn wf(s: State) -> bool {
    s.ok && s.bytes.len() == 128 && s.custodian >= 0
        && forall|r:int| s.regs.dom().contains(r) ==> val_ok(#[trigger] s.regs[r],s)
}
pub open spec fn get(s: State,r:int) -> Val { s.regs[r] }
pub open spec fn live(s: State,r:int) -> bool { s.regs.dom().contains(r) && !(s.regs[r] is Dead) }
pub open spec fn put(s: State,r:int,v:Val) -> State { State{regs:s.regs.insert(r,v),..s} }
pub open spec fn bad(s:State) -> State { State{ok:false,..s} }
pub open spec fn initial(bytes:Seq<u8>,origin:int,loan:int) -> State {
    State {bytes,origin,loan,regs:Map::empty().insert(1,Val::Ref{p:Ptr{origin,loan,offset:0},len:128,slice:false}),custodian:0,stack:Seq::empty(),ok:true}
}
pub open spec fn step(s:State,o:Op) -> State {
    if !s.ok { s } else { match o {
        Op::Clear(d) => put(s,d,Val::Dead),
        Op::Unit(d) => put(s,d,Val::Unit),
        Op::Num(d,n) => if 0 <= n <= 18446744073709551615int {put(s,d,Val::Num(n))}else{bad(s)},
        Op::Copy(d,r) => if live(s,r) {put(s,d,get(s,r))}else{bad(s)},
        Op::Move(d,r) => if live(s,r) && d!=r {put(put(s,r,Val::Dead),d,get(s,r))}else{bad(s)},
        Op::Reborrow(d,r) => if live(s,r) && get(s,r) is Ref && ptr_ok(get(s,r)->Ref_p,s) {
            put(s,d,get(s,r))
        }else{bad(s)},
        Op::Unsize(d,r) => if live(s,r) && get(s,r) is Ref && !get(s,r)->Ref_slice && get(s,r)->Ref_len==128 {
            put(put(s,r,Val::Dead),d,Val::Ref{p:get(s,r)->Ref_p,len:get(s,r)->Ref_len,slice:true})
        }else{bad(s)},
        Op::Metadata(d,r) => if live(s,r) && get(s,r) is Ref && get(s,r)->Ref_slice {
            put(s,d,Val::Num(get(s,r)->Ref_len))
        }else{bad(s)},
        Op::Raw(d,r) => if live(s,r) && get(s,r) is Ref && get(s,r)->Ref_slice {
            put(s,d,Val::Raw{p:get(s,r)->Ref_p,meta:Some(get(s,r)->Ref_len)})
        }else{bad(s)},
        Op::Wrap(d,r) => if live(s,r) && get(s,r) is Raw {
            put(s,d,Val::NonNull{p:get(s,r)->Raw_p,meta:get(s,r)->Raw_meta})
        }else{bad(s)},
        Op::Unwrap(d,r) => if live(s,r) && get(s,r) is NonNull {
            put(s,d,Val::Raw{p:get(s,r)->NonNull_p,meta:get(s,r)->NonNull_meta})
        }else{bad(s)},
        Op::Thin(d,r) => if live(s,r) && get(s,r) is Raw {
            put(s,d,Val::Raw{p:get(s,r)->Raw_p,meta:None})
        }else{bad(s)},
        Op::Offset(d,r,n) => if live(s,r) && live(s,n) && get(s,r) is Raw && get(s,r)->Raw_meta is None && get(s,n) is Num
            && ptr_ok(get(s,r)->Raw_p,s) && 0 <= get(s,n)->Num_0 <= 9223372036854775807int
            && get(s,r)->Raw_p.offset + get(s,n)->Num_0 <= s.bytes.len() {
            put(s,d,Val::Raw{p:Ptr{offset:get(s,r)->Raw_p.offset + get(s,n)->Num_0,..get(s,r)->Raw_p},meta:None})
        }else{bad(s)},
        Op::Iter(d,a,b) => if live(s,a) && live(s,b) && get(s,a) is NonNull && get(s,a)->NonNull_meta is None
            && get(s,b) is Raw && get(s,b)->Raw_meta is None
            && get(s,a)->NonNull_p.offset <= get(s,b)->Raw_p.offset {
            put(s,d,Val::Iter{start:get(s,a)->NonNull_p,end:get(s,b)->Raw_p})
        }else{bad(s)},
        Op::Test(r,n) => if live(s,r) && get(s,r)==Val::Num(n) {s}else{bad(s)},
        Op::Enter(callee,arg) => if live(s,arg) && get(s,arg) is Ref && val_ok(get(s,arg),s) && callee>=0 {
            State{custodian:callee,stack:s.stack.push(s.custodian),..s}
        }else{bad(s)},
        Op::Leave(callee,result) => if s.custodian==callee && s.stack.len()>0 && live(s,result)
            && get(s,result) is Iter && val_ok(get(s,result),s) {
            State{custodian:s.stack.last(),stack:s.stack.drop_last(),..s}
        }else{bad(s)},
        Op::Store(r,v) => if live(s,r) && get(s,r) is Raw && ptr_ok(get(s,r)->Raw_p,s)
            && get(s,r)->Raw_p.offset < s.bytes.len() {
            State{bytes:s.bytes.update(get(s,r)->Raw_p.offset,v),..s}
        }else{bad(s)},
    }}
}
pub open spec fn writes(s:State,o:Op) -> Set<int> {
    match o { Op::Store(r,_) => if live(s,r) && get(s,r) is Raw {Set::empty().insert(get(s,r)->Raw_p.offset)}else{Set::empty()}, _ => Set::empty() }
}
pub open spec fn run(s:State,ops:Seq<Op>) -> State
    decreases ops.len(),
{ if ops.len()==0 {s} else {run(step(s,ops[0]),ops.drop_first())} }
pub open spec fn footprint(s:State,ops:Seq<Op>) -> Set<int>
    decreases ops.len(),
{ if ops.len()==0 {Set::empty()} else {writes(s,ops[0]).union(footprint(step(s,ops[0]),ops.drop_first()))} }
pub proof fn embedding(bytes:Seq<u8>,origin:int,loan:int)
    requires bytes.len()==128,
    ensures wf(initial(bytes,origin,loan)),initial(bytes,origin,loan).bytes==bytes,
{ assert forall|r:int| initial(bytes,origin,loan).regs.dom().contains(r)
    implies val_ok(#[trigger] initial(bytes,origin,loan).regs[r],initial(bytes,origin,loan)) by {}; }
pub proof fn one_step_frame(s:State,o:Op,i:int)
    requires 0 <= i < s.bytes.len(), !writes(s,o).contains(i),
    ensures step(s,o).bytes.len()==s.bytes.len(),step(s,o).bytes[i]==s.bytes[i],
{ }
pub proof fn trace_frame(s:State,ops:Seq<Op>,i:int)
    requires 0 <= i < s.bytes.len(), !footprint(s,ops).contains(i),
    ensures run(s,ops).bytes.len()==s.bytes.len(),run(s,ops).bytes[i]==s.bytes[i],
    decreases ops.len(),
{
    if ops.len()>0 {
        one_step_frame(s,ops[0],i);
        trace_frame(step(s,ops[0]),ops.drop_first(),i);
    }
}
}
