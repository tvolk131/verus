# Provisional genuine wrapper ordering implementation

The local source stage is implemented. The unchanged seven-derive
`BorTag(NonZero<u64>)`, private field, arbitrary already-created genuine inputs
and shared aliases now have checked original Ord::cmp and PartialOrd::partial_cmp
bodies and numeric clients. This is provisional implementation evidence, not
parent acceptance, native correspondence or whole-goal completion.

Identical-invocation cold/warm runs each pass28/0. All FIVE candidates (existing
original Eq plus four boolean ordering functions) have true requires and fresh
soundness, full inherited-domain safety maximality and output exactness proofs:
cold five misses, warm five hits, fifteen mandatory checks on each invocation.
The helper and both original ordering derives have fresh ordinary body queries.
The four false Less/Equal/Greater/swapped clients each fail only wrapper::wrong,
28/1, with all mandatory candidate checks and required ordinary bodies healthy.

The unchanged original comparison baseline was2/7 on and off (the original six
comparison clients plus a same-reference client); macro requests and authored
requests failed frontend/profile gates. Solver authorization, own normal matching
vstd2250/0 and eight-tool freezing preceded those trials and production changes.
RESUMED_BASELINE.json seals that early milestone (commit e4e6e9f8). The original
checkpoint462c21e9 REPORT/RESULTS/SEAL/VALIDATION and missing-solver result remain
unaltered historical records. The new authorization is an explicit continuation,
not an independent re-review or a new mathematical premise.

## Implementation and typed boundary

Five production files change; two test files supply regressions. The former
NonZero ordering-excluded test now rejects unsupported arithmetic, while retaining
its non-u64 rejection. The original Item/packet fixtures and their gates are not
rewritten.

* infer_derive_ops.rs admits only PartialEq/PartialOrd/Ord on the existing
  syntactically nongeneric single NonZero<u64> field shape. Existing seven derives
  are preserved. New CLOSED proposed relations use existing unsigned u64 cmp_spec
  on `self.0@,rhs.0@`; PartialOrd proposes Some of that numeric relation. This adds
  BorTag trait candidates, not a NonZeroOrdSpecImpl or a standard-library law.
* wrapper_binary.rs admits only eq/cmp/partial_cmp through the existing typed
  genuine-core nonzero_owner predicate. Generic/lookalike/non-u64/multi-field
  shapes remain outside this profile. Original ordinary method signatures,
  trait identities, result types and complete ordinary body checks are required.
* wrapper_compare.rs proposes numeric boolean relations for that same field.
  Borrowed operators resolve Self=&BorTag; shared-reference decorations are
  normalized for the candidate owner ONLY, and that new reference case requires
  nonzero_owner. No universal wrapper/reference eligibility is added. Original
  call/arguments/evaluation/borrow handling are retained for all three checks.
  newtypes.rs already supplies the closed observation projection and is unchanged.
* infer.rs supplies a typed request predicate reused by the verifier.rs export
  barrier. It recognizes the same genuine owner and Ord/PartialOrd marker from
  actual FunctionStub/datatype data before pruning; it grants no eligibility or
  proof. This makes local-only refusal independent of Ord helper registration.

The extra wrapper-checker struct in INTERFACE.md was a proposed design sketch,
not a proof token. It was unnecessary: existing external.rs already selects the
requested ORIGINAL native derives as VerifyAsIs, so their ORIGINAL bodies enter
normal Rust-to-VIR translation and ordinary trait postcondition proofs. There is
no synthetic wrapper body, hand-written runtime implementation or descriptor
standing in for either derive. Only the foreign carrier uses the pre-existing
bounded MIR-to-actual-Function correspondence interface.

checked_nonzero_ord.rs, its structural tests, context/call/query hooks,
external.rs, automatic_derive.rs, import_export.rs, registry.rs, newtypes.rs and
all std/vstd models are byte-identical to engine913044c4/fix297332c2. Complete
same_serial type witnesses, attempt-all selected-output invalidation (including
failure at one destination), before/after-WF/autospec/mode validation, actual-loaded
MIR/Function mutations and fresh successful Normal helper identity gates remain.
No checks are disabled or replaced; no ordering certificate, import kind, schema,
cache protocol or epoch is added. Versions remain schema39/policy73/cache76.

## Actual body, source and query inventory

