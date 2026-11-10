# Independent review of three delivered components

Prepared integration head: `3f865efc`; early review scope commit: `21ba598c`;
replay harness/input freeze commit: `8962930b`. Only new files under
`evidence/native-components-model-review/` were changed. Raw outputs and exact
input/product maps remain private under `/tmp/native-components-model-review-3f865efc/`.
No subdelegates, other checkout inspection, unfinished original-reference-exit
output, shared build/install, production/compiler/vstd/controller edit, attack
experiment, push or PR was used. An initial overbroad tracked-file name listing
was unnecessary; subsequent inspection was restricted to the named scopes,
their completed dependencies and prepared Git history.

All three **mathematical components are accepted at their stated conditional or
MODEL scope**, after the independent inventory supplement described below.
**No native/source correctness certificate is accepted.** Two delivery issues
are routed to integration; neither is a failed semantic theorem.

## Findings for integration

1. **P2 — constructor direct replay is broken at the current integration head.**
   `native_constructor_erasure/check.py:135` rejects the current
   `source/tools/research/mutable_buffer_goal/TARGET.md` before any Coq command.
   Trial006 expects SHA256
   `787a17738b564b11429acd26bee483b5fe3952e2bce94941b98a90d26b6af1ac`;
   the prepared integration version is
   `608c03f4ad1bd851d317ae4ef2d3fbec832c7ee7c68e111ab54b039c9999b08c`.
   The exact file at delivered commit `3d668d29` matches the freeze. A private
   replay root was reconstructed from the frozen file list, using prepared Git
   history for that one changed document; every file, controller and candidate
   remained byte-identical to its frozen hash. That exact delivered replay
   passed. The direct integration failure is retained separately as
   `constructor-integration-drift/receipt.json`, success=false, commands=[].
   Reject the claim that the documented command currently replays directly in
   this integration checkout. Root should supply a newly frozen/replayed
   integration receipt acknowledging the changed documentary input, or document
   the exact historical snapshot procedure. No implementation file was repaired.

2. **P2 — pointer worker inventory omits 154 generated inspection helpers.**
   `native_pointer_distance/check.py:164` emits two `Goal True ... Qed` markers
   for each of 77 declarations. `Print Module NativePointerDistance.Assumptions`
   discovers `Unnamed_thm`, then `Unnamed_thm0` through `Unnamed_thm152`:
   154 additional opaque proofs of True. Lines 173/189 record only the 77 source
   declarations. The original recursive kernel command does include the entire
   Assumptions module, but the delivered per-declaration assumption inventory
   does not include those helpers. Reject a claim that the delivered 77-entry
   list inventories every declaration in its generated products. This review
   independently printed every helper's statement and assumptions and recursively
   checked the whole module: all 154 are closed. `Print Module` calls opaque
   theorems “Parameter”; individual `Print Assumptions` confirms these are proved
   constants, not new axioms. Root should inventory generated inspection modules
   dynamically or use declaration-free `Redirect` inspection as constructor does.
   The review supplement closes this evidence gap for the present mathematical
   acceptance; the delivered controller remains unchanged.

## 1. Existing-caller ordinary writer — ACCEPT, MODEL only

Exact scope: `a8df10a0`, proof candidate `5b329568`, accepted controller `0e984f41`,
`trial003-freeze.json`. Named receipt SHA256:
`1b9063d276c3ef505562143614531a0191230ee91bf3ca07d5b2c63d2af82f4c`.

The four interface definitions and `caller_xor_na_verified` retain the existing
`typeG`, E/L/I, original parent ownership and shared key view. They quantify all
finite n/k and byte contents, including empty and >128 keys. The actual body is
`zip_transform_na byte_xor` for min(n,k). The imported ordinary loop uses Na1
reads/write; no mathematical XOR helper is copied into the new scope.

