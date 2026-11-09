# Independent completion103 review and verification-only recheck

## Findings first

No new blocking proof-source, theorem-contract, fixture, semantic-control, import-binding, or source-preservation defect was found within this bounded conditional review. The four required exact rechecks meet their specified outcomes. This is NOT whole-goal or native implementation acceptance.

1. **HIGH — inherited native correspondence gates remain open, not a newly introduced completion103 defect.** Candidate `restored-positive.rs:33` requires `supplied`: branch ordering, ordered supplied leaves, and every stored key's child interval are caller premises. Candidate `restored-positive.rs:73` returns a decoded canonical page number and searches the separately supplied `leaves[selected]`; it does not fetch a page or prove that the fetched image equals that supplied slice. Neither actual fetched-page identity nor establishment/preservation of reachable native-tree descendant ranges is proved. Native-path preservation remains FAILED; Git continuity remains UNAVAILABLE. Those facts prevent native/whole-goal acceptance despite the successful local checks.
2. **MEDIUM — bounded historical recording limitations remain explicit.** Candidate `SETUP-FAILURES.json:1` and `FINALIZATION-FAILURES.json:1` disclose complete inherited shell environments missing for two non-verifier audit commands. All ten verifier argv/full environments and raw logs are retained and were read/compared. No raw missing shell record was invented. This review also encountered two non-verifier metadata-audit failures, conservatively charged separately and disclosed in `REVIEW-AUDIT-FAILURES.json:1`; their inherited shell environments were not captured, while all FOUR actual fresh verifier environments were captured exactly. No verifier setup/spawn failed and no proof/assumption was repaired.
3. **INFO — scope and negative-coverage boundaries must not be enlarged.** Candidate `attempt-005/actual-scope.json:1` and fresh `check-01-positive/actual-scope.json:1` identify 19 authored executable plus 19 proof bodies, not 38 newly verified imported native bodies. Both semantic controls are fixed-fixture controls only (`wrong-child.rs:1035`, `wrong-value.rs:1035`). Variable-mode positive coverage is present; separate variable-mode negative coverage is not.

Candidate-relative paths in this report resolve under `/private/tmp/branch-leaf-completion103-impl-R5nrcY0D`. Review-relative paths resolve under `/private/tmp/completion103-independent-review-glXjLOpk`. Historical checked102 means `/private/tmp/branch-leaf-composition102-impl-SqXAof8z/attempt-003/input.rs`.

## Precise verdict

- Generic theorem/meaning bridge/import-contract review: **PASS, conditionally on the frozen imported contracts and inherited engine/TCB boundaries**.
- Constructed fixed/variable fixtures and exact association assertions: **PASS**.
- Preservation and fixed semantic controls, including historical corrected counterparts: **PASS**.
- Four fresh full-local-scope retained-engine rechecks: **PASS**.
- Retained/recovered source continuity evidence: authenticated, **not** upgraded to exact native-path or Git continuity acceptance.
- Native fetched-image correspondence / reachable-tree range invariant: **NOT ESTABLISHED**.
- Whole-goal acceptance/adoption/integration/cell promotion: **NOT ACCEPTED; all 18 cells remain OPEN**.

## Authentication and chronology

Task start was 2026-11-08 01:57:39 UTC / November 7, 2026 19:57:39 CST. The absolute optional-work/new-application cutoff is 02:09:39 UTC; absolute completion deadline is 02:12:39 UTC. All task setup, authentication, execution, audit and sealing are charged to this elapsed window, not only verifier runtime. No new application is allowed after the four executions because the conservative six-charge cap is exhausted.

The four user-supplied parent document hashes, external candidate manifest/seal hashes, report/accounting pins, exact positive hash, historical checked102 source hash, and handoff archive hash all match. All 115 payloads plus MANIFEST/SEAL (117 exact candidate files) and all 320 distinct current bindings match at entry and immediately before/after every execution. Final exit authentication repeats those checks. There is no unexplained drift. Original planning README/shared status remain frozen; current state is read from CANDIDATE-STATUS, not stale planning language. The two historical102 metadata reconciliations are context-only and were not silently treated as live historical hashes.

