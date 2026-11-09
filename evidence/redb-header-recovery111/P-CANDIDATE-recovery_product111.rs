use vstd::prelude::*;
use vstd::assert_seqs_equal;
use recovery_reencoding021::*;
use recovery_roundtrip021::*;
#[path = "modules/contract111.rs"]
pub mod contract;
#[path = "modules/composition110.rs"]
pub mod product;

verus! {

pub open spec fn expected_file_length(expected: DatabaseHeader, file_len: u64) -> bool {
    &&& !expected.recovery_required ==> layout_bytes(header_layout(expected)) <= file_len
    &&& (expected.recovery_required || layout_bytes(header_layout(expected)) != file_len)
        ==> length_accepted(file_len, expected)
}

pub open spec fn expected_domain(expected: DatabaseHeader, file_len: u64) -> bool {
    &&& header_roundtrip_domain(expected)
    &&& expected.two_phase_commit
    &&& selected_slot_stable(expected)
    &&& expected_file_length(expected, file_len)
}

pub open spec fn selected_payload_matches(actual: DatabaseHeader, expected: DatabaseHeader) -> bool {
    &&& actual.primary_slot == expected.primary_slot
    &&& actual.primary_slot < 2
    &&& actual.transaction_slots@[actual.primary_slot as int].transaction_id
        == expected.transaction_slots@[expected.primary_slot as int].transaction_id
    &&& actual.transaction_slots@[actual.primary_slot as int].user_root
        == expected.transaction_slots@[expected.primary_slot as int].user_root
    &&& actual.transaction_slots@[actual.primary_slot as int].system_root
        == expected.transaction_slots@[expected.primary_slot as int].system_root
}

pub open spec fn expected_parser_header(expected: DatabaseHeader) -> UnrepairedDatabaseHeader {
    UnrepairedDatabaseHeader {
        inner: expected,
        primary_corrupted: expected.transaction_slots@[expected.primary_slot as int].corrupt_bytes.is_some(),
        secondary_corrupted: expected.transaction_slots@[(expected.primary_slot ^ 1) as int].corrupt_bytes.is_some(),
    }
}

pub proof fn expected_encoding_is_parsed(expected: DatabaseHeader)
    requires header_roundtrip_domain(expected),
    ensures std_bool_tcb_parsed_header(database_bytes(expected, true), 4096,
        expected_parser_header(expected)),
{
    hide(hash94::header_hash);
    hide(hash94::scalar_spec);
    header_encoding_geometry(expected);
    other_slot_bounds(expected.primary_slot);
    slot_encoding_reparses(expected.transaction_slots@[0]);
    slot_encoding_reparses(expected.transaction_slots@[1]);
    assert_seqs_equal!(database_bytes(expected, true).subrange(64, 192)
        == slot_bytes(expected.transaction_slots@[0]));
    assert_seqs_equal!(database_bytes(expected, true).subrange(192, 320)
        == slot_bytes(expected.transaction_slots@[1]));
    if expected.primary_slot != 0 {
        assert(expected.primary_slot == 1);
    }
}

pub proof fn encoded_origin_is_accepted(bytes: Seq<u8>, expected: DatabaseHeader, file_len: u64)
    requires bytes == database_bytes(expected, true), expected_domain(expected, file_len),
    ensures bytes.len() == 320, finalization_accepts_bytes(bytes, file_len),
{
    hide(hash94::header_hash);
    hide(hash94::scalar_spec);
    expected_encoding_is_parsed(expected);
    let parsed = expected_parser_header(expected);
    assert(!parsed.primary_corrupted);
    assert(!rejects(parsed));
    assert(!recovery_rejects(file_len, parsed));
    assert(exists|witness: UnrepairedDatabaseHeader|
        std_bool_tcb_parsed_header(bytes, 4096, witness)
            && !recovery_rejects(file_len, witness));
}

pub proof fn encoded_origin_selects_payload(bytes: Seq<u8>, expected: DatabaseHeader,
    file_len: u64, actual: DatabaseHeader, clean: bool)
    requires bytes == database_bytes(expected, true), expected_domain(expected, file_len),
        finalized_from_bytes(bytes, file_len, actual, clean),
    ensures selected_payload_matches(actual, expected),
        actual.transaction_slots@ == expected.transaction_slots@,
        layout_bytes(header_layout(actual)) == file_len,
{
    hide(hash94::header_hash);
    hide(hash94::scalar_spec);
    let parsed = choose|parsed: UnrepairedDatabaseHeader|
        std_bool_tcb_parsed_header(bytes, 4096, parsed)
            && finalized_from_header(parsed, file_len, actual, clean);
    encoded_header_reparses(expected, parsed);
    assert(!prefers_secondary(parsed));
}

pub fn decode_expected_header(data: &[u8], expected: &DatabaseHeader, file_len: u64)
    -> (result: Result<(DatabaseHeader, bool), DatabaseError>)
    requires data@ == database_bytes(*expected, true),
        header_roundtrip_domain(*expected), expected.two_phase_commit,
        selected_slot_stable(*expected), expected_file_length(*expected, file_len),
    ensures result.is_ok(),
        match result {
            Ok((actual, clean)) => finalized_from_bytes(data@, file_len, actual, clean)
                && selected_payload_matches(actual, *expected)
                && actual.transaction_slots@ == expected.transaction_slots@
                && layout_bytes(header_layout(actual)) == file_len,
            Err(_) => false,
        },
{
    proof { encoded_origin_is_accepted(data@, *expected, file_len); }
    let result = parse_and_finalize(data, file_len);
    match result {
        Ok((actual, clean)) => {
            proof { encoded_origin_selects_payload(data@, *expected, file_len, actual, clean); }
            Ok((actual, clean))
        },
        Err(error) => {
            proof { assert(false); }
            Err(error)
        },
    }
}

pub proof fn public_ok_recovers_expected(start: product::Product,
    config: product::acknowledgement::Config, actions: Seq<product::Action>,
    expected: DatabaseHeader, file_len: u64)
    requires product::acknowledgement::local_mode(config),
        product::product_invariant(start, config),
        start.control == product::acknowledgement::initial(config),
        product::run(start, config, actions).control.status == 1,
        start.geometry == file_len,
        start.second_header == database_bytes(expected, true),
        expected_domain(expected, file_len),
    ensures ({
        let final_state = product::run(start, config, actions);
        let durable_header = final_state.image.durable.subrange(0, 320);
        &&& !final_state.blocked
        &&& product::certificate(final_state, final_state.first_index, 24, final_state.first_header)
        &&& product::certificate(final_state, final_state.second_index, 28, final_state.second_header)
        &&& final_state.first_index < final_state.second_index
        &&& product::drained(final_state.pending)
        &&& durable_header == database_bytes(expected, true)
        &&& final_state.image.durable.len() == file_len
        &&& finalization_accepts_bytes(durable_header, file_len)
        &&& forall|actual: DatabaseHeader, clean: bool|
            finalized_from_bytes(durable_header, file_len, actual, clean)
                ==> selected_payload_matches(actual, expected)
        &&& 0 <= final_state.control.first_write < final_state.control.first_barrier
            < final_state.control.second_write < final_state.control.final_barrier
            < final_state.control.publication < final_state.control.acknowledgement
    }),
{
    hide(hash94::header_hash);
    hide(hash94::scalar_spec);
    product::checked_run(start, config, actions);
    product::public_ok_composes(start, config, actions);
    let final_state = product::run(start, config, actions);
    let durable_header = final_state.image.durable.subrange(0, 320);
    assert(final_state.second_header == start.second_header);
    encoded_origin_is_accepted(durable_header, expected, file_len);
    assert forall|actual: DatabaseHeader, clean: bool|
        finalized_from_bytes(durable_header, file_len, actual, clean)
            implies selected_payload_matches(actual, expected) by {
        encoded_origin_selects_payload(durable_header, expected, file_len, actual, clean);
    }
}

}
