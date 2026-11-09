# Campaign94 checkpoint008: new independent read-only source/evidence review

Review date: November 7, 2026 (America/Chicago). This is a new review, performed by the requested review agent itself, without delegation. It is NOT a verifier replay, implementation round, trust/scope adoption, or reset of any historical allowance.

## Findings first

**Recommendation: qualified support for the bounded authored selector/publication evidence ONLY. No blocking correctness/soundness defect was found in the six selected source bodies or the stated conditional contracts by this source/evidence inspection. Native database acceptance remains withheld. All eighteen final obligations remain OPEN.**

### C0 — Blocking correctness/soundness issue: none established in this bounded slice

Independently comparing the complete executable bodies and signatures, rather than trusting `source-correspondence.json`, supports correspondence of the six named methods after the explicitly enumerated erasures/projections below. The two recovery policies, corrupt-primary behavior, tie behavior, selected field/sequence frames, and original-relative corruption flags agree with the inspected native source. I found no contradictory caller premise or false conditional postcondition. This is NOT a fresh solver result or a finding that the compiler, dependencies, native callers, storage engine, or whole program are sound.

### EQ1 — Evidence qualification: selected transaction-slot equality is not whole-header recovery

`crash_prefix.rs:17` proves the selected `TransactionHeader` equals an old/prepared candidate and frames the recovery input's remaining payload. It does NOT prove that all recovered `DatabaseHeader` metadata equal an old/prepared header. In particular, `prepared_from(old_header, prepared_header, ...)` is not a precondition of this bridge; the candidate's relevant slot and increasing ID are supplied directly. Page size, region layout, recovery/unflushed flags and bytes-to-header decoding are not established by the candidate names.

The checkpoint REPORT's phrase “corresponding old/new header” must therefore be interpreted narrowly as **corresponding selected transaction-slot payload**, not equality of the whole database header or a native crash-image refinement. This is a handoff-language qualification, not a counterexample to the actual theorem. `selector_fields` relates only primary bit 1 and mode bit 4, not all selector-byte flags or all header fields. The other two bridges also rely on explicit supplied slot/field relations rather than deriving a full native parse.

**Changes-to-next-action:** carry this narrower statement into any parent summary; do not substitute the helper for a whole-header/native recovery obligation.

### EQ2 — Evidence qualification: final012 is supported; scratch010 remains withdrawn

All 167 manifest payload hashes and sizes match; SEAL binds that manifest. Final `crash_prefix.rs` equals attempt012's input, and receipt012 binds both `recovery_prefix-final.vir` and attempt012's `librecovery_prefix.rlib`. Attempt021 imports exactly those two hashes, and its ordinary, unannotated consumer actually calls `recover_checksum_candidates`. Its historical raw result is 1 verified / 0 errors.

Scratch010's receipt records a different VIR at `recovery_prefix.vir`, hash `81731185821d7e38179ea0644b26a42d96ee80f02c628851875402d2c3e7b359`. That file is absent. Attempt011 used that same export pathname and failed in the frontend. The surviving scratch010 source, rlib, 2/0 output and receipt are historical partial evidence only: **no final import/artifact credit** for scratch010. No missing recorded artifact other than that disclosed withdrawn VIR was found. A successful checksum consumer is not a separate external call test of the other two exported bridge functions.

**Changes-to-next-action:** use only final012 when referring to the final bridge artifact. Do not repair, regenerate or substitute for the withdrawn scratch export under this review.

### EQ3 — Evidence qualification: historical logs are not fresh proof reproduction

The 21 launch records match their per-attempt invocation objects. Raw JSON results and scope objects match receipts; all input-snapshot, stdout and stderr hashes match. Eight apparent authoring-path differences are preserved earlier drafts, each exactly matching its attempt's archived `input.rs`, not evidence of a changed frozen final input. The complete ledger and failures are detailed below.

These are retained historical verifier applications, not applications performed by this reviewer. Self-consistent logs, captured compiler metadata, checked-token identity, scope counters, green results and hashes do not certify full native/compiler/database acceptance. No independent machine-wide process census is supplied by a 21-entry self-recorded ledger.

The first scaffold patch failure, first token-audit failure and failed archival-copy incident are narrated in the checkpoint REPORT. Stale attempt001 labels, later runner correction, the changed signature normalization, scratch export-path reuse and the missing scratch VIR are directly inspectable; the exact shell failure chronology is not independently established by standalone shell transcripts in the frozen package. Likewise, the two final preservation JSON files are byte-identical and contain no capture timestamp: their contents alone do not establish the narrated post-attempt021 recapture ordering. This does not invalidate their static equality observations or the sealed final artifacts.

**Changes-to-next-action:** preserve these distinctions rather than claiming this review replayed proofs or independently authenticated every shell event.

### EQ4 — Evidence qualification: runtime identity is bounded; provenance gaps remain

The nine codec-runtime-005 files were freshly hashed, not launched; all match the invocation/runtime-binding map. Six selected Rust 1.99.1 aarch64 Apple sysroot files also match the inherited 3340-entry sysroot inventory. This review did not rehash every sysroot entry, inspect all compiler dependencies, or prove tool correctness. The runner source checks a historical 1084-entry source map; that map remains **NONEXHAUSTIVE**, not a compiler-closure inventory. The runtime binding's own independent-review field is `OPEN`; identity matching does not close it.

