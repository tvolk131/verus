"""Isolated source-connected redb repair-observation experiment, cycle80i."""

import argparse
import hashlib
import json
from pathlib import Path
import re

import check_redb_slot_selection as selection
import check_redb_slot_selection_opaque as opaque
from check_redb_page_size import item
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "source/tools/fixtures/transfers/transfer-redb-repair-observation80i"
PIN = "c82ab865bb362bc4af067d332adc44978ff64813"
DB = "src/db.rs"
HEADER_HASH = "35863472a48f9e0eb65a0f9ba0b2517a888a881478476aaafbcbfc77842850f9"
DB_HASH = "65c3107446039a3dacba03dae0b2a21c88109562ba81d7cadda0a8ea8c7a6169"
SCOPE_KEYS = selection.SCOPE_KEYS
LEDGERS = {
    "file-hashes.json": ("660c95790787d161302cf2a71eb9d0afbebaa56e3dd33d530c2ac625f43bb070", 2145),
    "runtime-hashes.json": ("8554d638ca7061606efc6cf080bb7a21898103ec0d0b3094b780ec26e84c328b", 9),
    "toolchain-hashes.json": ("367af9e5ee0e1be37bb1582f562dcf929f4eaf3c4acad6805f0d4c0b37f1f596", 61),
}


def check(condition, guard):
    if not condition:
        raise ValueError(guard)


def read(path):
    return selection.read_json(path)


def write(path, value):
    selection.write_json(path, value)


def audit_ledger(snapshot, name, root):
    expected, count = LEDGERS[name]
    check(digest(snapshot / name) == expected, "ledger_identity")
    ledger = read(snapshot / name)
    check(len(ledger) == count, "ledger_count")
    check(all(digest(root / relative) == value for relative, value in ledger.items()), "ledger_contents")
    return ledger


def extract(application):
    check(digest(application / selection.HEADER) == HEADER_HASH, "native_header")
    check(digest(application / DB) == DB_HASH, "native_db")
    pieces = selection.extract(application)
    pieces["primary_verifies"] = item((application / DB).read_text(), "    fn primary_verifies(")
    pieces["finalize_transaction_slots"] = item((application / selection.HEADER).read_text(),
        "    pub(super) fn finalize_transaction_slots(")
    pieces["from_bytes"] = item((application / selection.HEADER).read_text().split("impl TransactionHeader {", 1)[1],
        "    pub(super) fn from_bytes(")
    return pieces


def scaffold(pieces):
    declarations = json.loads((ROOT / "source/tools/fixtures/transfers/transfer-redb-slot-selection-opaque78/fixtures.json").read_text())["opaque_declarations"]
    source = "use vstd::prelude::*;\nuse std::{io, panic};\n" + declarations
    source += "verus! {\n#[derive(Debug)]\n#[non_exhaustive]\n" + pieces["StorageError"]
    source += "\npub type Result<T = (), E = StorageError> = core::result::Result<T, E>;\n"
    source += "pub type Checksum = u128;\nconst TRANSACTION_SIZE: usize = 128;\n"
    source += "pub struct TransactionId(pub u64);\n"
    for name in ("PageNumber", "BtreeHeader", "TransactionHeader", "DatabaseHeader", "UnrepairedDatabaseHeader"):
        declaration = pieces[name].replace("pub(crate)", "pub").replace("pub(super)", "pub")
        source += re.sub(r"(?m)^(    )([a-z_]+):", r"\1pub \2:", declaration) + "\n"
    return source


def render(application):
    pieces = extract(application)
    source = scaffold(pieces)
    source += (EVIDENCE / "contracts.verus.rs").read_text()
    source += "\nimpl UnrepairedDatabaseHeader {\n"
    source += "pub fn finalize_transaction_slots(self) -> (result: Result<DatabaseHeader>)\n"
    source += "    ensures gate_post(self, result),\n"
    source += selection.body(pieces["finalize_transaction_slots"]) + "\n}\n}\n"
    return source


