// F1: frozen source-connected projection; see FREEZE.md. No executable body edits.
use vstd::prelude::*;

#[verus_verify(infer_derive(PartialEq, PartialOrd, Ord))]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Amount { pub msats: u64 }

#[verus_verify]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PaymentFee { pub base: Amount, pub parts_per_million: u64 }

verus! {
pub open spec fn cap(x: int) -> int { if x > u64::MAX { u64::MAX as int } else { x } }
pub open spec fn floor_zero(x: int) -> int { if x < 0 { 0 } else { x } }
pub open spec fn within(a: PaymentFee, b: PaymentFee) -> bool {
    a.base.msats <= b.base.msats && a.parts_per_million <= b.parts_per_million
}
pub open spec fn fee_value(f: PaymentFee, payment: int) -> int {
    cap(cap(payment * f.parts_per_million as int) / 1_000_000 + f.base.msats as int)
}
pub open spec fn send_value(f: PaymentFee, payment: int) -> int { cap(payment + fee_value(f, payment)) }
pub open spec fn receive_value(f: PaymentFee, payment: int) -> int { floor_zero(payment - fee_value(f, payment)) }
}

#[verus_verify]
impl Amount {
    #[verus_spec(r => ensures r.msats == msats)]
    pub const fn from_msats(msats: u64) -> Self {
        Self { msats }
    }

    #[verus_spec(r => ensures match r { Some(v) => self.msats + other.msats <= u64::MAX && v.msats == self.msats + other.msats,
            None => self.msats + other.msats > u64::MAX })]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            msats: self.msats.checked_add(other.msats)?,
        })
    }

}

#[verus_verify]
impl PaymentFee {
    #[verus_spec(r => ensures r == within(*self, *limit))]
    pub fn is_within(&self, limit: &PaymentFee) -> bool {
        self.base <= limit.base && self.parts_per_million <= limit.parts_per_million
    }

    #[verus_spec(r => ensures match r { Some(v) => self.base.msats + rhs.base.msats <= u64::MAX && self.parts_per_million + rhs.parts_per_million <= u64::MAX
                && v.base.msats == self.base.msats + rhs.base.msats && v.parts_per_million == self.parts_per_million + rhs.parts_per_million,
            None => self.base.msats + rhs.base.msats > u64::MAX || self.parts_per_million + rhs.parts_per_million > u64::MAX })]
    pub fn checked_add(self, rhs: Self) -> Option<PaymentFee> {
        Some(PaymentFee {
            base: self.base.checked_add(rhs.base)?,
            parts_per_million: self.parts_per_million.checked_add(rhs.parts_per_million)?,
        })
    }

    #[verus_spec(r => ensures r.msats == send_value(*self, msats as int))]
    pub fn add_to(&self, msats: u64) -> Amount {
        Amount::from_msats(msats.saturating_add(self.absolute_fee(msats)))
    }

    #[verus_spec(r => ensures r.msats == receive_value(*self, msats as int))]
    pub fn subtract_from(&self, msats: u64) -> Amount {
        Amount::from_msats(msats.saturating_sub(self.absolute_fee(msats)))
    }

    #[verus_spec(r => ensures r.msats == fee_value(*self, msats as int))]
    pub fn fee(&self, msats: u64) -> Amount {
        Amount::from_msats(self.absolute_fee(msats))
    }

    #[verus_spec(r => ensures r == fee_value(*self, msats as int))]
    fn absolute_fee(&self, msats: u64) -> u64 {
        // The base fee is bounded well below `u64::MAX` for any fee that passed
        // the limit checks, but a fee decoded from an older database has not
        // necessarily passed them, so saturate rather than panic.
        msats
            .saturating_mul(self.parts_per_million)
            .saturating_div(1_000_000)
            .saturating_add(self.base.msats)
    }

}

// All hypotheses below express the conditional ordering theorem, never runtime restrictions.
verus! {
pub proof fn policy_dominance(a: PaymentFee, b: PaymentFee, payment: u64)
    requires within(a, b),
    ensures fee_value(a, payment as int) <= fee_value(b, payment as int),
        send_value(a, payment as int) <= send_value(b, payment as int),
        receive_value(a, payment as int) >= receive_value(b, payment as int),
{
    // MECHANICAL_PROOF_SLOT: initially empty ordinary/infer baseline.
}

pub fn composed_policy(a: &PaymentFee, b: &PaymentFee, payment: u64) {
    let allowed = a.is_within(b);
    let combined = (*a).checked_add(*b);
    let charge = a.fee(payment);
    let sent = a.add_to(payment);
    let received = a.subtract_from(payment);
    assert(allowed == within(*a, *b));
    assert(charge.msats == fee_value(*a, payment as int));
    assert(sent.msats == send_value(*a, payment as int));
    assert(received.msats == receive_value(*a, payment as int));
    assert(sent.msats >= payment);
    assert(received.msats <= payment);
    assert(payment + charge.msats <= u64::MAX ==> sent.msats == payment + charge.msats);
    assert(charge.msats <= payment ==> received.msats + charge.msats == payment);
    assert(match combined {
        Some(v) => v.base.msats == a.base.msats + b.base.msats
            && v.parts_per_million == a.parts_per_million + b.parts_per_million,
        None => a.base.msats + b.base.msats > u64::MAX
            || a.parts_per_million + b.parts_per_million > u64::MAX,
    });
    if allowed {
        proof { policy_dominance(*a, *b, payment); }
        let limit_charge = b.fee(payment);
        let limit_sent = b.add_to(payment);
        let limit_received = b.subtract_from(payment);
        assert(charge.msats <= limit_charge.msats);
        assert(sent.msats <= limit_sent.msats);
        assert(received.msats >= limit_received.msats);
    }
}
}
