# Campaign94 ongoing progress

The full goal remains ACTIVE. All six properties for each of one-phase,
two-phase and mixed-mode histories remain OPEN. No native commit/recovery or
logical-table correctness credit is claimed by the initial work below.

Latest continuation: `TAKEOVER.md` records the November 7, 2026 handoff,
outstanding review objections, and authorization boundaries. Earlier progress
entries retain their historical scope; they do not authorize new trust,
reset old allowances, or establish current proof acceptance.

## Initial freeze and persistence foundation

Preserved the exact 2304-file pre-campaign dirty source, all 1401 immutable and
1331 native guards, accepted93 seal/runtime and owned HEAD/index/branch.
Initial archive SHA256:
`3f1fedb9692505f39fe540129cf249faf0c42fa60db0b354e0bf3b2133773ff7`.
Private campaign:
`/private/tmp/verus-crash-consistency94-20261105`.

`source/tools/redb_crash_consistency94/persistence_model.rs` is an authored
byte-level foundation, not a native application proof. It separates persistent
bytes, backend-visible bytes and per-byte permitted pending values. Individual
bytes are atomic; different bytes may persist independently/out of order.
Successful sync drains pending choices. Power restart discards volatile state;
process restart retains backend/kernel state rather than pretending exit is a
power loss or successful flush. Actual native application buffers and file-length
changes are not yet modeled or refined to these operations.

Checked clean-state and transition existence witnesses prevent the core steps
from receiving credit only through inconsistent transition premises. Twenty
proof bodies check, including storage invariants, whole-byte selector updates,
and induction over arbitrary finite storage-step histories preserving explicitly
protected bytes. Compatible writes are a premise of that frame sublemma, not an
already-established property of native allocation/reclamation. Overlapping
rewrites of the same protected byte value are allowed; this is needed because
native header publication rewrites both slots.

Fresh ordinary export/import checks one consumer proof body. Three intended
backend controls reject: an unflushed visible write claimed durable, overwriting
an explicitly protected byte claimed frame-safe, and process restart claimed to
discard kernel-visible bytes. Controls earn no positive proof credit. No new
Infer feature, standard/library trust or application contract assumption is added.

Seven total direct attempts include all drafts: two frontend failures, one
two-obligation proof failure, two successful provider revisions (initial ten,
then seventeen proof bodies), the intended-negative run and fresh import.
No compiler/vstd/harness builds or network operations. Raw input,
command/environment/runtime bindings, stdout/stderr and results are retained
per attempt; `first-model-checkpoint.json` binds the successful artifacts and
the exact three failed proof identities. Query totals and checked proof-body
counts are separately recorded; native checked executable bodies are zero.

## Next dependencies

Native commit/cache/flush/recovery closure analysis is running independently.
Next work must relate root publication and validated recovery to the byte-level
model, prove actual validation coverage, and connect actual allocation and table
operations to state/history refinement. Historical slot-selection and repair-
observation results are candidate sublemmas only: their native decoding/hash/
storage premises must not be relabeled as established persistence correctness.

This checkpoint is not a cycle cap or campaign stop.

## Independent foundation review

The frozen foundation, four commit-order model proofs, ordinary model consumer
and six intended storage/order negatives were independently reproduced in five
launches. The review found no false checked foundation theorem or illicit model
axiom. It explicitly does NOT establish that native writes obey protection,
that arbitrary dirty selector generations are old/new, or that the order controls
depend on the commit-order artifact. Those controls are semantic model witnesses,
not mutation-sensitive coverage of that theorem interface. Fixed file length,
application buffers, failures, checksums and native correspondence remain open.

Read-only report:
`/private/tmp/campaign94-foundation-review-20261105-AQOpNK/REPORT.md`.
Seal SHA256: `ee00396cc10dcb26c03e735f43d4d4d1d456717cd7801f002b5fffdfd702e04c`.
Inherited and shared-object-store nonmutation remain NOT_CERTIFIED; observed
shared orchestration changes are not attributed to reviewed code.

## Native codec dependency checkpoint

The accepted runtime rejects unchanged native little-endian u32/u128 calls.
Added four direct standard-library bindings to existing mathematical vstd byte
specifications; these are explicit trusted primitive boundaries, NOT checked
Rust-core implementations or new Infer automation. Matched private vstd checks
2221/0, with the existing compiler binary unchanged and no compiler rebuild.
All original runtime bundles remain untouched. This is the only intentional
baseline production-source change so far: `source/vstd/bytes.rs`.

Fresh checked references establish exact native `get_u32` and `get_u64`, the
three PageNumber codec bodies, and the three BtreeHeader codec bodies: eight
complete native bodies with narrow proof-erasure token audits. Page roundtrip
requires the actual canonical field domain; arbitrary decoded bytes are safe
and decode to a canonical page, while reserved bits need not roundtrip as raw
bytes. BtreeHeader Debug formatting is outside scope and receives zero credit.
All-input u32/u128 word and byte roundtrips check. Fresh ordinary consumers check
three scalar clients and two page/root clients. Six scalar safety/semantic and
three root/domain negatives reject in the backend as intended.

Retained unsuccessful drafts include missing-import model frontend failure,
old-runtime unsupported-function frontend failure, two scalar proof drafts,
two page proof drafts, and one mixed backend/unsupported-BV control draft. The
latter earns no clean semantic-control credit; its corrected replay has exactly
three intended assertion failures. Two private runtime-preparation setup errors
occurred before any verifier launch; the third setup verifies matched vstd.
No failed attempt is relabeled as checked native scope.

