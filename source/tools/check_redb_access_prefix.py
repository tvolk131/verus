"""Replay redb's readonly range/length prefix, not storage or TransactionalMemory."""
import argparse
import json
import os
from pathlib import Path

from check_redb_address_range import HEADER, ITEM_HASHES
from check_redb_page_relations import domain
from check_redb_page_size import APPLICATION, BASE, SOURCE_HASH, item
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require, require_exact, require_publication, snapshot_inputs, validate_run

ROOT = Path(__file__).resolve().parents[2]
MANAGER = "src/tree_store/page_store/page_manager.rs"
MANAGER_HASH = "b062608a527c3f53aa505af870f2b803da3fa7b095169194c681cc6441cb558a"
PREFIX_HASH = "736042c5874ec5fb2cc02da02d3cb248dcb81a426e5027fe3feb8ed7e5c5b152"
CONFIG = "pub struct AccessConfig {pub page_size:u32,pub region_size:u64,pub region_header_with_padding_size:u64}\n"
COUNT = "((1u64 << order) as int * size as int)"
PARAMETERS = "region:u32,index:u32,order:u8,size:u32,region_size:u64,header:u64"
REQUIREMENTS = [condition.replace("pages_start", "header").replace("offset", "size") for condition in domain("index")] + [COUNT+"<=usize::MAX"]
CALL = "AccessConfig{page_size:size,region_size,region_header_with_padding_size:header}.access_length(PageNumber{region,page_index:index,page_order:order})"

def caller(name="length", goal="result as int=="+COUNT, requirements=None):
    return (f"fn {name}({PARAMETERS})->(result:usize)\n requires "+",".join(REQUIREMENTS if requirements is None else requirements)
            +f",\n ensures {goal},\n{{{CALL}}}\n")

def relationship():
    return (f"fn range_length({PARAMETERS})->(result:(usize,Range<u64>))\n requires "+",".join(REQUIREMENTS)
            +f",\n ensures result.0 as int==result.1.end as int-result.1.start as int,result.0 as int=={COUNT},\n"
            +"{let len="+CALL+"; let range=PageNumber{region,page_index:index,page_order:order}.address_range(size.into(),region_size,header,size); (len,range)}\n")

def classify(data, stderr, code):
    report = data.get("verus-infer", {})
    if code == 0:
        return "proved"
    if any(word in stderr.lower() for word in ("resource limit", "timed out", "panicked at")):
        return "resource_or_runtime_failure"
    if any(message in stderr for message in ("Verus Infer: bit-vector",
            "Verus Infer: polynomial proposal not satisfied", "bitvector assertion not satisfied")):
        return "mixed_failure" if "precondition not satisfied" in stderr or "postcondition not satisfied" in stderr else "auxiliary_theorem_failure"
    if report.get("status") in ("unsupported", "invalid_import") or data["verification-results"].get("encountered-vir-error"):
        return "frontend_rejection"
    if "precondition not satisfied" in stderr:
        return "safety_failure"
    if "postcondition not satisfied" in stderr:
        return "semantic_goal_failure"
    return "unclassified_failure"

