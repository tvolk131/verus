"""Independent acceptance of the relational-page and complete-request evidence."""

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

from infer_transfer_identity import INTERFACE_POLICY, INTERFACE_SCHEMA
import check_fedimint_full_request as full
import check_redb_page_relations as pages
from infer_eval_support import digest, tool_artifacts
from infer_experiment import require
from infer_transfer_dependencies import LOCK_SHA256, archive_sources, packages, validate_metadata


FRONTEND = {
    "malformed": "blocks == true",
    "unsupported": "bitcoin_hashes::sha256::Hash` is not supported",
    "wrong_proxy": "is not supported",
    "opaque_access": "must remain unused",
    "opaque_alias": "Verus Infer unsupported",
    "opaque_mutable": "Verus Infer unsupported",
    "opaque_by_value": "exact borrowed readonly record",
    "opaque_trait": "trait inference requires",
    "opaque_invariant": "Verus Infer unsupported",
}
IMPORT_FAULTS = ("policy", "proof", "summaries", "dependencies", "trusted_models", "missing")
IMPORT_DIAGNOSTICS = {
    "policy": "incompatible inference schema, safety policy, or producer",
    "proof": "incomplete inference proof guarantees",
    "summaries": "inferred contract fingerprint mismatch",
    "dependencies": "stale or missing inferred dependency fingerprint",
    "trusted_models": "stale or missing trusted-model fingerprint",
    "missing": "missing or invalid inference proof metadata",
}
CONTRACTS = {
    "incoming_circuit": "0a6d900c48d2418ecab8f76675b2a78b814c99255fa39d60b35168a0a655fd58",
    "lnv2_blocks_to_claim_deadline": "8f6a43eae35a1b0f9198f4fdac9d005b5d0cfac353a5f092d5c8c3218424d137",
}
REGRESSION_CONTRACT = "bec664ff636acfbea91cbc0570b6e5c65d30c7ad945c20349ec320f29d0834d5"
PAGE_CONTRACTS = {
    "PageNumber::address_range": {
        "requires": "aaff18872fe8120a975bd7a8b8b7ccab61b423970547ca2a4a6fa59b49c6f569",
        "ensures": "c879aa90dd10521bce15f3b1495f2bc961e0da29a2fb67b946e6f3ebd52f6530",
    },
    "PageNumber::page_size_bytes": {
        "requires": "fe1bd69b27b8f2c28257d5d5635e299a008c698c8ecb0ad4294ec3ba81c7155f",
        "ensures": "ffae6debe0117588d210a0b4f4afb43b62189bbd292301ece3113c0c879ae6dd",
    },
    "relay": {
        "requires": "2e719644a4599c1fb8dfa14a72e755ddfdf32061efdf38e4923a43fa486ba667",
        "ensures": "2d7bc8e7818b75191b5eb91d5fd2d7ea77fff23c4891a778867c490fbe66d910",
    },
}
RESOURCE_ERRORS = ("panicked at", "resource limit", "timed out", "out of memory", "killed by signal")


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON evidence key: " + key)
        result[key] = value
    return result


def read_json(path):
    return json.loads(Path(path).read_text(), object_pairs_hook=unique_object)


def identical(left, right):
    return json.dumps(left, sort_keys=True) == json.dumps(right, sort_keys=True)


def file_in(directory, relative):
    relative = Path(relative)
    require(relative.parts and not relative.is_absolute() and ".." not in relative.parts,
            "invalid evidence path")
    path = directory / relative
    require(path.resolve().is_relative_to(directory.resolve()) and not path.is_symlink()
            and path.is_file(), "missing or escaping evidence file: " + str(relative))
    return path


def command(receipt, label):
    found = [entry for entry in receipt["commands"] if entry.get("label") == label]
    require(len(found) == 1, "missing or duplicate command: " + label)
    entry = found[0]
    require(type(entry.get("exit_code")) is int and entry["exit_code"] >= 0
            and entry.get("timed_out", False) is False and entry.get("interrupted", False) is False,
            "command did not exit normally: " + label)
    require(isinstance(entry.get("argv"), list) and all(isinstance(arg, str) for arg in entry["argv"]),
            "invalid command argv")
    return entry


def all_commands(directory, receipt):
    labels = [entry.get("label") for entry in receipt["commands"]]
    require(all(isinstance(label, str) and re.fullmatch(r"[A-Za-z0-9_-]+", label) for label in labels)
            and len(set(labels)) == len(labels), "duplicate or malformed command labels")
    for label in labels:
        command(receipt, label)
        for suffix in ("stdout", "stderr"):
            file_in(directory, label + "." + suffix)


def ledger(directory, hashes, prefix=""):
    require(isinstance(hashes, dict) and hashes, "missing source ledger")
    for relative, expected in hashes.items():
        require(digest(file_in(directory, str(Path(prefix) / relative))) == expected,
                "source ledger drift: " + relative)


def snapshot(before, after=None, required=()):
    require(isinstance(before, dict) and before, "missing artifact guard")
    if after is not None:
        require(identical(before, after), "artifact guard changed")
    require({str(Path(path).resolve()) for path in required} <= set(before),
            "artifact guard omits required inputs")
    for name, expected in before.items():
        path = Path(name)
        require(path.is_absolute() and not path.is_symlink() and path.is_file(), "missing guarded artifact")
        stat = path.stat()
        require(identical(expected, dict(sha256=digest(path), size=stat.st_size,
                                         mtime_ns=stat.st_mtime_ns, inode=stat.st_ino)),
                "guarded input drift: " + name)


