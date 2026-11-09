# Independent combined review: frozen storage round109

## Verdict and authority

The two frozen authored libraries withstand this bounded independent review and
the six fresh full-crate verifier applications below. No unsound or false local
model theorem was identified. This is NOT acceptance, native-body verification,
an executable/native producer, a composition theorem, or authorization to adopt
any storage law. The important native-linkage objections below remain OPEN.
All eighteen final cells remain OPEN. S0, Infer, full-scope, import/export and
trust barriers are unchanged. No acceptance certificate is issued.

Anchor notation: A is /private/tmp/redb-storage109-lane1-20261108T034224Z;
B is /private/tmp/redb-storage109-lane2-wqh8Ege8; controls are
/private/tmp/redb-storage109-independent-controls-vKPHDj8s. Native source anchors
are relative to /private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/recovered-native/redb/src;
cached_file.rs and page_manager.rs are under tree_store/page_store. Other output
paths are relative to /private/tmp/redb-storage109-review-QFf6FvBP.

One reviewer worked directly, without subdelegation or a review chain. First
clock observation: 2026-11-08 04:07:22 UTC. Absolute handoff cutoff: 04:22:00 UTC,
earlier than first observation plus fifteen minutes (04:22:22). Optional-work
cutoff: 04:19:22. The round deadline remains 04:35:26. Exact final elapsed time,
scratch/free-space samples, final authentication and archive identities are in
SEAL.json and HANDOFF.json; they are not substituted for this start observation.

## Anchored findings and objections

### O1 — OPEN: pure enqueue is not native admission

A/buffered_flush109.rs:49 and :585 prove last-map-write-wins without changing the
backend. The singleton bridge at :691 explicitly selects one coalesced entry;
it is not a snapshot theorem for the entire native cache or all other map slots.
Native cached_file.rs:907 removes an existing buffered value, while :927 and
:929/:939 may call fallible eviction/writeback during admission, before the
explicit flush at :596. Thus actual admission can issue writes and fail even
when the authored enqueue only changes pending state. Native insertion also has
an absent-key requirement, not arbitrary replacing insert. Equal bytes and
matching labels cannot discharge this mismatch. The candidate correspondence
ledger properly marks the admission/resource/lifecycle binding OPEN. No lifecycle
pipeline was implemented or investigated in this review. This is a blocking
objection to native composition, not a counterexample to A's map theorem.

### O2 — OPEN: backend/crash laws and finite controls are not identical

A:98, :106 and :118 define chosen conditional laws: successful writes overwrite
their full range; failed writes select old/new visible bytes; persistence chooses
prior durable/current visible bytes; successful sync makes durable equal visible.
These are model definitions, NOT independently proved std/OS/device guarantees.
The controls' oracle.py:19 includes an additional UNKNOWN symbolic cell on failed
writes. That UNKNOWN branch does not automatically translate to A's old/new byte
selection. Delayed persistence of other historical values is also not exhausted
by A's transitions, as CORRESPONDENCE.md explicitly discloses. Likewise, the
controls have a persistent error latch while A's single-call interpreter has no
cross-call latch state. Therefore the finite controls and A agree on important
shared properties, but neither their entire transition domains nor native crash
coverage are established equivalent. No finite test is adopted as a primitive
contract. These disclosed boundaries do not invalidate the conditional theorem.

### O3 — OPEN: opaque continuations need an effect/history frame

B/ack_cfg.rs:85 generates a control interpreter, not storage bytes. Its local
mode at :52 and public-Ok theorem at :198 establish event ordering only. Native
transactions.rs:2098 has a nontrivial quick-repair retry/callback summary; :2147
through :2162 contain post-publication tracker/free/savepoint continuations;
:2170 calls a fallible epilogue. That epilogue has fallible stages at :2217,
:2222, :2228, :2230, :2244 and :2252. B includes their selected control/error
nodes, including header holds at PCs22/26, preparation PC21, final flush PC28,
manager publication PC31 and post-publication failures PCs35/37..41. Its summary
success edges do NOT prove those callbacks are inert or preserve durable roots.
The controls deliberately stop eighteen parameterizations at
OPEN_CALLBACK_EFFECT; a B marker history is not a refinement of those effects.
The real outer return passes helper assertions, logging and successful latch
disarm (transactions.rs:1842 and :1775), with resource/abnormal bindings still
OPEN. Generic panic/abort/crash outcomes are explicitly separate, not a proof
that concrete native interruption points or return/destruction glue are covered.

