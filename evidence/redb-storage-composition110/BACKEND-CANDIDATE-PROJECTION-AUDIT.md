# Source preservation and conditional projection

This is a source-connected **extracted mathematical model**, not original-Rust
verification. `native-preservation.json` hashes each exactly copied native probe
fragment. `SOURCE-ANCHORS.json` retains current native excerpts, exact inclusive
line anchors, whole-source hashes and excerpt hashes. The original files and the
parent's selected inputs remain read-only. No library or native application build
or native executable was run. Native constructor/destructor/lifecycle paths are
outside this lane's proof scope.

## Native binding probe

`native_probe.rs` contains the exact native `StorageBackend` trait, `BackendError`
and `StorageError` enum definitions, `From<io::Error> for StorageError`, `Result`
alias, `CheckedBackend` fields and selected `check_failure`, `close`, `sync_data`,
`write`, `write_best_effort` method text. The standalone crate supplies imports,
`extern crate alloc`, a `verus!` wrapper and an implementation wrapper; it omits
unselected modules, methods, derives and enum attributes. It uses real native
types/trait statements, not invented same-named primitive stand-ins. Those
adaptations do not justify a native proof; none is claimed. Attempt 01 actually
reaches the verifier, then fails translation on `io::Error` and
`panic::Location`. Its suggestion to add external type specifications was NOT
followed. AtomicBool/dynamic calls/Result/callback semantics were not reached as
successfully checked native control. There are zero checked original bodies.

## Branch-by-branch projection

| Exact native method or source | Extracted representation | Limit |
| --- | --- | --- |
| `check_failure`: load io_failed first; only if true load closed | `preflight(Flags)` and `exact_precedence` | Two ordinary Boolean observations, not Acquire semantics or simultaneous atomic snapshot |
| `write` / `sync_data`: preflight `?`, call, `is_err` latch, `map_err` | Distinct `Operation::Write` / `Sync`, `step`, `map_backend` | Original Box/dyn call, std Result implementation and atomic stores unverified |
| `write_best_effort`: preflight then direct write/map_err; no store | `Operation::BestEffort` and unchanged flags on backend failure | No byte/write-buffer/cache admission model |
| `close`: closed=true, io_failed=true, then fallible close | `close_stores`, `close_store_order`, unconditional close branch in `step` | Store order is a sequential event projection, not a concurrency proof |
| io Error conversion | `Err(int token)` to `Err(ProjectedError::Io(token))` | Token has no native io::Error representation binding |
| complete `StorageError` enum | Seven `ProjectedError` cases in native discriminant order | String->byte sequence, usize->nat, io/location->opaque int require unproved payload encodings |
| arbitrary repeated calls | `history` recursively processes an arbitrary finite `Seq<Choice>` | Selected four operations only; callers decide whether to keep calling after Err |
| successful operation prefix followed by error | `successful_guarded_prefix`, `successful_prefix_error_progress` | Prior effect tokens survive in the history; no rollback or memory/I/O frame follows |
| sequential caller `?` error ordering | `continuation`, `first_error_order`, `callback_error_transport` | Generic Result transport only; not a verified adapter for actual transaction continuations |

`Flags { io_failed:false, closed:true }` is explicitly supported and preflights
successfully. `incorrect_closed_first` is proven different on that state.
`closed ==> io_failed` is derived only for complete single-thread operations
starting clean: `history_contract` preserves it. It is not silently assumed for
arbitrary states. The intermediate close state after the first store can be
closed without io_failed, and other threads might observe separately loaded
values. Nothing here establishes an atomic observation, Acquire/Release
implementation, linearizability, data-race freedom or concurrency invariant.

Normal write/sync backend Err and explicit close, including close Err, permanently
latch this sequential model across arbitrary finite suffixes. A later **guarded**
operation refuses without calling the backend. Explicit close bypasses preflight
and still calls it even from failed/closed state: no blanket claim that every
method refuses after close. Successful normal/best-effort backend results propagate
Ok; failures preserve the abstract error token. The backend may already have
arbitrary effects when returning Err. `backend_effect: Some(choice.effect)` is an
unrestricted uninterpreted token, not a state transition law for bytes, file size,
memory, resources, crash images or durability. `None` means only that this modeled
call did not invoke the backend, not that the native world is framed.

The audited native `len`, `read`, `set_len` methods share the latching template,
but this crate does not pretend to bind their non-unit return/data types.
`unlock_range` checks closed only and intentionally allows a prior I/O failure;
it is not included in `guarded`. Other lock methods preflight but do not use the
normal I/O-latching template; they and BackendError::Unsupported conversion are
audited, not broadened into the four-operation theorem.

## Callback and public continuation cut

The exact transaction excerpt is AFTER successful `self.mem.commit(...)?` and
contains allocation-tracker draining, clearing pending non-durable commits,
allocator frees, savepoint state application and a conditional fallible
`process_data_freed_pages_after_commit(...)?`. The savepoint helper locks and
applies transaction-tracker state. The epilogue excerpt frees pages inside an
extraction callback, flushes roots, stores system-freed pages, finalizes checksums,
records allocations and calls fallible `non_durable_commit`. These are real
effects and possible errors, NOT inert annotations. Locks/assertions/unwind and
resource effects are also outside the checked model.

`continuation` deliberately accepts arbitrary per-callback effect tokens and
all seven projected error variants. It retains the failing call's effect and
every successful prefix effect, returns the FIRST error with its full projected
payload, and does not call the remaining abstract callbacks. A nonvacuous witness
has a successful effect 37 followed by Corrupted error effect 88; the marker stays
true, successful-prefix count is one and both effects are retained. The published
marker is an input record, NOT a durable-byte invariant. This proves no native
callback/durable-history frame. Actual source callbacks, public return transport,
panic behavior and post-publication durable preservation require a separate
native continuation/side-effect relation; result equality cannot supply it.

## Source-negative controls and interface

`source_negative_counterparts` checks that normal vs best-effort Err produce
different latch states, a closed-first branch is wrong on the explicit exceptional
state, a guarded operation is refused after close, but another explicit close
still calls. Attempts 04 and 08 additionally mutate the actual extracted step
body: respectively removing normal-error latching and incorrectly latching
best-effort. Both reach semantic postcondition checking and fail; neither is a
parser/type rejection. Corrected/restored source hashes are recorded separately.

Reusable conditional interface for lane A later: module `backend_control110`,
`Flags`, `Choice`, `Event`, `step_contract`, `history_contract`,
`failure_then_arbitrary_suffix`, `successful_guarded_prefix`,
`successful_prefix_error_progress`, `continuation_contract`, `first_error_order`.
An adapter would have to supply observed native flags/results, exact call order,
payload encodings and effects independently; no import was made from lane A and
no in-progress lane A output was read. This is not native adapter R.