Full TransactionHeader/header validation, checksum implementation and coverage,
tree correctness, file resizing, interrupted recovery, logical map refinement
and all eighteen final cells remain OPEN. Codec support has not yet received
independent review and is not accepted for rollout.

## Native closure research

The read-only native commit/recovery map identifies 183 principal functions and
a larger proposed dependency envelope, not a checked compiler closure. Whole
header writes, native buffering, reachable growth/shrink, automatic non-durable
epilogues and normal-close repair maintenance are required dependencies. A
commit error may occur AFTER durability; Err is not a rollback guarantee.
All four mode transitions and repeated recovery interruptions remain mandatory.
Report: `/private/tmp/campaign94-native-deps-20261105-PtSgT5lq/REPORT.md`.

## Leaf-count and concrete-byte support

The additional u16 count primitive is bound to its existing two-byte spec.
Four checked public u16/u32 expansion lemmas remove the separate closed-spec
visibility obstacle without changing the native decoder or adding application
assumptions. Runtime005 freshly verifies matched vstd 2225/0. External clients
verify all-input expansion equations and literal/extrema fixtures through a
new ordinary import; four discriminating controls reject. Scalar, native page,
native root and u16 roundtrip references pass on that exact latest runtime.
The original four-binding checkpoint/runtime003 remain immutable. Separate
independent review of this extension remains required.

The native leaf researcher reports all21 exact decoder/accessor bodies checked
on runtime004 (28/0), covering all fixed/variable key/value-width combinations,
including zero fixed widths, under a concrete byte-layout predicate. The fresh
three-body ordinary consumer checks. Remaining fixtures, controls and audits
are still running, and this milestone is not yet independently reviewed.
Native producers, sorting/search, mutation, allocation, persistence and logical
caller establishment are not established by conditional leaf accessor proofs.

## Whole-header publication model

Three additional checked model theorems handle a full-range header rewrite
after the preparation sync, full-header publication after a successful sync,
and atomic selection of primary/mode bits from one whole-byte generation.
Two fresh imported proof clients check; five controls reject the missing
barrier, changed protected field, false issuance acknowledgment, missing final
sync and hybrid selector-generation claims. The controls call the actual
theorem interfaces. Native encoder/frame correspondence and validation remain
open; these are not native commit correctness or checksum proofs.

## Independent codec reviews completed

The original four direct u32/u128 bindings and eight selected native codec
projections received a bounded independent review with no blocking finding:
`/private/tmp/verus-review94-native-codec-20261105-eViN6YaR/REPORT.md`.
The review used16 direct launches and one private matched-vstd verification
(2221/0). Three unsuccessful probe drafts receive no complete-scope/control
credit. Canonical page premises remain essential.

A separate new review authenticated all171 extension-checkpoint payloads and
found no blocking issue in the two direct u16 bindings and four checked public
expansion lemmas. Nine fresh launches check12 executable and4 proof bodies
(28 queries), with10 intended backend controls. Matched vstd2225/0 is audited
there, not freshly rerun. Report:
`/private/tmp/campaign94-codec-independent-GX3mIYdA/REPORT.md`.
The two reviews are not merged into a Rust-core implementation proof or native
protocol acceptance. Original codec checkpoints001/002 remain unchanged.

## Native leaf reference delivered

Final C0 provider on runtime005 checks21 complete native bodies and6 authored
lemmas,32/0. Fresh ordinary consumers, all four width modes, symbolic counts
1..65535, zero fixed widths, arbitrary one-pair payload/spare bytes and
metadata-preserving frames check. Thirteen isolated backend controls reject.
All31 attempts, including three zero-credit frontend failures, are sealed at
`/private/tmp/campaign94-native-leaf-c0.sFb2bySL/REPORT.md`.
A NEW independent C0 review is running. Structural grammar satisfiability is
not a sorted/unique map invariant or native byte-producer establishment.

## File length, partial I/O, and finite histories

`file_length_model.rs` checks19 proof bodies. Six fresh imported proof clients
check, including an arbitrary trace position/read index, intermediate crash
length witness, partial-byte write witness and errored-but-applied sync witness.
Six backend controls reject unsafe shrink, out-of-EOF write, atomic-length,
premature acknowledgment and rollback claims. This is a candidate environment
model, not a new trusted I/O contract or native correspondence proof. Its exact
remaining assumptions and excluded-from-this-sublemma operations are listed in
`FILE-LENGTH-MODEL.md`; no final campaign operation is removed.

Two frontend type drafts and multiple unsuccessful consumer/control drafts
are preserved. The final arbitrary-position trace client is universally
parameterized over position/read index; earlier nested-quantifier consumer
drafts did not verify and receive no complete-scope credit. The checked
provider's finite-trace induction remains unchanged. Failed resize, automatic
EOF extension, actual cache cancellation and platform sync/length persistence
remain open. Successful model sync is not asserted as native fact.

## Native in-memory header slot mechanics

`native_header_slots.rs` checks10 selected complete native ordinary-contract
bodies: four TransactionId methods, TransactionHeader::new, and five
DatabaseHeader slot read/update/swap methods. The full report lists13 checked
executable bodies because it also includes three generated Clone bodies;
two non-Copy Clone bodies have NO generated semantic contract. No clone-value,
ordering, hashing or debug-formatting theorem is claimed. One authored XOR
lemma and18 total queries check. All actual struct fields and10 erased bodies
match the native source under the enumerated visibility projection.