def decoder_baseline(application):
    pieces = extract(application)
    header = (application / selection.HEADER).read_text()
    source = scaffold(pieces) + "use core::mem::size_of;\n"
    source += "#[derive(Debug)]\n#[non_exhaustive]\n" + item((application / selection.ERROR).read_text(), "pub enum DatabaseError") + "\n"
    source += "#[verifier::external]\n" + item((application / selection.ERROR).read_text(), "impl From<StorageError> for DatabaseError") + "\n"
    source += "const FILE_FORMAT_VERSION1: u8 = 1;\nconst FILE_FORMAT_VERSION2: u8 = 2;\nconst FILE_FORMAT_VERSION3: u8 = 3;\n"
    source += header[header.index("const VERSION_OFFSET:"):header.index("pub(crate) const PAGE_SIZE:")]
    source += "#[verifier::external]\n" + item(header, "fn get_u64(") + "\n"
    source += "#[verifier::external]\npub fn xxh3_checksum(data: &[u8]) -> Checksum { unimplemented!() }\n"
    for owner, relative, markers in (
        ("TransactionId", selection.TRACKER, ("    pub(crate) fn new(",)),
        ("PageNumber", selection.BASE, ("    pub(crate) const fn serialized_size(", "    pub(crate) fn from_le_bytes(")),
        ("BtreeHeader", selection.BTREE, ("    pub(crate) const fn serialized_size(", "    pub(crate) fn from_le_bytes(")),
    ):
        source += "#[verifier::external]\nimpl " + owner + " {\n"
        for marker in markers:
            source += item((application / relative).read_text(), marker).replace("pub(crate)", "pub") + "\n"
        source += "}\n"
    source += "impl TransactionHeader {\n" + pieces["from_bytes"].replace("pub(super)", "pub") + "\n}\n}\n"
    return source


def endian_boundary():
    return "use vstd::prelude::*;\nverus! {\npub type Checksum = u128;\npub fn endian_boundary(bytes: [u8; 16]) -> Checksum { Checksum::from_le_bytes(bytes) }\n}\n"


def native_harness(application):
    pieces = extract(application)
    source = "use std::{io, panic};\ntype Checksum = u128;\nconst TRANSACTION_SIZE: usize = 128;\n"
    source += "#[derive(Debug)]\n#[non_exhaustive]\n" + pieces["StorageError"] + "\n"
    source += "pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;\n"
    source += "pub struct TransactionId(pub u64);\n"
    for name in ("PageNumber", "BtreeHeader", "TransactionHeader", "DatabaseHeader", "UnrepairedDatabaseHeader"):
        declaration = pieces[name].replace("pub(crate)", "pub").replace("pub(super)", "pub")
        source += re.sub(r"(?m)^(    )([a-z_]+):", r"\1pub \2:", declaration) + "\n"
    source += "impl UnrepairedDatabaseHeader {\n" + pieces["finalize_transaction_slots"].replace("pub(super)", "pub") + "\n}\n"
    branches = selection.body(pieces["primary_verifies"])
    branches = branches[branches.index("        match Self::verify_primary_checksums(mem.clone())") : branches.rindex("}")]
    source += "fn normalize_observation(observation: Result<bool>) -> Result<bool> {\n"
    source += branches.replace("Self::verify_primary_checksums(mem.clone())", "observation") + "}\n"
    return source + (EVIDENCE / "native-controls.rs").read_text()


def controls(source):
    def change(old, new):
        check(source.count(old) == 1, "mutation_cardinality")
        return source.replace(old, new)
    return {
        "gate_condition": (change("if self.primary_corrupted {", "if !self.primary_corrupted {"), "proof"),
        "gate_message": (change('"Primary commit slot is corrupted".to_string()',
            '"Different native error".to_string()'), "proof"),
        "gate_frame": (change("Ok(self.inner)", "{ let mut header = self.inner; header.primary_slot = 0; Ok(header) }"), "proof"),
        "normalizer_true": (change("Ok(verified) => Ok(verified),", "Ok(verified) => Ok(!verified),"), "proof"),
        "normalizer_corrupt": (change("Err(StorageError::Corrupted(_)) => Ok(false),",
            "Err(StorageError::Corrupted(_)) => Ok(true),"), "proof"),
        "normalizer_swallow": (change("Err(err) => Err(err),", "Err(err) => Ok(false),"), "proof"),
        "false_representation": (source[:-2] + """
pub fn false_representation(header: UnrepairedDatabaseHeader) -> (result: Result<DatabaseHeader>)
    requires !header.primary_corrupted,
    ensures match result { Ok(value) => value.primary_slot < 2, _ => false },
{ header.finalize_transaction_slots() }
}
""", "proof"),
        "false_fallback": (source[:-2] + """
pub fn false_fallback(header: UnrepairedDatabaseHeader) -> (result: Result<DatabaseHeader>)
    requires header.primary_corrupted, !header.secondary_corrupted,
    ensures result.is_Ok(),
{ header.finalize_transaction_slots() }
}
""", "proof"),
        "opaque_io_behavior": (source[:-2] + "pub fn opaque_io_behavior(error: io::Error) { let _code = error.raw_os_error(); }\n}\n", "frontend"),
        "location_escape": (source[:-2] + "pub fn location_escape<'a>(location: &'a panic::Location<'a>) -> &'static panic::Location<'static> { location }\n}\n", "lifetime"),
        "no_cheating": (source[:-2] + "pub uninterp spec fn forbidden_axiom() -> bool;\n}\n", "cheating"),
    }


