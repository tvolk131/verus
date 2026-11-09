# One next task: native leaf admission guard -> builder/lookup contract

Source-only recommendation, November 7, 2026 (America/Chicago). No proof,
acceptance, or execution result is claimed. All18 remain OPEN.

## Decision

Select **native leaf fit/count predicates -> capacity/count premises of the
existing builder-to-lookup contract**. This is a conditional native-source bridge,
not another partition lemma, tree model or ownership shortcut. Retained imports
permit a verification-only candidate; frontend/solver acceptance is untested.

Leaf100's candidate/review is untouched; its supplied seal is
0d370a2c977c9db13f42ba8bfdb9bedb9dcd3b5f6323d9f391e7b4196e3c0bce.
This task does not depend on its added assertions or pending verdict.

## Exact source and proof anchors

Abbreviations below expand to these absolute directories:

- W = /Users/tommyvolk/.t3/worktrees/verus/t3-60978068
- C = /private/tmp/verus-crash-consistency94-20261105
- L = C/native-leaf-lookup-007
- B = C/native-builder-prefix-006
- N = L/recovered-native/redb/src/tree_store
- R = L/midpoint-runtime-proposal/runtime
- V = L/midpoint-runtime-proposal/source/vstd

Goal/ledger: W/evidence/redb-crash-consistency94/{GOAL.md,STATUS.json,TAKEOVER.md}.
Selected load-bearing input SHA256 values/full paths are in INPUTS.sha256.

Native pin c82ab865bb362bc4af067d332adc44978ff64813:

- N/btree_base.rs:1017 `leaf_split_required`, complete body through :1031.
- N/btree_base.rs:1034 `leaf_fits_one_page`, complete body through :1048.
  Its actual operations are RawLeafBuilder::required_bytes followed by
  `required <= page_size && u16::try_from(num_pairs).is_ok()`.
- N/btree_base.rs:907 `LeafBuilder::should_split` calls the former with real
  pair count, accumulated key/value sizes, widths and allocator page size.
- N/btree_base.rs:988 `LeafBuilder::build`, :996-1007 new/append/drop: NOT selected.
- N/btree_cursor.rs:1284-1303 uses retained_after_removals and the latter
  predicate when deciding whether survivors pack; this full caller is NOT proved.
- N/btree_base.rs:1119/1139 required_bytes/new, :1211 append, :1256 Drop.

Existing checked-contract sources, not fresh verification claims:

- B/reference.rs:18 `metadata(n,kw,vw)` is
  M = 4 + (kw=None ? 4*n : 0) + (vw=None ? 4*n : 0).
- B/reference.rs:333 `RawLeafBuilder::required_bytes`: requires
  M+payload <= usize::MAX; ensures result=M+payload.
- B/reference.rs:132 `new`: n<=65535, page length>=4, debug-only metadata fit.
- B/reference.rs:475/709 `payload_sum`/`part_views`; :714 complete_grammar;
  :763 `build_leaf_projection`: count1..65535, equal key/value counts, exact
  reserved key sum, capacity, u32 used-end bound, matching optional widths;
  ensures full byte grammar and every exact key/value, plus frames.
- L/consumer.rs:7 `request` spells out those builder premises; :23
  `ordered_inputs` adds four-byte strictly NUMERIC increasing u32 keys;
  :97 `build_then_lookup` calls the checked builder and native lookup. Ensures
  Some iff an input key equals query bytes, exact returned query key bytes,
  and the original associated value bytes. Query length must be four.
- L/REPORT.md, B/REPORT.md and
  /private/tmp/verus-leaf007-independent-review-tOKSrsOe/REPORT.md qualify imports,
  source matching, midpoint, missing ownership/traits and authored loop/adapter.
- /private/tmp/verus-native-leaf-fastpath029/REPORT.md: prior native eligibility
  results concern *in-place* replace/insert, not this constructor's admission.
  Reports026/027/028 already cover replacement/removal/insertion with lookup.

N/btree_base.rs hashes to
c3a1f5ca0cefdde0bc7d1b40536bc4f2d68a8bce852115433621a8b37fb9ec60.
Recovered bytes remain qualified correspondence, NOT restored original native
paths/Git continuity, AST/nominal identity or whole-crate verification.