All 48 files beneath the recovered native `redb/src` match the corresponding SHA-256 entries in the bound recovery JSON. The three primary native hashes also match `source-correspondence.json`. This is recovered-content equality, NOT original-path or Git continuity. The original redb header, page manager, transaction tracker and `.git/HEAD` paths were freshly checked and are still missing. The recorded campaign-wide 1331 missing native inputs and 1401 immutable checks remain historical guard observations; I did not independently re-audit unrelated recovered repositories or every immutable dependency.

The historical initial/final maps contain 2347 equal source entries; owned branch, HEAD, index hash and status match, while shared `refs` differ. No whole-store Git nonmutation follows. Original native-path preservation stays **FAILED**; native Git continuity stays **UNAVAILABLE**; inherited/shared Git-object-store nonmutation stays **NOT_CERTIFIED**, without attributing a cause.

**Changes-to-next-action:** retain exact content and runtime qualifications; no provenance or compiler-closure upgrade.

### EQ5 — Evidence qualification: live parent context changed during review

Exit hashing detected changes to parent GOAL.md and STATUS.json, not to the frozen checkpoint or recovered native inputs. The parent now reports BLOCKED pending separate implementation/build-test authorization; all eighteen cells remain OPEN and whole-goal acceptance remains false. Other newly listed lanes were not followed or adopted. The parent-source table below records the initial reviewed bytes; exit hashes are:

- GOAL.md: `5769809914a6b218bb35b841bc442ba48728061f645a072a147e7d778ea3a099`.
- STATUS.json: `106acfe7abebf3fe17fee6b50a81735a228364e83f9af2f93efa79a93bbb9504`.

This reviewer did not write either file. An all-reviewed-inputs-unchanged claim would be false; only the frozen evidence/native/dependency inputs and SCOPE are unchanged. No causation or authorization upgrade is inferred. Changes-to-next-action: hand off this review against both recorded context versions, preserving the parent's authorization blocker.

### OD1 — Still-unproved outer dependency: native parse, validity and accepted-payload establishment

The caller supplies a primary index below two and corruption flags; the selected bodies do not establish them. Native `UnrepairedDatabaseHeader::from_bytes` derives the index from the god byte, validates geometry, parses both slots, and maps slot0/slot1 checksum booleans into flags relative to the original primary (`header.rs:123`, `:128`, `:185`, `:191`). Native `TransactionHeader::from_bytes` has version/error branches before checksum/root decoding (`:543`), computes a slot checksum (`:557`), parses roots and ID, and preserves invalid original bytes (`:589`). Those paths are not proved by this fixture.

The checksum candidate premise “an accepted new candidate has exactly this prepared payload” is the integrity burden, not a proved checksum oracle. It needs actual validation coverage and execution-specific no-undetected-relevant-corruption/collision qualifications; finite checksums are not universally injective. Full hash dispatch and validation of pages reachable from the roots remain outside this slice. Two-phase selector reasoning does not itself borrow a checksum-injectivity axiom, but its supplied validity/slot correspondence still needs establishment from the preparation barrier and actual writes.

### OD2 — Still-unproved outer dependency: actual commit, I/O, errors and acknowledgement

`publication.rs:33` is an authored in-memory composition, NOT a complete native `commit` body. It matches four actual native statements in their order, with `data_root` renamed to `user_root`; the intervening writes/locks/flushes are not erased proof code. Native commit also checks dirty pages and I/O errors, locks/clones state, attempts shrink, asserts increasing transaction ID, acquires header holds, writes the prepared header, conditionally flushes for the NEW two-phase mode, writes the switched header and flushes again (`page_manager.rs:1797`–`:1861`). Shared mode then clears the unflushed bit and writes again; resize, unpersisted-allocation clearing, state publication and return occur later (`:1862`–`:1891`).

Native `write_header` (`:1617`) chooses direct shared-file writes or a buffered mutable write/copy; `header_hold` (`:1021`) can acquire a fallible lock. A successful modeled sync is a conditional persistence relation, not proof that these native buffers, locks, backend writes, file lengths, flush return values or protection contracts implement it. The model has byte-level pending choices, no native application-buffer layer, no shrink/file-length semantics in this theorem, and no native I/O-error execution. Fallible operations exist even after the final flush. Therefore a persisted modeled image is not proof of public `Ok`/`Err` behavior or acknowledged database durability. `Err` must not be assumed to mean rollback.

### OD3 — Still-unproved outer dependency: consuming recovery and fresh flag correspondence

Native `finalize(mut self, file_len)` consumes the unrepaired wrapper and additionally reconciles file length/layout, handles errors and computes a clean flag (`header.rs:243`). `finalize_transaction_slots(self)` is another distinct native policy that keeps the recorded primary (`:364`); it is not replaced by this selected repairing-writer method.

