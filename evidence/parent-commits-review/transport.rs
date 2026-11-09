use vstd::prelude::*;
#[verus_verify(clone_via_view)]
#[derive(Clone)] pub struct Slots<T>(Vec<Option<T>>);
verus! {
 impl<T> View for Slots<T> {type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
}

verus! {
 struct State<T> { slots:Slots<(T,u16)>, epoch:u16 }
 impl<T> View for State<T> {
  type V=(Seq<Option<(T,u16)>>,u16);
  closed spec fn view(&self)->Self::V {(self.slots@,self.epoch)}
 }
 impl<T> State<T> {
  spec fn valid(&self)->bool {
   self@.1>0 && forall|i:int|0<=i<self@.0.len() && self@.0[i].is_some()
    ==> self@.0[i].unwrap().1<self@.1
  }
 }
 fn advance<T:Clone>(previous:&State<T>)->(r:State<T>)
  requires previous@.1<u16::MAX,
  ensures previous.valid() ==> r.valid(),r@.1==previous@.1+1,
 {
  let next=State{slots:previous.slots.clone(),epoch:previous.epoch+1};
  next
 }
}
