# Compiler-validation114: one independent source/receipt review

## Bounded verdict

ACCEPTABLE WITHIN THE AUTHORIZED SOURCE/RECEIPT SCOPE, with one nonblocking
documentation observation below. No remaining P1/P2 defect was established in
the three revised source repairs. This is not adoption, native validation,
runtime capture validation, a compiler-binary build, or correspondence proof.
The original revision1 review remains historically CHANGES REQUIRED; this report
does not edit or retrospectively approve its old candidate.

Compiler-validation114 receipts support one successful full rust_verify package
check and one successful filtered five-test library-harness command, not more.
The five tests are pure origin-status metadata tests ONLY. All18 remain OPEN;
source path FAILED; Git continuity UNAVAILABLE; compiler/source correspondence
UNESTABLISHED; native0/trust0; conditional/unadopted TCB unchanged; goal service
last observed BLOCKED. No automatic additional review/build chain is authorized.

## Findings, priority order

1. P3 / nonblocking documentation observation: both raw stderr streams begin
   with Cargo's failed last-use-data persistence warning and readonly-database
   error (command-1.stderr:1 and command-2.stderr:1; underlying error at :4-7).
   REPORT.md:121 discusses the two Rust dead-code warnings but does not separately
   summarize this Cargo warning. Preserve the distinction in downstream summaries:
   compilation succeeded despite nonfatal cache-bookkeeping write failures; this
   was not a pristine warning-free run. The full raw warning is retained, not
   suppressed or relabelled as compiler success/failure. It does not contradict
   either actual child exit0, establish a compiler/setup failure, or justify a
   retry. No candidate edit or extra command is requested by this observation.

No blocking finding was established. Receipt-level process-group absence and
fresh-target claims have the documentary, rather than OS-trace, bounds below.

## Revised source findings resolved, with limits

All source anchors in this section are relative to candidate snapshot/source/.

- Original P1, eager glue resolution: rust_verify/src/checked_normal_drop.rs:137
  scans drops using body data and captured maps, without TyCtxt, resolve_drop_glue,
  ADT destructor resolution, projected-type query or instantiated-body lookup.
  The refusal branch at :534 precedes normalize and stores Err for captured sites.
  The inherited drop_impl at :356 remains reachable only through the unchanged
  normalizer, not this new raw refusal helper. This removes the specific added
  eager glue query; it does not prove formatting/allocation totality or runtime
  absence of compiler query cycles in every path.
- Original P2, incomplete identity/incorrect successor labels: :166-167 explicitly
  state NOT_QUERIED and no resolved instance/shim identity. No incomplete pair is
  advertised as concrete glue provenance. :169 records replace_assignment_origin;
  :170 records async_cancellation_cleanup_target; :172 records the separate unwind
  cleanup target. These are diagnostics, not actual destructor/body identity or
  an execution-path theorem.
- Original P2, missing versus empty: :94 distinguishes missing, incomplete,
  completed-empty, completed-no-local-match and completed-with-local-matches.
  :140-141 retain both Options; :161-164 classify them before the optional vector
  becomes an empty diagnostic list. Missing bindings do not establish anonymity.
  rustc_mir_build/src/builder/checked_drop_origins.rs:98 explicitly completes an
  empty capture at Builder retention. The binder map is inserted at :96 only at
  retention; temporary observations before that remain incomplete.
- Builder observations preserve real typed Scope and TempLifetime inputs:
  checked_drop_origins.rs:21 and :29; expr/as_temp.rs:47 and :132. Scoped recursion
  returns the same temporary, fixed-temp reuse remains, and observation append
  does not replace drop scheduling. Multiple observations are retained, not
  promoted to one occurrence. No proof connects these Builder locals to an
  elaborated MIR occurrence, resource owner, AstId, destructor or lifecycle.
- Separate global locks, native flag reads and phase resets are not an atomic
  session/epoch snapshot (:36-40, checked_normal_drop.rs:197-211). No repeated-run
  isolation, overlap safety or concurrent-session theorem is supplied. The scan
  may conservatively refuse cleanup/ghost/unreachable unsupported sites. Neither
  this review nor the metadata tests executes actual Builder capture or MIR glue.

Nine legacy function byte slices independently match the frozen baseline:
supported_type, drop_impl, normalize, account, rewrite_block,
checked_reference_frames, prepare, check_native, elaborated. Candidate anchors:
checked_normal_drop.rs:222, :356, :424, :566, :682, :831, :896, :974, :1000.
independent-crosscheck.json records individual hashes. This is targeted textual
identity, not AST equivalence or proof of inherited admission/correspondence.
No new trust/admission/assumption change occurs in the exact four-file delta.