def projected_scope(report):
    return sorted([{key: row[key] for key in SCOPE_KEYS}
        for row in report["verus-infer"]["verification_scope"]["native_functions"]], key=lambda row: row["function"])


def validate_positive(report, code, source, scope):
    check(type(code) is int and code == 0, "exit_status")
    results = report["verification-results"]
    check(results.get("success") is True and results.get("encountered-error") is False
        and results.get("encountered-vir-error") is False
        and results.get("is-verifying-entire-crate") is True, "success_flags")
    check(type(results.get("verified")) is int and results["verified"] == 15
        and type(results.get("errors")) is int and results["errors"] == 0, "proof_counts")
    infer = report["verus-infer"]
    check(type(infer.get("schema_version")) is int and infer["schema_version"] == 1, "report_schema")
    check(infer.get("crate_success") is True and infer.get("status") == "no_inference"
        and infer.get("interface_written") is False, "publication_state")
    check(type(infer.get("requested_functions")) is int and infer["requested_functions"] == 0
        and type(infer.get("generated_functions")) is int and infer["generated_functions"] == 0
        and infer.get("functions") == [] and infer.get("imported_inference_dependencies") == [], "inference_scope")
    native = infer["verification_scope"]
    check(native.get("native_collection_completed") is True
        and native.get("complete_selected_scope_verified") is True, "scope_completion")
    check(type(native.get("checked_exec_bodies")) is int and native["checked_exec_bodies"] == 11
        and type(native.get("checked_proof_bodies")) is int and native["checked_proof_bodies"] == 3, "body_counts")
    check(projected_scope(report) == scope, "scope_identities")
    rows = native["native_functions"]
    check(all(type(row.get(key)) is bool for row in rows for key in
        ("automatically_derived", "selected", "inference_candidate", "ordinary_contract")), "scope_types")
    check(all(row["source"].startswith(str(source) + ":") for row in rows), "scope_sources")


def validate_bindings(row):
    for key in ("source", "stdout", "stderr"):
        check(digest(Path(row[key])) == row[key + "_sha256"], key + "_hash")
    check(read(row["stdout"]) == row["report"], "report_binding")
    command = row["command"]
    check(command[1] == row["source"], "argv_source")
    check(not any(argument == flag or argument.startswith(flag + "=") for argument in command
        for flag in ("--import", "--extern")), "argv_imports")
    check(not any(argument == "--export" or argument.startswith("--export=") for argument in command), "argv_export")
    check(not any(argument.startswith(("--no-verify", "--verify-", "--no-lifetime", "--no-auto-recommends", "--expand-errors"))
        for argument in command), "argv_scope")
    check(all(command.count(flag) == 1 for flag in
        ("--no-cheating", "--compile", "--output-json", "--num-threads", "--crate-name", "--sysroot", "-o", "-C")), "argv_flags")
    check(command[command.index("-o") + 1] == row["library"], "argv_library")
    check(command[command.index("--crate-name") + 1] == "observation", "argv_crate")
    check(command[command.index("--num-threads") + 1] == "1", "argv_threads")
    check(command[command.index("-C") + 1] == "debug-assertions=" + row["mode"], "argv_mode")
    check(row.get("no_cheating") is True and row.get("default_erasure") is True, "erasure_flags")
    expected = [row["verifier"], row["source"], "--sysroot", row["sysroot"], "--crate-type", "lib",
        "--crate-name", "observation", "--edition=2021", "--no-cheating", "--output-json", "--num-threads", "1",
        "--compile", "-o", row["library"], "-C", "debug-assertions=" + row["mode"]]
    check(command == expected, "argv_exact")
    path = Path(row["library"])
    if row["category"] == "positive":
        check(type(row.get("library_sha256")) is str
            and re.fullmatch(r"[0-9a-f]{64}", row["library_sha256"]) is not None
            and path.is_file() and path.stat().st_size > 0, "positive_library")
    check((path.is_file() and digest(path) == row["library_sha256"]) if row["library_sha256"] is not None
        else not path.exists(), "library_binding")


