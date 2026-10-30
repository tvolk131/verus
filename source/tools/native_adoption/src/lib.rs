//! Native, non-Aleph integration example. Exclusions supply no contracts.
use vstd::prelude::*;

#[verus_verify(infer_derive(Add, From, Into))]
#[verus_verify(external_derive(Sum))]
#[derive(Clone, Copy, derive_more::Add, derive_more::From, derive_more::Into, derive_more::Sum)]
pub struct Quantity(pub usize);

#[verus_infer]
pub fn unwrap(value: Quantity) -> usize {
    value.into()
}

#[verus_infer]
pub fn wrap(value: usize) -> Quantity {
    value.into()
}

#[verus_verify(external_derive(Encode, Decode))]
#[derive(codec::Encode, codec::Decode)]
pub struct Packet {
    pub payload: u16,
}

// The derive still exists for Rust, but excluding the requested implementation
// must not let the generated model become a usable verified contract.
#[cfg(feature = "exclude-requested-into")]
#[verus_verify(infer_derive(Into))]
#[verus_verify(external_derive(Into))]
#[derive(Clone, Copy, derive_more::Into)]
pub struct UncheckedConversion(pub u8);

#[verus_infer]
pub fn payload(packet: &Packet) -> u16 {
    packet.payload
}

verus! {
pub fn roundtrip(value: usize, packet: &Packet) {
    let wrapped = wrap(value);
    let original = unwrap(wrapped);
    assert(original == value);
    let result = payload(packet);
    assert(result == packet.payload);
}
}

#[cfg(feature = "use-excluded-sum")]
#[verus_verify]
pub fn excluded_sum(values: Vec<Quantity>) -> Quantity {
    <Quantity as core::iter::Sum<Quantity>>::sum(values.into_iter())
}

#[cfg(feature = "use-excluded-codec")]
#[verus_verify]
pub fn excluded_encode(packet: &Packet) -> Vec<u8> {
    codec::Encode::encode(packet)
}

#[cfg(feature = "wrong-goal")]
verus! {
pub fn false_roundtrip() {
    let result = unwrap(Quantity(7));
    assert(result == 8);
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_conversions_and_excluded_implementations_are_retained() {
        for value in [0, 1, usize::MAX] {
            assert_eq!(unwrap(wrap(value)), value);
        }
        assert_eq!([Quantity(2), Quantity(3)].into_iter().sum::<Quantity>().0, 5);
        let packet = Packet { payload: 513 };
        let bytes = codec::Encode::encode(&packet);
        let decoded = <Packet as codec::Decode>::decode(&mut &bytes[..]).unwrap();
        assert_eq!(payload(&decoded), 513);
    }
}