After selection swaps the primary pointer, the two corruption booleans are still relative to the ORIGINAL pointer. This is explicitly framed by `selection.rs:158`. For example, checksum input primary0-corrupt/secondary1-valid swaps to 1; repeating the same mutator with unchanged flags can swap back. This is an analytic source walk, not execution or finite-model proof credit. `corrupted_at` is appropriate for the original input in these premises, not a promise that unchanged flags become relative to the final pointer. No repeated-wrapper idempotence or interrupted-repair correctness follows. Restart induction must reparse the persisted image and re-establish flag meaning.

### OD4 — Still-unproved outer dependency: native resources and compositional histories

No allocation, String resource/ABI, panic/unwind, lock, clone/drop, Arc/Option/slice ownership, native dependency-closure or resource-engine proof credit is granted. S0 whole-native admission, ordinary Infer full-scope, import/export and fail-closed publication barriers remain unchanged. The existing six reviewed LE std assumptions remain **inherited trusted boundaries**, not Rust-core implementation proofs; no law, external body, hash oracle or I/O oracle is added or adopted here. Existing trusted library models and the compiler/solver remain inherited dependencies, not eliminated by `--no-cheating`.

Mixed previous-mode cases in a prefix helper are not induction over arbitrary mixed histories. Preservation/reclamation, logical tree/KV validity, successful operation after recovery, public error correctness and all six final properties remain unproved for every required history class.

## Source/body/signature/type review

Input aliases used throughout this report:

- `P = /Users/tommyvolk/.t3/worktrees/verus/t3-60978068`
- `C = /private/tmp/verus-crash-consistency94-20261105`
- `R = C/native-recovery-selection-008`
- `N = C/native-leaf-lookup-007/recovered-native/redb/src`
- `U = /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin`

Read `P/evidence/redb-crash-consistency94/{GOAL.md,SCOPE.md,STATUS.json}` and `R/{PLAN.md,REPORT.md}`. The current parent goal includes separate resource-lane context; none of its unrelated work was resumed or adopted. The historical PLAN's 8 GiB threshold does not override this review's stricter greater-than-16-GiB floor.

Independent comparison used a newly authored in-memory, comment/string-aware lexical reader and balanced-body comparison, not `audit.py`, its imported older tokenizer, or any old script execution/import. Method bodies were also inspected directly with native surrounding statements. This is bounded source equality, not a Rust parser/typechecker or a compiled ABI/layout proof. The independent lexer splits punctuation differently from the old audit, so its normalized token digests are not substitutes for that audit's tokenizer-specific digest strings.

| Method | Native `header.rs` line | Projected `selection.rs` line | Complete body/signature check |
| --- | ---: | ---: | --- |
| `select_primary_slot` | 324 | 148 | Equal; no inner proof block erased |
| `primary_slot` | 448 | 93 | Equal; no inner proof block erased |
| `secondary_slot` | 452 | 100 | Equal after one checked bounds proof block |
| `published_slot` | 458 | 108 | Equal; no inner proof block erased |
| `write_secondary_slot` | 467 | 129 | Equal after one checked bounds proof block |
| `swap_primary_slot` | 480 | 120 | Equal after one checked bounds proof block |

Explicit accounting:

1. Remove comments/whitespace, visibility differences and the named result binder; normalize a trailing parameter comma. Ignore specification clauses when comparing executable signatures. Preserve all executable statements, literal strings, argument types, receivers and return types under the selected aliases. The three erased in-body blocks are exactly `proof { other_slot_bounds(self.primary_slot); }`, not assumptions or executable rewrites.
2. `TransactionHeader`'s five declared fields, `DatabaseHeader`'s ten and `UnrepairedDatabaseHeader`'s three match by name/type/declaration order modulo visibility. Native `TRANSACTION_SIZE = 128` matches. Native Clone derives on the header structs are not reproduced/verified here. Matching declarations is not a claim about compiler-selected physical field offsets or ABI.
3. Native `TransactionId(u64)` at `transaction_tracker.rs:16` becomes a public tuple field; relevant `Copy/Clone/Eq/PartialEq/PartialOrd/Ord` derives remain, with authored inference annotations for comparison checking. Hash and Debug are omitted; no hashing/debug/ID-allocation/overflow theorem follows. The actual native ID methods are not checked by this selector fixture. The six native method count does not include generated code.
4. Native `crate::Result<T = (), E = StorageError>` is projected to `Result<T> = core::result::Result<T, StorageError>` for the selected `Result<bool>` use. Native non-exhaustive `StorageError` has other variants; the fixture retains only `Corrupted(String)`. Exact Corrupted messages are preserved, but full enum layout, discriminants, error conversions, allocation and ambient library ABI are not proved.
5. `BtreeHeader` is imported from the bound root codec, with its `PageNumber`, `Checksum = u128` and `length: u64` dependencies. The native/root-codec field shape was inspected (`N/tree_store/btree_base.rs:138`, source-runs068 input). This does not establish full native type/library closure. The existing checked comparison machinery is not an assumed comparison oracle, but its historical compiler result is not independently re-executed here.

Provider003's historical scope lists six selected native executable methods, four generated checked bodies (`clone`, `cmp`, `eq`, `partial_cmp`) and one authored bounds proof. Generated Eq's marker method is outside selected scope; generated specs/constant proxy are not extra proved native bodies. The 16 verification queries are not 16 native methods. Prerequisite001 separately exercises `left > right` and the literal error constructor.