`ARCHIVE-AUDIT.json` independently confirms 117 unique regular archive members, exact candidate membership, and every member byte-exact by SHA256. Nothing was extracted over the candidate. Candidate archive SHA256 is `ec0ee5608b3c5c76a442aae379d1ebdb96beab03e164d5713df7d313a8fb77dd`.

Full candidate authentication preceded the first fresh verifier execution. Some evidence was read during discovery before the full endpoint authentication, but no candidate runner/script was executed in place. All review writes are in the fresh review scratch; candidate, parent, providers, toolchain and runtime are read-only inputs. The existing matched runtime and selected historically bound Rust 1.99.1 artifacts are authenticated, not copied, rebuilt or represented as a fresh complete sysroot audit.

Candidate manifest SHA256: `b932d111092d0a19362b332bd70965f4dafbdd31543c3ee6125ab2cb7115e165`.
Candidate seal SHA256: `cd57e95cd9d8255b25a4f870ef24456030b7aa7fec4c7d83b0515e0a998d7c14`.
Candidate report SHA256: `b93ab64c00e53fc5e48b30d34f30a3b33afe0938ebd2d5624230b6778385052b`.
Candidate accounting SHA256: `5ee3fd41801d82922763122459c682eb44732514aa76d2c10b961deed91ef90f`.

## Independent semantic review

### Generic preservation and meaning bridge

`GENERIC-PRESERVATION.json` independently compares complete declaration spans, including signature, requires, ensures and body, against checked102 source SHA256 `e15f151f0211c269531c016cdd6956aba8de0ea5bdd08c472783a064fda10538`. All seven spans are byte-identical: meaning_bridge at candidate line 8 / old line 8; meaning 15/15; interval 27/19; supplied 33/25; union_has 43/35; matching_child 49/41; route_then_search 73/65. No narrowing, desired-result/existence premise or generic-assumption repair was introduced.

`meaning_bridge` is a checked proof, not a trusted/external-body declaration: both concrete u32 Value meanings expand to the same little-endian `vstd::bytes::spec_u32_from_le_bytes(bytes) as int`. The four-byte validity condition is retained. Independently inspected source anchors are `native-leaf-lookup-007/leaf.rs:479` and `native-branch-routing-023/leaf-007.rs:479`, both under `/private/tmp/verus-crash-consistency94-20261105`. This agreement is relative to the inherited imported/std/LE semantics, not an independent physical-machine/compiler proof.

### Interval, exclusion, presence and exact association

`interval` at line 27 is lower-exclusive and upper-inclusive: child > 0 implies previous separator < key, and child < count implies key <= next separator. Imported `native-branch-routing-023/clients-003.rs:31` defines routed_bytes with every preceding separator strictly less than the query and every following separator >= it; `route_u32` at line 41 requires ordered_bytes plus a four-byte query. Equality therefore routes LEFT, including separator256 -> child0 and separator65536 -> child1. Native route/search are not substituted.

`matching_child` at line 49 excludes every non-selected child: an earlier child's inclusive upper separator contradicts routed preceding strictness; a later child's exclusive lower separator contradicts routed following non-strictness. The meaning bridge is applied to valid four-byte separator/query encodings. This proves exclusion for exact matching keys throughout the supplied union without assuming that a match exists.

`route_then_search` at line 73 calls the actual imported `branch_clients023::route_u32` and then `lookup_client::search_u32(leaves[selected], ...)`. The imported search contract (`native-leaf-lookup-007/consumer.rs:48`) gives presence iff an exact byte key exists in that selected page, exact returned key bytes, and a witness index relating that key to its associated value bytes. Its ordered_leaf premise at line 30 includes layout validity, four-byte keys, and strict numeric ordering. Canonical four-byte u32 encoding (`consumer.rs:39`) reconciles numeric search equality with byte equality.

The generic proof transports selected presence to union presence directly; for union presence with no selected hit it chooses a real union witness, uses matching_child, and obtains the contradiction with the search contract. Thus the iff is not one-way or assumed. The postcondition's same-index selected-page key/value witness preserves exact association, not merely a value found somewhere in another leaf. PageNumber equality/canonicality is useful but still not fetched-page identity.

### Fixtures and non-vacuity

