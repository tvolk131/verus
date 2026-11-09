# Campaign94 native next-edge decision

## Verdict: NO-GO under current standing authority

No inspected original-body/interface combination supplies a genuinely native
execution-to-model next edge on the acknowledged-durability path within the
current exclusions. Do not launch another header consumer, model/spine theorem,
type-only backend probe, primitive fixture, or implementation on this handoff.
All eighteen property/history cells remain **OPEN**.

**One separate decision:** explicitly decide whether to reverse the existing
refusal of the **local `WritablePage`/`Drop`/lifecycle correspondence work**,
limited in objective to binding actual `PageManager::write_header` execution to
the pending header consumed by the flush product. Without an affirmative,
separately recorded scope reversal, stop native implementation on this edge.
This is the previously refused work itself, not a permissible task renamed
“header admission,” “Arc bridge,” or “buffer correspondence.” No such reversal
is inferred or exercised here. A reversal would be **implementation permission
only**, not adoption of primitive/resource/backend laws, native-producer
acceptance, proof validity, or cell closure. No build, diagnostic107-D1, source
integration, or exhausted/refused allowance is included in that decision.

## The specific missing edge

The reusable header result already exists. In original-method provider I23,
`DatabaseHeader::write_secondary_slot` (569), `swap_primary_slot` (560), and
`to_bytes` (1548) have contracts for exact slot replacement, selector change,
and `database_bytes`. H's authored `stage_two_phase` (I18:43) imports
`recovery_reencoding021::*`, `recovery_roundtrip021::*`, and
`rootcodec94::BtreeHeader`, and calls those methods (69,73,75,84). Its first
encoding preserves OLD mode; its final encoding selects the new ID and roots.
This is useful checked header algebra, not a new native cache execution proof.

The **next missing statement** is that the actual local write's resulting
resource state represents those encoded header bytes in the pending buffer
that actual flushing will consume. Neither an encoded array nor a slice-copy
postcondition establishes that statement:

- I08 `page_manager.rs:1617` branches in `write_header`. The local call at
  1624 obtains `storage.write(0, DB_HEADER_SIZE, true)?`, calls `mem_mut`, and
  copies `header.to_bytes(true)` into the returned temporary. It does not
  directly insert a byte sequence into the proof model's pending collection.
- I09 `cached_file.rs:881` `PagedCachedFile::write` acquires a striped lock,
  removes/reuses read-cache data, takes a write-cache value, performs possible
  eviction/writeback, allocates or obtains an `Arc<[u8]>`, and returns
  `WritablePage` (974). Its actual success/error and resource effects matter.
- `WritablePage` (24) contains `Arc<Mutex<LRUWriteCache>>`, offset, and
  `Arc<[u8]>`. `mem_mut` (35) uses `Arc::get_mut(...).unwrap()`. At the end of
  the write expression the original `Drop::drop` (41) locks the buffer and
  calls `return_value(offset, data.clone())`. `LRUWriteCache::return_value`
  (91) replaces the taken `Option` value through the underlying LRU cache;
  `take_value` (95) leaves that entry unavailable until return. This return is
  consequential: it makes the copied bytes available to later iteration.
- The original cache imports are `crate::db::StorageBackend`,
  `crate::sync::{Mutex, MutexGuard, RwLock}`,
  `crate::tree_store::page_store::lru_cache::LRUCache`,
  `crate::{CacheStats, Result, StorageError}`, `alloc::sync::Arc`, containers,
  and `core::sync::atomic::{AtomicBool, AtomicUsize, Ordering}`. The inspected
  header/product imports provide no resource contract connecting that actual
  lock/ownership/temporary-drop chain to the modeled pending bytes.

Consequently this edge inherently crosses the expressly refused
`WritablePage`/`Drop`/lifecycle boundary. Merely supplying a caller premise that
the write buffer contains the header, that the temporary was correctly
returned, or that all relevant ownership/frame facts hold would assume the
missing correspondence, not discharge it. Primitive Arc/Option/slice proposals
are not adopted and do not independently authorize this resource work.

## Why the present imports do not provide a different GO

I16 `stage_and_recover` has explicit caller premises for exact first/final
encodings, initial product invariant/control, **model** public Ok, geometry,
and a supplied slice equal to the model's durable prefix. Its proof calls H,
`public_ok_recovers_expected`, and `decode_expected_header`; none calls the
original `PageManager::write_header`, cache `write`, or temporary destructor.
I19's decoder (97) calls the imported parser/finalizer on supplied bytes.
I10's domain (109) retains 4096-byte geometry/canonical slots. Those contracts
do not obtain a native file slice or establish native buffer origin.

I11 `public_ok_composes` (274) proves the final durable header **in its product**
from local mode, product invariants, initial control and model success. I12
`local_mode` (52) explicitly excludes multi-process and shrink-enabled mode.
Therefore taking I08's direct `write_direct` branch (1618) is not an escape
from the refused local-resource edge into this existing product; it changes
the mode/path and still needs an actual backend relation. The source commit
also retains real post-sync operations before its Ok (I08:1873).

