# Independent focused recheck of the redb P2 repair

Date: November 2, 2026.

**Verdict: request a bounded correction and another independent review.** The
original separated-token cross-wire is fixed, and both requested fresh suites
and replays pass. One P2 command-provenance bypass remains in the advertised
unrecorded/duplicate-import gate. There is also one P3 regression-test gap.
Neither finding demonstrates a false selector theorem or a compiler soundness
defect. No candidate fixes were made.

## F1 — P2: reject or normalize joined import flags

Locations: `source/tools/check_redb_slot_selection_opaque.py:250` and
`source/tools/check_redb_slot_selection_opaque.py:260`.

Both the duplicate count and the empty-inventory check compare complete argv
tokens only with `--import` / `--extern`. They ignore valid joined option forms
such as `--import=provider=/path/provider.vir`. This is different from the
supported separate-token value `--import`, `provider=/path/provider.vir`.

Against an actual fresh 47-case receipt, take the `provider_debug_cold` row,
leave its `imports` dictionary empty, and append:

```text
--extern=provider=<actual provider_debug_repeat/provider.vir>
--import=provider=<actual provider_debug_repeat/libprovider.rlib>
```

Update the corresponding `commands[position]["argv"]` to the identical command.
The ENTIRE repaired `validate_matrix` accepts. All compiler reports, sources,
logs, outputs, hashes, certificates, proof counts, and fingerprints are left
unchanged. Both normal Python and Python `-O` independently accept this bad
matrix. Each mode also accepts isolated unrecorded joined `--import`, isolated
unrecorded joined `--extern`, and a joined duplicate of either flag alongside
the otherwise valid separate flags in a real consumer row: five accepted bad
full matrices per mode, not compiler proofs.

Actually executing the cross-wired joined command once, with ONLY its export
and library destinations redirected to fresh private outputs, exits 1 with:

```text
failed to deserialize imported library file `.../libprovider.rlib` - it may need to be rebuilt by Verus
```

This demonstrates that the joined import is recognized by the actual supplied
Verus runtime, not an ignored unknown option. No output VIR is produced. The
failure is an import metadata/deserialization failure, NOT a semantic proof
failure, a Rust lifetime failure, or an instruction to rebuild the runtime.

The observed defect is an incomplete correction of the receipt/provenance
gate, not a newly introduced production behavior. It contradicts the repair's
unrecorded-import rejection and duplicate-policy claims. Reject joined import
options explicitly, or normalize both supported option spellings before
checking counts, argument syntax, kind, and inventory bindings. Preserve valid
import aliasing; hard-coding the import alias is not required to fix this.

Read-only reproducer, from the reconstructed checkout or any directory:

```sh
python3 -B /private/tmp/verus-redb-p2-recheck.qaNvXj/repro_joined_flags.py
python3 -B -O /private/tmp/verus-redb-p2-recheck.qaNvXj/repro_joined_flags.py
```

The reproducer changes only an in-memory receipt and prints the unexpected
acceptance. The compiler invocation is already retained; do not reuse its
existing output destination. Evidence:

- `outputs/binding-probes-{normal,optimized}/accepted-joined-crosswired-both.json`
- `outputs/binding-probes-{normal,optimized}/results.json`
- `outputs/inline-unrecorded-normal/accepted-mutated-matrix.json`
- `outputs/inline-unrecorded-normal/actual.stderr` and `actual.stdout`
- `evidence/inline-probe-results.json`, including the exact actual argv and exit
  code. Its two receipt labels describe input receipts, not Python modes;
  optimized-interpreter confirmation is in `binding-probes-optimized`.

## F2 — P3: isolate the total-inventory-cardinality regression check

Locations: `source/tools/test_redb_slot_selection_opaque.py:289` and
`source/tools/check_redb_slot_selection_opaque.py:253`.

Removing ONLY `len(row["imports"]) == 2 and ` from the validator, in memory,
leaves the entire delivered 61-test focused suite passing normally and under
`-O`. The new surplus-inventory example adds another `.vir`, which is still
rejected by the remaining per-kind count. It does not test total cardinality
independently of kind cardinality.

A baseline with one correct VIR, one correct rlib, and one additional
hash-valid `.txt` (or source `.rs`) is the isolated case needed here. The
unmodified guard rejects at exactly `certified import requires exactly one VIR
and one rlib`; removal of only the total-cardinality conjunct accepts. A
consistently mutated real matrix with its existing source file additionally
listed as an import also passes the entire reader after only the inventory
guard is removed. Thus the guard is meaningful, and its absence is not masked
by a later binding or certificate check in these examples.

This is a regression-test weakness, not an additional current acceptance bug:
the unmodified correction rejects the surplus unclassified path correctly.
Add that isolated case to the delivered test. Evidence:

- `coverage_probe.py`, `evidence/coverage-normal.json`,
  `evidence/coverage-optimized.json`
- `outputs/binding-probes-{normal,optimized}/results.json`, entries
  `total-cardinality-clause` and `full_matrix_guard_pairs`
