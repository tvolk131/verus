# Source-only onephase protocol assessment108

**Verdict: the first native protocol gap is publication-to-storage event refinement, not another selector lemma.** Existing checked facts do not prove that the actual single-process `write_header`/`flush` execution produces the bytes and persistence transitions required by the recovery bridges. A native commit/ack/crash theorem cannot currently be implemented from those facts alone without new checked correspondence engineering. No such implementation, primitive-law adoption, diagnostic107-D1, or native producer is authorized by this assessment. All18 final cells remain OPEN; new proof/acceptance credit ZERO.

## Identity and limits

Parent P=`/Users/tommyvolk/.t3/worktrees/verus/t3-60978068`; C=`/private/tmp/verus-crash-consistency94-20261105`; N=`C/native-leaf-lookup-007/recovered-native/redb/src`; R=`C/native-recovery-selection-008`; E=`C/native-recovery-reencoding-021`; F=`C/native-parser-finalize-020`. Paths below use these aliases.

Started 2026-11-08T03:05:27Z (November7 locally). Before caller analysis, native SHA256/size matched both recovery.json and recovery-initial-blobs.json; Git-blob-content SHA1 also matches. Recorded pin `c82ab865bb362bc4af067d332adc44978ff64813`. Recovered CONTENT ONLY: original-path preservation FAILED, Git UNAVAILABLE, inherited object-store nonmutation NOT_CERTIFIED. No clean checkout/Git continuity/dependency closure certification.

`INPUTS.json` records exact paths/SHA256s/sizes, including 11 native/instruction files authenticated against recovery records. Evidence baseline was captured during the scan, not before the first document read. Native entry authentication preceded analysis; exit totals are in `ACCOUNTING.json`. Checked-source hashes agree with the independent reviews. Historical checking is inspected evidence, never a fresh replay here.

## Actual caller/control chain

1. `N/transactions.rs:1752` commit -> commit_with`:1757` -> commit_inner`:1770` -> helper`:1781`. Quick-repair/multi-process writers force two-phase (`:1785`,`:1792`), excluding them from onephase. Immediate branches at `:1837`. Root preparation starts `:1802`; its producers are NOT proved here.
2. `durable_commit:2061` supplies user/system roots and calls `self.mem.commit` at `:2139` after system checksum finalization `:2135`. `N/page_manager.rs` here means `N/tree_store/page_store/page_manager.rs`.
3. `page_manager.rs:1797`: dirty-page assertion/I/O check`:1809`, optional geometry adjustment`:1814`, clone`:1820`, strict ID assertion`:1825`, secondary mutation`:1831`, FIRST write_header`:1837`. Onephase skips flush`:1841`; swaps primary/sets mode=false and shared=false`:1847`, writes AGAIN`:1859`, flushes`:1861`, installs in-memory header`:1887`, returns Ok`:1891`.
4. Crucial branch: `write_header:1617` uses `write_direct` ONLY in multi-process mode, which the onephase caller excludes. Actual onephase goes through `storage.write(0,320,true)?.mem_mut().copy_from_slice(header.to_bytes(true))` at `:1623`. Two calls are not two established backend writes. They can replace the same buffered header; the abstract trace must account for coalescing/issuance, not label each copy `issue_write`. We do not analyze library destruction here.
5. `N/tree_store/page_store/cached_file.rs:881` obtains the writable buffer; flush_write_buffer`:490` issues cache entries via CheckedBackend::write`:495`; flush`:596` then calls sync_data`:599`. sync_file`:606` does NOT flush application buffers; write_barrier`:612` does NOT persist. These are source observations, not checked effects.
6. Primitive boundary: `N/db.rs:69` StorageBackend, write`:87`, sync_data`:84`, read`:76`, len`:71`. Optimized backend delegates to File::write_all_at/sync_data (`file_backend/optimized.rs:378`,`:373`); fallback uses write_all/sync_data (`fallback.rs:112`). Documentation is not checked correspondence.
7. ACK is outer returned Ok, not flush`:1861`. Bookkeeping/optional postcommit pass remain (`transactions.rs:2147`,`:2167`); durable_commit returns`:2177`, helper`:1867`, wrappers propagate. Postcommit internals are not investigated. Errors after durability are possible; Err does not imply rollback.
8. Recovery `page_manager.rs:879` reads locked header`:894`, physical length`:899`, refuses read-only repair`:902`, consumes finalize`:905`, rewrites cached encoding`:916`, flushes`:920`. Separate primary_unflushed flush/clear`:909` is outside E. Initial dirty-barrier persistence (begin_writable`:1473`) remains a caller obligation. Finalize success is not whole repair correctness.

## What is actually available to compose

- **Checked native bodies, qualified historical evidence:** R/selection includes secondary update/swap/accessors/select_primary_slot`:148`; E/reencoding-001 includes them at `:569`,`:588`, plus encoders`:1515`,`:1548`. Native selector (`header.rs:324`) keeps valid two-phase primary, otherwise newer checksum-valid slot/fallback. This does not validate its referenced tree.
- **Authored/model adapters:** R/publication.rs:33 `publish_fields` composes four native mutations but deliberately omits intervening I/O. R/crash_prefix.rs:17 `recover_checksum_candidates` ASSUMES intact old slot, candidate identity/validity and pointer/mode relation. `recover_acknowledged_image:103` ASSUMES `issue_write`, `sync_success`, parsed slot equality and primary validity. These assumptions are not native execution conclusions.
- **Checked constructive model:** P/source/tools/redb_crash_consistency94/persistence_model.rs:30 issue_write,`:49` persist_some,`:57` sync_success,`:63` power restart,`:69` process restart; header_order_model.rs:38 publishes full header AFTER modeled sync. StorageImage relations are not native flush summaries; hashes match R's historical attempt008/009 inputs.
- **Parser/finalizer:** F/composed-005.rs:1403 parse_and_finalize takes supplied bytes/u64 length, returns acceptance/error, finalized_from_bytes/layout equality; it does not read storage. E/roundtrip-005.rs:304 adds stable reencoding/reparse and second clean witness.
- **Interrupted reencoding:** E/interrupted-005.rs:119 requires encoded dirty origin, successful finalization, fixed supplied length, old/new-byte mixture and SAME target. repeated_writeback_images`:22` handles finite retries, framing flags. Commit/read_header do not yet establish these premises. Excluded behavior is not investigated.

