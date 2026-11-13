"""Consistency gates for the frozen stateful-bitmap goal, not execution attestation.

Expectations must come from the independently pinned replay definition, never
from the receipt being checked. Synthetic unit fixtures are not proof evidence.
"""

import hashlib
import json
from pathlib import Path


class ReceiptError(ValueError):
    pass


def require(condition, name):
    if not condition:
        raise ReceiptError(name)


def mapping(value, name):
    require(type(value) is dict, name)
    return value


def rows(value, name):
    require(type(value) is list and all(type(row) is dict for row in value), name)
    return value


def integer(value, name, minimum=0):
    require(type(value) is int and value >= minimum, name)
    return value


def unique_functions(value, name):
    selected = rows(value, name)
    require(all(type(row.get("function")) is str for row in selected), name)
    require(len({row["function"] for row in selected}) == len(selected), name)
    return {row["function"]: row for row in selected}


def validate_positive_report(document, expected):
    document = mapping(document, "report.document")
    result = mapping(document.get("verification-results"), "report.results")
    require(result.get("success") is True, "report.success")
    require(result.get("encountered-error") is False, "report.no-errors")
    require(result.get("encountered-vir-error") is False, "report.no-frontend-errors")
    require(result.get("is-verifying-entire-crate") is True, "report.entire-crate")
    require(type(result.get("errors")) is int and result["errors"] == 0, "report.error-count")
    require(type(result.get("verified")) is int and result["verified"] == expected["verified"],
            "report.verified-count")

    inference = mapping(document.get("verus-infer"), "report.inference")
    require(inference.get("crate_success") is True, "infer.crate-success")
    require(inference.get("interface_written") is expected["interface_written"],
            "infer.publication")
    require("generation_failure" in inference and inference["generation_failure"] is None,
            "infer.generation-failure")
    require(inference.get("failed_functions") == [], "infer.failed-functions")
    helpers = unique_functions(inference.get("functions"), "infer.helper-shape")
    require(set(helpers) == set(expected["helpers"]), "infer.helper-set")
    require(type(inference.get("requested_functions")) is int
            and inference["requested_functions"] == len(helpers), "infer.requested-count")
    require(type(inference.get("generated_functions")) is int
            and inference["generated_functions"] == len(helpers), "infer.generated-count")

    for name, contract in expected["helpers"].items():
        helper = helpers[name]
        require(helper.get("mode") == "exact", "helper.exact-mode")
        require(helper.get("required_checks_passed") is True, "helper.required-checks")
        require(helper.get("available_to_same_crate_callers") is True, "helper.available")
        require(helper.get("exported") is contract["exported"], "helper.export-status")
        require(helper.get("contract") == contract["contract"], "helper.contract")
        proofs = mapping(helper.get("proofs"), "helper.proofs")
        for obligation in ("soundness", "maximality", "exactness"):
            proof = mapping(proofs.get(obligation), "helper.proof-schema")
            require(proof.get("required") is True and proof.get("status") == "proved",
                    "helper.proof-" + obligation)
        metrics = mapping(helper.get("metrics"), "helper.metrics")
        integer(metrics.get("solver_micros"), "helper.fresh-work", minimum=1)
        require(metrics.get("cache") == contract["cache"], "helper.cache-state")

    scope = mapping(inference.get("verification_scope"), "scope.schema")
    require(scope.get("native_collection_completed") is True, "scope.collection-complete")
    require(scope.get("complete_selected_scope_verified") is True, "scope.verification-complete")
    bodies = unique_functions(scope.get("native_functions"), "scope.body-shape")
    require(set(bodies) == set(expected["bodies"]), "scope.body-set")
    for name, wanted in expected["bodies"].items():
        actual = bodies[name]
        require(actual.get("body_status") == wanted["body_status"], "scope.body-status")
        require(actual.get("selected") is wanted["selected"], "scope.body-selected")
        require(actual.get("inference_candidate") is wanted["inference_candidate"],
                "scope.body-inference")
        require(actual.get("ordinary_contract") is wanted["ordinary_contract"],
                "scope.body-contract")
        require(actual.get("automatically_derived") is wanted["automatically_derived"],
                "scope.body-derived")
    require(type(scope.get("checked_exec_bodies")) is int
            and scope["checked_exec_bodies"] == sum(
                body["body_status"] == "checked_exec_body" for body in bodies.values()),
            "scope.exec-count")
    require(type(scope.get("checked_proof_bodies")) is int
            and scope["checked_proof_bodies"] == sum(
                body["body_status"] == "checked_proof_body" for body in bodies.values()),
            "scope.proof-count")

    dependencies = rows(inference.get("imported_inference_dependencies"), "import.shape")
    require(all(type(row.get("crate_id")) is int and row["crate_id"] >= 0
                and type(row.get("fingerprint")) is str for row in dependencies), "import.schema")
    require(len({row["crate_id"] for row in dependencies}) == len(dependencies), "import.unique")
    require({row["crate_id"]: row["fingerprint"] for row in dependencies}
            == expected["dependencies"], "import.binding")
    require(inference.get("trusted_models") == expected["trusted_models"], "infer.trusted-models")
    return document


