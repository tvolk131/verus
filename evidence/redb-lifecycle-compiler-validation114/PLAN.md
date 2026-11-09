# Isolated pinned compiler check/test114

## Objective and binding

Mechanically validate revision2's real Builder/elaborated-MIR capture/refusal
source and its five pure capture-status tests. A successful build/test is not
native resource semantics, compiler correctness/source attestation, E2, a native
producer, cache correspondence, or a final database obligation. The native
Mutex/BuildHasherDefault admission and original lifecycle proof remain OPEN.

Revision2 source root:
`/private/tmp/redb-lifecycle113-origin-revision2-20261108T145432Z`.
MANIFEST.json SHA256:
`a586fe4665ee832499ea233969eb2914b941093186596f849d8cfeea9321c342`.
Authenticate all21 listed payloads and the three baseline paths before overlay.
Base whole source snapshot:
`/private/tmp/verus-crash-consistency94-20261105/drop-effects-parent-002/staged`.
Do not substitute the changed live parent or mix historical vstd/runtime sets.

## Fresh engineering limits

First parent clock2026-11-08T15:06:02Z; hard handoff16:06:02Z. Stop optional
work at15:54:02Z and reserve12minutes for receipt audit/retention. One compiler
worker; Cargo jobs1; at most two compiler-producing commands total: one check,
one filtered metadata-unit test. Count a reserved/failed setup as a consumed
command slot if it was intended to launch that check/test. No automatic third
compile, general test suite, binary release build, vstd build or native probe.

Keep fresh source/target/log/retained bytes below8GiB and global free disk
strictly above16GiB; sample before/during/after long commands and stop if unsafe.
The worker has <=40minutes from first clock, including staging/preflight, and
<=20minutes per actual compiler command, with earlier global deadlines applying.
These are engineering bounds, not theorem domains or repeated approval gates.

## Required execution shape

1. Stage only frozen source and necessary workspace dependency source trees in
   fresh/private tmp; overlay exactly three candidate files. Preserve baseline,
   source delta and all failures. Directly use pinned1.99.1 tool executables and
   actual sysroot in place, not an unrecorded rustup shim or toolchain copy.
2. Preserve Cargo.lock and operate offline/locked. Inspect exact workspace and
   offline support first. Existing cache sources may be read, not copied or
   cleaned; never update Git checkouts or fetch. If a required sandbox-approved
   cache lock is outside writable roots, use the normal scoped tool approval
   mechanism rather than aliases or bypasses. Stop for genuinely missing inputs.
3. The old build.rs eagerly demands Git even with VARGO_BUILD_* overrides.
   Apply only the honest metadata adaptation allowed by AUTHORIZATION: explicit
   unversioned/snapshot identity bound to an inventoried source digest, no fake
   Git HEAD/checkout, fabricated provenance or hidden helper. Record exact delta.
4. Before each compiler command record reserved slot, argv/cwd, allowlisted
   environment, direct/transitive executable and source/dependency identities,
   actual rustc/sysroot/support paths, fresh target location and resource sample.
   Retain full stdout/stderr, true child exit status, timeout/cancellation,
   source identity changes and actual test list/count/results. Kill/reap owned
   process groups on timeout/cancellation and never seal incomplete success.
5. Run the approved check, then the filtered origin_capture_status_tests command.
   Corrections must be small, disclosed and still within the two-command cap.
   A failed first check followed by a passing corrected test build is not a
   retroactive passing normal/non-test check. Distinguish those scopes.
6. After the worker's actual handoff, parent audits raw receipts and may delegate
   one source/receipt-only independent review, <=8minutes, no compilation/tests.
   Do not launch an automatic extra review/build chain. Retain sealed partial
   evidence when any cap/preflight boundary is reached.

Only scratch authored files may change, using apply_patch. Parent source, old
native/frozen evidence, runtime/toolchain/cache contents and previous allowances
remain unchanged. Compiler-source/provenance limitations are explicit, not
waived by successful compilation. S0/Infer/full-scope/import/export remain
fail-closed. All18 final cells remain OPEN regardless of check/test outcome.