Do not equate nominal R/F/E types. Use E's shared-type provider and port/recheck authored bridges, or prove a checked structural translation. Lexical equality is not nominal identity, nor cache correspondence.

## FIRST missing composition obligation

Relative to the checked field-mutation/codec frontier (with roots opaque), the first missing edge is **`page_manager.rs:1837`/`write_header:1623` -> exact cache-backed serialized-header publication**, maintained through the second write and `flush:1861` -> the existing storage relations. It must establish, rather than receive as free premises:

- which cache/issued/durable image actually contains the first and second native encodings, with the physical old-primary slot framed and the god-byte primary/mode bits from one generation;
- which values each interrupted backend event may persist, plus the actual parser's corruption flags and candidate identity, and successful final flush's complete header publication;
- the propagation from that barrier to the observed outer result, allowing errors with already-durable effects.

The candidate bridge also does not cover every early prefix as written: before the replacement reaches storage, the old secondary may remain intact but differ from the new prepared slot. Its premise `!corrupted_at(other) ==> other == prepared` is then unjustified. This is a scope mismatch, not a counterexample to the checked bridge and not a discovered redb bug. Prefix composition must cover stale secondary images explicitly; it cannot conjure candidate identity from a valid checksum. Strict ID ordering by itself supplies neither cache correspondence nor tree validity.

Root production, initial dirty barrier and entry invariant are unsolved caller premises. This first gap is relative to the header frontier, not every earlier native operation.

## ONE next implementation scope, with honest gate

**Proposed milestone: checked native onephase header-event refinement, fixed-geometry Immediate/ExclusiveWriter branch, from prepared root inputs through real header cache writes/final flush and the public result marker, into the existing native parser/selector.** This is one local protocol milestone, NOT a reduced final campaign scope.

Theorem/domain: any normal encoded dirty current-format header at physical length L; either primary index/prior successful mode; canonical opaque roots; new ID > selected old ID; one-step interval without geometry changes/subsequent transactions. Entry requires any checksum-eligible old secondary be older than selected primary, or a proved stronger history invariant; do not assume this for arbitrary repaired histories. Quantify actual native prefixes, power/process restart separately, and write/sync failures. Use only SCOPE's conditional no-undetected-relevant-corruption premise where needed, never hash injectivity; native validation coverage must be proved.

Postcondition: before ACK, reparsed/reselected HEADER SLOT is old or this transaction's complete slot, with stale/torn cases justified; after outer Ok/immediate crash, exact published header is durable and selects new slot. Err may leave either permitted state, never assumed rollback. Keep recovery/errors explicit. No root reachability, whole-database recovery, flag-clearing or KV/crash-atomicity conclusion. E's retries require a separately established encoded-origin/fixed-length/flag-framing cut.

Fresh plan ONLY after approval: (1) bind exact caller/E types; (2) prove buffered header writes/backend flush refine events, including coalescing/frames; (3) discharge early/stale-secondary and ACK parser/selector premises; (4) independently review an imported consumer/matched controls. Bound one candidate/one review, finite declared positive/control scopes and wall/disk/query limits; reset no old allowance. Stop at a missing native interface, not an oracle/axiom.

Matched controls: ACK before sync; substitute sync_file/write_barrier for flush; god-byte-only persistence with stale/torn secondary; wrong-buffer/slot bytes; split mode/pointer generations; rollback claim after post-sync error. Positives cover both indices/prior modes, stale secondary before issuance, torn-candidate fallback, successful outer ACK. Count only intended semantic rejection, never frontend/resource/compile/diagnostic failure. No controls run here.

**Authority gate:** native theorem NOT implementable from today's checked premises alone. SCOPE's byte-atomic-selector/sync semantics supply no native cache/backend effect correspondence. Minimum new authorization: bounded checked caller/storage-event integration, independently reviewed primitive read/write/sync/crash/error binding and actual cache-to-event interface. If this needs the missing source-to-capability producer, wait for separately authorized producer engineering; NONE is authorized here. Arc diagnostics/nonverifying builds do not supply this theorem. No library-destruction proposal/excluded pipeline restart. A model-only encoding trace would not discharge this native edge and is not a substitute milestone.

## Accounting and handoff

Only REPORT.md, INPUTS.json and ACCOUNTING.json are created in fresh scratch. Exit rechecks inputs. Zero verifier/solver/compiler/build/test/native runs, old-helper execution/import, network/download, Git writes, toolchain/runtime/cache copies, subdelegation, new trust/laws/assume/admit/external bodies, admission/export changes or adoption. Missing node command failed before running a script; stdlib Python/SHA performed authentication/accounting only. Entry free disk 53,790,888 KiB (>16GiB); exit sizes/disk recorded separately. No excluded task resumed. All18 OPEN; S0/fail-closed gates unchanged.