Five genuinely imported clients check swap-twice restoration, previous-slot
publication while unflushed, exact corruption-frame reset, finite-width ID
increment and empty-slot initialization. Six backend controls reject false
slot/reader/frame claims, ID exhaustion and an invalid primary index. Valid
primary index and non-exhausted ID remain UNPROVED native caller premises.
Whole header encoding, native checksum validation, I/O, publication and recovery
are not established by these in-memory helpers.

Two import-launch drafts lacked a transitive Rust search path, and another
used the old mutable-postcondition syntax; all are zero-credit frontend
failures. The private launcher now stages only hash-bound explicit ordinary
libraries in a unique dependency directory. One consumer draft needed the
existing array extensionality lemma; its corrected five-body import passes.
An initial token-audit draft mistook a brace inside a contract for a body;
the corrected balanced-delimiter audit passes all10 bodies. Both limitations
are recorded rather than relabeled as native semantic failures.

## Independent C0 review and producer boundary

The independent decoder review at
`/private/tmp/campaign94-c0-independent.LWwTD7SU/REPORT.md` found no blocking
defect in the bounded authored reference. Twenty fresh launches include the
32/0 provider, a NEW ordinary-import consumer at4/0, symbolic/literal/frame
witnesses, all13 control classes and three additional adversarial checks.
All21 native bodies, signatures, fields and lifetimes independently match.
The author's final3/0 consumer is attempt031, not attempt016 (6/0 safety
positive). Neither structural satisfiability nor this decoder reference
establishes native production, sorted uniqueness or logical KV correctness.

Exact producer research is sealed at
`/private/tmp/campaign94-builder-research.9YL8Q4Fs/REPORT.md`.
Only required_bytes and key_section_start check as a two-body arithmetic
subset. The full std path rejects std::thread::panicking. More fundamentally,
the native destructor intentionally rejects incomplete builders; it needs
both completed pair count and exact key-reservation consumption. Today's
unconditional Drop/no_unwind restriction cannot express that normal-drop
domain. Dropping the destructor, renaming it, assuming completion, switching
to no_std, or prematurely resolving borrowed fields would not prove the native
producer. A separate independent design assessment is in progress.

The producer investigation also exposed an invariant-mask dependency compiler
crash. Parent attempt063 separately rejects unsupported core::mem::drop;
attempt064 reduces the crash to a real destructor containing a tautological
assert_eq, with implicit scope exit. Both receive zero proof credit. A narrowly
staged compiler candidate will test conservative dependency retention only;
live engine, runtime005 and native bytes remain unchanged. Repairing this crash
cannot make an unconditional incomplete-builder theorem true.

## Scalar header-hash subset and remaining native correspondence

The sealed scalar reference at
`/private/tmp/campaign94-hash-proof-20261105-nDrhh7/REPORT.md` checks seven
complete native scalar bodies against explicit deterministic byte-function
specifications (19 provider queries,11 fresh import queries). Six safety and
four semantic controls reject. It used17/48 launches, no builds or production
edits. Exact full dispatch/wrapper witnesses fail frontend translation of
unsafe function-pointer parameters. No native dispatch specialization/linkage,
arbitrary-page SIMD or full no-unwind/global-frame closure is claimed. The
two bounded LE codec reviews are now complete; their trust remains explicit
std-boundary trust, not Rust-core implementation verification.

The all-six, both-mode and mixed-history campaign remains ACTIVE. Every one
of the18 final property/history cells remains OPEN.

## Independent publication review and isolated compiler repair checkpoint

The new bounded publication review is sealed at
`/private/tmp/campaign94-independent-review-20261105-9Ih9ExNe/REPORT.md`.
It found no blocking semantic defect in the conditional whole-header and
file-length models or the ten ordinary-contract native header/ID methods.
Twelve fresh launches include nine complete successes and seventeen intended
backend rejections. None establishes the platform environment, native caller
premises, publication/recovery composition or a final campaign cell.

Medium C94R-001 independently confirms that the legacy1084-path Cargo-root
manifest omits seven compiler inputs included through explicit source paths.
All seven match the original2304-path preservation inventory, but that legacy
map is not exhaustive compiler-input evidence. Historical evidence is unchanged;
it is not silently relabeled with a stronger source-binding claim.

The isolated mask-dependency candidate explicitly binds1091 paths and audits
seven final top-level compiler dependency files:452 compiled local inputs,
zero unbound local inputs. Before external trait demotion, an unresolved trait
method could skip ISet reachability, then require the pruned operations after
demotion. Conservative retention removes the reproduced Drop/external-trait
compiler crash. Positive ordinary-mask/trait fixtures check; incomplete Drop,
forbidden invariant opening and no-unwind controls still reject at the backend.
The candidate does not add destructor preconditions, suppress panic checks,
or establish native producer correctness. Live compiler sources remain unchanged;
candidate regressions and independent review are still pending.

The two previously uncontracted non-Copy header Clone bodies are a separate
bounded proof frontier. An authored candidate now checks explicit field/slot
relations on the actual generated Clone bodies and reassembles whole-value
identity with checked array extensionality:22/0 provider queries. Ordinary
imported clients, controls and independent review remain to be qualified.
Frontend and unsuccessful backend drafts are retained without semantic credit.

## Native destructor and scalar-hash independent assessments

