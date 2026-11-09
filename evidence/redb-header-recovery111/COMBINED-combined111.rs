#![allow(dead_code, unused_imports)]
use vstd::prelude::*;
use recovery_reencoding021::*;
use recovery_roundtrip021::*;
use rootcodec94::BtreeHeader;
#[path = "/private/tmp/redb-header-recovery111-H-20261108T052400Z/staging111.rs"]
pub mod staging;
#[path = "/private/tmp/redb-header-recovery111-P-cPSXKcfk/recovery_product111.rs"]
pub mod recovery;
use staging::contract111::*;

verus! {

pub fn stage_and_recover(header: &mut DatabaseHeader, transaction_id: TransactionId,
    user_root: Option<BtreeHeader>, system_root: Option<BtreeHeader>,
    data: &[u8], file_len: u64,
    Ghost(start): Ghost<recovery::product::Product>,
    Ghost(config): Ghost<recovery::product::acknowledgement::Config>,
    Ghost(actions): Ghost<Seq<recovery::product::Action>>)
    -> (result: Result<(DatabaseHeader, bool), DatabaseError>)
    requires
        stage_domain(*old(header), transaction_id, user_root, system_root),
        recovery::expected_file_length(committed_header(prepared_header(*old(header),
            transaction_id, user_root, system_root)), file_len),
        recovery::product::acknowledgement::local_mode(config),
        recovery::product::product_invariant(start, config),
        start.control == recovery::product::acknowledgement::initial(config),
        recovery::product::run(start, config, actions).control.status == 1,
        start.geometry == file_len,
        start.first_header == database_bytes(prepared_header(*old(header),
            transaction_id, user_root, system_root), true),
        start.second_header == database_bytes(committed_header(prepared_header(*old(header),
            transaction_id, user_root, system_root)), true),
        data@ == recovery::product::run(start, config, actions).image.durable.subrange(0, 320),
    ensures
        *final(header) == committed_header(prepared_header(*old(header),
            transaction_id, user_root, system_root)),
        result.is_ok(),
        match result {
            Ok((actual, clean)) => {
                &&& finalized_from_bytes(data@, file_len, actual, clean)
                &&& actual.primary_slot < 2
                &&& actual.primary_slot == old(header).primary_slot ^ 1
                &&& actual.transaction_slots@[actual.primary_slot as int].transaction_id == transaction_id
                &&& actual.transaction_slots@[actual.primary_slot as int].user_root == user_root
                &&& actual.transaction_slots@[actual.primary_slot as int].system_root == system_root
                &&& layout_bytes(header_layout(actual)) == file_len
            },
            Err(_) => false,
        },
{
    let ghost before = *header;
    let (first, final_bytes) = staging::stage_two_phase(header, transaction_id, user_root, system_root);
    proof {
        other_slot_bounds(before.primary_slot);
        let primary = before.primary_slot;
        if primary == 0 {
            assert(primary ^ 1 == 1) by(bit_vector) requires primary == 0;
        } else {
            assert(primary == 1);
            assert(primary ^ 1 == 0) by(bit_vector) requires primary == 1;
        }
        assert(header.transaction_slots@[header.primary_slot as int].corrupt_bytes.is_none());
        assert(selected_slot_stable(*header));
        assert(recovery::expected_domain(*header, file_len));
        assert(first@ == start.first_header);
        assert(final_bytes@ == start.second_header);
        recovery::public_ok_recovers_expected(start, config, actions, *header, file_len);
        assert(data@ == database_bytes(*header, true));
    }
    recovery::decode_expected_header(data, header, file_len)
}

}
