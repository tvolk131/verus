# Independent recovery021 review — findings first

November 7, 2026; America/Chicago. NEW independent READ-ONLY source/evidence
review of the sealed native-recovery-reencoding-021 extension. No subdelegation.

## Disposition and findings

**CONDITIONALLY SUPPORTED AS A BOUNDED SOURCE/EVIDENCE CHECKPOINT. No new blocking
soundness/correctness defect was established in the NEW021 extension within its
stated domain and inherited dependencies. This is NOT independent proof
reproduction, adoption, native resource proof credit, or final database acceptance.**

The separate six-body selector008 review is NOT duplicated or accepted here.
021's composition is conditional on those inherited selection contracts/policies.
Source identity and retained witness receipts are not a substitute for that review.
All eighteen final property/history cells remain OPEN.

### F1 — HIGH outer dependency: the native caller is not discharged

The interruption theorem starts from `database_bytes(before, true)`, not arbitrary
bytes returned by a native read. It assumes a dirty header in the roundtrip domain,
a successful original finalization at ONE supplied file length, and retries toward
the SAME recovered header. Immutable geometry, both transaction slots, mode,
`primary_unflushed`, and `recovery_required` are framed. Only selector choice and
the two count words can change. These are meaningful restrictions, not conclusions
about initialized databases, actual persistence, or reachable crash histories.

In particular, normal-slot encode/decode preserves the decoded header, not every
original input byte. A checksum-valid input with a nonzero unused/padding byte can
parse successfully but reencode with that byte zeroed and its checksum regenerated.
Thus parsing an arbitrary input does NOT establish that the actual old medium is
`database_bytes(parsed.inner, true)`, or that an actual interrupted rewrite changes
only the count/selector area. This is not a counterexample to the stated theorem;
it is a concrete reason not to erase its native-encoded-origin premise.

022's source-bound reassessment is corroborated by the native source excerpts:
`read_header` has direct reads, separate shared-reader behavior, lock transitions,
physical-length reads, read-only refusal, actual cache owners/destruction, a
flush-before-clearing-`primary_unflushed` branch, and later flag-clearing/error
paths outside021. `clean=true` from finalization does NOT clear either flag or
mean durable/repaired database state. Both policies in individual calls do NOT
prove mixed-mode histories.

**Concrete next-action effect:** retain021 only as a conditional byte-level
checkpoint. Before any later native use, the parent must reconcile selector008's
separate review and discharge the actual encoded-origin/length/cache/Drop/barrier/
flag/error premises under separately authorized work. Do not replace these with
an assumed native recovery/commit/flush oracle. This review starts no such work.

### F2 — MEDIUM evidence qualification: mutation030 remains uncovered

Application030's sole substantive error is resource exhaustion in
`TransactionHeader::to_bytes`, input.rs:1515. Its edit replaces raw corrupt-slot
return with a zero array. `110 verified / 1 failed function` is NOT an intended
semantic rejection and earns **ZERO complete-scope, mutation, or control credit**.
The source pair, original assessment and raw receipt are retained. No030 retry
occurred in this review; none is authorized or proposed. Passing corrupt-slot
roundtrip/claim counterparts do not fill this body-mutation coverage hole.

**Concrete next-action effect:** never report four successful native-body mutation
classes. There are three clean body rejections, one transition-premise rejection,
five claim rejections, and this explicitly uncredited class. Do not restart an
exhausted lane to improve the presentation.

### F3 — LOW evidence qualification:031 is a late location reassessment

031 changes only `if self.two_phase_commit` to `if false`. Its one substantive
error is the unchanged `assert(result@[9] == selector_byte(*self))` at input.rs:1585,
inside `DatabaseHeader::to_bytes` (1548–1598). The initial reader expected a
signature/postcondition diagnostic and retained ZERO. The separately retained
`source-bound-control-assessment.json` labels its later judgment `LATE_AFTER_STOP`.
The exact one-edit pair, all error headers, failed-function count and location
support that later **rejection-only** interpretation. There is no031 rerun,
second application, complete positive, retroactive initial approval, or native/
verifier bug established by the reader mismatch.

**Concrete next-action effect:** the final 14/9/11 accounting is supportable only
with BOTH assessments and their chronology visible. Preserve the original ZERO
record, the late label and the unchanged raw receipt.

### F4 — MEDIUM evidence qualification: live parent handoff documents drifted

All317 frozen021 files and all27 supplemental input files match their entry/exit
hashes. All2345 non-policy paths in the inherited2347-path parent guard also
match. However, live `GOAL.md`, `STATUS.json`, and `TAKEOVER.md` changed while
this read-only review was running; `SCOPE.md` did not. Consequently a blanket
claim that the entire2666-file initial observation, or the historical2347-path
guard including STATUS, stayed identical would be false. Exact hashes are below.

The later GOAL records a BLOCKED parent state awaiting separate authority, not
completion. A further exit observation records selector008 review delivery and
related progress metadata; its report was not opened or adopted by this reviewer.
All18 OPEN, S0 and no adoption persist. These changes do not alter frozen proof
inputs. This review wrote none of these files; no other actor/cause is attributed.

**Concrete next-action effect:** hand back against the unchanged sealed package,
with document drift disclosed rather than reset, restored, or silently excluded
from a claimed whole-snapshot match. No source repair or allowance change follows.

