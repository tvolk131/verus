fn slot_snapshot(slot: &TransactionHeader)
    -> (u8, Option<BtreeHeader>, Option<BtreeHeader>, u64, Option<[u8; TRANSACTION_SIZE]>)
{
    (slot.version, slot.user_root, slot.system_root, slot.transaction_id.0, slot.corrupt_bytes)
}

fn transaction(transaction_id: u64, byte: u8) -> TransactionHeader {
    TransactionHeader {
        version: byte,
        user_root: Some(BtreeHeader {
            root: PageNumber { region: 7, page_index: 11, page_order: 3 },
            checksum: u128::MAX,
            length: 97,
        }),
        system_root: None,
        transaction_id: TransactionId(transaction_id),
        corrupt_bytes: Some([byte; TRANSACTION_SIZE]),
    }
}

#[test]
fn native_selection_boundary_matrix() {
    let ids = [0, 1, 2, u64::MAX - 1, u64::MAX];
    let mut cases = 0;
    for primary_slot in 0..2 {
        for two_phase_commit in [false, true] {
            for primary_corrupted in [false, true] {
                for secondary_corrupted in [false, true] {
                    for recovery_required in [false, true] {
                        for primary_unflushed in [false, true] {
                            for first_id in ids {
                                for second_id in ids {
                                    let mut header = UnrepairedDatabaseHeader {
                                        inner: DatabaseHeader {
                                            primary_slot, recovery_required, two_phase_commit, primary_unflushed,
                                            page_size: 13, region_header_pages: 17, region_max_data_pages: 19,
                                            full_regions: 23, trailing_partial_region_pages: 29,
                                            transaction_slots: [transaction(first_id, 31), transaction(second_id, 37)],
                                        },
                                        primary_corrupted, secondary_corrupted,
                                    };
                                    let before = header.inner.clone();
                                    let primary_id = before.transaction_slots[primary_slot].transaction_id.0;
                                    let secondary_id = before.transaction_slots[primary_slot ^ 1].transaction_id.0;
                                    let expected = match (two_phase_commit, primary_corrupted, secondary_corrupted) {
                                        (true, true, _) => Err("Primary is corrupted despite 2-phase commit"),
                                        (true, false, _) => Ok(true),
                                        (false, true, true) => Err("Both commit slots are corrupted"),
                                        (false, true, false) => Ok(false),
                                        (false, false, true) => Ok(true),
                                        (false, false, false) => Ok(secondary_id <= primary_id),
                                    };
                                    let actual = header.select_primary_slot();
                                    let kept = match (expected, actual) {
                                        (Ok(expected_kept), Ok(actual_kept)) => {
                                            assert_eq!(expected_kept, actual_kept);
                                            actual_kept
                                        }
                                        (Err(expected_message), Err(StorageError::Corrupted(actual_message))) => {
                                            assert_eq!(expected_message, actual_message);
                                            true
                                        }
                                        (expected, actual) => panic!("mismatched result: {expected:?}, {actual:?}"),
                                    };
                                    assert_eq!(header.inner.primary_slot, if kept { primary_slot } else { primary_slot ^ 1 });
                                    assert_eq!(header.primary_corrupted, primary_corrupted);
                                    assert_eq!(header.secondary_corrupted, secondary_corrupted);
                                    assert_eq!(header.inner.recovery_required, before.recovery_required);
                                    assert_eq!(header.inner.two_phase_commit, before.two_phase_commit);
                                    assert_eq!(header.inner.primary_unflushed, before.primary_unflushed);
                                    assert_eq!(header.inner.page_size, before.page_size);
                                    assert_eq!(header.inner.region_header_pages, before.region_header_pages);
                                    assert_eq!(header.inner.region_max_data_pages, before.region_max_data_pages);
                                    assert_eq!(header.inner.full_regions, before.full_regions);
                                    assert_eq!(header.inner.trailing_partial_region_pages, before.trailing_partial_region_pages);
                                    for index in 0..2 {
                                        assert_eq!(slot_snapshot(&header.inner.transaction_slots[index]),
                                                   slot_snapshot(&before.transaction_slots[index]));
                                    }
                                    cases += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 1600);
    println!("native full-error-enum cases: {cases}");
}
