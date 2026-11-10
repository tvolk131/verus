# Packed tag implementation handoff

The exact requested constructor closure is **UNPROVED**. This lane supplies
ordinary checked partial representation facts and precise frontend/standard-spec
blockers. It supplies zero native correspondences, no source-safety, physical
permission or control-flow certificate, and no application acceptance.

The early typed/source milestone is commit `d70148e0` (parent `53178f50`).
Everything in this lane is new under `native_miri_tag_representation/`.
No engine, production, vstd, policy, dependency, shared toolchain/cache/config,
other checkout, sibling process/build, or parent source was changed. No delegation,
push, PR, axiom, assume, external-body stub, unsafe check, or Coq addition was used.

## Exact source and domains

`selection.json` freezes Rust commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`, the two full upstream file hashes,
and hashes of eight selected declaration/implementation pieces. The selected
closure is BorTag's original seven derives, `NonZero<u64>` field, new/get/inner/
succ/one, Default; Item's original five derives, u64 field, all five constants,
new/tag/perm/protected/set_permission; and Permission's original derives,
variants, four constants, to_bits/from_bits. `fmt::Debug` implementations of
BorTag/Item and unrelated module bodies are excluded; Permission's Debug derive
is retained but outside verification scope. Full upstream copies remain private.

BorTag::new has the entire u64 domain: zero returns None. A BorTag remains a
genuine full-width NonZero, including values above TAG_MASK. succ's intended
checked-add behavior includes MAX returning None; this is not formally certified.
Item::new's source has an existing tag <= TAG_MASK assertion; larger valid
BorTags take that assertion branch. Its internal normal-return contract uses
that exact existing bound, not an application restriction or tag budget.
Item.tag is normal only if the low 61 bits are nonzero. Permission updates and
field observations apply to every packed u64, including zero. Permission has
all four original values, and protected has both bool values.

## Accepted bounded evidence

All verification commands use the authenticated accepted eight-file bundle,
ordinary full selected crate checking, `--no-cheating`, normal source/lifetime/
trait/mode/WF/body checks, JSON scope, private temp/cache/output directories,
and no selected unchecked local executable contracts. Python acceptance was run
with `-O`; gates use explicit exceptions, never Python assert.

- Four independent bitfield proof functions: **26/0 on and off**. They establish
  constructor low-tag/permission/protector roundtrips for 0 < tag <= TAG_MASK,
  tag and bit-63 preservation by permission updates on arbitrary words, exact
  61-bit range characterization, and low-field preservation with arbitrary raw
  high-flag input. These mathematical packing functions are not replacements
  for Item::new or BorTag.
- Original executable body subset: **45/0 on and off**, with 10 checked exec and
  5 checked proof bodies. Original get/inner, perm/protected/set_permission,
  Permission to_bits/from_bits, and three original Clone derives are checked.
  The get contract observes the genuine standard NonZero view and proves it
  nonzero; ghost tag_identity proves view injectivity. set_permission preserves
  low bits, protector bit, and item_word_wf iff, for every initial word.
- Explicit closure-adapter subset: **49/0 on and off**, with 14 exec and 5 proof
  bodies. This additionally checks annotated `new` using a typed closure with a
  checked closure postcondition, one, Default, and the unchanged Item.tag body.
  succ and the full Item.new are excluded. The on run also compiled successfully
  with `--compile`, retaining a private hashed rlib.
- Four bitfield and four actual-carrier **false** semantic controls each fail
  exactly one named control query, with the identical healthy helper bodies and
  complete positive companion. They reject total nonzero packed words, a 61-bit
  budget on full-width nonzero tags, an inverted protector result, and a changed
  low tag after a permission update. None is a panic/precondition negative.

All inferred-function and assistance lists are empty; no interface was published.
Identical on/off successes establish no causal inference gain. This is manual
component proof evidence only.

## Retained blockers and excluded attempts

The literal unmodified closure compiles with Rust 1.99.1. Verus, on and off,
rejects `.map(BorTag)` and `.map(Self)` as datatype constructors used as function
values. A diagnostic closure spelling gets past that frontend error and exposes
unsupported `NonZero::<u64>::checked_add`. No suggested assume_specification was
added. succ is optional for lookup but retained in the exact frozen target.

The fully annotated Item.new trial reaches ordinary checking: 49 verified / 1
failed function, exactly `component::Item::new`. Its remaining diagnostics are
source debug_assert failure obligations for tag equality and protector equality.
The constructor's executable tokens, including its assertion, u64::from(bool),
and all three debug_asserts, are unchanged after ghost erasure. This failed
trial is neither a semantic negative nor an accepted constructor.

Two narrow runtime comparison probes fail their true postconditions: original
`a == b` and `a < b` have no accepted compatibility fact with the NonZero view in
this component. Ghost mathematical equality injectivity is separately proved;
it does not certify these ignored runtime derived bodies.

A narrow `u64::from(b)` property also fails its true postcondition. The current
standard From specification conditions its result on obeys_from_spec; the
inspected standard module has no bool-to-u64 instance. The first narrow trial
was mistakenly labeled a semantic-control expectation by the controller. It is
explicitly **excluded** from false semantic evidence in RESULTS; a fresh trial
classifies it as an ordinary proof/specification gap. There are eight accepted
false controls, not nine. This is a modeling/specification gap, not a Rust bug.

Other attempts remain in the 34-epoch inventory: initial public-contract field
opacity errors; an illegal exec call in a proof block; an unsupported NonZero
view expression in a bit-vector assertion; two revisions of malformed generated
subsets (contract-brace extraction, then an orphan doc comment); an initial
ordinary proof failure; and a carrier-only run that succeeded 3/0 when the
controller had expected frontend rejection. Their original expectation failures
and exit codes are retained, not relabeled successful proofs. Pre-epoch bwrap
startup and Python `select` import failures are separately recorded. There were
no resource exhaustion results, no removed panic branches counted as negatives,
and no source-assert reachability assumed impossible.

## Fidelity, erasure and trust

`fidelity.py` checks 11 retained runtime bodies token-for-token against the frozen
original after ghost proof-block erasure. The twelfth, BorTag.new, has precisely
the explicit typed-closure adapter in place of constructor-as-function; its
closure contract erases to the typed closure body. Declarations additionally
widen tuple-field/constant visibility for public contract WF, add the Permission
Structural marker, and add Verus namespaces/return binders/ghost definitions.
These do not change the carrier to raw u64. Full Item.new is compared for fidelity
but is not accepted. The original literal, original/closure runtime comparison,
and verified adapted subset all compile. Native comparisons cover 616 field/
constructor/update observations and 5929 input pairs, including zero, TAG_MASK
boundaries, high-bit-only words, MAX, None, and existing constructor assertions.
These are supporting finite evidence, not a general native-equivalence theorem,
source certificate, or proof of any unwind behavior. `native_equivalence_proved`
remains false. No explicit macro/compiler adapter is accepted as an exact bridge.

The TCB is the pinned Rust compiler/lifetime checks, Verus frontend/macros,
logical encodings, solver, and accepted vstd bundle. Standard NonZero is opaque
and its view/new/get/Clone behavior and inverse/nonzero axioms are inherited
trusted vstd specifications, not body-checked standard-library proofs here.
Option.map/unwrap, From, core panic, bool/integer traits, and derived-method
handling likewise use inherited standard machinery. Existing vstd specifications
include their own no_unwind facts; no new no-unwind specification was introduced
or used to assert whole-Miri/application execution is defined. The exact source
of the inspected vstd specifications is additional explanatory context; the
accepted bundle identities, not an unproved rebuild equivalence, bind the run.

The original generated Hash/Eq/PartialEq/PartialOrd/Ord implementations for
BorTag, Hash/Eq/PartialEq for Item, and Debug/Hash/Eq/PartialEq for Permission are
outside the selected checked scope. Clone is selected and checked for all three.
The generated Structural marker has no translated proof body. Full automatically
ignored/generated inventory is preserved in RESULTS. No runtime comparison
property is inferred from merely having a derive or a native comparison.

## Manual inventory and lookup interface

The exact and adapted accepted components each have five authored proof bodies:
packed_fields, updated_fields, tag_range, packed_fields_raw, tag_identity. The
first four contain 19 authored bit_vector assertions. Item.perm has a proof block
with two more bit_vector assertions; Item.set_permission has a proof block with
one updated_fields call. tag_identity directly invokes the existing trusted
inverse-view axiom twice. Total: **5 proof bodies, 2 inline proof blocks, 21
bit_vector assertions, 1 local manual lemma call, 2 existing standard axiom
calls**, plus all authored requires/ensures/spec definitions. No loops/invariants
or inferred helper calls occur. The adapted closure also has an authored closure
postcondition. The failed full constructor adds a third inline proof block,
quantified packing/identity helper instantiations, and its original debug checks;
it is not included in the positive inventory.

For lookup, the exact checked fragment supplies `get(&BorTag)` equals nonzero
NonZero view, `inner` identity, ghost view injectivity, and bitfield/update laws.
The typed target remains `item_word_wf(w) = (w & TAG_MASK) != 0`; the adapted
Item.tag result view equals that low field under this predicate. The exact
Item.tag/BorTag.new implementation bridge and runtime comparison compatibility
are **not available as accepted imports**. See INTERFACE.md for the whole target.
Do not integrate the adapted constructor as the original.

Next unproved sequents are (1) faithful normal translation of constructor-valued
map, or a proved adapter equivalence; (2) actual derived Eq/ordering compatibility
with the NonZero view; (3) actual bool conversion's standard body/spec connection;
then exact Item.new's source debug assertions and roundtrips, and Item.tag's
unwrap safety through exact BorTag.new. Full succ additionally needs the actual
NonZero checked_add connection. These frontend/standard-source needs are routed
to the integration lead by this report; no work in another lane is presumed.

## Reproduction and evidence

`source_selection.py` authenticates and emits the exact frozen closure privately.
`annotate.py` emits full attempted, adapted subset, exact-body subset, and
comparison diagnostic sources. `bitfields.rs` is directly checkable.
`trial.py` authenticates canonical receipt success and all eight artifact hashes,
records frozen source/tool/controller/helper identities and argv before each
execution, terminal exits, raw log and product hashes, and final guards. Outputs,
raw source copies, binaries, caches, logs and full receipts stay private in the
explicitly named /tmp directories in epochs.json. Historical canonical support
paths are keys only; only matching helpers in this checkout are opened.

`python3 -B -O source/tools/research/native_miri_tag_representation/audit.py`
rechecks the exact named 34 completed own epochs and every private product,
revalidates accepted outcome gates, and writes compact RESULTS.json. Its default
is audit, not old-driver execution. No old baseline builders are used.

Canonical source for exact positive subset:
`/tmp/native-miri-tag-exact-final-on-48_484rs/component.rs`.
Compiled adapted subset (clearly excluded from exact bridge acceptance):
`/tmp/native-miri-tag-subset-compiled-on-hcvo5oxz/component.rs`.
Private source/product/log SHA-256 inventories are bound by each receipt hash in
RESULTS. Parent owns independent acceptance, integration and completion.

Global status remains native=0; 44 minimal + 3 phase / 101 writer effects OPEN;
full HMAC UNPROVED; publication=false; goal_complete=false; goal active.

The whole-goal canonical every-u64/arbitrary initialized exclusive [u8;10]
acceptance target (6/0 without app-local mechanical proofs/calls/snapshots) is
unchanged. The independently frozen bitcoin_hashes 0.14 exact HMAC loop target,
for arbitrary initialized [u8;128] and every valid keylen including 0 and >128,
still requires both XOR-prefix and unchanged-suffix acceptance and remains
1/1 UNPROVED. This lane did not attempt or independently accept either whole goal.