def validate_negative(row):
    check(type(row["returncode"]) is int and row["returncode"] != 0, "negative_exit")
    result = row["report"]["verification-results"]
    check(result.get("success") is False, "negative_success")
    check(row["report"]["verus-infer"].get("crate_success") is False
        and row["report"]["verus-infer"].get("interface_written") is False, "negative_publication")
    stderr = Path(row["stderr"]).read_text()
    markers = {"proof": "postcondition not satisfied", "frontend": "not supported",
        "lifetime": "lifetime may not live long enough", "cheating": "not allowed with --no-cheating",
        "decoder": "is not supported", "endian": "is not supported"}
    check(markers[row["category"]] in stderr, "negative_diagnostic")
    if row["category"] == "proof":
        check(type(result.get("errors")) is int and result["errors"] >= 1
            and result.get("encountered-vir-error") is False, "negative_proof")
    if row["category"] in ("decoder", "endian"):
        check("from_le_bytes" in stderr and "not supported" in stderr, "endian_boundary")
    expected = read(EVIDENCE / "outcomes.json")[row["label"].rsplit("_", 1)[0]]
    check(row["category"] == expected["category"] and row["returncode"] == expected["returncode"], "negative_classification")
    check(result == expected["results"] and all(type(result[key]) is type(value)
        for key, value in expected["results"].items()), "negative_counts")
    infer = row["report"]["verus-infer"]
    check(type(infer.get("schema_version")) is int and infer["schema_version"] == 1, "negative_schema")
    check(infer["status"] == expected["status"] and all(infer["verification_scope"][key] == value
        and type(infer["verification_scope"][key]) is type(value) for key, value in expected["scope"].items()), "negative_scope")
    check(row["library_sha256"] is None and not Path(row["library"]).exists(), "negative_library")


def validate_native_counts(stdout):
    check("test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;" in stdout,
        "native_test_counts")


def validate_native_controls(receipt, application):
    native = receipt["native_controls"]
    check(type(native.get("tests")) is int and native["tests"] == 3
        and type(native.get("gate_inputs")) is int and native["gate_inputs"] == 64, "native_metadata")
    source = Path(native["source"])
    check(source.is_file() and native.get("source_sha256") == digest(source)
        and source.read_text() == native_harness(application), "native_source_binding")
    binary = source.parent / "native-controls"
    check(binary.is_file() and binary.stat().st_size > 0
        and native.get("binary_sha256") == digest(binary), "native_binary_binding")
    compiler = Path(receipt["cases"][0]["sysroot"]) / "bin/rustc"
    compile_command, test_command = receipt["commands"][-2:]
    check(compile_command["argv"] == [str(compiler), str(source), "--edition=2021", "--test", "-o", str(binary)]
        and test_command["argv"] == [str(binary)]
        and all(command["cwd"] == str(source.parent) for command in (compile_command, test_command)),
        "native_argv")
    check(all(type(command.get("exit_code")) is int and command["exit_code"] == 0
        for command in (compile_command, test_command)), "native_exit_status")
    names = ("native_compile.stdout", "native_compile.stderr", "native_tests.stdout", "native_tests.stderr")
    logs = native.get("logs", {})
    check(type(logs) is dict and set(logs) == set(names), "native_log_fields")
    check(all((source.parent / name).is_file() and logs.get(name) == digest(source.parent / name)
        for name in names), "native_log_binding")
    validate_native_counts((source.parent / "native_tests.stdout").read_text())


