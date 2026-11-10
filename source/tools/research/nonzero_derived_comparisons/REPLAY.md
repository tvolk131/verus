# Audit and replay

Run from this checkout, Python bytecode disabled and optimization enabled:

```sh
python3 -B -O source/tools/research/nonzero_derived_comparisons/audit.py
```

The audit reads only explicitly named completed epochs in epochs.json. It checks
receipt/freeze/product hashes, all eight authenticated accepted baseline tools,
retained owned bundles, final source hashes, exact helper and native body sets,
required obligations, controls, matching vstd, exports/imports and focused tests.
Intermediate mutable compiler targets are diagnostic; final frozen bundles are
acceptance inputs. It never executes old drivers, opens historical checkout
paths, or searches neighboring output directories.

Replay the final component against the retained immutable tools:

```sh
python3 -B -O source/tools/research/nonzero_derived_comparisons/owned.py /tmp/nonzero-derive-build-mmnfzmog macro --bundle /tmp/nonzero-derive-frozen-tools-8hnei2sw --expect positive --warm --export
python3 -B -O source/tools/research/nonzero_derived_comparisons/owned.py /tmp/nonzero-derive-build-mmnfzmog width --bundle /tmp/nonzero-derive-frozen-tools-8hnei2sw --expect positive
python3 -B -O source/tools/research/nonzero_derived_comparisons/owned.py /tmp/nonzero-derive-build-mmnfzmog false --bundle /tmp/nonzero-derive-frozen-tools-8hnei2sw --expect semantic
```

Each prints a fresh private output directory and writes durable argv, input
identities, exits, log/product hashes, final guards and receipt. Import the first
command's newly printed provider directory with `import_trial.py BUNDLE PROVIDER`.
The recorded accepted provider is /tmp/nonzero-owned-macro-4okwhwsz. `weak`,
`wrong_model`, `unsupported`, `order`, and `standard_order` are diagnostic cases;
their nonzero verifier exits do not mean healthy-helper semantic negatives.

A fresh independent build uses `build.py` with no arguments. It copies existing
installed package/Git objects privately, builds offline/locked with two jobs,
and prints its output path. It installs nothing. Then use `owned.py BUILD vstd`
for a normal matching verifying vstd build (no --no-cheating), followed by
`freeze_bundle.py BUILD VSTD_OUTPUT`. Application epochs always retain
--no-cheating and full checks. `--cargo-home` and `--target` on build.py may reuse
ONLY the reviewer's own private outputs, not a live sibling/parent build.

Focused repository suites were run by `tests.py BUNDLE OWN_TARGET OWN_CARGO_HOME`.
It uses a private target-verus layout, a private solver override and offline Cargo;
it edits no source/z3, shared configuration, toolchain or shared caches.
`body_identity.py PROVIDER` records genuine pinned Rust expansion plus original
VIR identity without authoring a replacement runtime implementation.

These are source-component checks. Parent must independently build/accept its
combined constructor+comparison source, reconcile versions, require additional
constructor body identities, and regenerate exports with the matching producer.
No replay command implies application publication or whole-goal completion.
