"""Mock-network setup and synthetic real-format Cargo/archive acceptance tests."""

from contextlib import redirect_stdout
import copy
import io
import json
import os
from pathlib import Path
import re
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import infer_transfer_dependencies as dependencies
import infer_transfer_receipts as acceptance
import prepare_infer_dependencies as preparation
from infer_eval_support import ArtifactGuard, digest


def graph(home):
    entries = []
    nodes = []
    for block in dependencies.LOCK.read_text().split("[[package]]")[1:]:
        fields = dict(re.findall(r'^(name|version|source) = "([^"\n]+)"$', block, re.M))
        name, version = fields["name"], fields["version"]
        identifier = name + "@" + version
        selected = re.search(r'^dependencies = (\[[^]]*\])', block, re.M)
        names = re.findall(r'"([^"\n]+)"', selected.group(1)) if selected else []
        source = home / "registry/src/crates-test" / (name + "-" + version)
        entries.append(dict(name=name, version=version, id=identifier, source=fields.get("source"),
                            manifest_path=str(source / "Cargo.toml")))
        nodes.append(dict(id=identifier, dependencies=names))
    identifiers = {entry["name"]: entry["id"] for entry in entries}
    for node in nodes:
        node["dependencies"] = [identifiers[name] for name in node["dependencies"]]
    return dict(packages=entries, resolve=dict(nodes=nodes))


def registry(home):
    selected = copy.deepcopy(dependencies.packages(dependencies.LOCK.read_text()))
    guarded = []
    for entry in selected:
        name = entry["name"] + "-" + entry["version"]
        source = home / "registry/src/crates-test" / name
        source.mkdir(parents=True)
        manifest = source / "Cargo.toml"
        manifest.write_text('name = "' + entry["name"] + '"\n')
        archive = home / "registry/cache/crates-test" / (name + ".crate")
        archive.parent.mkdir(parents=True, exist_ok=True)
        with tarfile.open(archive, "w:gz") as stream:
            stream.add(manifest, arcname=name + "/Cargo.toml")
        entry["checksum"] = digest(archive)
        guarded += [archive, manifest]
    return selected, guarded


class DependencyAcceptanceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="dependency acceptance ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.home = self.root / "cargo-home"
        self.selected, self.guarded = registry(self.home)
        self.addCleanup(patch.stopall)
        patch.object(acceptance, "packages", return_value=self.selected).start()
        for relative, content in {"application-inputs/Cargo.lock": dependencies.LOCK.read_text(),
                "dependency/Cargo.lock": dependencies.LOCK.read_text(), "dependency/Cargo.toml": acceptance.full.DEPENDENCY,
                "dependency/src/lib.rs": "pub use bitcoin::hashes::sha256;\n",
                "dependency-target/release/deps/libbitcoin-test.rlib": "synthetic native artifact"}.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        self.guarded += [self.root / "dependency" / name for name in ("Cargo.toml", "Cargo.lock", "src/lib.rs")]
        guard = ArtifactGuard(self.guarded).initial
        self.receipt = dict(commands=[], dependency_sources_before=guard, dependency_sources_after=copy.deepcopy(guard),
            dependency=dict(packages=self.selected, lock_sha256=dependencies.LOCK_SHA256,
                bitcoin_rlib_sha256=digest(self.root / "dependency-target/release/deps/libbitcoin-test.rlib"),
                artifacts={"release/deps/libbitcoin-test.rlib": digest(self.root / "dependency-target/release/deps/libbitcoin-test.rlib")}))
        for label, argv in (("dependency_resolve", ["cargo", "metadata", "--offline"]),
                            ("dependency_locked", ["cargo", "build", "--offline", "--locked", "--release"])):
            self.receipt["commands"].append(dict(label=label, argv=argv, exit_code=0))
        (self.root / "dependency_resolve.stdout").write_text(json.dumps(graph(self.home)))

    def check(self):
        return acceptance.full_dependencies(self.root, self.receipt)

    def test_accepts_complete_pinned_graph_archive_source_and_artifact_guards(self):
        self.assertEqual(self.check().name, "libbitcoin-test.rlib")

    def test_missing_duplicate_and_substituted_dependencies_or_artifacts_fail_closed(self):
        original = copy.deepcopy(self.receipt)
        mutations = [lambda receipt: receipt["dependency"]["packages"].pop(),
                     lambda receipt: receipt["dependency"]["packages"].append(receipt["dependency"]["packages"][0]),
                     lambda receipt: receipt["dependency"].update(lock_sha256="0" * 64),
                     lambda receipt: receipt["dependency"].update(artifacts={}),
                     lambda receipt: receipt["dependency"].update(bitcoin_rlib_sha256="0" * 64),
                     lambda receipt: receipt["commands"][0]["argv"].remove("--offline"),
                     lambda receipt: receipt["commands"][1]["argv"].remove("--locked"),
                     lambda receipt: receipt["commands"][0].update(exit_code=1),
                     lambda receipt: receipt["dependency_sources_before"].pop(str(self.guarded[0]))]
        for mutation in mutations:
            self.receipt = copy.deepcopy(original)
            mutation(self.receipt)
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.check()

    def test_metadata_must_bind_source_paths_and_all_real_dependency_edges(self):
        path = self.root / "dependency_resolve.stdout"
        original = json.loads(path.read_text())
        for edit in (lambda data: data["packages"][0].update(source="path+fake"),
                     lambda data: data["packages"][0].update(manifest_path="/substituted/Cargo.toml"),
                     lambda data: data["packages"].append(data["packages"][0]),
                     lambda data: data["resolve"]["nodes"][3]["dependencies"].pop()):
            data = copy.deepcopy(original)
            edit(data)
            path.write_text(json.dumps(data))
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                self.check()

    def test_source_and_artifact_byte_drift_cannot_be_hidden_by_equal_receipt_guards(self):
        self.guarded[1].write_text("tampered source")
        with self.assertRaises(ValueError):
            self.check()


class DependencyPreparationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="dependency preparation ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.application = self.root / "application"
        (self.application / ".git").mkdir(parents=True)
        self.inputs = {}
        for relative in preparation.INPUT_HASHES:
            path = self.application / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(dependencies.LOCK.read_text() if relative == "Cargo.lock" else "synthetic pinned source")
            self.inputs[relative] = digest(path)
        self.cached_home = self.root / "cached-home"
        self.selected, _ = registry(self.cached_home)
        for entry in self.selected:
            path = self.cached_home / "registry/index/crates-test/.cache" / preparation.index_path(entry["name"])
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic sparse index")
        (self.cached_home / "registry/index/crates-test/config.json").write_text(json.dumps(dict(dl="https://static.crates.io/crates")))
        self.destination, self.output = self.root / "prepared", self.root / "evidence"
        self.calls, self.failed_fetch = [], False
        self.addCleanup(patch.stopall)
        patch.object(preparation, "INPUT_HASHES", self.inputs).start()
        patch.object(preparation, "packages", return_value=self.selected).start()
        patch("infer_experiment.run_compiler", self.run_compiler).start()

    def run_compiler(self, argv, environment, timeout, cwd=None):
        self.calls.append((list(argv), dict(environment), timeout))
        stdout = ""
        if argv[0] == "git":
            stdout = str(self.application) if argv[-1] == "--show-toplevel" else acceptance.full.APPLICATION if argv[-1] == "HEAD" else ""
        elif argv[:2] == ["cargo", "fetch"]:
            if self.failed_fetch:
                return SimpleNamespace(returncode=1, stdout="", stderr="controlled mock registry endpoint unavailable")
            home = Path(environment["CARGO_HOME"])
            for path in self.cached_home.rglob("*"):
                if path.is_file():
                    destination = home / path.relative_to(self.cached_home)
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    if not destination.exists():
                        destination.write_bytes(path.read_bytes())
        elif argv[:2] == ["cargo", "metadata"]:
            stdout = json.dumps(graph(Path(environment["CARGO_HOME"])))
        else:
            self.fail("unexpected setup command: " + repr(argv))
        return SimpleNamespace(returncode=0, stdout=stdout, stderr="")

    def prepare(self, cached=False):
        with redirect_stdout(io.StringIO()):
            return preparation.prepare(self.application, self.destination, self.output, 19,
                                       self.cached_home if cached else None)

    def test_network_fetch_is_explicit_locked_bounded_and_followed_by_offline_validation(self):
        self.prepare()
        receipt = json.loads((self.output / "receipt.json").read_text())
        self.assertTrue(receipt["success"])
        self.assertTrue(receipt["network_fetch"])
        self.assertFalse(receipt["application_proofs_proved"])
        fetches = [entry for entry in self.calls if entry[0][:2] == ["cargo", "fetch"]]
        self.assertEqual(len(fetches), 1)
        self.assertIn("--locked", fetches[0][0])
        self.assertEqual(fetches[0][1]["CARGO_NET_OFFLINE"], "false")
        self.assertEqual(fetches[0][2], 19)
        metadata = next(entry for entry in self.calls if entry[0][:2] == ["cargo", "metadata"])
        self.assertIn("--offline", metadata[0])
        self.assertIn("--locked", metadata[0])
        self.assertEqual(metadata[1]["CARGO_NET_OFFLINE"], "true")
        self.assertFalse(any("build" in argv for argv, _, _ in self.calls))

    def test_cached_setup_has_no_network_fetch_and_checks_exact_registry_archives(self):
        self.prepare(cached=True)
        receipt = json.loads((self.output / "receipt.json").read_text())
        self.assertFalse(receipt["network_fetch"])
        fetch = next(entry for entry in self.calls if entry[0][:2] == ["cargo", "fetch"])
        self.assertIn("--offline", fetch[0])
        self.assertEqual(fetch[1]["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(len(receipt["packages"]), 21)

    def test_fetch_failure_leaves_fresh_failure_evidence_without_metadata_or_proof(self):
        self.failed_fetch = True
        with self.assertRaisesRegex(ValueError, "fetch failed"):
            self.prepare()
        receipt = json.loads((self.output / "receipt.json").read_text())
        self.assertFalse(receipt["success"])
        self.assertFalse(receipt["application_proofs_proved"])
        self.assertFalse(any(argv[:2] == ["cargo", "metadata"] for argv, _, _ in self.calls))
        self.assertIn("controlled mock", (self.output / "dependency_fetch.stderr").read_text())

    def test_inherited_git_cargo_wrappers_and_secrets_are_not_configuration_or_logs(self):
        poison = dict(GIT_DIR="/wrong/.git", GIT_WORK_TREE="/wrong", GIT_CONFIG_COUNT="1",
                      CARGO_TARGET_DIR="/wrong/target", CARGO_BUILD_RUSTC_WRAPPER="/wrong/wrapper",
                      RUSTC_WRAPPER="/wrong/wrapper", RUSTFLAGS="--cfg poison", CARGO_REGISTRIES_CRATES_IO_TOKEN="DO_NOT_PRINT")
        with patch.dict(os.environ, poison):
            self.prepare()
        for _, environment, _ in self.calls:
            for name in poison:
                self.assertNotIn(name, environment)
            self.assertEqual(environment["GIT_CONFIG_GLOBAL"], os.devnull)
            self.assertEqual(environment["GIT_CONFIG_NOSYSTEM"], "1")
        self.assertNotIn("DO_NOT_PRINT", "".join(path.read_text() for path in self.output.glob("*")))

    def test_existing_destinations_receipts_and_symlinks_cannot_be_overwritten(self):
        self.prepare()
        before = {path.name: path.read_bytes() for path in self.output.iterdir()}
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertEqual(before, {path.name: path.read_bytes() for path in self.output.iterdir()})
        self.output = self.root / "new-evidence"
        self.destination = self.root / "linked-destination"
        self.destination.symlink_to(self.root / "missing")
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_overlapping_paths_and_wrong_repository_roots_are_rejected(self):
        self.output = self.destination / "evidence"
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.destination.exists())
        self.output = self.root / "evidence"
        (self.application / ".git").rmdir()
        (self.application / ".git").symlink_to(self.cached_home)
        with self.assertRaisesRegex(ValueError, "repository root"):
            self.prepare()
        self.assertFalse(json.loads((self.output / "receipt.json").read_text())["success"])


if __name__ == "__main__":
    unittest.main()