The completed destructor design assessment at
`/private/tmp/campaign94-drop-design.J80twDA2/REPORT.md` finds no sound
annotation-only path to exact native producer closure. Actual typed drop-edge
obligations, real panic-state correspondence, explicit drop, generic/nested glue
and borrow-retirement frames require consequential verifier-semantics work.
The small mask-pruning crash repair does not supply these semantics. This is
being escalated under the original scope rather than bypassed with assumed
completion, renamed/omitted Drop or a different native configuration. The
decision and proposed separately gated workstream are in
`NATIVE-CLOSURE-DECISION.md`. No such implementation has been authorized here.

The completed scalar review at
`/private/tmp/campaign94-hash-independent-lxAncYQ/REPORT.md` found no blocking
bounded defect in the seven checked scalar bodies relative to the declared TCB.
Seven fresh applications reproduce19/0 provider queries, fresh and sealed11/0
imports, and six safety/four semantic controls. Actual native hash dispatch and
wrapper remain frontend-blocked; whole-chain no-unwind/global-frame closure is
open. Both codec trust reviews are complete within their bounds. Historical
1084-map incompleteness and Git object-store noncertification remain disclosed.
All18 final cells remain OPEN; neither review grants native protocol acceptance.

## Final bounded mask/Clone review

The new independent review at
`/private/tmp/c94-independent-review-20261105.LUfXsE8Y/REVIEW.md` supports the
conservative mask dependency repair and the checked native header Clone helper
claims within their stated bounds. Seventeen fresh applications include the
three codec/header providers, a NEW5/0 ordinary consumer and discriminating
mask/unwind/incomplete-Drop/Clone controls. One frontend draft is zero-credit;
there are no builds/rebuilds. The1091-path candidate association and452 local
compiled inputs are supported, not an exhaustive platform trust certificate.

The low stale-projection finding is handled with a separate authenticated copy
of the reviewer's current13-native-body reconstruction and strict fixture-hash
selection for future checkpoints. The historical002 receipt and sealed packages
remain unchanged. Runtime005's inherited nonexhaustive1084-map qualification is
not waived. The engine repair remains in the reviewed isolated candidate; no live
adoption or rollout is claimed. This bounded review does not authorize the larger
contextual Drop/panic/borrow-retirement workstream or close any final campaign cell.

## Authorized destruction-support side quest: first concrete baseline

The user authorized the bounded side quest and return to the unchanged native
goal. DROP-SUPPORT-SCOPE.md records the gates; no further application input is
needed for ordinary implementation choices. A1091-path isolated source stage
is seeded from the reviewed mask-repair source, leaving live compiler unchanged.

The completed mapping research at
`/private/tmp/campaign94-checked-drop-research.hPXidlG8/REPORT.md` identifies the
elaborated MIR query and concrete resolve_drop_glue/instance_mir APIs, the lost
binder/expression origins needed for real VIR places, and the need to re-run
resolution after effectful events. A native-rustc diagnostic supplies IR evidence,
not a destruction theorem. Real panic-state correspondence, cleanup poststates,
PageMut/WritablePage glue and generic/import escape coverage remain open.

Parent085 verifies the simple real borrowed-field Drop body but rejects its
caller's promised scope-exit writeback. Parent086 rejects actual core::mem::drop
at the frontend. These establish a small measurable capability target; neither
is a successful full scope or credited semantic negative. The diagnostic extractor
is being implemented separately. DROP-SITE-BASELINE.md distinguishes that mapping
work from actual checked effects and the larger native builder requirements.

## Diagnostic prototype reviewed and repaired; next: an actual destructor effect

The first prototype can print typed destruction-path observations, but proves
neither that a destructor executes nor what memory it changes. Independent
review found its partial-coverage labels truthful and found two output-path
collisions: reporting could occupy the log directory or the temporary proof
export. It also omitted the requested root-only scope flag.

Parent repaired these in a new isolated source stage. Report data stays in
memory until verification, erased compilation and proof publication finish.
Reserved output paths disable reporting with a warning; existing files and
symlinks are never overwritten. The actual proof-export temporary path is shared
with the exporter, and `verify_root` is recorded. One compiler build, matched
vstd2225/0, and twenty exact application outcomes pass. A selective-report test
reader mistake stopped the test script after seventeen runs; their authenticated
receipts were reused, and only the remaining three were launched. This was not a
verifier defect or a hidden rerun. No broad regressions ran in this checkpoint.

Read-only repair package:
`/private/tmp/verus-crash-consistency94-20261105/drop-diagnostic-repair-001/handoff`.
Seal SHA256:
`945f8f0e923fea0a7d391048a8ea0c9bb9438d2675a783ee3957b64c798d3c79`.
Repair checks are parent results; independent repair closure remains open. All
2344 initial live source paths and1401 immutable/1331 native guards were preserved
before this authorized documentation update. No live compiler adoption occurred.

The next capability target is the existing WriteBack example: its real Drop
writes seven through a borrowed reference, and its caller should prove that the
original value is now seven. Today the Drop body verifies but that caller fails.
The implementation must connect actual elaborated Rust destruction events to
the original loan and insert the checked effect before borrow retirement. It
must reject wrong-value/frame claims and unsupported reachable paths, retaining
all existing Drop restrictions. Printing or hashing more graphs is not success.
Native builder cleanup and all eighteen final database cells remain open.

## First checked-effect attempt: real mapping progress, required proof still fails

The isolated candidate now retains real compiler binder identities, observes
elaborated destruction and concrete cleanup code, inserts a call to the checked
Drop implementation, and compares the proof-side and erased executable paths
before export. Deliberately different executable bodies are rejected after a
successful backend phase. These are candidate mechanisms, not accepted semantics.

