"""Replay source-connected same-order redb page relations and honest boundaries."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile

from check_redb_address_range import HEADER, ITEM_HASHES, SAFETY, witness
from check_redb_page_size import APPLICATION, BASE, SOURCE_HASH, item
from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import LoggedCommands, positive_timeout, require, require_exact, require_publication, snapshot_inputs, validate_run

ROOT = Path(__file__).resolve().parents[2]
BYTE_COUNT = "((1u64 << order) as int * size as int)"

def domain(index, order="order", size="size"):
    """Every checked intermediate and the active original debug guard."""
    count = BYTE_COUNT.replace("order", order).replace("size", size)
    product = f"{index} as int * {count}"
    regional = f"(pages_start as int + {product})"
    base = "(region as int * region_size as int)"
    offset_base = f"(offset as int + {base})"
    start = f"({offset_base} + {regional})"
    return [f"{order}<64", f"{count}<=u64::MAX", f"{product}<=u64::MAX",
            f"{regional}<=u64::MAX", f"{regional}<region_size", f"{base}<=u64::MAX",
            f"{offset_base}<=u64::MAX", f"{start}<=u64::MAX", f"{start}+{count}<=u64::MAX"]

PARAMETERS = "region:u32,first:u32,second:u32,order:u8,size:u32,offset:u64,region_size:u64,pages_start:u64"
RESULT = "(result:(Range<u64>,Range<u64>))"
BODY = """{
    let a=PageNumber{region,page_index:first,page_order:order}.address_range(offset,region_size,pages_start,size);
    let b=PageNumber{region,page_index:second,page_order:order}.address_range(offset,region_size,pages_start,size);
    (a,b)
}
"""

def relation(name, condition, goal, positive=True):
    requirements = domain("first") + domain("second") + [condition]
    if positive:
        requirements.append("size>0")
    return (f"fn {name}({PARAMETERS})->{RESULT}\n    requires " + ",\n        ".join(requirements)
            + f",\n    ensures {goal},\n" + BODY)

RELATIONS = (relation("strict_start", "first<second", "result.0.start<result.1.start")
             + relation("injective_start", "first!=second", "result.0.start!=result.1.start")
             + relation("ordered_nonoverlap", "first<second", "result.0.end<=result.1.start")
             + relation("nonoverlap_all_sizes", "first<second", "result.0.end<=result.1.start", False))
ZERO = relation("zero_size", "first!=second && size==0",
                "result.0.start==result.0.end && result.0.start==result.1.start && result.1.start==result.1.end && result.0.end<=result.1.start", False)
WITNESSES = """
fn zero_witness()->(result:(Range<u64>,Range<u64>))
    ensures result.0.start==0,result.0.end==0,result.1.start==0,result.1.end==0,
        result.0.end<=result.1.start,
{
    let a=PageNumber{region:0,page_index:0,page_order:63}.address_range(0,1,0,0);
    let b=PageNumber{region:0,page_index:1,page_order:63}.address_range(0,1,0,0);
    (a,b)
}
fn mixed_witness()->(result:(Range<u64>,Range<u64>))
    ensures result.0.start==0,result.0.end==8192,result.1.start==4096,result.1.end==8192,
        result.0.start<result.1.end && result.1.start<result.0.end,
{
    let a=PageNumber{region:0,page_index:0,page_order:1}.address_range(0,8192,0,4096);
    let b=PageNumber{region:0,page_index:1,page_order:0}.address_range(0,8192,0,4096);
    (a,b)
}
"""
NEGATIVES = {
    "false_strict_zero": "fn wrong()->"+RESULT+" ensures result.0.start<result.1.start, {zero_witness()}\n",
    "false_injective_zero": "fn wrong()->"+RESULT+" ensures result.0.start!=result.1.start, {zero_witness()}\n",
    "false_mixed_nonoverlap": "fn wrong()->"+RESULT+" ensures result.0.end<=result.1.start || result.1.end<=result.0.start, {mixed_witness()}\n",
}

def extract(application, output):
    original = application / BASE
    require(digest(original) == SOURCE_HASH, "pinned application source changed")
    text = original.read_text()
    pieces = {"PageNumber": item(text, "pub(crate) struct PageNumber"),
              "page_size_bytes": item(text, "    pub(crate) fn page_size_bytes("),
              "address_range": item(text, "    pub(crate) fn address_range(")}
    # Validate every extracted item before writing any fixture.
    import hashlib
    for name, source in pieces.items():
        require(hashlib.sha256((source+"\n").encode()).hexdigest() == ITEM_HASHES[name],
                "pinned extraction changed: " + name)
    for name, source in pieces.items():
        (output / (name+".original.rs")).write_text(source+"\n")
    (output / "base.original.rs").write_bytes(original.read_bytes())
    inferred = pieces["PageNumber"]+"\nimpl PageNumber {\n"+"\n".join(
        "#[verifier::infer]\n"+pieces[name] for name in ("page_size_bytes", "address_range"))+"\n}\n"
    sources = {"relations": HEADER+inferred+RELATIONS+"}\n",
               "zero": HEADER+inferred+ZERO+"}\n",
               "witnesses": HEADER+inferred+WITNESSES+"}\n",
               "provider": HEADER+inferred.replace("pub(crate)", "pub")+"}\n"}
    relay = """
