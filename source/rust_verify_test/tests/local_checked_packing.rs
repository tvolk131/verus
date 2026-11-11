#![feature(rustc_private)]
mod common;
use common::*;

// Real boolean flag conversion participates in each writer's returned word.
const WRITER: &str = r#"
use vstd::prelude::*;
verus! {
const CHANNEL_MASK:u64=7u64 << 12u64;
fn packet(payload:u64,channel:u64,urgent:bool)->(word:u64)
    requires payload < 4096, channel < 8,
    ensures word & 4095u64 == payload,
        (word & CHANNEL_MASK) >> 12u64 == channel,
        (word & 32768u64 > 0) == urgent,
{
    payload | (channel << 12u64) | (u64::from(urgent) << 15u64)
}
}
"#;
#[test]
fn independent_packet_writer_and_false_controls() {
    let check = |name: &str, s: String| {
        verify_one_file(name, s, &["vstd", "--infer-goals", "--no-cheating", "--compile"])
    };
    assert!(check("packet", WRITER.into()).is_ok());
    for (name, s) in [
        ("range", WRITER.replace("payload < 4096", "payload < 8192")),
        ("channel", WRITER.replace("channel < 8", "channel < 16")),
        ("shift", WRITER.replace("channel << 12u64", "channel << 11u64")),
        ("flag", WRITER.replace("== urgent", "== !urgent")),
        (
            "xor",
            WRITER.replace("payload < 4096", "payload < 8192").replace("payload |", "payload ^"),
        ),
    ] {
        let e = check(name, s).unwrap_err();
        assert!(
            e.errors.iter().any(|e| e.message.contains("postcondition not satisfied")),
            "{}: {:?}",
            name,
            e
        );
    }
}
#[test]
fn finite_width_readers_and_zero_shift() {
    for w in [8, 16, 32, 64] {
        let source = format!(
            r#"
use vstd::prelude::*;
verus! {{
fn write(x:u{w},b:bool)->(r:(u{w},u64))
    ensures r.0==x, r.1==if b {{1u64}} else {{0u64}},
{{
    let flag=u64::from(b);
    (if flag>0 {{ x << 0u64 }} else {{ x >> 0u64 }}, flag)
}}
fn field(x:u{w})->(r:u{w}) ensures r<=3 {{ (x & 12u{w}) >> 2u64 }}
}}
"#
        );
        let r = verify_one_file(
            "width",
            source,
            &["vstd", "--infer-goals", "--no-cheating", "--compile"],
        );
        assert!(r.is_ok(), "width{}: {:?}", w, r);
        let source = format!(
            r#"
use vstd::prelude::*;
verus! {{
const KIND_MASK:u{w}=3u{w} << 4u64;
fn flags(low:u{w},kind:u{w},active:bool)->(r:u{w})
    requires low < 16, kind < 4,
    ensures r & 15u{w} == low, (r & KIND_MASK) >> 4u64 == kind,
        (r & 64u{w} > 0) == active,
{{ low | (kind << 4u64) | ((u64::from(active) as u{w}) << 6u64) }}
}}
"#
        );
        let r = verify_one_file(
            "finite_flags",
            source.clone(),
            &["vstd", "--infer-goals", "--no-cheating", "--compile"],
        );
        assert!(r.is_ok(), "flags width{}: {:?}", w, r);
        for (name, bad) in [
            ("finite_range", source.replace("low < 16", "low < 32")),
            ("finite_shift", source.replace("kind << 4u64", &format!("kind << {w}u64"))),
        ] {
            let e = verify_one_file(
                name,
                bad,
                &["vstd", "--infer-goals", "--no-cheating", "--compile"],
            )
            .unwrap_err();
            assert!(!e.errors.is_empty(), "width{}: {:?}", w, e);
        }
    }
}
#[test]
fn boundary_shift_still_rejected() {
    let s = WRITER.replace("channel << 12u64", "channel << 64u64");
    let e =
        verify_one_file("boundary", s, &["vstd", "--infer-goals", "--no-cheating", "--compile"])
            .unwrap_err();
    assert!(
        e.errors.iter().any(|e| e.message.contains("overflow") || e.message.contains("shift")),
        "{:?}",
        e
    );
}

#[test]
fn signed_values_do_not_gain_unsigned_range_assumptions() {
    let source = r#"
use vstd::prelude::*;
verus! {
fn flags(x:i64,b:bool)->(r:i64)
    requires x < 16,
    ensures r & 15i64 == x,
{ (x & 15i64) | ((u64::from(b) as i64) << 4u64) }
}
"#;
    let e = verify_one_file(
        "signed",
        source.into(),
        &["vstd", "--infer-goals", "--no-cheating", "--compile"],
    )
    .unwrap_err();
    assert!(e.errors.iter().any(|e| e.message.contains("postcondition not satisfied")), "{:?}", e);
}
