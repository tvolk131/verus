// Exercise the original outer body, real SCALE/hash/signature wrappers and mock
// Keychain. The mock does not reject out-of-range creators: retain that case as
// an explicit input-domain/panic control rather than assuming authentication fixes it.
#[test]
fn outer_validation_corpus() {
    use crate::units::{ControlHash, FullUnit, UncheckedSignedUnit, Unit};
    use crate::NodeMap;
    use aleph_bft_mock::{Data, Hasher64, Signature};
    let keychain = Keychain::new(NodeCount(3), NodeIndex(0));
    let validator = Validator::new(0, keychain, 1);
    let mut cases = 0;
    let mut panics = 0;
    for authenticated in [false, true] {
        for session in [0, 43] {
            for round in [0, 1, 2, u16::MAX] {
                for members in [0, 3, 4] {
                    for creator in [0, 3] {
                        let mut parents = NodeMap::with_size(NodeCount(members));
                        if round > 0 {
                            for index in 0..members {
                                parents.insert(NodeIndex(index), ([index as u8; 8], round - 1));
                            }
                        }
                        let control = ControlHash::<Hasher64>::new(&parents);
                        let pre = PreUnit::new(NodeIndex(creator), round, control);
                        let unit = FullUnit::<Hasher64, Data>::new(pre, None, session);
                        let message = Unit::hash(&unit).as_ref().to_vec();
                        let signer = if authenticated { creator } else { creator + 1 };
                        let signature = Signature::new(message, NodeIndex(signer));
                        let encoded = (unit.clone(), signature).encode();
                        let unchecked: UncheckedSignedUnit<Hasher64, Data, Signature> =
                            Decode::decode(&mut encoded.as_slice()).unwrap();
                        if authenticated && session == 0 && round == 1 && members == 3 && creator == 3 {
                            assert!(std::panic::catch_unwind(|| validator.validate_unit(unchecked)).is_err());
                            panics += 1;
                            continue;
                        }
                        let expected = if !authenticated {
                            Err(WrongSignature(unchecked.clone()))
                        } else if session != 0 {
                            Err(WrongSession(unit.clone()))
                        } else if round > 1 {
                            Err(RoundTooHigh(unit.clone()))
                        } else if members != 3 {
                            Err(WrongNumberOfMembers(unit.as_pre_unit().clone()))
                        } else {
                            unit.as_pre_unit().control_hash().validate(unit.coord())
                                .map_err(|error| ParentValidationFailed(unit.as_pre_unit().clone(), error))
                                .map(|()| unchecked.clone().check(&keychain).unwrap())
                        };
                        assert_eq!(validator.validate_unit(unchecked), expected);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!((cases, panics), (95, 1));
    println!("NATIVE_OUTER_VALIDATION_CASES=95;CREATOR_PANIC=1");
}
