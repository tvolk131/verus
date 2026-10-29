// Application meaning is independently supplied; no inferred summary is used
// to define the expected decision or its desired input coverage.
impl<H:Hasher,D:Data,S:Signature> vstd::std_specs::convert::FromSpecImpl<SignatureError<FullUnit<H,D>,S>>
 for ValidationError<H,D,S>{
 // Use the independently checked inferred contract, rather than imposing an
 // additional whole-object identity contract through the optional FromSpec.
 open spec fn obeys_from_spec()->bool{false}
 closed spec fn from_spec(error:SignatureError<FullUnit<H,D>,S>)->Self{
  ValidationError::WrongSignature(error.unchecked)
 }
}
impl<K:Keychain> Validator<K>{
 pub closed spec fn outer_domain<H:Hasher,D:Data>(
  &self,uu:UncheckedSignedUnit<H,D,K::Signature>,
 )->bool{
  let unit=&uu.signable;
  !self.keychain.authenticated::<H,D>(unit@,unit.creator_value(),uu.signature)
  || unit@.2!=self.session_id
  || unit@.0.1>self.max_round
  || unit@.0.2.0.len()!=self.member_count().0
  || unit@.0.1==0
  || unit@.0.0.0<unit@.0.2.0.len()
 }
 pub closed spec fn outer_policy<H:Hasher,D:Data>(
  &self,uu:UncheckedSignedUnit<H,D,K::Signature>,result:Result<H,D,K>,
 )->bool{
  let unit=&uu.signable;
  if !self.keychain.authenticated::<H,D>(unit@,unit.creator_value(),uu.signature){
   match result{Err(ValidationError::WrongSignature(original))=>original==uu,_=>false}
  }else if unit@.2!=self.session_id{
   match result{Err(ValidationError::WrongSession(copy))=>copy@==unit@,_=>false}
  }else if unit@.0.1>self.max_round{
   match result{Err(ValidationError::RoundTooHigh(copy))=>copy@==unit@,_=>false}
  }else{
   self.parent_policy(Signed{unchecked:uu},result)
  }
 }
 pub fn check_outer<H:Hasher,D:Data>(
  &self,uu:UncheckedSignedUnit<H,D,K::Signature>,
 )->(result:Result<H,D,K>)
  requires self.outer_domain(uu),
  ensures self.outer_policy(uu,result),
 {self.validate_unit(uu)}
}