- `outputs/binding-probes-{normal,optimized}/matrix-extra-source-inventory.json`

## What the correction gets right

The two-file correction is otherwise small and within the stated reader/test
scope. It fixes the formerly invalid one-path fixture and adds four test
methods. No production, engine, vstd, standard-model, Cargo, or identity change
is present in the correction. Identity remains cache77/policy71/schema38.

Real separated-token consumer cross-wires now reject at exactly `command
import binding changed: --import`, with the duplicate invocation record kept
consistent. Independently changing only the extern argument to the VIR rejects
at exactly `command import binding changed: --extern`. Genuine fresh receipts
and the historical independent genuine receipts accept.

There are 21 independent unit-level same-input rejection/removal/acceptance
pairs per Python mode, plus six full-matrix same-input pairs per mode. They
exercise the new binding, inventory, nonempty-name, empty-inventory, and total
cardinality checks without relying on a later rejection. Mutants exist only
in memory; candidate files are untouched.

The delivered tests themselves detect whole binding/name/empty-inventory
guard removal by missing-expected-exception failures. Removing the entire
inventory guard makes its first missing-counterpart example raise IndexError;
that is an incidental test error, not acceptance sensitivity. The independent
surplus-inventory pairs supply direct acceptance sensitivity for that guard.

Removing just `separator == "="` does not change acceptance: with `partition`
and a nonempty expected artifact path, `path == expected` already entails a
separator. This is logical redundancy, not another untested effective guard or
a new finding.

Binding probes cover missing and duplicate flags, swapped and wrong artifact
kinds, missing counterparts, extra VIR/rlib/unclassified/no-extension paths,
uppercase/wrong suffixes, wrong imported hashes, duplicate JSON import keys,
missing/empty names and delimiters, empty paths, double equals, reversed map
order, and paths containing equals. Malformed null/list/string/absent import
maps fail closed using the pre-existing AttributeError/KeyError behavior;
these are recorded separately, not presented as polished new diagnostics.

A separately named `--import` alias with unchanged actual VIR/rlib arguments
also compiles successfully in a fresh consumer invocation and passes positive
report and certificate/dependency validation. This confirms that the repair
need not ban harmless import renaming. Syntactic alias unit tests are not
claimed to establish that an arbitrary extern alias matches the source's use.

## Exact fresh versus audited counts

Fresh, unmodified requested executions:

- Normal: **61 focused Python tests**, all pass; **47 portable Verus cases**,
  all intended dispositions pass.
- Python `-O`: **61 focused Python tests**, all pass; **47 portable Verus
  cases**, all intended dispositions pass.
- Totals: 122 requested unit-test executions and 94 fresh compiler case
  invocations. Each 47-case matrix has 10 positive cases and 37 expected
  negatives: 10 semantic, 2 safety, 16 import, 7 frontend, and 2 lifetime.

Additional fresh reader checks, PER Python mode:

- 6 synthetic positive binding checks; 34 exact-diagnostic negative checks;
  4 pre-existing malformed-map shape rejections.
- 21 single-guard-removal acceptance pairs, a SUBSET of the 34 negatives.
- 1 genuine full-matrix acceptance; 6 exact-gate full-matrix negative/removal
  pairs; 5 observed joined-option full-matrix bypasses.
- 7 targeted parent-test mutant runs; separately, 61 test executions with
  ONLY the total-cardinality conjunct removed, all still pass. These mutant
  results are NOT extra unmodified-suite passes or application proof credit.

Supplemental compiler invocations: three total. One expected-failing joined
cross-wire; one final successful valid-alias consumer replay; one retained
initial alias attempt whose compiler positive validation passed but whose
reviewer certificate call incorrectly omitted the dependency identity. The
initial attempt receives no additional proof credit. Thus there are 97 actual
fresh Verus case invocations in this review, but the requested portable replay
count remains 47 per mode, not 48 or 49.

Audited, NOT counted as fresh compiler evidence:

- Two historical independent 47-case receipts (94 old compiler case records):
  each genuine matrix accepts under the repair, and a consistent separated
  cross-wire mutation of each rejects at the intended import-binding gate.
- One parent self-report, `redb-real-receipt-repair.json`, containing the
  claimed genuine acceptance, cross-wire rejection, and intended diagnostic.
  It does not itself identify a receipt or provide a new compiler invocation;
  the claims are independently corroborated here, not promoted to fresh runs.
- Historical 57-test, 14 semantic-probe, broad engine/VIR/division/common-transfer,
  native Cargo, full-vstd, and hosted-CI claims were not rerun or counted fresh.

Both fresh portable replays use the requested application/runtime/solver and
the explicitly supplied read-only installed sysroot. They omit `--snapshot`.
Both native-before/native-after records and their 4,995-entry current guarded
input inventories are equal. All homes, Cargo/rustup/XDG paths, caches, and
outputs are private and initially fresh. No dependency, target, or runtime
copies, network, native-crate/engine/vstd/Cargo builds, subdelegation, commits,
pushes, checkout/ref changes, or PR actions were performed.

