#!/usr/bin/env python3
"""Check original NodeMap lookup/insert bodies and a wrapped collector experiment.

size() uses an explicitly recorded constructor spelling in place of derived Into.
This is the supplied collector core with adapted storage, not clean-main Unit/Hasher.
"""
import argparse
import difflib
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import CORRECTED, FIXTURES, METHOD, NODE, ROOT, VALUE, item, prepare, validate_native
from eval_verus_infer_alephbft_records import infer_add, add_mutation, require_domain_rejection
from infer_experiment import require_exact, save_results, snapshot_inputs, validate_run
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

API="verification-infer/node_map.rs"
STATE="verification-infer/collector_state.rs"
PROPERTIES="verification-infer/parents.rs"
METHODS=("size", "get", "insert")


def node_map_source(original, inferred=True, constructor=True):
    methods=[]
    for name in METHODS:
        _,_,body=item(original,f"pub fn {name}(")
        if name=="size" and constructor:
            if body.count("self.0.len().into()")!=1: raise ValueError("size conversion changed")
            body=body.replace("self.0.len().into()","NodeCount(self.0.len())")
        attribute='    #[cfg_attr(verus_keep_ghost, verus_infer)]\n' if inferred else ''
        methods.append(attribute+"    "+body)
    return ("#[derive(Copy, Clone)]\n#[cfg_attr(verus_keep_ghost, verifier::verify)]\npub struct NodeIndex(pub usize);\n"
        '#[cfg_attr(not(verus_keep_ghost), derive(Clone, Eq, PartialEq, Debug))]\n'
        "#[cfg_attr(verus_keep_ghost, verifier::verify)]\npub struct NodeMap<T>(Vec<Option<T>>);\n"
        "#[cfg_attr(verus_keep_ghost, verus_verify)]\nimpl<T> NodeMap<T> {\n"+"\n".join(methods)+"\n}\n")


def wrap_collector(source):
    source=infer_add(source)
    source=source.replace("use super::consensus_threshold;","use super::{consensus_threshold, NodeMap, NodeIndex};")
    source=source.replace("candidates: Vec<Option<(T, u16)>>,","candidates: NodeMap<(T, u16)>,",1)
    source=source.replace("self.candidates","self.candidates.0")
    source=source.replace("            candidates,","            candidates: NodeMap(candidates),")
    a,b,body=item(source,"pub fn add(&mut self")
    body=body.replace("self.candidates.0.len()","self.candidates.size().0")
    body=body.replace("self.candidates.0[creator] = Some((payload, round));","self.candidates.insert(NodeIndex(creator), (payload, round));")
    body=body.replace("self.candidates.0[creator]","self.candidates.get(NodeIndex(creator))")
    body=body.replace("previous_round < round","*previous_round < round")
    return source[:a]+body+source[b:]


def api_mutation(source, method, old, new):
    a,b,body=item(source,f"pub fn {method}(")
    if body.count(old)!=1: raise ValueError("API mutation target is not unique")
    return source[:a]+body.replace(old,new,1)+source[b:]


