# Independent source-freeze completeness review

Base: `0912d2a70912cf69834e0755640563fb28d00892`; reviewed fix:
`bb5be3f0f729d075ca08fddc67d983dd25453850`.
Read the original `ed9fd420` findings, `1f727448` recheck, current semantic report,
TARGET and parent semantic/stable-key records. This review changes only this
new evidence directory. No subagents, implementation changes, live sibling
inspection, push/PR, Rust regression or Coq compilation.

## Prioritized finding

**P2 — a persistent change between freeze validation and the initial artifact
snapshot can still replace the extraction provenance binding.**
`source_map.py:154-159` and `audit.py:23-26` validate frozen hashes first, then
construct an independent `ArtifactGuard`. The guard measures current bytes;
neither entry point compares its initial hashes with the frozen expected hashes.
A binding changed after its validation digest but before that initial snapshot
therefore becomes the guard's accepted baseline. Its final check succeeds even
though the binding still disagrees with the immutable pretrial freeze.

Independent reproduction, from this checkout with a fresh private destination:

```sh
python3 -B evidence/source-freeze-completeness-review/recheck.py \
  --race-only --output /tmp/source-freeze-completeness-review-fresh-race
```

This runs both **unmodified** entry points under normal Python and Python `-O`.
It copies all seven required source/support bytes and the authentic extraction
into private directories. The pretrial freeze includes every mandatory entry
with its authentic hash plus one optional ordinary, immutable `ballast.bin`
(relative path; 128 MiB of zeros), last in validation order. Its first Linux
`IN_OPEN` notification proves all mandatory digest comparisons have completed.
An external writer then replaces the private extraction receipt with exactly
`{"success":true}\n` and changes the private binding to its hash while that
last digest is running. The pretrial freeze and both actual JSON outputs remain
unchanged. No controller/helper/interpreter hook, OS modification, byte
restoration, symlink or named pipe is involved. The changed binding and synthetic
receipt persist after the trial.

All **four trials exit 0 with `success=true`**. Each snapshots the altered binding
as a guarded input; it is the only frozen entry whose final hash differs from
its expected hash. `audit.py` also successfully runs its 12 source controls.
This is not a reviewed pretrial rebind: the recorded pretrial binding is the
original authentic one, and no extraction execution supports the new receipt.
Fresh measurement of the substituted bytes does not authenticate their origin.
The optional file enlarges the scheduling interval for a deterministic repro;
no such file is necessary to the ordering defect itself. This demonstrates a
caller-supplied complete-freeze issue, not a false result for the immutable
stock inputs used in the healthy replay.

The integration lead was notified before packaging. Recommended fix: compare
the guard's **initial snapshot hashes** against every frozen expected hash
before reading/authenticating the extraction (or construct the guard first and
validate that snapshot). Retain the final stability check. Use resolved paths
when indexing the guard. Add this persistent-change regression to both entry
points; do not rely on a second disconnected measurement or success metadata.
Only the report is changed here; the integration lead owns the implementation.

Selected accepted repro receipt SHA-256 values:

- Normal map: `620ab1dc7a10a913303fb91ae36ac63570f9ca69ef415768162999c02a320192`
- Normal audit: `46ed1efeb4651c72607fa4e50a13374f8139629f1f54a269e0c9e7aede08bd41`
- Optimized map: `5deaa744ca0b5174371f3867f5189a665dd4673ce68b89dd9419b3cd3bdb59a9`
- Optimized audit: `acc1ab5963150a4e9d36d1303e3c5b37a6537ca96fe8b8f81e03eac6de03cc5f`

`race-receipts.json` retains these receipts, original complete freezes and changed
binding identities. This finding concerns automated provenance acceptance only.
Every map operation remains `unproved_rejected`, with `publication=false` and
`native_bridge_proved=false`. It supplies no mathematical or native counterexample.

## Fix coverage and independent checks

The **omission hole itself is closed**. Both entry points explicitly require
all seven paths: both controllers, `source-bindings.json`, imported `lower.py`,
original `drop.rs`, `infer_eval_support.py` and `infer_experiment.py`. Imported
lowerer and support omissions are covered. They always snapshot that set;
the generator includes it. `proof_checks.py` is used by the separate proof
controller and is also present in the full active freeze.

The full independent test commands were:

```sh
python3 -B evidence/source-freeze-completeness-review/recheck.py \
  --output /tmp/source-freeze-completeness-review-0912d2a7-final
python3 -B evidence/source-freeze-completeness-review/validate.py \
  /tmp/source-freeze-completeness-review-0912d2a7-validation-final
python3 -B evidence/source-freeze-completeness-review/package.py
```

Use fresh paths for the first two commands; `package.py` packages the fixed
recorded paths. `validation.json` records exact subprocess argv, log/receipt
hashes, expected refusal messages, all 80 adversarial cases and the 129-input
before/after stability inventory. Results:

- 11 existing units pass under each interpreter mode. Their new three tests
  always spawn optimized children; the separate review harness independently
  exercises actual normal **and** optimized entry-point processes.
- Of 80 source-controller trials, four healthy complete-freeze trials pass;
  28 single mandatory omissions and 28 single pre-start byte mutations refuse.
  The eight synthetic-binding trials (omitted or complete frozen binding) and
  four synthetic-receipt-only trials refuse. Four persistent support changes
  **after** the initial snapshot refuse with `tool inputs changed during evaluation`.
  The remaining four are the accepted validation-to-snapshot repro above.