## Policies, fields and caller nonvacuity

The selector's only explicit entry premise is primary index `< 2`; corruption flags and transaction IDs are otherwise not restricted to a favorable branch.

| Input mode/validity/order | Native and projected result |
| --- | --- |
| Two-phase, primary corrupt | Corrupted error, even if secondary valid/newer; pointer unchanged |
| Two-phase, primary valid | `Ok(true)`; keep primary regardless of secondary order/corruption |
| Checksum, both corrupt | Corrupted error; pointer unchanged |
| Checksum, only primary corrupt | `Ok(false)`; swap even if secondary ID is lower or tied |
| Checksum, primary valid, secondary corrupt | `Ok(true)`; keep primary |
| Checksum, both valid, secondary strictly newer | `Ok(false)`; swap |
| Checksum, both valid, tie or older secondary | `Ok(true)`; keep primary |

The exact error texts are `Primary is corrupted despite 2-phase commit` and `Both commit slots are corrupted`. `Ok(true)` means the original primary was kept, not that all database validity obligations were proved.

Frames were checked field by field: selection/swap retain every non-pointer metadata field and `transaction_slots@`; selection also retains both original-relative corruption flags on all paths. The accessor contracts return the indexed slot; `published_slot` chooses the opposite slot iff primary-unflushed. `write_secondary_slot` changes only the secondary transaction ID, two roots and `corrupt_bytes = None`, retains that slot's version and the old primary slot, and preserves all metadata. These are array sequence observations, not the whole-array/whole-struct identity assertions that failed in draft005.

For the publication composition, the old primary is framed, preparation overwrites only the secondary, publication flips the pointer then sets mode/unflushed. A shared reader serves the former primary while the new primary is marked unflushed; this is an in-memory observation only and does not extend Campaign94's single-process scope to concurrent native readers.

Nonvacuity was checked analytically, not by running a finite model: either index 0 or 1 is possible; version3, roots `None`, IDs 10/11 (or 10/10 for the tie control), and independently chosen flag booleans inhabit all relevant policy cases without overflow. No branch is excluded by an impossible conjunction. Each final negative shares the corresponding final positive's premises and changes the asserted answer. The increasing-ID premise is consistent with native commit's actual assertion (`page_manager.rs:1825`), but proving native callers satisfy it remains separate.

The three conditional bridges were challenged as follows:

- **Checksum candidates:** the old slot must be valid and unchanged at its physical index; any accepted candidate at the other physical index must equal the supplied newer payload. Mode is explicitly linked to the recovered pointer/old mode. If parsed primary is the new slot, the old slot is its opposite and original-relative validity is handled accordingly. A corrupt new slot falls back only in checksum mode; an old two-phase pointer keeps old. The proof does not infer flag correctness or checksum adequacy from ID ordering.
- **Two-phase publication prefix:** successful preparation sync resets permitted byte values; the rewrite changes only the selector byte. Its whole old/new byte choice keeps mode and pointer together. With supplied valid slots/parse correspondence, unchanged old two-phase mode keeps old, unchanged old checksum mode may choose the newer secondary, and the new two-phase selector keeps new. This concerns publication of a NEW two-phase commit after either previous mode, not all interruption points of either commit mode. Slot equality and validity are premises, not consequences of the byte lemma.
- **Acknowledged image:** a successful modeled sync fixes the entire written header range. `published_from`, increasing IDs, primary validity, supplied slot equality and selector parsing then establish the published selected transaction and mode for either final mode; secondary validity need not be assumed when the valid published primary has the larger ID. It does not derive the actual write's encoding, public acknowledgement, whole-header metadata or shared-reader visibility from those assumptions.

The storage foundation inspected in attempt008 defines separate durable/visible byte sequences and per-byte permitted sets, bounded `issue_write`, nondeterministic `persist_some`, and explicit `sync_success`. Header-order attempt009 derives selector-only rewrite framing and successful full-range sync from those relations. These are constructive, consistent model definitions, not new external I/O axioms. Power-only and process-only restart relations exist in the foundation, but the three bridge clients do not prove complete native restart behavior or interrupted repair. No tiny-domain enumeration is substituted for native proof.

## Historical launch and draft/error accounting

All attempt IDs below are historical. Fresh review launches: **zero** verifier/compiler/solver/harness/Cargo/vstd/native applications; zero builds; zero retries; zero allowance resets. No network, Git commands/writes, old-script execution/import, source edits or trust changes were performed. Only read/parse/hash operations and creation of this new report were used.

