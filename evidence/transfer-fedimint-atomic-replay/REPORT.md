# Fedimint storage-boundary proof cycle: bounded conditional milestone

November 2, 2026. **CHECKED CONDITIONAL MODEL-ONLY.** Native async refinement,
RocksDB atomicity/durability, fund safety and consensus safety/liveness are NOT
proved. No independent acceptance is claimed for this result or candidate77.

## Starting state and confinement

Confirmed CLEAN HEAD `3b05c1912b9d7314ad00c832c4065f4362562e02` in the
tool-provided worktree. Applied the complete frozen combined patch EXACTLY ONCE,
after checking SHA256
`545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12`.
Subsequent work adds only the two unique `source/tools/*fedimint_atomic_replay.py`
files and `evidence/transfer-fedimint-atomic-replay/`. The 1,976 dirty77 baseline
files retain their original hashes. No checkouts, branches/ref changes, commits,
pushes, PRs or delegates were created. No production/shared-controller/guide/CI,
identity, native, parent, toolchain, runtime, dependency or Cargo-home edits.

Reused sealed `/private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77`, identity
cache77/policy71/schema38. No Cargo/compiler rebuilds or dependency fetches.
The only compiled Rust outputs are small authored proof-model libraries in fresh
private scratch. Approximately 55 GiB was available at inspection; no compiler
targets or dependency trees were copied. Git context is sanitized for all
runner/reconstruction commands with `GIT_OPTIONAL_LOCKS=0`.

## Inspected source versus proved result

Read the pinned actual `process_consensus_item_attempt` at engine.rs:974,
its retry loop, staged processing/marker/audit/commit sequence, current-session
cleanup, DB commit/notification/hook wrappers, diagnostic-only drop logger,
RocksDB snapshot/sync/commit/error mapping, and the locally installed matching
Rust/C++ commit/rollback/destruction boundaries. Exact inspected file hashes and
line-level remaining obligations are in `SOURCE_REFINEMENT.md`, the runner and
`inspected-dependencies.json`.

The checked model explicitly separates volatile request/phase/reply from durable
marker, effect-bundle accounting and unrelated frame. Its atomic transition
admits all or none; its SEPARATE crash semantics retain durable state, including
when the result/ack was lost. Unbounded finite-trace induction proves recovered
pairing and at most one bundle in this marker lifetime. Matching replay changes
no durable state; different item or peer at the same index rejects.

Atomicity, durability/recovery, module write confinement, serialized attempts,
equality reflection and marker lifetime are explicit AUTHOR-OWNED semantics and
boundaries, not assumed native application theorems. Fsync durability alone does
not imply transaction atomicity. `AcceptedItemKey` is an INDEX and native session
completion deletes its markers; no invented native session key or forever
exactly-once property is used. Full boundary/exclusion inventory: `README.md`.

Checked counts per positive: exactly **18 verified, 0 errors, 1 executable body,
16 proof bodies, 15 specification definitions**; whole selected authored scope,
32 exact function identities. The extra query is recursive-spec termination.
ZERO inferred functions, native unchanged slices or native correspondences.
`scope.json` binds the complete model and every named function/body identity.
No `assume`, `external_body`, `admit`, trusted application model or engine expansion.

## Fresh validation

Final normal and `python -O` tests: **27 + 27 passed**.
Final normal and `python -O` replays: **11 + 11 cases**, all expected outcomes.
Each replay includes two whole-scope default-erasure compiles with no-cheating,
seven semantic negative controls, one arithmetic-safety control and one seeded
ordinary-export failure control. The positive model additionally CHECKS explicit
non-atomic marker/effect and missing-dedup counterexample witnesses.

The final runner enforces exact typed success/error/body counts and scope flags,
all named executable/proof/spec identities, actual source paths and hashes,
ordinary/no-inference classification, no imported dependencies/trusted-model
rows, fresh output directories, command/environment and artifact hashes.
Normal and optimized modes both audited BEFORE/AFTER:

- 1,976 baseline files, 1,566 source and 1,085 Rust build inputs.
- 9 sealed runtime files, 61 toolchain files and all 3,395 frozen guarded inputs.
- 1,199 Fedimint and 132 redb native source entries, both clean at their pins.
- Four additional inspected installed dependency source files, read-only.

