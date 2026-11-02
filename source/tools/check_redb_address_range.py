"""Prove the byte-preserved pinned redb address-range helpers and ordinary callers."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile

from check_redb_page_size import APPLICATION, BASE, SOURCE_HASH, classify, item
from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import LoggedCommands, positive_timeout, require, require_exact, require_publication, validate_run


ITEM_HASHES = {
    "PageNumber": "2f35314985c2cba913372168d6fc73f5633b0de1bfc720f73fc5418460b9d867",
    "page_size_bytes": "2a0e31afa33b709b29c216b83b73ad5f69eb5501b8a3330a819996581573b5b8",
    "address_range": "22beb37e2f40bc1935fd83e046dcf7c631a28eab0ed86f041b5c940b4a9a9829",
}
HEADER = "use vstd::prelude::*;\nuse std::ops::Range;\nverus! {\n"
CALLERS = """
fn full_domain(region:u32,index:u32,order:u8,size:u32,offset:u64,region_size:u64,pages_start:u64)
    ->(result:Range<u64>)
    requires order<64, bytes(order,size)<=u64::MAX,
        index as int * bytes(order,size)<=u64::MAX,
        regional(index,order,size,pages_start)<region_size,
        region as int * region_size as int<=u64::MAX,
        offset as int + region as int * region_size as int
            + regional(index,order,size,pages_start) + bytes(order,size)<=u64::MAX,
    ensures result.end as int-result.start as int==bytes(order,size),
        result.start==offset as int+region as int*region_size as int+regional(index,order,size,pages_start),
{
    let page=PageNumber{region,page_index:index,page_order:order};
    let byte_count=page.page_size_bytes(size);
    let result=page.address_range(offset,region_size,pages_start,size);
    assert(result.end as int-result.start as int==byte_count);
    result
}
fn contained(region:u32,index:u32,order:u8,size:u32,offset:u64,region_size:u64,pages_start:u64)
    ->(result:Range<u64>)
    requires order<64, bytes(order,size)<=u64::MAX,
        index as int * bytes(order,size)<=u64::MAX,
        regional(index,order,size,pages_start)<region_size,
        regional(index,order,size,pages_start)+bytes(order,size)<=region_size,
        offset as int + region as int*region_size as int+region_size as int<=u64::MAX,
    ensures result.start>=offset as int+region as int*region_size as int,
        result.end<=offset as int+region as int*region_size as int+region_size as int,
        result.end as int-result.start as int==bytes(order,size),
{full_domain(region,index,order,size,offset,region_size,pages_start)}
fn counterexample()->(result:Range<u64>)
    ensures result.start==4090,result.end==8186,result.end>4096,
{PageNumber{region:0,page_index:0,page_order:0}.address_range(0,4096,4090,4096)}
fn largest()->(result:Range<u64>)
    ensures result.start==0,result.end==9223372036854775808,
{PageNumber{region:0,page_index:0,page_order:63}.address_range(0,u64::MAX,0,1)}
fn zero_length()->(result:Range<u64>) ensures result.start==u64::MAX,result.end==u64::MAX,
{PageNumber{region:0,page_index:0,page_order:63}.address_range(u64::MAX,1,0,0)}
fn exact_boundary()->(result:Range<u64>) ensures result.start==4096,result.end==8192,
{PageNumber{region:1,page_index:0,page_order:0}.address_range(0,4096,0,4096)}
"""
CALLERS = CALLERS.replace("regional(index,order,size,pages_start)",
                          "(pages_start as int + index as int * bytes(order,size))")
CALLERS = CALLERS.replace("bytes(order,size)", "((1u64 << order) as int * size as int)")
GENERIC, BOUNDARIES = CALLERS.split("fn counterexample()", 1)
BOUNDARIES = "fn counterexample()" + BOUNDARIES
SAFETY = {
    "shift64_zero": (0, 0, 64, 0, 1, 0, 0),
    "shift255_zero": (0, 0, 255, 0, 1, 0, 0),
    "page_multiply_overflow": (0, 0, 63, 0, 1, 0, 2),
    "index_multiply_overflow": (0, 2, 63, 0, (1 << 64)-1, 0, 1),
    "regional_add_overflow": (0, 1, 0, 0, (1 << 64)-1, (1 << 64)-1, 1),
    "region_multiply_overflow": (2, 0, 0, 0, (1 << 64)-1, 0, 0),
    "base_add_overflow": (1, 0, 0, (1 << 64)-1, 1, 0, 0),
    "start_add_overflow": (0, 0, 0, (1 << 64)-1, 2, 1, 0),
    "end_add_overflow": (0, 0, 0, (1 << 64)-1, 1, 0, 1),
    "guard_violation": (0, 0, 0, 0, 4096, 4096, 0),
    "empty_region": (0, 0, 63, 0, 0, 0, 0),
}


def witness(values):
    region, index, order, offset, region_size, pages_start, size = values
    return (f"PageNumber{{region:{region},page_index:{index},page_order:{order}}}"
            f".address_range({offset},{region_size},{pages_start},{size})")


def extract(application, output):
    path = application / BASE
    require(digest(path) == SOURCE_HASH, "pinned application source changed")
    text = path.read_text()
    pieces = {
        "PageNumber": item(text, "pub(crate) struct PageNumber"),
        "page_size_bytes": item(text, "    pub(crate) fn page_size_bytes("),
        "address_range": item(text, "    pub(crate) fn address_range("),
    }
    provenance = {}
    for name, contents in pieces.items():
        target = output / (name + ".original.rs")
        target.write_text(contents + "\n")
        if name in ITEM_HASHES:
            require(digest(target) == ITEM_HASHES[name], "pinned extraction changed: " + name)
        provenance[name] = dict(sha256=digest(target), source=BASE,
                                line=text[:text.index(contents)].count("\n")+1)
    executable = pieces["PageNumber"] + "\nimpl PageNumber {\n" + "\n".join(
        pieces[name] for name in ("page_size_bytes", "address_range")) + "\n}\n"
    inferred = executable
    for name in ("page_size_bytes", "address_range"):
        inferred = inferred.replace(pieces[name], "#[verifier::infer]\n"+pieces[name])
    sources = {"original": HEADER+inferred+GENERIC+"}\n",
               "boundaries": HEADER+inferred+BOUNDARIES+"}\n",
               "provider": HEADER+inferred.replace("pub(crate)", "pub")+"}\n"}
    guard_condition = "        (pages_start as int + index as int * ((1u64 << order) as int * size as int))<region_size,\n"
    require(guard_condition in GENERIC, "missing explicit debug guard premise")
    sources["release_domain"] = HEADER+inferred+GENERIC.replace(guard_condition, "", 1)+"}\n"
    relay = """
