"""Frozen R1 source/receipt guards; evidence is not an independent proof certificate."""

import argparse
import hashlib
import json
import re
from pathlib import Path


class GuardError(ValueError):
    pass


def require(condition, name):
    if not condition:
        raise GuardError(name)


def digest(value):
    return hashlib.sha256(value).hexdigest()


def body(source, marker):
    require(source.count(marker) == 1, "unique source anchor: " + marker)
    start = source.index(marker)
    line_end = source.find("\n", start)
    opening = source.find("{", start, line_end)
    if opening < 0:
        match = re.search(r"(?m)^    \{$", source[line_end:])
        require(match is not None, "executable body anchor: " + marker)
        opening = line_end + match.start() + 4
    depth = 0
    quoted = False
    escaped = False
    line_comment = False
    block_comment = 0
    index = opening
    while index < len(source):
        character = source[index]
        pair = source[index:index + 2]
        if line_comment:
            line_comment = character != "\n"
        elif block_comment:
            if pair == "/*":
                block_comment += 1
                index += 1
            elif pair == "*/":
                block_comment -= 1
                index += 1
        elif quoted:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                quoted = False
        elif pair == "//":
            line_comment = True
            index += 1
        elif pair == "/*":
            block_comment = 1
            index += 1
        elif character == '"':
            quoted = True
        elif character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[opening:index + 1]
        index += 1
    raise GuardError("unclosed body: " + marker)


def validate_bodies(source, scope):
    for row in scope["bodies"]:
        region = body(source, row["owner"]) if row.get("owner") else source
        selected = body(region, row["marker"])
        if row["classification"] == "exact native contiguous block, not a whole native method":
            selected = selected[1:-1].strip()
        require(digest(selected.encode()) == row["sha256"], "native body: " + row["name"])


def validate_constants(source, scope):
    for declaration in scope["constant_declarations"]:
        require(source.count(declaration) == 1, "native constant: " + declaration.split(":")[0])


def validate_domains(source, scope):
    for row in scope["domain_anchors"]:
        require(source.count(row["text"]) == row["count"], "original domain: " + row["name"])


def validate_records(source, scope):
    for row in scope["records"]:
        selected = body(source, row["marker"])
        selected = re.sub(r"\bpub(?:\([^)]*\))?\s+", "", selected)
        require(digest(selected.encode()) == row["sha256"], "native record/error payloads: " + row["name"])
    for declaration in scope["derive_declarations"]:
        require(declaration in source, "native derive declaration: " + declaration.splitlines()[-1])


def validate_arguments(arguments):
    for argument in arguments:
        require(argument not in ("--import", "--extern")
                and not argument.startswith(("--import=", "--extern=")), "zero application import cardinality")


BOUNDARY_BODIES = (
    "original_leaf_domain", "original_clear_short_domain", "original_corrupt_long_domain",
    "invalid_index_two_phase", "invalid_index_corrupt_early", "total_swap_domain",
)


def validate_receipt(report, success, published, additional_bodies=()):
    require(report["crate_success"] is success, "crate verification result")
    functions = report["functions"]
    require(len(functions) == 5, "exact helper cardinality")
    for suffix in ("BtreeHeader::serialized_size", "PageNumber::serialized_size", "TransactionId::eq",
                   "TransactionId::new", "get_u64"):
        require(sum(function["function"].endswith("::" + suffix) for function in functions) == 1,
                "exact helper identity: " + suffix)
    for function in functions:
        require(function["mode"] == "exact" and function["required_checks_passed"] is True,
                "fresh exact helper checks")
        for name in ("soundness", "maximality", "exactness"):
            proof = function["proofs"][name]
            require(proof["required"] is True and proof["status"] == "proved", "fresh " + name)
    require(report["imported_inference_dependencies"] == [], "zero imported inference dependencies")
    require(report["interface_written"] is published, "Infer-aware publication")
    if success:
        selected = report["verification_scope"]
        require(selected["native_collection_completed"] is True
                and selected["complete_selected_scope_verified"] is True, "whole selected scope")
        expected = ("DatabaseError::from", "DatabaseHeader::primary_slot", "DatabaseHeader::secondary_slot",
                    "DatabaseHeader::swap_primary_slot", "TransactionId::new", "TransactionId::cmp",
                    "TransactionId::partial_cmp", "TransactionId::eq", "TransactionHeader::from_id_tail",
                    "UnrepairedDatabaseHeader::assemble_physical_slots",
                    "UnrepairedDatabaseHeader::select_primary_slot",
                    "UnrepairedDatabaseHeader::select_after_layout_context", "get_u64",
                    "decoded_id_selection_client", "BtreeHeader::clone", "DatabaseHeader::clone",
                    "PageNumber::clone", "TransactionHeader::clone", "TransactionId::clone")
        require(additional_bodies in ((), BOUNDARY_BODIES), "declared additional body scope")
        expected += additional_bodies
        native = selected.get("native_functions")
        require(isinstance(native, list) and all(isinstance(row, dict)
                and isinstance(row.get("function"), str) and isinstance(row.get("body_status"), str)
                and type(row.get("selected")) is bool for row in native), "native body metadata")
        require(len({row["function"] for row in native}) == len(native), "unique native body identities")
        leaf = next(function["function"] for function in functions if function["function"].endswith("::get_u64"))
        namespace = leaf.removesuffix("get_u64")
        for suffix in expected:
            matches = [row for row in native if row["function"] == namespace + suffix]
            require(len(matches) == 1 and matches[0]["body_status"] == "checked_exec_body"
                    and matches[0]["selected"] is True,
                    "selected caller/body: " + suffix)
        checked = [row for row in native if row["body_status"] in ("checked_exec_body", "checked_proof_body")]
        require({row["function"] for row in checked} == {namespace + suffix for suffix in expected},
                "complete checked body set")
        for field, status in (("checked_exec_bodies", "checked_exec_body"),
                              ("checked_proof_bodies", "checked_proof_body")):
            require(type(selected.get(field)) is int and selected[field] >= 0
                    and selected[field] == sum(row["body_status"] == status for row in native),
                    "checked body count: " + field)


def validate_scaffold(source, scope):
    require(digest(source.encode()) == scope["fixture_sha256"], "full fixture/scaffold identity")
    require(not re.search(r"\b(?:assert|assume|admit|axiom)\s*[(!]", source), "no authored proof plumbing/assumptions")


def validate_source(source, scope, native):
    for relative, expected in scope["native_files"].items():
        require(digest((native / relative).read_bytes()) == expected, "native full file: " + relative)
    validate_bodies(source, scope)
    validate_constants(source, scope)
    validate_domains(source, scope)
    validate_records(source, scope)
    validate_scaffold(source, scope)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--scope", type=Path, required=True)
    parser.add_argument("--native", type=Path, required=True)
    arguments = parser.parse_args()
    validate_source(arguments.fixture.read_text(), json.loads(arguments.scope.read_text()), arguments.native)
    print(json.dumps({"source_guards": "passed", "fixture_sha256": digest(arguments.fixture.read_bytes())}))


if __name__ == "__main__":
    main()
