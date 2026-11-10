# Model source and status: source-only review

Recommendation: **scoped SOURCE-ONLY acceptance of the F1–F3 fixes**.
No new actionable source finding within this scope. The three findings in
`3dd71948:evidence/proof-checker-source-review/REVIEW.md` are addressed by
`0e1c7081190364a46d9b0e3b0b18eb72a545b31c`. This recommendation does not accept
the running matching batch, establish a native theorem, or complete the goal.

Reviewed prepared checkout: `9f91fd58c1321ee145f4225e5fbf796d5f7d19df`.
The five requested Python files are byte-identical to their `0e1c7081` versions
(`git diff --exit-code` passed). Since that fix, the active inspection freeze
changes only its two recorded output destinations; its input identities are
unchanged. Separate heap-initialization mathematics is outside this review.

## Prior findings and source reasoning

### F1 closed: receipt identities authenticate the retained model sources

`acceptance_inputs.py:12–24` requires the exact ordered four-module list and
exact four proof-term keys from `proof_checks.NAMES`. For each expected module,
it requires exactly one matching `.v` basename in `proof_artifacts`, with a
string SHA256 of exactly 64 lowercase hexadecimal characters. Missing and
ambiguous source records reject, including duplicate matching basenames under
different directories. Other malformed required fields also fail closed;
they are not converted to acceptance.

`native_source_entry/entry_audit.py:67–76` independently requires the model
receipt among the frozen private inputs and checks their identities. At
104–111, it requires a successful kernel-checked receipt, obtains all four
recorded source hashes, and compares model objects and sources against that
receipt. The old live candidate-source comparison is removed.

The important second check is at **139–143**: both `.vo` and `.v` identities
in the retained `guard.initial` must equal the authenticated model receipt.
The exact module set also determines all eight model paths added at 114.
The same guard contains the receipt and required private/public inputs
(112–117), authenticates those retained identities at 118–125, and remains in
use through the command checks and final check at 195. Thus a persistent
source substitution between the earlier live equality and guard creation
cannot become a trusted starting source identity. This is source reasoning;
no substitution or scheduling experiment was executed.

The helper's imports are covered transitively: the inspector explicitly
requires `acceptance_inputs.py` and `proof_checks.py` at 60–64;
`source_audit_inputs()` includes both shared support modules
(`source_map.py:19–24`). These public inputs are authenticated again against
the retained snapshot at 120–122.

### F2 closed: independently mandatory support and helper inputs

`native_semantic_bridge/check.py:42–47` now independently requires
`acceptance_inputs.py`, both files returned by `support_artifacts()`, and the
previous checker/parser/binding/foundation/source inventory in `freeze.files`.
This check precedes reading readiness at 50 and all prover commands. An omitted
support or helper cannot merely acquire a freshly measured starting identity.
All supplied frozen identities are compared with the retained input snapshot
at 92–94, and both shared support paths remain explicitly guarded at 84–85.

`freeze.py:21–29` includes the new helper directly and both support files via
`source_audit_inputs()`. The helper imports only the already required
`proof_checks` and `infer_experiment`; the latter imports `infer_eval_support`.
No new local imported trust input is omitted from these required sets. Active
model metadata has all **24** public hashes matching this checkout; active
inspection metadata has all **16** public and **six** private hashes matching.

### F3 closed: final guard failures cannot promote model proof status

`check.py:31–34` initializes model-kernel and native-bridge status false,
publication false, and discharged effects zero; `replay_receipt` initializes
success false (`infer_experiment.py:31–45`). At `check.py:160–166`, the proposed
successful fields are constructed in a separate dictionary. This does not
update the receipt.

`acceptance_inputs.py:27–32` rejects an empty guard collection, calls each guard
before updating the receipt, and lets failures propagate. The sole call at
`check.py:167` supplies the output guard, every copied/generated source guard,
and the original input guard, in that order. A failure from any of these
checks leaves the initial false model status unchanged and is persisted as a
failed receipt by `replay_receipt`. Only completion of all checks promotes
model-kernel status. Publication and native-bridge status remain false;
source-connection and original-array-future-import fields are explicitly false
in the successful result, as before. This establishes the requested final-guard
ordering, not a claim about every possible later I/O failure.

## Copied-source lifecycle and ordinary compatibility

`check.py:133–149` retains each copied `.v` against its pretrial identity before
compiling it; generated `AssumptionInventory.v` is bound to the generated text.
All five source guards survive until the final acceptance call. The output
guard is taken after ordinary compilation and before recursive checking
(153–158), and it is checked before status promotion. The inspector similarly
binds its copied `EntryInventory.v` before compilation (163–170) and checks that
guard after recursive checking (190–195). The earlier full copied-foundation
inventory and authenticated `_CoqProject` loadpaths remain in place
(`check.py:82–104`; `entry_audit.py:115,126–146`).

