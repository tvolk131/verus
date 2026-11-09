"""Replay the pinned pure bitmap interval mask, not allocator state or resize."""
import argparse
import hashlib
import json
import os
from pathlib import Path

from check_redb_access_prefix import classify
from check_redb_page_size import APPLICATION, item
from infer_transfer_identity import ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (
    LoggedCommands, positive_timeout, replay_receipt, require, require_exact,
    require_publication, snapshot_inputs, validate_run,
)


ROOT = Path(__file__).resolve().parents[2]
BITMAP = "src/tree_store/page_store/bitmap.rs"
SOURCE_HASH = "d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff"
ITEM_HASH = "ec376aba7d0f4d7c1cc1e1054f13c3c045174efe3ea9d2c78125b0cfee935fa5"
HEADER = "use vstd::prelude::*;\nverus! {\n"
MEMBER = "((result>>bit)&1u64==1u64) == (lo<=bit && bit<hi)"
METADATA_DIAGNOSTICS = {
    "stale_policy": "incompatible inference schema, safety policy, or producer",
    "false_exactness": "incomplete inference proof guarantees",
    "wrong_summary": "inferred contract fingerprint mismatch",
    "wrong_model": "stale or missing trusted-model fingerprint",
}


def import_guard(artifact, dependency):
    require(artifact.resolve() != dependency.resolve(), "import and export paths must differ")
    require(dependency.is_file(), "import fixture is missing")
    return ArtifactGuard([dependency])


def failure_diagnostic(expected, stderr, diagnostic=None):
    if diagnostic is not None:
        require(diagnostic in stderr, "intended rejection diagnostic is missing")
    if expected == "resource_or_runtime_failure":
        require("resource limit" in stderr.lower() and "panicked at" not in stderr.lower()
                and "timed out" not in stderr.lower(), "not an intentional solver-budget failure")


def requirements(debug):
    return "lo<hi,hi<=64,bit<64" if debug else "lo<64,1<=hi,hi<=64,bit<64"


def membership(debug, call="bits_in_range(lo,hi)"):
    return ("fn membership(lo:u32,hi:u32,bit:u32)->(result:u64)\n requires "
            + requirements(debug) + ",\n ensures " + MEMBER + ",\n{" + call + "}\n")


def update(debug):
    return ("fn update(word:u64,full:bool,lo:u32,hi:u32,bit:u32)->(result:u64)\n requires "
            + requirements(debug) + ",\n ensures ((result>>bit)&1u64==1u64) == "
            + "(if lo<=bit && bit<hi {full} else {(word>>bit)&1u64==1u64}),\n"
            + "{let mask=bits_in_range(lo,hi); if full {word | mask} else {word & !mask}}\n")


def extract(application, output):
    source = application / BITMAP
    require(digest(source) == SOURCE_HASH, "pinned bitmap.rs changed")
    text = source.read_text()
    helper = item(text, "fn bits_in_range(") + "\n"
    require(hashlib.sha256(helper.encode()).hexdigest() == ITEM_HASH,
            "pinned interval helper changed")
    (output / "bitmap.original.rs").write_bytes(source.read_bytes())
    (output / "bits_in_range.original.rs").write_text(helper)
    return "#[verifier::infer]\npub " + helper, dict(
        application_commit=APPLICATION, source=BITMAP, source_sha256=SOURCE_HASH,
        item_sha256=ITEM_HASH, line=text[:text.index(helper.rstrip())].count("\n") + 1,
        adapter="Add pub visibility and verifier::infer; executable body is unchanged",
        next_boundary="U64GroupedBitmap::resize needs length/data consistency and checked index/word arithmetic; no occupancy or storage contract assumed",
        update="Authored by-value observation of the resize OR/AND expressions; not extraction or verification of resize")


def native(helper):
    return helper.replace("#[verifier::infer]\n", "") + r'''
fn main() {
    let mut intervals=0u64;
    let mut observations=0u64;
    for lo in 0u32..64 { for hi in 1u32..=64 {
        if cfg!(debug_assertions) && lo>=hi {continue;}
        let mask=bits_in_range(lo,hi);
        for bit in 0u32..64 {
            assert_eq!(((mask>>bit)&1)==1,lo<=bit && bit<hi);
            for word in [0u64,1,0x5555_5555_5555_5555,u64::MAX] {
                for full in [false,true] {
                    let changed=if full {word|mask} else {word & !mask};
                    let expected=if lo<=bit && bit<hi {full} else {((word>>bit)&1)==1};
                    assert_eq!(((changed>>bit)&1)==1,expected);
                    observations+=1;
                }
            }
        }
        intervals+=1;
    }}
    println!("intervals={intervals}, observations={observations}");
}
'''


