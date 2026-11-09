# Selected-source branch accounting

This is a finite authored oracle and lexical source characterization, NOT a Rust
semantics proof, native producer, recovery test, or adoption recommendation.
Source anchors and **every lexical question-mark line** in the 14 inspected
functions are retained in `source_anchors.json`. These are not an exhaustive call
graph. Unsupported transitive behavior is OPEN, never an assumed inert callback.

## Cache/storage

- `cached_file.rs:490`: both stripe orders and both within-stripe orders for two
  entries in one stripe, all three failure indices, successful issuance prefixes,
  retained whole failing stripe, drained earlier stripes, untouched later stripes.
  Write Err enumerates per-cell old/intended/unknown effects and independent
  persistence choices. Unknown is a symbolic value, not every possible native byte.
- `cached_file.rs:596`: successful drain before sync; write error skips sync;
  sync error preserves the visible image and may preserve/persist durable bytes.
  CheckedBackend prior-I/O-error latch is represented. Native backend laws OPEN.
- `cached_file.rs:881`: same-offset pending replacement is an abstract enqueue
  operation, characterized against `take_value`. Its native admission is OPEN:
  required eviction/writeback (`933`, `943`), allocation/zero-fill, read path (`969`),
  budget/read-cache accounting, metrics, lock ownership and concurrent eviction.
  Eviction can issue writes before an explicit flush; the oracle does NOT assert
  that every native header admission is side-effect free, including on Err.
- WritablePage return through Drop (`cached_file.rs:40`), outstanding loans,
  cache insertion invariants, poisoning, read-cache transfer and its panics,
  atomic memory ordering and concurrency are OPEN. No drop pipeline implemented.
- `CheckedBackend::write:251` / `sync_data:242`: source checks failure, invokes
  backend and latches Err. Opaque backend side effects, panic/abort and OS/device
  guarantees remain OPEN. Successful sync copying visible to durable is a law of
  this reference test model only, not an established native primitive law.

## Header and manager commit

- `page_manager.rs:1617`: single-process buffered full-header path characterized;
  direct-write shared mode is REFUSED for the finite model, not silently buffered.
  Full header represented by three SYMBOLIC CELLS, not actual serialization,
  byte offsets, checksum/torn-write validity, selector atomicity or DB_HEADER_SIZE.
- `header_hold:1021`: own/lent source branches recorded; fallible hold errors before
  first and second header modeled. Native guard lifetime/unlock/resource binding OPEN.
- `commit:1797`: fixed geometry, two-phase, single-process Immediate is the selected
  scope. `check_io_errors` and both holds/admissions, both flush write/sync failures
  are finite failpoints. Prepared barrier precedes selector publication; public Ack
  needs final successful barrier. Memory install after publication is recorded.
- `1815` shrink and `1876` post-barrier resize errors are OPEN/outside fixed geometry.
  The latter can occur after durable publication and MUST NOT imply rollback.
  Shared-mode third hold/write (`1864`, `1865`), primary_unflushed publication and
  multiwriter tag are OPEN. The shared third write has no following flush in this
  body; do not invent one or promote single-process evidence to shared modes.
- Non-two-phase commit, transaction-id assertion, dirty pages, header/state
  mutexes, secondary-id assertion, unpersisted clearing, unwind/abort and all
  interference are OPEN. No claims about dropped transaction lifecycles.

## Public path and actual fallible continuation

- `transactions.rs:1752` delegates to `commit_with`; completed flag is set before
  poison checking (`1757`). Poisoned abort and abort failure (`1764`) are OPEN,
  not normal durable commits. `commit_inner:1770` latch arm/result/disarm is
  characterized, not a native allocation/lifecycle proof.
- `commit_inner_helper:1781`: quick repair and shared modes force two-phase;
  restored transaction state, user tree flush (`1803`), Immediate freed records
  (`1823`) and non-durable alternative (`1834`) are recorded. Immediate calls
  durable_commit (`1838`). Opaque helpers may mutate memory/pending storage or
  panic before/after returning; finite no-extra-effects witnesses are conditional.
- `durable_commit:2061`: pending-record persistence (`2070`), hold (`2074`), active
  transaction query (`2079`), free processing (`2082`), allocated-pages flush (`2086`),
  system root flush (`2091`), allocator table deletion (`2094`), checksum finalize
  (`2135`), manager commit (`2146`) have abstract Result failure points.
- Quick-repair create-table callback and loop (`2105`, `2115`, `2117`, `2125`,
  `2128`, `2132`) are collapsed to one explicitly opaque branch. Individual loop
  states, retry termination, callback storage effects, tree-last/remove and free
  callbacks are OPEN. The callback is NOT assumed semantically inert.
- After `mem.commit` success: allocation tracking, pending-commit clearing,
  system-page frees, releasing system_tables and apply_savepoint_state_on_commit
  occur before optional epilogue. These non-Result calls may panic/allocate/run
  nontrivial effects; resource/panic/abort branches remain OPEN. An explicit
  `OPEN_CALLBACK_EFFECT` path stops without Ack, including here.
- `2174`: optional enabled post-commit cleanup is a real fallible continuation,
  not an infallible bookkeeping suffix. `epilogue:2180` covers extraction (`2217`),
  root update (`2222`), system-freed records (`2228`), checksums (`2230`),
  non-durable manager commit (`2244`), tracker registration (`2252`). Each has a
  finite Err witness retaining committed user data. Epilogue early `!freed_any`
  return, disabled epilogue, and multi-process skip are distinct branches; full
  epilogue effects are OPEN, not a proof that memory/pending state is unchanged.
- Outer postcondition assertions/logging, latch cleanup, transaction/guard Drop,
  user/backend callbacks, final return glue and public error conversion are OPEN.

## Crash scope

Process restart discards pending process memory but observes the surviving
backend-visible image, CONDITIONALLY on that backend continuing to exist.
Power loss observes nondeterministic cell persistence between old durable and
visible images. Background persistence after a successful write is represented
at every event-prefix observation; write/sync errors also admit persistent effects.
The initial old secondary is stipulated valid and stale in this fixture. It is
not the freshly prepared slot before preparation. Raw selector/slot inspection
is NOT native recovery, repair, maximum-transaction selection, checksum validation
or a guarantee of recoverability. Full recovery and all eighteen cells stay OPEN.