## Deliverable and exact proof design

In a FRESH scratch candidate, copy the TWO entire native helper bodies above,
with contracts/proofs and visibility/result-binder changes only. Import
`builder_step::RawLeafBuilder`; do not invent a replacement builder or reimplement
required_bytes. Audit proof-erased executable tokens against those native ranges.
No new struct, trait, trusted specification, external body, or exported artifact.

1. Check native `leaf_fits_one_page(n,p,kw,vw,s)` with ONLY the machine-arithmetic
   precondition M+p<=usize::MAX. Exact result:
   `fit <==> M+p<=s && n<=65535`. Preserve n=0, n=65535 and n=65536 in the scalar
   policy domain; do not incorrectly restrict the helper to nonempty leaves.
   A true result implies metadata fit and at least four bytes of capacity.
   It does NOT imply nonempty count, key widths/order, exact payload totals or
   u32 representability when page_size can exceed u32::MAX.
2. Check native `leaf_split_required` under the same arithmetic precondition:
   `split <==> n>1 && !fit`. Derive `n>1 && !split ==> fit`.
   Explicitly reject the invalid converse shortcut `!split ==> fit`: a single
   oversized pair is deliberately not split and needs a larger native page.
3. Write ONE authored gate client `fit_then_lookup`, not called a native caller.
   Inputs: original mutable page, key/value slice arrays, widths, reserved key
   bytes, payload:usize, and four-byte query. Require:
   - n=keys.len()>=1 and values.len()==n, but NOT n<=65535;
   - reserved equals sum(key lengths); payload equals reserved+sum(value lengths);
   - M+payload<=usize::MAX and M+payload<=u32::MAX;
   - optional fixed widths match every supplied slice;
   - existing ordered_inputs(keys), query.len()==4.
   Crucially do NOT assume request, page-capacity fit, or the count cap.
   Call actual copied leaf_fits_one_page with s=page.len(). If false, return
   `(false,None)` without constructing anything. If true, prove the EXISTING
   L/consumer.rs:7 request, then call imported lookup_client::build_then_lookup
   on that original borrow. Return `(true,result)`.
   Postcondition: first component is exactly M+payload<=old(page).len() &&
   n<=65535; when true, second component has the exact existing Some/None,
   query-byte and associated-value guarantees. No result guarantee is inferred
   from false admission beyond None. This supplies formerly assumed request
   count/capacity facts from a real native predicate; all other premises remain
   visible. Do not claim native accumulator/page-allocator/Drop correspondence.

New: native packing policy -> constructor count/capacity -> existing image/lookup,
including singleton exception. Re-proving build_then_lookup given request earns
no credit. No exhaustive absence-of-other-evidence claim.

## Paired positive / semantic-negative design

Keep complete scopes, identical imports and valid domains in each pair.

- Capacity boundary: fit(2,8,Some(4),Some(0),12) is true. Positive proves true;
  negative asserts false on the SAME call: exact-fit equality, no bad precondition.
- Count boundary: fixed-zero widths, payload0, page_size4. Native fit(65535,...)
  true and fit(65536,...) false although required bytes are four in both cases.
  Negative claims the second is true/capacity alone suffices. No huge allocation
  or claim that fixed-zero keys are sorted u32 keys is involved.
- Singleton exception: n1,payload5,fixed widths Some(4)/Some(1),page_size8:
  required9; split=false and fit=false. Positive checks both. Negative asserts
  `!split ==> fit` on these same concrete valid scalar inputs.
- End-to-end positive: canonical LE u32 keys1 and256, empty values,
  exact-fit and spare-capacity pages; key width None/Some(4), independently value
  width None/Some(0). Use both present and absent four-byte queries. Call the
  gate client, not only a mathematical predicate. Also exercise capacity failure
  on a too-short page without invoking builder/new. Generic contracts, not
  these examples, supply the unbounded conditional statement.

Require localized intended BACKEND failures and clean paired positives.
Frontend/import/precondition errors, incomplete scopes and resource exhaustion
are not semantic negatives. Source mutation is optional within the same cap.