| Attempt | Historical raw outcome | Review disposition |
| ---: | --- | --- |
| 001 | 8/0; rc0 | Prerequisite check; stale `LEAF-LOOKUP-001` ID and lookup-scope label, correct new cwd/input/runtime; not selector closure |
| 002 | 14/2; rc1 | Missing local BV premise fails at 32- and 64-bit arch sizes; two deprecation warnings; zero complete-scope/control credit |
| 003 | 16/0; rc0 | Final six-method provider plus generated machinery/bounds proof |
| 004 | Frontend error; rc1 | Ghost place in executable branch; no backend proof credit |
| 005 | 0/2; rc1 | Whole-wrapper/whole-inner equality assertions fail; no identity theorem |
| 006 | 3/0; rc0 | Final authored in-memory publication composition |
| 007 | 2/0; rc0 | Final ordinary imported clients with metadata/slot-sequence frames |
| 008 | 20/0; rc0 | Existing byte-persistence model freshly checked historically, not by this reviewer |
| 009 | 3/0; rc0 | Existing header-order model freshly checked historically |
| 010 | 2/0; rc0 | Scratch bridge; source/rlib/output retained, VIR missing; withdrawn artifact credit |
| 011 | Frontend error; rc1 | Ghost-struct field unsupported in BV conversion; same scratch export path |
| 012 | 5/0; rc0 | Final three conditional bridges, distinct preserved final export |
| 013 | 4/0; rc0 | Earlier positive controls, superseded by018 |
| 014 | 0/1; rc1 | Intended false two-phase-newer assertion fails; recommendation notes; not clean control |
| 015 | 0/1; rc1 | Clean false two-phase-corrupt-is-Ok assertion fails |
| 016 | 0/1; rc1 | Clean false checksum-refuses-fallback assertion fails |
| 017 | 0/1; rc1 | Intended false checksum-tie-swaps assertion fails; recommendation notes; not clean control |
| 018 | 4/0; rc0 | Final four positives, explicit XOR bound premises, clean stderr |
| 019 | 0/1; rc1 | Final clean false two-phase-newer-swaps assertion fails |
| 020 | 0/1; rc1 | Final clean false checksum-tie-swaps assertion fails |
| 021 | 1/0; rc0 | Ordinary imported checksum consumer calls final012 contract |

The 21 ordinals/reservations are consecutive 1–21 under the historical 48-direct-application cap; eleven have rc0, ten rc1. Every saved launch has `--no-cheating`, release/debug-assertions=no, compiler-builds=0, vstd-builds=0, experimental-drop=false and midpoint-runtime-proposal=false. Attempts001–012 and021 request `--compile`; controls013–020 do not. Thus “zero compiler builds” does not mean “zero fixture crate compilation.” It means no recorded rebuild of the compiler/vstd/harness in this stage. Runner source contains no build command on this path; metadata alone is not an independent process audit.

The final new authored exec/proof declarations count is 20: prerequisites2, bounds proof1, publication3, clients2, crash bridges3, positives4, negatives4, consumer1. This counts final declaration slots, not every revision of those declarations, copied model proofs, generated derives, specification functions or the six unchanged native methods. All failed/superseded source revisions are retained as per-attempt inputs; no additional verifier launch is inferred merely from a failed setup/audit/copy operation.

Clean final controls015/016/019/020 each have one intended backend assertion error, no frontend error, no recommendation warning and no ordinary warning. Their positives018 are 4/0. This provides historical local discrimination, not mutation coverage for all crash schedules, native errors or recovery branches. No isolated negative for every outer bridge premise was supplied or claimed here.

## Exact input/hash and artifact binding record

All digests in this section are SHA-256 of exact bytes. Aliases resolve as defined above. All 167 payloads named in `R/MANIFEST.json` were independently rehashed and size-checked, including all 21 `input.rs`, `invocation.json`, `receipt.json`, `stdout.txt`, `stderr.txt`, saved runners, caches and surviving artifacts. Their **exact individual path/hash/size mapping is the referenced immutable MANIFEST itself**, bound by its full digest below; it is not replaced by a query total. Every payload is a nonsymlink regular file with mode0444; MANIFEST and SEAL are the only files outside that payload mapping. No missing/extra payload or payload hash/size mismatch was found.

For a second inventory commitment, sort full input paths lexically and concatenate `sha256 + two spaces + root-relative path + newline` for every file, then hash the UTF-8 bytes:

- All169 files under R, including MANIFEST/SEAL: `95a43fb5f03642fcd1124a1fda21e0c0fddf4e6a66b083509651e52588c1dc0b`.
- All48 files under N: `ef408b0e8f69f07aec05c8eb87890d3bffe2e3c56d7382dff372e671a92460a8`. Each individual value also matches `R/recovery.json` → `repositories.redb.blobs["src/" + relative_path].sha256`; that hash-bound map supplies the exact names and digests without duplicating 48 rows here. Hashing other src files is not semantic review of them.

### Primary frozen package inputs