### O4 — OPEN: no cross-model/native adapter R

B/ADAPTER.md:49 specifies, but does not prove,
R(native, cfg_state, cache_state, StorageImage). The missing obligations include
entry mode/resource/ownership/history relation; both full 320-byte offset-zero
header admissions (page_manager.rs:1617 and :1837/:1859); complete header and
selector/mode framing; each flush's entire A drain plus backend sync; error-prefix
simulation; and continuation/durable-history framing to the actual public return.
Three symbolic header cells are not the serialized header/checksum/recovery
format. Resource/lifecycle bindings, root reachability, allocator authority, all
histories and abnormal-path bindings stay OPEN. No matching-event/byte identity,
native producer, or composed acknowledged-durability theorem is established.
Do not equate the early old secondary with the newly prepared slot.

### I1 — confirmed historical reporting defect; already corrected by parent

B/run_check.py:71 scans per-attempt *.vir files, but selected --export arguments
name root files. B/REPORT.md:58 incorrectly states that no .vir was emitted;
root ack_cfg109.vir (375929 bytes) and consumer109.vir (391362 bytes) really exist
and authenticate. ACK-PUBLICATION-CORRECTION.json correctly preserves this defect
without changing historical evidence. Historical negatives 006/008/010/012 did
not request export, so their failures are NOT historical requested-export refusal
tests. Fresh checks below close only that local replay evidence gap. No .rlib,
binary-import test, complete artifact pair or native artifact acceptance follows.
The source-module consumer is exactly that, not a separately imported binary.
No new delegate or historical-evidence rewrite is needed for this known finding.

### I2 — bootstrap provenance qualification

All selected baseline records, all candidate/controls payloads and the direct
verifier/solver/toolchain records authenticate before every fresh verifier call.
The direct installed Python executable/framework and loaded controller modules
were captured at entry, without invoking /usr/bin/python3 or rustup. Supplemental
bootstrap/display/patch-tool identities and some additional controller/module and
parent-overlay identities were recorded later, in supplemental-identities.json;
they match retained inherited expectations where available, but this is NOT
retroactively claimed complete first-use entry/preexecution authentication of
every bootstrap utility. This explicit audit qualification remains; no exhaustive
loader/module closure or atomic snapshot/provenance claim is made. The frozen
operational compiler/source correspondence remains UNESTABLISHED; original native
path preservation FAILED and native Git continuity is UNAVAILABLE, independently
of modeled verification results.

## Model review details

**A:** model_entries (:137) and model_stripes (:166) recursively generate attempts
from arbitrary finite nested pending sequences and a freely supplied decision
function; a desired issue/sync trace is not a theorem precondition. Geometric
validity is explicit. check_stripes (:313, particularly :325) retains every entry
of the failing stripe, even successful earlier entries; earlier completed stripes
are empty and later stripes unchanged. successful_prefix_retained (:434) handles
every failing prefix length, rather than one selected example. Overlapping writes
use the actual enumeration-order fold; arbitrary_order_nonoverlapping_byte (:671)
requires unique byte coverage and does not invent native non-overlap. Partial
write effects are witnessed at :419; sync-Err progress/no rollback at :716.
model_flush (:195), flush_refinement (:385) and flush_sync_order (:730) prevent
sync after write Err and put sync after complete drain, retaining drained pending
state even when sync returns Err. Successful modeled durability follows the
conditional sync definition. Boundary::OpenAbnormal (:744) carries no state or
durability guarantee and has no native-reachability proof. Source inventory:
26 proof bodies, 31 verifier obligations, zero original native executable bodies.

**B:** the run invariant is proved inductively from initial (:170/:187), rather
than supplied as an unproved success trace to the top theorem. Public Ok under
local_mode yields markers 23 < 24 < 27 < 28 < 31 < 46. effective_two_phase (:48)
accounts for quick-repair/multi-process forcing, while None, enabled shrink and
multi-process routes retain explicit OPEN frontiers, not broadened credit. The
reachable public-Ok witness (:291) avoids vacuity. The reachable error witness
(:262) retains completed final-barrier/publication history without acknowledgement;
it does not assert rollback. The early crash witness (:277) keeps a valid distinct
old secondary after preparation and before the first header hold/write; later
secondary-field updates are author-defined projections, not native validity laws.
Source inventory: 15 proof bodies and 16 verifier obligations. Retained source-
module consumer: 17 proof bodies/18 obligations; inspected, NOT freshly replayed
in the six-check allowance. Zero original native executable bodies were checked.
No new assume/admit/axiom/external_body tokens were found in either frozen source.