def extract(application, output):
    require(digest(application/BASE)==SOURCE_HASH, "base.rs pin changed")
    source = (application/BASE).read_text()
    pieces = {name:item(source, marker) for name, marker in [
        ("PageNumber", "pub(crate) struct PageNumber"),
        ("page_size_bytes", "    pub(crate) fn page_size_bytes("),
        ("address_range", "    pub(crate) fn address_range(")]}
    for name, piece in pieces.items():
        path = output/(name+".original.rs")
        path.write_text(piece+"\n")
        require(digest(path)==ITEM_HASHES[name], "helper extraction changed: "+name)
    require(digest(application/MANAGER)==MANAGER_HASH, "page_manager.rs pin changed")
    manager = (application/MANAGER).read_text()
    transaction = manager[manager.index("impl TransactionalMemory {"):]
    method = item(transaction, "    pub(crate) fn get_page(&self, page_number: PageNumber, hint: PageHint)")
    prefix = method[method.index("        let range ="):method.index("        let mem =")]
    (output/"get_page.original.rs").write_text(method+"\n")
    (output/"get_page_prefix.original.rs").write_text(prefix)
    require(digest(output/"get_page_prefix.original.rs")==PREFIX_HASH, "access prefix changed")
    require(prefix.count(".into()") == 1 and prefix.count(".try_into().unwrap()") == 1)
    helpers = pieces["PageNumber"]+"\nimpl PageNumber {\n"+"\n".join(
        "#[verifier::infer]\n"+pieces[name] for name in ("page_size_bytes", "address_range"))+"\n}\n"
    access = CONFIG+"impl AccessConfig {\n#[verifier::infer]\npub fn access_length(&self,page_number:PageNumber)->usize {\n"+prefix+"        len\n}\n}\n"
    return helpers+access, dict(application_commit=APPLICATION, manager_sha256=digest(application/MANAGER),
        get_page_sha256=digest(output/"get_page.original.rs"), prefix_sha256=digest(output/"get_page_prefix.original.rs"),
        method_line=manager[:manager.index(method)].count("\n")+1, prefix_line=manager[:manager.index(prefix)].count("\n")+1,
        next_blocker="self.storage.read(range.start, len, hint)?; no storage/state model attempted",
        adaptation="Three-field readonly AccessConfig; prefix copied verbatim; append len return; omit check_page_order, storage.read, tracking and PageImpl construction")

def native(source):
    source = source.replace("#[verifier::infer]\n", "").replace("use vstd::prelude::*;\n", "")
    return source+"""
fn main() {
    let mut accepted = 0u64;
    let mut rejected32 = 0u64;
    for order in 0u8..64 {
        for size in [0u32,1,2,4096,u32::MAX] {
            let count=(1u128<<order)*u128::from(size);
            if count>u128::from(u64::MAX) {continue;}
            for region in [0u32,1,u32::MAX] {for index in [0u32,1,u32::MAX] {
                for header in [0u64,1,4096,u64::MAX] {for region_size in [0u64,1,8192,u64::MAX] {
                    let regional=u128::from(header)+u128::from(index)*count;
                    let base=u128::from(region)*u128::from(region_size);
                    let start=u128::from(size)+base+regional;
                    if regional>=u128::from(region_size)||start+count>u128::from(u64::MAX) {continue;}
                    assert_eq!(u32::try_from(count).is_ok(),count<=u128::from(u32::MAX));
                    if u32::try_from(count).is_err(){rejected32+=1;}
                    if count>usize::MAX as u128 {continue;}
                    let config=AccessConfig{page_size:size,region_size,region_header_with_padding_size:header};
                    let page=PageNumber{region,page_index:index,page_order:order};
                    let range=page.address_range(size.into(),region_size,header,size);
                    assert_eq!(u128::from(range.end-range.start),count);
                    let result=config.access_length(page);
                    assert_eq!(result as u128,count);
                    accepted+=1;
                }}
            }}
        }
    }
    assert!(accepted>1000 && rejected32>0);
    println!("host_pointer_bits={} safe_accesses={} outside_32bit_length_domain={}",usize::BITS,accepted,rejected32);
}
"""

