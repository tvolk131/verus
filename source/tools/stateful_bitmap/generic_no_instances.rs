use vstd::prelude::*;

verus! {
pub open spec fn observed32(data: Seq<u32>, index: int, lane: u32) -> bool {
    data[index] & (1u32 << lane) != 0
}

pub proof fn toggle_fact(value: u32, lane: u32)
    requires lane < 32,
    ensures ((value ^ (1u32 << lane)) & (1u32 << lane) != 0) == !(value & (1u32 << lane) != 0),
{
    assert(((value ^ (1u32 << lane)) & (1u32 << lane) != 0) == !(value & (1u32 << lane) != 0))
        by(bit_vector) requires lane < 32;
}

pub fn toggle_cell(data: &mut Vec<u32>, Ghost(reference): Ghost<Seq<u32>>, cell: usize, lane: u32)
    requires
        cell < old(data)@.len(), lane < 32,
        reference.len() == old(data)@.len(),
        forall|query: int| 0 <= query < old(data)@.len() ==>
            #[trigger] observed32(old(data)@, query, lane) == observed32(reference, query, lane),
    ensures
        final(data)@.len() == old(data)@.len(),
        forall|query: int| 0 <= query < final(data)@.len() ==>
            #[trigger] observed32(final(data)@, query, lane) ==
                (if query == cell { !observed32(reference, query, lane) } else { observed32(reference, query, lane) }),
{
    let ghost before = data@;
    let slot = &mut data[cell];
    *slot ^= 1u32 << lane;
    assert forall|query: int| 0 <= query < data@.len() implies
        observed32(data@, query, lane) ==
            (if query == cell { !observed32(reference, query, lane) } else { observed32(reference, query, lane) })
    by {
        if query == cell {
            toggle_fact(before[cell as int], lane);
        }
    }
}

pub open spec fn residue(data: Seq<u16>, index: int) -> int {
    data[index] as int % 3
}

pub fn selective_replace(data: &mut Vec<u16>, Ghost(seed): Ghost<Seq<u16>>,
    cell: usize, limit: usize, take: bool, chosen: u16, alternate: u16)
    requires
        cell < old(data)@.len(), seed.len() == old(data)@.len(),
        forall|query: int| 0 <= query < old(data)@.len() && query < limit && query % 2 == 0 ==>
            #[trigger] residue(old(data)@, query) == residue(seed, query),
    ensures
        final(data)@.len() == old(data)@.len(),
        forall|query: int| 0 <= query < final(data)@.len() && query < limit && query % 2 == 0 ==>
            #[trigger] residue(final(data)@, query) ==
                (if query == cell { (if take { chosen } else { alternate }) as int % 3 }
                 else { residue(seed, query) }),
{
    let ghost before = data@;
    let slot = &mut data[cell];
    if take {
        *slot = chosen;
    } else {
        *slot = alternate;
    }
    assert forall|query: int| 0 <= query < data@.len() && query < limit && query % 2 == 0 implies
        residue(data@, query) ==
            (if query == cell { (if take { chosen } else { alternate }) as int % 3 }
             else { residue(seed, query) })
    by {
    }
}
}
