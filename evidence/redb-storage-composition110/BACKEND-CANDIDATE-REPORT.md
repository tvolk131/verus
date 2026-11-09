# Round110 lane B: source-connected backend/control attempt

## Disposition

**SEALED CANDIDATE ONLY. Native binding remains blocked; zero original native
bodies verified.** A conditional extracted proof library passes a full-crate
existing-verifier check: final counter `verified=15`, `errors=0`, selected scope
complete, **13 checked proof bodies, zero checked executable bodies**. It proves
arbitrary finite control histories, not native Rust, bytes or durability.

No final-cell closure, native producer adoption, trust acceptance, integration or
gate change occurs. All eighteen final cells remain OPEN. Original native path
preservation remains FAILED, Git continuity UNAVAILABLE, frozen compiler/source
correspondence UNESTABLISHED. Diagnostic107-D1 remains unapproved. No excluded
admission/lifecycle/Drop/growth/mutation work was resumed or rerouted. The parent
must run the ONE combined independent review; this lane ran no subreview and no
subdelegation, read no lane A in-progress output, and made no parent/native edits.

## Authorization, authentication and resources

- Scope: frozen round110 PLAN hash
  `5ac77193513096bade9dc5cae35c8a4a7237ddbee6b1f527ec8c99c92edac377`;
  INPUTS hash `8efaf2981eeaee800cdab449fdfa394747675f171ceebbd90d96cafe439fa9c9`;
  AUTHORIZATION hash `a6a8e4632a709b25cc33d852d8b963cdc4e322db542ad733225a198ba48407e2`.
- First observation 2026-11-08 04:38:56 UTC. Own 25-minute cutoff 05:03:56 UTC;
  optional cutoff 05:00:56 UTC. Both precede the separate absolute handoff cap
  05:05:00 UTC. Round deadline is still 05:33:26 UTC; no budget extension.
- Fresh scratch only: `/private/tmp/redb-backend110-B-cCGFf2VR`. Charged verifier
  applications **10/16**, native diagnosis **1/3**. Every launch is bounded to
  90 seconds, one worker, no unrelated manual externs. Maximum observed verifier
  runtime is below two seconds. No engine changes, builds, downloads/network,
  native executions, toolchain/cache copies, Git writes or new trusted contracts.
- All **838 current selected inputs** authenticate at entry, before each launch
  and at exit. Current round110 hashes are used; the older historical drifts and
  two reconciled GOAL/STATUS documents are not silently replaced by older hashes.
  Parent GOAL/STATUS remains unchanged throughout this lane's authenticated scope.
- Direct framework Python, verifier launcher, rust_verify, z3, matching runtime
  vstd/builtin artifacts, pinned toolchain and loaded Python module identities are
  retained. Exact verifier argv and **entire configured environment** are in each
  invocation. No intentional Python/rustup shim execution. Bootstrap displays
  preceded entry; some packaging modules are first identified at evidence-entry.
  This is not retroactively complete first-use/exhaustive loader provenance.
  `BOOTSTRAP-LIMITS.md` discloses raw-bootstrap retention limitations explicitly.
- At 04:49:13 UTC: elapsed 617.906 seconds, scratch 6,750,668 bytes, free disk
  52,379,693,056 bytes. Initial free disk was 53,209,600,000 bytes. The lowest
  verifier-boundary sample was 48,771,522,560 bytes, strictly above 16 GiB.
  Final elapsed/scratch/free and archive overhead are sampled in HANDOFF.json;
  retained files remain far below 768 MiB. These are sampled boundaries, not OS
  quotas or claims about unsampled concurrent global disk use.

## Actual original-body probe and precise blocker

Attempt 01 preserves exact source statements/types for the selected native
CheckedBackend fields and `check_failure`, `close`, `sync_data`, `write`, and
`write_best_effort`, plus exact native trait/error/Result definitions needed for
type context. Its standalone imports/wrappers and omitted unselected items are
fully disclosed in `native-preservation.json` and `PROJECTION-AUDIT.md`.

The existing matching verifier actually runs. It fails native binding/translation:

```
error: `core::io::error::Error` is not supported
error: `core::panic::location::Location` is not supported
```