def run_suite(args):
    output=args.output.resolve()
    output.mkdir(parents=True,exist_ok=False)
    environment=dict(os.environ,VERUS_Z3_PATH=str(args.solver.resolve()),PYTHONDONTWRITEBYTECODE="1")
    receipt=dict(status="blocked",commands=[],cases=[],trust=["Existing canonical vstd From/Into/TryFrom/TryInto and Result::unwrap specifications", "Ordinary Verus backend and solver; transparent core Range correspondence", "No storage, locks, maps, occupancy or whole-get_page model"],baseline_tool_difference="Reviewed sealed70 tools lack parent's module-relative native_option import and core regression test; used only for initial diagnostic, never final evidence")
    commands=LoggedCommands(output,ROOT,receipt["commands"],timeout=args.timeout)
    with replay_receipt(output,receipt):
        for label,argv,expected in [("pin",["rev-parse","HEAD"],APPLICATION),("clean",["status","--porcelain","--untracked-files=all"],"")]:
            require(commands("application_before_"+label,["git","-C",str(args.application),*argv],environment)==0)
            require((output/("application_before_"+label+".stdout")).read_text().strip()==expected)
        definitions,receipt["extraction"]=extract(args.application,output)
        definitions=definitions.replace("pub(crate)","pub")
        provider=HEADER+definitions+"}\n"
        require(commands("engine_files",["git","ls-files","--cached","--others","--exclude-standard","--","source","dependencies","rust-toolchain.toml"],environment)==0)
        inputs=sorted(set((output/"engine_files.stdout").read_text().splitlines()))
        receipt.update(snapshot_inputs(ROOT,output/"inputs",inputs))
        guard=ArtifactGuard(tool_artifacts(args.verus.resolve(),args.solver.resolve(),extra=[ROOT/path for path in inputs]))
        commands.guard=guard
        receipt["artifacts"]=guard.initial

        def verify(label,source,expected="proved",inferred=3,dependency=None,compile=False,seed=None,source_label=None):
            source_label=source_label or label
            path=output/(source_label+".rs")
            path.write_text(source)
            artifact=output/(label+".vir")
            if seed is not None:
                artifact.write_bytes(seed)
            command=[str(args.verus.resolve()),str(path),"--crate-type=lib","--crate-name",source_label,"--output-json","--export",str(artifact),"-o",str(output/("lib"+label+".rlib")),"-C","debug-assertions=yes"]
            model_specific="global size_of usize" in source
            if not model_specific:
                command.append("--no-cheating")
            command.append("--compile" if compile else "--no-erasure-check")
            if dependency is not None:
                command += ["--extern","provider="+str(output/"libprovider.rlib"),"--import","provider="+str(dependency)]
            environment["VERUS_INFER_CACHE"]=str(output/"candidate-cache")
            code=commands(label,command,environment)
            stdout=(output/(label+".stdout")).read_text()
            stderr=(output/(label+".stderr")).read_text()
            data=json.loads(stdout)
            observed=classify(data,stderr,code)
            receipt["cases"].append(dict(label=label,classification=observed,report=data,artifact_exists=artifact.exists(),stale_export_seeded=seed is not None,no_cheating=not model_specific,architecture_configuration="canonical global size_of usize declaration" if model_specific else "portable Either32Or64"))
            require(observed==expected,label+": "+observed+"\n"+stderr[-2500:])
            validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),expected=="proved",allow_frontend=expected=="frontend_rejection")
            if expected!="frontend_rejection":
                functions=data["verus-infer"]["functions"]
                require(len(functions)==inferred,"inferred scope changed")
                require_exact(functions)
                require(data["verification-results"].get("is-verifying-entire-crate") is True)
                if expected=="semantic_goal_failure":
                    require("precondition not satisfied" not in stderr and "proposal not satisfied" not in stderr)
            require_publication(data,expected=="proved",artifact)
            return artifact

        seed=verify("provider",provider,compile=True).read_bytes()
        verify("portable_length",HEADER+definitions+caller()+relationship()+"}\n",compile=True)
        for width in (32,64):
            prefix=HEADER+f"global size_of usize == {width//8};\n"+definitions
            good=prefix+caller()+relationship()+"}\n"
            verify(f"length_{width}",good)
            verify(f"warm_{width}",good,source_label=f"length_{width}")
            cache=output/"candidate-cache"
            cache.mkdir(exist_ok=True)
            (cache/"poisoned-record-proposal.json").write_text('{"version":"verus-infer-v70","pre":true,"post":false}')
            for path in (output/"candidate-cache").glob("*.json"):
                path.write_text('{"version":"verus-infer-v70","pre":"poisoned"}')
            verify(f"poisoned_{width}",good,source_label=f"length_{width}")
            verify(f"false_length_{width}",prefix+caller(goal="result as int!="+COUNT)+"}\n", "semantic_goal_failure",seed=seed)
            verify(f"missing_domain_{width}",prefix+caller(requirements=["order<64"])+"}\n","safety_failure",seed=seed)
            bad=prefix+"fn rejected()->usize {AccessConfig{page_size:1,region_size:1,region_header_with_padding_size:0}.access_length(PageNumber{region:0,page_index:0,page_order:64})}\n}\n"
            verify(f"shift64_{width}",bad,"safety_failure",seed=seed)
            for name,values in {
                "byte_multiply_overflow":(0,0,63,2,1,0),
                "index_multiply_overflow":(0,4294967295,63,1,18446744073709551615,0),
                "region_multiply_overflow":(2,0,0,0,18446744073709551615,0),
                "offset_add_overflow":(1,0,0,1,18446744073709551615,0),
                "end_add_overflow":(0,0,0,1,18446744073709551615,18446744073709551614),
                "debug_guard":(0,0,0,0,1,1),
            }.items():
                region,index,order,size,region_size,header=values
                call=f"AccessConfig{{page_size:{size},region_size:{region_size},region_header_with_padding_size:{header}}}.access_length(PageNumber{{region:{region},page_index:{index},page_order:{order}}})"
                verify(f"{name}_{width}",prefix+"fn rejected()->usize {"+call+"}\n}\n","safety_failure",seed=seed)
            scalar=HEADER+f"global size_of usize == {width//8};\n"+"#[verifier::infer] fn narrow(number:u128)->usize {number.try_into().unwrap()}\nfn bad()->usize {narrow("+str(1<<width)+")}\n}\n"
            verify(f"out_of_range_{width}",scalar,"safety_failure",inferred=1,seed=seed)
            signed=HEADER+f"global size_of usize == {width//8};\n"+"#[verifier::infer] fn narrow(number:i64)->usize {number.try_into().unwrap()}\nfn bad()->usize {narrow(-1)}\n}\n"
            verify(f"negative_signed_{width}",signed,"safety_failure",inferred=1,seed=seed)
            if width==32:
                call="AccessConfig{page_size:1,region_size:1,region_header_with_padding_size:0}.access_length(PageNumber{region:0,page_index:0,page_order:32})"
                verify("prefix_outside_32bit_length",prefix+"fn rejected()->usize {"+call+"}\n}\n","safety_failure",seed=seed)
            client=HEADER+f"global size_of usize == {width//8};\n#[verifier::infer] pub fn relay(config:&provider::AccessConfig,page:provider::PageNumber)->usize {{config.access_length(page)}}\n"
            client+=caller().replace(CALL,"relay(&provider::AccessConfig{page_size:size,region_size,region_header_with_padding_size:header},provider::PageNumber{region,page_index:index,page_order:order})")+"}\n"
            verify(f"imported_{width}",client,inferred=1,dependency=output/"provider.vir")
            verify(f"imported_false_{width}",client.replace("result as int=="+COUNT,"result as int!="+COUNT),"semantic_goal_failure",inferred=1,dependency=output/"provider.vir",seed=seed)
            metadata_start=seed.rindex(b'{"schema":')
            metadata=json.loads(seed[metadata_start:-8])
            for control in ("stale_policy","poisoned_models"):
                changed=json.loads(json.dumps(metadata))
                if control=="stale_policy":
                    changed["policy"]="candidate-bound-evidence-v64"
                else:
                    changed["trusted_models"][0][1]="poisoned"
                dependency=output/(control+".vir")
                dependency.write_bytes(seed[:metadata_start]+json.dumps(changed,separators=(",",":")).encode()+b"VRINFER1")
                verify(f"{control}_{width}",client,"frontend_rejection",inferred=0,dependency=dependency,seed=seed)
        (output/"native.rs").write_text(native("use std::ops::Range;\n"+definitions))
        require(commands("native_compile",["rustup","run","1.99.1","rustc","--edition=2021","-C","opt-level=3","-C","debug-assertions=yes",str(output/"native.rs"),"-o",str(output/"native")],environment)==0)
        require(commands("native_execution",[str(output/"native")],environment)==0)
        for label,argv,expected in [("pin",["rev-parse","HEAD"],APPLICATION),("clean",["status","--porcelain","--untracked-files=all"],"")]:
            require(commands("application_after_"+label,["git","-C",str(args.application),*argv],environment)==0)
            require((output/("application_after_"+label+".stdout")).read_text().strip()==expected)
        guard.check()
        receipt["status"]="proved"
    print(json.dumps(dict(receipt=str(output/"receipt.json"),status=receipt["status"])))

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application",type=Path,required=True)
    parser.add_argument("--verus",type=Path,required=True)
    parser.add_argument("--solver",type=Path,required=True)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--timeout",type=positive_timeout,default=180)
    run_suite(parser.parse_args())

if __name__=="__main__":
    main()