The authenticated completed receipts retain the expected four-module order,
source inventory and proof terms, so the stricter source-record shape is
compatible with the actual existing model results. Removing the comparison
with today's candidate sources correctly uses receipt identities for the
retained historical proof source. Historical freezes with older checker bytes
remain historical; this review does not claim they replay under the new code.
The active freezes contain the new helper and supports. No proof source,
assumption policy, loadpath policy, native premise or input domain changes in
this fix. Ordinary model compilation and current inspection execution were
not rerun here.

## Independently checked immutable evidence

Authenticated the supplied completed `aea14e56` batch receipt:

`/tmp/verus-infer-buffer-cycle-20261109/full-snapshot-validation-aea14e56/receipt.json`

SHA256: `740b2bb5dca4ed353a8c3b135d5b87e068c78de4328e31c5e0652674791f920c`.

It records success, 11 units in each mode, 12 source / four readiness / three
acceptance / ten identity controls, 18 named model theorems plus six byte
obligations, 25 assumption constants and a 42-constant inspection. All recorded
command return codes are zero and the recorded before/after input snapshots
are equal. These are authenticated historical receipt claims, not commands
executed by this reviewer or validation of the changed controller.

Authenticated its completed model receipt:

`/tmp/verus-infer-buffer-cycle-20261109/full-snapshot-validation-aea14e56/proofs/receipt.json`

SHA256: `46e35308c297fd6c5eb9ce0fe94b80f86b094d4c6f59493962dee5eb8e34d562`.

Also authenticated the older completed model receipt actually pinned by the
active inspector and used by the permitted `RecordedSources` tests:

`/tmp/verus-infer-buffer-cycle-20261109/semantic-guard-validation-1efbd004/proofs/receipt.json`

SHA256: `790aaa8e2eadd1a668bef67cdce938dc3048318d85ee753057cfb805e7982e41`.

For each model receipt, all ten retained `.v`/`.vo` hashes match the recorded
identities. The four source hashes match this checkout and the original-source
fields in `parent-semantic-foundation.json`; all four object hashes match that
record's original accepted objects. Both receipts record successful normal
compilations, assumption inventory and recursive checking; only model-kernel
status is true. Native/publication statuses stay false and discharged native
effects stay zero. Assumption/kernel fields were read as authenticated data;
their output parsers were exercised only by the permitted tests.

Exact identities, command-status summaries, active-freeze checks and executed
test commands are retained in [checks.json](checks.json). Recorded parent paths
were treated as data; no live parent or sibling source checkout was inspected.

## Checks actually executed and limits

Executed only the permitted test classes, with `-B` and explicit class selection:

- `test_acceptance_inputs.RecordedSources`: **4/4 normal + 4/4 optimized**.
- `test_proof_checks.Assumptions`: **6/6 normal + 6/6 optimized**.
- Five in-memory `ast.parse` checks for the five requested Python files.
- Read-only immutable hash/data comparisons described above and Git source
  comparisons. Writes are limited to this new evidence directory and its commit.

See [normal log](pure-normal.log) and [optimized log](pure-optimized.log).
Read the new filesystem-based tests, but did **not** execute `FinalStatus`,
`RequiredSupport`, `SourceBindings`, `test_source_freeze`, source/race
controllers, filesystem concurrency/mutation/substitution experiments, or
Coq/Rust builds. Parent-reported eight new targeted tests per mode and the
planned 19-unit totals (eight old + three source + eight new) are distinct
from this reviewer's ten executed pure tests per mode.

The current batch under
`/tmp/verus-infer-buffer-cycle-20261109/model-source-status-validation-0e1c7081`
was not read, polled or watched. Its completion and matching receipts remain
the parent's responsibility. The previous automatically rejected broader
review supplies no acceptance and its excluded action was not retried. No
implementation or baseline edits, subagents, push or PR were made.

Read the current bridge and source-entry reports, parent semantic-foundation
record and goal TARGET. Separate `native_heap_initialization` mathematics,
including the reported 29 checked constants, is not reviewed or accepted here.

## Preserved goal limits

Canonical remains the inherited **6/0** result for every `u64` and arbitrary
initialized `[u8;10]`, with authored semantics only and no application-local
mechanical proof calls or snapshots. The full independently frozen
bitcoin_hashes 0.14 XOR prefix **and untouched suffix** remains **1/1, unproved**
for arbitrary initialized `[u8;128]` and all valid keys, including 0 and >128.
The observer is redundant for canonical, necessary as a group for prefix only;
actual shared checked assistance across both full properties is unconfirmed.

There are **zero native correspondences**, **44 minimal + three phase**
obligations and **101 full-writer effects** open. Model 18+6 and separately
reviewed heap results do not prove actual source typing, provenance, ancestor
loans or C/F import. No new contracts, premises, domain restrictions or native
facts are supplied. Publication and overall goal completion remain **false**.
