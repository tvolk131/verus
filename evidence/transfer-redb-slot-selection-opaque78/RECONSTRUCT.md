# Reconstruct and replay

The reviewed FULL dirty baseline is against CLEAN HEAD
`3b05c1912b9d7314ad00c832c4065f4362562e02`. Never use it as an incremental patch
on already dirty sources. Its authorized patch is
`/private/tmp/verus-cycle77-storage-integration.PYURYB/final-package-v2/combined.apply_patch`,
SHA-256 `e76b758498a692e30e5683486a4d59d453202529cd13c2c0169401542b31690e`.
It was restored exactly once in this implementation checkout.

The bounded addition-only delta and its hash/inventories are in
`/private/tmp/verus-redb-opaque78.reABJTPu/delivery/`. Apply `delta.apply_patch`
only AFTER that exact full baseline. `reconstruction.json` checks byte maps in
memory, without making another checkout, changing refs, or applying the
baseline again to the working tree. `inventory.json` binds every added file;
`file-hashes.json` binds the whole baseline-plus-delta tree. The delivery
manifest contains the delta SHA-256 and reconstruction result.

Normal proof replay intentionally has no historical worktree/snapshot
dependency and no sealed-runtime absolute default. From the reconstructed repo:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B source/tools/check_redb_slot_selection_opaque.py \
  --application /path/to/clean/pinned/redb \
  --verus /path/to/compatible/verus --solver /path/to/compatible/z3 \
  --output /fresh/persistent/scratch/opaque78-normal
PYTHONDONTWRITEBYTECODE=1 python3 -B -O source/tools/check_redb_slot_selection_opaque.py \
  --application /path/to/clean/pinned/redb \
  --verus /path/to/compatible/verus --solver /path/to/compatible/z3 \
  --output /fresh/persistent/scratch/opaque78-optimized
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover \
  -s source/tools -p 'test_redb_slot_selection*.py'
PYTHONDONTWRITEBYTECODE=1 python3 -B -O -m unittest discover \
  -s source/tools -p 'test_redb_slot_selection*.py'
```

The verifier must be compatible with cache77/policy71/schema38 and the frozen
standard-model/certificate identities. Its adjacent runtime artifacts must be
present. The installed Rust 1.99.1 sysroot is discovered read-only under the
initial rustup location; specify `--sysroot /installed/compatible/sysroot` if
discovery is ambiguous. The child uses private fresh homes/caches and bypasses
rustup execution; it does not install or build a toolchain or engine.

The application must be clean at
`c82ab865bb362bc4af067d332adc44978ff64813`; all 132 pinned tracked files are
content-checked, not merely trusted from Git status. Every output path must
be new and outside the repository. Failure preserves all output/log files.

To additionally audit original historical provenance, append
`--snapshot /path/to/final-package-v2`. That separate audit uses its full
file/source/build/runtime/native/absolute guarded ledgers and is naturally
machine-specific. Omitting it does NOT disable any proof, source-fidelity,
scope, certificate/import, seeded-removal or current ArtifactGuard check.
Both delivered acceptance replays omit it to exercise the normal path; the
original baseline ledgers are separately audited before and after this task.

The full-enum original frontend blocker remains a distinct expected negative
without the two opaque declarations. No failed attempt or old positive is a
substitute for a fresh successful full-matrix receipt. Read `REPORT.md` before
interpreting the theorem or the deliberately limited implicit-drop probe.
