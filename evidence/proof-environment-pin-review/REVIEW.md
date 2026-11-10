# Proof environment pin: independent source-only review

**Scoped SOURCE-ONLY acceptance: C1 is closed; no new actionable finding.**
Reviewed `fa63c1300c9f2eb84873ce30f64f5b7ea887e25c` from prepared checkout
`1f465e1d9fd236135d6aec910c8a1b82bebc7942`, against C1 in
`6deca82c:evidence/model-heap-initialization-review/REVIEW.md`.
All six changed Python files remain byte-identical to that fix. Only this new
evidence directory is changed. This accepts the environment-pin correction for
the selected active freezes, not the full goal or a new native theorem.

## C1 reasoning

`acceptance_inputs.py:35–40` resolves the requested path, looks it up in
`guard.initial`, and requires its **`sha256`** to equal
`trusted['toolchain']['GMP_packages']['sha256']`. `ArtifactGuard.snapshot`
actually records `sha256`, size, mtime and inode; there is no `source_sha` field.
A missing snapshot or unequal digest raises `ValueError` via explicit `require`,
including under `-O`. A stable replacement cannot pass merely by staying stable.

The expected digest is the existing
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
Its source, `mutable_buffer_goal/foundation-baseline.json`, hashes to
`cb2a9217ec1df37630442dd27fe4290e2845a1176aef5e0aa366d164d6443dcd`.
Every active proof/inspection freeze pins that metadata and makes it mandatory.
It is not an unauthenticated, freshly measured expected value.

The pinned `run-rocq` hashes to
`bffce97908442f9af5a279d7dda96682c55ef8fbad525b7cc0f7efdf71a89b7e`.
Its source reads `gmp-private/manifest.json` below the literal toolchain directory
and prepends every `env_prefixes` entry before `opam exec`. All three selected
caller paths resolve to exactly that manifest:

- `native_semantic_bridge/check.py:78–87`: derives it from the authenticated
  `trusted.readiness.path`, snapshots it, then calls the helper. Mandatory
  inputs are checked at 42–47; retained public/foundation identities and wrapper
  identity at 91–116. The first prover command is at 129.
- `native_source_entry/entry_audit.py:116–119`: derives it from `f.ready`, whose
  active frozen path is the same toolchain readiness marker. Required public
  and private inputs are enforced at 60–79, retained identities at 120–147;
  the first prover command is at 182.
- `native_heap_initialization/check.py:65–76`: snapshots the literal manifest
  under `BASE`, then calls the helper at 69–70. The exact mandatory input set
  is enforced at 50. Loaded JSON bytes, including trusted metadata, are matched
  to the retained snapshot at 72–76; dependency/tool pins at 77–96. The first
  prover command is at 129.

The pin calls precede some retained metadata checks, but **all checks must pass
before any prover command**. A failure cannot bypass metadata authentication.
The helper and its transitive local imports (`proof_checks`, `infer_experiment`,
`infer_eval_support`) are independently mandatory in all three inventories;
the inspector obtains both shared support files via `source_audit_inputs()`.
All remain guarded through command and final checks.

Pin/identity failures leave the bridge and heap model-proof flags false,
native flags false, effects discharged zero and publication false. The inspector
never promotes a model/native proof flag. `replay_receipt` starts success false
and persists exceptions as failures. Bridge status promotion still follows all
guards through `accept_model_result`; heap promotion follows its final guards
at 149–152. These are source conclusions, not newly executed refusal trials.

## Active freezes and controls

Authenticated all current frozen public inputs: bridge **24**, inspection **16**,
heap **12**, controls **14**, plus the inspection's **six** private inputs.
Exact freeze/file hashes are in [checks.json](checks.json).
The active files are `native_semantic_bridge/gmp-pin-freeze.json`,
`native_source_entry/entry-gmp-pin-freeze.json`, and heap
`gmp-pin-freeze.json` / `controls-gmp-pin-v2-freeze.json`.
Older freezes remain historical; their old controller identities are not claimed
to replay with today's code. No `.v` changed in the fix, and all three selected
candidate directories are unchanged from `6deca82c`.

