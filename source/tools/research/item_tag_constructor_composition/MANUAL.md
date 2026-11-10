# Manual support, visibility and trust

Accepted tag component: authored public closed BorTag::value and Item::word,
open internal item_word_wf, authored new/get/tag and companion requires/ensures.
Zero proof functions, zero proof blocks, zero snapshots, zero theorem/axiom calls,
zero local mechanical assertions, no application closure adapter and no runtime
replacement Eq. Authored roundtrip branches over ordinary standard Option results. The
boundary companion states both internal wf and low61==TAG_MASK; wf is logically
redundant at this boundary but explicitly supplied because SMT did not normalize
the shift/mask fact automatically. It permits EVERY high-flag combination, and
the main lookup theorem covers ALL internally valid words. Arbitrary full-width
nonzero BorTags remain valid independently of Item's low61 representation.

BorTag infer_derive's generated closed observation, eq_spec/obeys_eq_spec and
marker are inventoried in ordinary function_details/spec/native sets, not counted
as new runtime implementations. The original generated Eq body and the THREE
mandatory original-body obligations are checked. BorTag Hash, Eq marker method,
PartialOrd/Ord and Item Hash/PartialEq/Eq marker remain explicitly outside selected
body scope. Both original derived Clone bodies are checked. Generated ordinary
BorTag::ctor% and Item::ctor% are separately mandatory despite omission from HIR.

Private fields remain private; constants remain private. Public closed semantics
permit client contracts without exposing their definitions. Original derives are
preserved. The full Permission enum additionally has Verus Structural. Permission
runtime derived equality is handled by inherited structural derive machinery;
it is NOT a newly checked native PartialEq body in this lane. Full constructor
failure therefore supplies no full-source/native correctness conclusion.

Constructor-support positive and FAILED full constructor have one authored,
private checked proof function packed_fields_raw, containing four bit_vector
assertions. Item::perm adds one proof block with two bit_vector assertions.
Item::protected adds one proof block with one bit_vector assertion. The FAILED
full constructor adds one more proof block with a forall assertion and one local
packed_fields_raw call for each quantified instantiation. Thus support: one
proof body, two inline blocks, seven bit_vector assertions, no local theorem
calls in exec bodies; failed full: one proof body, THREE inline blocks, seven
bit_vector assertions, one quantified assertion and one checked local lemma
call. No inverse-view axiom calls, unchecked summary or helper snapshots.
This help proves arbitrary raw flag packing; it does not invent a bool conversion
law. Full new/get/tag executable statements are unchanged after ghost erasure.

Inherited TCB: exact pinned Rust/rustc/lifetime/trait checking, Verus frontend/
macros/ordinary VIR/SST and its type/mode/WF/logic encodings, inference obligation
controller/model fingerprints, Z3, matching accepted vstd/core specifications.
NonZero's abstract view, nonzero/inverse/constructor/get/Clone properties are
inherited trusted standard specifications, not body-checked core proofs here.
Option::new/map/unwrap, Fn-family standard boundary, integer/bool casts, primitive
comparisons, panic/assert/debug_assert and structural derive machinery also rely
on inherited verifier specifications/translation. Existing standard no_unwind
facts do not become an added application premise or whole-program claim.

Private export is local evidence only. Imported lookup uses checked signatures
and import compatibility/dependency machinery, not closed field-observation or
original executable bodies. The additional imported runtime Eq/value relation is
UNPROVED because distinct closed observations have no imported compatibility law.
All selected/excluded sets are explicit in RESULTS; no empty assistance list,
query count or primitive MIR whitelist substitutes for scope/body checking.

No native correspondences, source-safety, physical-permission, application or
reference C/F acceptance. Full Item::new and actual bool/library bridge remain
UNPROVED. Publication=false; goal_complete=false.
