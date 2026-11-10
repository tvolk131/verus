# Independent review: borrow-entry handoffs and compiler binding

Conditional MODEL mathematics and the compiler-inspection checkpoint PASS fresh
independent review. No blocking mathematical or final-replay finding was found.
This is not whole-goal acceptance or a native certificate.

The only finding is **P3, documentation**, at
`source/tools/research/tree_borrows_entry_handoff/MILESTONE.json:6–23`:
`successful_compilation_receipts` contains `interface-02` with exit_code 1.
Treat it as failed elaboration history; remove it from that successful list or
rename the list. The authenticated final-01 / kernel-final-01 /
inventory-final-01 receipts all succeed, independently reproduced below.
No reviewed source or historical evidence was edited to fix this nit.

Review base is clean `37950d9d0f4d3d3b20b8b091e10be748df353b5a` (the exact full hash is in scope-freeze.json).
The early scope/interface commit is `eeca1cdc`; the authenticated pretrial and
explicit compiler-path-mapping commit is `69249063`. GIT_AUTHENTICATION.json
records full delivered commit IDs and hashes every reviewed directory file
against immutable Git objects: worker 89494dce / integrated 15e3c268, worker
995809af / integrated 5f4a04c5, and compiler 7f415a57. All corresponding current
bytes match. The original candidate004 at 04b51409 hashes
434ed59ef957c98daf4a0c6825d63d2203d56fc6dc666de003122e4d882b14e1;
its UNCOMPILED13 historical status is not the final proof result.

## Fresh checking and product identity

Private root: `/tmp/borrow-entry-handoff-review-20261110-t3-a6eab20d`.
No prior worker output was overwritten. PRETRIAL.json authenticates delivered
sources, receipt/log/product maps and the exact rehoming of driver path constants
before formal trials. Exact sources were copied into fresh private compilation
directories. No old baseline prepare/build ran. Existing completed upstream
objects and tools were read-only inherited inputs.

* Stacked Borrows: normal Coq 8.9.1 / OCaml 4.07.1 compilation of candidate.v
  and composition.v, explicit pinned -coqlib, upstream namespace stbor and own
  handoff. Full recursive coqchk -o succeeds: 282 libraries, 40,050 constants.
  Independent kernel-discovered names plus Print Module and individual
  Check/Print Assumptions queries cover all **25 declarations: 22 lemmas and
  3 definitions, all individually CLOSED**. No helper was omitted.
* Tree Borrows: normal Rocq 9.2 / OCaml 4.14.2 compilation in EntryHandoff.
  Full recursive rocq check -o succeeds: **458 libraries / 51,983 constants**.
  Independent kernel names plus Print Module and source record inspection cover
  **17 declarations: 15 kernel constants (including five field projections),
  the record and its constructor, all individually CLOSED**. This primitive
  record produces no separate eliminator constants; any generated constants
  would also appear in the complete kernel/module inventory.
* Both runs use normal recursive checking, no exclusions, admissions, added
  axioms, unsafe kernels, type-in-type, assumed positivity, disabled checking or
  source-defined-execution hypotheses. Each toolchain and namespace stayed
  separate. Before/after foundation authentication agrees.

The independent inventory is in stbor-INVENTORY.json and tree-INVENTORY.json,
including exact types, assumption text, compiled module printouts, checked
library sets and context summaries. COMMANDS.json binds command argv, cwd,
environment, exits and logs; PRIVATE_ARTIFACTS.json binds all 271 fresh private
files including sources, logs, compiler inputs/products, objects and receipts.
No fresh setup, tactic, elaboration or controller failure occurred.

Both rebuilt Stacked objects are byte-identical to delivery:

* candidate.vo: a36d4ee9ed2da396a44aba069a70bff0582deabe0428cd407771c23abc872ff8
* composition.vo: 55fab0cdc9286f39fe0e95751aa4727049c3f44a667754cabc0402163ef98f8b

All 282 loaded Stacked object hashes, serialized dependency digests and imports
match delivery. The fresh acceptance.json itself is byte-identical to delivery:
`e178df9b7adb2714ed6ae177271db895da11e972ab2488d7088d8a1445b666e0`.
The separate new driver receipt proves this came from a completed new run rather
than relying on that identity alone.

