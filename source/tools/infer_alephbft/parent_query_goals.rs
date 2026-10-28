use units_api::{NodeMap, ParentQueries};
use vstd::prelude::*;
verus! {

pub struct ParentValidator<Q: ParentQueries> {
    pub parents: Q,
    pub threshold: usize,
}

impl<Q: ParentQueries> ParentValidator<Q> {
    // This bounded proof target composes checked queries. It covers the quorum
    // and future-round stages, not the native complete ControlHash error enum.
    #[verifier::infer]
    pub fn validate(&self, round: u16) -> Result<(), u16> {
        let previous = round - 1;
        if self.parents.count_range(previous, previous) < self.threshold {
            return Err(previous);
        }
        if self.parents.has_after(previous) {
            return Err(round);
        }
        Ok(())
    }
}

pub fn check<Q: ParentQueries>(validator: &ParentValidator<Q>, round: u16) -> (result: Result<
    (),
    u16,
>)
    requires
        round > 0,
    ensures
        result.is_ok() == (validator.parents.count_range_value(
            (round - 1) as u16,
            (round - 1) as u16,
        ) >= validator.threshold && !validator.parents.after_value((round - 1) as u16)),
        validator.parents.count_range_value((round - 1) as u16, (round - 1) as u16)
            < validator.threshold ==> result == Err::<(), u16>((round - 1) as u16),
        validator.parents.count_range_value((round - 1) as u16, (round - 1) as u16)
            >= validator.threshold && validator.parents.after_value((round - 1) as u16) ==> result
            == Err::<(), u16>(round),
{
    validator.validate(round)
}

pub fn check_concrete(validator: &ParentValidator<NodeMap<u16>>, round: u16)
    requires
        round > 0,
{
    let _ = check(validator, round);
}

} // verus!
#[cfg(verus_keep_ghost_body)]
fn main() {}
#[cfg(not(verus_keep_ghost_body))]
fn main() {
    use units_api::{NodeCount, NodeIndex, project_parent_rounds};
    for size in 0..8 {
        for mask in 0..(1usize << size) {
            let mut parents = NodeMap::with_size(NodeCount(size));
            for index in 0..size {
                if mask & (1 << index) != 0 {
                    parents.insert(NodeIndex(index), (index as u64, (index % 5) as u16));
                }
            }
            let rounds = project_parent_rounds(&parents);
            for index in 0..size {
                assert_eq!(
                    rounds.get(NodeIndex(index)).copied(),
                    parents.get(NodeIndex(index)).map(|entry| entry.1)
                );
            }
            let validator = ParentValidator { parents: rounds, threshold: 2 * size / 3 + 1 };
            for round in 1..7 {
                let previous = round - 1;
                let mut count = 0;
                let mut future = false;
                for index in 0..size {
                    if let Some(value) = validator.parents.get(NodeIndex(index)) {
                        count += usize::from(*value == previous);
                        future |= *value > previous;
                    }
                }
                assert_eq!(validator.parents.count_range(previous, previous), count);
                assert_eq!(validator.parents.has_after(previous), future);
                let expected = if count < validator.threshold {
                    Err(previous)
                } else if future {
                    Err(round)
                } else {
                    Ok(())
                };
                assert_eq!(validator.validate(round), expected);
            }
        }
    }
    println!("INFER_PARENT_QUERY_CASES=1530");
}
