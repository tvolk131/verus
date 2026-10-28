// Supplied application meanings and goals for the original collector bodies.
// These invariants and counting specifications are inputs, not inferred facts.
verus! {
    spec fn direct<T>(candidate: Option<(T, u16)>, round: u16) -> bool {
        match candidate {
            Some((_, r)) => r as int + 1 == round,
            None => false,
        }
    }

    spec fn count_prefix<T>(candidates: Seq<Option<(T, u16)>>, round: u16, len: nat) -> nat
        recommends len <= candidates.len(),
        decreases len,
    { candidates.count_prefix(|slot: Option<(T, u16)>| direct(slot, round), len) }

    spec fn reference_count_prefix<T>(candidates: Seq<Option<(T, u16)>>, round: u16, len: nat) -> nat
        recommends len <= candidates.len(),
        decreases len,
    {
        if len == 0 { 0 } else {
            reference_count_prefix(candidates, round, (len - 1) as nat)
                + if direct(candidates[len - 1], round) { 1nat } else { 0nat }
        }
    }

    proof fn library_count_agrees<T>(candidates: Seq<Option<(T, u16)>>, round: u16, len: nat)
        requires len <= candidates.len(),
        ensures count_prefix(candidates, round, len) == reference_count_prefix(candidates, round, len),
        decreases len,
    { if len > 0 { library_count_agrees(candidates, round, (len - 1) as nat); } }
    

    

    


impl<H:Hasher> UnitsCollector<H>{
        pub closed spec fn parents(&self) -> Seq<Option<(H::Hash, u16)>> {
            self.candidates@
        }

        pub closed spec fn round(&self) -> u16 {
            self.for_round
        }

        pub closed spec fn count(&self) -> usize {
            self.direct_parents.0
        }

        pub closed spec fn valid(&self) -> bool {
            &&& 1 <= self.for_round <= u16::MAX as int
            &&& self.direct_parents.0 <= self.candidates@.len()
            &&& forall|i: int| 0 <= i < self.candidates@.len() ==>
                match #[trigger] self.candidates@[i] {
                    Some((_, r)) => r < self.for_round,
                    None => true,
                }
            &&& self.direct_parents.0 == count_prefix(
                self.candidates@, self.for_round, self.candidates@.len())
        }

        pub closed spec fn ready(&self, creator: usize) -> bool {
            &&& self.direct_parents.0 >= (2 * self.candidates@.len()) / 3 + 1
            &&& creator < self.candidates@.len()
            &&& direct(self.candidates@[creator as int], self.for_round)
        }
 fn check_new(n:NodeCount)->(result:Self)
 ensures result.valid(),result.round()==1,result.count()==0,result.parents().len()==n.0,
 { Self::new_initial(n) }
 fn check_next(previous:&Self)->(result:Self)
 requires previous.valid(),previous.round()<u16::MAX,
 ensures result.valid(),result.round() as int==previous.round() as int+1,result.count()==0,result.parents()==previous.parents(),
 { Self::from_previous(previous) }
 fn check_add<U:Unit<Hasher=H>>(&mut self,unit:&U)
 requires old(self).valid(),unit.creator_value().0<old(self).parents().len(),
 ensures final(self).valid(),final(self).round()==old(self).round(),
 final(self).parents().len()==old(self).parents().len(), final(self).count()>=old(self).count(),
 forall|i:int|0<=i<final(self).parents().len() && i!=unit.creator_value().0 ==> final(self).parents()[i]==old(self).parents()[i],
 final(self).parents()[unit.creator_value().0 as int] == if unit.round_value()<old(self).round() && match old(self).parents()[unit.creator_value().0 as int]{None=>true,Some((_,r))=>r<unit.round_value()}{Some((unit.hash_value(),unit.round_value()))}else{old(self).parents()[unit.creator_value().0 as int]},
 {
 

 self.add_unit(unit);
 }
 fn check_selection(&self,node:NodeIndex)->(result:Result<&NodeMap<(H::Hash,Round)>,ConstraintError>)
 requires self.valid(),node.0<self.parents().len(),
 ensures result.is_ok()==self.ready(node.0),result.is_ok() ==> result.unwrap()@==self.parents(),
 self.count()<(2*self.parents().len())/3+1 ==> result==Err::<&NodeMap<(H::Hash,Round)>,ConstraintError>(ConstraintError::NotEnoughParents),
 self.count()>=(2*self.parents().len())/3+1 && !direct(self.parents()[node.0 as int],self.round()) ==> result==Err::<&NodeMap<(H::Hash,Round)>,ConstraintError>(ConstraintError::MissingOwnParent),
 {self.prospective_parents(node)}

}

}
