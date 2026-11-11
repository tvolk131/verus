# Reproduction

Use a fresh isolated review checkout/private root; never overwrite the supplied completed directories or this review's completed receipts. All commands run from the repository root with `python3 -B` (final acceptance also `python3 -O -B`). No network/install is needed. `review.py setup` copies only permitted completed checksum-bound dependency inputs and own tracked source, initializes private Git metadata, and authenticates locked archives and the solver. Every invocation freezes argv, environment, complete tracked source, tool and fixture identities plus log hashes. See PRIVATE_ROOT for this completed run.

```
python3 -B evidence/certified-packing-combined-review/review.py setup
python3 -B evidence/certified-packing-combined-review/review.py build
```

This calls Rust1.99.1 cargo build --offline --locked -j2 for rust_verify, verus, builtin, both macro crates and vstd_build. PATH starts with /home/dev/.cargo/bin; CARGO_HOME, CARGO_TARGET_DIR, CARGO_BUILD_BUILD_DIR, cache, HOME and TMPDIR are own private directories. CARGO_BUILD_JOBS=2. The exact environment/commands are in each private freeze.json. The private source manifest is byte-identical to this checkout; production .rs/.toml/lock bytes match21eeabbb. The snapshot informational Git SHA is separately recorded in build metadata.

Before vstd/applications, this review ran `strip --strip-debug` on its own bundle/rust_verify and bundle/verus, recording before/after SHA256 in strip-receipt.json. That final own bundle is then immutable for every positive provider and consumer (and byte-identical test harness copies). A replay may retain its own debug symbols, but must use and record the same compiler bytes on both sides; it will have distinct certificate identities.

```
python3 -B evidence/certified-packing-combined-review/review.py vstd
python3 -B evidence/certified-packing-combined-review/fixtures.py
python3 -B evidence/certified-packing-combined-review/authenticate.py
python3 -B evidence/certified-packing-combined-review/tests.py build
python3 -B evidence/certified-packing-combined-review/tests.py run
python3 -B evidence/certified-packing-combined-review/matrix.py bool
python3 -B evidence/certified-packing-combined-review/stable.py bool
python3 -B evidence/certified-packing-combined-review/stable.py full
python3 -B evidence/certified-packing-combined-review/stable.py packet
python3 -B evidence/certified-packing-combined-review/stable.py packet-public-api
python3 -B evidence/certified-packing-combined-review/matrix.py observers
python3 -B evidence/certified-packing-combined-review/packet_client.py
python3 -B evidence/certified-packing-combined-review/complete_controls.py
python3 -B evidence/certified-packing-combined-review/auxiliary.py
python3 -B evidence/certified-packing-combined-review/erasure.py
python3 -B evidence/certified-packing-combined-review/negative.py
python3 -B evidence/certified-packing-combined-review/runtime_identity.py
python3 -B evidence/certified-packing-combined-review/extra.py transitive
python3 -B evidence/certified-packing-combined-review/extra.py cargo
python3 -B evidence/certified-packing-combined-review/extra.py ordinary-controls
python3 -B evidence/certified-packing-combined-review/extra.py boundaries
python3 -B evidence/certified-packing-combined-review/audit.py
python3 -O -B evidence/certified-packing-combined-review/audit.py
```

`fixtures.py` calls only pure generator/fidelity with its historical upstream inputs rebound to permitted completed fixture bytes. It independently checks the original eight body hashes and exact full SHA256. No historical controller's build/write epoch is executed. `stable.py` archives cold outputs before warm use of the exact same fixture and output path/cache. `audit.py` authenticates receipts, requires all mandatory runs, equal immutable tools, all C/W AIR and helper certificates, exact accepted typed bodies/domains/native owners, all three fresh Eq precision checks, actual cache miss/hit, imported opacity and required failure/stale-output results. It does not require raw whole-API summary fingerprints to match.

The auxiliary-only exact source inserts `assert((0u64 | (1u64 << 1u64)) == 0u64) by(bit_vector)` into the ordinary symbolic bool provider after the healthy conversion. This is intentionally false; the combined engine reports10/1 with failed_functions=[] and must not export. Erased-only source adds `#[cfg(not(verus_keep_ghost_body))] fn erased_type_error()->u64 { () }`: ordinary verification reports2/0, final erased compilation reportsE0308, and must not export. Both seed the selected path with a genuine fresh provider .vir before running, and cover explicit plus Cargo-wrapper selection. All source-level negative controls are private; none modifies production.

The focused test matrix is149 VIR +2 production API units +108 compiler tests (3+5+4+12+3+13+68). Existing fndef_types::exec_calling_no_vstd is the sole ignored test. Representative Cargo output shapes use the actual --VIA-CARGO entry, not the unavailable optional cargo-verus package's cargo-spawn path. Historical worker96/335, old local full710 and old full-help52/0 are not fresh combined results. Parent all36/Python148/canonical26/observer23 results are outside this independent run.