## Actual compiled inputs, configuration and outputs

Candidate C = /private/tmp/redb-compiler-validation114-worker-20261108T151020Z.
Revision R = /private/tmp/redb-lifecycle113-origin-revision2-20261108T145432Z.
Frozen F = /private/tmp/verus-crash-consistency94-20261105/drop-effects-parent-002/staged.
Historical review H = /private/tmp/redb-lifecycle113-review-20261108T144703Z-dXq3MG.
F, not the changed live parent, is the source baseline.

Independent whole-tree hashing found 1,095 frozen and 1,095 snapshot files and
exactly four changed paths, no deletions: source/rust_verify/build.rs plus the
three authorized revision2 paths. The full unified delta was independently
regenerated in memory and matches complete-source-delta.patch exactly. The three
overlay bytes match R/candidate exactly, and the historical handoff and frozen
three baselines match. source-correction.patch is empty. Both before/after source
inventories match the current snapshot exactly and independently recompute to:
c9cb297436ab24053c254ed95bd4e520ef6fca5b86832d217b37302cb5ad7b23.
Historical Cargo.lock is 30cd48d355d863cc714c0768bb1b69adb618d77fcd395c916acedb8e63456bc4.

build.rs:8-24 explicitly uses unversioned snapshot metadata and bypasses eager
Git queries only in that mode; supplied version/SHA overrides cannot fabricate a
Git identity in snapshot mode. Actual retained build-script output in BOTH fresh
targets emits profile debug, version unversioned-source-snapshot, SHA
unversioned:snapshot-sha256:c9cb297436ab24053c254ed95bd4e520ef6fca5b86832d217b37302cb5ad7b23,
and toolchain 1.99.1-aarch64-apple-darwin. This digest is source-byte identity,
not a commit, bootstrap attestation or proof that the installed compiler derives
from those sources. The adaptation is local metadata plumbing, not integration.

The unchanged source/.cargo/config.toml supplies proc_macro_span,
procmacro2_semver_exempt, span_locations, verus_keep_ghost, and inherited Cargo
environment settings. Both actual relevant rustc invocations carry all four cfgs.
Explicit Cargo target/build dirs override historical target-verus. Both launches
use the direct installed 1.99.1 cargo/rustc paths, offline/locked, jobs1,
incremental0, debug-info0 and distinct target-check/target-tests. The receipt,
launch, scope and ledger argv/cwd/environment fields agree independently.

Slot1: cargo check --offline --locked --jobs 1 --package rust_verify --verbose.
15:13:08.687506Z to 15:14:01.891616Z, 53.203171625s, child0/EXITED_ZERO.
Raw command-1.stderr:378 checks rustc_mir_build/src/lib.rs; :381 checks
rust_verify/src/lib.rs; :439 checks rust_verify/src/main.rs. All emit
dep-info,metadata: checking the binary target is NOT linking a compiler executable.
Command1 has 228 direct Cargo verbose invocations; stderr440 lines, stdout0.

Slot2: cargo test --offline --locked --jobs 1 --package rust_verify --lib --verbose
checked_normal_drop::origin_capture_status_tests:: -- --nocapture --test-threads=1.
15:14:31.474817Z to 15:15:41.009236Z, 69.533342041s, child0/EXITED_ZERO.
command-2.stderr:351 compiles rustc_mir_build; :357 compiles rust_verify lib with
--test; :416 launches only the filtered harness. 203 direct verbose invocations;
stderr416 lines, stdout10. Independently parsed stdout yields exactly the five
scope names, all ok, and exactly 5 passed / 7 filtered / 0 failed or ignored.
Source has 12 ordinary test declarations: these five, five inherited checked-drop
tests, one import_export and one tuple_clone. The seven were filtered, not passed.
The linked library-test executable is not a working verifier/compiler engine.

Both raw stdout/stderr files were read completely from disk and hashed against
receipts, with complete direct-invocation comparison and diagnostic extraction;
long terminal display truncation is not mistaken for complete on-screen capture.
All 1,447 check and 1,387 test artifact-inventory records independently match disk.
The sealed target bytes were all authenticated; neither binary was executed here.

## Receipts, failure honesty and provenance bounds

Reviewed REPORT.md, COMMAND-LEDGER.json, RESERVATIONS.json, both complete raw
command streams, launches/scopes/receipts/resources/source inventories/artifact
inventories, actual-command-scopes.json, metadata outputs/adaptation, selected
results, dependency closure/inventories, final reconciliation/VALIDATION, worker.py
and audit.py, R source/validation/report, and H's original findings. Whole-payload
authentication includes retained source, evidence and targets, not just selections.