### F5 — HIGH outer dependencies / MEDIUM provenance qualification

No new application axiom or trusted implementation body was introduced by021.
That does NOT eliminate inherited TCB: TWO std bool conversion contracts, six
LE API boundaries, the fixed112 hash/codec/StructuralEq/compiler/std boundaries,
and existing array extensionality remain assumptions or inherited bounded
dependencies, not proved Rust-core/library implementations. The fixed112 dispatch
contains unreachable `requires false` branches for other lengths; no medium/long/
SIMD/general-page-hash closure or collision freedom follows. No trust/schema law
was installed by this review.

Original native paths remain MISSING; original-path preservation FAILED; native
Git continuity UNAVAILABLE. Recovered content at the supplied pin is not restored
original-path/Git continuity. Compiler map NONEXHAUSTIVE1084 and historical/shared
objectstore NOT_CERTIFIED remain explicit. Author snapshots and read-only hash
matches are not kernel execution/build provenance or continuous write attestation.

**Concrete next-action effect:** keep S0 whole-native admission, ordinary Infer
full-scope, import/export and fail-closed publication barriers unchanged. Neither
this report nor the current ownership-law design work authorizes trusted native
I/O summaries, scoped admission, Rust-core claims, or a final-cell promotion.

## Exact inputs and independent method

Path abbreviations used only in this report:

- P = `/private/tmp/verus-crash-consistency94-20261105/native-recovery-reencoding-021`
- N = `/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/recovered-native/redb/src`
- D = `/Users/tommyvolk/.t3/worktrees/verus/t3-60978068/evidence/redb-crash-consistency94`
- B = `/private/tmp/verus-crash-consistency94-20261105/native-parser-finalize-020`
- R = `/private/tmp/campaign94-finalize020-independent-review.0zjhIkeW`
- I = `/private/tmp/verus-crash-consistency94-20261105/native-io-boundary-022`

Native content pin supplied by the campaign:
`c82ab865bb362bc4af067d332adc44978ff64813`.

Read P/AUTHORIZATION.md, REPORT.md, SUMMARY.json, REPRODUCTION.md, SEAL.json,
MANIFEST.json; final and relevant draft sources; source/declaration/application/
import-retention/runtime ledgers; all34 raw input/invocation/receipt/stdout/stderr
sets; controls and both030/031 assessments; the four reassessments; late-handoff
and preservation qualifications. Read D/GOAL, SCOPE, STATUS, TAKEOVER and reread
the changed handoff state. Read I/REPORT and its source binding; inspected the
pertinent native read_header/cache/Drop/flush/flag/error excerpts.

Authentication was independently performed with standard filesystem/JSON/SHA256
operations, not P/reproduce.py, audit.py, or imported historical helper code.
All315 payloads match exact sizes and SHA256, total143319910 bytes. Observed
manifest SHA256 is exactly
`474753727618543b4bcf102250557aa4ca302118c21968306c1245dc5f68ff35`.
The317-file inventory is precisely315 payloads plus MANIFEST/SEAL, with no extra
files, symlinks, writable entries, or executable files. Integrity was NOT treated
as semantic approval.

An independently authored, in-memory lexical comparison bounded each relevant
impl/member, ignored whitespace/comments, erased explicit proof blocks, proof-only
hide calls and ghost locals, and normalized named result binders/visibility and
an insignificant trailing parameter comma. No old checker/helper was executed or
imported. All36 selected executable bodies and normalized signatures matched
recovered native source. The three NEW native bodies were additionally read
side by side, including all branches and byte ranges. Five struct field/type/order
records match after visibility normalization. Twenty-six common scalar constant
initializers compare identically; magic bytes, version1/2/3 constants, MAX_PAGE_INDEX
and MAX_REGIONS were also directly checked against their native declarations.

This is lexical evidence, not native nominal identity, AST/instruction equivalence,
compiler correctness or full dependency closure. Read-only extraction/lookup and
JSON-order comparison issues were corrected in memory, earning ZERO proof credit.
Final receipt comparisons use structured deep equality, not key order.

### Principal021 hashes

Every path below is relative to P; all digests are SHA256.