```text
R/MANIFEST.json  7f8a481f8660acac17cf858985266159b57f3c2487cc2c944618c0a4d05173e4
R/SEAL.json  9514053067d06dd127ea09bff783f0d908d3547e1988f9ac2f3d84b7b4b762f3
R/PLAN.md  791ee9fee584f18275ac220b92c992281a7bea50ef7fc48f2a607e289368b021
R/REPORT.md  e6705d9ea098c09f1d6da7a04cb54fcded47b877cb8e27a40b3d4018f0d891b4
R/selection.rs  b5818a0c3d6207ec59d2c791d2b4a888ef9e84e252ee1a82d193e3c477bb9bec
R/publication.rs  bb97a70407014f07467fb0ebc460c26f849b8a3b5db62e04fdc4629402612da9
R/crash_prefix.rs  515318455d06b506818c887749f67c4b2b544a79b6ae99dabca6c3df9beb38b7
R/clients.rs  24b316bedd009bad1ab4b82dcbc097fbc2dbd975b6b2f5833ac1916511e37686
R/prefix_consumer.rs  243f927f9546b6044af8755505b9320a3e61e4807d155ce0a12f45ad46848f13
R/controls_positive.rs  92369d6c05ae38a1b261b776fe6ea30640f1347d3b8ecc807121f542820bb3f1
R/negative_two_phase_newer.rs  b84d7811fda4858cb5c94d5e71ce86b0d6a8b14f5a86ac9107bebfd6feff4c6f
R/negative_two_phase_corrupt.rs  03627ba4c91d298abf06f9a7f954b5bebd9b56e9b7fe7895e8541c4898b1eeb5
R/negative_checksum_fallback.rs  8ea59b8e5a6a4ed91336e88c1e7f1ea91b7b1161b029a61f5cdb3f6013a938a2
R/negative_checksum_tie.rs  3df3ffb870f4f417e6dbec17f6ec0b9048649ecbc0747c7bcc87f2845fb3b655
R/prerequisites.rs  47d4b5a083df835c1dc34fe6eeb6ba5bfed1b1dce364a38b2cf5ecddcabfbcca
R/source-correspondence.json  53b0ce6dbbf876d311e32d91436f5ed4d5299a93fba5cfd1f71162e83aa6cc07
R/launches.jsonl  ae56db762ac6e38747dd0f0de5795651c4fb55c3bd26915fdb32fde6e55c3be4
R/recovery.json  68c5020788d8ed6c0121f8b203c27b27ea710e020c667f51444330fc27895877
R/preservation-initial.json  5691db4633898a3302fc30f782246e5936f286f0abd29aeaa3bbb5e873f00421
R/preservation-before-consumer-exit.json  a8fff806d795e8279c5eac4da6254399116ab1ef09f94726e2bb40c8bc393660
R/preservation-final.json  a8fff806d795e8279c5eac4da6254399116ab1ef09f94726e2bb40c8bc393660
R/audit.py  a207e51b7aedac3b4bbe8d13bc34400e2c5431e7c5afb824aafdeefa309d60a5
R/check.py  20281ef17d960bb15aa4ab886ff79fde58c60771277915bfbf2d592ebd714bda
R/run.py  aee1d96e07917df4ea0bd602ab3ed0adfb1fb92c457e8a390d18ef4e45432ee3
R/seal.py  cdd8b8f776130bf2fbfe3718b702b577f56fd62826ff92c7288e10a768f4dca2
```

SEAL's manifest digest equals the actual MANIFEST digest, payload count is167, state remains `READ_ONLY_BOUNDED_REFERENCE_PENDING_INDEPENDENT_REVIEW`, and all-eighteen-cells is OPEN. This review does not rewrite that historical state. The seal is an internal content binding, not an independent proof certificate or signed execution-time attestation.

### Parent/native/dependency source inputs

```text
P/evidence/redb-crash-consistency94/GOAL.md  74caefc4c6ea3049e6b8857d01fe1eb4abb834bd1029ef1c7db24b9d903ee263
P/evidence/redb-crash-consistency94/SCOPE.md  b2b4bf90ccb3d6aaeecf1b24a29f1e1e1fc7bd84a9f910ced13e45051d835285
P/evidence/redb-crash-consistency94/STATUS.json  2c6e2838e1c2dd27cd9592cdc011efdb173e340d929588e7246953e89f1cc331
N/tree_store/page_store/header.rs  35863472a48f9e0eb65a0f9ba0b2517a888a881478476aaafbcbfc77842850f9
N/tree_store/page_store/page_manager.rs  b062608a527c3f53aa505af870f2b803da3fa7b095169194c681cc6441cb558a
N/transaction_tracker.rs  423252fc12806ff37aed5a61ab9f0217bf35a2c2724dc40f9930dc94dbf3519e
N/tree_store/btree_base.rs  c3a1f5ca0cefdde0bc7d1b40536bc4f2d68a8bce852115433621a8b37fb9ec60
C/source-runs/attempt-039/input.rs  32ba044097da816fccf25e3e3f186760eae0d09df43dde5336ef77419f8618b3
C/source-runs/attempt-068/input.rs  ca15357f236a1918789136ca7fe302cf769bcc9ab658e28e4ec1312f37dcc68c
P/source/tools/redb_crash_consistency94/persistence_model.rs  a49afd5f74929ac729f3c5eada69de8927518f5e509fbddeeaac620002cb48fc
P/source/tools/redb_crash_consistency94/header_order_model.rs  41ae59a8439bd4e59a3d4dff720243f9ad5917f9fd115b33cd0099f397a82d68
P/source/vstd/bytes.rs  4decb7a30b140817bd472d84ebc6e4d44c15f1090d4508a0d454b9b6abfbb8d0
C/codec-runtime-005/runtime-binding.json  9fb1c329109511e40f042ec50c3ad8062ea2681815b671070a7a5a52f4469165
/private/tmp/verus-review94-byte-step-20261105-UyVDNcAR/preservation-baseline.json  e5006a5e48cef5316dd506606f5a24603c8930dc1827129c92b262136d0f1540
```

