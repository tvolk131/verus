//! Typed admission only. All positives MUST still fail at the later inactive gate.
#![feature(rustc_private)]
mod common;
use common::infer::InferCrates;
const FLAG: &str = "--experimental-owned-unsafe-cell";
const LATER: &str = "inactive: F1 admission passed;";
const FROZEN: &str =
    include_str!("../../tools/research/redb_crash_protocol/native_state_link/owned_goals.rs");
fn source(body: &str) -> String {
    format!(
        "use vstd::prelude::*; use core::cell::UnsafeCell; type C=UnsafeCell<u64>; verus! {{ {body} }}"
    )
}
fn raw_source(body: &str) -> String {
    format!("use vstd::prelude::*; {body}")
}
fn run(src: &str, infer: bool, extra: &[&str]) -> std::process::Output {
    let c = InferCrates::new();
    let file = c.directory.path().join("admission.rs");
    std::fs::write(&file, src).unwrap();
    let mut cmd = std::process::Command::new(c.profile.join("verus"));
    cmd.arg(file)
        .args([
            "--crate-type=lib",
            "--crate-name=admission",
            "--edition=2024",
            "--no-cheating",
            "--compile",
            FLAG,
            "--output-json",
        ])
        .arg("-o")
        .arg(c.directory.path().join("libadmission.rlib"))
        .env("VERUS_INFER_CACHE", c.directory.path().join("fresh-cache"))
        .args(extra);
    if infer {
        cmd.arg("--infer-goals");
    }
    cmd.output().unwrap()
}
std::thread_local! {static FAILURES:std::cell::RefCell<Vec<String>>=const {std::cell::RefCell::new(vec![])};}
fn outcome(label: &str, r: std::process::Output, expected: &str, positive: bool) {
    let err = String::from_utf8_lossy(&r.stderr);
    let mut failures = vec![];
    if r.status.success() {
        failures.push(format!("{label}: admission cannot enable body proofs"));
    }
    if !err.contains(expected) {
        failures.push(format!("{label}: expected {expected:?}\n{err}"));
    }
    if !positive && err.contains(LATER) {
        failures.push(format!("{label}: unsupported unit admitted"));
    }
    match serde_json::from_slice::<serde_json::Value>(&r.stdout) {
        Ok(j) if j["verification-results"]["success"] == false => {
            let infer = &j["verus-infer"];
            if j["func-details"] != serde_json::json!({})
                || infer["functions"] != serde_json::json!([])
                || infer["requested_functions"] != 0
                || infer["generated_functions"] != 0
                || infer["verification_scope"]["checked_exec_bodies"] != 0
                || infer["verification_scope"]["checked_proof_bodies"] != 0
                || infer["interface_written"] != false
            {
                failures.push(format!(
                    "{label}: admission/refusal must not create proof/helper/interface credit: {j}"
                ));
            }
        }
        _ => failures.push(format!("{label}: missing honest failure JSON\n{err}")),
    }
    FAILURES.with(|f| f.borrow_mut().extend(failures));
}
fn finish() {
    let errors = FAILURES.with(|f| std::mem::take(&mut *f.borrow_mut()));
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}
#[test]
fn unchanged_positives_and_scalar_headers_reach_only_later_inactive_gate() {
    let cases = [
        ("frozen", FROZEN.to_string()),
        (
            "alias_and_acyclic",
            source(
                "fn make(v:u64)->C { C::new(v) } fn read(c:C)->u64 {c.into_inner()} pub fn client(v:u64)->(r:u64) ensures r==v, {read(make(v))}",
            ),
        ),
        (
            "exclusive_helper",
            source(
                "fn inner<'a>(c:&'a mut C)->&'a mut u64 {c.get_mut()} pub fn client(c:&mut C,v:u64)->(r:u64) ensures r==v, {*inner(c)=v;*inner(c)}",
            ),
        ),
        (
            "spec_acyclic",
            source(
                "spec fn eq(v:u64,w:u64)->bool {v==w} pub fn client(v:u64)->(r:u64) ensures eq(r,v), {v}",
            ),
        ),
        ("scalar_tuple", source("pub fn pair(v:(u64,bool))->(r:(u64,bool)) ensures r==v, {v}")),
        (
            "scalar_quantifier",
            source("pub fn q(v:u64)->(r:u64) ensures forall|x:u64| x==x, r==v, {v}"),
        ),
        ("requires", source("pub fn req(v:u64)->(r:u64) requires v==v, ensures r==v, {v}")),
        (
            "nested_checked",
            source(
                "pub fn outer(v:u64)->u64 { #[verifier::verify] fn inner(v:u64)->u64 {v} inner(v) }",
            ),
        ),
        (
            "good_macro",
            format!(
                "macro_rules! scalar {{ ($v:expr)=>{{$v}}; }} {}",
                source("pub fn good(v:u64)->(r:u64) ensures r==v, {scalar!(v)}")
            ),
        ),
    ];
    for infer in [false, true] {
        for (label, s) in &cases {
            outcome(label, run(s, infer, &[]), LATER, true);
        }
    }
    for infer in [false, true] {
        outcome(
            "frozen repeated",
            run(FROZEN, infer, &[]),
            "functions=3, bodies=6, headers=3",
            true,
        );
    }
    finish();
}
#[test]
fn complete_unit_rejects_typed_escapes_captures_modes_and_recursion() {
    let cases = [
        (
            "unsupported_native_from_mut",
            "pub fn bad(x:&mut u64)->&mut C {C::from_mut(x)}",
            "unsupported native operation",
        ),
        (
            "spec_recursion",
            "spec fn recurse(v:u64)->bool {recurse(v)}",
            "actual executable/spec local recursion",
        ),
        (
            "dead_capture",
            "pub fn bad(c:C)->u64 {if false {let f=||c.into_inner();0} else {0}}",
            "closure/capture",
        ),
        ("shared_signature", "pub fn bad(c:&C)->u64 {0}", "shared reference/escape"),
        ("raw_signature", "pub fn bad(c:&mut C)->*mut u64 {c.get()}", "raw pointer/escape"),
        (
            "native_shared_operation",
            "pub fn bad(v:u64)->u64 {let c=C::new(v); let p=c.get();v}",
            "raw pointer/escape",
        ),
        ("shared_adjustment", "pub fn bad(c:&mut C)->u64 {let s=&*c;0}", "shared reference/escape"),
        ("cell_tuple", "pub fn bad(v:(C,u64))->u64 {v.1}", "closed scalar/tuple/native-cell"),
        ("container_without_cell", "pub fn bad(v:Option<u64>)->u64 {0}", "user ADT/container"),
        ("nested_cell", "pub fn bad(v:UnsafeCell<C>)->u64 {0}", "non-native-u64"),
        ("array", "pub fn bad(v:[u64;2])->u64 {0}", "closed scalar/tuple/native-cell"),
        (
            "trait_premise",
            "pub fn bad(c:C)->u64 where C:Send {0}",
            "local trait/type/projection bounds",
        ),
        ("generic", "pub fn bad<T>(v:u64)->u64 {v}", "type/const generics"),
        ("const_generic", "pub fn bad<const N:usize>(v:u64)->u64 {v}", "type/const generics"),
        (
            "function_value",
            "pub fn bad(v:u64)->u64 {let f=C::new;f(v).into_inner()}",
            "function value/callback",
        ),
        (
            "closure_owned_capture",
            "pub fn bad(v:u64)->u64 {let c=C::new(v);let f=move || c.into_inner();f()}",
            "closure/capture",
        ),
        (
            "closure_mut_capture",
            "pub fn bad(c:&mut C,v:u64)->u64 {let mut f=|| {*c.get_mut()=v;};f();v}",
            "closure/capture",
        ),
        ("spec_function_cell", "spec fn live(c:C)->bool {true}", "spec surface/capture"),
        ("ghost_parameter", "pub fn bad(#[verifier::spec] c:C)->u64 {0}", "spec surface/capture"),
        (
            "ghost_result",
            "#[verifier::returns(spec)] pub fn bad(v:u64)->C {C::new(v)}",
            "spec surface/capture",
        ),
        ("ghost_local", "pub fn bad(c:C)->u64 {let ghost snapshot=c;0}", "spec surface/capture"),
        (
            "predicate_capture",
            "pub fn bad(cell:C)->(r:u64) ensures {let x=cell;true}, {0}",
            "spec surface/capture",
        ),
        (
            "quantified_cell",
            "pub fn bad(v:u64)->(r:u64) ensures forall|c:C| true, {v}",
            "spec surface/capture",
        ),
        (
            "actual_recursion",
            "pub fn bad(c:&mut C,v:u64)->(r:u64) ensures r==v, {bad(c,v)}",
            "actual executable/spec local recursion",
        ),
        (
            "mutual_recursion",
            "fn a(v:u64)->u64 {b(v)} fn b(v:u64)->u64 {a(v)}",
            "actual executable/spec local recursion",
        ),
        ("shared_promotion", "pub fn bad(v:u64)->u64 {let s=&0u64;v}", "shared reference/escape"),
        (
            "nested_shared",
            "pub fn bad(v:u64)->u64 { #[verifier::verify] fn hidden(c:&C)->u64 {0} v }",
            "shared reference/escape",
        ),
        ("cast_in_body", "pub fn bad(v:u64)->u64 {v as u64}", "cast/raw conversion"),
        (
            "external_body",
            "#[verifier::external_body] fn hidden(c:&C) {} pub fn bad(v:u64)->u64 {v}",
            "external/abstract/skipped/trusted",
        ),
        ("static", "static S:u64=0; pub fn bad(v:u64)->u64 {v}", "local item Static"),
        ("adt", "struct Wrap {c:C} pub fn bad(v:u64)->u64 {v}", "local item Struct"),
        (
            "drop",
            "struct HasDrop; impl Drop for HasDrop {fn drop(&mut self){}}",
            "local item Struct",
        ),
        (
            "trait_abstract",
            "trait Hook {fn run(&self,c:&C);} pub fn bad(v:u64)->u64 {v}",
            "local item Trait",
        ),
        ("dyn", "pub fn bad(f:&dyn Fn(&C))->u64 {0}", "shared reference/escape"),
        ("panic_unwind", "pub fn bad(v:u64)->u64 {panic!()}", "closed scalar/tuple/native-cell"),
    ];
    for infer in [false, true] {
        for (label, s, reason) in cases {
            outcome(label, run(&source(s), infer, &[]), reason, false);
        }
    }
    finish();
}
#[test]
fn macros_fake_headers_and_unverified_items_receive_no_exemptions() {
    let cases = [
        (
            "macro_capture",
            format!(
                "macro_rules! capture {{ ($c:expr)=>{{move || $c.into_inner()}}; }} {}",
                source("pub fn bad(v:u64)->u64 {let c=C::new(v);let f=capture!(c);f()}")
            ),
            "closure/capture",
        ),
        (
            "macro_raw",
            format!(
                "macro_rules! raw {{ ($c:expr)=>{{$c as *mut C}}; }} {}",
                source("pub fn bad(c:&mut C)->u64 {let p=raw!(c);0}")
            ),
            "raw pointer/escape",
        ),
        (
            "macro_spec_capture",
            format!(
                "macro_rules! use_cell {{ ($c:expr)=>{{ {{let x=$c;true}} }}; }} {}",
                source("pub fn bad(cell:C)->(r:u64) ensures use_cell!(cell), {0}")
            ),
            "spec surface/capture",
        ),
        (
            "unverified",
            format!("fn hidden(v:u64)->u64 {{v}} {FROZEN}"),
            "native/unverified/skipped",
        ),
        (
            "external_parent",
            format!(
                "#[verifier::external] mod skipped {{ use core::cell::UnsafeCell; pub fn shared(c:&UnsafeCell<u64>)->u64 {{0}} }} {FROZEN}"
            ),
            "external/abstract/skipped/trusted",
        ),
        (
            "fake_constrain",
            raw_source(
                "#[verifier::spec] fn fake(a:u64,b:u64)->bool {true} #[verifier::verify] pub fn bad(v:u64)->u64 {verus_builtin::ensures(|r:u64| [fake(r,bad(v)),true]);v}",
            ),
            "noncanonical builtin constrain_type",
        ),
        (
            "wrong_self_argument",
            raw_source(
                "#[verifier::verify] pub fn bad(v:u64)->u64 {verus_builtin::ensures(|r:u64| [verus_builtin::constrain_type(r,bad(v+1)),true]);v}",
            ),
            "original parameter places",
        ),
        (
            "wrong_self_identity",
            raw_source(
                "#[verifier::verify] fn other(v:u64)->u64 {v} #[verifier::verify] pub fn bad(v:u64)->u64 {verus_builtin::ensures(|r:u64| [verus_builtin::constrain_type(r,other(v)),true]);v}",
            ),
            "self call identity",
        ),
    ];
    for infer in [false, true] {
        for (label, s, reason) in &cases {
            outcome(label, run(s, infer, &[]), reason, false);
        }
    }
    finish();
}
#[test]
fn frontend_skip_markers_on_items_and_ancestors_are_refused() {
    for marker in ["const_body", "reveal_fn"] {
        let attr = format!("#[verus::internal({marker})]");
        let cases = [
            (
                "native_body",
                raw_source(&format!(
                    "#[verifier::verify] {attr} pub fn skipped(v:u64)->u64 {{core::cell::UnsafeCell::new(v).into_inner()}}"
                )),
            ),
            (
                "false_contract",
                raw_source(&format!(
                    "#[verifier::verify] {attr} pub fn skipped(v:u64)->u64 {{verus_builtin::ensures(|r:u64| [verus_builtin::constrain_type(r,skipped(v)),false]);v}}"
                )),
            ),
            (
                "ancestor_module",
                raw_source(&format!(
                    "{attr} mod skipped {{mod inner {{#[verifier::verify] pub fn client(v:u64)->u64 {{core::cell::UnsafeCell::new(v).into_inner()}}}}}}"
                )),
            ),
            (
                "nested_function",
                raw_source(&format!(
                    "#[verifier::verify] pub fn outer(v:u64)->u64 {{#[verifier::verify] {attr} fn skipped(v:u64)->u64 {{v}} v}}"
                )),
            ),
        ];
        for infer in [false, true] {
            for (label, s) in &cases {
                outcome(
                    &format!("{marker}/{label}/infer={infer}"),
                    run(s, infer, &[]),
                    "frontend-skipped const_body/reveal_fn local item or ancestor is forbidden",
                    false,
                );
            }
        }
    }
    // A false predicate itself is valid admission input; ordinary checking is
    // still pending at this inactive checkpoint. Only its skip marker rejects.
    for infer in [false, true] {
        outcome(
            "unmarked_false_contract",
            run(
                &raw_source(
                    "#[verifier::verify] pub fn checked(v:u64)->u64 {verus_builtin::ensures(|r:u64| [verus_builtin::constrain_type(r,checked(v)),false]);v}",
                ),
                infer,
                &[],
            ),
            LATER,
            true,
        );
    }
    finish();
}
#[test]
fn frontend_directives_and_special_consts_are_refused() {
    for (marker, is_const) in [
        ("reveal_group", false),
        ("item_broadcast_use", true),
        ("size_of", true),
        ("structural_const_wrapper", true),
    ] {
        let attr = format!("#[verus::internal({marker})]");
        let item = if is_const {
            format!("#[verifier::verify] {attr} const K:()={{}};")
        } else {
            format!("#[verifier::verify] {attr} pub fn skipped() {{}}")
        };
        let cases = [
            ("item", raw_source(&item)),
            (
                "ancestor",
                raw_source(&format!(
                    "{attr} mod outer {{mod inner {{#[verifier::verify] pub fn client(v:u64)->u64 {{v}}}}}}"
                )),
            ),
            (
                "unused_nested_item",
                raw_source(&format!("#[verifier::verify] pub fn outer(v:u64)->u64 {{{item} v}}")),
            ),
            (
                "macro_item",
                raw_source(&format!("macro_rules! marked {{()=>{{{item}}};}} marked!();")),
            ),
        ];
        for infer in [false, true] {
            for (label, s) in &cases {
                outcome(
                    &format!("{marker}/{label}/infer={infer}"),
                    run(s, infer, &[]),
                    "frontend directive/special-const local item or ancestor is forbidden",
                    false,
                );
            }
        }
    }
    finish();
}
#[test]
fn crate_root_opt_in_does_not_replace_ordinary_body_coverage() {
    for root_attr in
        ["#![verifier::verify]", "#![verus::internal(verus_macro)]", "#![verifier::exec]"]
    {
        for infer in [false, true] {
            let plain = format!("{root_attr}\n{}", raw_source("pub fn unit() {}"));
            outcome(
                root_attr,
                run(&plain, infer, &[]),
                "native/unverified/skipped local body is forbidden",
                false,
            );
            for body in [
                "#[verifier::verify] pub fn unit() {}",
                "#[verifier::verify] mod checked {pub fn unit() {}}",
            ] {
                let marked = format!("{root_attr}\n{}", raw_source(body));
                outcome(root_attr, run(&marked, infer, &[]), LATER, true);
            }
            outcome(
                "actual default-verify option",
                run(&plain, infer, &["--no-external-by-default"]),
                LATER,
                true,
            );
        }
    }
    // Do not skip the crate owner entirely: all root exclusions still apply.
    for infer in [false, true] {
        let body = raw_source("#[verifier::verify] pub fn unit() {}");
        outcome(
            "root external exclusion",
            run(&format!("#![verifier::external]\n{body}"), infer, &[]),
            "external/abstract/skipped/trusted",
            false,
        );
        outcome(
            "root skipped-body exclusion",
            run(&format!("#![verus::internal(const_body)]\n{body}"), infer, &[]),
            "frontend-skipped const_body/reveal_fn",
            false,
        );
    }
    finish();
}
#[test]
fn imported_direct_abstract_dyn_and_projected_escapes_are_refused() {
    let c = InferCrates::new();
    let ext = c.directory.path().join("callbacks.rs");
    let lib = c.directory.path().join("libcallbacks.rlib");
    std::fs::write(&ext,"use core::cell::UnsafeCell; pub fn mutate(c:&UnsafeCell<u64>) {unsafe{*c.get()=7;}} pub trait Hook {fn run(&self,c:&UnsafeCell<u64>);} pub struct Witness; pub trait Associated {type Cell;} impl Associated for Witness {type Cell=UnsafeCell<u64>;} pub fn identity<T>(x:T)->T{x}").unwrap();
    let rustc = std::path::PathBuf::from(std::env::var("RUSTUP_HOME").unwrap())
        .join("toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc");
    let r = std::process::Command::new(rustc)
        .arg(&ext)
        .args(["--crate-type=lib", "--crate-name=callbacks", "--edition=2024"])
        .arg("-o")
        .arg(&lib)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let extern_arg = format!("callbacks={}", lib.display());
    let cases = [
        (
            "imported_direct",
            source(
                "pub fn bad(v:u64)->(r:u64) ensures r==v, {let c=C::new(v);callbacks::mutate(&c);c.into_inner()}",
            ),
            "imported/foreign/abstract callback",
        ),
        (
            "imported_generic",
            source("pub fn bad(v:u64)->u64 {callbacks::identity(C::new(v)).into_inner()}"),
            "imported/foreign/abstract callback",
        ),
        (
            "imported_abstract",
            source("pub fn bad<H:callbacks::Hook>(h:&H,c:&C)->u64 {h.run(c);0}"),
            "type/const generics",
        ),
        (
            "imported_dyn",
            source("pub fn bad(h:&dyn callbacks::Hook,c:&C)->u64 {h.run(c);0}"),
            "shared reference/escape",
        ),
        (
            "imported_projection",
            format!(
                "type Hidden=<callbacks::Witness as callbacks::Associated>::Cell; {}",
                source("pub fn bad(c:Hidden)->u64 {c.into_inner()}")
            ),
            "source associated projection",
        ),
    ];
    for infer in [false, true] {
        for (label, s, reason) in &cases {
            outcome(label, run(s, infer, &["--extern", &extern_arg]), reason, false);
        }
    }
    finish();
}
