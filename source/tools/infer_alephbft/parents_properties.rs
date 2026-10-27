#![feature(proc_macro_hygiene)]
use vstd::prelude::*;

#[path = "../crypto/src/node_count_value.rs"]
mod node_count_value;
pub use node_count_value::NodeCount;
#[path = "../crypto/src/quorum_method.rs"]
mod quorum_method;

// The existing shared collector consumes a scalar threshold. This verified
// adapter connects it to the production NodeCount API, without assumed specs.
// BRIDGE_ATTRIBUTE
pub fn consensus_threshold(n: usize) -> usize { NodeCount(n).consensus_threshold().0 }

#[path = "collector_state.rs"]
mod collector;

verus! {
// No domain restriction: this also checks that every inferred safety predicate
// on the corrected method follows from the ordinary usize type bounds.
fn threshold_properties(n: usize) -> (q: NodeCount)
    ensures
        q.0 as int == (2 * n as int) / 3 + 1,
        q.0 > 0,
        n > 0 ==> q.0 <= n,
        3 * q.0 as int > 2 * n as int,
        3 * (q.0 as int - 1) <= 2 * n as int,
{ NodeCount(n).consensus_threshold() }
}
