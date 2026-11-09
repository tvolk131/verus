# Completed application-led Infer follow-up

## Status

The current uncommitted follow-up is complete at cache74 / policy68 / schema38,
based on checkpoint `3b05c1912b9d7314ad00c832c4065f4362562e02` and the
`verus-infer` lineage, not main. No follow-up commit, PR, merge or Git ref update
is included. Existing PR10 was not changed by this completion work.

## Useful application results

redb now has source-connected codec roundtrip/injectivity, page-size
equation/bounds, exact safe address-range domains, same-order ordering and
nonoverlap, zero-size boundaries and a mixed-order overlap witness. The new
readonly access-prefix extraction preserves original `.into()` and
`.try_into().unwrap()` syntax and proves range/length consistency and usize
representability without an order-at-most-20 shortcut. Authored codec and
page-size callers no longer need handwritten packing/shift/product proof glue.

The access prefix is a three-field readonly configuration projection, adds a
length return, omits check_page_order/tracking/construction, and stops before
storage.read. Portable native/no-cheating checking is distinct from configured
32/64-bit verification; no native 32-bit execution is claimed. Allocator,
storage, concurrency, whole-get_page and whole-database correctness remain out
of scope.

Fedimint's deadline proof retains all seven request fields, the real pinned
SHA256 Hash type, tuple constant and both original helper items. Deadline
policy, coverage, boundaries, monotonicity and independence from unused fields
verify. Deleting the intercepted-forward guard still yields exact helper
inference but fails the independent authored policy and removes stale exports.
Unused borrowed opaque Hash transport asserts no SHA/layout/equality/trait or
cryptographic laws. Async persistent state/external effects still block the
funding caller; authenticated inputs, fees and fund safety are not proved.

## Infer and automation improvements

The integrated work supports guarded record early returns, native u64 byte
APIs, checked numeric conversions, scalar accumulator composition, variable
shifts, literal panic/Range modeling, native Option callbacks, borrowed-only
unused opaque fields and checked polynomial/bit-vector caller assistance.
Required soundness/maximality/exactness obligations and publication gates stay
active. Intent is authored separately; no application identity shortcut or
unchecked application summary is introduced.

The native-failure publication repair defers inference-aware interface output
until requested native/erasure checking succeeds. Fresh and seeded, inferred,
import-only and goal-assisted failure paths are covered. Verification-only
publication is explicitly not native evidence. Existing standard-library
specifications/frontend correspondence remain disclosed trust.

Six pinned scenarios have a common setup/CI/receipt frontdoor. Numeric access
is a dedicated seventh replay, not yet a common CI registration. Relational
receipt corrections pin domains/equations, exact counts and complete body
scope, guard both provider artifacts, bind imported identities, and require
successful prerequisites. Semantic, safety, frontend/import and checked
auxiliary failures remain separate. Warm record runs regenerate summaries;
they are not claimed as cache hits.

## Validation and review

- Parent compiler/vstd validation: 2,221 vstd checks, zero errors; 140 VIR
  library tests; 592 integration tests across 32 binaries. These precede the
  final Python-only receipt correction and are not claimed as rerun afterwards.
- Final parent Python correction: 219 tests normally and 219 under `-O`;
  fresh six-scenario replay and all 39 dedicated access controls pass.
- Independent production/publication composition review: no substantive
  finding; 434 Python test-method executions, six scenarios/122 raw controls,
  39 access controls and 80 focused compiler invocations. Two invalid CLI
  setups are excluded; 78 qualified observations include known completeness
  declines, not 78 positive successes. Larger Rust results were audited only.
- The independent receipt-correction re-check was declined by its service
  before producing a review and was not retried. Additional parent checks
  accept the genuine receipt twice per mode and reject 135 inconsistent
  variants per mode at their intended validation stage; 1,596 guarded paths
  and recorded refs match. This is not an independent correction audit.

The independent production review observed one changed shared Cargo
`.global-cache` SQLite bookkeeping file. All other 17,773 guarded original
paths, including registry inputs, match. Exclusive writer attribution is
unresolved. This is not a source/runtime/proof defect, but a blanket claim of
unchanged shared Cargo-home bytes would be false. An isolated writable Cargo
home seeded from immutable registry inputs is a future operational improvement.

No hosted Linux/manual workflow, real online dependency fetch or full
application feature-equivalence proof is added. Known clipped/casted literal
one, U128 shift-amount and constructor/formal/temporary receiver completeness
limits remain explicit; unsupported correct goals can still fail closed.

## Evidence

Prior compiler/production snapshot:
`/private/tmp/verus-infer-final74._6rqc4bo/handoff/HANDOFF.md`.
Corrected-source snapshot and fresh transfers:
`/private/tmp/verus-infer-receipt-hardening74.oQUMpJ/handoff/HANDOFF.md`.
Independent production review:
`/private/tmp/verus-final74-review-jbTSpZxJ/REPORT.md`.
Parent receipt-consistency validation and review limitation:
`/private/tmp/verus-local-receipt-validation74.ptdrgH/REPORT.md`.
Final reconstruction package and provenance index:
`/private/tmp/verus-infer-completion74.d23eXz/handoff/HANDOFF.md`.

The two historical handoffs and their raw receipts remain immutable. The final
package adds this completion report without changing the corrected source
ledger, runtime, application bodies/domains or accepted proof evidence.
