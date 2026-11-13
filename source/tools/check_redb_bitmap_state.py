"""Prove pinned original bitmap get/set/clear against an authored explicit view."""
import argparse
import hashlib
import json
import os
from pathlib import Path

from check_fedimint_fee_encoding import certificate_import, check_client_import, compiler_artifacts
from check_redb_bitmap_range import METADATA_DIAGNOSTICS, failure_diagnostic, import_guard
from check_redb_page_size import APPLICATION, item
from infer_transfer_identity import CACHE_VERSION, ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA, require_no_foreign_dependencies
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import (
    LoggedCommands, positive_timeout, replay_receipt, require, require_exact,
    require_publication, snapshot_inputs, validate_run,
)


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "source/tools/fixtures/transfers/transfer-redb-bitmap-state77"
BITMAP = "src/tree_store/page_store/bitmap.rs"
SOURCE_HASH = "d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff"
ITEM_HASHES = {
    "U64GroupedBitmap": "0c87301dce18a42505a105c8bfcf676c6882403c37b7f6738a5bfe0294690105",
    "data_index_of": "45395cb73a6c7f5252ca1552a4159ea5367fdb40694a06fcf5a982c71896be1d",
    "select_mask": "1bdfc2ea01eec85ab14fce238c30a7cea534c06f7593a75917a7870b1c184d3a",
    "get": "277a149cb920efe8f82ba1c28e25073843571c7464dbfd2560216fb755ff2b93",
    "set": "ae5e8143d3763f71a0440150370a8e578998db6ae884fd0743ce039624ab2d6a",
    "clear": "ba20a17b9dea935783cd8c5315f9bd0584c4e768fb4c92d4b150c7fa21ec644b",
}
SIGNATURES = {
    "data_index_of": "    fn data_index_of(",
    "select_mask": "    fn select_mask(",
    "get": "    pub fn get(",
    "set": "    pub fn set(",
    "clear": "    pub fn clear(",
}
HEADER = "use vstd::prelude::*;\nverus! {\n"
IDENTITY = list(ENGINE_IDENTITY)


def environment_for(output, solver):
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith("GIT_") and key not in (
                       "RUSTFLAGS", "RUSTDOCFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")}
    environment.update(GIT_OPTIONAL_LOCKS="0", PYTHONDONTWRITEBYTECODE="1",
                       CARGO_HOME=str(output / "cargo-home"),
                       CARGO_TARGET_DIR=str(output / "cargo-target"), CARGO_BUILD_JOBS="1",
                       VERUS_Z3_PATH=str(solver.resolve()),
                       VERUS_INFER_CACHE=str(output / "candidate-cache"))
    return environment


def body(executable):
    return executable[executable.index("{"):]


def extract(application, output):
    source = application / BITMAP
    require(digest(source) == SOURCE_HASH, "pinned bitmap.rs changed")
    text = source.read_text()
    implementation = text[text.index("impl U64GroupedBitmap {"):]
    pieces = {"U64GroupedBitmap": item(text, "pub(crate) struct U64GroupedBitmap") + "\n"}
    pieces.update({name: item(implementation, marker) + "\n"
                   for name, marker in SIGNATURES.items()})
    provenance = {}
    for name, piece in pieces.items():
        require(hashlib.sha256(piece.encode()).hexdigest() == ITEM_HASHES[name],
                "pinned executable extraction changed: " + name)
        require((EVIDENCE / (name + ".source.rs")).read_text() == piece,
                "checked-in extraction changed: " + name)
        (output / (name + ".original.rs")).write_text(piece)
        provenance[name] = dict(sha256=ITEM_HASHES[name], source=BITMAP,
                               body_sha256=hashlib.sha256(body(piece).encode()).hexdigest(),
                               line=text[:text.index(piece.rstrip())].count("\n") + 1)
    (output / "bitmap.original.rs").write_bytes(source.read_bytes())
    return pieces, dict(application_commit=APPLICATION, source=BITMAP,
                        source_sha256=SOURCE_HASH, items=provenance,
                        adaptations=["Expose struct/fields and scalar helpers for proof/import clients",
                                     "Insert authored checked contracts, named return binders, and view",
                                     "Infer only select_mask; data_index_of tuple contract is author-written",
                                     "Two authored generic word lemmas are checked by bit_vector"],
                        unchanged="All five method bodies, including formatted clear assertion, are byte-identical; struct visibility alone is adapted")