def validate_matrix(receipt, application):
    check(receipt.get("success") is True, "matrix_success")
    check(type(receipt.get("identity")) is list and receipt["identity"] == [78, 72, 38]
        and all(type(value) is int for value in receipt["identity"])
        and receipt.get("whole_recovery_proved") is False and receipt.get("durability_proved") is False
        and receipt.get("exports") == "none", "matrix_metadata")
    source = render(application)
    variants = {"positive": (source, "positive"), **controls(source),
        "decoder_baseline": (decoder_baseline(application), "decoder"), "endian_boundary": (endian_boundary(), "endian")}
    expected = {label + "_" + mode: (contents, category, mode)
        for mode in ("yes", "no") for label, (contents, category) in variants.items()}
    rows = receipt["cases"]
    check(len(rows) == len(expected) and len({row["label"] for row in rows}) == len(expected)
        and {row["label"] for row in rows} == set(expected), "matrix_labels")
    commands = receipt["commands"]
    check(len(commands) == len(rows) + 2 and [command["label"] for command in commands]
        == [row["label"] for row in rows] + ["native_compile", "native_tests"]
        and all(command["argv"] == row["command"] and type(command["exit_code"]) is int
            and command["exit_code"] == row["returncode"] for command, row in zip(commands, rows)), "matrix_commands")
    for row in rows:
        contents, category, mode = expected[row["label"]]
        check(row["category"] == category and row["mode"] == mode, "matrix_classification")
        check(row["source_sha256"] == hashlib.sha256(contents.encode()).hexdigest(), "matrix_source")
        validate_bindings(row)
        if category == "positive":
            validate_positive(row["report"], row["returncode"], Path(row["source"]), read(EVIDENCE / "scope.json"))
        else:
            validate_negative(row)
    validate_native_controls(receipt, application)


def source_fidelity(application, source):
    pieces = extract(application)
    actual = selection.body(item(source, "pub fn finalize_transaction_slots("))
    check(actual == selection.body(pieces["finalize_transaction_slots"]), "native_gate_body")
    native = selection.body(pieces["primary_verifies"])
    arms = native[native.index("            Ok(verified)"):native.rindex("        }")]
    auxiliary = selection.body(item(source, "pub fn normalize_observation("))
    auxiliary = auxiliary[auxiliary.index("        Ok(verified)"):auxiliary.rindex("    }")]
    check([line.strip() for line in arms.splitlines()] == [line.strip() for line in auxiliary.splitlines()], "native_match_arms")
    result = {name: {"sha256": hashlib.sha256(pieces[name].encode()).hexdigest(),
        "body_sha256": hashlib.sha256(selection.body(pieces[name]).encode()).hexdigest(),
        "native_line": (application / (DB if name == "primary_verifies" else selection.HEADER)).read_text().splitlines().index(pieces[name].splitlines()[0]) + 1}
        for name in ("finalize_transaction_slots", "primary_verifies", "from_bytes")}
    result["records"] = {}
    for name in ("PageNumber", "BtreeHeader", "TransactionHeader", "DatabaseHeader", "UnrepairedDatabaseHeader"):
        native = pieces[name].replace("pub(crate)", "pub").replace("pub(super)", "pub")
        adapted = re.sub(r"(?m)^(    )([a-z_]+):", r"\1pub \2:", native)
        check(adapted in source, "record_fidelity")
        result["records"][name] = {"native_sha256": hashlib.sha256(pieces[name].encode()).hexdigest(),
            "adapted_sha256": hashlib.sha256(adapted.encode()).hexdigest()}
    check("#[derive(Debug)]\n#[non_exhaustive]\n" + pieces["StorageError"] in source, "full_error")
    decoder = decoder_baseline(application)
    check(selection.body(item(decoder, "    pub fn from_bytes(")) == selection.body(pieces["from_bytes"]), "decoder_body")
    result["normalization_arms_sha256"] = hashlib.sha256("\n".join(line.strip() for line in arms.splitlines()).encode()).hexdigest()
    return result


