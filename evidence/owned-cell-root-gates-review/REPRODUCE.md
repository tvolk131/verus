# Reproduction and evidence limits

Read SCOPE.md, AUDIT.md, BINDING.json, REPORT.md and RESULTS.json together.
Scripts use only the Python standard library plus the repository's existing
infer_experiment.py/infer_eval_support.py. No Cargo build or source/harness/tool
transformation is performed. Raw compiler sources, outputs, execve traces and
receipts are private under /tmp/owned-cell-root-gates-review-9761b5f5; compact
committed evidence records their SHA256 values. Do not publish raw repeated logs.

The snapshot receipt SHA256 must be
fb56f75c515ab64a6c87b8ac465a1bb282b819f0d5aa10a9069eb102cf8c8f81.
Authenticate all eight tool files AND four checks binaries before copying;
recheck.py prepare does so. Keep original checks binaries in snapshot/checks/,
with verus-root and all eight tools at snapshot/. No wrappers/symlinks that
change current_exe.parent.parent; no compiler-cache or build-tree copies.

From a checkout containing these scripts and the pinned prepared production
source, use a fresh private output (default output must not already exist):

```
export PYTHONDONTWRITEBYTECODE=1
export ROOT_GATES_REVIEW_OUTPUT=/tmp/owned-cell-root-gates-review-unique-replay
python3 evidence/owned-cell-root-gates-review/recheck.py prepare
python3 evidence/owned-cell-root-gates-review/recheck.py replay
python3 evidence/owned-cell-root-gates-review/cli_witnesses.py
```

prepare records its new binding; it does not manufacture the provenance of the
parent-built tools. The original source-build provenance is an authenticated
parent receipt assertion, not an independently rebuilt compiler. Native u64
identity/MIR controls run against the existing Rust 1.99.1 core. Seven supporting
tools are inherited unchanged; NORMAL vstd2257/0 is inherited, NOT reverified.
The exact Rust environment and libraries are in BINDING.json. Every suite and
CLI command has fresh private TMPDIR/cache/output. Existing InferCrates creates
its temporary crates under the private TMPDIR and overrides caches within them.

The binary replay asserts exact inventories from --list, actual pass counts,
source names for all twelve focused tests and all nine internal-representation
tests among full VIR172. strace observes execve; it does not replace any tool.
Count top-level rust_verify calls separately from internal trait-conflict-driver
subprocesses. The refusal suite intentionally has two calls without
--no-cheating to test option refusal; all other focused calls carry it. The
admission suite additionally invokes native rustc once for its import fixture;
that is not a Verus proof. All CLI cases use --no-cheating and --compile.

The exact CLI argv is generated from cli_witnesses.py: snapshot/verus, the
case source, --crate-type=lib --crate-name=witness --edition=2024 --no-cheating
--compile --output-json -o <private-case>/witness.rlib, optional true default
option, optional --infer-goals, optional --experimental-owned-unsafe-cell.
No --no-lifetime/--no-verify/--no-trait-conflicts/--no-erasure-check flags appear
in these direct cases. The unchanged refusal suite tests forbidden flags as
negative controls. No source privacy or parse failure earns proof credit.

Observed tests and costs are receipts of original parent-artifact replay and
independent source witnesses on those tools. Source inspection of additional
metadata fields is explicitly separated from the nine runtime fixture rows.
No arbitrary malformed-AST completeness claim, new synthetic build, independent
rebuild, native body proof, complete Function correspondence, field authority,
F2/F3 or activation follows. The full REDB application theorem is not accepted.
