# Capabilities and authored inputs

These are bounded profiles, not coverage guarantees for arbitrary Rust. A source
shape must be eligible, its dependencies modeled or checked, and all required
queries completed. The current [guide](infer.md) explains modes and reports.

| Area | Mechanical support | Still authored or outside scope |
| --- | --- | --- |
| Scalar values | Supported integer arithmetic, branches, casts, checked/wrapping/saturating operations and composition | Intended meaning, required domain and caller property |
| Structured returns | Division/remainder, reassigned loop-free `let mut` locals (merged across branches) and panicking arms in functions returning records/Option | Mutable `&mut self` state outside the record profiles (e.g. `ref mut` patterns) |
| Callees | Inferred callees, approved models, and any callee with a functional contract (`returns E` / `ensures ret == E`), including `vstd` std specs and hand-written contracts; diverging `requires false` callees | Relational callee contracts (no automatic sound-only fallback) |
| Tuples, arrays and slices | Selected complete outputs, indexing, lengths, subslices and frames | Representation meaning and high-level content properties |
| Option/Result | Supported tagged returns, early exits, readonly callbacks and typed error paths | Application success/error policy and unmodeled callbacks |
| Wrappers and records | Supplied sequence/tuple views, supported operators/derives and bounded state summaries | Abstraction, validity, dependency laws and private semantic interfaces |
| Collections and loops | Selected construction/fold/counting, pointwise/growth/range and selector profiles | Semantic invariants, exit goals, termination/domain constraints outside recognized forms |
| Ordinary caller goals | Checked scalar/bit-vector/packing and finite-set proposals | Assertions, postconditions and semantic relationships to prove |
| Word/sequence frames | Checked generic word laws and selected update/count/padding transport | Word view, canonical representation and application invariants |
| Canonical byte encoding | Bounded u64/radix128/ten-byte arithmetic and iterator annotations | Canonical predicate and semantic prefix/exit invariants |
| Mutable-loop observations | Selected snapshots and current/future referent bridges | General untouched-tail frame and native ownership refinement |
| Foreign code | Closed recognized body-backed bool conversion and local NonZero ordering support | General foreign calls, unsupported derives/callbacks and native representation laws |
| Cross-crate use | Certified supported interfaces with dependency/model/version checks | Library trust and local-only capabilities that refuse export |
| Diagnostics and re-checks | Every rejection in one run; blocked callers; `--verify-function` partial re-checks (never certified or exported) | Readable contracts: callee preconditions are still inlined (e.g. ~13 kB for REDB's `DatabaseLayout::calculate`) |

## Application status

| Case | Established scope | Unproved boundary |
| --- | --- | --- |
| REDB bitmap/tree | Selected membership/count/frame and ancestor propagation contracts | Whole database, all native containers/callers |
| REDB headers | Selected in-memory selection/finalization and frame contracts under stated slot preconditions | Durable acknowledgement, lock/cache write-trace coupling and served-snapshot recovery |
| REDB layout | All 19 functions reached by the 2026-11-13 round-trip pilot infer exact contracts with no executable-line change (maintained regression `redb_layout_infers_without_executable_changes`) | The round-trip property's nonlinear proof, `reduce_last_region` (`&mut` record state) and `usable_bytes` (relational `unwrap_or_default` spec) |
| REDB allocator | Authored canonical validity, exact allocation/free intervals, logical rejection and conservation wrappers | Executable weighted counting, full API/native integration, durability/crash recovery |
| Fedimint quorum | Numeric policy and authored set-based intersection/honesty corollaries | Membership/admission, cryptography and runtime protocol wiring |
| Fedimint signer map | Selected insertion/replacement and initializer bookkeeping in recorded contexts | Native count consumers, recognized-peer and full signature-validation linkage |
| Canonical varint | Full selected u64 canonical predicate with supplied semantic invariants | Arbitrary encoders/iterators and automatic semantic discovery |
| HMAC pad loop | Selected prefix observation proof | Full native unused-suffix frame; original HMAC/native objective deferred and unachieved |
| Native owned-cell experiment | Historical private helper/control queries and frontend observations | Original clients, final-input-state, erasure and application coupling; archived separately |

The portable allocator and quorum sources are maintained regression inputs.
Other historical contexts, including their manual costs, exclusions and failed
attempts, are at `archive/infer-research-2026-11-12`. Do not infer full application
coverage from a successful helper, a model theorem or a finite native test.

## What counts as progress

Freeze a meaningful property and its full intended domain before adding
assistance. Report semantic definitions separately from mechanical annotations.
Measure which local assertions/lemmas/invariants are actually removed; retain
unsupported cases in the coverage denominator. A feature enabled in two projects
is not demonstrated shared value unless its assistance is necessary in both.

A useful result has ordinary checking, healthy dependency proofs, named body and
helper coverage, and semantic/safety/export controls. A false postcondition must
fail for that postcondition, not because a helper, parser or resource limit failed.
The [adoption workflow](infer_adoption.md) describes this process.
