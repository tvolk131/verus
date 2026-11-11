# Reproduction

Read PREREGISTERED.md first. `baseline.py` audits only the named completed baseline
and authenticates its eight-tool bundle. No old audit is executed. Every new
controller lives in this directory; `review.py` is a rebound copy of the completed
review's offline-build controller and `fixtures.py` rebinds the pure source generator
to the permitted completed full-help bytes. `generate('full')` and `fidelity()`
are called unchanged. Controllers use exception gates, including under Python -O.

Use `python3 -B` throughout. `review.py setup` creates an own private copied source,
Cargo archives/index/git inputs and solver. `review.py build` builds six packages
with Rust1.99.1, offline/locked, -j2, rustup-first PATH. `review.py vstd` builds and
checks matching vstd. `execute.py` freezes actual final eight-tool bytes then runs
the C/W full and packet plus focused provider/client/negative/stale controls.
`tests.py build` builds requested focused test binaries and VIR/compiler units;
`tests.py run` executes them with matching own vstd/compiler/macros. Fresh
labels are immutable; an existing receipt is never overwritten.

An independently built ablation uses a separate own root and exactly the same
source, except the unique duplicate-selection `if` is prefixed `false &&`.
It disables selection and key construction only; no formula is changed. Set
PACKING_DEDUP_ROOT to that own directory for build/vstd/`execute.py ablation`.
Do not mix compiler-bound provider exports between bundles. ABLATION_ROOT records
this experiment's independent root. Build paths and resulting binary identities
necessarily differ; each bundle self-binds all executing bytes and dependencies.

After all trials, `audit.py --write` writes compact RESULTS.json;
`audit.py` and `python3 -O -B audit.py` recheck exact receipts/tool/source hashes,
all query inventories and outcomes read-only. Report timing is runner seconds:
child execution plus post-run source/tool guards, excluding pre-run freeze work.
It includes the compiler's own identity hashing and the candidate's key/hash
construction. Concurrent builds/trials and unoptimized compiler code make this
an obligation-count experiment, not a release performance benchmark. Separate
microbenchmarks of hashing/selection are not claimed.

Read RESULTS.json and REPORT.md for actual outcomes and EXCLUSIONS.md for all
retained setup failures or excluded trials. No mutable supplied directory is
written. Never follow historical receipt paths into a live parent target.