def provider_source(pieces):
    declaration = pieces["U64GroupedBitmap"].replace("pub(crate) struct", "pub struct")
    declaration = declaration.replace("    len:", "    pub len:").replace("    data:", "    pub data:")
    contracts = json.loads((EVIDENCE / "contracts.json").read_text())
    methods = []
    for name in SIGNATURES:
        if name == "select_mask":
            rendered = "    #[verifier::infer]\n" + pieces[name].replace("    fn ", "    pub fn ", 1)
        else:
            rendered = contracts[name] + body(pieces[name])
        require(rendered.endswith(body(pieces[name])), "executable body adaptation: " + name)
        methods.append(rendered)
    return (HEADER + declaration + "impl U64GroupedBitmap {\n"
            + (EVIDENCE / "abstraction.verus.rs").read_text() + "\n" + "\n".join(methods)
            + "}\n" + (EVIDENCE / "word_facts.verus.rs").read_text() + "}\n")


def client_source(imported=False):
    prefix = "use provider::U64GroupedBitmap;\n" if imported else ""
    relay = ("#[verifier::infer]\npub fn relay(bit:usize)->u64 "
             "{U64GroupedBitmap::select_mask(bit)}\n"
             "fn mask_client(bit:u32)->(result:u64) requires bit<64, "
             "ensures result == (1u64 << bit), {relay(bit as usize)}\n") if imported else ""
    return prefix + HEADER + relay + (EVIDENCE / "clients.verus.rs").read_text() + "}\n"


def native_source(pieces):
    return (pieces["U64GroupedBitmap"] + "impl U64GroupedBitmap {\n"
            + "\n".join(pieces[name] for name in SIGNATURES) + "}\n"
            + (EVIDENCE / "native_tests.rs").read_text())


def classify(data, stderr, code):
    if code == 0:
        return "proved"
    lowered = stderr.lower()
    if any(word in lowered for word in ("resource limit", "timed out", "panicked at")):
        return "resource_or_runtime_failure"
    if "E0499" in stderr or "cannot borrow" in stderr and "more than once" in stderr:
        return "aliasing_frontend_rejection"
    report = data.get("verus-infer", {})
    if report.get("status") == "invalid_import" or any(
            message in stderr for message in METADATA_DIAGNOSTICS.values()):
        return "import_rejection"
    if report.get("status") == "unsupported" or data["verification-results"].get("encountered-vir-error"):
        return "frontend_rejection"
    if "bitvector assertion not satisfied" in stderr:
        return "authored_lemma_failure"
    if any(word in stderr for word in ("precondition not satisfied", "possible bit shift", "possible arithmetic")):
        return "safety_failure"
    if "postcondition not satisfied" in stderr:
        return "semantic_goal_failure"
    return "unclassified_failure"


def positive_scope(crate_name):
    methods = {"U64GroupedBitmap::" + name for name in SIGNATURES}
    lemmas = {"word_facts::set_bit_frame", "word_facts::clear_bit_frame"}
    clients = {"observe_clear", "observe_set", "padding_flag", "spare_words"}
    if crate_name == "provider":
        return methods, lemmas, "U64GroupedBitmap::select_mask", 23
    if crate_name in ("local_debug", "local_release"):
        return methods | clients, lemmas, "U64GroupedBitmap::select_mask", 29
    require(crate_name in ("client_debug", "client_release"), "unexpected positive crate identity")
    return clients | {"relay", "mask_client"}, set(), "relay", 24


