# Cycle79 parent-owned compatibility review

## Result and scope

No actionable compatibility findings in the ten-file candidate-relative delta. This is an independent compatibility review, not approval of the production Rust implementation or a whole-application theorem. No fixes or subdelegation were performed.

The review used only the assigned isolated worktree `/Users/tommyvolk/.t3/worktrees/verus/t3-2f408805`, initially clean at `3b05c1912b9d7314ad00c832c4065f4362562e02`. The supplied full dirty snapshot was applied exactly once using `apply_patch`.

- Combined patch SHA256: `1e6884c7a7dc762796fc172e8e0caedb6e818037fc23fb3c7b952b2dd323561d`.
- Ten-file candidate-relative patch SHA256: `1e637a6c1aacdb3f0f982c69866ed583cea46ee968095e04737bad81a16fa351`.
- All 2,136 reconstructed files matched the supplied hash inventory, with no extra or missing repository files, both before and after testing.
- All 1,579 source-ledger entries, 1,088 build-input entries, eight runtime entries, and 61 toolchain entries matched. Production/build inputs are identical to the sealed delivery candidate. Production review was not duplicated.

## Compatibility conclusions

The shared engine identity is exactly cache78/policy72/schema38. The live original certificate reader and both certificate test fixtures use the new evidence location, while old certificate and opaque fixtures remain byte-identical historical evidence.

The new provider certificate was compared directly against the raw `provider.vir` trailer and JSON certificate in `/private/tmp/verus-prefix-infer79.1eVEuO/redb-current-qualification`, not derived from altered replay receipts. Its provider output is an audited successful whole-crate 23 verified/0 errors. Both expectation files exactly equal that independent qualification's files.

Exact producer:

```text
0.2026.11.01.3b05c19.dirty:1.99.1-aarch64-apple-darwin (overridden by '/Users/tommyvolk/.t3/worktrees/verus/t3-5eafa72c/rust-toolchain.toml'):3b05c1912b9d7314ad00c832c4065f4362562e02
```

The complete ordered 380-model inventory has SHA256 `013ca2c073015c74eecc716a5886844a65529fb8a11198ec8f5b813426962376` using the validator's sorted-key JSON serialization. Exactly three identities were added, with no removed identities:

- `Alloc::vec::impl&%3::extend_from_slice` (the byte append registration).
- `Vstd::slice::impl&%0::view`.
- `Vstd::seq::Seq::add`.

All 377 common identities have changed fingerprints. These were not treated as three fingerprint substitutions: the complete fresh-bound inventory is pinned. The bound vstd artifacts differ from the historical runtime; current `vstd.vir` is `a05d9a5b1a81b7ba4e5f18ab48706564a30367457a1842e687e9865989957f29` and current `libvstd.rlib` is `e107f079b8a6d8e761a690c36dd4f37fdffb27248fe6762a876e4a5676fd2846`. The sealed build receipt identifies compiler10 and records the exact products; this review did not rebuild or freshly reverify vstd.

The new opaque fixture preserves the historical expected scopes/contracts, all six boundaries, opaque declarations, and 43-row provider scope exactly. All historical base inputs remain listed. Only the precise hashes of `check_redb_slot_selection.py` and `infer_transfer_identity.py` change; the unchanged historical opaque fixture is additionally hash-bound, taking the base-input inventory from 23 to 24 entries. Historical evidence was not silently upgraded.

The documentation maintains the primitive-byte/local-receiver boundary, ordered sequence equation, mandatory proof gates and explicit namespace/backend/durability nonclaims. No new whole-native-module claim is introduced by this compatibility delta.

## Fresh validation performed here

- Normal Python: 65 focused slot-selection tests and 18 new prefix tests pass.
- Optimized Python (`-O`): the same 65 focused slot-selection tests and 18 new prefix tests pass.
- Normal Python: fresh opaque replay passes all 47 cases using sealed runtime78/compiler10, the pinned native redb checkout, and the requested 1.99.1 sysroot.
- Optimized Python: an independent fresh output directory passes the same 47 cases.
- Each replay has ten positive, ten semantic-negative, two safety-negative, sixteen malformed-import, seven frontend-boundary and two lifetime-boundary cases. These categories are not all application semantic proofs.
- All twenty positive artifacts across those two replays exactly match the qualified producer and complete 380-model fingerprint pin.
- Independent probes pass in both Python modes: 845 checks/mode, comprising 830 deliberate rejection checks, three positive controls and twelve isolated guard-removal sensitivity controls.

