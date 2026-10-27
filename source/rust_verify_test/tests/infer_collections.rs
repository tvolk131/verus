#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] generic_shared_vec_length_and_copy_read verus_code! {
        use vstd::prelude::*;
        #[verifier::infer] fn length<T>(v: &Vec<T>) -> usize { v.len() }
        #[verifier::infer] fn read<T: Copy>(v: &Vec<T>, i: usize) -> T { v[i] }
        fn properties<T: Copy>(v: &Vec<T>, i: usize) -> (x: T)
            requires i < v@.len(),
            ensures x == v@[i as int],
        { let n = length(v); assert(n == v@.len()); read(v, i) }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("soundness: PROVED")).count(), 2);
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("safe-domain maximality: PROVED")).count(), 2);
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(), 2);
    }
}

test_verify_one_file! {
    #[test] single_slot_effect_preserves_length_and_all_other_slots verus_code! {
        use vstd::prelude::*;
        #[verifier::infer_precision_rlimit(1)]
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) { v[i] = x; }
        fn caller<T: Copy>(v: &mut Vec<T>, i: usize, x: T)
            requires i < old(v)@.len(),
            ensures final(v)@.len() == old(v)@.len(), final(v)@[i as int] == x,
                forall|j: int| 0 <= j < final(v)@.len() && j != i ==> final(v)@[j] == old(v)@[j],
        { replace(v, i, x); }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d| d.message.contains("VERIFIED SOUND; maximal safe domain: NOT ESTABLISHED; exact output: NOT ESTABLISHED")));
        assert!(!diags.notes.iter().any(|d| d.message.contains("output exactness: PROVED")));
    }
}

test_verify_one_file! {
    #[test] guarded_assignment_accepts_every_index verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) {
            if i < v.len() { v[i] = x; }
        }
        fn caller<T: Copy>(v: &mut Vec<T>, i: usize, x: T)
            ensures final(v)@ == if i < old(v)@.len() { old(v)@.update(i as int, x) } else { old(v)@ },
        { replace(v, i, x); }
    } => Ok(())
}

test_verify_one_file! {
    #[test] early_guard_accepts_every_index verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) {
            if i >= v.len() { return; }
            v[i] = x;
        }
        fn caller<T: Copy>(v: &mut Vec<T>, i: usize, x: T)
            ensures final(v)@ == if i < old(v)@.len() { old(v)@.update(i as int, x) } else { old(v)@ },
        { replace(v, i, x); }
    } => Ok(())
}

test_verify_one_file! {
    #[test] concrete_scalar_vectors_use_the_same_models verus_code! {
        use vstd::prelude::*;
        #[verifier::infer] fn length(v: &Vec<u8>) -> usize { v.len() }
        #[verifier::infer] fn read(v: &Vec<u8>, i: usize) -> u8 { v[i] }
        #[verifier::infer(sound)] fn replace(v: &mut Vec<u8>, i: usize, x: u8) { v[i] = x; }
        fn caller(v: &mut Vec<u8>, x: u8)
            requires old(v)@.len() > 0,
            ensures final(v)@[0] == x, final(v)@.len() == old(v)@.len(),
        { let n = length(&*v); assert(n > 0); let _ = read(&*v, 0); replace(v, 0, x); }
    } => Ok(())
}

test_verify_one_file! {
    #[test] unsafe_write_call_still_fails verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) { v[i] = x; }
        fn bad(v: &mut Vec<u8>) { let n = v.len(); replace(v, n, 1); /* FAILS */ }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] unsafe_read_call_still_fails verus_code! {
        use vstd::prelude::*;
        #[verifier::infer] fn read<T: Copy>(v: &Vec<T>, i: usize) -> T { v[i] }
        fn bad(v: &Vec<u8>) { let _ = read(v, v.len()); /* FAILS */ }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] sound_state_summary_does_not_invent_other_updates verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) { v[i] = x; }
        fn bad(v: &mut Vec<u8>) requires old(v)@.len() >= 2, {
            replace(v, 0, 7);
            assert(v@[1] == 7); // FAILS
        }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] exact_state_precision_is_rejected verus_code! {
        use vstd::prelude::*;
        #[verifier::infer] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) { v[i] = x; }
    } => Err(e) => assert_vir_error_msg(e, "state maximality/exactness are not yet checked")
}

test_verify_one_file! {
    #[test] extra_mutation_is_not_ignored verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) {
            v[i] = x; v[0] = x;
        }
    } => Err(e) => assert_vir_error_msg(e, "single slot assignment")
}

test_verify_one_file! {
    #[test] proof_assumptions_are_not_a_state_inference_escape verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace(v: &mut Vec<u8>, i: usize, x: u8) {
            proof { assume(false); }
            v[i] = x;
        }
    } => Err(e) => assert_vir_error_msg(e, "single slot assignment")
}

test_verify_one_file! {
    #[test] mutable_alias_scaffolding_is_outside_the_template verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) {
            let other = &mut *v; other[i] = x;
        }
    } => Err(e) => assert_vir_error_msg(e, "single slot assignment")
}

test_verify_one_file! {
    #[test] moving_noncopy_elements_is_outside_the_template verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T>(v: &mut Vec<T>, i: usize, x: T) { v[i] = x; }
    } => Err(e) => assert_vir_error_msg(e, "Verus Infer unsupported")
}

test_verify_one_file! {
    #[test] owned_vectors_are_outside_the_template verus_code! {
        use vstd::prelude::*;
        #[verifier::infer] fn length<T>(v: Vec<T>) -> usize { v.len() }
    } => Err(e) => assert_vir_error_msg(e, "Verus Infer unsupported")
}

test_verify_one_file! {
    #[test] supplied_count_invariant_composes_with_the_inferred_effect verus_code! {
        use vstd::prelude::*;
        #[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, x: T) { v[i] = x; }
        #[verifier::infer] fn length<T>(v: &Vec<T>) -> usize { v.len() }
        spec fn count(s: Seq<bool>, n: nat) -> nat
            recommends n <= s.len(), decreases n,
        { if n == 0 { 0 } else { count(s, (n-1) as nat) + if s[n-1] { 1nat } else { 0nat } } }
        proof fn count_update(s: Seq<bool>, i: int, value: bool, n: nat)
            requires 0 <= i < s.len(), n <= s.len(),
            ensures count(s.update(i, value), n) as int == count(s, n) as int
                + if i < n { (if value {1int} else {0int}) - (if s[i] {1int} else {0int}) } else {0int},
            decreases n,
        { if n > 0 { count_update(s, i, value, (n-1) as nat); } }
        proof fn count_bound(s: Seq<bool>, n: nat)
            requires n <= s.len(), ensures count(s, n) <= n, decreases n,
        { if n > 0 { count_bound(s, (n-1) as nat); } }
        fn preserve(v: &mut Vec<bool>, total: &mut usize, i: usize)
            requires i < old(v)@.len(), !old(v)@[i as int],
                *old(total) as int == count(old(v)@, old(v)@.len()),
            ensures *final(total) as int == count(final(v)@, final(v)@.len()),
        {
            let n = length(v);
            proof {
                count_update(v@, i as int, true, v@.len());
                count_bound(v@.update(i as int, true), v@.len());
            }
            replace(v, i, true);
            *total += 1;
        }
    } => Ok(())
}