The required unchanged WriteBack caller still does not verify. The first obstacle
is that ordinary mutable-borrow lowering rejects its immutable `let guard`, even
though Rust can destroy an immutable binding. A separate mutable renamed example
gets past that check but still cannot establish the effect on the original
borrowed value. Therefore changing mutability alone is not a solution: the
destruction event needs checked original-field loan framing and retirement.

The candidate is sealed, isolated and unaccepted:
`/private/tmp/verus-crash-consistency94-20261105/drop-effects-001/handoff/REPORT.md`.
All4338 payloads authenticate; seal SHA256
`c61908df28e212f141281e60b0142791c29c8628d8b26243dc9b5534cf9f9669`.
Four compiler attempts, two vstd attempts, two unsuccessful harness attempts and
47 direct applications are retained. Both matched-vstd runs verify2225/0;
internal mutant tests remain uncompiled/unrun. Disclosed masked controls and
earlier metadata-recording gaps receive no retroactive credit. Final preservation
checks retain2345 live paths and all1401 immutable/1331 native guarded paths.

Next is an independent diagnosis of the existing event mapping and the smallest
sound destruction-specific access/nested-loan design. No global mutability
relaxation, assumed reference equality, new std primitive or source-level
replacement Drop is authorized. Stage allocation exhaustion triggers this
reassessment, not termination of the user-authorized side quest. No live adoption,
native builder acceptance or final database-cell closure follows.

## Parent continuation: original borrowed-value example now discriminates

The unchanged immutable WriteBack example now verifies its real Drop and the
caller's original value becoming seven (2/0); OFF still fails that caller (1/1).
Renamed immutable owners, unaffected original references, two distinct owners,
sequential scopes and a two-reference owner's preserved field also verify.
Wrong-value/reference/scope claims and a false real destructor reject at backend
postconditions. Actual executable divergence still rejects after backend success.

This remains an isolated, default-OFF, UNACCEPTED compiler candidate. Besides
canonical call timing and checked reference-frame postconditions, it adds an
explicit compiler ownership rule: release consumed final reference fields AFTER
the real checked normal/no-unwind destructor. That rule needs independent
semantic review; successful SMT checks alone do not justify it. Global datatype
resolution axioms and source mutability rules are unchanged. No live compiler or
native code is adopted or changed.

The earlier delegated architecture review returned no assessment. Local diagnosis
and this substantively new implementation do not turn that failure into review
credit. The new semantic gate concerns the actual field-release rule, exact
ownership/destruction correspondence and rejection of inappropriate reuse.

Six compiler/vstd pairs, three five-test mapping builds and 47 direct applications
are retained, including eight early compiler-construction failures and the two
fifth-runtime caller failures. Only the sixth runtime establishes the milestone.
Final-source mapping tests pass 5/5. Fresh executions of authenticated existing
harnesses pass 43 external-trait, 118 mutable-reference and 73 invariant tests.
An initial Python-launcher library-path mistake produced 234 pre-verifier loader
failures; the corrected 234 launches pass and remain separately itemized.

Read-only checkpoint:
`/private/tmp/verus-crash-consistency94-20261105/drop-effects-parent-002/handoff/REPORT.md`.
All3685 payloads rehash; seal SHA256
`5fb0bcd223b0be2959adbc5b2614053c5c627ba855f548c46a92c3e38029253b`.
Before this parent documentation update, all2345 live paths,1401 immutable and1331
native guards, seed packages/runtime and owned Git identity matched. Historical
Git-object-store nonmutation remains NOT_CERTIFIED. Full rehashed-MIR mutation
coverage and independent diagnostic-repair closure remain open.

Native builder cleanup, panic/TLS state, exceptional paths and generic/imported
destruction still need work. All eighteen original database completion cells
remain OPEN; this is not redb commit, recovery, KV or Infer acceptance.

## Independent review of the working borrowed-value rule

The reviewer reproduced the unchanged example (ON 2/0; OFF 1/1) and found no
demonstrated unsound acceptance within the deliberately narrow subset. The
post-call field-release rule has a defensible ownership argument, but remains
private and unaccepted: testing of the real compiler mapping is incomplete.
Five tests of an already-built accounting structure do not test how the actual
compiler captures and connects Rust's ownership, source binders and destructor.

The reviewer also corrected a parent evidence error: the NEW straight-line move
fixture still fails on an unsupported bool before the ownership-transfer gate.
It earns ZERO isolated-transfer-control credit. The straight-return fixture does
reach the intended return restriction. No bool/move semantics will be broadened
merely to get a cleaner negative test. The original sealed checkpoint stays
unchanged; a separate erratum withdraws its stronger claim.

Review: `/private/tmp/verus-crash-consistency94-20261105/review-normal-drop-20261105-1956/REPORT.md`.
All142 review payloads and the published seal authenticate. The next bounded
stage challenges actual compiler captures/origins/glue and erased execution
through the full pipeline, with separately bound test instrumentation and no
live adoption. Both diagnostic collision repairs reproduce, but broader
diagnostic closure is still open. All18 original database cells remain OPEN.

## Validation attempt interrupted; independent byte-method work continues

The delegated real-pipeline validation attempt failed at the provider safeguard
after read-only input inspection. It produced no implementation, compiler build,
verifier result or validation assessment. F2 remains OPEN. The task is not being
reworded or retried via another execution route; the compiler candidate stays
private and unaccepted. The failure supplies no evidence of a code defect and
no explanation of the safeguard's internal trigger.

