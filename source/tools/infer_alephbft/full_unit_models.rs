// Verification interface for the source-connected FullUnit experiment.
// SCALE encoding is an explicit library boundary. Lock operations use vstd's
// verified lock with explicit release, rather than parking_lot's RAII guards.
use std::{collections::hash_map::DefaultHasher, hash::Hasher as StdHasher};
use vstd::std_specs::hash::DefaultHasherAdditionalSpecFns;
use vstd::std_specs::clone::{CloneIdentity, lemma_cloned_identity};
use vstd::rwlock::{RwLock, RwLockPredicate};
use codec_model::encoded_fields;

verus! {
broadcast use {lemma_cloned_identity, vstd::array::group_array_axioms, vstd::seq::group_seq_axioms};
pub type Round = u16;
pub type SessionId = u64;
pub trait Hasher: Clone {
    type Hash: Copy + CloneIdentity;
    spec fn digest(bytes: Seq<u8>) -> Self::Hash;
    fn hash(x: &[u8]) -> (result: Self::Hash)
        ensures result == Self::digest(x@);
}
#[derive(Copy, Clone)]
pub struct Hasher64;
impl Hasher for Hasher64 {
    type Hash = [u8; 8];
    open spec fn digest(bytes: Seq<u8>) -> Self::Hash {
        choose|value: [u8; 8]| value@ == vstd::bytes::spec_u64_to_ne_bytes(
            DefaultHasher::spec_finish(seq![bytes]))
    }
    // HASHER_BODY
}

#[derive(Copy, Clone)]
pub struct UnitCoord { round: Round, creator: NodeIndex }
impl UnitCoord {
    pub closed spec fn parts(&self) -> (NodeIndex, Round) { (self.creator, self.round) }
    // COORD_METHODS
}

#[derive(Clone)]
pub struct ControlHash<H: Hasher> { parents: NodeMap<Round>, combined_hash: H::Hash }
impl<H: Hasher> View for ControlHash<H> {
    type V = (Seq<Option<Round>>, H::Hash);
    closed spec fn view(&self) -> Self::V { (self.parents@, self.combined_hash) }
}
impl<H: Hasher> ControlHash<H> {
    pub open spec fn __verus_clone_relation(&self, result: Self) -> bool {
        result@.0 =~= self@.0 && vstd::pervasive::cloned(self@.1, result@.1)
    }
    // CONTROL_METHODS
}

#[derive(Clone)]
pub struct PreUnit<H: Hasher> { coord: UnitCoord, control_hash: ControlHash<H> }
pub type PreView<H> = (NodeIndex, Round, (Seq<Option<Round>>, <H as Hasher>::Hash));
impl<H: Hasher> View for PreUnit<H> {
    type V = PreView<H>;
    closed spec fn view(&self) -> Self::V { (self.coord.creator, self.coord.round, self.control_hash@) }
}
impl<H: Hasher> PreUnit<H> {
    pub open spec fn __verus_clone_relation(&self, result: Self) -> bool { result@ == self@ }
    // PREUNIT_METHODS
}

pub type UnitView<H, D> = (PreView<H>, Option<D>, SessionId);
pub ghost struct CachePredicate<H: Hasher> { pub expected: H::Hash }
impl<H: Hasher> RwLockPredicate<Option<H::Hash>> for CachePredicate<H> {
    open spec fn inv(self, value: Option<H::Hash>) -> bool {
        value.is_none() || value.unwrap() == self.expected
    }
}
pub struct FullUnit<H: Hasher, D: Copy + CloneIdentity> {
    pre_unit: PreUnit<H>, data: Option<D>, session_id: SessionId,
    hash: RwLock<Option<H::Hash>, CachePredicate<H>>,
}
impl<H: Hasher, D: Copy + CloneIdentity> View for FullUnit<H, D> {
    type V = UnitView<H, D>;
    closed spec fn view(&self) -> Self::V { (self.pre_unit@, self.data, self.session_id) }
}
impl<H: Hasher, D: Copy + CloneIdentity> FullUnit<H, D> {
    #[verifier::type_invariant]
    closed spec fn cache_valid(&self) -> bool {
        self.hash.pred().expected == H::digest(encoded_fields::<UnitView<H, D>>(self@))
    }
    /// Explicit model for the codec derive; no hash result is assumed.
    fn encode(&self) -> (bytes: Vec<u8>)
        ensures bytes@ == encoded_fields::<UnitView<H, D>>(self@),
    { codec_model::encode_with_view(self) }

    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> (result: R)
        requires forall|bytes: &[u8]| bytes@ == encoded_fields::<UnitView<H, D>>(self@) ==> f.requires((bytes,)),
        ensures exists|bytes: &[u8]| bytes@ == encoded_fields::<UnitView<H, D>>(self@) && f.ensures((bytes,), result),
    { f(&self.encode()) }
    // FULLUNIT_METHODS
}
impl<H: Hasher, D: Copy + CloneIdentity> Clone for FullUnit<H, D> {
    // CLONE_BODY
}

pub trait Unit: Clone {
    type Hasher: Hasher;
    spec fn creator_value(&self) -> NodeIndex;
    spec fn round_value(&self) -> Round;
    spec fn hash_value(&self) -> <Self::Hasher as Hasher>::Hash;
    spec fn control_value(&self) -> (Seq<Option<Round>>, <Self::Hasher as Hasher>::Hash);
    spec fn session_value(&self) -> SessionId;
    fn hash(&self) -> (result: <Self::Hasher as Hasher>::Hash) ensures result == self.hash_value();
    fn coord(&self) -> (result: UnitCoord)
        ensures result.parts() == (self.creator_value(), self.round_value());
    fn control_hash(&self) -> (result: &ControlHash<Self::Hasher>) ensures result@ == self.control_value();
    fn session_id(&self) -> (result: SessionId) ensures result == self.session_value();
    // DEFAULT_METHODS
}
impl<H: Hasher, D: Copy + CloneIdentity> Unit for FullUnit<H, D> {
    type Hasher = H;
    closed spec fn creator_value(&self) -> NodeIndex { self.pre_unit.coord.creator }
    closed spec fn round_value(&self) -> Round { self.pre_unit.coord.round }
    closed spec fn hash_value(&self) -> H::Hash { H::digest(encoded_fields::<UnitView<H, D>>(self@)) }
    closed spec fn control_value(&self) -> (Seq<Option<Round>>, H::Hash) { self.pre_unit.control_hash@ }
    closed spec fn session_value(&self) -> SessionId { self.session_id }
    // UNIT_METHODS
}

pub fn check_repeated_hash<H: Hasher, D: Copy + CloneIdentity>(unit: &FullUnit<H, D>) {
    let first = Unit::hash(unit);
    let second = Unit::hash(unit);
    assert(first == second);
}
pub fn check_clone_observations<H: Hasher, D: Copy + CloneIdentity>(unit: &FullUnit<H, D>) {
    let copied = unit.clone();
    assert(copied@ == unit@);
    let copied_hash = Unit::hash(&copied);
    let original_hash = Unit::hash(unit);
    assert(copied_hash == original_hash);
}
}