| Input | Exact digest |
| --- | --- |
| AUTHORIZATION.md | `72bd7f64a71a23a5319daac29df96db7d5b343a605125186a1666a09c823b737` |
| REPORT.md | `ab7d3c4b4fb407a9d34a6339b2a2a6aa0f153a26a7bdfdeafc28498b915d6e6e` |
| SUMMARY.json | `cef3d1049f50159bf05183e28d2284346aef52ca9317de213b361607bdb6fff6` |
| REPRODUCTION.md | `e61527e4cb5a359cff6a6aed16d5184f9cfd2466f632fc0fd8284a4dde208528` |
| SEAL.json | `8b9604be4b6c0faa9bdf80970ab64f12cda3892854a9e5e6d2bf1b72a0abfd79` |
| application-ledger.json | `5b0cd5737fcf9df8987d0c76b2744728c2476d871811ece3c2f45042563a41a4` |
| launches.jsonl | `fb0b74d85368e44033b8711dcbe1c8febfbedbef05c0ff016bc96672facbe302` |
| runtime-binding.json | `3854da46a7f383950ff53d180b1f97b92aacd8c5284083a38a5782a718069654` |
| source-correspondence.json | `0041fedbc1ca3b5546c6138022ee91c2c287bce5221f28d787f10392d9900c9f` |
| declaration-ledger.json | `5450520ca50da2001d7be622fc6ef8f3219002a24ced3bd7229c904ee3dcc394` |
| import-retention-ledger.json | `8b1fee20da227e84071430c031ba4ddab70c842280021f98d4d5a34fed8cf67f` |
| preservation-initial.json AND preservation-final.json | `aadb1140c614d82e038af2d98e154be3fe3b1f1fb8e63bb4a7a84c0b754d3f2c` |
| reencoding-001.rs | `772e12e6d6da508e00f2ac26129c7a65d2f80f9144cadadf0959a5518459b759` |
| roundtrip-005.rs | `c93f31d41d6affafa39eea680fef7d1771c24b73bd83abc4eff503b8a513466d` |
| interrupted-005.rs | `643a8874d55bbe45c33c436e820c7b145f659fdf521603367f896162c80e5c07` |
| clients-003.rs | `dc06bb89bc18837ef6318893f9e052a7a8d50a437b9bcf222b8aad9c336ad00c` |
| fixtures-002.rs | `feb761d0367ab70ba45df23072c8b19e383bfd79da928b87971ee67e9c2576fa` |
| controls-positive-002.rs | `129ff04cae731861fac15e89b438a297e187e235dedcb032ef8d35352d362b5a` |

Other315-payload inputs, including all raw receipts, draft and control sources,
are exactly enumerated/hash-bound by the authenticated MANIFEST. None was run.

### Recovered native inputs

Paths relative to N. I/source-inputs binds the first twelve; all twelve were
independently rehashed and matched. Its021 seal reference also matches P/SEAL.
Hashing a full file does not mean all of its code was semantically reviewed.

| Input | Exact digest |
| --- | --- |
| db.rs | `65c3107446039a3dacba03dae0b2a21c88109562ba81d7cadda0a8ea8c7a6169` |
| sync.rs | `933159b9b6b93500df1d517f961fe8c07c3fc07ed54aa2527a0f95a4ffea5016` |
| transactions.rs | `02b383d6d984a14dfc4827ec04241d78637591f91ff754b42f9671b305d18bde` |
| tree_store/page_store/base.rs | `ce2ae1188b5bad2babe2f7b7df19d82adc2b8baff6b41b957e978437686938c4` |
| tree_store/page_store/cached_file.rs | `cb901d44531b23f87d1bd31f72a24f1dc69308b5417db2167b7508921ffda486` |
| tree_store/page_store/file_backend/fallback.rs | `95df75e618e6817b169b716248414fe90843706396755ded380a33411336d5a7` |
| tree_store/page_store/file_backend/mod.rs | `bb7a2e1ac88d3c2cbf9045266c7122ce7768dea456aa2ceeb89d39d097daf441` |
| tree_store/page_store/file_backend/optimized.rs | `c1b8ad3c235be3d778f54861f7e21c0f4b0c8114989172459c7ce7f7db18c182` |
| tree_store/page_store/file_backend/range_lock.rs | `e9ee9a0278eb37302ad9d1c22c0d0cee7f1fb7f9e04cfad5dec1a0a3516dd746` |
| tree_store/page_store/header.rs | `35863472a48f9e0eb65a0f9ba0b2517a888a881478476aaafbcbfc77842850f9` |
| tree_store/page_store/lru_cache.rs | `a5896391195b2da0b94b14d1641fb2916528fb520406214162544e8253dcdcd4` |
| tree_store/page_store/page_manager.rs | `b062608a527c3f53aa505af870f2b803da3fa7b095169194c681cc6441cb558a` |
| transaction_tracker.rs | `423252fc12806ff37aed5a61ab9f0217bf35a2c2724dc40f9930dc94dbf3519e` |
| error.rs | `b7d0aaa4074d7746ce9af95188deb9472a08578ebf3214a9aa20f8f008e32266` |
| tree_store/page_store/layout.rs | `0b559f9a739c6960bfdc0fd84c03f3bd8e083a824a74f9ea41b162f1fb99d670` |

### Minimal inherited evidence, not a re-review of prior lanes

| Input | Exact digest |
| --- | --- |
| B/composed-005.rs | `b84913d4f1283a727641f009250376c92434f73320052439c11f1f7a25c2776f` |
| B/fixtures-005.rs | `24ece3f9bf2ab2b887fadd2b61358e5a62e6a9907059bc4ba86eec2aee369d90` |
| R/REPORT.md | `3c698d37269dc29d7c5430b8d085bea23b78269fa774e40d6ef2bedb376cbea3` |
| R/FINDINGS.md | `85a2d6c50d7608aae9ea89ef567fc16420f3973f24ae1c4db46f3a87d5a1a49c` |
| R/VERDICT.json | `16a1c8ed311dfc7465d481863c96e13d6c406b924048b49b7dee2eea37ce1b97` |
| R/SEAL.json | `5e1642d9f08058a3a566aff5e9dbf0f04ca5549fc1bb58b93e9bedecb3f7ae2c` |
| R/MANIFEST.json | `caa7a814fd7f32c7e6e942a7ab5110933e2926810ccaf691221ef1da4808e11c` |
| I/REPORT.md | `2de29c05efe7c88061e0cbf5c269035383fe11ddd6b16a34931690375bfe21a6` |
| I/source-inputs.json | `ae1a01cfed4013038bca9eab85b2b20b537aa7efef69360da5aa1190117cb530` |
| /private/tmp/verus-crash-consistency94-20261105/native-header-hash-linkage-013/dispatch-001.rs | `e14bbc89e701dc4417bac6a975fef0f91d96753975982f7f0ca87b4f7d998897` |
| /private/tmp/campaign94-parser016-lf6g2rnI/BOOL-PROVENANCE.json | `2f3487fa7ca39b6c757769865866caf791a2c14b01b3caa7deb0ef450de5d7e3` |
| /private/tmp/campaign94-parser016-review-Z6MMgRJr/FINDINGS.md | `5ac13ac06eefa29425dde0840c0488041171b1d6ba266eefad8ff2a1bdb26fdd` |

