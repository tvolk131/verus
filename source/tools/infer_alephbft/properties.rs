use vstd::prelude::*;

// The runner checks this alias against the pinned production types/src/lib.rs.
pub type Round = u16;
#[path = "../consensus/src/extension/common_vote.rs"]
mod common_vote;
use common_vote::common_vote;

verus! {
fn parity_schedule(round: u16) {
    let vote = common_vote(round);
    // MANUAL_LEMMA
    assert(vote == (round != 3 && (round <= 4 || (round & 1u16) == 1u16)));
}

// These original remainder-based properties already verify without assistance.
fn ordinary_schedule(round: u16) {
    let vote = common_vote(round);
    assert(vote == (round != 3 && (round <= 4 || round % 2 == 1)));
}

fn alternation(round: u16)
    requires round >= 5, round < u16::MAX,
{
    let a = common_vote(round);
    let b = common_vote(round + 1);
    assert(a != b);
}

fn guarded_parity(round: u16) {
    let vote = common_vote(round);
    if round >= 5 {
        // MANUAL_LEMMA
        assert(vote == ((round & 1u16) == 1u16));
    }
}
}