worker.py:294 checks reserved slot identity, rejects an existing launch receipt,
requires a fresh empty target, and writes scope before launch. :86 manages a new
process group, records real child returncode, preserves timeout/signal/resource
failure status, kills surviving descendants and finalizes receipts in finally.
audit.py:129 refuses success unless both exits/statuses/groups, source stability,
five tests and reconciliations agree. Reservations predate staged setup according
to the retained records. Exactly two compiler-producing slot receipts are present;
all 14 managed receipts report exit0. No failed/cancelled compiler slot or retry is
present in this evidence. Exploratory absent-path/glob failures and truncation are
disclosed in EXPLORATORY-PREFLIGHT-NOTES.md; the nonfatal cache warning is above.

These observations are not a complete process trace: Cargo verbose does not show
every OS process, native dependency helper, shared-library load or escape from a
process group. No independent historical process monitoring was performed.
Freshness, exact launch count and absent descendants are corroborated harness/
receipt claims, not globally proven facts. The wrapper is not a security boundary.

Independent current rehashes match all 364 inventoried external package trees
(19,410 files), all 3,340 inventoried toolchain files and 13 selected platform
identities. The dependency inventory conservatively exceeds the actually compiled
closure; metadata resolution includes target/feature conservatism. Installed
source and tool-byte authentication does not authenticate Git history, toolchain
bootstrap, compiler correctness, all SDK/header/OS/dyld inputs, or actual execution
of every inventoried file. No runtime MIR/unique occurrence/glue/lifecycle/native
Arc/Mutex/LRU correspondence follows from these source hashes or compiler results.

Worker resources independently reconcile: 33 periodic samples satisfy recorded
8GiB scratch and >16GiB free limits; minimum sampled free is 66,629,582,848 bytes.
Worker final preseal allocation was 943,616,000 bytes. Samples are not continuous
monitoring. Review resources are separately recorded below and in SEAL.json.

## Entry/exit authentication and review accounting

Entry and exit SEALED-MANIFEST SHA256 both:
c73d2fba54d4144d3705d78a6d5d6cf9d29bf5eaeeb80dce6d9736fd001d6dd9.
All 4,064 payload byte counts and hashes match on both passes (931,698,725 bytes).
Entry and exit R/MANIFEST SHA256 both:
a586fe4665ee832499ea233969eb2914b941093186596f849d8cfeea9321c342.
All 21 revision2 inputs match on both passes (225,521 bytes).
All 1,478 recorded live-parent inputs match at entry and exit, with zero drift;
the parent baseline file also matches the retained copy, SHA256
155a7ad93b1d225268eea100c3262dcabbbfbc901ed1cb78cc34129c95ab60a3.
New unlisted parent audit artifacts are outside that fixed baseline identity claim.

First UTC clock: 2026-11-08T15:21:49Z; hard handoff deadline 15:29:49Z.
Authentication entry began 15:22:40.700528Z; exit began 15:27:17.983431Z and took
1.15304225s. Sampled review free space: 67,807,756,288 bytes initially (df),
67,796,029,440 at entry, 66,732,503,040 supplemental, 66,317,299,712 at exit;
all exceed16GiB. Final UTC/elapsed/scratch/free values are in SEAL.json.
No continuous global free-space guarantee is claimed.

No applicable AGENTS.md exists in the review/candidate/revision/frozen source
ancestor scopes searched. Unrelated discovered dependency/native instructions
were outside all authored file scopes. Review setup had no execution or
authentication failure. Several terminal display responses truncated; direct
complete-file reads, hashing, focused ranges and retained parsed evidence avoid
relying on those truncated displays. This is a display limitation, not a lost log.

Reviewer compiler/build/test/verifier/solver/native applications0; network0;
subdelegation0; trust/admission/assumption changes0; parent/candidate/frozen/tool/
cache/Git edits0. Only reading, hashes/diffs and fresh private review evidence
generation occurred. Every authored review file is created via apply_patch;
Python uses -B, no candidate helper is imported/executed, no tools are copied.
This is the ONE fresh authorized review. Stop at this handoff.

## Content manifest

CONTENT-MANIFEST.json lists each report/evidence/script payload with byte length
and SHA256. It excludes itself and SEAL.json to avoid recursion. SEAL.json hashes
that content manifest and records final accounting. Entry/exit authentication,
independent-crosscheck.json and supplemental-crosscheck.json retain machine
evidence; authenticate.py and crosscheck.py disclose the read-only methodology.
