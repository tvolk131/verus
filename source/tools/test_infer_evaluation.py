#!/usr/bin/env python3
"""Exercise real evaluator subprocesses, pairing, timeouts and build-race handling."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from infer_eval_support import ArtifactChanged, ArtifactGuard, grouped_statistics, measured
from eval_verus_infer_cargo import reports_in

TOOLS = Path(__file__).resolve().parent


class EvaluationTests(unittest.TestCase):
    def test_guard_detects_rewrites_with_same_size_and_timestamp_and_missing_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "driver"
            path.write_bytes(b"old")
            original = path.stat()
            guard = ArtifactGuard([path])
            path.write_bytes(b"new")
            os.utime(path, ns=(original.st_atime_ns, original.st_mtime_ns))
            with self.assertRaises(ArtifactChanged):
                guard.check()
            path.unlink()
            with self.assertRaises(ArtifactChanged):
                guard.check()
            with self.assertRaises(ArtifactChanged):
                ArtifactGuard([path])

    def test_worker_memory_and_output_are_per_invocation(self):
        run = measured([sys.executable, "-c", "a=bytearray(8*1024*1024);print('measured')"],
                       dict(os.environ), 10, TOOLS, memory=True)
        self.assertEqual(run["stdout"].strip(), "measured")
        self.assertEqual(run["returncode"], 0)
        self.assertGreater(run["wall_micros"], 0)
        if sys.platform == "darwin" or sys.platform.startswith("linux"):
            self.assertGreater(run["memory"]["bytes"], 8*1024*1024)
        else:
            self.assertIn("unavailable", run["memory"])

    def test_timeout_is_a_harness_failure_with_preserved_output(self):
        for memory in (False, True):
            with self.assertRaises(subprocess.TimeoutExpired) as caught:
                measured([sys.executable, "-c", "import time;print('started',flush=True);time.sleep(30)"],
                         dict(os.environ), .2, TOOLS, memory=memory)
            self.assertIn("started", caught.exception.stdout)

    def test_statistics_exclude_harness_errors_and_keep_expected_negative_observations(self):
        rows = [dict(case="case", phase="cold", wall_micros=n, errors=errors)
                for n, errors in [(10, []), (30, []), (9999, ["timeout"])]]
        group = grouped_statistics(rows)["case/cold"]
        self.assertEqual(group["wall_micros"]["median"], 20)
        self.assertEqual(group["wall_micros"]["p90"], 30)
        self.assertEqual(group["excluded_failures"], 1)

    def test_cargo_reports_parse_multiple_crates_with_interleaved_diagnostics(self):
        text = 'cargo output\n{"verus-infer":{"status":"verified"}}\nnot json {\n'
        text += '{"verus-infer":{"status":"caller_proof_failed"}}\n'
        self.assertEqual([r["status"] for r in reports_in(text)], ["verified", "caller_proof_failed"])

    def evaluator(self, mutate=False, malformed=False):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            verifier = root / "verus"
            for name in ("rust_verify", "vstd.vir", "libvstd.rlib", "libverus_builtin.rlib",
                         "libverus_builtin_macros.dylib", "libverus_state_machines_macros.dylib", "z3"):
                (root / name).write_bytes(b"stable build input")
            verifier.write_text(f'''#!{sys.executable}
import json,os,pathlib,sys
cache=pathlib.Path(os.environ['VERUS_INFER_CACHE']);hit=cache.exists();cache.mkdir(exist_ok=True)
args=sys.argv;pathlib.Path(args[args.index('--export')+1]).write_bytes(b'interface')
proofs={{o:{{'required':True,'status':'proved'}} for o in ('soundness','maximality','exactness')}}
f={{'function':'fake::f','mode':'exact','proofs':proofs,'required_checks_passed':True,
'exported':True,'candidate_id':'a'*64,'contract':{{'requires':'true','ensures':'result == 1'}},
'metrics':{{'cache':'hit' if hit else 'miss','solver_micros':10,'requires_nodes':1,'ensures_nodes':3}}}}
if {malformed!r}: del f['metrics']['solver_micros']
print(json.dumps({{'verus-infer':{{'schema_version':1,'status':'verified','crate_success':True,
'interface_written':True,'functions':[f],'metrics':{{'generation_micros':10}}}}}}))
if {mutate!r}: pathlib.Path(__file__).with_name('rust_verify').write_bytes(b'changed binary')
''')
            verifier.chmod(0o755)
            (root / "input.rs").write_text("fn main(){}")
            manifest = root / "cases.json"
            manifest.write_text(json.dumps(dict(schema_version=1, cases=[dict(name="fake", source="input.rs",
                                         features=["harness_fixture"], expected_status="verified")])) )
            output = root / "results"
            run = subprocess.run([sys.executable, str(TOOLS / "eval_verus_infer.py"), "--verus", str(verifier),
                                  "--manifest", str(manifest), "--output-dir", str(output), "--repetitions", "3"],
                                 env=dict(os.environ, VERUS_Z3_PATH=str(root / "z3")),
                                 text=True, capture_output=True)
            result = json.loads((output / "results.json").read_text())
            if malformed:
                self.assertNotEqual(run.returncode, 0)
                self.assertEqual(len(result["runs"]), 6)
                self.assertTrue(all("harness failure" in r["errors"][0] for r in result["runs"]))
                self.assertEqual(result["summary"]["statistics"]["fake/cold"]["wall_micros"]["n"], 0)
            elif mutate:
                self.assertNotEqual(run.returncode, 0)
                self.assertEqual(len(result["runs"]), 1)
                self.assertIn("rust_verify", result["aborted"])
                self.assertEqual(result["summary"]["statistics"]["fake/cold"]["wall_micros"]["n"], 0)
            else:
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertEqual(len(result["runs"]), 6)
                self.assertEqual([r["report"]["functions"][0]["metrics"]["cache"] for r in result["runs"]],
                                 ["miss", "hit"]*3)
                self.assertEqual(result["summary"]["statistics"]["fake/cold"]["wall_micros"]["n"], 3)

    def test_malformed_report_persists_harness_failures_and_excludes_timings(self):
        self.evaluator(malformed=True)

    def test_repetitions_pair_fresh_caches_and_proofs(self):
        self.evaluator()

    def test_build_mutation_aborts_before_warm_run_and_persists_failure(self):
        self.evaluator(mutate=True)


if __name__ == "__main__":
    unittest.main()