The proof reuses accepted caller9's `shorter_run_resources`,
`byte_child_transform_guarded`, `xor_bytes_exact`, `ordinary_alive_under_child`
and the accepted ordinary loop/session library (21 source declarations).
The reader splits the single ordinary child L into R/G. R enters the actual
physical callback and is returned; G guards unique child access. The same child
payload is closed by the inherited accessor before `Join R G` reconstructs L;
the inherited shorter-run protocol subsequently ends only the child lifetime.
The parent at κ, its arbitrary ancestor Frame, shared key view and original
future are preserved. The postcondition gives exact min-XOR current contents,
length, unchanged suffix and byte ranges. There is no fresh world, extra
physical/range/output premise, duplicated L, Future=Current, or parent end.

Symbolic credits `(g+1)+(1+min(n,k)*3*(g+1))`, lifetime aliveness, Loc and Cur
remain explicit MODEL antecedents. This proves neither actual Rust entry nor a
runtime resource budget. Closed or allowed assumptions do not manufacture those
premises. Source candidates match the initial candidates after removing only
import lines; `caller.v` is byte-identical even including imports. Trial001 is a
notation/frontend failure. Trial002 compiled and passed kernel checking but its
stale module-name inventory rejected the receipt; neither is semantic negative
evidence. Their indexed receipt hashes were authenticated.

Fresh normal coqc and recursive coqchk passed. Independent Print Module matches
all five source declarations and 20 named inventory helpers, with individual
assumptions for all 25. Body/procedure/credit are closed; contract uses inherited
dependent functional extensionality; the theorem also uses inherited
`Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq`. All 20 helpers are closed. No constructors,
new obligations or additional logical assumptions occur. The final controller's
module-list-based kernel inventory catches both `interface` and `caller`, and
all helpers. Every fresh source/object and printed proof-term digest matches the
worker.

## 2. Constructor erasure — ACCEPT exact delivered conditional composition

Exact scope: `3d668d29`, early `6bb72bfb`, `final-freeze.json`, trial006 receipt
SHA256 `2f9679b44cded4fcb68052c062b8a6a4942b92d99bc099f2cac4d07625e84cb3`.
Direct integrated replay readiness is rejected by finding 1.

`Interface.source_semantics` is an uninstantiated data record. There is no
actual Rust source instance. `V_entry` projects independently supplied lawful
entry, entry control, the original `_1`, length128 and arbitrary initialized
bytes. Omitting empty-history/tag0/offset0/zero-byte restrictions does not prove
coverage of all lawful Rust histories. No future-run filter occurs.

`local_certificate` requires progress at every invariant running state and
preservation for **every enabled event/outcome**, with Fault mapped to False.
Continue retains invariant and running; Normal/Unwind retain invariant. The
entry certificate must additionally establish original-address observability
and the invariant/frame for every V_entry. All remain theorem arguments.
`execution_invariant`, `bounded_erasure`, `entry_erasure`,
`enabled_fault_refutes_certificate` and `entry_normal_bytes` correctly compose
these hypotheses. They prove finite-prefix fault exclusion/frame/progress
conditionally; they establish neither actual source validity, certificate
existence, termination, guaranteed normal return nor original Verus C/F.

The source producer binds the exact two-phase `_4 = &mut (*_1)` reservation,
activation, unsize `_4 -> _3`, constructor consumption of `_3`, all 44 operations
and three phase obligations. Graph membership/count and the four structured-IR
controls are shape diagnostics. Static loan0 is not the incoming dynamic tag.
`root_descriptor_attempt_impossible` and nine other boundary theorems reuse
`fragment.unsupported_rejected`: this supplied descriptor relation has no rule
for those operations. They are not global impossibility/unsafe-Rust results.
StorageLive is already unsupported before the root reborrow. No native
certificate may be accepted from this library.

Fresh exact-snapshot normal coqc and recursive coqchk passed. Print Module and
individual assumptions cover all 96 declarations, constructors, record
projections, generated eliminators and helpers; the generated Assumptions module
has no declarations. Every declaration and recursive kernel summary is closed;
zero new obligations. TRUST_INVENTORY's explicit arguments and false actual
source/certificate flags agree with the statements. All five compiled objects
(including the inspection object) match worker bytes.

## 3. Pointer distance — ACCEPT conditional fragment after helper supplement

