use super::*;

const BUFFER: &str = include_str!("../../../tools/native_byte_buffer/src/lib.rs");
const HINTS: &str = include_str!("byte_spare_hints.txt");

#[test]
fn checked_runtime_lengths_reject_forged_or_ignored_native_methods() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let p = crates.build_source(
            "hint_provider",
            HINTS,
            &[],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
        let c = r#"use vstd::prelude::*;verus!{fn count(n:usize)->(r:usize) ensures r==n,{hint_provider::length(n)}}"#;
        let r = crates.build_source(
            "hint_client",
            c,
            &["hint_provider"],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    }
    let cases = [
        (HINTS.replace("(self.left,Some(self.left))", "(0,Some(0))"), "postcondition"),
        (
            HINTS.replace(
                "impl ExactSizeIterator for Counter {}",
                "impl ExactSizeIterator for Counter {fn len(&self)->usize{0}}",
            ),
            "postcondition",
        ),
        (
            HINTS.replace(
                "#[verus_verify] impl ExactSizeIterator for Counter {}",
                "impl ExactSizeIterator for Counter {fn len(&self)->usize{0}}",
            ),
            "actual selected native override",
        ),
        (
            HINTS
                .replace(
                    "#[verus_verify] impl Iterator for Counter {",
                    "impl Iterator for Counter {",
                )
                .replace(" fn next", " #[verus_verify] fn next"),
            "entire impl must be verified",
        ),
        (
            HINTS.replace(
                "has_exact_size_hint(&self)->bool{true}",
                "has_exact_size_hint(&self)->bool{false}",
            ),
            "postcondition",
        ),
        (
            format!(
                "{}\nverus!{{fn no_vacuity(n:usize){{let c=Counter{{left:n}};let _=c.len();assert(false);}}}}",
                HINTS
            ),
            "assertion failed",
        ),
        (HINTS.to_string() + "\nimpl Drop for Counter {fn drop(&mut self){}}\n", "custom Drop"),
        (
            HINTS.replace("fn next(&mut self)", "fn count(self)->usize{0} fn next(&mut self)"),
            "custom forward adapter override",
        ),
    ];
    for (src, diagnostic) in cases {
        let r = crates.build_source(
            "hint_provider",
            &src,
            &[],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(!r.status.success());
        assert!(!crates.vir("hint_provider").exists());
        assert!(
            String::from_utf8_lossy(&r.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
    let bad = HINTS.replace("(self.left,Some(self.left))", "(0,Some(0))");
    let r = crates.build_source("hint_provider", &bad, &[], &["--no-cheating"], true);
    assert!(!r.status.success());
    assert!(!crates.vir("hint_provider").exists());
    // A wrong auxiliary logical observation cannot dictate the runtime count.
    let wrong_observation =
        HINTS.replace("exact_len(&self)->usize{self.left}", "exact_len(&self)->usize{0}");
    let r = crates.build_source(
        "hint_provider",
        &wrong_observation,
        &[],
        &["--infer-goals", "--no-cheating"],
        true,
    );
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
}

#[test]
fn original_byte_filler_composes_and_publication_remains_fail_closed() {
    let crates = InferCrates::new();
    let consumer = r#"use vstd::prelude::*;verus!{broadcast use vstd::group_vstd_default;
      fn bytes(a:u8,b:u8)->(r:Result<Vec<u8>,u8>)
        ensures r.is_ok(),r->Ok_0@.len()==2,r->Ok_0@[0]==a,r->Ok_0@[1]==b,
      {let mut v=Vec::new();v.push(Ok(a));v.push(Ok(b));byte_provider::fill(v)}
      fn first_error(code:u8)->(r:Result<Vec<u8>,u8>) ensures r==Err(code),
      {let mut v=Vec::new();v.push(Ok(17));v.push(Err(code));v.push(Err(99));
       let ghost input=v@;let r=byte_provider::fill(v);
       proof {assert(input[0]==Ok(17));assert(input[1]==Err(code));assert(input[2]==Err(99));
       if r.is_ok(){assert(input[1]==Ok(r->Ok_0@[1]));}
       else {let ghost i=choose|i:int| 0<=i<input.len() && input[i]==Err(r->Err_0) && (forall|j:int| 0<=j<i ==> input[j].is_ok());
         if i>1 {assert(input[1].is_ok());}assert(i==1);}}
       r}
    }"#;
    for _ in 0..2 {
        let p = crates.build_source(
            "byte_provider",
            BUFFER,
            &[],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
        let c = crates.build_source(
            "byte_client",
            consumer,
            &["byte_provider"],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
    }
    // Exercise the erased artifact, not just its logical interface.
    let smoke = crates.directory.path().join("erased_smoke.rs");
    std::fs::write(&smoke,r#"fn main(){assert_eq!(byte_provider::fill(vec![]),Ok(vec![]));assert_eq!(byte_provider::fill(vec![Ok(0),Ok(255)]),Ok(vec![0,255]));assert_eq!(byte_provider::fill(vec![Ok(17),Err(42),Err(9)]),Err(42));}"#).unwrap();
    let binary = crates.directory.path().join("erased_smoke");
    let run = std::process::Command::new("rustc")
        .args(["+1.99.1", "--edition=2021"])
        .arg(&smoke)
        .arg("--extern")
        .arg(format!(
            "byte_provider={}",
            crates.directory.path().join("libbyte_provider.rlib").display()
        ))
        .arg("-L")
        .arg(format!("dependency={}", crates.profile.display()))
        .arg("-L")
        .arg(format!("dependency={}", crates.directory.path().display()))
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(std::process::Command::new(&binary).status().unwrap().success());
    let unchecked = crates.build_source(
        "byte_client",
        consumer,
        &["byte_provider"],
        &["--infer-goals", "--no-lifetime"],
        true,
    );
    assert!(!unchecked.status.success());
    assert!(!crates.vir("byte_client").exists());
    assert!(
        String::from_utf8_lossy(&unchecked.stderr)
            .contains("requires normal tracked lifetime checks")
    );
    for flags in [
        vec!["--infer-goals", "--no-verify"],
        vec!["--infer-goals", "--verify-root", "--verify-function", "fill"],
        vec!["--infer-goals", "--no-lifetime"],
    ] {
        let r = crates.build_source("byte_provider", BUFFER, &[], &flags, true);
        assert!(!r.status.success());
        assert!(!crates.vir("byte_provider").exists());
        let text = String::from_utf8_lossy(&r.stderr);
        assert!(
            text.contains("requires full-crate verification")
                || text.contains("requires normal tracked lifetime checks"),
            "{}",
            text
        );
    }
    let uninitialized = "use vstd::prelude::*;use vstd::byte_vec::*;verus!{pub fn bad(){let mut v=allocate(1);let(_,Tracked(s))=acquire(&mut v);let tracked r=seal(s);finish(&mut v,1,Tracked(r));}}";
    for flags in [vec!["--no-cheating"], vec!["--no-verify"]] {
        let r = crates.build_source("byte_provider", uninitialized, &[], &flags, true);
        assert!(!r.status.success());
        assert!(!crates.vir("byte_provider").exists());
    }
    let cases = [
        (BUFFER.replace("ptr = ptr.add(1)", "ptr = ptr.add(2)"), "invariant"),
        (
            BUFFER.replace("output.set_len(len)", "output.set_len(len.saturating_sub(1))"),
            "postcondition",
        ),
        (
            BUFFER.replace(
                "let mut ptr = output.as_mut_ptr();",
                "let mut ptr = output.as_mut_ptr(); let alias = ptr;",
            ),
            "unsupported byte-buffer profile",
        ),
        (
            BUFFER.replace("core::ptr::write(ptr, byte?)", "fake_write(ptr, byte?)")
                + "\nunsafe fn fake_write<T>(_: *mut T,_:T){}\n",
            "unsupported byte-buffer primitive identity",
        ),
    ];
    for (src, expected) in cases {
        let r = crates.build_source(
            "byte_provider",
            &src,
            &[],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(!r.status.success());
        assert!(!crates.vir("byte_provider").exists());
        assert!(
            String::from_utf8_lossy(&r.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
}

#[test]
fn byte_spare_permissions_enforce_initialization_exclusivity_and_generations() {
    let cases = [
        (
            "one_past",
            r#"fn test<'a>(p:*mut u8,Tracked(s):Tracked<ByteSpare<'a>>)
          requires s.wf(),s.contents().len()==s.capacity(),s.pointer(p,s.capacity() as int),
          {let tracked mut s=s;write(p,7,Tracked(&mut s));}"#,
            "precondition",
        ),
        (
            "uninitialized",
            "fn test(){let mut v=allocate(1);let(_,Tracked(s))=acquire(&mut v);let tracked r=seal(s);finish(&mut v,1,Tracked(r));}",
            "precondition",
        ),
        (
            "invalid_length",
            "fn test(){let mut v=allocate(1);let(p,Tracked(mut s))=acquire(&mut v);write(p,7,Tracked(&mut s));let tracked r=seal(s);finish(&mut v,2,Tracked(r));}",
            "precondition",
        ),
        (
            "duplicate",
            "fn test(){let mut v=allocate(1);let(p,Tracked(mut s))=acquire(&mut v);let tracked duplicate=s;write(p,7,Tracked(&mut s));}",
            "moved value",
        ),
        (
            "destroyed",
            "fn test(){let(p,Tracked(mut s))={let mut v=allocate(1);acquire(&mut v)};write(p,7,Tracked(&mut s));}",
            "does not live long enough",
        ),
        (
            "reallocated",
            "fn test(){let mut v=allocate(1);let(p,Tracked(mut s))=acquire(&mut v);v.reserve(2);write(p,7,Tracked(&mut s));}",
            "more than once",
        ),
        (
            "transferred",
            "fn test(){let mut v=allocate(1);let(p,Tracked(mut s))=acquire(&mut v);write(p,7,Tracked(&mut s));let tracked r=seal(s);finish(&mut v,1,Tracked(r));write(p,8,Tracked(&mut s));}",
            "moved value",
        ),
        (
            "stale",
            "fn test(){let mut v=allocate(1);let(p,Tracked(mut s))=acquire(&mut v);write(p,7,Tracked(&mut s));let tracked old=seal(s);let(q,Tracked(mut s))=acquire(&mut v);write(q,8,Tracked(&mut s));let tracked current=seal(s);finish(&mut v,1,Tracked(old));}",
            "precondition",
        ),
        (
            "foreign",
            "fn test(){let mut a=allocate(1);let mut b=allocate(1);let(p,Tracked(mut s))=acquire(&mut a);let(q,Tracked(mut t))=acquire(&mut b);write(p,7,Tracked(&mut t));}",
            "precondition",
        ),
        (
            "sealed_reallocation",
            "fn test(){let mut v=allocate(1);let(p,Tracked(mut s))=acquire(&mut v);write(p,7,Tracked(&mut s));let tracked r=seal(s);v.reserve(1000);finish(&mut v,1,Tracked(r));}",
            "precondition",
        ),
    ];
    for (name, body, expected) in cases {
        let src = format!("use vstd::prelude::*;use vstd::byte_vec::*;verus!{{{body}}}");
        let e =
            verify_one_file(name, src, &["vstd", "--infer-goals", "--no-cheating"]).unwrap_err();
        assert!(e.errors.iter().any(|d| d.message.contains(expected)), "{}: {:?}", name, e);
    }
    let good = r#"use vstd::prelude::*;use vstd::byte_vec::*;verus!{
      fn single(x:u8)->(r:Vec<u8>) ensures r@==seq![x],{
        let mut v=allocate(1);let(p,Tracked(mut s))=acquire(&mut v);write(p,x,Tracked(&mut s));
        let _one_past=advance(p,1,Tracked(&s));let tracked receipt=seal(s);finish(&mut v,1,Tracked(receipt));v
      }
      fn empty()->(r:Vec<u8>) ensures r@.len()==0,{
        let mut v=allocate(0);let(p,Tracked(s))=acquire(&mut v);let _=advance(p,0,Tracked(&s));
        let tracked receipt=seal(s);finish(&mut v,0,Tracked(receipt));v
      }
    }"#;
    verify_one_file("byte_spare_valid", good.into(), &["vstd", "--infer-goals", "--no-cheating"])
        .unwrap();
}