## Matched import feasibility and bounded execution plan

Use R/verus and existing 1.99.1-aarch64-apple-darwin sysroot at
/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin.
Import matched existing pairs (all present; 1,537,930 bytes combined):

- leaf_decoder: L/runs/attempt-009/libleaf_decoder.rlib + L/leaf-final.vir
- builder_step: L/runs/attempt-010/libbuilder_step.rlib + L/builder-final.vir
- lookup_client: L/runs/attempt-011/liblookup_client.rlib + L/client-final.vir

L/runs/attempt-011/invocation.json gives exact logical flags/environment and
dependency path; add lookup_client and its directory to --extern/--import/-L.
Reuse read-only artifacts in place: do NOT copy the ~125MB runtime or toolchain.
Run new verification-only source units with --no-cheating, --crate-type=lib,
--edition=2024, --num-threads 1, --output-json, --rlimit 60 and release assertions
disabled, retaining the recorded sysroot/environment bindings. REMOVE --compile
and --export. No Cargo, rustc, vstd/compiler/harness build or native execution.
New helper+caller source can share a unit; retained providers/lookup are ordinary
external imports. Release-only inherited builder/composition is not upgraded to
debug or independently re-proved by this task.

V/std_specs/convert.rs:68-88,:158-193 supplies TryFrom/usize->u16;
V/std_specs/result.rs:128-140 supplies is_ok: inherited assumptions, not new axioms.
Check those bindings,
L/midpoint-runtime-proposal/runtime-binding.json and the import receipt before
running. Do not replace runtime, rebuild it, or assume an unsupported call away.

Maximum 16 verifier applications, ONE worker, <=15min wall, scratch<=128MiB and
free disk>16GiB throughout; stop at the FIRST exhausted bound. Suggested ledger:
1 import-only control; 2 native helper/generic gate draft; 3 complete corrected
candidate; 4 full positive boundary/end-to-end witnesses; 5-7 three isolated
semantic negatives; 8 independent complete positive repeat. Applications9-16
are a hard reserve for bounded proof-only corrections, not promised work.
Count every launch/failure. No evidence replay or old accounting replenishment.
Store argv/env/source hashes, outcomes, diagnostics, scope counts, timing and peak
scratch. Stop on import/runtime mismatch or required engine/trust changes.

## Why this task, and what remains open

It supports ONLY a sub-obligation of **logical_key_value_refinement / one_phase**:
when this real native admission predicate accepts concrete accounted inputs, the
existing conditional byte builder and search can be called safely and return the
right association. It proves no complete one-phase history, commit, persistence,
error, recovery, two-phase or mixed-mode theorem; no cell closes.

Rejected alternatives: another partition/search restatement duplicates leaf100;
new replace/remove/insert lookup compositions duplicate026-029; native full
LeafBuilder::build and Btree::get_helper require real allocator/PageMut or
PageImpl/Arc/guard/glue/Drop boundaries. The former's minimal missing boundary is
N/btree_base.rs:1005-1007 actual drop(builder) -> legal original-page access/return,
including :1256 destructor completion/panic path, NOT an assumed release rule.
The selected guard bridge does not cross or erase that checked-untrusted-engine
boundary. No engine gap is identified as necessary for this smaller task.

Inherited: six LE boundaries, conditional private midpoint (not Rust-core proof
or adoption), std/Rust/solver TCB, comparison-only traits, authored loop/transfer,
qualified provenance. Ownership/Drop/panic/glue/I/O stay OPEN. No new axioms,
trust/scope/export policy or changed barriers. No growth049, zero-growth050,
mutation030, padding95/provider-refused padding or refused drop-pipeline work;
no old ledger/budget reset.

Assessment: reads/hashes and this fresh directory only; no network/Git writes,
parent edits/subdelegation or verifier/solver/compiler/build/test/native execution.
Missing-path reads were corrected. At 00:37:48Z free space was 57,244,196KiB
(initial 57,506,020KiB), above 16GiB; scratch was 16KiB. Started by 00:33:00Z;
final report/hashes remain far below 2MiB. No execution or acceptance credit.