Ordinary, separate native proof work can still progress: establish what each
successful RawLeafBuilder append writes to the actual borrowed bytes. This uses
runtime005, not the experimental destructor feature. A selected-method theorem
does not prove construction, Drop, original-loan release, caller establishment,
panic handling, or any final database guarantee. Its boundaries stay explicit.

## Exact native append byte-step reference checks

The separate method proof now succeeds on unchanged runtime005, without the
experimental destruction feature. Five exact native bodies check: append and
its selected arithmetic/endpoint helpers. The full-page contract specifies both
payload writes and optional LE offset words, a single count increment, scalar
preservation, and unchanged bytes everywhere else. Checked lemmas give payload
readback, decoded offsets and byte frames. Arbitrary bytes, all four width modes
including fixed zero, and symbolic counts through65535 remain in scope under
explicit capacity, position, width and integer preconditions.

Final provider:8/0 queries (five native exec, one authored witness, two proofs).
NEW ordinary imports:3/0 client bodies plus2/0 symbolic/all-width/zero-fixed
witnesses. Three clean incorrect-claim controls reject their intended backend
postconditions. All14/24 applications remain recorded, including setup/frontend
and authored-witness failures; one earlier negative with recommendation notes
earns no clean isolated-control credit. No compiler/vstd/Cargo/harness builds.

The reference deliberately omits native new and actual Drop. Its authored direct
construction witnesses are NOT native construction or lifetime evidence; they
can describe partial builders the real destructor would reject. No error
rollback, completion invariant, C0 grammar composition, caller establishment,
sorted/unique keys or final database guarantee is inferred. The checked
borrowed-slice future frame uses existing specification support, not the new
post-Drop compiler assumption. Independent review remains required.

Read-only checkpoint:
`/private/tmp/verus-crash-consistency94-20261105/native-builder-step-004/REPORT.md`.
All106 payloads authenticate; seal SHA256
`380ba9e52c42c644906706595a44ea65c5667169d509839451002a7ecc042ad5`.
The successful full initial inventory was after application001; its timing
limitation and two non-verifier reader errors remain explicit. That snapshot
and the final check preserve2345 live paths,1401 immutable and1331 native guards,
owned Git and native Git identity. This parent evidence update follows that
frozen check. Historical objectstore remains NOT_CERTIFIED. F2 compiler
validation and all18 original database cells remain OPEN.

## Append reference independently reviewed

The independent reviewer found no blocking localized defect in the conditional
append reference. All106 reference payloads authenticate, and all five native
bodies, signatures, private fields and lifetime match independently. Seven fresh
applications include the full provider, NEW imports, all-width and fixed-endpoint
above-u32 witnesses, required_bytes counts above65535, and three clean intended
incorrect-claim rejections. The review preserves the exact explicit domains and
does not infer native constructor/Drop/caller/producer/database correctness.

Review: `/private/tmp/verus-review94-byte-step-20261105-UyVDNcAR/REPORT.md`.
The parent authenticated all204 read-only review payloads; seal SHA256
`25c8f0da5a30c4fa2d0d34ccb0e8fb9d347f3f6a9c0b40b703963cf4234b0a6e`.
The next independent proof step is the real native constructor's initial byte
relation, with release/debug cfgs stated separately. This uses runtime005 and
does not adopt or claim to complete the experimental destructor feature.

## Native release constructor now connects to one-pair readback

The selected native constructor now checks alongside the five append bodies.
It establishes the exact release header bytes and preserves the remaining page,
for every count0..65535, all optional widths including fixed0, and arbitrary
reserved key bytes. A four-byte-page imported witness is symbolic over that
whole count domain; it does not claim a completed usable leaf or safe Drop.

Fresh ordinary imports connect native new to append's full byte formula and,
for one completely filled pair, connect the produced bytes to the existing C0
native decoder. Reading returns exactly the arbitrary original key and value,
with arbitrary spare tails. A separately authored, checked consuming adapter
explicitly returns the stored slice. This adapter is NOT a native method or a
proof of the real builder's destructor, implicit release or lifetime closure.

Final provider010 checks14/0 (six native exec, inherited witness, authored
transfer, five proof bodies); unchanged C0 provider011 checks32/0. Imported
boundary012 checks1/0, new/append/readback014 checks2/0, and full-byte
constructor/append015 checks1/0. Two natural incorrect claims reject solely at
their intended backend postconditions. All19/32 applications and rejected
drafts are retained; exactly12/12 NEW authored methods, zero compiler/vstd/Cargo/harness rebuilds.

Debug initialization remains UNPROVED: the unchanged native mutable-slice
IntoIterator loop lacks useful interface facts. No replacement loop, new trusted
boundary or experimental destruction feature is used. The one-pair readback
client additionally meets C0's u32 endpoint bounds; the more general append
provider's larger fixed-width domain is not narrowed.

Read-only checkpoint:
`/private/tmp/verus-crash-consistency94-20261105/native-builder-init-005/REPORT.md`.
All152 payloads authenticate; seal SHA256
`958188468a4a5cab1b75e50062e23b1ac88a4cded8cc532d82c71715ced66833`.
Fresh initial live inventory preceded every launch; final2345 live paths,
1401 immutable/1331 native guards and owned/native Git state match. No shared
ref differences appeared in this stage. Historical runtime/source-map and
toolchain qualifications remain explicit; objectstore remains NOT_CERTIFIED.
This parent documentation update occurs AFTER that frozen preservation check.
Independent review is pending. All18 database cells and compiler F2 remain OPEN.

## Release constructor and one-pair readback independently reviewed

