# Stage A: implemented private-source / internal-encoding separation

**Working compiler code, inactive production installer.** Stage A implements the
previously missing lowering rule instead of requiring it as a theorem from opaque
semantics. Stage B is unimplemented: actual `rev_iter` remains uninterpreted; no
native constructor helper or useful Rev loop is newly available. This is an
experimental compiler-semantic/TCB extension, not accepted unchanged-trust work,
not an independent review, and not a native-memory correspondence theorem.
Native0/HMAC1/1/44+3/101 remain OPEN; publication=false, goal_complete=false.

Checkpoint f90a6679 and all its committed/raw evidence are preserved. Interface
commit984270dc preceded routing changes. Working implementation/tests are
104faab5. Evidence is committed separately. Parent still owns integration and
independent review. Accepted engine remains d3b2aa6e/schema39/policy74/cache77.

## Implemented rule and closure

`DatatypeTransparency::Internal(InternalRepresentation)` separates two queries:

* Source access follows the original private field visibility. Rust privacy,
  ordinary source WF, public-contract visibility and lifetime/trait checks remain
  in place. Existing Never and WhenVisible cases keep their prior behavior.
* Backend encoding is the ordinary nominal datatype in EVERY module. The ADT
  identity, constructor, field selector and generic parameter mapping are uniform
  for all logical values, aliases, parameters and post-update values of that type.
  This is not a fresh-constructor or callsite observation map.

The closed factory admits only the pinned core Rev family: one public nominal
ADT, one generic parameter, one private exec field of that parameter, one braced
variant, no destructor/invariant/extra bounds/sized constraint/extensional-equality
override. It snapshots nominal identity, parameter, proxy and owning module and
revalidates the actual datatype/field objects at ordinary WF and backend Ctx entry.
Thirteen descriptor mutations and a wrong loaded identity are rejected in tests.

The inactive rustc-side validator checks the ACTUAL loaded core stable crate ID,
ADT/field DefPathHash components using typed hash APIs, struct/variant shape,
generic declaration, field generic type and private owning-module visibility.
No spelling-only selection grants authority. There is no source attribute or
production translation path installing this mode. Rust compiler code remains
TCB; the VIR factory is not an independently proved native-identity certificate.
The diagnostic driver exercises the native-metadata validator and factory together.

## Why existing datatype/borrow rules apply

Pruning retains Internal's fields in every module and does not introduce abstract
monomorphic sorts for it. Ctx invariant analysis, poly native/boxed coercions,
datatype_to_air and SST-to-AIR type/invariant generation use the encoding query.
Resolvedness collection, datatype resolvedness axioms and mutable-reference reach
analysis likewise use the actual fields. Source/inference visibility decisions
remain source-facing. Source constructor-function registration excludes Internal.

No new selector-computation axiom, future equality, assume, trusted ensure or
standard-library contract is added. AIR uses the existing transparent-datatype
machinery for C/F and injectivity/eta, and the existing typed boxing machinery.

For the ordinary VIR place `Field(DerefMut(Local(q)))`, the unchanged normal
borrow lowering reads `Field(current(q))`, creates its normal reference temporary
p, and generates its existing two phases. Phase2 writes `future(p)` to that same
field place. Existing SST-to-AIR `update_field` rebuilds the ordinary nominal
constructor, and its inherited mutable-reference current update preserves the
outer reference's future. Thus source-visible core field operations and logical
encoding commute through the same struct-place rules. The new mode routes the
fields to those rules; it does not independently assert a native current/future
correspondence. The test adapters are cfg(test) and call the actual existing
lowerers, not a synthesized version of their equations.

Direct sequence View hooks were inspected. They validate a supplied source View
body projecting a sole Vec field and preserve a certified observation after
pruning; they do not change private datatype encoding or define Rev's accessor.
No View marker is used as Rev selector authority.

## Focused results with fresh matching tools

Own offline/locked Rust1.99.1 build, jobs2, explicit RUSTUP_HOME and private
cache/home/tmp/final target/intermediate build directories. Compiler, builtin,
macros and full NORMAL vstd are freshly matched. No parent bundle supplied app
proof inputs. Final compiler source manifests agree across build, vstd and checks.

