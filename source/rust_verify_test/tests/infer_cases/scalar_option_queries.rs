use super::*;

// An unrelated sensor protocol: the ordinary provider checks a tiny local
// static, while the consumer infers two eager ok_or conversions and ? ordering.
const PROVIDER: &str = r#"use vstd::prelude::*;
verus! {pub open spec fn observed(b:u8)->Option<u8> {if b==1 {Some(7)} else if b==2 {Some(9)} else {None}}}
#[verus_verify]
#[verus_spec(r => ensures r==observed(b))]
pub fn read(b:u8)->Option<u8> {
 #[verus_verify] #[verus_spec(ensures LEVELS@==seq![255u8,7u8,9u8])]
 static LEVELS:[u8;3] = [255,7,9];
 if b<3 {let n=LEVELS[b as usize]; if n==255 {None} else {Some(n)}} else {None}
}
"#;
const CONSUMER: &str = r#"use vstd::prelude::*;use sensor_provider::*;
verus! {
#[verifier::infer] pub fn readings(a:u8,b:u8)->Result<u8,(u8,bool)>{
 let x=read(a).ok_or((a,true))?;
 let y=read(b).ok_or((b,false))?;
 Ok((x << 4) | y)
}
pub fn policy(a:u8,b:u8)->(r:Result<u8,(u8,bool)>)
 ensures r==match observed(a){None=>Err((a,true)),Some(x)=>match observed(b){None=>Err((b,false)),Some(y)=>Ok((16*x+y) as u8)}},
{readings(a,b)}
}
"#;

#[test]
fn scalar_option_queries_and_local_static_compose_across_crates() {
    let crates = InferCrates::new();
    for attempt in 0..2 {
        let r = crates.build_source(
            "sensor_provider",
            PROVIDER,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
            true,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        if attempt == 0 {
            let r = crates.build_source(
                "sensor_consumer",
                CONSUMER,
                &["sensor_provider"],
                &["--no-cheating"],
                true,
            );
            assert!(!r.status.success());
            assert!(!crates.vir("sensor_consumer").exists());
            assert!(
                String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"),
                "{}",
                String::from_utf8_lossy(&r.stderr)
            );
        }
        let r = crates.build_source(
            "sensor_consumer",
            CONSUMER,
            &["sensor_provider"],
            &["--infer-goals", "--no-cheating", "--output-json"],
            true,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        assert_eq!(report["functions"].as_array().unwrap().len(), 1);
        for p in ["soundness", "maximality", "exactness"] {
            assert_eq!(report["functions"][0]["proofs"][p]["status"], "proved", "{report}");
        }
    }
    let wrong = PROVIDER.replace("= [255,7,9]", "= [255,8,9]");
    let r = crates.build_source(
        "sensor_provider",
        &wrong,
        &[],
        &["--infer-goals", "--no-cheating"],
        true,
    );
    assert!(!r.status.success());
    assert!(!crates.vir("sensor_provider").exists());
    assert!(
        String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"),
        "{}",
        String::from_utf8_lossy(&r.stderr)
    );
    for flags in [
        vec!["--infer-goals", "--no-verify"],
        vec!["--infer-goals", "--verify-root", "--verify-function", "read"],
    ] {
        let r = crates.build_source("sensor_provider", PROVIDER, &[], &flags, true);
        assert!(!r.status.success());
        assert!(!crates.vir("sensor_provider").exists());
    }
}

#[test]
fn packing_assistance_preserves_overlap_and_truncation_domains() {
    let source = r#"use vstd::prelude::*;
#[verus_infer] fn pack(h:u16,l:u16)->u16{(h<<5)|l}
verus!{fn policy(h:u16,l:u16)->(r:u16)
 requires h<2048,l<32, ensures r as int == 32*h+l,
 {pack(h,l)}}
"#;
    verify_one_file("word_pack", source.into(), &["vstd", "--infer-goals", "--no-cheating"])
        .unwrap();
    for (label, bad) in [
        ("pack_overlap", source.replace("l<32", "l<64")),
        ("pack_truncation", source.replace("h<2048", "h<4096")),
    ] {
        let e =
            verify_one_file(label, bad, &["vstd", "--infer-goals", "--no-cheating"]).unwrap_err();
        assert!(
            e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")),
            "{:?}",
            e
        );
    }
}

#[test]
fn ok_or_error_argument_is_eager_and_custom_names_supply_no_model() {
    let source = r#"use vstd::prelude::*;
#[verus_infer] fn eager(x:Option<u8>,n:u8)->Result<u8,u8>{x.ok_or(n+1)}
verus!{fn wrong(){let _=eager(Some(1),255);}}
"#;
    let e = verify_one_file(
        "eager_option_error",
        source.into(),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
    let source = r#"use vstd::prelude::*;
#[verus_verify] struct Fake;
#[verus_verify] impl Fake {fn ok_or(self,e:u8)->Result<u8,u8>{Err(e)}}
#[verus_infer] fn reject(x:Fake)->Result<u8,u8>{x.ok_or(1)}
"#;
    assert!(verify_one_file("fake_ok_or", source.into(), &["vstd", "--no-cheating"]).is_err());
}
