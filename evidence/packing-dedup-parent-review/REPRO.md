# Independent parent review reproduction

Use the isolated prepared checkout at `5d79f711adc331d917e3b55271f44533c01903fb`.
All commands use Python `-B`; gate failures are exceptions, valid under `-O`.
Only this new evidence directory is committed; raw sources, logs, AIR, binaries,
caches, test executables and full receipts remain at the path in PRIVATE_ROOT.
Do not execute setup against that existing private root: setup requires a fresh
path. For a new replay, set PACKING_DEDUP_ROOT to a new nonexistent /tmp path.

Commands actually used, from the checkout root:

```
python3 -B evidence/packing-dedup-parent-review/authenticate.py --write
python3 -O -B evidence/packing-dedup-parent-review/authenticate.py
python3 -B evidence/packing-dedup-parent-review/review.py setup
python3 -B evidence/packing-dedup-parent-review/review.py build
python3 -B evidence/packing-dedup-parent-review/prepare_bundle.py
python3 -B evidence/packing-dedup-parent-review/review.py vstd
python3 -B evidence/packing-dedup-parent-review/tests.py build
python3 -B evidence/packing-dedup-parent-review/fixtures.py
python3 -B evidence/packing-dedup-parent-review/execute.py
python3 -B evidence/packing-dedup-parent-review/versions.py
python3 -B evidence/packing-dedup-parent-review/tests.py run
python3 -B evidence/packing-dedup-parent-review/audit.py --write
python3 -O -B evidence/packing-dedup-parent-review/audit.py
```

Authentication finished before fresh provider trials. Build overlapped read-only
worker authentication. Test compilation overlapped fresh provider/consumer trials;
finished tests ran independently of those trials. All Cargo compilation uses
Rust1.99.1, offline/locked, two jobs, rustup-first PATH. Cargo archives/index/git
inputs and Z3 were copied only from the named completed input directory and
verified against supplied hashes/Cargo.lock. No shared cache writes occur.

The six build packages are rust_verify, verus, verus_builtin,
verus_builtin_macros, verus_state_machines_macros and vstd_build. The review
removes only debug sections from the own copied rust_verify before vstd; both
hashes and the strip command are recorded. Final eight-tool bytes are frozen
before provider proofs and remain identical for every positive provider and
ordinary/transitive consumer. Tests select copies of those same bytes; the
existing tuple unit's hardcoded own release-path vstd fixture aliases this same
matching bundle. No production file or test assertion is changed.

The committed runners are rebound copies of reviewed worker controllers, with
new authentication/audit/version controls and the additional constructor
permission-equality limit. Pure generator/source/inventory helpers load only
own checkout files. Historical checkout labels are read through own Git objects;
no other checkout, active target, configuration, cache, process or replay state
was read. The candidate and disabled-selection roots named in authenticate.py
are completed artifacts, read-only. Baseline audit authenticates the named
completed eight-tool baseline and inventories AIR; old .vir files never feed
fresh positive imports. No separate ablation build is claimed here: the completed
source-only ablation is fully reauthenticated, supplemented by strict structural
source review and fresh typed/formula inventory matches.

After publication, audit/authenticate run without --write and refuse evidence
changes. Final evidence sealing hashes every committed evidence file except the
seal itself. Precise invocation arguments, source/tool/fixture/log hashes and
outcomes are in RESULTS.json and its private full receipts.
