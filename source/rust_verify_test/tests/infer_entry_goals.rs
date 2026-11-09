#![feature(rustc_private)]
mod common;
use common::infer::InferCrates;

const REPLACE: &str = r#"
use vstd::prelude::*;
verus! {
pub open spec fn observed(data: Seq<u16>, index: int) -> int {
    data[index] as int % 3
}
pub fn replace(data: &mut Vec<u16>, Ghost(seed): Ghost<Seq<u16>>,
    cell: usize, limit: usize, take: bool, chosen: u16, alternate: u16)
    requires
        cell < old(data)@.len(), seed.len() == old(data)@.len(),
        forall|query: int| 0 <= query < old(data)@.len() && query < limit && query % 2 == 0 ==>
            #[trigger] observed(old(data)@, query) == observed(seed, query),
    ensures
        final(data)@.len() == old(data)@.len(),
        forall|query: int| 0 <= query < final(data)@.len() && query < limit && query % 2 == 0 ==>
            #[trigger] observed(final(data)@, query) ==
                (if query == cell { (if take { chosen } else { alternate }) as int % 3 }
                 else { observed(seed, query) }),
{
    let ghost before = data@;
    let slot = &mut data[cell];
    if take { *slot = chosen; } else { *slot = alternate; }
    assert forall|query: int| 0 <= query < data@.len() && query < limit && query % 2 == 0 implies
        observed(data@, query) ==
            (if query == cell { (if take { chosen } else { alternate }) as int % 3 }
             else { observed(seed, query) })
    by { }
}
}
"#;

fn run(source: &str, enabled: bool, success: bool) {
    let crates = InferCrates::new();
    let mut options = vec!["--no-cheating", "--output-json", "--num-threads", "1"];
    if enabled {
        options.push("--infer-goals");
    }
    let result = crates.build_source("entry_case", source, &[], &options, true);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(result.status.success(), success, "{stderr}");
    if !success {
        assert!(stderr.contains("assertion failed"), "{}", stderr);
        assert!(!crates.vir("entry_case").exists());
    }
}

#[test]
fn authored_forall_uses_only_original_entry_instances() {
    run(REPLACE, true, true);
}

#[test]
fn disabled_entry_assistance_retains_the_routine_failure() {
    run(REPLACE, false, false);
}

#[test]
fn alpha_renaming_does_not_select_by_spelling() {
    run(&REPLACE.replace("query", "position").replace("seed", "initial"), true, true);
}

#[test]
fn wrong_replacement_and_missing_guard_stay_checked() {
    run(&REPLACE.replace("*slot = alternate", "*slot = chosen"), true, false);
    run(&REPLACE.replace("&& query % 2 == 0 implies", "implies"), true, false);
}

#[test]
fn earlier_write_cannot_be_reinterpreted_as_the_entry_snapshot() {
    run(
        &REPLACE.replace("let ghost before = data@;", "data.set(0, 1); let ghost before = data@;"),
        true,
        false,
    );
}
