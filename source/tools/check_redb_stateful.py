"""Offline, source-built replay of native bitmap transitions and complete reference."""

import argparse
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import subprocess
import sys
import time
from typing import Callable

sys.dont_write_bytecode = True

from bitmap_stateful_acceptance import (
    ReceiptError, mapping, require, unique_functions, validate_positive_run,
)
from infer_transfer_identity import INTERFACE_POLICY, INTERFACE_SCHEMA


FIXTURES = Path(__file__).resolve().with_name("stateful_bitmap")
REFERENCE88_FIXTURES = FIXTURES.with_name("stateful_bitmap_reference88")
NATIVE_REVISION = "c82ab865bb362bc4af067d332adc44978ff64813"
NATIVE_FILE = "src/tree_store/page_store/bitmap.rs"
NATIVE_SHA256 = "d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff"
TOOLCHAIN = "1.99.1-aarch64-apple-darwin"
EXPECTED_FRONTEND_BLOCKER = dict(
    classification="EXPECTED_FRONTEND_BLOCKER", source="required_words_native.rs",
    diagnostic="error: `core::num::impl&%8::div_ceil` is not supported",
    body="U64GroupedBitmap::required_words", case="required_words_blocker", theorem_credit=0)
BUILD_PACKAGES = ("rust_verify", "verus", "vstd_build", "verus_builtin",
                  "verus_builtin_macros", "verus_state_machines_macros")
RUNTIME_FILES = ("rust_verify", "verus", "vstd_build", "libverus_builtin.rlib",
                 "libverus_builtin_macros.dylib", "libverus_state_machines_macros.dylib",
                 "libvstd.rlib", "vstd.vir", "z3")


def digest(path):
    result = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1048576), b""):
            result.update(chunk)
    return result.hexdigest()