def engine_ledger(root, hashes):
    environment = os.environ.copy()
    environment = {key: value for key, value in environment.items() if not key.startswith("GIT_")}
    environment.update(GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1")
    result = subprocess.run(["git", "-C", str(root), "ls-files", "--cached", "--others",
                             "--exclude-standard", "--", "source", "dependencies", "rust-toolchain.toml"],
                            env=environment, capture_output=True, text=True, timeout=30, check=True)
    expected = {name for name in result.stdout.splitlines() if (root / name).is_file()}
    require(set(hashes) == expected, "incomplete engine source ledger")
    ledger(root, hashes)


def proof_command(directory, receipt, label, verifier, source):
    entry = command(receipt, label)
    argv = entry["argv"]
    require(Path(argv[0]).resolve() == verifier.resolve() and str(source) in argv,
            "wrong tool or source for proof")
    require(argv.count("--export") == 1 and "--no-cheating" in argv and "--output-json" in argv,
            "missing proof/publication flags")
    forbidden = ("--no-verify", "--no-lifetime", "--verify-function", "--verify-module",
                 "--verify-root", "--no-auto-recommends-check")
    require(not any(arg.split("=")[0] in forbidden for arg in argv), "disabled or partial verification")
    require(Path(argv[argv.index("--export") + 1]) == directory / (label + ".vir"),
            "unexpected export destination")
    return entry


def frontend(directory, receipt, label, verifier, source, marker):
    entry = proof_command(directory, receipt, label, verifier, source)
    require(entry["exit_code"] > 0, "frontend control did not exit with an ordinary failure")
    stderr = file_in(directory, label + ".stderr").read_text()
    require(marker in stderr and not any(text in stderr.lower() for text in RESOURCE_ERRORS),
            "frontend control has the wrong diagnostic category")
    require(not any(text in stderr for text in ("postcondition not satisfied", "precondition not satisfied",
                "possible arithmetic underflow/overflow", "polynomial proposal not satisfied",
                "bitvector assertion not satisfied")), "proof failure mislabeled as frontend rejection")
    data = read_json(file_in(directory, label + ".stdout"))
    result, inference = data["verification-results"], data["verus-infer"]
    require(result.get("encountered-error") is True or result.get("encountered-vir-error") is True,
            "missing actual frontend error")
    require(result.get("success", False) is False and inference.get("crate_success") is False
            and inference.get("interface_written") is False, "frontend status must be false booleans")
    require(not (directory / (label + ".vir")).exists(), "frontend rejection retained seeded export")
    require(not (directory / (label + ".vir")).is_symlink(), "frontend export is a dangling link")
    return data


def metadata(blob):
    require(blob.endswith(b"VRINFER1"), "missing inference trailer")
    decoder = json.JSONDecoder(object_pairs_hook=unique_object)
    for index in range(len(blob) - 8):
        if blob[index:index + 1] != b"{":
            continue
        try:
            text = blob[index:-8].decode()
            value, end = decoder.raw_decode(text)
            if end == len(text) and isinstance(value, dict) and {"schema", "policy", "proofs", "summaries"} <= value.keys():
                return value, hashlib.sha256(text.encode()).hexdigest()
        except (ValueError, UnicodeError):
            continue
    raise ValueError("missing valid inference certificate")


def full_sources(constant, definition, implementation):
    policy = full.POLICY + full.FOLLOWUP + full.IRRELEVANCE
    sources = {
        "provider": full.proof_source(constant, definition, implementation),
        "regression": full.proof_source(constant, definition, full.regress(implementation)),
        "false_policy": full.proof_source(constant, definition, implementation, policy.replace("blocks == 0,", "blocks == 1,", 1)),
        "malformed": full.proof_source(constant, definition, implementation, policy.replace("blocks == 0,", "blocks == true,", 1)),
        "unsupported": full.proof_source(constant, definition, implementation, opaque=""),
        "wrong_proxy": full.proof_source(constant, definition, implementation, opaque=full.OPAQUE.replace("sha256::Hash", "bitcoin::hashes::sha512::Hash")),
        "opaque_access": full.proof_source(constant, definition, implementation.replace("        let circuit =", "        &self.payment_hash;\n        let circuit =")),
        "opaque_alias": full.proof_source(constant, definition, implementation.replace("        let circuit =", "        let alias = self;\n        let _hash = &alias.payment_hash;\n        let circuit =")),
        "opaque_mutable": full.proof_source(constant, definition, implementation, policy + "#[verifier::infer] pub fn opaque_mutable(request: &mut InterceptPaymentRequest) {}\n"),
        "opaque_by_value": full.proof_source(constant, definition, implementation, policy + "#[verifier::infer] pub fn opaque_by_value(request: InterceptPaymentRequest) -> [u8; 8] { request.amount_msat.to_le_bytes() }\n"),
        "opaque_trait": full.proof_source(constant, definition, implementation.replace("        let circuit =", "        let _equal = self.payment_hash == self.payment_hash;\n        let circuit =")),
        "opaque_invariant": full.invariant_control(constant, definition, implementation),
        "overflow": full.proof_source(constant, definition, implementation, policy + "pub fn overflow(r: &InterceptPaymentRequest, x: u32) -> u32 { r.incoming_circuit(); x + 1 }\n"),
        "panic": full.proof_source(constant, definition, implementation, policy + 'pub fn original_panic(r: &InterceptPaymentRequest) { r.incoming_circuit(); panic!("safety control"); }\n'),
        "client": "use vstd::prelude::*;\nuse full_provider::InterceptPaymentRequest;\nverus! {\n" + policy + "}\n",
    }
    return sources


def full_inputs(directory, receipt):
    require(receipt["source_sha256"] == full.INPUT_HASHES, "full application source identities changed")
    ledger(directory, full.INPUT_HASHES, "application-inputs")
    application = Path(command(receipt, "application_head")["argv"][2])
    ledger(application, full.INPUT_HASHES)
    for label, arguments, expected in (("application_head", ["rev-parse", "HEAD"], full.APPLICATION),
                                       ("application_clean", ["status", "--porcelain"], ""),
                                       ("application_clean_after", ["status", "--porcelain"], "")):
        entry = command(receipt, label)
        require(entry["exit_code"] == 0 and entry["argv"] == ["git", "-C", str(application), *arguments]
                and file_in(directory, label + ".stdout").read_text().strip() == expected,
                "pinned application inspection failed")
    constant, definition, implementation, derive = full.extract_request(
        file_in(directory, "application-inputs/" + full.LIGHTNING).read_text())
    expected = dict(request=full.REQUEST_HASH, implementation=full.IMPL_HASH, methods=full.METHOD_HASHES,
                    constant=full.text_digest(constant), omitted_derive=full.text_digest(derive),
                    opaque_correspondence=full.text_digest(full.OPAQUE))
    require(receipt["items"] == expected, "missing or altered source/item/correspondence inventory")
    adaptations = [
        dict(change="Omit the adjacent Debug/Serialize/Deserialize/Clone derive only", original=derive,
             before_sha256=full.text_digest(derive), after_sha256=full.text_digest("")),
        dict(change="Add one Infer attribute before each complete original method item",
             before_sha256=full.text_digest(implementation), after_sha256=full.text_digest(
                 implementation.replace("    pub fn ", "    #[verifier::infer]\n    pub fn "))),
        dict(change="Wrap exact request, constant and annotated impl in verus! and import real sha256 type",
             request_sha256=full.REQUEST_HASH),
        dict(change="Add explicit opaque external-type identity correspondence; no SHA/layout/trait laws",
             sha256=full.text_digest(full.OPAQUE)),
    ]
    require(receipt["adaptations"] == adaptations, "incomplete or altered source adaptation inventory")
    require(type(receipt["transfer"].get("field_count")) is int
            and receipt["transfer"]["field_count"] == 7, "complete seven-field request required")
    require(any("external-type correspondence" in trust and "bitcoin_hashes::sha256::Hash" in trust
                for trust in receipt["transfer"]["trust"]), "missing explicit external Hash trust boundary")
    sources = full_sources(constant, definition, implementation)
    for name, text in sources.items():
        require(file_in(directory, name + ".rs").read_text() == text, "proof source drift: " + name)
    native = "use bitcoin::hashes::sha256;\n" + constant + "\n" + definition + "\n" + implementation + "\n"
    require(file_in(directory, "native.rs").read_text() == native, "native helper source drift")
    funding = receipt["funding_caller"]
    require(funding.get("status") == "blocked" and isinstance(funding.get("first_blocker"), str)
            and "async persistent lookup" in funding["first_blocker"], "funding caller must remain blocked")
    require(set(funding["fixtures"]) == set(full.CALLERS), "incomplete funding source trace")
    for name, (relative, marker, expected_hash) in full.CALLERS.items():
        original = full.item(file_in(directory, "application-inputs/" + relative).read_text(), marker)
        require(full.text_digest(original) == expected_hash
                and file_in(directory, name + ".source.rs").read_text() == original,
                "funding caller source drift")
        require(funding["fixtures"][name] == dict(source=relative, marker=marker, sha256=expected_hash),
                "funding trace identity changed")
    return sources


def full_dependencies(directory, receipt):
    application_lock = file_in(directory, "application-inputs/Cargo.lock").read_text()
    selected = packages(application_lock)
    dependency = receipt["dependency"]
    require(identical(dependency["packages"], selected), "incomplete or substituted pinned dependency graph")
    require(dependency["lock_sha256"] == LOCK_SHA256
            and digest(file_in(directory, "dependency/Cargo.lock")) == LOCK_SHA256,
            "selected dependency lock changed")
    require(file_in(directory, "dependency/Cargo.toml").read_text() == full.DEPENDENCY
            and file_in(directory, "dependency/src/lib.rs").read_text() == "pub use bitcoin::hashes::sha256;\n",
            "dependency manifest/source changed")
    guards = receipt["dependency_sources_before"]
    archive_paths = [Path(name) for name in guards if name.endswith(".crate")]
    require(len(archive_paths) == 21, "incomplete dependency archive guard")
    required = []
    for package in selected:
        name = package["name"] + "-" + package["version"]
        candidates = [path for path in archive_paths if path.name == name + ".crate"]
        require(len(candidates) == 1, "missing or ambiguous pinned archive")
        archive = candidates[0]
        source = archive.parents[2] / "src" / archive.parent.name / name
        required.extend(archive_sources(archive, source, package["checksum"]))
    required.extend(directory / "dependency" / name for name in ("Cargo.toml", "Cargo.lock", "src/lib.rs"))
    snapshot(guards, receipt["dependency_sources_after"], required)
    require(set(guards) == {str(path.resolve()) for path in required}, "unexpected dependency source guard scope")
    artifacts = dependency["artifacts"]
    actual = {str(path.relative_to(directory / "dependency-target")) for path in
              (directory / "dependency-target").rglob("*") if path.is_file() and
              (path.suffix in (".rlib", ".rmeta", ".dylib", ".json") or ".fingerprint" in path.parts)}
    require(set(artifacts) == actual and actual, "incomplete dependency artifact ledger")
    ledger(directory, artifacts, "dependency-target")
    bitcoin = [name for name in artifacts if re.fullmatch(r"release/deps/libbitcoin-[a-z0-9]+\.rlib", name)]
    require(len(bitcoin) == 1 and artifacts[bitcoin[0]] == dependency["bitcoin_rlib_sha256"],
            "wrong Bitcoin artifact")
    for label, operation in (("dependency_resolve", "metadata"), ("dependency_locked", "build")):
        entry = command(receipt, label)
        require(entry["exit_code"] == 0 and entry["argv"][:2] == ["cargo", operation]
                and "--offline" in entry["argv"], "dependency proof setup was not offline")
        if operation == "build":
            require("--locked" in entry["argv"] and "--release" in entry["argv"], "unlocked dependency build")
    metadata_report = read_json(file_in(directory, "dependency_resolve.stdout"))
    observed = validate_metadata(metadata_report, selected)
    for package in selected:
        name = package["name"] + "-" + package["version"]
        archive = next(path for path in archive_paths if path.name == name + ".crate")
        expected_source = archive.parents[2] / "src" / archive.parent.name / name / "Cargo.toml"
        require(Path(observed[(package["name"], package["version"])]["manifest_path"]).resolve() == expected_source.resolve(),
                "build source differs from guarded archive source")
    return directory / "dependency-target" / bitcoin[0]


def exact_helpers(data, name, positive):
    inference = data["verus-infer"]
    functions = inference["functions"]
    expected = {"full_" + name + "::InterceptPaymentRequest::" + method for method in CONTRACTS}
    require(len(functions) == 2 and {function["function"] for function in functions} == expected,
            "both original helpers must be checked exactly once")
    for function in functions:
        method = function["function"].split("::")[-1]
        expected_hash = REGRESSION_CONTRACT if name == "regression" and method != "incoming_circuit" else CONTRACTS[method]
        require(function["contract"].get("requires") == "true"
                and full.text_digest(function["contract"]["ensures"]) == expected_hash,
                "helper domain/equation changed")
        require(function.get("exported") is positive and function.get("available_to_same_crate_callers") is True,
                "helper publication/availability changed")
        require(function["metrics"].get("cache") == "miss"
                and function["metrics"].get("cache_policy") == "regenerated_state", "dishonest record cache claim")
    require(inference.get("requested_functions") == 2 and inference.get("generated_functions") == 2,
            "helper inference scope changed")
    require(inference.get("generation_failure") is None, "helper generation failed")


def full_scope(inference, name, positive):
    scope = inference["verification_scope"]
    require(scope.get("native_collection_completed") is True
            and scope.get("complete_selected_scope_verified") is positive, "incomplete native verification scope")
    methods = {"deadline_policy", "deadline_boundary", "deadline_monotonicity", "deadline_irrelevance"}
    if name != "client":
        methods |= {"InterceptPaymentRequest::" + method for method in CONTRACTS}
    if name in ("overflow", "panic"):
        methods.add("overflow" if name == "overflow" else "original_panic")
    functions = scope["native_functions"]
    observed = {function["rust_function"] for function in functions}
    constants = set() if name == "client" else {"VERUS_UNERASED_PROXY__NO_INCOMING_CIRCUIT"}
    require(observed == methods | constants and len(functions) == len(observed), "native body inventory changed")
    for function in functions:
        status = ("selected_item_without_translated_body" if function["rust_function"] in constants else
                  "checked_exec_body" if positive else "selected_body_not_certified_by_successful_full_run")
        require(function.get("selected") is True and function.get("automatically_derived") is False
                and function.get("body_status") == status, "skipped or fake native proof body")
    require(type(scope.get("checked_exec_bodies")) is int
            and scope["checked_exec_bodies"] == (len(methods) if positive else 0)
            and type(scope.get("checked_proof_bodies")) is int and scope["checked_proof_bodies"] == 0,
            "unexpected checked-body counts")


def check_full_request(path, receipt, verifier, root, check_proof):
    directory, transfer = path.parent, receipt["transfer"]
    require(transfer.get("stale_export_rejected") is True, "missing full-request publication control")
    all_commands(directory, receipt)
    sources = full_inputs(directory, receipt)
    bitcoin = full_dependencies(directory, receipt)
    engine_ledger(root, receipt["engine_source_sha256"])
    provider_rlib, provider_vir = directory / "libfull_provider.rlib", directory / "provider_warm.vir"
    snapshot(receipt["provider_artifacts_before"], receipt["provider_artifacts_after"], [provider_rlib, provider_vir])
    require(set(receipt["provider_artifacts_before"]) == {str(provider_rlib), str(provider_vir)},
            "unexpected provider guard scope")
    solver = Path(command(receipt, "provider_cold")["environment"]["VERUS_Z3_PATH"])
    compiler = file_in(directory, "rustc.stdout").read_text().strip()
    application = Path(command(receipt, "application_head")["argv"][2])
    required = [*tool_artifacts(verifier, solver), *(root / name for name in receipt["engine_source_sha256"]),
                *(directory / (name + ".rs") for name in [*sources, "native"]),
                *(directory / (name + ".source.rs") for name in full.CALLERS),
                *(application / name for name in full.INPUT_HASHES),
                *(Path(name) for name in receipt["dependency_sources_before"]), Path(compiler),
                *(path for path in bitcoin.parent.iterdir() if path.suffix in (".rlib", ".rmeta", ".dylib"))]
    snapshot(receipt["artifacts_before"], receipt["artifacts_after"], required)
    require(digest(file_in(directory, "native.rlib")) == receipt["native_rlib_sha256"], "native artifact changed")
    dependency_arguments = ["--extern", "bitcoin=" + str(bitcoin), "-L", "dependency=" + str(bitcoin.parent)]
    require(command(receipt, "rustc")["argv"] == ["rustup", "which", "--toolchain", "1.99.1", "rustc"]
            and command(receipt, "native_build")["argv"] == [compiler, str(directory / "native.rs"),
                "--edition=2021", "--crate-type=lib", "-C", "overflow-checks=yes", "-o", str(directory / "native.rlib"),
                *dependency_arguments], "native typing did not use the pinned source/compiler/dependency")
    positives = ["provider_cold", "provider_warm", "client_cold", "client_warm"]
    semantics = ["regression_cold", "false_policy_cold"]
    safety = ["overflow_cold", "panic_cold"]
    frontends = [name + "_cold" for name in FRONTEND] + ["client_metadata_" + fault for fault in IMPORT_FAULTS]
    baseline_labels = {"unsupported_baseline", "baseline_opaque_baseline"}
    cases = receipt["cases"]
    rows = {row["name"] + "_" + row["phase"]: row for row in cases}
    require(len(rows) == len(cases), "duplicate full-request cases")
    optional = set(rows) & baseline_labels
    require(not optional or optional == baseline_labels, "incomplete optional baseline comparisons")
    require(set(rows) == set(positives + semantics + safety + frontends) | optional, "incomplete or extra full-request controls")
    for field, labels in (("positive_cases", positives), ("semantic_negative_cases", semantics),
                          ("safety_negative_cases", safety), ("frontend_negative_cases", frontends + sorted(optional))):
        declared = transfer.get(field)
        require(isinstance(declared, list) and len(declared) == len(labels) and set(declared) == set(labels),
                "incomplete or duplicate declared controls: " + field)
    for field in ("auxiliary_negative_cases", "polynomial_auxiliary_negative_cases"):
        require(transfer.get(field, []) == [], "unexpected auxiliary classification")
    proof_labels = set(rows)
    require(all(entry["exit_code"] == 0 for entry in receipt["commands"] if entry["label"] not in proof_labels),
            "failed prerequisite command")
    seed = digest(file_in(directory, "provider_cold.vir"))
    proofs, rejected, baseline = [], [], []
    certificate, fingerprint = metadata(provider_vir.read_bytes())
    require(type(certificate.get("schema")) is int and certificate["schema"] == INTERFACE_SCHEMA
            and certificate.get("policy") == INTERFACE_POLICY
            and all(certificate.get(key) is True for key in ("soundness", "maximality", "exactness"))
            and len(certificate["proofs"]) == 2, "provider certificate is not fully checked")
    require({entry[0]["path"]["segments"][-1] for entry in certificate["proofs"]} == set(CONTRACTS)
            and all(entry[1].get(key) is True for entry in certificate["proofs"]
                    for key in ("soundness", "maximality", "exactness")), "provider certificate omitted exact helper checks")
    identities = {entry[0]["path"]["krate"]["Id"][1] for entry in certificate["proofs"]}
    require(len(identities) == 1, "ambiguous provider crate identity")
    imported = [dict(crate_id=next(iter(identities)), fingerprint=fingerprint)]
    for label, row in rows.items():
        name = "provider" if row["name"] == "baseline_opaque" else row["name"]
        source = file_in(directory, name + ".rs")
        require(row["source_sha256"] == digest(source), "case source hash differs")
        positive = label in positives
        require(row.get("seeded_export_sha256") == (None if positive else seed), "unbound stale export seed")
        artifact = directory / (label + ".vir")
        require(row.get("export_sha256") == (digest(artifact) if positive else None)
                and (artifact.exists() is positive) and not artifact.is_symlink(), "stale export not removed")
        if label in frontends or label in optional:
            tool = verifier
            if label in optional:
                tool = Path(command(receipt, label)["argv"][0])
                require(tool.resolve() != verifier.resolve(), "baseline comparison substituted for current proof")
                snapshot(receipt["baseline_artifacts_before"], receipt["baseline_artifacts_after"], [tool])
            marker = FRONTEND.get(name, "Verus Infer:")
            if row["phase"].startswith("metadata_"):
                marker += " " + IMPORT_DIAGNOSTICS[row["phase"].removeprefix("metadata_")]
            if label == "baseline_opaque_baseline":
                marker = "Verus Infer unsupported"
            data = frontend(directory, receipt, label, tool, source, marker)
            require(row["category"] == "frontend", "frontend category changed")
            (baseline if label in optional else rejected).append(dict(label=label, category="frontend",
                exit_code=command(receipt, label)["exit_code"], stderr_sha256=digest(file_in(directory, label + ".stderr"))))
        else:
            proof_command(directory, receipt, label, verifier, source)
            kind = "arithmetic" if name == "overflow" else "precondition" if name == "panic" else "postcondition"
            proofs.append(check_proof(directory, receipt, label, positive, verifier, kind, import_only=name == "client"))
            data = read_json(file_in(directory, label + ".stdout"))
            result, inference = data["verification-results"], data["verus-infer"]
            full_scope(inference, name, positive)
            count = 4 if name == "client" else 10 if label in semantics else 11
            require(type(result.get("verified")) is int and result["verified"] == count
                    and type(result.get("errors")) is int and result["errors"] == (0 if positive else 1),
                    "unexpected complete-request proof counts")
            require(row["category"] == ("positive" if positive else "semantic" if label in semantics else "safety"),
                    "proof category changed")
            if name == "client":
                require(type(inference.get("requested_functions")) is int and inference["requested_functions"] == 0
                        and type(inference.get("generated_functions")) is int and inference["generated_functions"] == 0
                        and identical(inference.get("imported_inference_dependencies"), imported), "unbound imported equations")
                require(inference.get("status") == "no_inference", "import client was not an import-only proof")
            else:
                exact_helpers(data, name, positive)
                if positive:
                    require(inference.get("status") == "verified" and inference.get("failed_functions") == [],
                            "provider inference was not fully verified")
            if label in semantics:
                require(inference.get("failed_functions") == ["full_" + name + "::deadline_policy"],
                        "semantic control failed outside authored deadline policy")
            stderr = file_in(directory, label + ".stderr").read_text()
            if name == "regression":
                require("request.incoming_chan_id != 0 || request.htlc_id != 0" in stderr,
                        "guard deletion did not refute intercepted-forward policy")
            if name == "panic":
                require('panic!("safety control")' in stderr and "false," in stderr,
                        "panic is not the requires-false safety control")
        require(identical(data["verification-results"], row["verification"])
                and identical(data["verus-infer"], row["inference"]), "receipt case does not match raw compiler report")
        argv = command(receipt, label)["argv"]
        expected_argv = [str(tool.resolve()) if label in frontends or label in optional else str(verifier.resolve()),
            str(source), "--crate-type=lib", "--edition=2021", "--crate-name", "full_" + row["name"],
            "--no-cheating", "--output-json", "--num-threads", "1", "--export", str(artifact), *dependency_arguments]
        if name == "client":
            require(argv.count("--import") == 1 and argv.count("--extern") == 2
                    and "full_provider=" + str(provider_rlib) in argv, "client omitted provider rlib/import")
            expected_vir = provider_vir if row["phase"] in ("cold", "warm") else directory / ("bad-" + row["phase"].removeprefix("metadata_") + ".vir")
            require(argv[argv.index("--import") + 1] == "full_provider=" + str(expected_vir), "wrong imported interface")
            expected_argv += ["--extern", "full_provider=" + str(provider_rlib), "--import", "full_provider=" + str(expected_vir),
                              "-L", "dependency=" + str(directory)]
        if row["name"] == "provider":
            expected_argv += ["--compile", "-o", str(provider_rlib)]
        require(argv == expected_argv, "unexpected proof command/options or substituted dependency: " + label)
    for fault in IMPORT_FAULTS:
        def edit(value):
            if fault == "policy": value["policy"] = "wrong-policy"
            elif fault == "proof": value["exactness"] = False
            elif fault == "summaries": value["summaries"] = "0" * 64
            elif fault == "dependencies": value["dependencies"] = [dict(crate_id=0, fingerprint="0" * 64)]
            else: value["trusted_models"][0][1] = "0" * 64
        original = provider_vir.read_bytes()
        expected = original[:-8] if fault == "missing" else full.replace_metadata(original, edit)
        require(file_in(directory, "bad-" + fault + ".vir").read_bytes() == expected, "wrong corrupted-import control")
    return dict(proofs=proofs, frontend_controls=rejected, baseline_comparisons=baseline,
                dependency_packages=receipt["dependency"]["packages"], items=receipt["items"],
                funding_caller=receipt["funding_caller"], stale_export_rejected=True)


def page_proof(data, name, positive, imported, relay_dependency):
    relations = {"strict_start", "injective_start", "ordered_nonoverlap", "nonoverlap_all_sizes"}
    profiles = {
        "relations": (24, relations),
        "zero": (7, {"zero_size"}),
        "witnesses": (22, {"zero_witness", "mixed_witness"}),
        "provider": (6, set()),
        "client": (21, relations | {"relay"}),
        "false_polynomial": (9, {"wrong"}),
        **{control: (22, {"zero_witness", "mixed_witness", "wrong"}) for control in pages.NEGATIVES},
        **{control: (6, {"rejected"}) for control in pages.SAFETY},
    }
    verified, bodies = profiles[name]
    result, inference = data["verification-results"], data["verus-infer"]
    require(type(result.get("verified")) is int and result["verified"] == verified
            and type(result.get("errors")) is int and result["errors"] == (0 if positive else 1),
            "unexpected relational-page proof counts")
    methods = {"relay"} if name == "client" else set(PAGE_CONTRACTS) - {"relay"}
    functions = inference["functions"]
    require(len(functions) == len(methods)
            and {function["function"] for function in functions} == {name + "::" + method for method in methods},
            "relational helper scope changed")
    for function in functions:
        method = function["function"].split("::", 1)[1]
        contract = function["contract"]
        require(all(isinstance(contract.get(key), str) and full.text_digest(contract[key]) == expected
                    for key, expected in PAGE_CONTRACTS[method].items()), "relational helper domain/equation changed")
        require(function.get("exported") is (positive and name in ("provider", "client"))
                and function.get("available_to_same_crate_callers") is True,
                "relational helper publication/availability changed")
        require(function["metrics"].get("cache") == "miss"
                and function["metrics"].get("cache_policy") == "regenerated_state", "dishonest record cache claim")
        if name == "client":
            require(identical(function.get("dependencies"), [relay_dependency]),
                    "unbound relational relay dependency")
    require(all(type(inference.get(key)) is int and inference[key] == len(methods)
                for key in ("requested_functions", "generated_functions"))
            and inference.get("generation_failure") is None, "relational inference scope/generation changed")
    failed = [] if positive else [name + "::" + ("rejected" if name in pages.SAFETY else "wrong")]
    require(inference.get("failed_functions") == failed
            and inference.get("status") == ("verified" if positive else "caller_proof_failed"),
            "relational proof failed outside declared caller")
    require(identical(inference.get("imported_inference_dependencies"), imported if name == "client" else []),
            "unbound relational imported equations")
    if name != "client":
        bodies = bodies | methods
    scope = inference.get("verification_scope")
    require(isinstance(scope, dict) and scope.get("native_collection_completed") is True
            and scope.get("complete_selected_scope_verified") is positive, "incomplete relational native scope")
    native = scope["native_functions"]
    require(len(native) == len(bodies) and {function["rust_function"] for function in native} == bodies,
            "relational native body inventory changed")
    status = "checked_exec_body" if positive else "selected_body_not_certified_by_successful_full_run"
    require(all(function.get("selected") is True and function.get("automatically_derived") is False
                and function.get("body_status") == status for function in native), "skipped or fake relational native body")
    require(type(scope.get("checked_exec_bodies")) is int
            and scope["checked_exec_bodies"] == (len(bodies) if positive else 0)
            and type(scope.get("checked_proof_bodies")) is int and scope["checked_proof_bodies"] == 0,
            "unexpected relational checked-body counts")


def check_page_relations(path, receipt, verifier, root):
    directory, transfer = path.parent, receipt["transfer"]
    all_commands(directory, receipt)
    positives = ["relations_cold", "relations_warm", "relations_poisoned_cache", "zero_ordinary",
                 "witnesses_checked", "provider_cold", "client_cold", "client_warm"]
    negatives = [name + "_negative" for name in pages.NEGATIVES]
    safety = [name + "_negative" for name in pages.SAFETY]
    polynomial = ["false_polynomial_negative"]
    metadata_cases = ["client_metadata_" + name for name in ("policy", "exactness", "summaries", "trusted_model")]
    for field, labels in (("positive_cases", positives), ("semantic_negative_cases", negatives),
                          ("safety_negative_cases", safety), ("polynomial_auxiliary_negative_cases", polynomial),
                          ("metadata_negative_cases", metadata_cases), ("auxiliary_negative_cases", [])):
        require(transfer.get(field, []) == labels, "incomplete relational-page controls: " + field)
    rows = {row["label"]: row for row in receipt["cases"]}
    require(len(rows) == len(receipt["cases"]) and set(rows) == set(positives + negatives + safety + polynomial + metadata_cases),
            "duplicate or incomplete relational-page cases")
    prerequisites = {"application_before_pin", "application_before_clean", "engine_files",
                     "application_after_pin", "application_after_clean"}
    require({entry["label"] for entry in receipt["commands"] if entry["label"] not in rows} == prerequisites,
            "incomplete relational prerequisite commands")
    require(all(entry["exit_code"] == 0 for entry in receipt["commands"] if entry["label"] not in rows),
            "failed relational prerequisite command")
    inspection = command(receipt, "application_before_pin")["argv"]
    require(len(inspection) == 5, "invalid relational application inspection command")
    application_path = inspection[2]
    for phase in ("before", "after"):
        for suffix, arguments, expected in (("pin", ["rev-parse", "HEAD"], pages.APPLICATION),
                ("clean", ["status", "--porcelain", "--untracked-files=all"], "")):
            label = "application_" + phase + "_" + suffix
            entry = command(receipt, label)
            require(entry["argv"] == ["git", "-C", application_path, *arguments]
                    and entry.get("cwd") == str(root)
                    and file_in(directory, label + ".stdout").read_text().strip() == expected,
                    "relational application inspection changed: " + label)
    enumeration = command(receipt, "engine_files")
    inputs = file_in(directory, "engine_files.stdout").read_text().splitlines()
    require(enumeration["argv"] == ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--",
                                   "source", "dependencies", "rust-toolchain.toml"]
            and enumeration.get("cwd") == str(root)
            and len(inputs) == len(receipt["source_hashes"]) and set(inputs) == set(receipt["source_hashes"]),
            "relational engine source enumeration changed")
    require(digest(file_in(directory, "base.original.rs")) == pages.SOURCE_HASH, "redb source drift")
    with tempfile.TemporaryDirectory(prefix="infer-page-acceptance-") as temporary:
        application, generated = Path(temporary) / "application", Path(temporary) / "generated"
        original = application / pages.BASE
        original.parent.mkdir(parents=True)
        original.write_bytes(file_in(directory, "base.original.rs").read_bytes())
        generated.mkdir()
        extraction = pages.extract(application, generated)
        require(receipt["extraction"] == extraction, "redb original item identities changed")
        expected = {source.name: digest(source) for source in generated.glob("*.rs")}
        require(receipt["fixture_hashes"] == expected, "redb fixture inventory changed")
        ledger(directory, expected)
    engine_ledger(root, receipt["source_hashes"])
    ledger(directory, receipt["source_hashes"], "inputs")
    solver = Path(command(receipt, "relations_cold")["environment"]["VERUS_Z3_PATH"])
    snapshot(receipt["artifacts"], required=tool_artifacts(verifier, solver,
             extra=[root / name for name in receipt["source_hashes"]]))
    rejected = []
    provider = file_in(directory, "provider_cold.vir")
    provider_rlib = file_in(directory, "libprovider.rlib")
    snapshot(receipt["provider_artifacts_before"], receipt["provider_artifacts_after"], [provider, provider_rlib])
    require(set(receipt["provider_artifacts_before"]) == {str(provider), str(provider_rlib)},
            "unexpected relational provider guard scope")
    original = provider.read_bytes()
    certificate, fingerprint = metadata(original)
    require(type(certificate.get("schema")) is int and certificate["schema"] == INTERFACE_SCHEMA
            and certificate.get("policy") == INTERFACE_POLICY
            and all(certificate.get(key) is True for key in ("soundness", "maximality", "exactness"))
            and len(certificate["proofs"]) == 2, "relational provider certificate is not fully checked")
    require({entry[0]["path"]["segments"][-1] for entry in certificate["proofs"]} == {"page_size_bytes", "address_range"}
            and all(entry[1].get(key) is True for entry in certificate["proofs"]
                    for key in ("soundness", "maximality", "exactness")), "relational provider omitted exact helper checks")
    identities = {entry[0]["path"]["krate"]["Id"][1] for entry in certificate["proofs"]}
    require(len(identities) == 1, "ambiguous relational provider crate identity")
    imported = [dict(crate_id=next(iter(identities)), fingerprint=fingerprint)]
    provider_data = read_json(file_in(directory, "provider_cold.stdout"))
    helpers = [function for function in provider_data["verus-infer"]["functions"]
               if function["function"] == "provider::PageNumber::address_range"]
    require(len(helpers) == 1 and isinstance(helpers[0].get("candidate_id"), str)
            and re.fullmatch(r"[0-9a-f]{64}", helpers[0]["candidate_id"]),
            "missing relational provider helper candidate")
    helper_path = next(entry[0]["path"] for entry in certificate["proofs"]
                       if entry[0]["path"]["segments"][-1] == "address_range")
    relay_dependency = dict(function="::".join([helper_path["krate"]["Id"][0], *helper_path["segments"]]),
                            candidate_id=helpers[0]["candidate_id"], mode="exact")
    require(read_json(file_in(directory, "relations-cache/poisoned.json")) ==
            dict(version="verus-infer-v70", pre=True, post=False), "missing poisoned-cache positive control")
    for label, row in rows.items():
        source = file_in(directory, label.rsplit("_", 1)[0] + ".rs") if label not in metadata_cases and label != "relations_poisoned_cache" else file_in(directory, "client.rs" if label in metadata_cases else "relations.rs")
        proof_command(directory, receipt, label, verifier, source)
        argv = command(receipt, label)["argv"]
        expected_argv = [str(verifier.resolve()), str(source), "--crate-type", "lib", "--crate-name", source.stem,
            "--no-cheating", "--output-json", "--compile", "--export", str(directory / (label + ".vir")),
            "-o", str(directory / ("lib" + source.stem + ".rlib")), "-C", "debug-assertions=yes"]
        if source.stem == "client":
            interface = directory / ("provider_poison_" + label.removeprefix("client_metadata_") + ".vir") if label in metadata_cases else provider
            expected_argv += ["--extern", "provider=" + str(directory / "libprovider.rlib"), "--import", "provider=" + str(interface)]
        if label != "zero_ordinary" and label not in safety:
            expected_argv.append("--infer-goals")
        require(argv == expected_argv, "unexpected relational-page proof command/options: " + label)
        expected_success = label in positives
        classification = ("proved" if expected_success else "semantic_goal_failure" if label in negatives else
                          "caller_obligation_failure" if label in safety else "auxiliary_theorem_failure" if label in polynomial else "frontend_rejection")
        require(row["artifact"] == str(directory / (label + ".vir"))
                and row["classification"] == classification and row["artifact_exists"] is expected_success
                and row["stale_export_seeded"] is (not expected_success), "relational control classification/seed changed")
        data = read_json(file_in(directory, label + ".stdout"))
        require(identical(data, row["report"]), "relational report differs from raw proof log")
        if label in metadata_cases:
            fault = label.removeprefix("client_metadata_")
            diagnostic = IMPORT_DIAGNOSTICS[{"exactness": "proof", "trusted_model": "trusted_models"}.get(fault, fault)]
            frontend(directory, receipt, label, verifier, source, "Verus Infer: " + diagnostic)
            def edit(value):
                if fault == "policy": value["policy"] = "candidate-bound-evidence-v0"
                elif fault == "exactness": value["exactness"] = False
                elif fault == "summaries": value["summaries"] = "0" * 64
                else: value["trusted_models"][0][1] = "0" * 64
            corrupted = file_in(directory, "provider_poison_" + fault + ".vir")
            require(corrupted.read_bytes() == full.replace_metadata(original, edit), "incorrect redb metadata corruption")
            argv = command(receipt, label)["argv"]
            require(argv.count("--import") == 1
                    and argv[argv.index("--import") + 1] == "provider=" + str(corrupted), "wrong redb corrupted import")
            rejected.append(dict(label=label, category="metadata", exit_code=command(receipt, label)["exit_code"]))
        else:
            page_proof(data, source.stem, expected_success, imported, relay_dependency)
    return dict(metadata_controls=rejected, polynomial_auxiliary_count=1,
                semantic_negative_count=len(negatives), safety_negative_count=len(safety))