Nor is the already attempted backend lane a ready downstream substitute.
I22 is the retained actual frontend diagnostic: unsupported `io::Error` and
`panic::Location`, not a verified backend body. I09 `check_failure` (195),
`sync_data` (242), and `write` (251) use native atomics and dynamic backend calls.
In I21 the native atomic load/store specifications (66,72) impose ordering
preconditions and `no_unwind` but **no stored-value postcondition**; its preamble
explicitly excludes reasoning about stored values. A different permission-
carrying atomic wrapper would not be unchanged native Rust. This observation
is not an assertion that every conceivable checked atomic technique fails.

I14/I15's extracted control/history interface (`Flags`, `Choice`, `Event`,
`history_contract`, `failure_then_arbitrary_suffix`, `continuation_contract`)
requires a separate native adapter for observations, dynamic effects and exact
payloads. Its backend effects are unconstrained tokens, even on Err. Clearing
the two frontend errors alone would establish neither atomic stored-value/
history correspondence nor actual writes, sync durability or crash behavior.
I09:490's fallible stripe writes and later drains, and `flush` at 596 calling
`sync_data` only after that loop, make this a semantic dependency, not a
missing type annotation. No new import or caller premise can be invented here
to turn those conditional contracts into the missing native relation.

## Authority and unchanged status

I01 permits distinct routine implementation/source-connected attempts, not a
reset of prior allowances or an exception to excluded lifecycle/Drop work.
Round111's completed allowance is not used for this investigation. I06 reports
`no_inference`, four authored executable bodies, 59 local proof bodies and zero
new original native executable bodies; I07 reviews the conditional contracts,
not native acceptance. This investigation launches no verifier or application.

No growth049, zero-growth050, mutation030, refused-padding95, refused pipeline,
diagnostic107-D1, compiler/build/Cargo/test/native run, network, Git write,
source edit, producer integration or assumption/trust/admission change occurs.
S0/Infer/full-scope/import/export remain unchanged and fail-closed. The inherited
standard-boolean TCB stays conditional and unadopted. Original-path preservation
remains FAILED, native Git continuity UNAVAILABLE, compiler/source
correspondence UNESTABLISHED. I04 distinguishes recovered-content matching
from historical path continuity and native semantics; no repair is claimed.

## Bounded research accounting

