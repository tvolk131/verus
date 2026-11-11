# Correction C1, after freeze 1ef4207e

First runs /tmp/redb-r2-baseline-01, ordinary and infer, reject E0308 before verification because this accepted vstd has finite Set::new -> Option<Set>. This is an authored specification/setup failure, not a REDB unsupported API or native correctness result. component.rs remains byte-for-byte frozen.

corrected.rs changes only the allocated-set definition from the ill-typed Set::new predicate to Set::<int>::range(0, leaf.len as int).filter(membership). This denotes exactly the same intended bounded logical set with a finite type. No domain/body/property changes, no assumptions, no mechanical proof progress. The corrected definition and all other specs are frozen before corrected trials. A later proof would have to prove ordinary membership/count consequences with the checked finite range/filter laws, charging that as mechanical work.