The independent rejection probes perturb every one of the 380 fingerprints separately in each of the original and opaque certificate validators. They also exercise stale policy/producer, typed schema and fixture identities, complete archived certificate/fixture rejection, missing/extra/duplicate/reordered/application/old-fingerprint inventories, and every historical base-input hash. Import-command probes use an actual fresh consumer row and cover VIR/rlib kind swapping, joined flags, duplicate flag cardinality, extra/missing import inventory kinds, names/separators and unrecorded imports.

The twelve sensitivity controls remove exactly one selected `require` statement in an isolated in-memory function copy, never editing repository sources. Each corresponding malformed probe then becomes accepted, demonstrating that the targeted gate—not an unrelated earlier failure—causes the baseline rejection. These controls cover policy/producer/model gates in both certificate readers, fixture identity typing, and import kind/joined/cardinality/inventory/unrecorded-import gates. The intentionally weakened functions were never used by compiler replays.

Reproduction details are in `focused-test-commands.json`, `opaque-commands.json`, and `independent_probes.py`. Run the latter with `python3 -B` and `python3 -B -O`; opaque replay reproduction must choose a new, nonexistent output directory outside the repository. All shell invocations removed inherited `GIT_*` and set `GIT_OPTIONAL_LOCKS=0`; Python bytecode caches were disabled.

## Audited parent results, not fresh runs here

From `/private/tmp/verus-prefix-infer79.1eVEuO/final-validation`:

- Full Python discovery: audited 455 tests passing per mode, with zero recorded command exit codes. This review did not rerun full discovery.
- Shared frontdoor: all six completed scenario receipts were independently reread and revalidated, including proof outputs, failure/publication controls and the stricter full-request/page-relations receipt checks. All 1,359 source hashes match this review snapshot and all 1,367 top-level artifact hashes match their recorded bytes. This review did not rerun the six scenarios.
- Parent opaque replays: audited 47 cases per mode, revalidated complete matrices and case bindings, and checked equality of 5,005 guarded entries before/after each run. These are separate from the fresh 47-case/mode replays above.

The receipt audit initially encountered a cross-worktree helper-path mismatch: the checking code naturally required its own isolated helper pathname rather than the recorded parent's pathname. The completed audit maps only the in-memory helper `__file__` to the recorded parent path after checking byte equality; it does not remove an artifact guard or edit any parent file. `audit_parent.py` and `parent-audit.json` record this explicitly. Its only audit subprocess is read-only `git ls-files`, with optional locks disabled.

## Loader configuration qualification

The sealed earlier nested shared attempt's stderr reports `Library not loaded: @rpath/librustc_driver-451a19fa7a19fc36.dylib` and `no LC_RPATH's found`. The supplied inherited `VERUS_USE_RUSTUP=0` explanation is consistent with the launcher: that value selects direct `rust_verify`, which requires the caller's loader setup; the nested attempt did not retain the required DYLD setting. This is not evidence that the ordinary supported rustup launch path requires a production loader change.

The parent's final-validation wrapper removes inherited `VERUS_*`, `DYLD_*`, Cargo and Rust override state before launching the shared suite; its final six-scenario run passes. As a separate fresh countercheck, this reviewer ran the full-error provider through the ordinary supported rustup launcher with no `VERUS_USE_RUSTUP` variable and no DYLD variables. That invocation compiles and verifies 23/0 successfully. Its exact command, environment-presence checks and output are recorded in `loader-supported/receipt.json`. No launcher or production loader code was changed, and no general claim about arbitrary unsupported nested environment configurations is made.

## Integrity and nonclaims

`integrity-receipt.json` reports zero failures. Own HEAD, branch and index hash remain unchanged; both native repository heads remain pinned and clean, and all guarded native/toolchain inputs retain their bytes. All 2,148 package files and 6,353 sealed delivery files retain their initial hashes. The historical original-worktree ledger was not misrepresented as a current-parent freeze; current reconstruction was instead checked against the supplied package and sealed candidate.

Two shared orchestration checkpoint ref changes were observed between read-only snapshots; no authorship or causality is attributed, and this reviewer issued no ref-writing commands. No native, original, parent or production source edits, Cargo builds, commits, branch/ref writes, pushes, PR actions or network operations were performed. Reviewer scripts, compiler outputs and receipts are private to this review output directory; the only repository write was the one requested full-snapshot application.

This review establishes bounded compatibility and conditional extracted-selection/frame evidence only. It does not establish whole native module verification, whole namespace confinement, backend/storage semantics, transaction-wide behavior, crash durability, consensus or funds safety. Production soundness approval remains the separate production review's responsibility.
