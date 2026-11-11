//! Logical diagnostic only. These datatypes are NOT native core Rev/Iter.
use vstd::prelude::*;
verus! {
pub struct Carrier<I> { pub inner: I }
pub open spec fn c<I>(v:I) -> Carrier<I> { Carrier {inner:v} }
pub open spec fn f<I>(r:Carrier<I>) -> I { r.inner }
pub proof fn computation<I>(v:I, r:Carrier<I>)
    ensures f(c(v))==v, c(f(r))==r,
{}
pub proof fn equality<I>(a:Carrier<I>, b:Carrier<I>)
    ensures (a==b) == (f(a)==f(b)),
{}
// Fully generic use forces ordinary polymorphic encoding as well as datatype
// computation. This is not a native box/unbox correspondence theorem.
pub struct BoxContext<I> { pub value: I }
pub proof fn poly_context<I>(a:Carrier<I>)
    ensures f(BoxContext {value:a}.value)==f(a),
{}
pub struct State<A> {
    pub values:Seq<A>, pub lawful:bool, pub none:bool, pub metric:Option<nat>,
    pub front:spec_fn(int)->Option<A>, pub back:spec_fn(int)->Option<A>,
}
pub open spec fn rem<A>(r:Carrier<State<A>>) -> Seq<A> { f(r).values.reverse() }
pub open spec fn law<A>(r:Carrier<State<A>>) -> bool { f(r).lawful }
pub open spec fn none<A>(r:Carrier<State<A>>) -> bool { f(r).none }
pub open spec fn metric<A>(r:Carrier<State<A>>) -> Option<nat> { f(r).metric }
pub open spec fn preserved<A>(a:State<A>, b:State<A>) -> bool {
    &&& a.lawful == b.lawful
    &&& a.lawful ==> a.none==b.none
    &&& a.lawful ==> a.metric.is_some()==b.metric.is_some()
}
pub open spec fn decrease<A>(a:State<A>, b:State<A>) -> bool {
    a.lawful && a.values.len()>0 && b.metric.is_some() ==>
        a.metric.is_some() && a.metric.unwrap()>b.metric.unwrap()
}
pub open spec fn back_law<A>(a:State<A>, b:State<A>, out:Option<A>) -> bool {
    &&& preserved(a,b)
    &&& decrease(a,b)
    &&& b.lawful ==> if a.values.len()>0 {
        b.values==a.values.drop_last() && out==Some(a.values.last())
    } else { b.values==a.values && out==None && b.none }
}
pub open spec fn front_law<A>(a:State<A>, b:State<A>, out:Option<A>) -> bool {
    &&& preserved(a,b)
    &&& decrease(a,b)
    &&& b.lawful ==> if a.values.len()>0 {
        b.values==a.values.drop_first() && out==Some(a.values[0])
    } else { b.values==a.values && out==None && b.none }
}
pub open spec fn reversed<A>(a:State<A>) -> State<A> {
    State { values:a.values.reverse(), lawful:a.lawful, none:a.none, metric:a.metric, front:a.back, back:a.front }
}
pub proof fn reverse_identities<A>(s:Seq<A>)
    ensures s.reverse().len()==s.len(), s.reverse().reverse()==s,
        s.len()>0 ==> s.reverse()[0]==s.last(),
        s.len()>0 ==> s.reverse().last()==s[0],
        s.len()>0 ==> s.drop_last().reverse()==s.reverse().drop_first(),
        s.len()>0 ==> s.drop_first().reverse()==s.reverse().drop_last(),
{
    broadcast use vstd::seq::group_seq_axioms;
    assert(s.reverse().reverse() =~= s);
    if s.len()>0 {
        assert(s.drop_last().reverse() =~= s.reverse().drop_first());
        assert(s.drop_first().reverse() =~= s.reverse().drop_last());
    }
}
pub proof fn lift_next<A>(a:State<A>, b:State<A>, out:Option<A>)
    requires back_law(a,b,out),
    ensures front_law(reversed(a),reversed(b),out),
        law(c(a))==law(c(b)),
        a.lawful ==> none(c(a))==none(c(b)),
        a.lawful ==> metric(c(a)).is_some()==metric(c(b)).is_some(),
{
    reverse_identities(a.values);
    reverse_identities(b.values);
}
pub proof fn lift_back<A>(a:State<A>, b:State<A>, out:Option<A>)
    requires front_law(a,b,out),
    ensures back_law(reversed(a),reversed(b),out),
{
    reverse_identities(a.values);
    reverse_identities(b.values);
}
pub proof fn constructor_observations<A>(a:State<A>)
    ensures law(c(a))==a.lawful, rem(c(a))==a.values.reverse(),
        none(c(a))==a.none, metric(c(a))==a.metric,
{}
// All reference values are carried as generic A (instantiate at &T); equality
// transports them without cloning, dereferencing or fabricating a native ref.
pub proof fn indexed_reference<'a,T>(s:Seq<&'a T>, n:usize, k:int)
    requires 0<=k<s.len(), n as int+s.len()<=usize::MAX,
    ensures Seq::new(s.len(), |j:int| ((n as int+j) as usize,s[j])).reverse()[k]
        == ((n as int+s.len()-1-k) as usize,s[s.len()-1-k]),
{
    broadcast use vstd::seq::group_seq_axioms;
}
// Pointwise logical observation mapping at arbitrary worlds. The association
// of native old/final field reborrows with these worlds is NOT assumed/proved.
pub proof fn worlds<A>(s:spec_fn(int)->State<A>, w:int)
    ensures rem(c(s(w)))==s(w).values.reverse(), law(c(s(w)))==s(w).lawful,
        metric(c(s(w)))==s(w).metric, none(c(s(w)))==s(w).none,
{}
pub proof fn peeks<A>(s:State<A>, k:int)
    ensures (reversed(s).front)(k)==(s.back)(k), (reversed(s).back)(k)==(s.front)(k),
        reversed(reversed(s))==s,
{
    reverse_identities(s.values);
}
}