#[verifier::infer]
pub fn relay(page:&provider::PageNumber,offset:u64,region_size:u64,pages_start:u64,size:u32)->Range<u64> {
    page.address_range(offset,region_size,pages_start,size)
}
"""
    sources["client"] = HEADER + relay + GENERIC.replace("PageNumber{", "provider::PageNumber{").replace(
        "page.address_range(offset,region_size,pages_start,size)",
        "relay(&page,offset,region_size,pages_start,size)") + "}\n"
    for name, values in SAFETY.items():
        sources[name] = HEADER+inferred+"fn rejected()->Range<u64>{"+witness(values)+"}\n}\n"
    domain = GENERIC.split("fn contained(", 1)[0]
    wrong = domain.replace("fn full_domain(", "fn wrong(", 1)
    length = "result.end as int-result.start as int==((1u64 << order) as int * size as int)"
    sources["false_containment"] = HEADER+inferred+wrong.replace(
        length, "result.end<=offset as int+region as int*region_size as int+region_size as int", 1)+"}\n"
    sources["false_length"] = HEADER+inferred+wrong.replace(
        length, "result.end as int-result.start as int!=((1u64 << order) as int * size as int)", 1)+"}\n"
    sources["false_containment_auxiliary"] = HEADER+inferred+BOUNDARIES.replace(
        "result.end==8186", "result.end<=4096", 1)+"}\n"
    for enabled in (True, False):
        sources["cfg_"+str(enabled).lower()] = HEADER+inferred+(
            "fn actual_cfg(){assert("+("" if enabled else "!")+"cfg!(debug_assertions));}\n}\n")
    for name, source in sources.items():
        (output / (name+".rs")).write_text(source)
    (output / "native.rs").write_text("use std::ops::Range;\n"+executable+NATIVE)
    return provenance


NATIVE = """
fn main() {
    std::panic::set_hook(Box::new(|_information|{}));
    let page=PageNumber{region:0,page_index:0,page_order:0};
    assert_eq!(page.address_range(0,4096,4090,4096),4090..8186);
    let guard=std::panic::catch_unwind(||page.address_range(0,4096,4096,0));
    assert_eq!(guard.is_err(),cfg!(debug_assertions));
    let mut cases=0u64;
    for region in 0u32..=3 {
        for index in 0u32..=15 {
            for order in 0u8..=63 {
                for size in [0u32,1,2,4096,u32::MAX] {
                    let byte_count=(1u128<<order)*u128::from(size);
                    let regional_start=7+u128::from(index)*byte_count;
                    let region_size=regional_start+byte_count+1;
                    let start=11+u128::from(region)*region_size+regional_start;
                    let end=start+byte_count;
                    if end>u128::from(u64::MAX) || region_size>u128::from(u64::MAX) {continue;}
                    let page=PageNumber{region,page_index:index,page_order:order};
                    let actual=page.address_range(11,region_size as u64,7,size);
                    assert_eq!(u128::from(actual.start),start);
                    assert_eq!(u128::from(actual.end),end);
                    assert_eq!(u128::from(actual.end-actual.start),byte_count);
                    cases+=1;
                }
            }
        }
    }
    for order in [64u8,255] {
        let page=PageNumber{region:0,page_index:0,page_order:order};
        let outcome=std::panic::catch_unwind(||page.address_range(0,1,0,std::hint::black_box(0)));
        assert_eq!(outcome.is_err(),cfg!(debug_assertions));
    }
    println!("{{\\\"cases\\\":{},\\\"debug_assertions\\\":{},\\\"formal_proof\\\":false}}",cases,cfg!(debug_assertions));
}
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("application", "verus", "solver", "output"):
        parser.add_argument("--"+option, type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="redb-address-range-", dir=args.output.resolve()))
    engine = Path(__file__).resolve().parents[2]
    environment = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(args.solver.resolve()))
    transfer = dict(scenario="redb_address_range", application_commit=APPLICATION, status="blocked",
        source_fidelity=False, positive_cases=[], semantic_negative_cases=[], safety_negative_cases=[],
        auxiliary_negative_cases=[], metadata_negative_cases=[], stale_export_rejected=False,
        limitations=["Only extracted original helpers and explicit ordinary callers; no whole-database theorem",
                     "Native debug/release oracle is finite supporting evidence, not a formal Rust library proof",
                     "Warm record/owned candidates regenerate; poisoned caches are not claimed as cache hits",
                     "Generic arithmetic callers need no assistance; constant boundary callers use --infer-goals"],
        trust=["Ordinary Verus backend, SMT solver, and existing vstd conversion and requires-false panic specifications",
               "Only canonical public transparent core Range<u64> with the existing ExRange proxy is admitted",
               "No new axioms, trusted application summaries, suppressed assertions, or unproved ghost hints"])
    receipt = dict(status="blocked", output=str(output), application_commit=APPLICATION, transfer=transfer,
                   host=host_info(), commands=[], cases=[], verification_accepted=False, success=False)
    run = LoggedCommands(output, engine, receipt["commands"], timeout=args.timeout)
    try:
        require(run("application_pin", ["git", "-C", str(args.application), "rev-parse", "HEAD"], environment)==0)
        require((output/"application_pin.stdout").read_text().strip()==APPLICATION, "application pin mismatch")
        receipt["extraction"] = extract(args.application, output)
        transfer["source_fidelity"] = True
        guard = ArtifactGuard(tool_artifacts(args.verus.resolve(), args.solver.resolve()))
        run.guard = guard

        def verify(name, phase, success, classification="proved", debug="yes", dependency=None, seed=None, assistance=False):
            label = name+"_"+phase
            artifact = output/(label+".vir")
            if seed is not None:
                artifact.write_bytes(seed)
            environment["VERUS_INFER_CACHE"] = str(output/(name+"-cache"))
            command = [str(args.verus.resolve()), str(output/(name+".rs")), "--crate-type", "lib", "--crate-name", name,
                       "--no-cheating", "--output-json", "--compile", "--export", str(artifact),
                       "-o", str(output/("lib"+name+".rlib")), "-C", "debug-assertions="+debug]
            if dependency:
                command += ["--extern", "provider="+str(output/"libprovider.rlib"), "--import", "provider="+str(dependency)]
            if assistance:
                command += ["--infer-goals"]
            status = run(label, command, environment)
            data = json.loads((output/(label+".stdout")).read_text())
            stderr = (output/(label+".stderr")).read_text()
            observed = classify(data, stderr, status)
            validate_run(dict(returncode=status,stdout=json.dumps(data),stderr=stderr), success,
                         allow_frontend=classification=="frontend_rejection")
            functions = data.get("verus-infer", {}).get("functions", [])
            receipt["cases"].append(dict(label=label, classification=observed, report=data,
                artifact=str(artifact), artifact_exists=artifact.exists(), stale_export_seeded=seed is not None))
            require((status==0)==success, label+": unexpected exit; "+stderr[-3000:])
            require(observed==classification, label+": unexpected classification: "+observed)
            count = 0 if classification=="frontend_rejection" else (1 if name=="client" else 2)
            require(len(functions)==count, label+": inference count changed")
            require_exact(functions)
            require_publication(data, success, artifact)
            if classification!="frontend_rejection":
                require(data["verification-results"].get("is-verifying-entire-crate") is True,
                        label+": partial checking is not transfer evidence")
            category = ("positive_cases" if success else
                        "safety_negative_cases" if classification=="caller_obligation_failure" else
                        "semantic_negative_cases" if classification=="semantic_goal_failure" else
                        "metadata_negative_cases" if classification=="frontend_rejection" else
                        "auxiliary_negative_cases")
            transfer[category].append(label)
            return artifact

        originals = [verify("original", phase, True) for phase in ("cold", "warm")]
        cache = output/"original-cache"
        cache.mkdir(exist_ok=True)
        (cache/"poisoned.json").write_text('{"version":"verus-infer-v68","pre":true,"post":false}')
        verify("original", "poisoned_cache", True)
        verify("boundaries", "assisted", True, assistance=True)
        provider = verify("provider", "cold", True)
        for phase in ("cold", "warm"):
            verify("client", phase, True, dependency=provider)
        seed = originals[0].read_bytes()
        for name in SAFETY:
            verify(name, "negative", False, "caller_obligation_failure", seed=seed)
        for name in ("false_containment", "false_length"):
            verify(name, "negative", False, "semantic_goal_failure", seed=seed)
        verify("false_containment_auxiliary", "negative", False, "proof_blocked", seed=seed, assistance=True)
        verify("original", "debug_no", True, debug="no")
        verify("release_domain", "debug_no", True, debug="no")
        verify("release_domain", "debug_yes_negative", False, "caller_obligation_failure", seed=seed)
        verify("boundaries", "debug_no", True, debug="no", assistance=True)
        verify("guard_violation", "debug_no", True, debug="no")
        verify("empty_region", "debug_no", True, debug="no")
        verify("cfg_true", "debug_yes", True)
        verify("cfg_false", "debug_no", True, debug="no")
        provider_bytes = provider.read_bytes()
        start = provider_bytes.rfind(b'{"schema":')
        require(start>=0 and provider_bytes.endswith(b"VRINFER1"), "missing checked metadata trailer")
        metadata = json.loads(provider_bytes[start:-8])
        for mutation in ("policy", "exactness", "trusted_model"):
            changed = json.loads(json.dumps(metadata))
            if mutation=="policy":
                changed["policy"] = "candidate-bound-evidence-v61"
            elif mutation=="exactness":
                changed["exactness"] = False
            else:
                changed["trusted_models"][0][1] = "0"*64
            poisoned = output/("provider_poison_"+mutation+".vir")
            poisoned.write_bytes(provider_bytes[:start]+json.dumps(changed,separators=(",", ":")).encode()+b"VRINFER1")
            verify("client", "metadata_"+mutation, False, "frontend_rejection", dependency=poisoned, seed=seed)
        transfer["stale_export_rejected"] = True
        for configuration, options in (("debug", ["-C", "debug-assertions=yes", "-C", "overflow-checks=yes"]),
                                       ("release", ["-O", "-C", "debug-assertions=no", "-C", "overflow-checks=no"])):
            binary = output/("native_"+configuration)
            require(run(configuration+"_compile", ["rustc", "--edition=2021", str(output/"native.rs"), "-o", str(binary)]+options, environment)==0)
            require(run(configuration+"_run", [str(binary)], environment)==0)
        guard.check()
        receipt.update(status="proved", verification_accepted=True, success=True,
                       fixture_hashes={path.name:digest(path) for path in sorted(output.glob("*.rs"))})
        receipt["native"] = {configuration:json.loads((output/(configuration+"_run.stdout")).read_text())
                             for configuration in ("debug", "release")}
        receipt["required_proofs_proved"] = sum(3*len(case["report"].get("verus-infer", {}).get("functions", []))
                                                for case in receipt["cases"])
        transfer["status"] = "proved"
    except BaseException as error:
        receipt["blocker"] = str(error) or type(error).__name__
        transfer["blocker"] = receipt["blocker"]
        raise
    finally:
        (output/"receipt.json").write_text(json.dumps(receipt, indent=2)+"\n")
        print(output, flush=True)


if __name__ == "__main__":
    main()