R's selected judgments match its seal-bound manifest. Its reported20 verifier
applications were not reproduced. Its bounded parser/finalize judgment does not
approve021 or replace selector008 review.021's020 review278-payload and
constructor019C648-payload authentications were LATE, not retroactive initial
bindings. The constructor was not semantically reviewed here. Historical bool
source-availability statements do not negate later library-source acquisition.

## Source and theorem assessment

### Three new bodies, shared types and explicit projections

- `reencoding-001.rs:1507`: `TransactionId::raw_id(self) -> u64` is exactly
  `self.0`; the named proof result is the only return-syntax annotation.
- `:1515`: `TransactionHeader::to_bytes(&self) -> [u8;128]` returns corrupt bytes
  FIRST and verbatim, otherwise asserts version3, zero-initializes128 bytes,
  emits optional roots and the actual raw transaction ID, hashes exactly the
  first112 bytes through `hashdispatch013::xxh3_checksum`, and writes a16-byte LE
  checksum. Root ranges8..40 and40..72, ID104..112 and checksum112..128 match.
- `:1548`: `DatabaseHeader::to_bytes(&self,bool) -> [u8;320]` optionally writes
  magic, emits primary and flags in byte9, five LE u32 geometry/count fields at
 12..32, and both complete slots at64..192 and192..320. The native primary
  `try_into().unwrap()`, boolean branches and constant expressions are retained.

The encoder's executable preconditions are explicit: a slot either has retained
corrupt bytes or version3; a database header has primary<2 and serializable slots.
They are not a totality theorem for arbitrary manually forged native fields.
The corrupt-byte return happens before the native version assertion; the broader
encoder contract correctly permits that branch even when version3 is not known.

After exactly the disclosed first-line documentation and `assert_seqs_equal`
import changes, the entire020 source prefix is byte-identical; the added portion
contains serialization specs and these three bodies. Original009 finalization
and arithmetic remain unchanged, not candidate010 repairs. The six fixtures
are byte-identical to020 after only the crate-import rebind.

The selected36-body inventory is6 RegionLayout,9 DatabaseLayout,8 DatabaseHeader,
5 UnrepairedDatabaseHeader,2 TransactionHeader,2 TransactionId,3 free read/flag
helpers and1 StorageError conversion. This is a selected local dependency
composition with imported TCB, not the whole native crate. In particular, the
six selector members are carried as inherited conditional dependencies, not
independently accepted by this review.

Projected records preserve field order/types but widen visibility. TransactionId
is a local u64 newtype; Hash/Debug and other unneeded native derive/trait surface
are not thereby certified. DatabaseError's variant shapes match; StorageError
is explicitly reduced to `Corrupted(String)`, omitting native Io/PreviousIo/
DatabaseClosed/Unsupported/LockPoisoned/etc. Error ABI, exact formatted text,
allocation/unwind behavior and library bodies are not established. Generated
Clone/comparison and StructuralEq/compiler machinery remain separately classified.

### Roundtrip domain, checksums, canonicality and error completeness

`roundtrip-005.rs:9` makes the strong corrupt-slot condition visible:
slot.version=3 AND raw[0]=3 AND `parsed_slot(raw,slot,true)`. That last relation
fixes decoded roots/ID and an actual checksum mismatch. `parsed_slot` alone
omits raw[0]'s version linkage, so it would be insufficient. The outer parser
does check raw versions and the domain-establishment lemmas use that fact.
No raw version/checksum premise is smuggled in as an axiom.

Normal slots require canonical page roots. Encoding supplies version3, exact
root encoding, zeroed unused bytes, actual ID and a freshly computed checksum.
The LE roundtrip lemmas and canonical-page word roundtrip connect those bytes
to parsing. No assertion of hash injectivity is needed for this equal-input
roundtrip; undetected real corruption and tree/data-page validity remain outside.
Corrupt slots, by contrast, preserve ALL128 raw bytes, including checksum/padding,
and decoded-field agreement is an explicit premise. This distinction matters.

`header_roundtrip_domain` fixes page_size4096, primary<2, bounded immutable
geometry and both slot domains. Clean stored counts need valid geometry;
dirty stored counts are not trusted or bounded before reconstruction. The
proof establishes full local header equality using slot uniqueness and inherited
array extensionality; it is not a byte-identity claim for all accepted raw inputs.