All hashes match. These are input audits, NOT freshly executed baseline suites.
The parent's 304-test, VIR/division and previous application suites were NOT
rerun here; hosted CI was not dispatched. Prior parent repairs still require
the new independent combined review requested by the brief.

## Legacy publication observation

All final runs are ordinary model-only functions with no Infer goals/imports.
`interface_written=false` while ordinary `.vir` artifacts exist, including for
failed proofs. Both seeded controls replace the seed with a DIFFERENT ordinary
artifact before the proof failure is returned. This is recorded legacy behavior,
NOT inference-aware publication safety, stale-export removal, cache equivalence
or safe import/export. No `.vir` is imported; artifact existence is never treated
as proof acceptance. Early exploratory probes used `--infer-goals`; no inference
credit or publication guarantee is drawn from those probes.

## Failed attempts (retained, not counted as passes)

1. `invariant` as a spec-function name triggered a Verus parser error. Renamed
   the authored predicate to `valid_state`; no source domain or goal was weakened.
2. A tuple-valued `choose` did not follow from a two-binder existential. The
   step proof failed one postcondition/recommendation. Changed the proof to the
   matching two-binder `choose`; all original transition cases remain.
3. The first integrated negative reader expected `no_inference` on failure;
   actual ordinary proof failures report `caller_proof_failed`. Compiler
   rejection was correct. Fixed the classification check and fixtures, retaining
   the whole-scope semantic-error and no-inferred-function gates.

`attempts/` retains the original logs and commands. The two early model source
snapshots are explicitly reconstructed from the edit sequence, not claimed as
contemporaneous captures. The reconstructed old reader exactly matches the SHA
recorded in its original partial receipt. No failure is relabelled a pass.

## Artifacts, hashes and reconstruction

Fresh live outputs, including hashed `.vir`/`.rlib` artifacts:
`/private/tmp/verus-fedimint-atomic-cycle-q1z97f34/delivery-validation/`.
Text logs, final validation commands and both receipts are copied byte-for-byte
under `runs/`. Binary compiler artifacts stay in private scratch, not the text
apply_patch delta; they are not provider/import dependencies. Replay regenerates
them; binary hashes need not be path-independent across fresh compilations.

Key SHA256 values:

- model: `e373a378184c79523b32d134a8ba61a04c5210adb9a1bfa935fcacd9d86bdb4c`
- checker: `660db5dd1a80ac859494e46cbaee95ca9f7a6c8229cef1c5b09b473b35d557e4`
- tests: `f8072854bf5500005d1d65f1742aebb347d313978b354d176ca65720df6253f9`
- normal receipt: `9acda54c4c0595c89ed5a2bedc0b25299cc1adef3a81b2e49120e32d2a801c76`
- optimized receipt: `f721f01e19838d4a3bc4156e061f01c5f09f9a498baf14a6364eff30d927c53f`

Delivery directory:
`/private/tmp/verus-fedimint-atomic-cycle-q1z97f34/delivery/`.
`inventory.json` gives the exact NEW-file inventory, byte counts and SHA256s.
`dirty77-relative.apply_patch` is an additions-only delta against the EXACT
already-dirty77 baseline, NOT another full patch against CLEAN HEAD. Its final
SHA is in `dirty77-relative.apply_patch.sha256` and the external manifest, avoiding
a self-hashing report/patch cycle. Do not apply the baseline patch a second time.

`reconstruct.py` reads checkpoint Git objects and replays the 94-path frozen
patch, checks all 1,976 dirty77 file hashes, then replays this incremental delta
and checks EVERY resulting file hash. This is an explicit in-memory byte-tree
reconstruction, NOT a new Git checkout/worktree. It does not count as another
application of the frozen patch to this worktree. To regenerate delivery in NEW
private scratch:

```
python3 -B evidence/transfer-fedimint-atomic-replay/reconstruct.py --delivery /private/tmp/atomic-new-delivery
```

See the external `reconstruction.json` for exact counts and equality results.
Model-only replay/test commands are in `README.md` and `runs/validation.json`.
This is a useful bounded conditional milestone for parent integration and NEW
independent review, not completion of native transaction/recovery verification.