- First real clock: **2026-11-08T06:06:55Z**; absolute fresh-task deadline:
  **2026-11-08T06:14:55Z** (eight minutes, unrelated to round111's allowance).
- Entry `df -k /private/tmp`: **71,558,064 KiB available = 73,275,457,536
  bytes**, strictly above 16 GiB (17,179,869,184 bytes).
- One worker, no subdelegation. Only newly created scratch directory:
  `/private/tmp/campaign94-native-next-20261108T060655-Yh353x`.
- Only authored artifact is this report, written with `apply_patch`; no input
  copy, build output, log file, helper script, archive or evidence mutation.
  Fresh scratch limit is 128 MiB; report limit is 16 KiB.
- Exit input authentication completed **2026-11-08T06:13:34Z**, 399 seconds
  after the first clock. At that boundary disk available was **71,691,568 KiB
  = 73,412,165,632 bytes**, strictly above 16 GiB. The finalization sample at
  06:13:02Z measured **16 KiB** allocated scratch and a 13,752-byte report
  before the last anchor/accounting corrections; even that sample's lower
  free-space observation, 67,196,312 KiB, was strictly above the disk floor.
  These are boundary samples, not a continuous quota/free-space measurement.

## Used primary inputs: read-time and exit identities

The SHA-256 digests below are whole-file integrity identities taken during the
read phase; inspection of sources/reports was targeted to the cited interfaces
and limits. Exit reauthentication is recorded below. Reads, hashing and stat
were sequential observations, **not an atomic or first-read snapshot**. No
claim of compiler provenance, native proof, symlink/Git restoration, or external
trust follows from matching hashes. Historical manifests/results cited inside
these inputs were not recursively replayed or authenticated as entire sets.

Path prefixes (literal expansion, not symlink resolution):

- `E110` = repository `evidence/redb-storage-composition110/`
- `E111` = repository `evidence/redb-header-recovery111/`
- `N` = `/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/recovered-native/redb/src/tree_store/page_store/`
- `R` = `/private/tmp/verus-crash-consistency94-20261105/native-recovery-reencoding-021/`
- `S` = `/private/tmp/redb-header-recovery111-20261108T051418Z/shared/`
- Repository = `/Users/tommyvolk/.t3/worktrees/verus/t3-60978068/`

| ID | Input path | Bytes | Read-phase SHA-256 |
|---|---|---:|---|
| I01 | E110/CONTINUATION-AUTHORIZATION.md | 1937 | eac012366e8d0bca988299489d5dfe0bd7c0f71ac444b254de6c96ff6dea9142 |
| I02 | E111/NATIVE-CLOSURE-LEDGER.md | 5273 | 1745bbdb1add999cfbfdba565b3204b14f80c25db451a0759bd377eff60f6ca2 |
| I03 | E111/NATIVE-LINKAGE-SOURCE.json | 12250 | 11f39386690e339d182da23fbcf0b118dd5fa86cb37f9255634eb2d463377e44 |
| I04 | E111/PRESERVATION-QUALIFICATION.md | 2101 | bc892f6a1767376bc1e90202bce069b7d2f305e5246ca5e7dcb75a8606527fa9 |
| I05 | E111/README.md | 7089 | eead9d1127d645467b35800ee08274468dac396e76648a49f6e8677234855da1 |
| I06 | E111/ROUND-RESULTS.json | 5511 | f2a30d2f1e442bdf63d37b1f58a7827b0fb1e54b251c0ec4875f4fbc4825b790 |
| I07 | E111/INDEPENDENT-REVIEW.md | 8700 | 05ff68c5b4e7eef13cab0785804e05d40da0565e794d1877edb166ca87ff535a |
| I08 | N/page_manager.rs | 157364 | b062608a527c3f53aa505af870f2b803da3fa7b095169194c681cc6441cb558a |
| I09 | N/cached_file.rs | 56801 | cb901d44531b23f87d1bd31f72a24f1dc69308b5417db2167b7508921ffda486 |
| I10 | R/roundtrip-005.rs | 17033 | c93f31d41d6affafa39eea680fef7d1771c24b73bd83abc4eff503b8a513466d |
| I11 | S/composition110.rs | 19396 | 00289ba9a9189b9b24491ef68ae84fa74e75a04d1c19277db4e6bafa26456f5e |
| I12 | S/ack_cfg.rs | 14077 | 447a7fcfa23fcdec6114be0ec4eb27274ed4133efbc7d81fdac896dffd9306a2 |
| I13 | E110/NATIVE-BINDING-FRONTIER.md | 3573 | 713a96270a3356f4fbd085ef0ae907beb8b613c855c866cde7d237eef20eb8f0 |
| I14 | E110/BACKEND-CANDIDATE-REPORT.md | 12880 | 6710a59af0bb79717526cc4c00d490227084d635058dc44fbef26628c95933ce |
| I15 | E110/BACKEND-CANDIDATE-PROJECTION-AUDIT.md | 7867 | 7f7dd0d2fa3d66ffb85169ee61270fb7479742b49b5e80cb870d1caa92437ba6 |
| I16 | E111/COMBINED-combined111.rs | 3602 | d590effe76335c59044da26cdd51486e0aa87e1dd175eff2f88a9fc8de58c673 |
| I17 | S/contract111.rs | 1544 | 839e12109cc0ae0699536cbc7f401dc6d2b7d5f6f44eeccf8c9453bf450277ef |
| I18 | /private/tmp/redb-header-recovery111-H-20261108T052400Z/staging111.rs | 7424 | 0d894c65c6315d1c0de84c6f06a797b9b448d55b9539c542d020502345b6bc31 |
| I19 | /private/tmp/redb-header-recovery111-P-cPSXKcfk/recovery_product111.rs | 7823 | f091f82098b565a863f0f32cf01dd17d46b31ee167fc6d0588a5399a0de928b1 |
| I20 | E110/NATIVE-BINDING-FRONTIER.json | 3284 | 001e55a60ed438c89112cc3e2e87b8029ba937a46e22170c4538032394b8397f |
| I21 | /private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/midpoint-runtime-proposal/source/vstd/std_specs/atomic.rs | 4168 | 146999c62edcd4c9fa032445492408605fe57b6f6dfd2279434e7c44b046d2b5 |
| I22 | /private/tmp/redb-backend110-B-cCGFf2VR/attempt-01/stderr.txt | 1879 | 80d6c867fe64d8b17b57cce43c9c806d57a0be10fbebdf8f7e32a1a9e83ed1b2 |
| I23 | R/reencoding-001.rs | 71313 | 772e12e6d6da508e00f2ac26129c7a65d2f80f9144cadadf0959a5518459b759 |

**Exit identities:** each input I01 through I23 was individually re-read by
`shasum -a 256` and `stat` at 2026-11-08T06:13:34Z, after the last source
inspection (and also at 06:12:21Z). Every exit digest equals its
read-phase digest in the table, with exactly the listed byte length. Thus that
table also specifies each complete exit identity, not just an aggregate seal.
Observed digest mismatches: **0/23**. This is sequential exit authentication,
not a claim that inputs could not change between observations. Initial source
hashes for the five distinct source paths named by I03 match I03's recorded
identities; that limited comparison does not authenticate all historical inputs.

Handoff only. No automatic review chain or follow-on execution.