`recover_encode_finalize` requires only data.len()>=320 and takes arbitrary u64
file_len. It actually parses/finalizes, encodes, reparses and finalizes again.
Its success equivalence uses the actual parser/finalizer acceptance relation;
on success it preserves the first observed clean boolean and proves the second
finalization clean=true and header equality. The first boolean is NOT assumed
true. The successful finalization relation establishes the needed roundtrip
domain; successful recovery is not a new precondition of this generic routine.

Inherited parser error precedence is geometry first, then slot0 version, then
slot1 version: versions1/2 give UpgradeRequired, other invalid versions project
to Corrupted. Checksum mismatch on version3 is a retained corruption flag,
not automatically an outer parse error. Finalize rejects invalid/truncated/
nonrepresentable lengths and policy-forbidden corruption, with Corrupted error
projection, not a new exact-message theorem. Err is propagated without pretending
there was a native write or rollback. The interruption helper's unreachable
branches are justified only under its SUCCESS premise; they do not cover native
I/O failures. Strings/allocation/panic effects remain inherited/outer boundaries.

### Interrupted writeback and arbitrary finite induction

`interrupted-005.rs:9` permits each of320 bytes to independently equal its old or
target byte. That overapproximates prefix writes, but byte9 is a WHOLE old/new
byte, not arbitrary torn bits. Geometry/slots are identical across the two
encoded endpoints, while24..32 may contain arbitrarily torn count words.
Because recovery_required stays true, those words are reconstructed from the
same accepted file_len rather than used as trusted stored layout.

The proof uses the original successful finalization to establish the target's
domain and selection stability. The parsed primary is either the old or target
primary. Conditional on inherited policies: a two-phase image keeps its primary
and rejects a corrupt primary; a one-phase image selects the valid preferred
slot, including the strictly-newer-ID case, with ties stable. Reparse corruption
orientation follows the parsed primary bit. The target is recovered whether
the bit is old or already updated; no new slot contents are invented.

`repeated_writeback_images` at:22 is a genuine unbounded-finite SOURCE induction,
not a finite enumeration or a fixed retry cap. It requires a nonempty finite
Seq, initial image equal to the old encoding, every successor byte chosen from
the preceding image or the SAME target, and 0<=index<len. Base index0 is the
initial image. At index>0, recursion on index-1 decreases a nonnegative integer;
the explicit step bound instantiates the transition, and old-or-target membership
is closed transitively byte by byte. The final trigger/bound correction changes
proof instantiation, not the algorithm or transition premise.

This lemma proves an image-membership invariant. Combining it with the executable
interruption helper recovers the same target at each finite index under that
helper's domain. It is not itself an executable native retry loop, liveness under
infinite crashes, an eventual successful flush, a changing-target/length theorem,
or a mixed-mode history theorem. Withdrawal034's rejection at the transition
assertion corroborates dependence on the premise; it is not a fresh proof here.

Both flags `primary_unflushed` and `recovery_required` are preserved by
`same_recovery_payload`. They are never proved cleared. The helper returns the
actual torn-image finalization boolean but specifies only the recovered header,
its exact length and stable selection, not that this boolean must always be true.

### Ordinary imported client and satisfiability

`clients-003.rs:9` is an ordinary executable rlib/metadata consumer, not an opaque
proof-only witness. For symbolic cut<=320 it calls the native old encoder and
actual parser/finalizer; propagates actual Err; on success calls the target
encoder, performs the real slice prefix copy, and invokes the actual interrupted
recovery composition. Cut0 and cut320 are included by the same bounds, not
unexecuted finite test cases claimed as runtime measurements.

It returns `(recovered, clean)` using clean obtained from the FIRST actual
`parse_and_finalize`, not an existentially chosen boolean. The torn retry's
boolean is discarded; do not confuse this interface with a guarantee about
that retry's clean status. The source change from clients-002 strengthens and
exposes the observed witness rather than weakening the final claim.

`native_torn_prefix_witness` constructs concrete local headers with h=0,c=1,
p=4096,len=8192, dirty=true, IDs5/9, primary0 and absent roots. The two_phase
argument remains symbolic. It calls the imported client for every symbolic cut
and checks the conditional policy result: primary0 for two-phase, primary1 for
one-phase, retaining both IDs. No native database constructor or disk is run.

The inherited fixtures construct actual320-byte arrays, compute actual fixed112
header checksums, and flip one checksum bit for intentional corruption. Thus
the recorded checks do not depend on an impossible assumed checksum equation.
Their length cases include8191/8193 rejection and12288 enlargement. Five021
positive counterparts exercise mode choice, corrupt-slot retention, arbitrary
dirty counts and two-phase primary-corruption error. These sources and retained
complete positives support satisfiability under the inherited TCB, not a
model-consistency or native initialization theorem independently proved here.

## Raw evidence and conservative accounting

Independently compared all34 invocation files to the34 launch records, exact
source/receipt/invocation/stdout/stderr hashes to the ledger/manifest, raw stdout
`verification-results` and `verus-infer.verification_scope` to receipt objects,
and every captured imported input to an authenticated retained-byte entry.
All390 retained-import records have at least one matching retained exact-byte
path, including superseded exports. There were no unresolved binding differences.
All recorded invocations use --no-cheating, no function/module scope filters,
the bound aarch641.99.1 sysroot/runtime and one verifier thread. These are audits
of AUTHOR process records, not executions by this reviewer or kernel attestation.

### Complete positives:14 historical applications, not14 final source files