Exact scope: `2c0b8f23`, interface `efea4e77`, implementation `3713bff6`,
`final-freeze.json`, trial011 receipt SHA256
`04aedea513457712d9afaf49addbf694c7d33f2676408adec426b19ba6dca47b`.
Delivered complete-inventory readiness is rejected by finding 2.

The pinned compiler's ptr-to-pointer-width integer conversion, NUW subtraction
and exact unsigned division motivate `unsigned_sub`/`exact_udiv`. The proof
establishes finite64-bit address ordering, no subtraction wrap, signed byte
length <=isizeMAX, and exact division by1 from the conditional span bounds.
Inspection of exact rustc/LLVM source is not compiler preservation.

Allocated spans use the existing window/pointer/slice_valid/shifted definitions
with arbitrary containing-allocation cursor. Strict-interior pad offsets are
allowed; legal one-past formation uses existing Offset bounds and is distinct
from dereference. `SpanEmpty` includes equal q=p; `EmptyAddress` requires only
nonnull64-bit representation, no allocation, heap lookup or readable cell.
Lengths are unconstrained by128 beyond target representation bounds.
`zip_empty_key` and `zip_long_key` cover zero and >128. `formation_operational`
and `subtraction_operational` prove actual existing lrust head reductions with
unchanged state/no events/no forks; no new intrinsic rule or axiom appears.

For both UbChecks settings the authored scalar chain excludes ZST, wrong
address order and size-assert failure conditionally on a legal u8 span.
Computed hints are `(n,Some n)`, projected sizes feed specialized Zip's exact
min(128,n), index0 and unchanged endpoint descriptors. Nine in-memory source
controls change structured enums, operands, comparisons, branches, widths and
exception edges and run validation directly; rejection is not a hash test.
Original panic/cleanup/drop/resume edges remain recorded and unsupported as
source simulations. No native conclusion erases them.

Source validity -> actual field/cast operands -> span, and actual source-machine
intrinsic/control correspondence remain unproved. The span is not source
permission or application authority. Normal fragment return is not a theorem
about Rust termination or cleanup. All actual-source flags remain false.

Fresh normal coqc and recursive coqchk passed. Independent Print Module confirms
77 source declarations including constructors/eliminators, plus the 154 helper
proofs in finding 2. Individual assumptions for all 231 declarations are closed;
recursive kernel axioms=[], zero obligations. All three worker/fresh compiled
objects match byte-for-byte.

## Authentication and reproducibility

`results.json` records exact receipts, full discovered inventories, individual
assumptions, command exits/times and every worker/fresh source/object SHA256
comparison. There are no deterministic object differences. Private pretrial and
final manifests retain exact source/tool/object/dependency/log pins; their
hashes are committed. All final retained-input and product guards precede
success. Historical source-root labels in worker receipts were remapped by
relative path to the prepared checkout or exact private frozen snapshot;
no historical checkout directory was opened.

Readiness AND its successful named `durable-build-2/receipt.json` authenticate
VerusBelt `115a618dd35ff8337112cf2081a7f5391d3973d5`, all 523 source files and
284 objects, loadpaths, coqc/coqchk/runner/opam, tool libraries, completed model
and MIR dependencies, pinned compiler/core source and GMP manifest
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
Normal recursive checks use no unsafe flags or exclusions. The independent
supplement additionally compares the complete compiler-discovered cst sets
with kernel output, including inspection helpers.

To reproduce in a fresh private location, change only review.py's RAW constant
before its pretrial freeze; commit the scope/harness first, then run with
`python3 -B`: `review.py prepare`, `review.py replay caller`,
`review.py replay constructor`, `review.py replay distance`; freeze inventory.py
before `inventory.py caller`, `inventory.py constructor`, `inventory.py distance`;
freeze finalize.py before `finalize.py`. The committed run's RAW path is
single-use. The exact invocation vectors are retained in private receipts.
Review-only Redirect warnings report an unset output directory and explicitly
use the private current working directory; their complete stderr is retained.

Canonical6/0 and full HMAC1/1 are inherited, not rerun. Native=0;
44 minimal +3 phase /101 writer effects remain open; publication=false,
goal incomplete. No Rust full suite, HMAC run, source-validity theorem,
new trusted summary or native completion is claimed.