Tree Entry.vo is not byte-identical: fresh
`d639c0d85a9ddbf312e6640b2954b9c4437447ef719cdbcf89313de30a341233`,
delivered `9c7b77ede542b0a95cd244ed631189c7eeb0d7915c071536048ca8b1a540d0b6`.
Rocq embeds the physical Entry.v path (observed at byte offset 9922); the new
path is nine bytes longer, as is the object. Exact source SHA is unchanged:
`92d5b6c7949d038d5c75f299ff5969dd46f11160f5bb91fca9f1602e1bc69dcf`.
Separate fresh Rocq invocations load delivered versus rebuilt objects and Print
every declaration with all terms/universes exposed. **All 17 printed bodies,
including opaque proofs, match byte-for-byte.** All 457 upstream loaded objects
and the exact library set also match. PRODUCT_COMPARISON.json binds this
mathematical comparison; no unjustified Entry.vo byte identity is claimed.

## Mathematical interface review

Stacked `composition.v:102–160` binds the actual FnEntry result tag p to the
next TwoPhase operand, with entry state, location, size and call unchanged;
q is the next fresh tag. Public `published_fnentry_two_phase` starts from
positive n, Wf, an actual top live call, and valid_block. The upstream
valid_block is structural: freeze, stack-domain membership and access1_pre
(permission, granting index/tag and incompatible-younger-protector conditions).
It assumes neither caller-top nor successful retag, pool_safe, or future
successful/defined execution. The theorem derives both retag equalities and
actual published head_step witnesses.

FnEntry installs exactly Unique/p/Some c. The first access may discard
incompatible unprotected younger items: no blanket caller-tail preservation is
claimed. The second grant prepends exactly SharedReadWrite/q/None and preserves
the *entire normalized entry stack* as its tail. model_retag_frame composes
heap, calls, next-call, WF, outside-span stack equality and active-protector
membership preservation. Generic positive n and explicit 10/128 instances are
covered. Heap/scalar-vector stuttering proves no XOR computation. The n=0
stutter/dangling lemma is expressly historical-model-only; it says nothing
about current Miri or the lawful empty-reference domain.

Tree `Entry.v:6–39,85–170` evaluates entry_eligible on the ORIGINAL tree at
all offsets in the range and every original node. `rel_dec tr parent itag =
Child _` means the node is parent or an ancestor, not a descendant; these must
not be Disabled. Other nodes with active protectors and initialized permissions
must be neither Active nor Disabled. Inactive/unprotected/lazy foreign nodes
are not gratuitously excluded. The proof accounts for original parent,
ancestors, foreign relatives, and the new child using actual insertion-relation
lemmas and the exhaustive READ transition cases.

Public entry_progress requires only state_wf, actual block lookup, represented
parent, live cid, positive size and that structural predicate. Freshness follows
from state_wf. Creation is derived by reducing create_child, not supplied as
success. The required READ is constructed AFTER insertion, using the actual
access-compatibility success lemma. RetagIS, final WF, tag increment and
outside-block frames are conclusions. The internal compatibility lemma's
creation equality is not a public progress premise. The result record's five
success/WF/frame fields are conclusions. Entry.v:176–204 additionally preserves
original tag/protector/default permissions and same-block lazy permissions
outside the interval. Those frames are not a native HMAC suffix theorem.

## Compiler checkpoint and controller review

Pinned rustc 1.99.1 / 58cf8f9c35b355199727627d07ae34a4e2846d67 was used.
The exact ordinary macro-erased canonical candidate-008 and frozen xor_key
fixture were queried AFTER compiler analysis, not inferred from source text.
Structural TyKind Array(u8), mutability and compiled layouts identify canonical
[u8;U64_LEN] as 10/align1 and HMAC [u8;128] as 128/align1. Both satisfy
Unpin/UnsafeUnpin/Freeze. Key [u8] also satisfies all three but is unsized:
static_size=null, minimum_size=0, align1; its minimum size is NOT runtime key
length. There is no new key-length, initialization, provenance or permissions
requirement.

Ordinary generic checking establishes the three trait requirements for every
const N in [u8;N] without supplied trait bounds, and for unsized [u8]. All
**nine commands** passed their intended outcomes (six exit0; three independent
PhantomPinned/UnsafeCell trait controls exit1). Independently inspecting fresh
stderr finds exactly E0277 as the error-code class for all three controls.
They are ordinary type-check rejections, not semantic application negatives.
All query data matches delivery exactly. All final source/tool/binary guards
passed; all fresh logs and generated control-source hashes are retained.

The native controller uses explicit require checks, which survive its accepted
-B -O replay. RUSTC_BOOTSTRAP exposes private APIs/features and does not disable
checking. The Stacked controller's asserts remained enabled in ordinary Python;
we independently checked its kernel/module/dependency/assumption results and
completed post-guard products. Tree's final source and receipts were assessed
independently of its inaccurate historical milestone label. Historical failed
trials remain setup/elaboration/controller history and contribute no semantic
negative evidence.