#[verifier::infer]
pub fn relay(page:&provider::PageNumber,offset:u64,region_size:u64,pages_start:u64,size:u32)->Range<u64> {
    page.address_range(offset,region_size,pages_start,size)
}
"""
    client = RELATIONS.replace("PageNumber{", "provider::PageNumber{")
    for index in ("first", "second"):
        client = client.replace(
            f"provider::PageNumber{{region,page_index:{index},page_order:order}}.address_range(offset,region_size,pages_start,size)",
            f"relay(&provider::PageNumber{{region,page_index:{index},page_order:order}},offset,region_size,pages_start,size)")
    sources["client"] = HEADER+relay+client+"}\n"
    for name, caller in NEGATIVES.items():
        sources[name] = HEADER+inferred+WITNESSES+caller+"}\n"
    sources["false_polynomial"] = HEADER+inferred+relation(
        "wrong", "first<second", "result.0.start>result.1.start")+"}\n"
    for name, values in SAFETY.items():
        sources[name] = HEADER+inferred+"fn rejected()->Range<u64>{"+witness(values)+"}\n}\n"
    for name, source in sources.items():
        require(not any(fragment in source for fragment in ("proof {", "by(bit_vector)", "by(nonlinear_arith)",
                    "assume(", "external_body", "axiom", "order<=20")), "fixture contains forbidden proof glue")
        (output/(name+".rs")).write_text(source)
    return {name:dict(sha256=digest(output/(name+".original.rs")), source=BASE,
                      line=text[:text.index(contents)].count("\n")+1) for name,contents in pieces.items()}


def classify(data, stderr, status):
    if any(s in stderr.lower() for s in ("panicked at", "resource limit", "timed out")):
        return "resource_or_internal_failure"
    result = data.get("verification-results", {})
    if result.get("encountered-vir-error") or "success" not in result or "error[E" in stderr:
        return "frontend_rejection"
    if status == 0 and result.get("success") is True:
        return "proved"
    functions = data.get("verus-infer", {}).get("functions", [])
    if not functions or not all(f.get("required_checks_passed") for f in functions):
        return "inference_failure"
    if "polynomial proposal not satisfied" in stderr or "bitvector assertion not satisfied" in stderr:
        return "auxiliary_theorem_failure"
    pre, post = "precondition not satisfied" in stderr, "postcondition not satisfied" in stderr
    if pre and not post:
        return "caller_obligation_failure"
    if post and not pre:
        return "semantic_goal_failure"
    return "proof_blocked"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("application", "verus", "solver", "output"):
        parser.add_argument("--"+option, type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="redb-page-relations-", dir=args.output.resolve()))
    environment = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", RUST_MIN_STACK="20971520",
                       VERUS_Z3_PATH=str(args.solver.resolve()), PYTHONDONTWRITEBYTECODE="1")
    transfer = dict(scenario="redb_page_relations", application_commit=APPLICATION, status="blocked",
        source_fidelity=False, positive_cases=[], semantic_negative_cases=[], safety_negative_cases=[],
        auxiliary_negative_cases=[], polynomial_auxiliary_negative_cases=[], metadata_negative_cases=[],
        stale_export_rejected=False,
        limitations=["Extracted original helpers and authored relational callers; no allocator or database invariant",
                     "Full checked arithmetic and active debug guard; native release wraparound is outside the theorem",
                     "Warm readonly/owned summaries regenerate and reprove; no cache-hit claim",
                     "Zero size refutes strict/injective starts while endpoint nonoverlap still holds",
                     "Distinct mixed-order page numbers can overlap; allocator occupancy requires a separate invariant"],
        trust=["Ordinary Verus backend, NIA/BV solvers, existing vstd conversions and requires-false panic specification",
               "Existing canonical transparent core Range<u64>/ExRange correspondence; no opaque external type adaptation",
               "No new axioms, unchecked summaries, arithmetic assumptions or skipped executable checks"])
    receipt = dict(status="blocked", output=str(output), application_commit=APPLICATION, transfer=transfer,
                   host=host_info(), commands=[], cases=[], verification_accepted=False, success=False)
    run = LoggedCommands(output, ROOT, receipt["commands"], timeout=args.timeout)
    def check_application(phase):
        for suffix, arguments, expected in (("pin", ["rev-parse", "HEAD"], APPLICATION),
                  ("clean", ["status", "--porcelain", "--untracked-files=all"], "")):
            label = "application_"+phase+"_"+suffix
            require(run(label, ["git", "-C", str(args.application), *arguments], environment)==0)
            require((output/(label+".stdout")).read_text().strip()==expected, "application checkout pin/cleanliness changed")
    try:
        check_application("before")
        receipt["extraction"] = extract(args.application, output)
        transfer["source_fidelity"] = True
        require(run("engine_files", ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--", "source", "dependencies", "rust-toolchain.toml"], environment)==0)
        inputs = sorted(set((output/"engine_files.stdout").read_text().splitlines()))
        guard = ArtifactGuard(tool_artifacts(args.verus.resolve(), args.solver.resolve(), extra=[ROOT/p for p in inputs]))
        receipt["artifacts"] = guard.initial
        receipt.update(snapshot_inputs(ROOT, output/"inputs", inputs))
        run.guard = guard
        def verify(name, phase, success, expected="proved", assistance=True, dependency=None, seed=None, category=None):
            label = name+"_"+phase
            artifact = output/(label+".vir")
            if seed is not None:
                require(seed, "empty stale-interface seed")
                artifact.write_bytes(seed)
            environment["VERUS_INFER_CACHE"] = str(output/(name+"-cache"))
            command = [str(args.verus.resolve()), str(output/(name+".rs")), "--crate-type", "lib", "--crate-name", name,
                "--no-cheating", "--output-json", "--compile", "--export", str(artifact),
                "-o", str(output/("lib"+name+".rlib")), "-C", "debug-assertions=yes"]
            if dependency is not None:
                provider_guard.check()
                command += ["--extern", "provider="+str(output/"libprovider.rlib"), "--import", "provider="+str(dependency)]
            if assistance:
                command.append("--infer-goals")
            status = run(label, command, environment)
            if dependency is not None:
                provider_guard.check()
            stdout, stderr = (output/(label+".stdout")).read_text(), (output/(label+".stderr")).read_text()
            data = json.loads(stdout)
            observed = classify(data, stderr, status)
            receipt["cases"].append(dict(label=label, classification=observed, report=data,
                artifact=str(artifact), artifact_exists=artifact.exists(), stale_export_seeded=seed is not None))
            require(observed==expected, label+": unexpected classification: "+observed+"; "+stderr[-2500:])
            validate_run(dict(returncode=status, stdout=stdout, stderr=stderr), success,
                         allow_frontend=expected=="frontend_rejection")
            functions = data.get("verus-infer", {}).get("functions", [])
            require(len(functions)==(0 if expected=="frontend_rejection" else 1 if name=="client" else 2), "inference scope changed")
            require_exact(functions)
            require_publication(data, success, artifact)
            if expected!="frontend_rejection":
                require(data["verification-results"].get("is-verifying-entire-crate") is True, "partial checking")
            if category is None:
                category = "positive_cases" if success else {"semantic_goal_failure":"semantic_negative_cases",
                    "caller_obligation_failure":"safety_negative_cases", "frontend_rejection":"metadata_negative_cases",
                    "auxiliary_theorem_failure":"polynomial_auxiliary_negative_cases"}[expected]
            transfer[category].append(label)
            return artifact
        positive = verify("relations", "cold", True)
        verify("relations", "warm", True)
        poison = output/"relations-cache"/"poisoned.json"
        poison.parent.mkdir(exist_ok=True)
        poison.write_text('{"version":"verus-infer-v70","pre":true,"post":false}')
        verify("relations", "poisoned_cache", True)
        verify("zero", "ordinary", True, assistance=False)
        verify("witnesses", "checked", True)
        provider = verify("provider", "cold", True)
        provider_guard = ArtifactGuard([provider, output/"libprovider.rlib"])
        receipt["provider_artifacts_before"] = provider_guard.initial
        for phase in ("cold", "warm"):
            verify("client", phase, True, dependency=provider)
        seed = positive.read_bytes()
        for name in NEGATIVES:
            verify(name, "negative", False, "semantic_goal_failure", seed=seed)
        for name in SAFETY:
            verify(name, "negative", False, "caller_obligation_failure", seed=seed, assistance=False)
        verify("false_polynomial", "negative", False, "auxiliary_theorem_failure", seed=seed)
        raw = provider.read_bytes();start = raw.rfind(b'{"schema":')
        require(start>=0 and raw.endswith(b"VRINFER1"), "missing provider metadata")
        metadata = json.loads(raw[start:-8])
        for mutation in ("policy", "exactness", "summaries", "trusted_model"):
            changed = json.loads(json.dumps(metadata))
            if mutation=="policy": changed["policy"] = "candidate-bound-evidence-v0"
            elif mutation=="exactness": changed["exactness"] = False
            elif mutation=="summaries": changed["summaries"] = "0"*64
            else: changed["trusted_models"][0][1] = "0"*64
            path = output/("provider_poison_"+mutation+".vir")
            path.write_bytes(raw[:start]+json.dumps(changed,separators=(",", ":")).encode()+b"VRINFER1")
            verify("client", "metadata_"+mutation, False, "frontend_rejection", dependency=path, seed=seed)
        check_application("after")
        guard.check()
        provider_guard.check()
        receipt["provider_artifacts_after"] = provider_guard.snapshot()
        transfer.update(status="proved", stale_export_rejected=True)
        receipt.update(status="proved", verification_accepted=True, success=True,
            fixture_hashes={p.name:digest(p) for p in output.glob("*.rs")},
            required_proofs_proved=sum(3*len(c["report"].get("verus-infer",{}).get("functions",[])) for c in receipt["cases"]))
    except BaseException as error:
        receipt["blocker"] = str(error) or type(error).__name__
        transfer["blocker"] = receipt["blocker"]
        raise
    finally:
        (output/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
        print(output, flush=True)

if __name__ == "__main__":
    main()