def replay(args):
    output = args.output.resolve()
    check(not output.exists() and not output.is_relative_to(ROOT), "fresh_output")
    output.mkdir()
    verifier, solver, application = args.verus.resolve(), args.solver.resolve(), args.application.resolve()
    receipt = {"success": False, "commands": [], "cases": [], "identity": [78, 72, 38],
        "python_optimized": not __debug__, "whole_recovery_proved": False, "durability_proved": False,
        "inference": "none; all semantic contracts and proofs authored", "exports": "none", "application": str(application)}
    with replay_receipt(output, receipt):
        sysroot = opaque.sysroot_for()
        environment = opaque.environment_for(output, solver, sysroot)
        receipt["native_before"] = opaque.native_state(application, environment)
        baseline = audit_ledger(args.snapshot, "file-hashes.json", ROOT)
        runtime = audit_ledger(args.snapshot, "runtime-hashes.json", verifier.parent)
        paths = [ROOT / relative for relative in baseline]
        paths += [verifier.parent / relative for relative in runtime]
        paths += [application / relative for relative in receipt["native_before"]["sources"]]
        paths += [path for path in EVIDENCE.rglob("*") if path.is_file()]
        paths += [Path(__file__), Path(__file__).with_name("test_redb_repair_observation80i.py")]
        toolchain = audit_ledger(args.snapshot, "toolchain-hashes.json", Path("/"))
        paths += [Path("/") / relative for relative in toolchain]
        guard = ArtifactGuard(paths)
        write(output / "guarded-inputs-before.json", guard.initial)
        try:
            source = render(application)
            receipt["source_fidelity"] = source_fidelity(application, source)
            fixture = read(EVIDENCE / "scope.json")
            runner = LoggedCommands(output, output, receipt["commands"], timeout=60)
            variants = {"positive": (source, "positive"), **controls(source),
                "decoder_baseline": (decoder_baseline(application), "decoder"),
                "endian_boundary": (endian_boundary(), "endian")}
            for mode in ("yes", "no"):
                for label, (contents, category) in variants.items():
                    name = label + "_" + mode
                    directory = output / name
                    directory.mkdir()
                    path = directory / "source.rs"
                    library = directory / "libobservation.rlib"
                    path.write_text(contents)
                    command = [str(verifier), str(path), "--sysroot", str(sysroot), "--crate-type", "lib",
                        "--crate-name", "observation", "--edition=2021", "--no-cheating", "--output-json", "--num-threads", "1",
                        "--compile", "-o", str(library), "-C", "debug-assertions=" + mode]
                    env = dict(environment, VERUS_INFER_CACHE=str(directory / "cache"))
                    code = runner(name, command, env)
                    row = {"label": name, "mode": mode, "category": category, "command": command,
                        "verifier": str(verifier), "sysroot": str(sysroot), "returncode": code,
                        "source": str(path), "stdout": str(output / (name + ".stdout")),
                        "stderr": str(output / (name + ".stderr")), "library": str(library),
                        "library_sha256": digest(library) if library.is_file() else None,
                        "no_cheating": True, "default_erasure": True}
                    for key in ("source", "stdout", "stderr"):
                        row[key + "_sha256"] = digest(row[key])
                    row["report"] = read(row["stdout"])
                    receipt["cases"].append(row)
                    validate_bindings(row)
                    if category == "positive":
                        check(contents == render(application), "positive_source")
                        validate_positive(row["report"], code, path, fixture)
                    else:
                        validate_negative(row)
            native_source = output / "native-controls.rs"
            native_source.write_text(native_harness(application))
            native_binary = output / "native-controls"
            for label, command in (
                ("native_compile", [str(sysroot / "bin/rustc"), str(native_source), "--edition=2021", "--test", "-o", str(native_binary)]),
                ("native_tests", [str(native_binary)]),
            ):
                check(runner(label, command, environment) == 0, "native_test_command")
            validate_native_counts((output / "native_tests.stdout").read_text())
            receipt["native_controls"] = {"source": str(native_source), "source_sha256": digest(native_source),
                "binary_sha256": digest(native_binary), "tests": 3, "gate_inputs": 64,
                "logs": {name: digest(output / name) for name in
                    ("native_compile.stdout", "native_compile.stderr", "native_tests.stdout", "native_tests.stderr")}}
        finally:
            guard.check()
            write(output / "guarded-inputs-after.json", guard.snapshot())
            receipt["native_after"] = opaque.native_state(application, environment)
            check(receipt["native_before"] == receipt["native_after"], "native_unchanged")
            receipt["guarded_inputs"] = len(guard.paths)
            audit_ledger(args.snapshot, "file-hashes.json", ROOT)
            audit_ledger(args.snapshot, "runtime-hashes.json", verifier.parent)
            audit_ledger(args.snapshot, "toolchain-hashes.json", Path("/"))
        validate_matrix(dict(receipt, success=True), application)
    print(json.dumps({"output": str(output), "cases": len(receipt["cases"]), "success": receipt["success"]}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--snapshot", type=Path, required=True)
    args = parser.parse_args()
    replay(args)


if __name__ == "__main__":
    main()