compiler-rebound-freeze.json records every changed path and original digest.
Old /home/dev/projects/verus labels were rebound to identical own committed
bytes; the four bundle libraries were rebound to the completed delivered clone,
authenticated by bundle.json
`aae1f6b375c4b6f558d6cc62d4268925f959105a79bde920dc102884d72a3012`
and the original per-file digests. The pinned sysroot and Miri-source paths
stayed fixed. No historical checkout was read.

Source inspection of pinned Miri stacked_borrows/mod.rs:62–89 confirms the
mutable FnEntry Unpin && UnsafeUnpin branch yields Unique / Write / strong
protector, whereas the earlier TwoPhase branch yields SharedReadWrite / no
access / no protector. This is compiler answers plus source inspection, not a
verified translation or a derivation of runtime grants. Freeze was queried as
well; it is not an extra conjunct of that mutable Miri branch.

## Authentication and trust

The supplied Stacked ready/completed receipt roots authenticate exactly:
9259b8f303e52e38f1b47709429f2be23a81a4e44ed557b6058184a95e653e51 /
0d85e9ec05f29413973137f03bc840ccb1e22170d50169b2837331e58770f649.
The upstream is d1471b10fe4cb4f506371b2e3acdd927d0a0c076. Tree ready root is
f56490bb8441dbd73c37fe46fe970dc008bb8b2b67ee2b651a04c34d2b1ca523,
with Simuliris e3abfcdc301f92250d4227fc03c00463bf57b982. Delivered native
receipt root authenticates as
e4a94a85107c5430cd1289ec6f27a921e4879da0dbe755b50124f85a3073df75.
Repository source/support maps, named completed receipt logs, pinned binaries,
installed plugins/libraries, loaded source/object maps, and pre/post guards were
checked. Completed historical build receipts authenticate inherited libraries;
this review neither rebuilds nor independently certifies their full foundations.

ASSUMPTIONS.json lists exact library-set differences and verbatim contexts:

* Actual Stacked closure: proof_irrelevance, functional_extensionality_dep,
  JMeq_eq, Eq_rect_eq.eq_rect_eq. Each of the 25 own declarations remains CLOSED;
  imported global context is not attributed as assumptions of each own lemma.
* The wider 518-module Stacked foundation additionally lists classic,
  Equations funext, the upstream admitted FixWf_unfold, and six sealed Iris
  iProp interface fields backed by the checked upstream implementation. The
  authenticated historical accepted013 has 519 modules (foundation + candidate);
  our exact closure adds composition and omits 238 of those historical modules.
* Actual Tree closure: functional_extensionality_dep only. The broader 507-module
  foundation additionally lists classic and constructive_indefinite_description.
  Our closure adds the own module and omits 50 baseline libraries. There is no
  pool_safe/SafeImplies derivation of original source safety here.

Exact new receipt paths and SHA256 values are in results.json. In particular:
Tree kernel `c3aa59b93a536f4ed22f603d8167e39acd705297ccb43a7bc59fa3f7a06be7c0`;
compiler `a6b8626f0d291460495f4f04749319bf9876889b9332a27cf441bbe98fcbe0c1`.
All inventories and retained private artifact maps are reviewable independently
of this prose summary.

## Unchanged whole-goal boundary

Native correspondences remain ZERO; all 44 minimal + 3 phase simulations and
101 writer effects remain OPEN; publication=false and goal_complete=false.
All-lawful actual Rust eligibility, actual FnEntry input/context and carrier
installation, initialized-byte/provenance/ancestor/protector/C-F interpretation,
finite-tag/error/cleanup behavior, empty/dangling references and current-Miri
correspondence remain unproved. Neither imported model assumptions nor compiler
trait answers establish these obligations.

Exact frozen canonical unsigned varint remains required for every u64 and
arbitrary initialized exclusive [u8;10], inherited6/0. Independent exact
bitcoin_hashes0.14 HMAC remains required for arbitrary initialized [u8;128],
ALL valid key lengths including 0 and >128, BOTH XOR prefix and untouched suffix,
inherited1/1 UNPROVED. No app-local mechanical proof/call/snapshot accepted
fixture was added. Actual same checked assistance, normal checks, fidelity,
inference, publication and fresh whole-goal acceptance gates remain required;
this narrow checkpoint review does not discharge them or rerun their fixtures.

Only this evidence directory and our private root were written. No delegation,
production edits, install, push, PR, another checkout, sibling/baseline-reviewer
inspection/waiting, unsafe/exclusion experiment or toolchain mutation occurred.