def identity(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def record(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def artifact(path):
    path = Path(path).absolute()
    require(path.is_file() and not path.is_symlink(), "missing regular artifact: " + str(path))
    return dict(path=str(path), sha256=digest(path))


def source_manifest(repo):
    repo = Path(repo).resolve()
    source = repo / "source"
    workspace = (source / "Cargo.toml").read_text()
    members = re.search(r"\bmembers\s*=\s*\[(.*?)\]", workspace, re.S)
    require(members is not None, "missing literal workspace members")
    roots = [source / member for member in re.findall(r'"([^"\n]+)"', members.group(1))]
    roots += [source / relative for relative in re.findall(r'\bpath\s*=\s*"([^"\n]+)"', workspace)]
    roots += [source / "vstd", source / ".cargo"]
    paths = {repo / "rust-toolchain.toml", source / "Cargo.toml", source / "Cargo.lock"}
    visited = set()
    while roots:
        root = roots.pop().resolve()
        require(root.is_relative_to(repo) and root.is_dir(), "nonlocal/missing source root: " + str(root))
        if root in visited:
            continue
        visited.add(root)
        for directory, subdirectories, filenames in os.walk(root):
            subdirectories[:] = sorted(name for name in subdirectories
                                       if name not in (".git", "target", "target-verus", "__pycache__"))
            require(not any((Path(directory) / name).is_symlink() for name in subdirectories),
                    "source directory symlink")
            for name in filenames:
                path = Path(directory) / name
                require(not path.is_symlink(), "source file symlink: " + str(path))
                paths.add(path)
                if name == "Cargo.toml":
                    for relative in re.findall(r'\bpath\s*=\s*"([^"\n]+)"', path.read_text()):
                        dependency = path.parent / relative
                        if dependency.is_dir():
                            roots.append(dependency)
    return {str(path.relative_to(repo)): digest(path) for path in sorted(paths)}


def git_read(checkout, *arguments):
    environment = dict(PATH="/usr/bin:/bin", GIT_OPTIONAL_LOCKS="0", GIT_CONFIG_NOSYSTEM="1",
                       GIT_CONFIG_GLOBAL=os.devnull)
    result = subprocess.run(["/usr/bin/git", "-C", str(checkout), *arguments],
                            env=environment, capture_output=True, text=True, check=True)
    return result.stdout.strip()


def native_binding(checkout):
    checkout = Path(checkout).resolve()
    require(git_read(checkout, "rev-parse", "HEAD") == NATIVE_REVISION, "wrong native revision")
    require(git_read(checkout, "status", "--porcelain", "--untracked-files=all") == "",
            "dirty native checkout")
    result = artifact(checkout / NATIVE_FILE)
    require(result["sha256"] == NATIVE_SHA256, "wrong native bitmap bytes")
    return dict(checkout=str(checkout), revision=NATIVE_REVISION, bitmap=result)


def full_source_manifest(repo):
    repo = Path(repo).resolve()
    names = git_read(repo, "ls-files", "--cached", "--others", "--exclude-standard", "-z").split("\0")
    paths = sorted({name for name in names if name})
    require(bool(paths), "empty full source inventory")
    result = {}
    for name in paths:
        path = repo / name
        require(path.is_relative_to(repo) and path.is_file() and not path.is_symlink(),
                "missing/nonregular full source input: " + name)
        result[name] = digest(path)
    return result


def tokens(text):
    text = re.sub(r"//[^\n]*|/\*.*?\*/", "", text, flags=re.S)
    return re.findall(r"[A-Za-z_][A-Za-z_0-9]*|[0-9]+|[^\s]", text)


def braced(text, marker):
    require(text.count(marker) == 1, "ambiguous body marker: " + marker)
    start = text.index("{", text.index(marker))
    depth = 1
    for position in range(start + 1, len(text)):
        depth += (text[position] == "{") - (text[position] == "}")
        if depth == 0:
            return text[start + 1:position]
    raise ReceiptError("unterminated body: " + marker)


def connect_transition(native, reference):
    helper = braced(reference, "pub fn bits_in_range(")
    require(tokens(helper) == tokens(braced(native, "fn bits_in_range(")), "native mask projection")
    start = reference.index("    let word_first_bit =")
    end = reference.index("    assert forall|", start)
    step = reference[start:end].replace("&mut data[", "&mut self.data[")
    require(tokens(step) == tokens(braced(native, "for word in start_word..=end_word")),
            "native transition projection")
    return dict(mask_tokens=tokens(helper), transition_tokens=tokens(step),
                limitation="Single growing-range word transition; not complete resize.")


def connect_access(native, reference, required_words):
    structure = "pub(crate) struct U64GroupedBitmap"
    require(braced(native, structure) == braced(reference, structure)
            == braced(required_words, structure), "native struct bytes")
    original = braced(native, "impl U64GroupedBitmap")
    annotated = braced(reference, "impl U64GroupedBitmap")
    bodies = {}
    for name in ("data_index_of", "select_mask", "len", "get", "set", "clear"):
        body = braced(original, "fn " + name + "(")
        require(body == braced(annotated, "fn " + name + "("), "native access body bytes: " + name)
        bodies[name] = body
    bodies["bits_in_range"] = braced(native, "fn bits_in_range(")
    require(bodies["bits_in_range"] == braced(reference, "fn bits_in_range("), "native access mask bytes")
    bodies["required_words"] = braced(original, "fn required_words(")
    require(bodies["required_words"] == braced(required_words, "fn required_words("),
            "native required_words bytes")
    return dict(native_bodies=bodies, checked_access_bodies=7, blocked_bodies=["required_words"],
                authoring="Explicit contracts, checked BV lemma, snapshots and forall proof; no Infer credit.")


@dataclass(frozen=True)
class CompleteResizeProvider:
    source: Path
    expected_report: dict
    resize_function: str
    project_native_resize: Callable[[str], str]
    enabled: bool = True
    audit_native: Callable = None


def reference88_report(section, crate):
    frozen = json.loads((REFERENCE88_FIXTURES / "expected.json").read_text())[section]
    executable = frozen.get("exec", frozen.get("native_exec", []) + frozen.get("authored_exec", []))
    bodies = {}
    for status, names in (("checked_exec_body", executable),
                          ("checked_proof_body", frozen.get("proof", [])),
                          ("specification_definition_or_declaration", frozen.get("spec", []))):
        for name in names:
            bodies[crate + "::" + name] = dict(body_status=status, selected=True,
                inference_candidate=False, automatically_derived=False,
                ordinary_contract=status != "specification_definition_or_declaration"
                    and not (section == "bridge" and name == "extremes"))
    return dict(verified=frozen["verified"], interface_written=False, helpers={},
                dependencies={}, trusted_models=[], bodies=bodies)


def project_reference88_resize(source):
    from stateful_bitmap_reference88.audit import tokens as native_tokens, extract, erase
    implementation = extract(native_tokens(source), ["impl", "U64GroupedBitmap"])
    body = extract(implementation, ["fn", "resize", "("])
    return " ".join(erase(body)[0])


def audit_reference88(native, source):
    from stateful_bitmap_reference88.audit import audit
    result = audit(native, source)
    require(result["all_nine_bodies"] is True and result["exact_struct"] is True,
            "complete reference nine-body native projection")
    return result


COMPLETE_RESIZE_PROVIDERS = {
    "reference88": CompleteResizeProvider(REFERENCE88_FIXTURES / "reference.rs",
        reference88_report("reference", "stateful_resize"), "stateful_resize::U64GroupedBitmap::resize",
        project_reference88_resize, enabled=False, audit_native=audit_reference88),
}


def connect_complete(provider, native):
    body = provider.expected_report["bodies"].get(provider.resize_function)
    require(body is not None and body["body_status"] == "checked_exec_body"
            and body["ordinary_contract"] is True, "complete resize exact body scope")
    projected = provider.project_native_resize(provider.source.read_text())
    wanted = braced(native, "pub fn resize(&mut self, new_len: u32, full: bool)")
    require(tokens(projected) == tokens(wanted), "complete resize native projection")
    full_projection = provider.audit_native(native, provider.source.read_text()) if provider.audit_native else None
    return dict(source=artifact(provider.source), expected_report=provider.expected_report,
                resize_function=provider.resize_function, native_tokens=tokens(projected),
                full_native_projection=full_projection)


def private_environment(output, rustup, sysroot, producer):
    build = output / "build"
    for name in ("home", "tmp", "cache", "cargo-home", "target", "rustup-home/toolchains"):
        (build / name).mkdir(parents=True, exist_ok=True)
    (build / "rustup-home/toolchains" / TOOLCHAIN).symlink_to(sysroot, target_is_directory=True)
    return dict(
        HOME=str(build / "home"), TMPDIR=str(build / "tmp"), XDG_CACHE_HOME=str(build / "cache"),
        CARGO_HOME=str(build / "cargo-home"), CARGO_TARGET_DIR=str(build / "target"),
        CARGO_BUILD_BUILD_DIR=str(build / "target"), CARGO_BUILD_JOBS="2", CARGO_NET_OFFLINE="true",
        RUSTUP_HOME=str(build / "rustup-home"), RUSTUP_TOOLCHAIN=TOOLCHAIN,
        RUSTC=str(sysroot / "bin/rustc"), RUSTDOC=str(sysroot / "bin/rustdoc"),
        RUSTC_BOOTSTRAP="1", RUST_MIN_STACK="20971520", VERUS_IN_VARGO="1",
        VARGO_BUILD_SHA=producer, VARGO_BUILD_VERSION="stateful-source", VARGO_TOOLCHAIN=TOOLCHAIN,
        RUSTFLAGS="--sysroot " + shlex.quote(str(sysroot)) +
                  " --cfg proc_macro_span --cfg procmacro2_semver_exempt --cfg span_locations --cfg verus_keep_ghost",
        PATH=str(rustup.parent) + ":/usr/bin:/bin:/usr/sbin:/sbin", GIT_OPTIONAL_LOCKS="0",
        GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1", PYTHONDONTWRITEBYTECODE="1",
        VERUS_Z3_PATH=str(output / "runtime/z3"),
        VERUS_EXTRA_ARGS="--sysroot " + shlex.quote(str(sysroot)))


def seed_cache(cache, destination):
    cache = Path(cache).resolve()
    require(cache.is_dir(), "cargo cache unavailable (offline; no downloads)")
    commands = []
    for name in ("registry", "git"):
        source = cache / name
        require(source.is_dir(), "missing cargo cache component: " + str(source))
        target = destination / name
        command = ["/bin/cp", "-cRL" if sys.platform == "darwin" else "-RL", str(source), str(target)]
        result = subprocess.run(command, check=True)
        commands.append(dict(argv=command, returncode=result.returncode))
        require(not any(path.is_symlink() for path in target.rglob("*")), "cargo cache must be copied")
    return commands


def check_imports(argv, imports):
    require(not any(value.startswith(("--import=", "--extern=", "-L")) for value in argv),
            "noncanonical import flags")
    require(argv.count("--import") == len(imports) == argv.count("--extern"), "import kind/count")
    for flag, kind in (("--import", "interface"), ("--extern", "library")):
        actual = [argv[position + 1] for position, value in enumerate(argv) if value == flag
                  and position + 1 < len(argv)]
        wanted = [name + "=" + dependency[kind]["path"] for name, dependency in imports.items()]
        require(actual == wanted, "import argument kind binding: " + flag)
    for dependency in imports.values():
        for kind in ("interface", "library"):
            require(artifact(dependency[kind]["path"]) == dependency[kind], "import artifact drift")


def check_context(context):
    require(source_manifest(Path(context["repo"])) == context["source_manifest"], "compiler source drift")
    if "full_source_manifest" in context:
        require(full_source_manifest(context["repo"]) == context["full_source_manifest"], "full source drift")
    require(native_binding(context["native"]["checkout"]) == context["native"], "native source drift")
    for item in context["guarded"]:
        require(artifact(item["path"]) == item, "guarded artifact drift: " + item["path"])
    execution = mapping(context.get("execution"), "execution context")
    require(set(execution) == {"rustup", "sysroot", "runtime", "output", "ready"}
            and type(execution["ready"]) is bool
            and all(type(execution[name]) is str and Path(execution[name]).is_absolute()
                    for name in ("rustup", "sysroot", "runtime", "output"))
            and Path(execution["sysroot"]).name == TOOLCHAIN, "execution context schema")
    guarded = {item["path"] for item in context["guarded"]}
    selected = {execution["rustup"], str(Path(execution["sysroot"]) / "bin/rustc")}
    if execution["ready"]:
        selected.update(str(Path(execution["runtime"]) / name) for name in RUNTIME_FILES)
    require(selected <= guarded, "selected execution inputs are not guarded")


def application_sources():
    filenames = dict(stateful_transition="transition.rs", stateful_generic="generic_no_instances.rs",
                     generic_off="generic_no_instances.rs", stateful_access="access_composition.rs",
                     access_negative_clear="negative_clear_remains_set.rs",
                     access_negative_mask="negative_mask_zero.rs", required_words_blocker="required_words_native.rs")
    filenames.update({name: name + ".rs" for name in ("wrong_branch", "false_frame", "bad_index", "missing_guard")})
    sources = {name: (FIXTURES / filename).read_text() for name, filename in filenames.items()}
    sources["stateful_consumer"] = composition_source(sources["stateful_transition"])
    sources["required_words"] = (FIXTURES / "required_words_native.rs").read_text()
    sources["stateful_bridge"] = (REFERENCE88_FIXTURES / "bridge.rs").read_text()
    sources["stateful_resize_consumer"] = reference88_import_source()
    disabled = {"generic_off", "stateful_access", "access_negative_clear", "access_negative_mask",
                "required_words_blocker", "required_words", "stateful_bridge", "stateful_resize_consumer"}
    return {name: dict(source_sha256=hashlib.sha256(source.encode()).hexdigest(), enabled=name not in disabled)
            for name, source in sources.items()}


def application_paths(context, case):
    require(type(case) is str and case in context["applications"], "unregistered application case")
    output = Path(context["execution"]["output"])
    application = output / "applications" / case
    run = output / "runs" / case
    return dict(input=application / "input.rs", stdout=run / "stdout", stderr=run / "stderr",
                interface=application / (case + ".vir"), library=application / ("lib" + case + ".rlib"))


def application_command(context, case, imports=None):
    paths, execution = application_paths(context, case), context["execution"]
    argv = [execution["rustup"], "run", TOOLCHAIN, str(Path(execution["runtime"]) / "verus"), str(paths["input"]),
            "--sysroot", execution["sysroot"], "--crate-type=lib", "--edition=2021", "--crate-name", case,
            "--no-cheating", "--compile", "--output-json", "--num-threads", "1",
            "--export", str(paths["interface"]), "-o", str(paths["library"])]
    if context["applications"][case]["enabled"]:
        argv += ["--infer-goals"]
    for name, dependency in (imports or {}).items():
        argv += ["--import", name + "=" + dependency["interface"]["path"],
                 "--extern", name + "=" + dependency["library"]["path"]]
    return argv


def bound_application(run, case, context, imports=None):
    paths = application_paths(context, case)
    require(context["execution"]["ready"] is True, "application runtime not qualified")
    require(run.get("argv") == application_command(context, case, imports), "application command binding")
    check_imports(run["argv"], imports or {})
    contents = {}
    for name in ("input", "stdout", "stderr"):
        binding = mapping(run.get(name + "_artifact"), "execution artifact schema: " + name)
        require(binding.get("path") == str(paths[name]) and artifact(paths[name]) == binding,
                "execution artifact binding: " + name)
        if name != "input":
            require(run.get(name) == binding["path"], "consumed artifact path: " + name)
        contents[name] = paths[name].read_bytes()
        require(hashlib.sha256(contents[name]).hexdigest() == binding["sha256"], "consumed artifact drift: " + name)
    require(hashlib.sha256(contents["input"]).hexdigest() == context["applications"][case]["source_sha256"],
            "registered application input")
    require(all(run.get(name) == str(paths[name]) for name in ("library", "interface")),
            "application output binding")
    return contents


def accept_positive(run, expected, context, imports=None):
    check_context(context)
    bound_application(run, expected["case"], context, imports)
    require(run.get("timeout") is False, "positive timeout")
    for name in ("input", "stdout", "stderr"):
        binding = run[name + "_artifact"]
        require(artifact(binding["path"]) == binding and run["artifacts"][name] == binding,
                "execution artifact binding: " + name)
    return validate_positive_run(run, expected)


def certificate_name(function):
    path = mapping(mapping(function, "certificate function").get("path"), "certificate path")
    crate = path.get("krate")
    if crate == "Core":
        prefix = "core"
    else:
        crate = mapping(crate, "certificate crate").get("Id")
        require(type(crate) is list and len(crate) == 2 and type(crate[0]) is str,
                "certificate crate identity")
        prefix = crate[0]
    segments = path.get("segments")
    require(type(segments) is list and all(type(segment) is str for segment in segments),
            "certificate function segments")
    return "::".join([prefix, *segments])


def bind_certificate(expected, certificate, producer):
    require(certificate.get("producer") == "stateful-source:" + TOOLCHAIN + ":" + producer,
            "certificate producer does not bind current source")
    require(type(certificate.get("schema")) is int and certificate["schema"] == INTERFACE_SCHEMA
            and certificate.get("policy") == INTERFACE_POLICY, "certificate schema/policy")
    require(all(certificate.get(key) is True for key in ("soundness", "maximality", "exactness")),
            "certificate guarantees")
    proofs = certificate.get("proofs")
    require(type(proofs) is list and all(type(row) is list and len(row) == 2 for row in proofs),
            "certificate proofs")
    names = [certificate_name(function) for function, proof in proofs]
    require(len(names) == len(set(names)) and set(names) == set(expected["helpers"]),
            "certificate helper set")
    require(all(type(proof) is dict and all(proof.get(key) is True
                for key in ("soundness", "maximality", "exactness")) for function, proof in proofs),
            "certificate helper guarantees")
    trust = certificate.get("trusted_models")
    require(type(trust) is list and all(type(row) is list and len(row) == 2
            and type(row[1]) is str and re.fullmatch(r"[0-9a-f]{64}", row[1]) for row in trust),
            "certificate trusted models")
    models = [dict(function=certificate_name(function), fingerprint=fingerprint)
              for function, fingerprint in trust]
    names = [row["function"] for row in models]
    require(len(names) == len(set(names)) and set(names) == {
        row["function"] for row in expected["trusted_models"]}, "certificate trusted model inventory")
    dependencies = certificate.get("dependencies")
    require(type(dependencies) is list and all(type(row) is dict and type(row.get("crate_id")) is int
                and type(row.get("fingerprint")) is str for row in dependencies), "certificate dependencies")
    require(len({row["crate_id"] for row in dependencies}) == len(dependencies)
            and {row["crate_id"]: row["fingerprint"] for row in dependencies} == expected["dependencies"],
            "certificate dependency binding")
    bound = dict(expected, trusted_models=models)
    return bound


def failure_expectation(case):
    return json.loads((FIXTURES / "failure_expected.json").read_text())[case]


def normalize_failure_inference(inference, input_path, trusted_models):
    inference = json.loads(json.dumps(mapping(inference, "failure inference schema")))
    metrics = mapping(inference.get("metrics"), "failure metrics")
    require(set(metrics) == {"generation_micros", "total_micros"}
            and all(type(value) is int and value >= 0 for value in metrics.values())
            and metrics["total_micros"] > 0, "failure fresh metrics")
    inference["metrics"] = dict(generation_micros=0, total_micros=0)
    helpers = unique_functions(inference.get("functions"), "failure helper schema")
    for helper in helpers.values():
        require(type(helper.get("candidate_id")) is str
                and re.fullmatch(r"[0-9a-f]{64}", helper["candidate_id"]), "failure candidate identity")
        helper["candidate_id"] = "@candidate"
        metrics = mapping(helper.get("metrics"), "failure helper metrics")
        for key, minimum in (("solver_micros", 1), ("generation_micros", 0)):
            require(type(metrics.get(key)) is int and metrics[key] >= minimum, "failure helper fresh work")
            metrics[key] = 0
    models = inference.get("trusted_models")
    require(type(models) is list, "failure trust schema")
    for model in models:
        require(type(model) is dict and set(model) == {"function", "fingerprint"}
                and type(model["function"]) is str and model["function"] in trusted_models
                and model["fingerprint"] == trusted_models[model["function"]], "failure trusted model binding")
        model["fingerprint"] = "@trusted"
    return json.loads(json.dumps(inference).replace(str(input_path), "@input"))


def failure_diagnostics(stderr, input_path):
    headers = re.findall(r"(?m)^error[^\n]*", stderr)
    blocks = re.split(r"(?m)^error: ", stderr)[1:]
    require(len(headers) == len(blocks), "failure diagnostic format")
    return [dict(message=block.splitlines()[0], locations=[location.replace(str(input_path), "@input")
                for location in re.findall(r"(?m)^\s*(?:-->|:::) (.*?:\d+:\d+)\s*$", block)]) for block in blocks]


def accept_failure(run, case, context):
    check_context(context)
    contents = bound_application(run, case, context)
    expected = failure_expectation(case)
    require(type(run.get("returncode")) is int and run["returncode"] == 1
            and run.get("timeout") is False, "failure did not fail normally")
    for name in ("interface", "library"):
        path = Path(run[name])
        require(not path.exists() and not path.is_symlink(), "failure published " + name)
    stderr = contents["stderr"].decode()
    require(not any(message in stderr.lower() for message in
            ("panicked at", "timed out", "resource limit", "internal compiler error")), "failure infrastructure error")
    report = json.loads(contents["stdout"])
    require(identity(mapping(report.get("verification-results"), "failure results schema"))
            == identity(expected["verification"]), "failure exact results")
    normalized = normalize_failure_inference(report.get("verus-infer"), run["input_artifact"]["path"],
                                            context["trusted_models"])
    require(identity(normalized) == identity(expected["inference"]), "failure exact inference schema")
    require(failure_diagnostics(stderr, run["input_artifact"]["path"]) == expected["diagnostics"],
            "failure intended diagnostics")
    return report


def accept_negative(run, kind, context, expected_bodies):
    cases = {name.split("::", 1)[0] for name in expected_bodies}
    require(len(cases) == 1, "negative expected case")
    case = next(iter(cases))
    require(failure_expectation(case)["kind"] == kind, "negative registered kind")
    return accept_failure(run, case, context)


def accept_frontend_blocker(run, expected, context):
    require(expected == EXPECTED_FRONTEND_BLOCKER, "unregistered frontend blocker")
    return accept_failure(run, expected["case"], context)


class Replay:
    def __init__(self, repo, native, output, rustup, sysroot, cargo_cache, solver):
        self.repo, self.output = Path(repo).resolve(), Path(output).resolve()
        require(not self.output.exists(), "output must be new")
        require(not any(self.output.is_relative_to(Path(path).resolve())
                        for path in (self.repo, native, cargo_cache, sysroot)),
                "output must be outside source/native/cache/toolchain inputs")
        self.output.mkdir(parents=True)
        self.rustup, self.sysroot = Path(rustup).absolute(), Path(sysroot).resolve()
        require(self.rustup.is_file() and self.sysroot.name == TOOLCHAIN, "canonical local toolchain required")
        require(re.search(r'^channel\s*=\s*"1\.99\.1"\s*$',
                          (self.repo / "rust-toolchain.toml").read_text(), re.M) is not None,
                "source toolchain drift")
        manifest = source_manifest(self.repo)
        self.producer = "source-" + identity(manifest)
        self.context = dict(repo=str(self.repo), source_manifest=manifest, native=native_binding(native),
                            full_source_manifest=full_source_manifest(self.repo),
                            execution=dict(rustup=str(self.rustup), sysroot=str(self.sysroot),
                                           runtime=str(self.output / "runtime"), output=str(self.output), ready=False),
                            applications=application_sources(), trusted_models={},
                            guarded=[artifact(path) for path in sorted(FIXTURES.iterdir()) if path.is_file()]
                            + [artifact(path) for path in sorted(Path(__file__).parent.glob("*.py"))])
        toolchain_paths = [path for directory in (self.sysroot / "bin", self.sysroot / "lib",
                            self.sysroot / "lib/rustlib/aarch64-apple-darwin/lib")
                           for path in directory.iterdir() if path.is_file()]
        self.context["guarded"] += [artifact(path) for path in sorted(toolchain_paths)]
        self.context["guarded"] += [artifact(self.rustup), artifact(solver)]
        record(self.output / "source-manifest.json", manifest)
        record(self.output / "full-source-manifest.json", self.context["full_source_manifest"])
        record(self.output / "native.json", self.context["native"])
        self.environment = private_environment(self.output, self.rustup, self.sysroot, self.producer)
        cache_commands = seed_cache(cargo_cache, Path(self.environment["CARGO_HOME"]))
        record(self.output / "setup.json", dict(cache_seed=str(Path(cargo_cache).resolve()),
                                                cache_commands=cache_commands,
                                                environment=self.environment,
                                                toolchain_inputs=len(toolchain_paths)))
        self.runtime = self.output / "runtime"
        self.runtime.mkdir()
        shutil.copyfile(solver, self.runtime / "z3")
        (self.runtime / "z3").chmod(0o755)
        self.counts = dict(build=0, vstd=0, application=0)
        self.runs = []

    def execute(self, name, argv, category, environment=None, timeout=240):
        limits = dict(build=1, vstd=1, application=24)
        require(self.counts[category] < limits[category], "execution budget exhausted: " + category)
        self.counts[category] += 1
        directory = self.output / "runs" / name
        directory.mkdir(parents=True)
        invocation = dict(argv=[str(value) for value in argv], cwd=str(self.repo / "source"),
                          environment=environment or self.environment, timeout_seconds=timeout,
                          category=category, source_manifest_sha256=identity(self.context["source_manifest"]),
                          producer=self.producer, native=self.context["native"])
        record(directory / "invocation.json", invocation)
        record(self.output / "launch-counts.json", self.counts)
        started = time.monotonic()
        run = dict(argv=invocation["argv"], timeout=False, returncode=None,
                   stdout=str(directory / "stdout"), stderr=str(directory / "stderr"))
        with Path(run["stdout"]).open("wb") as stdout, Path(run["stderr"]).open("wb") as stderr:
            try:
                process = subprocess.Popen(invocation["argv"], cwd=invocation["cwd"],
                                           env=invocation["environment"], stdout=stdout, stderr=stderr,
                                           start_new_session=True)
                try:
                    run["returncode"] = process.wait(timeout=timeout)
                except subprocess.TimeoutExpired:
                    run["timeout"] = True
                    os.killpg(process.pid, signal.SIGKILL)
                    run["returncode"] = process.wait()
            except OSError as error:
                run["spawn_error"] = str(error)
        run.update(seconds=time.monotonic() - started, stdout_artifact=artifact(run["stdout"]),
                   stderr_artifact=artifact(run["stderr"]))
        record(directory / "result.json", run)
        self.runs.append(dict(name=name, invocation=artifact(directory / "invocation.json"),
                              result=artifact(directory / "result.json")))
        print(name + ": " + str(run["returncode"]), flush=True)
        return run

    def build(self):
        argv = [self.rustup, "run", TOOLCHAIN, self.sysroot / "bin/cargo", "build", "--offline",
                "--locked", "--release", "--message-format=json"]
        for package in BUILD_PACKAGES:
            argv += ["-p", package]
        run = self.execute("compiler-build", argv, "build", timeout=1800)
        require(run["returncode"] == 0 and not run["timeout"], "offline compiler build failed")
        check_context(self.context)
        for name in RUNTIME_FILES:
            if name not in ("libvstd.rlib", "vstd.vir", "z3"):
                shutil.copy2(Path(self.environment["CARGO_TARGET_DIR"]) / "release" / name,
                             self.runtime / name)
        environment = dict(self.environment, VSTD_KIND="IsVstd")
        environment.pop("VARGO_BUILD_SHA")
        environment.pop("VARGO_TOOLCHAIN")
        argv = [self.rustup, "run", TOOLCHAIN, self.runtime / "rust_verify", "--internal-test-mode",
                "--sysroot", self.sysroot, "--crate-type=lib", "--export", self.runtime / "vstd.vir",
                "--out-dir", self.runtime, "--multiple-errors", "2", "--is-vstd", "--compile",
                "-C", "opt-level=3"]
        for name, filename in (("verus_builtin", "libverus_builtin.rlib"),
                               ("verus_builtin_macros", "libverus_builtin_macros.dylib"),
                               ("verus_state_machines_macros", "libverus_state_machines_macros.dylib")):
            argv += ["--extern", name + "=" + str(self.runtime / filename)]
        for feature in ("std", "alloc", "nonzero_internals"):
            argv += ["--cfg", 'feature="' + feature + '"']
        argv += ["vstd/vstd.rs"]
        run = self.execute("matched-vstd", argv, "vstd", environment, timeout=900)
        require(run["returncode"] == 0 and not run["timeout"]
                and re.search(r"verification results:: [1-9][0-9]* verified, 0 errors",
                              Path(run["stdout"]).read_text()) is not None, "matched vstd failed")
        self.context["guarded"] += [artifact(self.runtime / name) for name in RUNTIME_FILES]
        self.context["execution"]["ready"] = True
        check_context(self.context)
        record(self.output / "build-receipt.json", dict(context=self.context, runs=self.runs,
                                                        producer=self.producer))

    def application(self, name, source, expected_report=None, enabled=True, imports=None, negative=None,
                    frontend_blocker=None):
        check_context(self.context)
        directory = self.output / "applications" / name
        directory.mkdir(parents=True)
        input_path = directory / "input.rs"
        input_path.write_text(source)
        library, interface = directory / ("lib" + name + ".rlib"), directory / (name + ".vir")
        argv = [str(self.rustup), "run", TOOLCHAIN, str(self.runtime / "verus"), str(input_path),
                "--sysroot", str(self.sysroot), "--crate-type=lib", "--edition=2021", "--crate-name", name,
                "--no-cheating", "--compile", "--output-json", "--num-threads", "1",
                "--export", str(interface), "-o", str(library)]
        if enabled:
            argv += ["--infer-goals"]
        for crate, dependency in (imports or {}).items():
            argv += ["--import", crate + "=" + dependency["interface"]["path"],
                     "--extern", crate + "=" + dependency["library"]["path"]]
        check_imports(argv, imports or {})
        environment = dict(self.environment, VERUS_INFER_CACHE=str(directory / "candidate-cache"))
        require("VSTD_KIND" not in environment, "application inherited vstd kind")
        input_binding = artifact(input_path)
        run = self.execute(name, argv, "application", environment)
        require(artifact(input_path) == input_binding, "application input drift during execution")
        run.update(library=str(library), interface=str(interface), input_artifact=input_binding)
        if frontend_blocker:
            report = accept_frontend_blocker(run, frontend_blocker, self.context)
            run.update(classification="EXPECTED_FRONTEND_BLOCKER", theorem_credit=0)
            record(self.output / "runs" / name / "expectation.json", frontend_blocker)
        elif negative:
            report = accept_negative(run, negative, self.context, expected_report["bodies"])
            record(self.output / "runs" / name / "expectation.json",
                   dict(argv=argv, failure_kind=negative, body_scope=expected_report["bodies"]))
        else:
            run["artifacts"] = {key: artifact(path) for key, path in
                                dict(input=input_path, stdout=run["stdout"], stderr=run["stderr"],
                                     library=library, interface=interface).items()}
            if expected_report["helpers"] or expected_report["interface_written"]:
                from infer_transfer_receipts import metadata
                certificate, fingerprint = metadata(interface.read_bytes())
                expected_report = bind_certificate(expected_report, certificate, self.producer)
                run.update(certificate=certificate, fingerprint=fingerprint)
            expected = dict(case=name, argv=argv, artifacts=run["artifacts"], report=expected_report)
            record(self.output / "runs" / name / "expectation.json", expected)
            report = accept_positive(run, expected, self.context, imports)
            if name == "stateful_transition":
                self.context["trusted_models"] = {model["function"]: model["fingerprint"]
                                                 for model in expected_report["trusted_models"]}
        run["report"] = report
        record(self.output / "runs" / name / "accepted.json", run)
        self.runs[-1]["accepted"] = artifact(self.output / "runs" / name / "accepted.json")
        self.runs[-1]["expectation"] = artifact(self.output / "runs" / name / "expectation.json")
        return run


def composition_source(reference):
    signature = reference[reference.index("pub fn native_step("):reference.index("{\n    let ghost before")]
    wrapper = signature.replace("pub fn native_step(", "pub fn transition_client(")
    return "use vstd::prelude::*;\nuse stateful_transition::*;\nverus! {\n" + wrapper + """{
    native_step(data, Ghost(entry), old_len, new_len, word, full);
}
pub fn mask_client(lo: u32, hi: u32) -> (result: u64)
    requires lo < hi, hi <= 64,
    ensures result == (u64::MAX << lo) & (u64::MAX >> ((64 - hi) as u32)),
{
    bits_in_range(lo, hi)
}
}
"""


def reference88_import_source():
    return (REFERENCE88_FIXTURES / "import.rs").read_text().replace("reference88", "stateful_resize")


def access_cases(runner, include_required_words=True):
    expected = json.loads((FIXTURES / "access_expected.json").read_text())
    native = Path(runner.context["native"]["bitmap"]["path"]).read_text()
    connection = connect_access(native, (FIXTURES / "access_composition.rs").read_text(),
                                (FIXTURES / "required_words_native.rs").read_text())
    record(runner.output / "access-native-projection.json", connection)
    runner.application("stateful_access", (FIXTURES / "access_composition.rs").read_text(),
                       expected["access"], enabled=False)
    for name, filename, base in (("access_negative_clear", "negative_clear_remains_set.rs", "access"),
                                 ("access_negative_mask", "negative_mask_zero.rs", "access_negative_mask")):
        bodies = {key.replace("stateful_access::", name + "::"): value
                  for key, value in expected[base]["bodies"].items()}
        runner.application(name, (FIXTURES / filename).read_text(), dict(bodies=bodies),
                           enabled=False, negative="frame")
    if include_required_words:
        helper = dict(verified=1, interface_written=False, helpers={}, dependencies={}, trusted_models=[],
                      bodies={"required_words::U64GroupedBitmap::required_words": dict(
                          body_status="checked_exec_body", selected=True, inference_candidate=False,
                          ordinary_contract=False, automatically_derived=False)})
        runner.application("required_words", (FIXTURES / "required_words_native.rs").read_text(),
                           helper, enabled=False)
    return dict(access_established=True, expected_frontend_blockers=[],
                required_words_established=include_required_words,
                native_projection=artifact(runner.output / "access-native-projection.json"))


def replay(runner, complete_provider=None):
    expectations = json.loads((FIXTURES / "expected.json").read_text())
    source = (FIXTURES / "transition.rs").read_text()
    native = Path(runner.context["native"]["bitmap"]["path"]).read_text()
    connection = connect_transition(native, source)
    record(runner.output / "native-projection.json", connection)
    if complete_provider is not None:
        complete_source = complete_provider.source.read_text()
        connect_complete(complete_provider, native)
        runner.context["applications"]["stateful_resize"] = dict(
            source_sha256=hashlib.sha256(complete_source.encode()).hexdigest(), enabled=complete_provider.enabled)
        runner.context["guarded"].append(artifact(complete_provider.source))
    runner.build()
    provider = runner.application("stateful_transition", source, expectations["transition"])
    runner.application("stateful_generic", (FIXTURES / "generic_no_instances.rs").read_text(),
                       expectations["generic"])
    for name, kind in (("wrong_branch", "semantic"), ("false_frame", "frame"),
                       ("bad_index", "safety"), ("missing_guard", "semantic")):
        expected = dict(bodies={key.replace("stateful_transition::", name + "::"): value
                                for key, value in expectations["transition"]["bodies"].items()})
        runner.application(name, (FIXTURES / (name + ".rs")).read_text(), expected, negative=kind)
    expected = dict(bodies={key.replace("stateful_generic::", "generic_off::"): value
                            for key, value in expectations["generic"]["bodies"].items()})
    runner.application("generic_off", (FIXTURES / "generic_no_instances.rs").read_text(),
                       expected, enabled=False, negative="ablation")
    imports = {"stateful_transition": {key: provider["artifacts"][key] for key in ("interface", "library")}}
    from infer_transfer_receipts import metadata
    _, fingerprint = metadata(Path(imports["stateful_transition"]["interface"]["path"]).read_bytes())
    identities = {entry[0]["path"]["krate"]["Id"][1] for entry in provider["certificate"]["proofs"]}
    require(len(identities) == 1, "ambiguous provider crate identity")
    consumer = dict(verified=3, interface_written=True, helpers={},
                    bodies={"stateful_consumer::" + name: dict(body_status="checked_exec_body", selected=True,
                            inference_candidate=False, ordinary_contract=True, automatically_derived=False)
                            for name in ("transition_client", "mask_client")},
                    dependencies={next(iter(identities)): fingerprint},
                    trusted_models=expectations["transition"]["trusted_models"])
    composition = runner.application("stateful_consumer", composition_source(source), consumer, imports=imports)
    access = access_cases(runner, include_required_words=complete_provider is None)
    runner.application("stateful_bridge", (REFERENCE88_FIXTURES / "bridge.rs").read_text(),
                       reference88_report("bridge", "stateful_bridge"), enabled=False)
    complete = None
    if complete_provider is not None:
        complete = connect_complete(complete_provider, native)
        record(runner.output / "complete-provider.json", complete)
        full = runner.application("stateful_resize", complete_provider.source.read_text(),
                                  complete_provider.expected_report, enabled=complete_provider.enabled)
        if complete_provider is COMPLETE_RESIZE_PROVIDERS["reference88"]:
            ordinary_imports = {"stateful_resize": {key: full["artifacts"][key]
                                for key in ("interface", "library")}}
            runner.application("stateful_resize_consumer", reference88_import_source(),
                reference88_report("import", "stateful_resize_consumer"), enabled=False, imports=ordinary_imports)
            complete["ordinary_import_composition"] = True
            access["required_words_established"] = True
    check_context(runner.context)
    receipt = dict(status="complete_resize_established" if complete else "transition_access_established_resize_incomplete",
                   transition_established=True, full_resize_complete=complete is not None,
                   access_established=access["access_established"], access=access,
                   composition_established=composition["returncode"] == 0,
                   context=runner.context, producer=runner.producer, runs=runner.runs, launches=runner.counts,
                   complete_provider=complete, native_projection=artifact(runner.output / "native-projection.json"),
                   build=artifact(runner.output / "build-receipt.json"),
                   trust="Matched vstd retains its existing audited external bodies/axioms; bootstrap is not no-cheating. "
                         "It includes the explicitly approved u32::div_ceil correspondence, not a proof of Rust core. "
                         "Applications are no-cheating. Contracts, lemmas, snapshots and loop proofs are authored. "
                         "Reference88 composition imports ordinary contracts, not certified inferred contracts.")
    record(runner.output / "receipt.json", receipt)
    return receipt


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cargo-cache", type=Path, default=Path.home() / ".cargo")
    parser.add_argument("--rustup", type=Path, default=Path.home() / ".cargo/bin/rustup")
    parser.add_argument("--sysroot", type=Path, default=Path.home() / ".rustup/toolchains" / TOOLCHAIN)
    parser.add_argument("--solver", type=Path)
    parser.add_argument("--resize-provider", choices=tuple(COMPLETE_RESIZE_PROVIDERS))
    parser.add_argument("--transition-only", action="store_true",
                        help="exit zero for the explicitly limited transition track, without claiming full resize")
    args = parser.parse_args(argv)
    repo = Path(__file__).resolve().parents[2]
    runner = None
    new_output = not args.output.exists()
    try:
        runner = Replay(repo, args.native_checkout, args.output, args.rustup, args.sysroot,
                        args.cargo_cache, args.solver or repo / "source/z3")
        receipt = replay(runner, COMPLETE_RESIZE_PROVIDERS.get(args.resize_provider))
        print(receipt["status"] + ": " + str(args.output / "receipt.json"))
        return 0 if receipt["full_resize_complete"] or args.transition_only else 2
    except (ReceiptError, OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        if runner is not None:
            record(runner.output / "failure.json", dict(status="failed", error=str(error),
                                                        launches=runner.counts, runs=runner.runs))
        elif new_output and args.output.is_dir():
            record(args.output / "failure.json", dict(status="setup_failed", error=str(error),
                                                       launches=dict(build=0, vstd=0, application=0)))
        print("stateful replay failed: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