def check_positive_scope(data, crate_name):
    results, inference = data["verification-results"], data["verus-infer"]
    require(results.get("success") is True and inference.get("crate_success") is True
            and inference.get("interface_written") is True, "non-boolean positive publication state")
    executions, proofs, inferred_name, verified = positive_scope(crate_name)
    scope = inference.get("verification_scope", {})
    require(scope.get("native_collection_completed") is True
            and scope.get("complete_selected_scope_verified") is True,
            "positive native scope incomplete")
    require(type(results.get("verified")) is int and results["verified"] == verified
            and type(results.get("errors")) is int and results["errors"] == 0
            and type(scope.get("checked_exec_bodies")) is int
            and scope["checked_exec_bodies"] == len(executions)
            and type(scope.get("checked_proof_bodies")) is int
            and scope["checked_proof_bodies"] == len(proofs), "positive native proof counts changed")
    native = scope.get("native_functions")
    require(isinstance(native, list) and all(isinstance(row, dict) for row in native),
            "missing checked native body scope")
    checked = [row for row in native if row.get("body_status") in
               ("checked_exec_body", "checked_proof_body")]
    expected = {crate_name + "::" + name: "checked_exec_body" for name in executions}
    expected.update({crate_name + "::" + name: "checked_proof_body" for name in proofs})
    require(len(checked) == len(expected)
            and {row.get("function"): row.get("body_status") for row in checked} == expected,
            "checked native body identities changed")
    full_inferred = crate_name + "::" + inferred_name
    require(all(row.get("selected") is True and row.get("automatically_derived") is False
                and row.get("inference_candidate") is (row["function"] == full_inferred)
                and row.get("ordinary_contract") is (row["function"] != full_inferred)
                for row in checked), "checked native body flags changed")
    functions = inference["functions"]
    require(len(functions) == 1 and functions[0].get("function") == full_inferred
            and functions[0].get("exported") is True
            and functions[0].get("required_checks_passed") is True
            and functions[0].get("contract") == dict(requires="((bit as u64) <= 63)",
                ensures="result == ((1 as u64) << (bit as u64))"),
            "inferred helper identity or contract changed")
    work = functions[0].get("metrics", {}).get("solver_micros")
    require(type(work) is int and work > 0, "required inference checks did no solver work")


def check_case(data, stderr, code, expected, artifact, inferred, diagnostic=None, *, crate_name="provider"):
    observed = classify(data, stderr, code)
    require(observed == expected, "classification: " + observed + " expected " + expected + "\n" + stderr[-3000:])
    failure_diagnostic(expected, stderr, diagnostic)
    if expected != "resource_or_runtime_failure":
        validate_run(dict(returncode=code, stdout=json.dumps(data), stderr=stderr), expected == "proved",
                     allow_frontend=expected in ("frontend_rejection", "aliasing_frontend_rejection", "import_rejection"))
    require_publication(data, expected == "proved", artifact)
    if expected == "proved":
        check_positive_scope(data, crate_name)
        functions = data["verus-infer"]["functions"]
        require(len(functions) == inferred, "unexpected inference scope")
        require_exact(functions)
        require(all(function["metrics"]["solver_micros"] > 0 for function in functions),
                "required inference checks did no solver work")
        require(data["verification-results"].get("is-verifying-entire-crate") is True,
                "partial crate verification")
    if expected == "semantic_goal_failure":
        require("precondition not satisfied" not in stderr and "bitvector assertion not satisfied" not in stderr,
                "semantic control failed for another reason")
    return observed


def metadata(seed):
    footer = seed.rindex(b'{"schema":')
    require(seed.endswith(b"VRINFER1"), "missing inference trailer")
    return footer, json.loads(seed[footer:-8])