`FIXTURE-AUDIT.json` independently checks all 16 query callers and their unguarded six-u8 signatures. Fixed images are branch88/leaf14 bytes with Some(4)/Some(1); variable images are branch96/leaf30 bytes with None/None endpoint tables. The six fixed/variable leaf proofs derive layout endpoints, monotonic bounds, pair count, encoded keys, exact symbolic value bytes and numeric ordering from image equalities. Branch fixture proofs derive key count two and separators256/65536; helper premises are byte/endpoint facts discharged by constructed arrays, not an assumed lawful fixture.

`fixture_lawful_fixed` at line 528 and `fixture_lawful_variable` at line 579 assemble ordered leaves and per-key child intervals, then the dispatcher establishes supplied. No executable positive caller requires supplied/order/range as an unproved caller input: it builds concrete arrays and calls those proof helpers. All values first/second/third/fourth/fifth/sixth remain independent unrestricted u8 arguments, not constants or distinctness premises.

Each mode checks present keys 1,256,257,65536,65537,u32::MAX with expected children0,0,1,1,2,2 and exact associated argument values respectively first..sixth; absent0/2 yield None. Query assertions cover exact returned key bytes as well as value bytes. `numeric_not_lexical` at line 631 checks the lexical inversions 1 versus256 and256 versus65536. This is finite constructed-image non-vacuity under the inherited contracts, not a universal native-tree establishment proof.

`VARIANT-PRESERVATION.json` independently confirms every one of the 47 positive named entities is byte-identical in both negatives and both corrected counterparts. `TRUST-SCREEN.json` corroborates source review: no new assume/admit/axiom/trusted/external-body/admission markers occur. Existing ghost assert(false) steps are checked contradictions, not assumptions.

## All ten historical applications

The raw stdout/stderr, complete invocation environments, timing, full actual-verification JSON, full scope and source for all ten applications were read and retained under `historical/`. Every full JSON equals its raw stdout parse, including func-details, verus-infer, verus and verification-results, not just counter excerpts. Scope JSON equals the whole JSON's scope. Flags and normalized full environments match throughout. Historical attempt001 timing.summary contains the whole JSON envelope; subsequent timings use counter summaries. The audit explicitly unwraps verification-results for that first timing record and never rewrites historical evidence.

- 001: 14 verified / 2 errors, 39.927747s, rlimit failures in fixed/variable lawful fixture proofs. Not a semantic control.
- 002: 16/3, 26.001392s, rlimit failures in fixed/variable branch fixture proofs and fixed witness. Not a semantic control.
- 003: 16/3, 36.131919s, rlimit failures in fixed/variable branch fixture proofs and fixed witness. Not a semantic control.
- 004: 19/2, 25.212370s, rlimit failures in fixed and variable witnesses. Not a semantic control.
- 005: complete positive 38/0, 4.442380s, 19 exec + 19 proof, entire crate.
- 006: wrong-child 38/1, 5.082249s; only backend assertion at input.rs:1063:12.
- 007: wrong-value 38/1, 5.258390s; only backend assertion at input.rs:1064:12.
- 008: corrected wrong-child 39/0, 5.364536s.
- 009: corrected wrong-value 39/0, 4.489993s.
- 010: byte-identical positive restoration 38/0, 4.048746s.

Wrong-child constructs valid fixed images at values11,12,22,23,33,34; query256 correctly routes left (selected0), deliberately searches middle, proves no hit and supplied-union presence, and then makes the sole false iff assertion. Wrong-value query257 correctly routes/searches child1, proves exact returned value22 and first11 != third22, then falsely claims value11. Neither negative is credited for process exit alone or a frontend/precondition/bounds/resource/timeout/panic/import failure.

`COUNTERPARTS.json` and independent diffs compare same-length source line arrays: corrected wrong-child changes only line1063 to `assert(hit.is_none() && union_has(...));`; corrected wrong-value changes only line1064 from seq![first] to seq![third]. Their 39/0 results plus the actual false/asserted contrary facts support meaningful, non-vacuous fixed-fixture controls. They were audited historically, not freshly relaunched; the task requests only four fresh applications.

