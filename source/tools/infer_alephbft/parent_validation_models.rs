// The input is an already checked Signed value. These interfaces preserve its
// source fields and accessor; authentication and Signable/Index/codec marker
// obligations are outside this parent-only experiment. The Keychain count
// equation and application policy are supplied, not inferred.

use vstd::std_specs::clone::CloneView as Data;
verus! {
pub trait Signature {}
impl<T> Signature for T {}
pub trait Keychain {
 type Signature:Signature;
 spec fn count_value(&self)->NodeCount;
 fn node_count(&self)->(r:NodeCount) ensures r==self.count_value();
}
pub struct UncheckedSigned<T,S>{signable:T,signature:S}
pub struct Signed<T,K:Keychain>{unchecked:UncheckedSigned<T,K::Signature>}
impl<T,K:Keychain> Signed<T,K>{
#[verifier::infer]
// ORIGINAL_SIGNED_ACCESSOR
}
pub type SignedUnit<H,D,K> = Signed<FullUnit<H,D>,K>;
pub type UncheckedSignedUnit<H,D,S> = UncheckedSigned<FullUnit<H,D>,S>;
pub type ControlHashError<H> = Error<H>;
}
mod parent_validation { use super::*; verus! {
// ORIGINAL_VALIDATION_ERROR
// ORIGINAL_VALIDATOR
 type Result<H,D,K> = core::result::Result<SignedUnit<H,D,K>,ValidationError<H,D,<K as Keychain>::Signature>>;
impl<K:Keychain> Validator<K>{
 #[verifier::infer]
 // ORIGINAL_PARENT_VALIDATION
}

impl<H:Hasher,D:Data,K:Keychain> Signed<FullUnit<H,D>,K> {
 pub closed spec fn pre_value(&self)->PreView<H> {self.unchecked.signable.pre_unit@}
}

impl<K:Keychain> Validator<K> {
 pub closed spec fn member_count(&self)->NodeCount {self.keychain.count_value()}

 pub closed spec fn parent_policy<H:Hasher,D:Data>(
  &self,su:SignedUnit<H,D,K>,result:Result<H,D,K>
 )->bool {
  let pre=&su.unchecked.signable.pre_unit;
  let members_match=pre@.2.0.len()==self.member_count().0;
  match result {
   Ok(value)=>members_match && pre.control_hash.expected_validation(pre.coord).is_ok() && value==su,
   Err(ValidationError::WrongNumberOfMembers(copy))=>!members_match && copy@==pre@,
   Err(ValidationError::ParentValidationFailed(copy,error))=>members_match && copy@==pre@
    && pre.control_hash.expected_validation(pre.coord)==Err::<(),Error<H>>(error),
   _=>false,
  }
 }
 pub fn check_parents<H:Hasher,D:Data>(&self,su:SignedUnit<H,D,K>)->(result:Result<H,D,K>)
  requires
   su.pre_value().2.0.len()!=self.member_count().0
   || su.pre_value().1==0
   || su.pre_value().0.0<su.pre_value().2.0.len(),
  ensures self.parent_policy(su,result),
 {self.validate_unit_parents(su)}
}
}}