def fresh_artifacts(artifact, library, seed=None):
    require(artifact.resolve() != library.resolve(), "export and library paths must differ")
    require(not any(path.exists() or path.is_symlink() for path in (artifact, library)),
            "stale positive/negative outputs before seeding")
    if seed is not None:
        artifact.write_bytes(seed)


def provider_certificate(seed):
    footer, certificate = metadata(seed)
    require(type(certificate.get("schema")) is int and certificate["schema"] == INTERFACE_SCHEMA and certificate["policy"] == INTERFACE_POLICY,
            "unexpected certificate identity")
    require(all(certificate[key] is True for key in ("soundness", "maximality", "exactness")),
            "incomplete provider certificate")
    require(len(certificate["proofs"]) == 1 and certificate["trusted_models"] == [],
            "unexpected provider inference/model scope")
    require(all(certificate["proofs"][0][1][key] is True
                for key in ("soundness", "maximality", "exactness")), "incomplete exported proof")
    fingerprint = hashlib.sha256(seed[footer:-8]).hexdigest()
    require_no_foreign_dependencies(certificate)
    return certificate, certificate_import(certificate, fingerprint)


def corrupt_certificate(certificate, fault):
    require(fault in METADATA_DIAGNOSTICS, "unknown import fault")
    changed = json.loads(json.dumps(certificate))
    if fault == "stale_policy":
        changed["policy"] = "candidate-bound-evidence-v68"
    elif fault == "false_exactness":
        changed["exactness"] = False
    elif fault == "wrong_summary":
        changed["summaries"] = "corrupt"
    else:
        changed["trusted_models"] = [[changed["proofs"][0][0], "corrupt"]]
    return changed


def expected_cases():
    expected = {}
    for mode in ("debug", "release"):
        for kind in ("provider", "import"):
            for phase in ("cold", "warm"):
                expected[kind + "_" + mode + "_" + phase] = "proved"
        expected["local_" + mode] = "proved"
        for name in ("get_negated", "set_ands", "clear_sets", "set_changes_length", "set_corrupts_other_word"):
            expected[name + "_" + mode] = "semantic_goal_failure"
        for operation in ("get", "set", "clear"):
            for name in ("invalid_bit", "invalid_index", "short_data", "empty"):
                expected[operation + "_" + name + "_" + mode] = "safety_failure"
    expected.update(false_whole_logical_flag="semantic_goal_failure", missing_word_facts="semantic_goal_failure",
                    missing_representation="safety_failure", invalid_shift="safety_failure",
                    alias_near_miss="aliasing_frontend_rejection", solver_budget="resource_or_runtime_failure")
    expected.update({name: "import_rejection" for name in METADATA_DIAGNOSTICS})
    return expected


def check_completion(cases):
    expected = expected_cases()
    require(len(cases) == len(expected) and {case["label"] for case in cases} == expected.keys(),
            "missing or duplicate replay cases")
    for case in cases:
        success = expected[case["label"]] == "proved"
        require(case["classification"] == case["expected"] == expected[case["label"]],
                "case classification drift")
        require(case["artifact_exists"] is success and case["stale_export_seeded"] is not success,
                "publication/seed coverage mismatch")
        fingerprint = case["export_sha256"] if success else case["seeded_export_sha256"]
        require(isinstance(fingerprint, str) and len(fingerprint) == 64, "missing publication/seed hash")
        require(success or case["export_sha256"] is None, "failed case retained stale export")