## Six fresh existing-verifier applications

Exactly six charged applications and six actual launches, sequentially, one
verifier worker and --num-threads 1. No launch error or timeout. Each has a fresh
explicit --export destination under check-NN. Every directory retains its exact
source, invocation.json (complete explicit child environment, argv, cwd, source
hash, export path, timeout), unedited stdout.txt/stderr.txt and status.json.
Mutation metadata and patch output are retained for the two negatives. Sources
01/03 and 04/06 are byte-identical to their respective frozen candidates, not
merely semantically restored. Each verification reports entire-crate=true.

| Check | Variant | Raw exit | Verified/errors | Seconds | Exact requested export outcome |
|---|---|---:|---:|---:|---|
| 01 | A frozen full positive | 0 | 31/0 | 1.373 | check-01/buffered_flush109.vir exists, 847390 bytes |
| 02 | A failed-write rollback mutant | 1 | 30/1 | 0.795 | check-02/buffered_flush109.vir absent |
| 03 | A byte-identical restored positive | 0 | 31/0 | 0.784 | check-03/buffered_flush109.vir exists, 847390 bytes |
| 04 | B frozen full positive | 0 | 16/0 | 0.460 | check-04/ack_cfg109.vir exists, 375929 bytes |
| 05 | B PC27 jumps toward ACK46 | 1 | 14/2 | 0.505 | check-05/ack_cfg109.vir absent |
| 06 | B byte-identical restored positive | 0 | 16/0 | 0.510 | check-06/ack_cfg109.vir exists, 375929 bytes |

Check02 changes only `decision.ok || decision.changed.contains(address)` to
`decision.ok`; the localized partial-write assertion at source:426 fails.
Check05 changes only next_pc to jump PC27 -> 46; step invariant preservation
and successful-prefix reachability postconditions fail. These are semantic
verification failures, not parser errors/crashes or presumed negative outcomes.
Fresh paths were absent before launch; neither negative produced its requested
export. No check produced a .rlib. Different positive export hashes are recorded
without claiming deterministic serialization across different source paths.

Fresh positive export SHA256 values:
- 01: ad7628de372e6a82bb0d2169e500eb0640ca61442a1ff26e43b7684858ada6f6
- 03: 601996bbb9c3d60fd6d8a8bc81a7bb60e8b8930a2de82b1832ec9dca73f56f53
- 04: 76b675b9f655e0059b0d498ee87047497101a041398e2780a0a167c53c974cbf
- 06: d83d912b6623ee76d5e84e9c156a93be6295a7c8a1e70cdc2f1fd466898a7d2b

The pinned recovered runtime verus launcher, rust_verify, z3, vstd/builtins and
selected installed 1.99.1-aarch64-apple-darwin toolchain records were used; no
build/codegen/native application was run. All six use --no-cheating, no manual
extern/import, rlimit20 for A / rlimit30 for B and a 90-second process-group timeout.
The individual recorded invocations are authoritative; no seventh verifier run
or fresh consumer/import application was attempted.

## Historical inspection and independent Python replay

historical-inspection.json inspects all 26 retained verifier-launch directories:
12 A and 14 B. It preserves parsed raw statuses, full invocation data, diagnostics,
source/stdout/stderr hashes, verification summaries and actual export arguments.
This is inspection, NOT independent replay of those historical applications.
A's 13 charges include the disclosed failed mutation setup; absent attempt002 is
a numbering defect, not a missing launch. Attempts010/011 carry the same retained
eager-drain mutation after the failed restoration patch; 011 is not credited as
a positive. A's other retained rollback/successful-sync controls were inspected.
B's parser/resource/invariant failures and four intended semantic negatives were
inspected along with their corrected counterparts. The known root-export issue
is kept distinct from theorem success and artifact import.

Fresh Python-only replay copied seven selected control inputs into controls-replay.
The ONLY relocation edit sets the copied authenticate.py ROOT to its own scratch
directory, using apply_patch, so generation cannot write the original evidence.
Both subprocesses use the direct framework Python, a minimal explicit environment,
90-second limit, full raw stdout/stderr and actual status. All 21 unittest tests
pass; generation passes; all seven regenerated inventories/coverage products are
byte-identical to the originals. They retain 844 storage cases, 195 commit/opaque
cases, 12644 crash observations, six semantic mutant/corrected pairs and seven
source-mutant/corrected pairs. The outcomes include 18 OPEN_CALLBACK_EFFECT cases.
Lexical refusal and parse failure are not mislabeled mutant rejection. This
replay gives finite Python/reference/source-token evidence only, not native
execution, universal proof, byte-format recovery or acceptance.