The first diagnostic names the native BackendError/StorageError/io conversion
context; the second names native StorageError::LockPoisoned. Both full diagnostic
streams and original selected source are retained under attempt-01. No export
exists at its fresh requested path. The suggested external-type specification was
NOT added. Real AtomicBool loads/stores, Box/dyn backend calls, std Result
is_err/map_err, callbacks and memory/I/O have not thereby been checked or bound.
No assume_specification, external_body, admit, axiom or substitute trusted
primitive contract was introduced. A single actual probe sufficed to establish
this local blocker; further native-type weakening was not attempted.

## Checked fallback scope

Final source: `backend_control110.rs`, SHA256
`149da8096fcbcc95a329c3cd01bca9007def9461145a5f31f0ed2e590feeae07`.
This is an authored spec/proof crate, not adapted scalar code mislabeled original
Rust. The exact patch is `backend_control110.rs.patch`; chronological source
changes, including the failed witness correction and both semantic mutants, are
retained per attempt.

1. **Exact preflight precedence for all four Boolean states.** io_failed is tested
   first; closed only selects DatabaseClosed when io_failed is true. The explicit
   state io_failed=false/closed=true returns Ok. A closed-first alternative is
   proved different. `closed ==> io_failed` is only a preserved complete-operation
   single-thread invariant from clean state, not an atomic observation premise.
2. **Four distinct operations.** Normal Write and Sync preflight then latch any
   backend Err and preserve its abstract I/O error payload. BestEffort preflights
   but an isolated backend Err leaves flags unchanged. Close bypasses preflight,
   sets closed then io_failed BEFORE the fallible backend call; even close Err
   retains both flags. An explicit three-state store-order projection exposes the
   intermediate closed-only state. Another explicit Close still calls the
   backend: only subsequent selected guarded operations are proved to refuse.
3. **Arbitrary finite cross-call histories.** Recursive symbolic history proofs
   preserve failed/closed flags, sequential reachability and per-event laws for
   arbitrary choices, outcomes and unrestricted effect tokens. After a normal
   backend Err or Close, every later selected guarded call is refused without a
   backend invocation. Arbitrary successful guarded prefixes are counted and
   retain their effects before a later failure; the failing backend call can
   itself have an effect. Err is never equated with rollback.
4. **Full projected errors and callback order.** All seven StorageError cases and
   projected payloads are represented. Abstract callback continuation returns the
   first error unchanged, counts the successful prefix, includes the failing
   call's effect and does not call the remaining abstract callbacks. The proofs
   quantify over arbitrary finite callback histories and arbitrary error payloads.
   Nonvacuous witnesses cover normal write/sync error, best-effort error, close
   error, successful write+sync, and a post-publication-marker Corrupted error
   after one successful callback with effects `[37,88]` retained.

The final checked proof bodies are exact_precedence, step_contract,
history_contract, failure_then_arbitrary_suffix, error_payload_transport,
close_store_order, successful_guarded_prefix, successful_prefix_error_progress,
continuation_contract, callback_error_transport, first_error_order,
nonvacuous_witnesses and source_negative_counterparts. Existing scope JSON records
all selected specification definitions separately. Engine counter **verified=15**
is retained exactly; an exhaustive VC/SMT-assertion count is not exposed and is
not fabricated by relabeling that counter. Scope exec-body count is zero.

## Attempts, semantic negatives and publication observations

| Attempt | Actual outcome | verified/errors | Fresh requested export |
| --- | --- | --- | --- |
| 01 | Native type/binding diagnostic, exit 1 | not supplied | absent |
| 02 | Initial witness proof failed, exit 1 | 10/1 | absent |
| 03 | Corrected full-crate positive, exit 0 | 11/0 | present |
| 04 | Normal error no longer latches: semantic failure, exit 1 | 8/3 | absent |
| 05 | Corrected counterpart, exit 0 | 11/0 | present |
| 06 | Byte-identical restored positive, exit 0 | 11/0 | present |
| 07 | Extended prefix/store-order/transport positive, exit 0 | 15/0 | present |
| 08 | Incorrectly latch best-effort Err: semantic failure, exit 1 | 12/3 | absent |
| 09 | Extended corrected counterpart, exit 0 | 15/0 | present |
| 10 | Extended byte-identical restored positive, exit 0 | 15/0 | present |