def run_suite(args):
    output = args.output.resolve()
    for immutable in (ROOT, args.application.resolve(), args.verus.resolve().parent):
        require(output != immutable and immutable not in output.parents,
                "output must be outside source/application/runtime trees")
    output.mkdir(parents=True, exist_ok=False)
    environment = environment_for(output, args.solver)
    receipt = dict(status="blocked", commands=[], cases=[], identity=IDENTITY,
                   engine_changes=0, authored_bitvector_lemmas=2,
                   trust=["Rust/Verus frontend and backend, solver, existing Vec/native specifications",
                          "Author-owned view/contracts and two checked lemmas; no new trusted summaries or axioms",
                          "Exact source extraction is hash-checked, not a compiler-correspondence proof"],
                   boundaries=["No constructor/deserialization validity, resize, allocator, storage or durability proof",
                               "Exported high-level methods are ordinarily checked authored contracts, not inferred contracts",
                               "Only the scalar select_mask contract is inferred; tuple helper contract is ordinarily checked"])
    commands = LoggedCommands(output, ROOT, receipt["commands"], timeout=args.timeout)
    with replay_receipt(output, receipt):
        for label, argv, expected in [("pin", ["rev-parse", "HEAD"], APPLICATION),
                                      ("clean", ["status", "--porcelain", "--untracked-files=all"], "")]:
            require(commands("application_before_" + label, ["git", "-C", str(args.application), *argv], environment) == 0)
            require((output / ("application_before_" + label + ".stdout")).read_text().strip() == expected,
                    "native application pin/state changed")
        pieces, receipt["extraction"] = extract(args.application, output)
        require(commands("engine_files", ["git", "ls-files", "--cached", "--others", "--exclude-standard",
                                           "--", "source", "dependencies", "rust-toolchain.toml"], environment) == 0)
        inputs = sorted(set((output / "engine_files.stdout").read_text().splitlines()))
        receipt.update(snapshot_inputs(ROOT, output / "inputs", inputs))
        compiler = args.compiler.resolve()
        paths = [ROOT / relative for relative in inputs]
        paths += [entry for entry in EVIDENCE.iterdir() if entry.is_file()]
        paths += [entry for entry in args.verus.resolve().parent.iterdir() if entry.is_file()]
        paths += [args.solver.resolve(), *compiler_artifacts(compiler)]
        require(commands("application_files", ["git", "-C", str(args.application), "ls-files"], environment) == 0)
        paths += [args.application / relative for relative in (output / "application_files.stdout").read_text().splitlines()]
        guard = ArtifactGuard(paths)
        receipt["guarded_artifacts"] = guard.initial
        commands.guard = guard
        source = provider_source(pieces)
        provider_guard = None

        def verify(label, text, debug=True, expected="proved", source_label=None, dependency=None,
                   compile=False, seed=None, inferred=1, diagnostic=None, extra=()):
            source_label = source_label or label
            path = output / (source_label + ".rs")
            if path.exists():
                require(path.read_text() == text, "source replay changed")
            else:
                path.write_text(text)
            artifact = output / (source_label + ".vir")
            library = output / ("lib" + source_label + ".rlib")
            fresh_artifacts(artifact, library, seed)
            dependency_guard = import_guard(artifact, dependency) if dependency is not None else None
            command = [str(args.verus.resolve()), str(path), "--crate-type=lib", "--crate-name", source_label,
                       "--output-json", "--no-cheating", "--num-threads", "1", "--infer-goals",
                       "--export", str(artifact), "-o", str(library),
                       "-C", "debug-assertions=" + ("yes" if debug else "no")]
            if compile:
                command.append("--compile")
            if dependency is not None:
                command += ["--extern", "provider=" + str(output / "libprovider.rlib"),
                            "--import", "provider=" + str(dependency)]
            command += list(extra)
            if provider_guard is not None:
                provider_guard.check()
            if dependency_guard is not None:
                dependency_guard.check()
            code = commands(label, command, environment)
            if provider_guard is not None:
                provider_guard.check()
            if dependency_guard is not None:
                dependency_guard.check()
            data = json.loads((output / (label + ".stdout")).read_text())
            stderr = (output / (label + ".stderr")).read_text()
            observed = classify(data, stderr, code)
            receipt["cases"].append(dict(label=label, classification=observed, expected=expected,
                                         report=data, source_sha256=digest(path), debug_assertions=debug,
                                         stale_export_seeded=seed is not None, artifact_exists=artifact.exists(),
                                         seeded_export_sha256=hashlib.sha256(seed).hexdigest() if seed is not None else None,
                                         export_sha256=digest(artifact) if artifact.exists() else None))
            check_case(data, stderr, code, expected, artifact, inferred, diagnostic, crate_name=source_label)
            return artifact, library, data

        def archive(label, artifact, library):
            directory = output / "archive"
            directory.mkdir(exist_ok=True)
            for path in (artifact, library):
                if path.exists():
                    destination = directory / (label + path.suffix)
                    require(not destination.exists(), "archive would overwrite evidence")
                    path.rename(destination)

        seeds = {}
        for debug in (True, False):
            mode = "debug" if debug else "release"
            environment["VERUS_INFER_CACHE"] = str(output / ("cache_" + mode))
            phases = []
            for phase in ("cold", "warm"):
                label = "provider_" + mode + "_" + phase
                artifact, library, data = verify(label, source, debug, source_label="provider", compile=True)
                functions = data["verus-infer"]["functions"]
                require(functions[0]["metrics"]["cache"] == ("miss" if phase == "cold" else "hit"),
                        "expected cold miss / warm hit")
                certificate, expected_import = provider_certificate(artifact.read_bytes())
                seeds[mode] = artifact.read_bytes()
                phases.append(functions[0])
                if phase == "cold":
                    archive(label, artifact, library)
                else:
                    provider_guard = ArtifactGuard([artifact, library])
            require(phases[0]["contract"] == phases[1]["contract"],
                    "cold/warm contracts changed")
            cache_entries = list((output / ("cache_" + mode)).glob("*.json"))
            require(cache_entries, "missing candidate cache")
            receipt["observed_cache_" + mode] = [json.loads(entry.read_text()) for entry in cache_entries]
            require(all(entry["version"] == CACHE_VERSION
                        for entry in receipt["observed_cache_" + mode]), "unexpected cache identity")
            verify("local_" + mode, source + client_source(), debug, compile=True)
            for phase in ("cold", "warm"):
                artifact, library, data = verify("import_" + mode + "_" + phase, client_source(True), debug,
                                                source_label="client_" + mode, dependency=output / "provider.vir", compile=True)
                require(data["verus-infer"]["functions"][0]["metrics"]["cache"] == ("miss" if phase == "cold" else "hit"),
                        "import cache phase mismatch")
                check_client_import(data, expected_import)
                archive("import_" + mode + "_" + phase, artifact, library)
            native = output / ("native_" + mode + ".rs")
            native.write_text(native_source(pieces))
            binary = output / ("native_" + mode)
            require(commands("native_compile_" + mode, [str(compiler), "--edition=2021", "--test", "-C", "opt-level=2",
                             "-C", "debug-assertions=" + ("yes" if debug else "no"), str(native), "-o", str(binary)], environment) == 0,
                    "native compilation failed")
            require(commands("native_test_" + mode, [str(binary), "--test-threads=1", "--nocapture"], environment) == 0,
                    "native tests failed")
            if debug:
                archive("provider_debug_warm", output / "provider.vir", output / "libprovider.rlib")
                provider_guard = None

        seed = seeds["release"]
        mutations = {
            "get_negated": ("group & U64GroupedBitmap::select_mask(bit_index) != 0", "group & U64GroupedBitmap::select_mask(bit_index) == 0"),
            "set_ands": ("group |= Self::select_mask(bit_index);", "group &= Self::select_mask(bit_index);"),
            "clear_sets": ("self.data[index] &= !Self::select_mask(bit_index);", "self.data[index] |= Self::select_mask(bit_index);"),
            "set_changes_length": ("self.data[index] = group;", "self.data[index] = group;\n        self.len = 0;"),
            "set_corrupts_other_word": ("self.data[index] = group;", "self.data[index] = group;\n        self.data[0] = 0;"),
        }
        for debug in (True, False):
            mode = "debug" if debug else "release"
            for name, (before, after) in mutations.items():
                require(source.count(before) == 1, "ambiguous semantic mutation")
                verify(name + "_" + mode, source.replace(before, after), debug,
                       "semantic_goal_failure", seed=seed)
            for operation in ("get", "set", "clear"):
                for name, length, words, bit in [("invalid_bit", 1, 1, 1), ("invalid_index", 1, 1, 128),
                                                 ("short_data", 65, 1, 64), ("empty", 0, 0, 0)]:
                    initialize = "let mut data=Vec::new();" + "data.push(0u64);" * words
                    negative = (HEADER + "use provider::U64GroupedBitmap; fn bad(){" + initialize
                                + f"let mut bitmap=U64GroupedBitmap{{len:{length},data}}; bitmap.{operation}({bit});" + "}}\n")
                    verify(operation + "_" + name + "_" + mode, negative, debug, "safety_failure",
                           dependency=output / "provider.vir", inferred=0, seed=seed)
        wrong_flag = source.replace("result == (final(self).data@[bit as int / 64] == u64::MAX)",
                                    "result == (forall|query:int| 0 <= query < final(self).len ==> final(self).live(query))")
        require(wrong_flag != source, "flag mutation missing")
        verify("false_whole_logical_flag", wrong_flag, expected="semantic_goal_failure", seed=seed)
        verify("missing_word_facts", source.replace((EVIDENCE / "word_facts.verus.rs").read_text(), ""),
               expected="semantic_goal_failure", seed=seed)
        verify("missing_representation", source.replace("self.len as int <= 64 * self.data@.len()", "true"),
               expected="safety_failure", seed=seed)
        negative = (HEADER + "use provider::U64GroupedBitmap; fn bad(){U64GroupedBitmap::select_mask(64);}}\n")
        verify("invalid_shift", negative, expected="safety_failure", dependency=output / "provider.vir", inferred=0, seed=seed)
        alias = (HEADER + "use provider::U64GroupedBitmap; fn two(first:&mut U64GroupedBitmap,second:&mut U64GroupedBitmap){}"
                 "fn alias(bitmap:&mut U64GroupedBitmap){two(bitmap,bitmap);}}\n")
        verify("alias_near_miss", alias, expected="aliasing_frontend_rejection", dependency=output / "provider.vir",
               inferred=0, seed=seed, diagnostic="cannot borrow")
        verify("solver_budget", source, expected="resource_or_runtime_failure", seed=seed,
               extra=("--rlimit", "0.001"))
        footer, certificate = metadata(seed)
        for fault in METADATA_DIAGNOSTICS:
            changed = corrupt_certificate(certificate, fault)
            dependency = output / ("dependency_" + fault + ".vir")
            dependency.write_bytes(seed[:footer] + json.dumps(changed, separators=(",", ":")).encode() + b"VRINFER1")
            verify(fault, client_source(True), expected="import_rejection", dependency=dependency, inferred=0,
                   seed=seed, diagnostic=METADATA_DIAGNOSTICS[fault])
        for label, argv, expected in [("pin", ["rev-parse", "HEAD"], APPLICATION),
                                      ("clean", ["status", "--porcelain", "--untracked-files=all"], "")]:
            require(commands("application_after_" + label, ["git", "-C", str(args.application), *argv], environment) == 0)
            require((output / ("application_after_" + label + ".stdout")).read_text().strip() == expected,
                    "native application changed")
        guard.check()
        check_completion(receipt["cases"])
        receipt["status"] = "proved"
    print(json.dumps(dict(receipt=str(output / "receipt.json"), status=receipt["status"])))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application", type=Path, required=True)
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    run_suite(parser.parse_args())


if __name__ == "__main__":
    main()