The complete native declaration is authenticated from our own committed record:
SHA2560107890f5ffee6b94a4179c0351a4ff121e0e61b9886067e94370a65391e9abd.
The final source contains all seven original derives and a private genuine field.
Closed value and generated closed observer/trait relations stay closed.

Exact checked chain:

1. Original total/partial borrowed clients invoke actual a.cmp(b)/partial_cmp(b).
2. Original generated Ord::cmp takes the two shared field references and calls
   actual core NonZero<u64>::cmp. The existing exact source-call hook binds that
   original typed call to the actual checked carrier helper, retaining arguments.
3. Original generated PartialOrd::partial_cmp calls original wrapper Ord::cmp and
   returns Some(result). It proves the numeric relation through that full chain.
4. The ordinary logical checked_nonzero_u64_ord% helper has the accepted Stage1
   actual nine-local/two-block MIR correspondence, two checked inverse getter
   expansions and actual nested u64 Ord call. Original genuine copies, shared
   borrows, storage order, possible unwind and all type/dependency checks remain.
5. Actual get::<u64> has its uninterpreted Copy/Transmute/Return body; actual u64
   Ord::cmp is a bound dependency on its inherited numeric model. No Transmute
   representation equality or primitive enum/discriminant semantics is added.
6. Ordinary less/le/greater/ge and inferred boolean functions retain actual
   inherited wrapper comparison calls and existing core/vstd trait contracts.
   Standard default/reference method models are inherited trust, not newly
   checked native library bodies.

The final audit inventories the actual complete VIR Function and Normal query
hashes of helper/cmp/partial_cmp, full VIR/AIR and candidate query artifacts, exact
native selected-body rows, source and tool/controller hashes. Original Clone and
requested Eq are separately inventoried. Hash and Eq marker methods are outside
selected scope; Copy has no executable method. Neither ignored bodies nor the
logical synthetic helper are counted as checked native executable bodies.

A separate read-only callback against the pinned compiler records ALL SEVEN
actual instantiated full MIR bodies/signatures: total, partial, derived cmp,
derived partial_cmp, carrier cmp, get and primitive cmp. Rustc-only macro-erased
expansion and MIR retain the original runtime calls and exclude the logical
helper. These observations and hashes bind bytes, not a source/native refinement
theorem. Raw sources/MIR/expanded/VIR/AIR/receipts/binaries stay private.

## Validation and negative classifications

The full-domain theorem is the arbitrary-input ordinary source proof with true
requires, not a finite test. All same-reference cmp/partial_cmp clients pass.
Ten numeric observations in both operand orders cover1, equality,2^61, highbit,
MAX-1 and MAX; their corpus is25/0 including the same checked derives/helper and
original clients. They are supporting observations only. Public boundary contracts
use the existing CLOSED value function; no field visibility is widened.

False application contracts are semantic negatives only after all required
helper/inference health gates. Each four-client rejection has exactly the named
wrong function failed and ordinary full-scope verification,28/1. An isolated
wrapper bit-vector auxiliary failure is37/1 with failed_functions=[] and required
helper/derive/candidate queries healthy; it exercises global failure separately.

Wrong-view (constant), weak-view (parity) and reversed-view proposals fail actual
soundness/output-exactness checks while maximality proves. The inference engine
stops dependent ordinary checking and the fresh-helper gate also refuses. These
are explicitly INFERENCE REJECTIONS, not healthy-helper semantic client negatives;
other candidates not run are not credited. A correct authored observer gives25/0.
The earlier reversed expression without an explicit u64 cast is E0308 frontend
setup, not semantic evidence. Proposal markers/views never establish a theorem.

Unsupported generic/non-u64/multi-field/missing-Ord-delegate/lookalike controls
fail closed at frontend/profile gates without an unintended concrete helper.
No panic, timeout, filter, disabled check or compiler abort is counted as a source
semantic result. Disabled and filter controls serve only export-refusal evidence.

