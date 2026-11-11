# Local reproduction

Start in a fresh own checkout of prepared 913044c4. Only this new evidence
directory is writable. Run Python with -B; the read-only audit also supports -O.
Setup authenticates/copies named completed offline archives/index/git caches and
Z3, creates a new private mode0700 root and snapshots tracked source via own Git.
No shared Cargo target/cache or historical positive VIR is reused. The Item132
fixture has separate explicit user authorization, exact hash and completed-run
freeze authentication in compatibility.py. No pin path is followed.

```sh
python3 -B evidence/local-nonzero-ord-fix-review/build.py setup
python3 -B evidence/local-nonzero-ord-fix-review/build.py build
python3 -B evidence/local-nonzero-ord-fix-review/authenticate.py --write
python3 -B evidence/local-nonzero-ord-fix-review/build.py vstd
python3 -B evidence/local-nonzero-ord-fix-review/trials.py freeze
python3 -B evidence/local-nonzero-ord-fix-review/build_tests.py units
python3 -B evidence/local-nonzero-ord-fix-review/build_tests.py focused
python3 -B evidence/local-nonzero-ord-fix-review/matrix.py exact
python3 -B evidence/local-nonzero-ord-fix-review/matrix.py local
python3 -B evidence/local-nonzero-ord-fix-review/matrix.py bool
python3 -B evidence/local-nonzero-ord-fix-review/dual.py
python3 -B evidence/local-nonzero-ord-fix-review/independent.py
python3 -B evidence/local-nonzero-ord-fix-review/relation.py
python3 -B evidence/local-nonzero-ord-fix-review/compatibility.py
python3 -B evidence/local-nonzero-ord-fix-review/packet.py
python3 -B evidence/local-nonzero-ord-fix-review/matrix.py exports
python3 -B evidence/local-nonzero-ord-fix-review/matrix.py filters-v2
python3 -B evidence/local-nonzero-ord-fix-review/matrix.py global-gates
python3 -B evidence/local-nonzero-ord-fix-review/tests.py vir
python3 -B evidence/local-nonzero-ord-fix-review/tests.py units
python3 -B evidence/local-nonzero-ord-fix-review/tests.py focused
python3 -B evidence/local-nonzero-ord-fix-review/structural.py prepare
python3 -B evidence/local-nonzero-ord-fix-review/structural.py trial
python3 -B evidence/local-nonzero-ord-fix-review/audit.py --write
python3 -B evidence/local-nonzero-ord-fix-review/audit.py
python3 -O -B evidence/local-nonzero-ord-fix-review/audit.py
```

All Cargo commands use --offline --locked -j2, Rust1.99.1. Application commands
include --no-cheating --compile (except explicit no-erasure refusals), normal
lifetime/trait/body/erasure checks, and whole-crate verification for positive
proofs. Cargo-mode proofs use the ordinary driver and full metadata/erasure route.
The exact expanded argv/environment/input hashes are private run freezes, with
argv and receipt hashes summarized in RESULTS.json. Immutable labels reject
accidental reruns into completed receipts. Harness profiles point only to own
frozen tools. Repeated tests.py profile setup rebinds those same local symlinks.

After the original-source tests, structural.py creates a private diagnostic source
copy and builds a separate compiler with unchanged checker plus review-only clone
checks at both actual entry sites. It does not mutate the original source tree or
frozen binaries. Its variant has one changed module, guards, executable identity,
24 rejected clone identities and originals recorded privately. It runs the direct
source through complete normal checks; no mutated Function is returned to proof.
This is diagnostic coverage evidence, separate from source semantic proof runs.

Delivered private root: /tmp/local-nonzero-ord-bd431qjo. Raw source snapshots,
logs, caches, binaries, .vir, AIR and serialized Function records stay there.
Normal and optimized acceptance audits are read-only and use need/RuntimeError,
never Python assert. No parent pipeline, source snapshot, target, cache/config,
process table, sibling checkout, network or publication input is required.