## Authentication, identities, failures and resources

authentication-entry.json, preexecution-01..06, controls-preexecution snapshots
and authentication-exit.json recheck all 369 current selected INPUTS records and
the 93 A / 111 B / 133 controls payloads against their sealed manifests, plus
pinned roots/archives and controller identities. Each main authentication has
783 distinct paths. The five explicitly disclosed historical document drifts
remain disclosed; their selected CURRENT records match. The ongoing parent README
was read for context, deliberately not authenticated as an immutable input.
Additional used overlays and late-observed utility identities have their own
supplemental exit comparison. No parent/native/candidate/source/STATUS/original
evidence file was edited. All newly authored edits are in this fresh scratch.

archive-inspection.json independently compares every regular archive member with
its loose counterpart: A96, B113, controls135 members, all matching. This is not
an exhaustive/atomic provenance or original-path continuity claim.

Key frozen SHA256 identities:
- PLAN.md: ecc47ad6648c025d6ee5d0d6698ff6fc9f0afda18e79f30f7e40f60dbc1fac90
- INPUTS.json: ed423164d494ad5176e3fd04ddee983153fc7e29b4a7d43e1cebe7efac063bb4
- A HANDOFF.json: f92d36f00ca9de1c8561031cb338b430dd2e7459912c5029a80f6ab0cb3b96aa
- A MANIFEST.json: 2fb03dedb6cf85e336ee6aba824876b1adf1d2506ebcb39dcf1faecbf49dc9fc
- A source: 29485224c99ba2f994012ff53b2ddc98c9cab79daaf7b3fc2a855b952d2fdc4f
- B MANIFEST.json: 2ceca5762e9c983e2ed6a2f48ee0efee0f89237500c6c3898f987346f52ae053
- B source: 447a7fcfa23fcdec6114be0ec4eb27274ed4133efbc7d81fdac896dffd9306a2
- Controls MANIFEST.json: 764153f9e8b0b6d6f5738d1e16a38bcd9e3b4fd6552cead8161c4db6d714d01e
- A archive: f681a1c4bf93b5f36991d6403338b982c1ae1556ff2144acac8db3e7c51b38a0
- B archive: 3fab8b799531a87e3671ec6abb6bdbc40ec6262789d11ce76c1a529d2407c18b
- Controls archive: 961fecdf8c77c350eef61fc2c84d501727d8117abcecde04451e0bfbbedf54c5

The first combined read-only inspection returned shell127 because a zsh loop
used the special `path` variable, unintentionally replacing that process's PATH;
it was immediately rerun with file_path. An optional rg probe also reported a
nonexistent additional-inputs.json; the actual retained inputs were then used.
A pre-launch controller amendment changed mutation patch context to the complete
source line before ANY verifier attempt; initial script/authentication snapshots
are preserved. These are disclosed exploratory/controller issues, not hidden
verifier launches, failed mutant rejections or extra application credit. All six
actual verifier setups/launches succeeded; two intentional semantic runs exited1.
One final report-only anchor-correction patch initially failed context matching;
it was reapplied with exact context before sealing, with no verifier application.

Observed scratch before sealing was about 11.4 MiB, far below 512 MiB; free-space
samples throughout exceed 50 GiB, strictly above 16 GiB. Authentication/launch and
seal boundaries are sampled observations, not continuously enforced disk or
peak-memory quotas. No toolchains/caches were copied. No builds, Cargo, rustup,
network, Git writes, native application execution, new trusted assumptions, gate
changes, source integration or trust adoption occurred. None of growth049,
zero-growth050, mutation030, refused-padding95 or refused drop-pipeline work was
resumed or rerouted.

## Minimal next step

Parent may retain this sealed review and fresh raw evidence, preserving the known
publication correction overlay and all eighteen OPEN cells. Keep the missing
adapter R and admission/backend/callback/history/resource/abnormal bindings OPEN;
do not infer permission to implement the excluded lifecycle pipeline. Any later
attempt to establish these missing bindings needs its own authorization. No
additional review chain, implementation, integration or acceptance is requested.