`native_heap_initialization/guard_controls.py:16,28–31` selects `--proof-freeze`
(default: active GMP proof freeze), resolves it before `relative_to(ROOT)`, and
requires the control inventory to equal its proof inventory plus the selected
proof freeze and driver. The active v2 freeze hashes both correctly. The source
still enumerates three provenance refusals in each Python mode, requiring an
empty prover-command list and the intended failure diagnostic. These are not
the five previously checked resource countertheorems.

The retained first control attempt hashes to
`9cbc8814c52f73087aceb0301592dd8f55171b289939fab3c474490c73e1234c`.
It records the relative-path setup exception, success false, no commands and
zero semantic negatives. It supplies no semantic evidence. The parent-reported
six successful controls and 11 helper tests per mode were not rerun here.

## Completed evidence authenticated, not rebuilt

Authenticated these completed receipts against the committed parent hashes:

- Corrected heap `gmp-pin-heap-positive/receipt.json`:
  `a3aea2cd291724a26f76082739455cbd4cfa8b64b3d3dcb465a465ed5c6d739d`.
  Five successful commands; all 29 assumption entries closed; all eight retained
  source/object products match their receipt, and all four objects match the
  original parent record. Its input snapshot pins the current trusted metadata,
  wrapper and manifest. Model checking is true; source entry/publication false.
- Earlier completed batch `model-source-status-validation-0e1c7081/receipt.json`:
  `ce90a7f12ac6e17a39168b97e2709d9343b74f7588d2d27db72d4baf46176f46`.
  All 13 commands and recorded log hashes pass; before/after snapshots agree.
  It records 19 units per mode, 12 source / four readiness / three acceptance /
  ten identity controls, 18+6 models, 25 assumptions and 42 inspected constants.
- That batch's model receipt:
  `4b877d40b9c8ac8af344c3e72f0c91590c55ad0a566288f8c5f43dbb3ac905fb`;
  inspection receipt:
  `714b7397df1b483322e3fc512ac11067ed5df352ca963892807ea33917f4d396`.
  Both retained snapshots already contain the required manifest digest. All ten
  model products and 90 inspection outputs authenticate; the four model objects
  match the original accepted hashes. Native/import/publication statuses remain
  false, with zero discharged native effects.

Also authenticated the older model receipt explicitly pinned by the inspector
and read by `RecordedSources`:
`790aaa8e2eadd1a668bef67cdce938dc3048318d85ee753057cfb805e7982e41`,
including its ten retained products. Thus the new condition is satisfied by the
actual completed historical model/inspection snapshots. This is not a claim
that the two changed controllers were freshly replayed under their new freezes.
The earlier source-only acceptance `e3b50f29` of F1–F3 is preserved.

## Reviewer checks and limits

Executed only the two permitted read-only `PinnedEnvironment` methods
(`test_actual_reviewed_environment`,
`test_environment_must_be_in_the_retained_snapshot`), four `RecordedSources`
tests and six `Assumptions` tests: **12/12 normal and 12/12 optimized**.
See [normal](pure-normal.log) and [optimized](pure-optimized.log) logs.
Performed 11 in-memory syntax checks, Git identity comparisons and read-only
authentication using [audit.py](audit.py); its exact results are in
[checks.json](checks.json). The first audit attempt assumed split batch logs;
the completed batch uses pinned `.log` files. The reader was corrected and passed.

Read the test code, current reports, parent records and TARGET. Did not execute
`FinalStatus`, `RequiredSupport`, `SourceBindings`, the manifest-writing test,
artifact-control scripts, concurrency/substitution experiments, Coq/Rust builds
or installs. No shared-baseline/implementation edits, subagents, pushes or PRs.
No active typed-model-input parent outputs or other worktrees were inspected.

Canonical remains inherited **6/0** over every `u64` and arbitrary initialized
`[u8;10]`; full frozen HMAC prefix **and suffix** remains **1/1, unproved** over
arbitrary initialized `[u8;128]` and all valid key lengths, including 0 and >128.
Checked model 29 and separate model-input 26 do not establish actual Rust typing,
provenance, ancestor loans or C/F interpretation. Model-input 26 and upcoming
poststate mathematics are outside this review. Zero native correspondences,
44 minimal plus three phase obligations and 101 effects remain open. Shared
checked assistance across both full properties is unconfirmed. Publication and
full-goal completion remain **false**.