The two model authoring sources equal attempt008/009 input snapshots. Only their needed storage definitions and header-order lemmas were followed semantically. The historical sysroot baseline was read solely for the runtime preflight dependency; its unrelated evidence was not adopted. Existing native redb AGENTS instructions were read; their testing rule explicitly exempts report-only analysis.

### Selected producer/consumer artifact bindings

```text
R/prerequisites.vir  1a16e5bf6ad359dffc0168c9a6514cc257a42c154f03a01b8fdcf5cdb605266c
R/selection.vir  1e406532f5e3d0af7825f51f2ebeebbe8742095500c6be62bcc57bce80fc67e9
R/runs/attempt-003/librecovery_selection.rlib  d5f3c444b09d35df40dadbe2204ba933eec45384f1b794a50b5aaa87578c2ce1
R/publication.vir  1331d452eb65d8b1b4f38a5b62752f7223789b24ef0c8578bc89c80323b346b7
R/runs/attempt-006/libpublication.rlib  4656eacacd33ed8187dd5ecb528fcf5e1d2076b36eb1956beb08f2697d7812f6
R/persistence94.vir  cfd860c009ceafd6d140efa638b47198e8a73ba93b3ecb0891bee79c47b453a8
R/runs/attempt-008/libpersistence94.rlib  8d343051a08ec32a6ba0e2b092e2e7964f558580cf4737822c6e7d4ab677abd7
R/headerorder94.vir  013c1ec8d6727a53b6521a781e12ff812f7f161b935f48a57c80805c54981c45
R/runs/attempt-009/libheaderorder94.rlib  c3314f2b3d585ebbe449bdfbfc4e89a4d1c94a101083b1f7f44cd142b21eb144
R/recovery_prefix-final.vir  625d5044a96bb5dac9581ee6e18b2f96c900b0c63fd7eb8270184e5b5cef7610
R/runs/attempt-012/librecovery_prefix.rlib  ea6eb8b7080663a2a7d692fd0d01e75c049c9e2df7a6f2ed04b3bcdf925188ac
R/runs/attempt-010/librecovery_prefix.rlib  0f6e5234b5cc4d26d0bf4d5ce3049dbac5910a55e97616a37dbb6d9c081e4c84
C/source-runs/attempt-039/pagecodec94.vir  05190192ba4481404598177e57564bb148807baa3cc22bf0e166f4c349068252
C/source-runs/attempt-039/libpagecodec94.rlib  67e9b43dedcc4c0eebfa750582f9a492662403c23eabfc082f1a245c24c5f071
C/source-runs/attempt-068/rootcodec94.vir  01cb6dff8df3ae0dc2220abce98573c9a4c9c7b2383a4efbbe1cf7af02f874ba
C/source-runs/attempt-068/librootcodec94.rlib  8f76bb317b9315c24d9af99be35cb79c6343383bb30f304e1565d37fd37a5a90
```

Receipt010's missing scratch VIR is intentionally NOT listed as a checked existing artifact. The eleven final sources match attempts001/003/006/007/012/015/016/018/019/020/021 as appropriate. All extern/import bytes in every invocation match their recorded hashes, except superseded authoring-path source bytes, which match that invocation's archived input instead. No stale draft or scratch export was used in the final021 binding.

### Raw per-attempt receipt identities

Paths are `R/runs/attempt-NNN/receipt.json`. Each receipt's exact stdout/stderr hashes were checked against those files; each invocation equals its corresponding launch row, including argv/environment/runtime/input hashes. The hash-bound MANIFEST supplies all individual raw log and invocation hashes.

```text
001  0ecfac2bc4a69525cfa2ad1b095ab38f8a5d5dd7ecc301be7e007734fdb40353
002  23987b57a86c9d8461dc1a409390637cc2321465e93e0c4f99fa1bebb222e063
003  bc4839810926a031c965aafde699998b46da33cddfa20459c52c369abeeba9c1
004  ec34f36b9be5deee1c7168b63abad14e1a42f7e038a0aa573aa4ce70448e4725
005  70c118202cdbd2e8273b92428475943a4570c187d16f6745a14e15df24dc7aed
006  d6e5b45790ecd93da811073923e3a8ce1c0ff8e1fc50909696547c4d14dc6123
007  221441c9c278fb49cf671eff81f3f7db25f3b562fcab1cab63c60cf5a169fc7e
008  645f0b7d8ec79b421f6b79a2c755ec98dd2e03b055be953c0d00aa748f4d823f
009  5f9f788bf7a4924f4ed55bfbccebc586b4b3e87f96e3c4384f1d63cf4eb4797e
010  d736b6a1d99505c8274abdb46779b4774e9446f8789e43e65718567789f4ce26
011  e1f63fdc871c302b292fe5443a6cdc07fbdf9adbc31b7f71318ccbfbecf9a3f0
012  a98f68726b119f7e1c2bf160baa09dd674ef212a56370a999c4c2f5c62dfa15a
013  c4fdb1965a530c9c3b37758ab723052588001faa43e8c32292d441919db3d815
014  da7d748b0c50abd13e8d56f213c279949b5b449adebe91a7fe8cb6d79339b66c
015  fbaffd5ef65a46ccd6e350fe72b563dbd0d432f162cafd9e6a979e6d1b6880a5
016  dad9ada122b2464bb3ad94a3e8edbd185480df1dae2a9db20b0f2f924addb976
017  09605cd9c0e9e18228bc8a116b260e74a245d1325866f1cb96f5ec58b44217fd
018  836753a50572b33fef1128320d8cebef65c4f5402dfb1265b092322bcde0ce83
019  41164a7cbc6b490a55f36b17f905543172b104952a7be3ae4a7bfe85c802a8cf
020  306c31bd8e9e7ef1440a242720323333f4be8d28fd78c17a13db57b309dd179b
021  0146d2188f0f9f6c2af71664d25c65a5598a31abbd7ff038897c22d5cb605076
```

