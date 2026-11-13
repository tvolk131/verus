fn slot(transaction_id: u64) -> TransactionHeader {
    TransactionHeader {
        version: 3,
        user_root: None,
        system_root: None,
        transaction_id: TransactionId(transaction_id),
        corrupt_bytes: None,
    }
}

fn header(index: usize, primary_bad: bool, secondary_bad: bool, two_phase: bool, unflushed: bool) -> UnrepairedDatabaseHeader {
    UnrepairedDatabaseHeader {
        inner: DatabaseHeader {
            primary_slot: index,
            recovery_required: true,
            two_phase_commit: two_phase,
            primary_unflushed: unflushed,
            page_size: 4096,
            region_header_pages: 0,
            region_max_data_pages: 1024,
            full_regions: 2,
            trailing_partial_region_pages: 37,
            transaction_slots: [slot(17), slot(u64::MAX)],
        },
        primary_corrupted: primary_bad,
        secondary_corrupted: secondary_bad,
    }
}

#[test]
fn native_gate_all_flags_and_index_boundaries() {
    for index in [0, 1, 2, usize::MAX] {
        for primary_bad in [false, true] {
            for secondary_bad in [false, true] {
                for two_phase in [false, true] {
                    for unflushed in [false, true] {
                        let result = header(index, primary_bad, secondary_bad, two_phase, unflushed).finalize_transaction_slots();
                        if primary_bad {
                            match result {
                                Err(StorageError::Corrupted(message)) => assert_eq!(message, "Primary commit slot is corrupted"),
                                _ => panic!("corrupt recorded primary was accepted or wrong error"),
                            }
                        } else {
                            let accepted = result.unwrap();
                            assert_eq!(accepted.primary_slot, index);
                            assert!(accepted.recovery_required);
                            assert_eq!(accepted.two_phase_commit, two_phase);
                            assert_eq!(accepted.primary_unflushed, unflushed);
                            assert_eq!(accepted.page_size, 4096);
                            assert_eq!(accepted.region_header_pages, 0);
                            assert_eq!(accepted.region_max_data_pages, 1024);
                            assert_eq!(accepted.full_regions, 2);
                            assert_eq!(accepted.trailing_partial_region_pages, 37);
                            assert_eq!(accepted.transaction_slots[0].transaction_id.0, 17);
                            assert_eq!(accepted.transaction_slots[1].transaction_id.0, u64::MAX);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn source_fragment_preserves_every_error_kind_and_payload() {
    assert!(matches!(normalize_observation(Ok(true)), Ok(true)));
    assert!(matches!(normalize_observation(Ok(false)), Ok(false)));
    assert!(matches!(normalize_observation(Err(StorageError::Corrupted("arbitrary payload".into()))), Ok(false)));
    assert!(matches!(normalize_observation(Err(StorageError::ValueTooLarge(usize::MAX))), Err(StorageError::ValueTooLarge(usize::MAX))));
    let error = io::Error::from_raw_os_error(123);
    match normalize_observation(Err(StorageError::Io(error))) {
        Err(StorageError::Io(error)) => assert_eq!(error.raw_os_error(), Some(123)),
        _ => panic!("I/O error was swallowed"),
    }
    assert!(matches!(normalize_observation(Err(StorageError::Unsupported)), Err(StorageError::Unsupported)));
    assert!(matches!(normalize_observation(Err(StorageError::PreviousIo)), Err(StorageError::PreviousIo)));
    assert!(matches!(normalize_observation(Err(StorageError::DatabaseClosed)), Err(StorageError::DatabaseClosed)));
    let location = panic::Location::caller();
    match normalize_observation(Err(StorageError::LockPoisoned(location))) {
        Err(StorageError::LockPoisoned(actual)) => assert!(std::ptr::eq(location, actual)),
        _ => panic!("poison location was not preserved"),
    }
}

#[test]
fn valid_secondary_does_not_enable_read_only_repair() {
    assert!(matches!(header(0, true, false, false, false).finalize_transaction_slots(), Err(StorageError::Corrupted(_))));
    let accepted = header(usize::MAX, false, false, false, false).finalize_transaction_slots().unwrap();
    assert_eq!(accepted.primary_slot, usize::MAX);
}