A genuine same-runtime Bool provider/export is2/0, ordinary goals-off import1/0,
false import0/1; a genuine Cargo-produced Bool prior is also2/0. Selected explicit,
Cargo, simultaneous and equal paths use these real prior bytes, not sentinels.
The initial20 cases cover wrapper/mixed/private plus disabled checks; additional
false-wrapper/global-auxiliary/late-erasure/filter and Cargo-driver-prior cases
bring the matrix to40; eight additional partial-only on/off cases bring it to48. Every selected stale output is gone. Auxiliary Bool runs
are10/1 with failed_functions=[], late erased Rust runs are2/0 followed by E0308.
The final runtime creates no usable ordering interface or new publication path.
A provisional partial-only control exposed a policy gap:6/0 unexpectedly wrote
an interface without an Ord helper. Its digest is retained and the selected file
was immediately deleted; it was never imported or used as positive evidence.
The new typed request barrier fixes that gap, independently of helpers and goals
on/off. It preserves the valid existing PartialOrd local proof (6/0) and refuses
all eight selected routes. The final compiler/vstd/eight-tool cohort was rebuilt
and all required evidence rerun after this fix. Prototype runtime results remain
development diagnostics, not substituted final matching-runtime positives.

On the SAME final eight-tool bytes, the Stage1 direct/off/partial/aliases gates
are2/0,0/1,5/0,2/0. Exact unchanged Item132/0 and packet31/0 export their existing
ordinary interfaces; their ignored ordering bodies do not acquire a helper or
proof credit. Earlier development epochs are retained as diagnostics only and
are not substituted for these matching final-runtime gates.

The final test inventory and exact receipts are in IMPLEMENTATION_RESULTS.json.
The final matching-runtime suites PASS: VIR149/0, library8/0 and focused52/0 (existing40 plus
scalar comparison7 and new wrapper ordering5). This is not parent full717 or a
new claim for parent-owned matrices. Checks run with explicit
RUSTUP_HOME=/home/dev/.rustup, offline locked Rust1.99.1 jobs2, matching compiler,
macros and normal verified vstd. Full application proofs use --no-cheating;
that flag is not forwarded into the inherited vstd build.

## Trust, manual inputs and exclusions

Trust remains the existing Rust/frontend/MIR-to-VIR pipeline, ordinary trait and
shared-reference semantics, accepted bounded Stage1 checker, inherited opaque
NonZero View/get and u64 Ord models, logical encoding/solver, and final erased
Rust compilation. New eligibility/candidate code joins compiler implementation;
projection is an untrusted proposal checked against the original source, not a
kernel theorem or standard-library assumption. No new axiom, assume_specification,
external_body, inferred false safety domain or tag-budget premise is admitted.

Manual inputs are the scoped design and numeric relation, bounded macro/profile/
projection implementation, source fixtures, diagnostic controllers and explicit
classification/audit/report work. No application proof helper, adapter, public
field, replacement runtime implementation or new standard law was supplied.

Cargo controls invoke the actual --VIA-CARGO driver entry with a frozen Cargo
package environment and metadata emission. The selected sibling paths are the
actual paths computed by that driver.

All command/source/tool/controller freezes and last guards are private and
immutable. Final audits are read-only and exception/require-gated under normal
and optimized Python. The old missing-solver checkpoint is preserved. Excluded
setup includes an initial controller directory collision, calling exec functions
inside assertions, the unhandled reference-Self candidate baseline, boundary
public-field rejection, a reversed-view type error and a controller import-name
collision. They are retained separately, never reclassified as theorems. Test-only
fixture corrections have appended source freezes on identical compiler/vstd bytes;
production changes rebuild their own matching tools/vstd before accepted trials.

Only the newly authorized exact solver file was read from the completed review
bundle, digest e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765;
original/copy hashes and executable mode are recorded. Other private inputs are
only the originally authorized checksum-listed Cargo files and pinned Item source.
No live parent/sibling source/build/cache/config/state/process inspection, agents,
network, checkout creation, push, PR or publication/approval questions.

Native correspondence remains0. A next unproved broader sequent is refinement of
the actual compiled call chain to these model-relative source relations, including
native/provenance/borrow-frame adequacy; this stage supplies no such theorem.
Canonical6/0 and HMAC1/1 UNPROVED are inherited, all44+3/101 remain OPEN.
publication=false; goal_complete=false. Parent retains integration and acceptance.


Final local source commits, in order after checkpoint462c21e9:
e4e6e9f8 (authorized solver and baseline), ed1071d1 (original local ordering
implementation), b729d780 (request-level local-only barrier and regression),
then the evidence commit containing this report. The executing tools bind final
source bytes; commit labels do not substitute for tool/source hashes.
