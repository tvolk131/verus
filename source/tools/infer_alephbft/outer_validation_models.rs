// Supplied authentication boundary. The adapter stands for native
// UncheckedSigned::check's hash/as_ref/index/Keychain::verify chain, whose
// implementation and cryptographic security are not proved here. It binds the
// decision to the complete logical unit, creator and supplied signature.
verus! {
pub struct SignatureError<T,S>{pub unchecked:UncheckedSigned<T,S>}
impl<H:Hasher,D:Data,S:Signature> UncheckedSigned<FullUnit<H,D>,S>{
 pub closed spec fn checked_value<K:Keychain<Signature=S>>(
  self,keychain:&K,
 )->core::result::Result<Signed<FullUnit<H,D>,K>,SignatureError<FullUnit<H,D>,S>>{
  if keychain.authenticated::<H,D>(self.signable@,self.signable.creator_value(),self.signature){
   Ok(Signed{unchecked:self})
  }else{Err(SignatureError{unchecked:self})}
 }
 pub fn check<K:Keychain<Signature=S>>(
  self,keychain:&K,
 )->(result:core::result::Result<Signed<FullUnit<H,D>,K>,SignatureError<FullUnit<H,D>,S>>)
  ensures result==self.checked_value(keychain),
 {
  if !keychain.authenticate(&self.signable,&self.signature){
   return Err(SignatureError{unchecked:self});
  }
  Ok(Signed{unchecked:self})
 }
}
}