Attempt 02 needed additional recursive reveal fuel only in the concrete witness;
the failed source and four emitted postcondition diagnostics remain untouched.
Attempt 04 removes the single normal error-latch expression, and 08 replaces it
with unconditional backend-Err latching, incorrectly affecting BestEffort. Both
reach semantic postconditions in step_contract, nonvacuous_witnesses and
source_negative_counterparts. They are NOT parser/frontend rejection controls.
Failed full-crate runs are not credited with certified checked bodies just
because the raw engine verified some units.

Sources 03/05/06 are byte-identical with SHA256
`b108c30b6bb0a77310ff2efe986e31c3bfa8183360f63399de3bd50a22b3830c`.
Sources 07/09/10 are byte-identical with the final SHA above. All ten attempts,
positive AND negative, explicitly request `--export` at initially absent paths.
Exact destinations are
`/private/tmp/redb-backend110-B-cCGFf2VR/attempt-NN/backend_control110.vir`,
except 01 uses `native_probe.vir`. Each receipt records exact presence/hash after
the actual exit. Different positive export hashes are not source-byte differences:
the exports originate at different fresh source paths. No separately imported
binary, .rlib/artifact pair, S0 admission or native producer acceptance is claimed.

## Source audit, effects and explicit limits

`SOURCE-ANCHORS.json` retains exact current snippets and hashes. Native
cached_file.rs hash is
`cb901d44531b23f87d1bd31f72a24f1dc69308b5417db2167b7508921ffda486`;
transactions.rs hash is
`02b383d6d984a14dfc4827ec04241d78637591f91ff754b42f9671b305d18bde`.
Current method anchors are check_failure 195-205, close 207-213, sync 242-249,
write 251-258, best-effort 263-266. len/read/set_len use the same latch template;
unlock 178-186 checks ONLY closed and is deliberately NOT covered by guarded.
No latching law is broadened onto all lock methods or arbitrary native calls.

The actual transaction continuation 2140-2178 and helpers at 1870 and 2180 onward
are not inert. After mem.commit succeeds it drains allocation tracking, clears
pending non-durable commits, frees allocator pages, applies savepoint state and
may execute a fallible post-commit pass. The pass's callback frees pages; it also
flushes table roots/checksums, records allocations and calls non_durable_commit.
Those actual effects/panics/resources are NOT verified by generic Result
transport. The model's published marker cannot frame native durable history.

Backend effects are arbitrary/unbound tokens, including on Err. There is no byte
image, crash/durability law, native callback frame, actual Result payload encoding,
heap/resource semantics, overflow/OS behavior, Acquire/Release implementation,
atomic-snapshot/concurrency proof, or complete native adapter R. The ordinary
existing verifier/vstd trust base is unchanged; no new trusted assumptions.
The older review's UNKNOWN failed-write effect vs old/new byte-model difference
is not collapsed: this lane deliberately constrains no backend byte effect at all.

## Minimal next dependency cut and handoff

The reusable interface is the extracted Flags/Choice/Event/history and Result
continuation lemmas, listed in PROJECTION-AUDIT.md. A later adapter must supply a
checked relation for the real native observations, dynamic backend calls, exact
payload encodings and source call order under an explicit sequential/concurrent
regime. Independently it still needs backend effect/crash and post-publication
callback/history frames; control results alone cannot provide them. Fixing this
native binding blocker is not authorization to add trusted stand-ins or builds.

`RESULTS.json` contains exact counters, statuses, hashes and failure kinds.
`RESOURCE-SAMPLES.json`, each invocation/receipt, authentication snapshots and
`packaging-identity-comparison.json` retain provenance/accounting. `MANIFEST.json`
hashes the sealed payload; `SEAL.json` binds report/manifest/exit authentication;
`handoff.tar.gz` retains it and is read back against every manifest record.
`HANDOFF.json` is the external archive digest, final sample and handoff pointer.
No source changes or further verifier applications are planned after sealing.