### Runtime and selected sysroot bytes, read/hash only

The first nine paths are relative to `C/codec-runtime-005/runtime`; the final six are relative to U. No executable was launched.

```text
verus  da0aad5ec3359b3037f522f3153570242a65529d8696b2b7209fd48599046628
rust_verify  7596883c92ef9a8eecc298ebbd9106b564cc9baef6e9f54de40f4f87c56e4395
z3  edae32f9e37ea4b5bb35310d72f0e352d0dc07626cac4e9e30bc1ea9a5bc8efb
libvstd.rlib  6387a503118e1de4de23aaa9597ae2ac51803dac95db710ae59f639cb156880c
vstd.vir  8c5283b51d515a4fe177664566149f6f790ae895a6c99a7d8c31781cbf38617e
vstd_build  14a7df403de736cd53e507a380dd1a83d3603d7fab1f0c4b319c4a952bfab306
libverus_builtin.rlib  941acbc2b162f5d94508af7dc3123ba8b427a22582ce105eafc21fe36ead2935
libverus_builtin_macros.dylib  5292f70af0af16cac8ae67f569a6c564e897033a64d72836174b2e14bfc68dc9
libverus_state_machines_macros.dylib  01bd79661de4e3a0a625abdbdb2eae0ce50ab69349ca467b07c31995401e6aaa
bin/rustc  1ba7bb8e273bd64125f4048d2ecce530e0a6470e565ab1b3aecf0b82b532f7b1
lib/libLLVM.dylib  2a8a4612d921ac0bd6be3b7b601f9a71d93b8cdf0a23f36d4c2df15cb616c981
lib/librustc_driver-451a19fa7a19fc36.dylib  ebf983c7473eac96e90a4201ca969801ac53a3cf4abc8f27b0021d53fb148395
lib/rustlib/aarch64-apple-darwin/lib/liballoc-2ce5006d2257bd69.rlib  36ff6b263aced406c2e8026cf5ee40d48182683d5e9e5ad6deeb41f3b351612e
lib/rustlib/aarch64-apple-darwin/lib/libcore-3e4eb39a5fe4e537.rlib  0205c70e6755f55478c511adb5231ff6b10cbd46f4198f0991354d0b20c702a4
lib/rustlib/aarch64-apple-darwin/lib/libstd-c8df3f17af30326b.rlib  7421245549290b5b40d81ebdd6dc294a2003b946bebb1ed16d614336226d1dca
```

## Final barriers and handoff

| Required final property | One-phase | Two-phase | Mixed-mode |
| --- | --- | --- | --- |
| Crash atomicity | OPEN | OPEN | OPEN |
| Acknowledged durability | OPEN | OPEN | OPEN |
| Preservation of recoverable state | OPEN | OPEN | OPEN |
| Recovery including interrupted recovery | OPEN | OPEN | OPEN |
| Error correctness | OPEN | OPEN | OPEN |
| Logical key/value refinement | OPEN | OPEN | OPEN |

Native correspondence/dependency closure, trusted-boundary acceptance and independent final full-scope review remain OPEN. No ordinary admission/publication readiness, S1 adoption or resource proof credit is granted. No parent source, evidence, checkpoint, fixture, global trust or Git state was modified by this review. The only new deliverable is this report in its fresh private directory; no follow-on task is launched or authorized.

Exit validation: all246 baselined files were rehashed. The169 frozen package files,48 recovered native src files and29 other inputs are unchanged except the two live parent context files disclosed in EQ5:244 unchanged,2 changed. All91 printed path/hash rows match the initial captured bytes; every argv extern/import is hash-bound. Report size is below40960 bytes. Available disk at the exit check was135712079872 bytes (126.39GiB), greater than16GiB. Initial available disk was130581000KiB (124.53GiB). No report-only code/test execution was substituted for a proof replay.

**Recommendation:** retain checkpoint008 as qualified bounded supporting evidence, with EQ1's selected-slot wording and EQ2's final012-only binding. **Minimum concrete next action:** the parent records those qualifications without replay or integration, and keeps the actual native bytes-to-header/flag/accepted-payload correspondence as an unmet prerequisite rather than an oracle. All eighteen obligations remain OPEN. Stop at this report handoff.