* **157/0 VIR tests**, zero ignored. Eight new Stage A tests exercise actual WF,
  pruning, generic/closed poly coercions, boxed roundtrip, AIR C/F computation,
  eta/injectivity, false selector control, source/private/public-contract gates,
  serialization refusal, empty-function/no-helper detection, descriptor mutation,
  resolvedness, and ordinary field-reborrow/writeback. The reborrow test checks
  both current and future writeback and preservation of outer future. Removing
  ordinary writeback makes the backend check fail. The SST/AIR tests are synthetic
  IR diagnostics, not native source/function/refinement certificates.
* **2250/0 full NORMAL vstd**, no no-verify or filtered verification. The mode is
  inactive in this library; no old standard specification was strengthened.
* **Actual loaded metadata test passed** for
  Rev<Enumerate<slice::Iter<'a,genuine Item>>>. Exact core ADT hash14147836729157185704,
  field hash5029879297925419223, stable crate17281971461272426270. Wrong actual
  Enumerate ADT and a mutated descriptor are refused. Source lifetime signature,
  instantiated type arguments and target are retained in the raw output. This
  validates metadata/descriptor shape, not actual native constructor/step bodies.
* **3/0 source borrow smoke**, zero inference/helpers: ordinary private-field
  reborrow and caller checking under normal lifetime/trait/body/erasure/whole-crate
  checks. This source smoke uses existing source-visible struct semantics; the
  Internal-mode integration coverage is the explicit synthetic IR tests above.
* Actual native Rev field access fails **E0616**, and a made-up
  `#[verifier::internal_representation]` is rejected as an unrecognized attribute.
  These are source/privacy controls, not semantic assertion negatives.
* **Original genuine Item132/0**, exact allowed fixture unchanged, no-cheating,
  compile, full crate and ordinary inference checks. Its existing inventory has
  16 certified exec bodies, zero proof bodies, and one existing generated helper;
  the descriptor adds zero helpers. No cold/warm improvement is claimed.

Exact body/helper/trusted-model inventories are retained privately and hash-bound
by RESULTS.json and INVENTORY.json. Old wrapper28/0 with5candidates/15freshchecks,
packet31/0 and bool provider/import results remain inherited, not rerun here.
No native Verus reverse-enumerate loop success or tagged fallback is claimed.

## Local-only boundary and format

The selected pre-crate is inspected before pruning for Internal, independently
of infer-goals/helper counts. If present, export is refused and the existing
all-destinations invalidator clears explicit and Cargo destinations. The helper-
independent detection is unit-tested with an empty function list. The new early
export branch has no source-reachable producer yet; its complete activated
end-to-end refusal matrix remains a Stage B/integration obligation.

The Rust VIR enum has a new variant. That variant is explicitly serde-skipped:
serialization of it fails and deserialization cannot construct it; source code
cannot name an attribute to install it. Existing modes' semantics are retained;
no cross-version format compatibility or new import/publication authority is
claimed. No global schema, policy or cache epoch changed. Parent owns later wire
and semantic-definition fingerprint reconciliation. Before activation, Stage B
must bind the exact observation definition/dependencies and enforce old producer,
core, target and semantic-binding mismatches; Stage A does not claim those gates
complete merely because its descriptor is nonserializable.

## Evidence and remaining work

Final build root: /tmp/rev-stage-a-05c25_w6. Fifteen immutable command epochs retain
all input/source/tool freezes, logs and receipts. Final eight tools, pinned solver
mode/hash, pinned compiler/core artifacts and the exact offline archive/index/git
inputs are guarded. Intermediate Cargo build files also stay inside this private
root. Executed controller versions are archived by hash before small replay-path
improvements. No network, shared mutable cache, live sibling/parent source/target/
process reads, subagents, shell worktrees, push or PR were used.

Excluded setup/test errors remain intact: a test ownership typo, an invalid test
placement of explicit Box/Unbox before poly (panic), and pinned rustc API/module-ID
mismatches. They are not semantic negatives. Earlier successful subsets/builds are
superseded rather than pooled with final matching results. audit.py performs
read-only explicit-exception acceptance identically in normal Python and -O.

Stage B remains: exact loaded vstd rev_iter identity/declaration/attribute binding
and selector DEFINITION, actual source/type lowering installer, actual constructor
MIR/source-call routing, independently validated whole emitted Functions before/
after WF/autospec/modes, fresh ordinary helper bodies, active local-only/cache/
fingerprint/refusal controls, and original native identity/lawfulness/forloop
checks. No native Iterator/Rev body is silently treated as verified. No native
safety, physical permissions or application alpha/current/future premise is added.
