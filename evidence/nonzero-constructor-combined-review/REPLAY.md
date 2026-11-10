Audit the retained accepted review, without rerunning old drivers:

```sh
python3 -B -O evidence/nonzero-constructor-combined-review/audit.py
```

RESULTS.json contains ordered epoch paths, receipt hashes, exact argv/exits,
source/body/helper/candidate identities and final guards. Full per-epoch freezes,
raw logs/products and accepted controller snapshots remain private. audit.py reads
only this work's ledger and explicitly named completed delivered epochs; it never
follows historical checkout paths or reads a live parent/sibling target.

For a fresh replay, preserve the accepted work.json/checkpoint.json first: setup
creates a NEW private work and rewrites those owned metadata files. It copies only
available Cargo.lock-checked archives, index metadata and bare Git objects into a
private CARGO_HOME; it copies no Cargo config or dependency build outputs. The
bare repository metadata contains only core bare/format/filemode settings.

```sh
python3 -B -O evidence/nonzero-constructor-combined-review/review.py setup
python3 -B -O evidence/nonzero-constructor-combined-review/review.py build
python3 -B -O evidence/nonzero-constructor-combined-review/review.py vstd
python3 -B -O evidence/nonzero-constructor-combined-review/freeze.py
python3 -B -O evidence/nonzero-constructor-combined-review/probes.py macro
for case in import width off false weak reversed u32 ordering compose; do
  python3 -B -O evidence/nonzero-constructor-combined-review/probes.py "$case" || exit
done
python3 -B -O evidence/nonzero-constructor-combined-review/identity.py
python3 -B -O evidence/nonzero-constructor-combined-review/old_policy.py
python3 -B -O evidence/nonzero-constructor-combined-review/probes.py tests
python3 -B -O evidence/nonzero-constructor-combined-review/final_tests.py
```

The old-policy symbol client in probes.py is a retained diagnostic expected to
hit Rust dependency resolution. Use old_policy.py for the unchanged certified
interface rejection check. It loads the original interface without using its Rust
symbols and rejects policy/schema/producer compatibility; no metadata is changed.

All application verifier argv include --no-cheating, normal checks and no scope
filters. Normal vstd does not receive --no-cheating. Cargo uses --offline --locked,
CARGO_BUILD_JOBS=2 and private target/home/cache paths. Nothing installs or changes
shared caches/config/toolchains. If normal sandbox startup fails RTM_NEWADDR, use
the exact tool permission mechanism; do not change networking or host settings.

The compiler's Git producer label is 53ece81f.dirty for this accepted bundle;
production source is byte-identical to3aa7f49a. Any new build/producer must export
and import its own matching providers/dependents. Do not reuse old-policy evidence
or adjust body/query scopes to force standalone5/0. The audit is for the retained
review ledger; its excluded diagnostic receipt path intentionally names this run.