- Both actual entry points pass with the committed active 21-file freeze in
  both modes. Fresh maps equal the committed map after expanding its instance
  dictionary: six instances, 44 rejected operations and three phase obligations.
  Each fresh audit has 12 `source_eligibility_rejection` controls and
  `semantic_negative_count=0`. No eligibility refusal is counted as a proof.
- Normal and optimized `freeze.py` outputs are **byte-identical** to active
  `source-freeze-fix.json`, SHA-256
  `ec9cdd4c64336e23921c75f21e5c62e1cac6e1e051a29463a8be7708e2eb53e1`.
  All freeze members hash correctly. The unchanged proof controller's required
  trust-input and exact four-module inventories are compatible with it.
- `check.py` and `proof_checks.py` equal `1f727448`; all four v12 proof sources
  equal `8935534e`. Authenticated the earlier full replay/proof receipts and
  seven command-log pins. The four actual parent `.vo` hashes match the original
  objects and declared pins, and its assumption inventory has 25 constants.
  These are authenticated inherited compilation/kernel results, **not fresh
  compilation or kernel checking by this review**. No expensive unchanged build
  was repeated; shared extraction/baseline/tool files were never modified.

Healthy map receipt SHA-256 (both modes):
`b32e962450102be15ea30816f4e469a0b3e380d924b10b618030a4c5b8b87614`.
Healthy audit receipt SHA-256 (both modes):
`756b29c15de0e07841ea997b5a3dcd7b079650478d25c831deffc6faa6627621`.
Raw adversarial results SHA-256:
`a7b0c448b41fbfe19e8045e9dc09de16d9bdbaa8806f7027373e40acb6bba000`.
Packaged validation SHA-256:
`48825eedc3e71ca591b2cff1c7e34b22ce313775f258685449751b913ebe1aad`.

## Bounded source-entry and fact-provenance audit

Scoped acceptance: `df15a254` delivers an **inspection**, not an entry theorem.
Authenticated its parent-pinned receipt
`4aa0336715f2fff0d285f05e45f1733c04783a82a2144b6cf3ea078f111a537b`,
all delivered inspection outputs, all 42 statement hashes/texts and per-constant
assumptions, the inspection object
`4117e44f47e664c0e6753bedfcbb6fd61186465fd9ad81730616f0b415532cff`,
and four delivered compile/kernel log hashes. Reparsed the assumptions and
kernel policy: 30 closed constants, 12 FE-dependent; the kernel summary contains
only the disclosed FE/Eq_rect_eq axioms and no unsafe features.
Candidate commands introduce **zero declarations**. `Fail Check` remains an
availability control, not a semantic negative or impossibility proof.

All 14 `entry-v1-freeze.json` repository entries match `git show df15a254:<path>`;
its six private input pins still match. Only historical source-map/audit bytes
differ from this checkout. The historical freeze was neither rewritten nor
replayed against changed current controllers. No 42-constant build was repeated.
Its receipts do not certify future arbitrary caller freezes; the automated-gate
finding above is separate from acceptance of these delivered inspection bytes.

The report and MISSING_LAYER correctly distinguish model initialization from
actual source entry. The incoming argument is `_1`; static loan 0 is issued for
local reborrow `_4`, then unsized through `_3`. A model ownership premise cannot
supply the missing source-state/original-context interpretation at the same
pointer, bytes, provenance, ancestor loans and C/F observations. Physical/phase
simulation, pending-reference resolution and original-array exit/import remain
separate open obligations.

Authenticated the canonical fact-provenance receipt
`73d69327af8dd6c954803713ed1f95a91f4e5a3b192e3285372b20f09abf69f4`,
its diagnostic pretrial record and historical observer-off build receipt, fixture,
four source-file identities, four AIR/SST logs and all seven stated anchors.
The recorded 6/0 run has executable `{u64}`, proof `{}`, helpers `{}`. In actual
source, array unsizing uses `external_body`; slice `iter_mut` uses
`assume_specification`. AIR explicitly **assumes** the constructor summary at
the reported call; the snapshot/wrapper/macro facts are available across the
loop. No newly checked native correspondence follows. The documentation
correctly makes no claim of individual indispensability, unsat-core measurement
or established shared full-property assistance. The 45-case ablation remains
the causal evidence: observer redundant for canonical, necessary as a group for
prefix only. No additional Rust run was performed here.

## Recommendation and limits

Accept the completeness requirement, static omission/mutation refusals,
generator compatibility, exact frozen model evidence and bounded inspection
classification. Address the P2 snapshot-binding gap before claiming unattended
provenance protection against persistent changes throughout startup. There is
no finding against the unchanged mathematical candidate or exact delivered
inspection, and no false native theorem has been accepted.

Snapshots do not protect against a malicious OS/runtime or a writer restoring
bytes between snapshots; no such attack is claimed or used here. A genuinely
new reviewed extraction may be rebound in an authorized **pretrial** config step.
Measured current hashes and success metadata alone are not that review.

The overall goal remains incomplete: canonical 6/0 covers every u64 and arbitrary
initialized [10], while the original bitcoin_hashes 0.14 pad-XOR prefix **and
untouched suffix** on arbitrary initialized [128], for all valid keys including
0 and >128, remains inherited 1/1 and unproved. Dangling empty keys and
disjoint key/pad regions within one allocation remain in scope. Same actually checked shared
assistance remains unconfirmed. There are **zero native correspondences**;
all 44 minimal operations, three phase obligations and 101 full-writer effects
remain open; publication is false. No domain narrowing, assumed frame, client
permission or whole-goal acceptance is introduced.