## Reconstruction and integrity

The tool-provided isolated checkout was initially clean at
`3b05c1912b9d7314ad00c832c4065f4362562e02`, on
`refs/heads/t3/independent-p2-recheck`. There is no applicable AGENTS.md in
the checkout or its checked ancestor directories, and the reconstructed
patches introduce none. The read-only redb AGENTS permits report/analysis
without native builds. Git subprocesses sanitize inherited `GIT_*`, set
`GIT_OPTIONAL_LOCKS=0`, and use private HOME/TMPDIR.

Reconstruction used `apply_patch` exactly ONCE EACH, in this order:

1. Full dirty77 baseline, SHA256
   `e76b758498a692e30e5683486a4d59d453202529cd13c2c0169401542b31690e`.
   All **2,092** baseline files match immediately afterward.
2. Original six-file redb delta, SHA256
   `cbd298f436debf9dac299b776b238e595b3cb2aba83312e69358217e512463a5`.
   All **2,098** original final files match.
3. Two-file correction, SHA256
   `61a49149207457b3ed18892a06b78771e97f78ee27235c18cd7e0dd9d16bde8f`.
   Both before and both after hashes in `inventory.json` match; all **2,098**
   repaired final files match, with no missing or extra files.

No combined parent patch or Fedimint delta was applied. Only those authorized
reconstructions write the isolated checkout. It remains exactly the delivered
repaired tree after testing, with 216 changed paths against clean HEAD.
`git diff --check` passes. Own HEAD, branch, and index hash remain unchanged;
the original main checkout's HEAD/branch/index/status also remain unchanged.
The original main checkout is clean at
`bad5b8896274d6df1be70e547eb81b004ab62ba1`, branch `verus-infer`; it is NOT
the isolated reconstruction's requested base and was never reset or patched.

Before/after ledger checks have zero mismatches: 1,570 source inputs, 1,085
Rust build inputs, 9 runtime files, 61 installed-toolchain inputs, 3,511
absolute guarded inputs, 132 redb native files, 1,199 Fedimint native files,
and 701 indexed historical candidate artifacts. Redb stays clean at the
required native pin `c82ab865bb362bc4af067d332adc44978ff64813`; native index
and branch metadata remain unchanged. Fedimint is ledger-only here, with no
companion work or proof claims. An additional 632 package/historical-review/
parent-record files frozen by this reviewer also remain byte-identical.

One shared orchestration-namespace ref addition was observed in both own and
original views of their common Git store:

```text
refs/t3/orchestration-v2/checkpoints/MTc2ODBlNGY3MGMyYzM5MzBjYjdmYTRiOWI4OThjZDI/ordinal/67
c4fcc874263eaede60b06544e82b83a8069c83a3
```

No ref removals were observed. This is disclosed separately from the unchanged
HEAD/index/branch/status checks. No attribution of the ref addition is made.
There was no explicit ref-writing action by this reviewer.

Two initial reviewer setup errors occurred BEFORE any patch application:
assuming the native ledger had symbolic HEAD, and counting embedded patch
marker strings instead of complete envelope lines. Both are retained under
`evidence/setup-attempt*.json`. The alias-checker dependency-argument setup
error is retained in `evidence/alias-attempt-1.json` and its distinct output
directory. None caused candidate changes or patch reapplication.

## Retained theorem and trust boundaries

Acceptance remains restricted to the extracted selector/accessor/swap bodies,
authored selection/frame contracts, complete native StorageError declaration,
and the existing `primary_slot < 2` condition. Source fidelity and certificate
checks are intact. Opaque declarations for actual io::Error and generic
panic::Location are authored trusted TYPE abstractions, not inferred standard
behavior and not function-level assumptions.

No approval is given for Error/Location operations, explicit drop, implicit
destructor behavior, effects/release/panic behavior, native caller establishment
of the precondition, whole-native-type/whole-crate refinement, recovery/tree
validity, storage/fsync/crash durability, observational equivalence, or whole
database safety. An implicit-consumption empty/unit contract does not prove a
destructor. Newtype comparison plumbing is the inference capability; this
repair adds none. Rust/Verus/SMT and existing trusted standard/opaque models
remain trust boundaries. The unchanged reused vstd/runtime have NOT received
a fresh full-vstd proof or independent compiler rebuild. Registry fingerprint
inventories are not counts of newly proved application behavior.

## Delivery paths

All paths below are inside `/private/tmp/verus-redb-p2-recheck.qaNvXj`:

- `REPORT.md`, `REVIEW-RECEIPT.json`, `FINAL-INVENTORY.json`
- `outputs/replay-normal/receipt.json`
- `outputs/replay-optimized/receipt.json`
- `outputs/binding-probes-normal/results.json`
- `outputs/binding-probes-optimized/results.json`
- `evidence/{before,after}-integrity.json`, `evidence/reconstruction.json`
- `evidence/{units,replay}-results.json`, `evidence/coverage-{normal,optimized}.json`

The report and receipt are bounded review findings, not candidate changes or
an authorization to widen the production/theorem scope.