def validate(run, expected):
    data = validate_run(run, expected)
    functions=data.get("verus-infer",{}).get("functions",[])
    kinds=[f.get("summary_kind") for f in functions]
    if len(functions)!=6 or kinds.count("record_state")!=2 or kinds.count("record_view")!=2 or kinds.count("value")!=2:
        raise ValueError(f"unexpected candidate kinds: {kinds}")
    require_exact(functions)
    for f in functions:
        if f["summary_kind"]!="value" and (f["metrics"]["cache"]!="miss" or f["metrics"]["cache_policy"]!="regenerated_state"):
            raise ValueError("record templates must regenerate and reprove")
    return data


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--alephbft",type=Path,required=True)
    parser.add_argument("--verified-collector",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--verus",type=Path,default=ROOT/"source/target-verus/release/verus")
    parser.add_argument("--z3",type=Path,default=ROOT/"source/z3")
    parser.add_argument("--native-tests",action="store_true")
    parser.add_argument("--native-target",type=Path)
    args=parser.parse_args();output=args.output.resolve();repo=args.alephbft.resolve()
    if output.exists() and any(output.iterdir()): parser.error("output must be new or empty")
    supplied=(args.verified_collector or repo/"consensus/src/creation/collector/state.rs").resolve()
    if not supplied.is_file(): parser.error("provide the supplied collector core")
    output.mkdir(parents=True,exist_ok=True);verus=args.verus.resolve();z3=args.z3.resolve()
    extra=[Path(__file__),supplied,FIXTURES/"parents_properties.rs"]
    extra += [Path(__file__).with_name(n) for n in ("eval_verus_infer_alephbft.py","eval_verus_infer_alephbft_parents.py","eval_verus_infer_alephbft_counting.py","eval_verus_infer_alephbft_records.py","infer_eval_support.py")]
    guard=ArtifactGuard(tool_artifacts(verus,z3,extra=extra))
    result=dict(host=host_info(),tool_artifacts=guard.initial,runs=[],native=[])
    checkout=output/"checkout"
    def save():
        save_results(output, result)
    def snapshot(name, native=False):
        relatives = [VALUE, METHOD, API, STATE, PROPERTIES]
        if native:
            relatives += ["verification-infer/native.rs", "verification-infer/native_reference.rs", "verification-infer/native/Cargo.toml"]
        return snapshot_inputs(checkout, output / ("native-inputs" if native else "inputs") / name, relatives)
    try:
        shared,origin=prepare(repo,checkout)
        (checkout/METHOD).write_text(shared.replace("(*self * 2) / 3 + NodeCount(1)",CORRECTED))
        original_node=git(checkout,"show",f"{REVISION}:{NODE}")
        (output/"original-node.rs").write_text(original_node)
        api=node_map_source(original_node)
        original=supplied.read_text();wrapped=wrap_collector(original)
        properties=(FIXTURES/"parents_properties.rs").read_text().replace("// BRIDGE_ATTRIBUTE","#[verus_infer]")
        properties=properties.replace('#[path = "collector_state.rs"]','include!("node_map.rs");\n\n#[path = "collector_state.rs"]')
        (checkout/PROPERTIES).write_text(properties)
        result["source"]=dict(**origin,collector=str(supplied),collector_sha256=digest(supplied),
            collector_origin="supplied manually verified core with adapted storage, not clean-main generic Unit/Hasher collector",
            node_map_revision=REVISION,lookup_and_insert_original_bodies=True,
            size_adaptation="self.0.len().into() -> NodeCount(self.0.len()); derived Into inference remains unsupported",
            api_scope="three extracted original API methods; unrelated codec/iterator/clone implementations are not verified",
            visibility="API included in the harness parent module so its private storage is transparent to the collector; opaque cross-crate views remain future work",
            retained="supplied invariant, other methods' contracts, loop annotations, count equivalence and behavioral regression oracle")
        for name in ("get","insert"):
            if item(api,f"pub fn {name}(")[2]!=item(original_node,f"pub fn {name}(")[2]: raise ValueError("original API body changed")
        env=dict(os.environ,VERUS_Z3_PATH=str(z3),RUST_MIN_STACK="20971520",VERUS_INFER_CACHE=str(output/"candidate-cache"))
        cases=("unassisted","inferred","inferred-warm","wrong-lookup","wrong-insert","wrong-size","wrong-round","wrong-counter","duplicate-count","changed-frame","missing-accounting","extra-overflow-domain")
        for name in cases:
            candidate_api=api;state=wrapped;expected=name in ("inferred","inferred-warm")
            if name=="wrong-lookup": candidate_api=api_mutation(api,"get","self.0[node_id.0]","self.0[0]")
            if name=="wrong-insert": candidate_api=api_mutation(api,"insert","self.0[node_id.0]","self.0[0]")
            if name=="wrong-size": candidate_api=api_mutation(api,"size","NodeCount(self.0.len())","NodeCount(self.0.len()+1)")
            if name=="wrong-round": state=add_mutation(state,"(payload, round));","(payload, 0));")
            if name=="wrong-counter": state=add_mutation(state,"self.direct_parents += 1","self.direct_parents += 2")
            if name=="duplicate-count": state=add_mutation(state,"*previous_round < round","*previous_round <= round")
            if name=="changed-frame": state=add_mutation(state,"self.direct_parents += 1;","self.direct_parents += 1; self.for_round += 1;")
            if name=="missing-accounting": state=state.replace("#[verus_spec(requires old(self).valid(), ensures final(self).valid())]","#[verus_spec(ensures final(self).valid())]",1)
            if name=="extra-overflow-domain": state=add_mutation(state,"        if creator >=","        let extra = creator + 1;\n        if creator >=")
            (checkout/API).write_text(candidate_api);(checkout/STATE).write_text(state)
            command=[str(verus),str(checkout/PROPERTIES),"--crate-type=lib","--crate-name=aleph_wrappers","--no-cheating","--output-json"]
            if name!="unassisted": command.append("--infer-goals")
            inputs=snapshot(name);guard.check();run=measured(command,env,60,ROOT);guard.check()
            (output/f"{name}.stdout.json").write_text(run["stdout"]);(output/f"{name}.stderr.txt").write_text(run["stderr"])
            data=validate(run,expected)
            if name in ("missing-accounting","extra-overflow-domain","wrong-size"): require_domain_rejection(data,run["stderr"])
            if name=="inferred-warm" and any(f["metrics"]["cache"]!="hit" for f in data["verus-infer"]["functions"] if f["summary_kind"]=="value"):raise ValueError("scalar cache candidates were not reused and rechecked")
            result["runs"].append(dict(case=name,expected_success=expected,classification="verified" if expected else "unassisted_proof_failure" if name=="unassisted" else "semantic_rejection",command=command,returncode=run["returncode"],wall_micros=run["wall_micros"],verification=data["verification-results"],report=data["verus-infer"],**inputs))
            save();print(f"{name}: expected outcome",flush=True)
        if args.native_tests:
            native_dir=checkout/"verification-infer/native";native_dir.mkdir()
            manifest=native_dir/"Cargo.toml"
            manifest.write_text('[package]\nname="aleph-wrapper-shared-core"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[lib]\npath="../native.rs"\n[dependencies]\nderive_more={version="1.0",features=["full"]}\n')
            # Keep the pinned bodies, including size().into(), for a native
            # differential check of the explicitly adapted constructor spelling.
            methods="\n".join(item(original_node,f"pub fn {n}(")[2] for n in METHODS)
            (checkout/"verification-infer/native_reference.rs").write_text(
                "use super::{NodeIndex,NodeCount};\npub struct NodeMap<T>(Vec<Option<T>>);\nimpl<T> NodeMap<T>{\n"+methods+
                "\n}\nimpl<T:Clone> NodeMap<T>{pub fn new(n:usize)->Self{Self(vec![None;n])}}\n")
            _,_,bridge=item(properties,"pub fn consensus_threshold(")
            (checkout/"verification-infer/native.rs").write_text(
                '#[path="../crypto/src/node_count_value.rs"]\nmod node_count_value;\npub use node_count_value::NodeCount;\n#[path="../crypto/src/quorum_method.rs"]\nmod quorum_method;\n'
                +bridge+'\ninclude!("node_map.rs");\n#[path="collector_state.rs"]\nmod collector;\n'
                '#[cfg(test)]\nmod native_reference;\n#[cfg(test)]\nmod api_tests{use super::*;\n'
                '#[test] fn size_lookup_insert_match_pinned_implementation(){for n in 0..8{let mut a=NodeMap(vec![None;n]);let mut b=native_reference::NodeMap::new(n);assert_eq!(a.size().0,b.size().0);for i in 0..n{assert_eq!(a.get(NodeIndex(i)),b.get(NodeIndex(i)));a.insert(NodeIndex(i),i as u64+10);b.insert(NodeIndex(i),i as u64+10);for j in 0..n{assert_eq!(a.get(NodeIndex(j)),b.get(NodeIndex(j)));}}}}}\n')
            native_env=dict(os.environ);native_env.pop("RUSTUP_TOOLCHAIN",None)
            if args.native_target: native_env["CARGO_TARGET_DIR"]=str(args.native_target.resolve())
            native_cases=[("original",api,original,True,"collector::tests"),
                ("wrapped",api,wrapped,True,"collector::tests"),
                ("wrong-lookup",api_mutation(api,"get","self.0[node_id.0]","self.0[0]"),wrapped,False,"collector::tests"),
                ("wrong-insert",api_mutation(api,"insert","self.0[node_id.0]","self.0[0]"),wrapped,False,"collector::tests"),
                ("extra-overflow-domain",api,add_mutation(wrapped,"        if creator >=","        let extra = creator + 1;\n        if creator >="),False,"collector::tests"),
                ("api-value-equivalence",api,wrapped,True,"api_tests"),
                ("wrong-size",api_mutation(api,"size","NodeCount(self.0.len())","NodeCount(self.0.len()+1)"),wrapped,False,"api_tests")]
            for name,native_api,state,expected,selection in native_cases:
                (checkout/API).write_text(native_api);(checkout/STATE).write_text(state)
                command=["cargo","test","--offline","--manifest-path",str(manifest),"--lib",selection,"--","--nocapture"]
                inputs=snapshot(name,True);guard.check();run=measured(command,native_env,180,checkout);guard.check()
                (output/f"native-{name}.txt").write_text(run["stdout"]+run["stderr"])
                validate_native(run,expected,name)
                result["native"].append(dict(case=name,expected_success=expected,command=command,returncode=run["returncode"],wall_micros=run["wall_micros"],**inputs))
                save();print(f"native {name}: expected outcome",flush=True)
            if (native_dir/"Cargo.lock").exists(): (output/"native-Cargo.lock").write_text((native_dir/"Cargo.lock").read_text())
        (checkout/API).write_text(api);(checkout/STATE).write_text(wrapped)
        (output/"wrapper-inference.patch").write_text("".join(difflib.unified_diff(original.splitlines(keepends=True),wrapped.splitlines(keepends=True),fromfile="a/collector_state.rs",tofile="b/collector_state.rs")))
        guard.check();result["success"]=True
        (output/"summary.md").write_text("# Aleph NodeMap composition\n\nOriginal get/insert bodies and an explicitly constructor-adapted size method receive independent exact checks. The supplied collector uses these checked helpers and preserves its whole valid domain. This is adapted supplied-core storage, not the clean-main generic Unit/Hasher collector. Opaque cross-crate collection views and derived Into inference remain open.\n\n"+"\n".join(f"- {r['case']}: {r['classification']}" for r in result["runs"])+"\n")
    except Exception as error:
        result["success"]=False;result["error"]=str(error);raise
    finally:save()

if __name__=="__main__":main()