| Scope | Debug OFF attempt / result | Debug ON attempt / result | Checked exec / proof bodies |
| --- | --- | --- | --- |
| Final provider reencoding-001 | 001:111/0 | 024:111/0 | 43 / 9 |
| Final roundtrip-005 | 006:13/0 | 025:13/0 | 3 / 9 |
| Final interrupted-005 | 013:5/0 | 026:5/0 | 1 / 4 |
| Inherited fixtures-002 rebound | 010:16/0 | 027:16/0 | 6 / 0 |
| Imported clients-003 | 017:3/0 | 028:3/0 | 2 / 0 |
| Five positive counterparts | 018:15/0 | 029:15/0 | 5 / 0 |
| Earlier roundtrip-004 | 005:8/0 | not a final two-profile claim | 2 / 5 |
| Earlier interrupted-002 | 008:4/0 | not a final two-profile claim | 1 / 3 |

All six final source hashes are identical between their OFF/ON input copies.
Provider43 exec =36 selected native +3 generated Clone +3 generated comparisons
+1 inherited authored020 composition; its9 proofs are inherited. Query totals
are NOT counts of certified native bodies. Outside/generated ignored trait
surface and the underlying standard machinery are not silently added to closure.

### Nine intended rejection-only applications

| Attempt | Exact changed claim/member | Substantive diagnostic location | Tally |
| --- | --- | --- | --- |
| 019 | one-phase falsely claims primary0 instead of1 | input.rs:36 assertion | 2 verified /1 failed function |
| 020 | two-phase falsely claims primary1 instead of0 | :36 assertion | 2/1 |
| 021 | retained corrupt slot falsely claimed None | :36 assertion | 2/1 |
| 022 | reconstructed full_regions falsely claimed u32::MAX | :39 assertion | 2/1 |
| 023 | corrupt two-phase primary falsely claimed Ok | :26 assertion | 2/1 |
| 031 | omit two-phase flag encoding | :1585 assertion, late reassessment | 110/1 |
| 032 | force primary byte to0 | :1585 assertion | 110/1 |
| 033 | force checksum-corrupted=false | :170 postcondition | 110/1 |
| 034 | withdraw all rewrite-transition premises | :34 transition assertion | 4/1 |

Five isolated claim sources differ from their matching successful function by
exactly ONE claim; signatures and other body tokens agree. Three clean body
controls and030 each have the advertised one-edit pair;034 only removes the
quantified transition premise. Every credited rejection has exactly one
substantive error header plus its abort summary, located in the intended claim/
affected member, with no hidden resource/frontend/coded error. These negatives
were debug OFF; there is no claimed two-profile negative suite. Failed-function
counts are not diagnostic counts. All negative scopes are incomplete and earn
ZERO complete-positive-body credit.

### Eleven ZERO-credit applications

002 resource exhaustion;003 full-slice precondition;004 array/XOR postcondition
obligations;007 incomplete encoded-slot facts;009 unresolved old crate import
after a failed private-copy patch;011 quantifier trigger frontend failure;
012 incomplete induction instantiation;014 two incomplete client functions;
015 existential-only client postcondition;016 missing layout-to-count projection;
030 resource-limited raw-corrupt-slot mutation. All original inputs/outputs are
retained, including multiple diagnostics within a failed function. None is a
complete positive, clean negative, or inferred database/compiler defect. Auxiliary
patch/listing/reader failures are not additional verifier applications.

The tally remains **34/40 =14 complete positives +9 rejection-only +11 ZERO**;
unused6 slots do not authorize any replay. Exactly24 distinct NEW AUTHORED
exec/proof identities appear across all drafts/final/control source versions:
11 exec +13 proof. An independent declaration scan agrees with the ledger.
Breakdown:12 roundtrip,5 interruption,2 clients,5 semantic counterparts. The
three newly carried native bodies are classified as native bodies, not new
authored helper identities. Spec definitions are not charged exec/proof bodies;
six unchanged020 fixtures and the shared provider's inherited declarations are
not silently counted as new. The24-identity allowance is EXHAUSTED and unchanged.

### Final artifact/import binding and both profiles

The final published `.vir`/`.rlib` pairs in P/artifacts exactly match successful
per-attempt001,006,013,010,017 outputs respectively. No failed draft artifact is
substituted for the final source. Seven historical --compile application exports
are recorded (001,005,006,008,010,013,017), distinct from compiler/vstd/Cargo/
harness builds. Debug024–029 verify the final sources and export metadata but
do NOT build six new debug rlibs; their imports are the explicitly captured
existing release-produced artifacts. This is two-profile proof evidence, not
native execution or separately rebuilt whole-debug-library closure.

Exact retained metadata digests:

| Source / OFF attempt | OFF `.vir` SHA256 | ON attempt / `.vir` SHA256 |
| --- | --- | --- |
| provider001 | `06ad69cf1e0779c47c4f9fb7883498b6bcc575d97cde8891f2cd3e70ba5d67c6` | 024 `fe767c8b8a9add952d4b65de8dea5ca1bf6074a7a46932865aa7cc4a1acdaf66` |
| roundtrip006 | `f14bc273af279407344afbf4fd3b1d95dd7d9ce682722b5b946bb4b5fd073b0d` | 025 `c701a1ff065de596e41b8726b458755d2c5463acc27a820b1988482f6375c3f3` |
| interruption013 | `dc0d7d86811956f332cf497346f00100a8d89ca3248037de7e115afc7f89ba9c` | 026 `0bc7f1cba8d42914c3a3053e67edac487ba43ae92c630346321a836cb14c1212` |
| fixtures010 | `f8daee62a9fdf60c351ef2ed4ad13e3f3ed7bcd337ce5179cb3e7ac321b4067f` | 027 `2ce3a331ae6944cacd7d5fe8246d12dd7f0771726d151b97b56697f69950e68f` |
| clients017 | `a7ef972dcabcef2818dc9ffd0e631c2beece312ceb6323db3f9a651aa84978a5` | 028 `0a769723bee21fb510422771bf1da9bd6f7be72adf25b11d4353490f7ff57377` |
| positive counterparts018 | `cd4e2e18b6e614cbaf0696fbc578744bcb3ab97d0901a5789feeb94b67522e8b` | 029 `de3311696bd84a3c51cf21121f3a5fe00aa0663bdc88455dacc4f450d90449ad` |

The final five published rlibs are respectively:

- `librecovery_reencoding021.rlib`: `9e4a09990a8f4732b80881cb81f95ac7206319b1f833118b70c20306d62ae0b6`
- `librecovery_roundtrip021.rlib`: `451f71ebda52facc9f480a8886de3848560dfd76985c2be91b4982784de03ad5`
- `libinterrupted_recovery021.rlib`: `6d644da9caa113faea4c9d5c608c59fef93a91b4c48a3e18a5e3e3e689a41657`
- `libreencoding_fixtures021.rlib`: `ead7e0b8a5578ff95bc8af5b3e355727808ff1d4fd1ced31676080102775f8b1`
- `librecovery_clients021.rlib`: `fe5931b27d6cac1037d4d103d762eb4e92100dc58e7d67efd128731549b59060`

Inherited artifact pairs (metadata / executable library), exactly hash-bound in
P/runtime-binding and the invocation input maps:

| Dependency | `.vir` SHA256 | `.rlib` SHA256 |
| --- | --- | --- |
| pagecodec94 | `49cf74f0927d2f2e391f4f800ff67839a1fd3c26f05ecc2e1079bd84bcc327b4` | `c249ff5fcb87385bfe4601cba792ab67d6173d949425cf605b9caedf4750929b` |
| rootcodec94 | `38266985dee653bc30c6e0f7736f30959db32a3296e038ed16b3e172f2c52625` | `6f53a4ce59fc681611b0776e1ac1aa4e900331973374637f9584a1cf583715c7` |
| hash94 | `17f7641fb3c60ec1dbdaa46a638b5c14adbff3dd0dafca2e3cbb4c28a293aa5f` | `a61f3437ea52f0ddcfb8d6c5156c6e16742505a7bb68000581ab7f882cf7cc9d` |
| hashdispatch013 | `bf0ad986772c1b991a1d8fbf6f92bad97141f0310251a92fa0b44103c1a79da0` | `47066c0459b386e3a255f4a74aec5a13d4d236d284beb9e96b9fbf0c57d4d508` |

The actual fixed112 wrapper013 is imported; neither the scalar-only012 annotation
adapter nor candidate010 arithmetic artifact is used. Boundary classification
remains `EXPLICIT_INHERITED_STD_BOOL_TCB_NOT_RUST_CORE_PROOF`. Hash equality of
metadata/binary pairs supports retained binding, not independent binary build
attestation or semantic approval of every imported theorem.

##022 native frontier — source checked, no oracle assumed

Native page_manager.rs:868 reads320 bytes directly and calls the outer parser.
At:879 the ordinary path releases its initial shared header hold, obtains physical
length, computes needs_recovery from layout/dirty OR primary_unflushed, rejects
read-only repair, and calls finalize. Repair reacquires exclusive protection.
At the primary_unflushed branch, flush must succeed BEFORE clearing that flag;
the copied header is then flushed and length is queried again for an assertion.
Shared-reader behavior is a separate branch, not covered by021's4096 composition.

cached_file.rs:24/40 shows the REAL WritablePage fields and Drop. It returns the
same data Arc and offset through the same cache Arc after locking. `mem_mut`
uses `Arc::get_mut(...).unwrap()`. `return_value` at:91 requires a reserved entry
and asserts it was None. `flush_write_buffer` at:490 unwraps entries, performs
backend writes, moves buffers to the read cache and clears stripes. Correct
ownership/uniqueness, loan end, generated destruction, panic/poison handling and
absence of live reservations at flush are not consequences of a byte-copy proof.

At:596 flush = flush_write_buffer then sync_data; at:606 sync_file performs only
sync_data; at:612 write_barrier changes visibility bookkeeping, not persistence.
Direct write invalidates/cancels cached ranges before backend write; direct read
allocates and calls backend read. raw_file_len delegates to the backend length.
CheckedBackend latches len/read/write/set_len/sync_data failures and later
distinguishes PreviousIo/DatabaseClosed. A failed operation or sync may have
partial/already-persistent effects. A later length query can fail after a flush.

page_manager.rs:1617 write_header chooses direct I/O for shared writable modes
and the actual cached WritablePage path otherwise. At:1638 clear_recovery_required
changes in-memory flag state BEFORE write_header/flush can error. Those effects
require outcome-complete caller/state reasoning; Err cannot simply mean rollback.
022 also identifies commit publication, mode/primary shared-byte atomicity and
post-barrier bookkeeping. None is discharged by the byte-frame lemma. Primitive
storage/lock/crash/error contracts remain a proposed outer boundary subject to
review, not installed axioms or accepted native I/O/resource support here.