def file_bytes(record, expected, name, nonempty):
    record = mapping(record, "artifact.schema")
    require(record == expected, "artifact.binding")
    path = record.get("path")
    require(type(path) is str and Path(path).is_absolute(), "artifact.path")
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), "artifact.exists:" + name)
    payload = path.read_bytes()
    require(not nonempty or bool(payload), "artifact.nonempty:" + name)
    require(hashlib.sha256(payload).hexdigest() == record.get("sha256"), "artifact.hash:" + name)
    return payload


def command_option(arguments, name, value):
    require(arguments.count(name) == 1 and arguments.index(name) + 1 < len(arguments)
            and arguments[arguments.index(name) + 1] == value, "command.option:" + name)


def produced_path(record):
    value = record.get("produced_path", record.get("path"))
    require(type(value) is str and Path(value).is_absolute(), "artifact.produced-path")
    return value


def validate_positive_run(run, expected):
    run = mapping(run, "run.schema")
    require(type(run.get("returncode")) is int and run["returncode"] == 0, "run.exit-status")
    arguments = run.get("argv")
    require(type(arguments) is list and all(type(argument) is str for argument in arguments),
            "command.schema")
    require(arguments == expected["argv"], "command.binding")
    require(all(arguments.count(flag) == 1 for flag in
                ("--compile", "--no-cheating", "--output-json", "--crate-type=lib")),
            "command.mandatory-flags")
    require(not any(argument.startswith(("--import=", "--extern=", "--export=", "-o="))
                    for argument in arguments), "command.canonical-flags")
    artifacts = mapping(run.get("artifacts"), "artifact.inventory")
    wanted = expected["artifacts"]
    require(set(artifacts) == set(wanted) == {"input", "stdout", "stderr", "library", "interface"},
            "artifact.inventory")
    require(arguments.count(produced_path(wanted["input"])) == 1, "command.input")
    command_option(arguments, "-o", produced_path(wanted["library"]))
    command_option(arguments, "--export", produced_path(wanted["interface"]))
    command_option(arguments, "--num-threads", "1")
    for name in ("input", "library", "interface"):
        file_bytes(artifacts[name], wanted[name], name, nonempty=True)
    stderr = file_bytes(artifacts["stderr"], wanted["stderr"], "stderr", nonempty=False)
    require(not any(message in stderr.lower() for message in
                    (b"panicked at", b"timed out", b"resource limit", b"internal compiler error")),
            "run.no-resource-or-internal-error")
    stdout = file_bytes(artifacts["stdout"], wanted["stdout"], "stdout", nonempty=True)
    try:
        document = json.loads(stdout)
    except (ValueError, UnicodeError) as error:
        raise ReceiptError("report.json") from error
    return validate_positive_report(document, expected["report"])
