"""Bounded native diagnostics only; does not invoke or build a verifier."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "source/tools"))
from infer_eval_support import ArtifactGuard, digest

RUSTC = Path("/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc")
CORE = RUSTC.parent.parent / "lib/rustlib/src/rust/library/core/src/cell.rs"
guard = ArtifactGuard([RUSTC, CORE, ROOT / "source/tools/research/redb_crash_protocol/native_state_link/owned_native.rs"])
rows = []
env = dict(os.environ)

def invoke(label, args):
    started = time.monotonic()
    p = subprocess.run(args, cwd=ROOT, env=env, text=True, capture_output=True, timeout=30)
    rows.append(dict(label=label, command=[str(a) for a in args], returncode=p.returncode,
                     seconds=time.monotonic()-started, stdout=p.stdout, stderr=p.stderr))
    return p

with tempfile.TemporaryDirectory(prefix="owned-cell-design-challenge-") as tmp:
    binary = Path(tmp) / "witness"
    p = invoke("native_witness_compile", [RUSTC, "--edition=2024", HERE / "native_witness.rs", "-o", binary])
    assert p.returncode == 0, p.stderr
    p = invoke("native_witness_run", [binary])
    assert p.returncode == 0, p.stderr
    for cfg, code in [("sync", "E0277"), ("overlap", "E0499"), ("escape", "E0515"), ("private", "E0616")]:
        p = invoke("native_reject_"+cfg, [RUSTC, "--edition=2024", "--cfg", cfg, HERE / "native_rejections.rs", "-o", Path(tmp) / cfg])
        assert p.returncode != 0 and code in p.stderr, p.stderr

guard.check()
result = dict(interpretation="Native Rust witnesses and safety/privacy diagnostics only. Finite values are not all-u64 verification; admission and final-input-state proof remain untested.",
              artifact_guard_passed=True, rustc_sha256=digest(RUSTC), core_cell_sha256=digest(CORE), rows=rows)
(HERE / "NATIVE_RESULTS.json").write_text(json.dumps(result, indent=2)+"\n")
print("PASS: native scope witnesses and 4 expected native rejection diagnostics; no verifier invoked")
