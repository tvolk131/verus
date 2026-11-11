# Stage A code interface, after f90a6679

The earlier diagnostic stop is preserved. This continuation explicitly implements
the previously missing compiler rule; it does not require a native-memory theorem
as an entry condition. No further standalone Carrier library is planned.

Add a closed compiler-only InternalRepresentation for pinned actual core Rev<I>.
Its checked factory validates loaded semantic identity and complete nominal VIR
shape (one private exec field of the single generic parameter, no destructor or
invariant). Revalidate the actual descriptor-bearing datatype at WF/backend entry.
No source attribute constructs a descriptor. Stage A leaves the installer inactive.

Append Internal to DatatypeTransparency, carrying the closed descriptor, and split
queries: source access uses original private visibility; logical encoding uses
one ordinary nominal datatype in EVERY module. Existing Never/WhenVisible behavior
is unchanged. Pruning, monotype collection, poly coercions, type invariants, AIR
constructors/selectors and resolvedness follow encoding. Source WF, error display,
inference and constructor registration keep source visibility gates. Existing
normal struct place/borrow/current-future/writeback lowering then uses the SAME
constructor/selector. Do not assert selector or future equalities independently.

Internal is explicitly nonserializable/nonimportable (serde skipped variant), and
local-only prototype output is refused before pruning. No schema/policy/cache
constant changes. The appended Rust enum is an internal format change; no claim
of cross-version VIR compatibility is made. Later reconciliation belongs to parent.

Focused tests must exercise module-uniform nominal identity (generic and closed),
pruning, poly boxing, constructor/selector equality, ordinary field updates and
mutable-reference current/future lowering, source privacy negatives, changed
descriptor rejection and serialization refusal. Synthetic IR tests are diagnostic.
Where feasible independently validate loaded actual pinned core Rev metadata.
Build compiler/macros and full NORMAL vstd together offline/locked/jobs2 with
private cache/target/home/tmp before app proofs. Preserve immutable failed epochs.

Direct sequence View hooks validate a source body projecting a sole Vec field and
retain a certified observation after pruning; they do not change datatype encoding
or define native Rev's private field. They are not reused as selector authority.

Stage B (exact loaded rev_iter definition, checked aggregate helper/whole Function,
source routing and native minimal loop) is separately gated and may remain absent.
No native body verification or physical-memory theorem follows from Stage A.
Native0/HMAC1/1/44+3/101 OPEN, publication=false, goal_complete=false.
