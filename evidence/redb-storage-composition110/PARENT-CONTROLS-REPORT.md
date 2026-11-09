# Independent parent error-history controls110

These Python-only controls challenge the selected native-control projection.
They are finite evidence, not native execution, atomic/concurrency semantics,
backend byte laws, a resource producer, a universal proof or acceptance.

## Source connection and domain

The pinned recovered cached_file.rs SHA256 is
cb901d44531b23f87d1bd31f72a24f1dc69308b5417db2167b7508921ffda486.
Five selected CheckedBackend bodies have exact normalized token guards:
check_failure, close, write, sync_data and write_best_effort. The reused round109
lexer strips selected comments/strings and balances braces; it is NOT a Rust
parser, type resolver or formal extraction. Other methods and external effects
are not silently included in this projection. Six source mutant/corrected pairs
are lexical MISMATCH/MATCH observations, not verifier rejections.

The independent sequential oracle covers normal write, sync, best-effort write
and explicit close. Every backend call has an independent Boolean Result and
an arbitrary one-bit opaque effect token. Returning Err does not discard that
effect, and returning Ok does not imply an inert callback. The token is not a
physical storage image or adopted I/O contract. Refused calls do not consume a
supplied backend outcome within this interpreter; real environmental/interference
effects remain outside the model. No destructor/lifecycle pipeline is examined.

Important source-specific controls distinguish:

- io_failed is tested before closed; closed=true/io_failed=false alone does not
  reject in check_failure. This state is included rather than hidden behind an
  unproved atomic-snapshot or single-thread reachability assumption.
- Normal write/sync Err permanently latches failed in this sequential projection;
  best-effort Err alone does not. A following normal write is still admissible
  after best-effort failure, but not after a normal latched failure.
- Explicit close sets both flags BEFORE calling the backend, even when that
  callback returns Err. The method itself has no preflight/reclose guard;
  actual permitted call count and lifecycle bindings remain OPEN.
- Backend error transport and arbitrary callback effects remain distinct from
  preflight PreviousIo/DatabaseClosed refusal and never imply rollback.

## Actual checks and preservation

Thirteen unittest tests pass in attempt-001 and the hardened attempt-002.
The second runner parses the actual unittest success count and compares complete
entry/post-test/exit selected identity records. The first successful run is
retained unchanged; its runner reported a literal expected count, subsequently
hardened without a failing test or retrospective rewrite.

Exhaustive finite enumeration checks all four initial flag combinations and
all operation/Result/effect schedules of length zero through three:
17,476 histories, 51,264 steps, 24,168 backend callbacks and 27,096 preflight
refusals. Six semantic mutants violate their targeted control property, and
each corrected counterpart has zero violations. This finite history bound is
explicitly NOT arbitrary-history proof credit. The two delegated proof lanes
are separate implementation attempts subject to independent combined review.

Each attempt retains source snapshots, exact subprocess argv/environment, actual
status, raw stdout/stderr, source/semantic controls and selected authentication.
All 838 frozen round110 inputs and selected direct Python/lexer dependencies
match before tests, after tests and at exit. Loaded stdlib/OS/loader closure and
atomic first-use bootstrap identity are not exhaustively established. Existing
native path/Git and frozen engine-source qualifications remain unchanged.

No verifier/solver, build, native application, network, Git write, parent source
integration, trusted assumption or gate change occurred. New files are solely
round110 scratch/retained evidence. All eighteen final cells remain OPEN.
