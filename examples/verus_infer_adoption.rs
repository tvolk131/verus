// rust_verify/tests/example.rs expect-success --infer-goals
//! A small adoption workflow: inferred implementation facts, authored API
//! meaning, and automatic or explicit proof assistance.
//! Verify with --infer-goals --no-cheating; compile with --compile.
use vstd::prelude::*;

// ANCHOR: debit
#[verus_infer]
pub fn debit(balance: u64, amount: u64) -> u64 {
    balance - amount
}
// ANCHOR_END: debit

verus! {
// ANCHOR: coverage
pub fn spend_available(balance: u64, amount: u64) -> (result: u64)
    requires amount <= balance,
    ensures result as int == balance as int - amount as int,
{
    debit(balance, amount)
}
// ANCHOR_END: coverage

pub mod budget {
    use vstd::prelude::*;

    // ANCHOR: view
    pub struct Budget {
        balance: u64,
        capacity: u64,
    }

    impl View for Budget {
        type V = (u64, u64);
        closed spec fn view(&self) -> Self::V {
            (self.balance, self.capacity)
        }
    }

    impl Budget {
        pub open spec fn valid(&self) -> bool {
            self@.0 <= self@.1
        }
    }
    // ANCHOR_END: view

    impl Budget {
        pub fn new(capacity: u64) -> (result: Self)
            ensures result.valid(), result@ == (capacity, capacity),
        {
            Self { balance: capacity, capacity }
        }
    }

    // ANCHOR: state
    impl Budget {
        #[verifier::infer]
        fn debit_raw(&mut self, amount: u64) {
            self.balance = super::debit(self.balance, amount);
        }

        pub fn spend(&mut self, amount: u64)
            requires old(self).valid(), amount <= old(self)@.0,
            ensures
                final(self).valid(),
                final(self)@.0 as int == old(self)@.0 as int - amount as int,
                final(self)@.1 == old(self)@.1,
        {
            self.debit_raw(amount);
        }
    }
    // ANCHOR_END: state
}

#[verifier::infer]
pub fn bucket(key: u64) -> u8 { (key & 15) as u8 }

// ANCHOR: proof
pub fn assisted_bucket_bound(key: u64) {
    let index = bucket(key);
    assert(index < 16);
}

pub fn explicit_bucket_bound(key: u64) {
    let index = bucket(key);
    assert(((key & 15) as u8) < 16) by(bit_vector);
    assert(index < 16);
}
// ANCHOR_END: proof
}

fn main() {
    assert_eq!(spend_available(10, 4), 6);
    let mut budget = budget::Budget::new(10);
    budget.spend(4);
    for key in [0, 15, 16, u64::MAX] {
        assisted_bucket_bound(key);
        explicit_bucket_bound(key);
    }
    println!("Verus Infer adoption example passed");
}