Main completion103 ledger remains 12 charges / 10 launches, about789.5seconds. The ~0.016second difference between accounting-record and final-seal elapsed times is normal finalization timing, not another application. Its two disclosed non-verifier recording gaps remain. Old102 remains 9 charges / 7 launches, partial, no successful controls/restoration, with its earlier research wall violation retained. This review does not reset or waive either ledger.

## Four fresh exact rechecks

All four use the original matched midpoint runtime, selected Rust1.99.1 sysroot, all seven exact release VIR/rlib pairs, unchanged flags and full recorded historical environments from attempts005/006/007/010. Only source/cwd and writable environment paths redirect into review scratch. `IMPORT-PAIRS.json` authenticates each of the14 pair artifacts. Overwritten branch023/artifacts is never used. There is no function filter, --compile, --export, build, network or native application execution. Sequential processes use --num-threads1 and the original single solver-worker invocation. Each has a min(60seconds, remaining launch-window) timeout and process-group kill on timeout.

- `check-01-positive/`: exact positive, SHA256 `1e501a924f43adf863a80dc1454fdb1e506f071ed74caf79e02b02868bf5fcaf`; 38/0, return0, 4.466035s.
- `check-02-wrong-child/`: SHA256 `7ad4249add65940d7cf0ddc70326ebb817894827021d157287de8cf105c5a91e`; 38/1, return1, 5.338320s; only intended backend assertion1063:12.
- `check-03-wrong-value/`: SHA256 `b9ac093ebf21494f999e31badbce931891a5ec3daac7f1801e5124fc72e5cccb`; 38/1, return1, 4.974090s; only intended backend assertion1064:12.
- `check-04-restored-positive/`: byte-identical positive, same SHA256; 38/0, return0, 4.702306s.

Each check retains exact input.rs, invocation.json with the COMPLETE actual child environment (no inherited merge), stdout/stderr, whole actual-verification.json, actual-scope.json, timing-result.json and output hashes. `FRESH-RESULT-AUDIT.json` rejects unrelated error categories and confirms both intended diagnostic locations. `SCOPE-AUDIT.json` confirms every source-named function is selected, no generated/imported native bodies count as local checked bodies, and both positive inventories are 19 exec +19 proof. Negative scopes correctly withhold complete-selected-scope certification and show zero certified-body counters on an unsuccessful overall run; their raw verification-results still report38 verified/1error. That reporting distinction is not inflated into a claim of39 successful bodies.

Fresh verifier runtime totals19.480751seconds, with max5.338320seconds/application, no timeout/spawn failure. Four verifier charges plus two conservatively charged review-only metadata failures equal6 charges/4 launches. No further applications are permitted. The first metadata error was an extra closing parenthesis in a read-only listing; the second was the review audit incorrectly expecting attempt001 timing.summary to be only counters. Both were audit/setup corrections only; no candidate proof/assumption/source/invocation was repaired or verifier rerun. Full inherited shell environments for those two metadata commands are not claimed. Initial ls/PATH and nonexistent /usr/bin/date discovery messages were non-application read-only diagnostics, also disclosed.

## Remaining gates and handoff

Required next gates are actual fetched-image/page-number correspondence, native reachable-tree interval/range establishment and preservation, native source-path/Git continuity acceptance, and the still-open end-to-end database obligations. None is discharged by supplied-array composition or finite fixture construction. Preserve the inherited midpoint/std/slice/array/StructuralEq/LE/trait/comparison/loop/adapter/compiler/solver boundary, selected-sysroot-only audit scope, and ownership/Drop/panic/glue/allocator/I/O/history limits. Variable-width witness bytes do not certify native fixed-u32 policy. Absence of an unrequested export does not prove export refusal.

No axioms, assumptions, admissions, external bodies, trust/export/admission policy changes, runtime adoption, integration or native execution were introduced. No growth049, zero-growth050, mutation030, refused-padding95 or drop-pipeline work was resumed/rerouted. No subdelegation occurred. All18 cells remain OPEN.

`ACCOUNTING.json`, entry/check-pre/check-post/exit authentication receipts, all raw check directories and independent audit JSONs are payloads of the review manifest. `SEAL.json` records final wall time, resources, report/accounting/manifest hashes and exact payload membership. The review content seal is only an integrity receipt, NOT a certificate or whole-goal acceptance.