The new independent review finds no blocking defect within the exact release-only
reference. All123 read-only review payloads authenticate; seal SHA256
`8800de5d0a39f95f7acf77f21ef89a5d85ac0d408d389ec606df8e5509fd2d56`.
Report: `/private/tmp/campaign94-init-independent.LjxfdaeI/REPORT.md`.
Fourteen fresh applications include eight complete scopes, two frontend failures,
one rejected draft, two clean negative controls and the expected debug rejection.
Three new review methods give additional ordinary-import observations; no builds
or source repairs. Native body/field/lifetime correspondence independently matches.

The result is not actual native Drop or a producer/caller lifetime theorem.
One-pair readback keeps exact key reservation and explicit u32 used-end bounds;
debug remains unproved. The next byte-level proof will induct over repeated native
appends and connect a fully filled multi-entry image to the checked decoder.
It will retain the explicit authored transfer and all original database gaps.
This remains separate from the safeguard-interrupted compiler validation.

## Repeated native appends now produce an entire readable byte image

The new conditional release proof handles symbolic count1..65535, not just one
entry. A checked prefix invariant proves next-append readiness and preserves all
earlier offsets/payloads. Completion gives the existing C0 grammar, exact bytes
for every key/value, count and key-reservation completion, reserved byte1 and
spare-tail preservation. All four width modes, fixed0, empty/arbitrary payloads
and arbitrary tails remain supported under explicit capacities and C0's u32
used-end bound. No general append-domain or count0-constructor narrowing follows.

The composing loop is AUTHORED; it calls unchanged native new/append, then the
explicit authored slice-transfer adapter. This still is NOT the actual native
producer/destructor/caller lifetime. It establishes neither sorted/unique keys
nor storage/recovery or final database properties. Actual Drop, debug and F2
remain open; the safeguard-interrupted task is not retried or rerouted.

Final provider015 checks26/0 (six native exec, three authored/inherited exec,
12 proofs); unchanged C0 provider013 checks32/0. NEW ordinary imported clients017
check4/0: arbitrary-index native readback, last entry, out-of-range None, a
concrete empty/nonempty two-pair example and symbolic fixed-zero counts. Two
incorrect claims reject solely at intended assertions; both corrected positive
counterparts pass. All23/48 launches include every frontend/resource/proof draft
failure. Sixteen/20 NEW proof/client methods plus six separately listed pure
specification functions; no compiler/vstd/Cargo/harness rebuilds.

Measured direct-application time totals63.503s, including failures and ordinary
fixture compilation. Same final source/runtime/rlimit60 verifies26/0 in both
contexts: isolated4.627s/4.855s, shared1.658s. These few warm samples are not a
statistical benchmark or universal speed claim. Isolation helped draft diagnosis
but is not justified as a permanent speed default. No production flags change.
Quantifier-trigger coupling, scalar-shape loop frames and checked prefix-length
facts are concrete future assistance candidates, not new Infer feature/ablation
or burden-reduction credit.

Read-only report:
`/private/tmp/verus-crash-consistency94-20261105/native-builder-prefix-006/REPORT.md`.
All170 payloads authenticate; seal SHA256
`2738d3d7184f365cad67df41afb0b9767c746df5ae9b8d57763be9f62ba64183`.
Fresh initial baseline precedes001 and final follows023;2345 live paths,
1401 immutable/1331 native guards and owned/native Git match. Historical source
inventory/toolchain/objectstore qualifications remain explicit. This parent
documentation update follows that frozen check. NEW independent review is next;
all18 original database cells remain OPEN.

## Independent multi-entry review supports the bounded byte composition

Authenticated all189 read-only review payloads, exact membership and report/
manifest/seal hashes. No blocking defect was found within the explicit RELEASE-
ONLY premises. The reviewer independently matched all six native bodies,
signatures, fields and lifetime. Thirteen/24 fresh applications and five/5 new
methods include ten complete successful scopes and three intended assertion
failures, each with a passing counterpart. Fresh ordinary imports also check
arbitrary index/range observations, every count1..65535, fixed0 and arbitrary
reserved/spare bytes, plus unsorted/duplicate concrete keys. The latter are
byte witnesses, NOT sorted-map correctness.

Review: `/private/tmp/campaign94-prefix-independent.HFq5Eg40/REPORT.md`.
Seal SHA256:
`7f2397a8a00413afa376836f4f660919f680e556c3bbe6d8b5ce251c1a9083a0`.
Actual Drop, native producers/callers/lifetimes, debug, panic/errors, storage,
sorted/unique KV and all18 database/history cells remain OPEN. No compiler
candidate is adopted and the safeguard-interrupted validation is not retried.

Next bounded work is discovery of native leaf lookup and its comparator closure:
can a sorted leaf return the right value for a present key and report absence
otherwise? Exact native bodies, ordinary imported decoder contracts and checked
comparison must remain distinguishable from any authored specialization or
stdlib boundary. No lookup proof or feasibility is claimed at this checkpoint.

## Sorted-leaf lookup now has a conditional checked reference

For a well-formed leaf with strictly increasing four-byte u32 keys, the actual
native binary search finds the exact original key/value bytes or correctly says
the key is absent. It also terminates safely and returns the insertion partition.
The decoder/search provider passes37/0 with debug assertions both ON and OFF:
21 unchanged C0 bodies plus four complete search/u32 method projections and six
authored proofs. The narrowed comparison-only traits are NOT the original full
native Value/Key interface or an arbitrary-comparator theorem.