## Preservation, activity limits and handoff

Entry and exit digest construction: SHA256 over JSON encoding of sorted
`[absolute_path, SHA256(file_bytes)]` entries, sorted with the Node localeCompare
comparator used in this review. These are read-only endpoint observations, not
claims about every transient filesystem event.

| Observed set | Files | Entry SHA256 | Exit SHA256 |
| --- | --- | --- | --- |
| Frozen P payloads + MANIFEST/SEAL | 317 | `90220b25f19ea093909cfc15a410df6566217e0283e6dc2481a738220cd70252` | SAME |
| Parent2347 guard excluding its two policy files SCOPE/STATUS | 2345 | `1fc7f75b481cfdfadba28b18a82f09e5a5c5e9f9943049d6a553dcee9f7fa207` | SAME |
| Combined stable primary source/package subset | 2662 | `d426864a31e2e3bdebf8e0197da8b106047128045f6c83cce2b9d5fe3adc8456` | SAME |
| Additional N/B/R/I/013/016 inputs listed above, observed when inspected | 27 | `3d1e42c8341f50030833c489bd354e64888fa88a2eb19b51aca751e691c3b492` | SAME |
| Complete initial set INCLUDING all four live D documents | 2666 | `ffa71c163d41a693cf6ce97e16fffa2ff2c6e3a457b5a6c72983f8cea3559f11` | `13e344a61f2eeea68520371ccc199c7e8162341d468ba127e95943bb71bc75e6` |

The policy files were NOT quietly dropped from the initial preservation claim:

| D file | Entry SHA256 | Exit SHA256 |
| --- | --- | --- |
| GOAL.md | `74caefc4c6ea3049e6b8857d01fe1eb4abb834bd1029ef1c7db24b9d903ee263` | `73712a31ff2729640407c7127e790b0518c376f96250dcb01fa5fd5474bdfeb9` |
| SCOPE.md | `b2b4bf90ccb3d6aaeecf1b24a29f1e1e1fc7bd84a9f910ced13e45051d835285` | SAME |
| STATUS.json | `5d91c4e668fe49091107d0387e326ccf8cd3bf6c847acad8a06e52b7ce9419c9` | `da4da330eb11c6bf43f6050c7c03b29c25ce2096f91f5650081f307d112d4b00` |
| TAKEOVER.md | `aa25223ba8b9722442607eb349700e9d469fb5037e5c6be8c635c36ba8185a07` | `7dbe13ab43e9fcbf432321fa8bdb44a721745d2a1e0b3ac7a2c9ee39df1e13d3` |

The full historical2347-path parent guard includes SCOPE and STATUS; therefore
2346/2347 of that exact list match, with only live STATUS different. Frozen
preservation-initial/final equality is the AUTHOR's historical record, not a new
assertion that today's parent equals every old snapshot. This review did not
freshly recertify every historical immutable alias, sysroot file, owned Git state,
shared ref, or the objectstore. Those qualifications remain inherited.021's
shared-ref additions are explicitly unattributed and NOT_CERTIFIED; no refs were
reset, imported or certified by this review.

Disk at initial `df -k`:131637900 KiB available =134797209600 bytes (>125GiB).
Final check:134631841792 bytes (>125GiB).
Both are strictly above16GiB=17179869184 bytes; only this bounded report is written.
No toolchain copies or space-consuming proof/build artifacts were created.

Activity attributable to this review:

| Activity | Count / disposition |
| --- | --- |
| Compiler/verifier/solver/native/harness/Cargo/vstd executions or builds | 0 |
| Verifier applications, application compile/export, optional21 replay,030 retries | 0 |
| Old reproduce/audit/launch/checker helper executions or imports, even zero-launch | 0 |
| Parent/native/proof/compiler/library source edits | 0 |
| New axioms, trust/schema installs, export/admission changes, allowance resets | 0 |
| Git mutation commands, network operations, toolchain copies | 0 |
| Delegation, subdelegation, follow-on tasks | 0 |
| Authorized output | This NEW REPORT.md only, in a fresh private directory |

No author log is fresh proof reproduction. The frozen seal's historical
`independent_review: REQUIRED_NOT_STARTED` remains unedited. This NEW report
supplies a source/evidence judgment, not trust/adoption.

| Final obligation | One-phase | Two-phase | Mixed histories |
| --- | --- | --- | --- |
| Crash atomicity | OPEN | OPEN | OPEN |
| Acknowledged durability | OPEN | OPEN | OPEN |
| Preservation of recoverable state | OPEN | OPEN | OPEN |
| Recovery including interrupted recovery | OPEN | OPEN | OPEN |
| Error correctness | OPEN | OPEN | OPEN |
| Logical key/value refinement | OPEN | OPEN | OPEN |

**18 OPEN;0 closed. New native resource proof credit: ZERO.** This handoff changes
only the availability of an independent bounded021 source/evidence assessment.
It does not settle selector008, authorize replay/implementation/builds, adopt
library laws, or change S0/ordinary Infer/full-scope/import/export barriers.
Return this report to the parent and stop. No follow-on work is initiated.