def run_suite(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    environment = dict(os.environ, GIT_OPTIONAL_LOCKS="0", PYTHONDONTWRITEBYTECODE="1",
                       VERUS_Z3_PATH=str(args.solver.resolve()),
                       VERUS_INFER_CACHE=str(output / "candidate-cache"))
    receipt = dict(status="blocked", commands=[], cases=[], identity=list(ENGINE_IDENTITY),
                   trust=["Original Rust/Verus frontend, ordinary backend and solver",
                          "Existing vstd requires-false panic model; no new axioms",
                          "Transparent scalar helper only; no bitmap representation or allocator invariant"])
    commands = LoggedCommands(output, ROOT, receipt["commands"], timeout=args.timeout)
    with replay_receipt(output, receipt):
        for label, argv, expected in [("pin", ["rev-parse", "HEAD"], APPLICATION),
                                      ("clean", ["status", "--porcelain", "--untracked-files=all"], "")]:
            require(commands("application_before_" + label,
                             ["git", "-C", str(args.application), *argv], environment) == 0)
            require((output / ("application_before_" + label + ".stdout")).read_text().strip() == expected)
        helper, receipt["extraction"] = extract(args.application, output)
        require(commands("engine_files", ["git", "ls-files", "--cached", "--others", "--exclude-standard",
                                          "--", "source", "dependencies", "rust-toolchain.toml"], environment) == 0)
        inputs = sorted(set((output / "engine_files.stdout").read_text().splitlines()))
        receipt.update(snapshot_inputs(ROOT, output / "inputs", inputs))
        guard = ArtifactGuard(tool_artifacts(args.verus.resolve(), args.solver.resolve(),
                                             extra=[ROOT / path for path in inputs] + [args.application / BITMAP]))
        commands.guard = guard
        receipt["artifacts"] = guard.initial
        provider_guard = None

        def verify(label, source, debug=True, expected="proved", inferred=1,
                   dependency=None, seed=None, goals=True, compile=False, extra=(), source_label=None,
                   diagnostic=None):
            source_label = source_label or label
            path = output / (source_label + ".rs")
            path.write_text(source)
            artifact = output / (label + ".vir")
            dependency_guard = import_guard(artifact, dependency) if dependency is not None else None
            if seed is not None:
                artifact.write_bytes(seed)
            command = [str(args.verus.resolve()), str(path), "--crate-type=lib", "--crate-name", source_label,
                       "--output-json", "--no-cheating", "--num-threads", "2", "--export", str(artifact),
                       "-o", str(output / ("lib" + label + ".rlib")),
                       "-C", "debug-assertions=" + ("yes" if debug else "no")]
            if compile:
                command.append("--compile")
            if goals:
                command.append("--infer-goals")
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
            stdout = (output / (label + ".stdout")).read_text()
            stderr = (output / (label + ".stderr")).read_text()
            data = json.loads(stdout)
            observed = classify(data, stderr, code)
            receipt["cases"].append(dict(label=label, classification=observed, report=data,
                                         source_sha256=digest(path), debug_assertions=debug,
                                         stale_export_seeded=seed is not None, artifact_exists=artifact.exists()))
            require(observed == expected, label + ": " + observed + "\n" + stderr[-2500:])
            failure_diagnostic(expected, stderr, diagnostic)
            if expected != "resource_or_runtime_failure":
                validate_run(dict(returncode=code, stdout=stdout, stderr=stderr), expected == "proved",
                             allow_frontend=expected == "frontend_rejection")
            if expected not in ("frontend_rejection", "resource_or_runtime_failure"):
                functions = data["verus-infer"]["functions"]
                require(len(functions) == inferred, "unexpected inference scope")
                require_exact(functions)
                require(data["verification-results"].get("is-verifying-entire-crate") is True)
                if expected == "semantic_goal_failure":
                    require("precondition not satisfied" not in stderr and "proposal not satisfied" not in stderr)
            require_publication(data, expected == "proved", artifact)
            return artifact

        provider = HEADER + helper + "}\n"
        seed = verify("provider", provider, compile=True).read_bytes()
        provider_guard = ArtifactGuard([output / "provider.vir", output / "libprovider.rlib"])
        receipt["provider_artifacts"] = provider_guard.initial
        for debug in (True, False):
            mode = "debug" if debug else "release"
            good = HEADER + helper + membership(debug) + update(debug) + "}\n"
            verify("membership_" + mode, good, debug, compile=True)
            verify("warm_" + mode, good, debug, source_label="membership_" + mode)
            for cache in (output / "candidate-cache").glob("*.json"):
                cache.write_text('{"poisoned":true}')
            verify("poisoned_" + mode, good, debug, source_label="membership_" + mode)
            for name, lo, hi in [("lo64", 64, 64), ("hi0", 0, 0), ("hi65", 0, 65),
                                 ("hi_max", 0, 4294967295), ("reversed", 10, 3), ("equal", 3, 3)]:
                expected = "proved" if not debug and name in ("reversed", "equal") else "safety_failure"
                source = HEADER + helper + f"fn boundary()->u64 {{bits_in_range({lo},{hi})}}\n}}\n"
                verify(mode + "_" + name, source, debug, expected, seed=seed if expected != "proved" else None,
                       goals=False)
            source = HEADER + helper + "fn false_mask()->(result:u64) ensures result==0, {bits_in_range(0,64)}\n}\n"
            verify("false_mask_" + mode, source, debug, "semantic_goal_failure", seed=seed, goals=False)
            mutant = helper.replace("bits_at_or_above_lo & bits_below_hi", "bits_at_or_above_lo ^ bits_below_hi")
            require(mutant != helper)
            source = HEADER + mutant + "fn authored_goal()->(result:u64) ensures result==u64::MAX, {bits_in_range(0,64)}\n}\n"
            verify("semantic_mutant_" + mode, source, debug, "semantic_goal_failure", seed=seed, goals=False)
            (output / ("native_" + mode + ".rs")).write_text(native(helper))
            require(commands("native_compile_" + mode, ["rustup", "run", "1.99.1", "rustc", "--edition=2021",
                             "-C", "opt-level=3", "-C", "debug-assertions=" + ("yes" if debug else "no"),
                             str(output / ("native_" + mode + ".rs")), "-o", str(output / ("native_" + mode))], environment) == 0)
            require(commands("native_run_" + mode, [str(output / ("native_" + mode))], environment) == 0)
        relay = "#[verifier::infer]\npub fn relay(lo:u32,hi:u32)->u64 {provider::bits_in_range(lo,hi)}\n"
        client = HEADER + relay + membership(True, "relay(lo,hi)") + "}\n"
        verify("imported", client, dependency=output / "provider.vir")
        verify("imported_warm", client, dependency=output / "provider.vir")
        footer = seed.rindex(b'{"schema":')
        metadata = json.loads(seed[footer:-8])
        require(metadata["schema"] == INTERFACE_SCHEMA and metadata["policy"] == INTERFACE_POLICY)
        for fault in ("stale_policy", "false_exactness", "wrong_summary", "wrong_model"):
            changed = json.loads(json.dumps(metadata))
            if fault == "stale_policy":
                changed["policy"] = "candidate-bound-evidence-v68"
            elif fault == "false_exactness":
                changed["exactness"] = False
            elif fault == "wrong_summary":
                changed["summaries"] = "corrupt"
            else:
                changed["trusted_models"][0][1] = "corrupt"
            dependency = output / ("import_" + fault + ".vir")
            dependency.write_bytes(seed[:footer] + json.dumps(changed, separators=(",", ":")).encode() + b"VRINFER1")
            verify(fault, client, expected="frontend_rejection", inferred=0, dependency=dependency, seed=seed,
                   diagnostic=METADATA_DIAGNOSTICS[fault])
        source = HEADER + helper + "#[verifier::infer] pub fn rotated(lo:u32,hi:u32)->u64 {bits_in_range(lo,hi).rotate_left(1)}\n}\n"
        verify("unsupported_rotation", source, expected="frontend_rejection", inferred=0, seed=seed,
               diagnostic="rotate_left` is not supported")
        verify("solver_budget", HEADER + helper + membership(True) + "}\n",
               expected="resource_or_runtime_failure", seed=seed, extra=("--rlimit", "0.001"))
        for label, argv, expected in [("pin", ["rev-parse", "HEAD"], APPLICATION),
                                      ("clean", ["status", "--porcelain", "--untracked-files=all"], "")]:
            require(commands("application_after_" + label,
                             ["git", "-C", str(args.application), *argv], environment) == 0)
            require((output / ("application_after_" + label + ".stdout")).read_text().strip() == expected)
        guard.check()
        receipt["status"] = "proved"
    print(json.dumps(dict(receipt=str(output / "receipt.json"), status=receipt["status"])))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application", type=Path, required=True)
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    run_suite(parser.parse_args())


if __name__ == "__main__":
    main()