Fresh ordinary imports compose this with the reviewed multi-entry byte builder.
That composition remains RELEASE-ONLY and uses the explicit authored loop and
slice-transfer adapter; it is NOT native Drop, producer/caller or lifecycle proof.
Counts1..65535, all u32 queries, arbitrary valid values/empty values/spare tails,
variable/fixed4 keys and independent variable/fixed values remain supported.
No count0, unsorted/duplicate keys, disk, commit/recovery or final-cell credit.

Actual usize::midpoint first failed at the frontend. One prospectively declared
PRIVATE trusted std API contract plus matching vstd2225/0 unblocked that call.
It states mathematical floor midpoint, not an application/search axiom or a
checked Rust-core implementation. NEW independent boundary review is pending;
no live std/compiler adoption or extension of the earlier codec reviews follows.

The stage used23/48 applications,19/20 new executable/proof methods and1/2 private
vstd builds; no compiler/Cargo/harness builds. Five clean backend controls reject
incorrect values, incorrect absence, unsorted inputs, malformed queries and an
incorrect midpoint. All six failed drafts remain uncredited and itemized.
Provider009, builder010, client011 and witnesses014 form the final matching
artifact bundle; exact provider022/023 supplies the debug/release repeats.

Important preservation exception: the original temporary redb/Fedimint source
files and native Git metadata disappeared during this stage. No cause or actor
is attributed. Launches stopped during investigation. All1331 exact guarded file
CONTENTS were recovered read-only from surviving pinned Git archives into NEW
private non-repository snapshots; every original hash matches. Twelve symlink
referents are materialized contents, NOT preserved original link metadata.
Original-path preservation FAILED and original native Git continuity is
UNAVAILABLE. The70 other immutable inputs survive unchanged. The final2345 live
paths and owned Git identity match the pre001 snapshot; this documentation update
follows that check. Do not describe the original1401/1331 paths as preserved.

Sealed1676-payload reference:
`/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/REPORT.md`.
Seal SHA256:
`39cd35936577c41b43949776012c984a874892729b4da5fa15760fc71721b04c`.
Only022/023 separately time preflight and subprocess: warm preflight1.944/0.979s
and subprocess1.110/0.969s. Earlier per-run times omit preflight; no cold benchmark
or general speedup follows. NEW independent lookup/boundary review is next.
All18 database cells remain OPEN, historical Git objectstore remains
NOT_CERTIFIED, and safeguard-interrupted compiler/destructor validation remains
unanswered and uncredited—not retried or rerouted by this separate lookup work.

## Lookup review is qualified; parallel work does not replace the main goal

The new independent review authenticates203 payloads (seal
286358f51477c3f14193f5230a047759f26e0f17c7655fc63427971809129e53).
Fresh provider37/0 in release/debug, builder26/0 and client3/0 reproduce; seven
backend controls discriminate. Supplemental013 fails two exported partition
restatements, so its entire selected scope and controls receive ZERO credit.
No target counterexample or mere-trigger explanation is established by that
failure. Original native paths remain missing; midpoint is explicitly assumed,
not Rust-core verified or adopted. Three added shared checkpoint refs are
disclosed without attribution. This is not an all-fresh-checks-PASS verdict.

Review: /private/tmp/verus-leaf007-independent-review-tOKSrsOe/REPORT.md.
After the user's request for more parallel work, three bounded isolated lanes
are planned in evidence/parallel-proof-lanes94/PLAN.md. They cover ordinary
normal-return cleanup support, a distinct Fedimint wallet fee proof and a cached
third-repository decoder trial. The main line remains actual redb commit/recovery,
not more leaf helpers. No blocked task is rerouted. All18 final cells stay OPEN.

## Native recovery selection and publication now have a checked reference

November6: six complete native header/selector bodies and full selected field
layouts authenticate against the recovered pin. Provider003 verifies16/0;
fresh ordinary policy/frame clients007 verify2/0. Two-phase recovery deliberately
does not pick a newer secondary, and rejects a corrupted primary. Checksum
recovery can select/fall back to a valid secondary; ties keep the primary.

The native helper calls and mode/unflushed assignments compose in their actual
order in an explicitly authored in-memory fragment (006,3/0). Three conditional
bridges then connect prepared candidates, interrupted two-phase publication,
and acknowledged images of both modes to actual native selection (012,5/0).
They expose, rather than discharge, the missing native parsing/corruption and
write/flush/protection correspondence. Fresh imported bridge client021 verifies
1/0. Four isolated wrong policy/error/tie claims reject with passing counterparts.

The21/48-application checkpoint is sealed with167 read-only payloads:
`/private/tmp/verus-crash-consistency94-20261105/native-recovery-selection-008/REPORT.md`.
Seal SHA256:9514053067d06dd127ea09bff783f0d908d3547e1988f9ac2f3d84b7b4b762f3.
All drafts, warning-qualified controls and the withdrawn010 scratch VIR remain
explicitly disclosed. Independent review is pending; no full native protocol or
database property is accepted. The2347-path source snapshot matches before this
documentation update; original native paths/Git continuity remain unavailable.

The three isolated parallel lanes returned without compiler adoption: cleanup
hits its slice-owner/no-unwind domain boundary, Fedimint wallet fees hit native
u64::div_ceil, and rust-hex hits generic AsRef then IterMut metric certification.
Their checked dependency partials and incremental patches remain retained in
the child worktrees and private scratch, pending deliberate review/integration.
None is a full native-target proof or a newly established vulnerability.

Next main-line work targets the layout reconciliation used by consuming native
recovery, including its error cases. The all-six/both-mode/mixed-history goal is
unchanged, and all18 final cells remain OPEN.
