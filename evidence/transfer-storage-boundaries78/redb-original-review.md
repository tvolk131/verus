# Independent review: full-error redb slot-selection milestone

Date: November 2, 2026. Verdict: **one P2 evidence-reader finding; request the
bounded correction and a NEW review.** No observed selector-theorem or compiler
soundness defect. No candidate, parent, native, engine, vstd or standard-model
fix was made. The only repository writes were the two authorized reconstructions.

## F1 — P2: bind import flags to their respective artifact kinds

Location: `source/tools/check_redb_slot_selection_opaque.py:251`, specifically
the membership test on line 252 in `validate_bindings`.

The reader checks that the arguments of `--import` and `--extern` are members
of the same two-path import dictionary. It does not require `--import` to name
the certified `.vir` or `--extern` to name its erased `.rlib`. Swapping these
two arguments, and keeping the duplicate command record consistent, passes
the ENTIRE `validate_matrix` with the actual fresh compiler reports, sources,
logs, certificates, hashes, counts and fingerprints unchanged. Reproduced
normally and under Python `-O` against the independent fresh receipts.

This is an evidence/provenance gate defect, not a way to make Verus prove a
false selector theorem. All original fresh runner commands have the correct
pairing, independently checked below. Nevertheless, the advertised receipt
validation accepts an invalid invocation as the source of the proof evidence.
Actually replaying the cross-wired command on a fresh destination exits 1:
`extern location for provider is of an unknown type`, followed by the `.rlib`
filename requirement and E0463. This is an early Rust dependency-resolution
failure, NOT a semantic proof failure or a successful certified import.

Minimal read-only reproduction, from the reconstructed review checkout:

```python
import copy, sys
from pathlib import Path
sys.path.insert(0, "source/tools")
import check_redb_slot_selection_opaque as check
root = Path("/private/tmp/verus-redb-full-error-independent.M3mn67")
receipt = check.original.read_json(root / "outputs/normal/receipt.json")
changed = copy.deepcopy(receipt)
index = next(index for index, row in enumerate(changed["cases"])
             if row["label"] == "consumer_debug")
command = changed["cases"][index]["command"]
external, imported = command.index("--extern") + 1, command.index("--import") + 1
command[external], command[imported] = command[imported], command[external]
changed["commands"][index]["argv"] = copy.deepcopy(command)
check.validate_matrix(changed)  # Unexpected acceptance.
```

Correction requested, not implemented: validate the two arguments against
their respective VIR/rlib identities, with an actual-receipt cross-wire
negative and a gate-removal test that reaches that intended binding gate.
No engine or standard-model change is needed for this finding.

Persistent reproduction evidence:

- `outputs/gate-probes-normal-final3/crosswired_import_kinds-accepted.json`
- `outputs/gate-probes-optimized-final3/crosswired_import_kinds-accepted.json`
- `outputs/import-binding-repro-final/results.json`
- `outputs/import-binding-repro-final/crosswired_import_kinds/actual.stderr`
- `repro_import_binding.py`, `gate_probes.py`

An exploratory renamed `--import` alias also passed the reader AND actual
Verus compilation. That alias experiment is NOT a finding, is not claimed to
be an intended import rejection, and is retained in the first reproduction
attempt. F1 is specifically the reproducibly invalid cross-wired file kinds.

## Reconstruction and immutable inputs

Used the provided isolated worktree, initially clean at
`3b05c1912b9d7314ad00c832c4065f4362562e02`. Read the applicable instructions;
there is no scoped AGENTS file in the Verus worktree. The read-only native
redb AGENTS permits report/analysis without native builds. No checkout,
branch, ref, commit, push, PR, network operation or subdelegation was performed.
Git subprocesses remove inherited `GIT_*` and use `GIT_OPTIONAL_LOCKS=0`.

The FULL 210-path dirty77 patch was applied with `apply_patch` exactly ONCE,
then the six-file ADD-ONLY delta exactly ONCE. The hashes match the requested
values:

- Baseline: `e76b758498a692e30e5683486a4d59d453202529cd13c2c0169401542b31690e`.
- Delta: `cbd298f436debf9dac299b776b238e595b3cb2aba83312e69358217e512463a5`.

Before the delta, all 2,092 baseline files match. After reconstruction and
after all final probes, all 2,098 files match the baseline-plus-six inventory
and the separately delivered complete file hash map. There are exactly 216
changed paths against clean HEAD; baseline and delta application logs and
statuses are retained. HEAD is unchanged; `git diff --check` passes.

Before/after ledger checks pass for 1,570 original source inputs, 1,085 Rust
build inputs, nine runtime files, 61 installed-toolchain inputs, 3,511 absolute
guarded inputs, and the native ledgers (132 redb and 1,199 Fedimint files).
All 701 indexed original milestone artifacts still match their hashes.
Native redb remains clean at
`c82ab865bb362bc4af067d332adc44978ff64813`; the native blobs are also checked
directly against all 132 pinned tracked files by each fresh runner.
Fedimint received only the requested historical ledger integrity read;
its companion work and proof claims are excluded.

These are source/artifact provenance checks, NOT an independent compiler or
native-crate rebuild. Reused vstd and the runtime are byte-unchanged. No fresh
full-vstd claim is made. All compiler proof crates, homes, Cargo/rustup/XDG
caches and outputs are private scratch. No Cargo, compiler/engine/vstd/native
rebuild, shared Cargo/rustup write, target copy or dependency installation was
performed. Final scratch size was approximately 63 MiB before sealing.

See `evidence/{baseline,reconstructed,final,final-after-matrix}-integrity.json`,
`evidence/{baseline,reconstructed}-status.txt`, and
`evidence/independent-source-proof-audit.json`.

## Source fidelity and exact theorem boundary

Independently cross-checked the six complete native source hashes and eleven
frozen native items at their pinned line positions. The complete attributed
`StorageError` is present byte-for-byte: `Corrupted(String)`,
`ValueTooLarge(usize)`, `Io(io::Error)`, `Unsupported`, `PreviousIo`,
`DatabaseClosed`, and `LockPoisoned(&'static panic::Location<'static>)`, with
native `Debug` and `non_exhaustive` attributes. All four selector/accessor/swap
bodies are byte-identical. All five complete record field sets, both physical
transaction records, `Checksum = u128`, the 128-byte payload, Result alias,
full-u64 IDs and original TransactionId derive list are retained. Module-path
flattening, visibility exposure and omitted unused record derives remain
explicit extraction adaptations, not whole-native-type refinement.

Exactly two newly authored opaque external TYPE declarations target actual
`std::io::Error` and generic `std::panic::Location<'a>`. Inspection of the
unchanged frontend confirms that the external ADT identity is used, generic
arguments/bounds are checked, and opaque transparency is `Never`. These are
trusted datatype abstractions, not inferred specifications of std behavior.
There are no newly authored standard-function specs, application axioms or
assumed selector theorem. A function-level `external_body` probe rejects under
`--no-cheating`; it is not interchangeable with these datatype declarations.

The sole representation condition remains `primary_slot < 2`. Existing authored
selection contracts, seven clients, XOR proof, branch goals and field frame
remain unchanged. The only full-enum contract adjustment excludes all other
error variants; the actual original body proves that exclusion. Only newtype
comparison plumbing is inferred. The original reduced-error milestone and its
full-enum frontend blocker remain historical evidence, not this milestone's
positive proof or an assumed reduced-to-full refinement.

Fresh imported probes transport actual io errors and generic borrowed locations;
independent probes also construct full-enum Io and static LockPoisoned payloads.
The non-static-to-static escape rejects before SMT obligations. std Error and
Location operations and explicit drop remain unsupported. Implicit consumption
can compile and verify an empty/unit contract; this is NOT proof of destructor
behavior, effects, release, panic behavior or observational equivalence.

No native caller establishment of the precondition, full native crate/type
refinement, recovery/tree validity, storage/fsync/crash durability or whole-db
safety is approved. Primitive/standard/opaque abstraction, Rust/Verus/SMT,
existing trusted models and unchanged reused vstd remain trust boundaries.

## Fresh evidence, separate from historical audits

Fresh independent normal and `-O` replays each pass exactly 47 Verus cases:
10 positives, 10 semantic failures, two missing-representation failures,
16 certificate/import rejections, seven frontend rejections (including the
separate unseeded original blocker), and two lifetime escapes. Every one of
the 36 seeded inference-aware failed exports per mode is removed. Negative
classification checks distinguish wrong postconditions, unsafe preconditions,
import/certificate errors, unsupported operations, and Rust lifetimes; runtime
failures are not substituted for these outcomes.

Each fresh provider is 23 obligations / 0 errors, with exactly 43 scope rows:
15 exec, one proof, 21 spec declarations, four outside-scope derives and two
selected scaffolds without translated bodies. Three new error projections are
spec definitions, not checked executable bodies. Debug receives no proof
credit. Consumers are 5/0, three exec; imported identity probes are 7/0, five
exec; implicit-drop probes are 6/0, four exec, without destructor credit.

All successful native scope identities/flags and source paths are checked,
with strict typed counts/completion/required-check flags and positive actual
solver work. Every provider cold/repeat reports candidate-cache MISS, not
cached proof acceptance. Both mode-specific caches contain exactly two files
with the exact string CACHE_VERSION. Actual stdout, source, stderr, VIR and
rlib hashes are bound. The independent reader computes certificate fingerprints
from the actual VIR trailer bytes and checks those against the actual consumer
certificate AND compiler report. It also verifies the real commands' correct
VIR/rlib pairing, independently of the defective receipt membership test.
Both runs preserve identical 4,995-entry current input guards before/after.

The unchanged 377 Core/Alloc/Vstd model fingerprint inventory hashes to
`676ba31b43183dac21a4863e549c53b2daae45d5cd041b1c889ee322105b0246`.
It is a registry inventory, not 377 newly used/proved application behaviors.
Actual per-run certificate fingerprints and scope/work counts are recorded in
`evidence/independent-source-proof-audit.json` rather than inferred from a source
manifest or copied from historical successful exports.

Both focused Python modes freshly pass 57 tests, including the 20 new tests;
the 20 are a subset, not additional application proof credit. Fourteen final
independently authored compiler probes pass their intended dispositions:
four positive identity/payload/u64 endpoint goals, three postcondition
failures, one missing-caller-precondition failure, one lifetime escape and five
distinct frontend/opaque/function-boundary rejections. All ten negative
independent probes remove their seeded export. See
`outputs/independent-semantic-final/results.json`.

Fresh reader testing per Python mode accepts the actual matrix and rejects
37 targeted receipt/certificate mutations. Each rejection asserts its specific
intended gate message. Removing that exact check in an in-memory copy of that
validator makes the same unit input pass (37 paired single-check-removal
tests). Shared `require_exact` cases target that earlier gate, not an incidental
later guard. This is unit-level gate sensitivity, NOT a claim that removing
one check bypasses the entire runner; overlapping gates exist. The invalid
cross-wired command is separately accepted by the unmodified full matrix
reader, as documented in F1. No mutation receives application-proof credit.

Additionally, three END-TO-END full-matrix mutation/removal pairs per Python
mode cover typed checked-body counts, a foreign unselected scope source and a
rehashed source with a changed native payload domain. Each unmodified full
reader rejects at the intended gate; removal of only that gate then makes the
entire consistently mutated matrix pass, without incidental later rejection.
Synthetic mutated logs and reports are clearly marked and are not compiler
proof evidence. See `outputs/matrix-gate-probes-{normal,optimized}/results.json`.

Historical broader Python/engine/VIR/division/common-transfer suites, original
native Cargo tests, full vstd and hosted CI were not rerun or promoted to
fresh results. The baseline remains cache77/policy71/schema38.

## Portability and retained attempts

Both actual acceptance replays omit `--snapshot` and use only current repo
fixtures, the pinned native application, supplied runtime/solver, and explicit
read-only installed sysroot. Source inspection confirms no call to the old
`audit_baseline`; the old BASELINE constant in the imported historical helper
is unused. Optional snapshot auditing only adds checks and cannot disable the
source, proof, scope, certificate, cleanup or current-guard gates. Runtime and
sysroot paths here are supplied inputs, not hidden old snapshot/worktree
defaults. No old root was renamed/deleted and no clean-machine relocation
experiment is claimed. F1 remains a reader command-binding portability issue:
the accepted mutated command itself cannot replay successfully.

Failed reviewer attempts are retained, not counted as passing final evidence:
the initial Python 3.9 `hashlib.file_digest` setup error, the first exploratory
semantic matrix (spec-vs-exec endpoint syntax, an accidentally spec-targeted
frame mutation, and a duplicate standard-type proxy), three gate-targeting
attempts exposing overlapping/earlier checks, and the first import reproduction
that incorrectly expected a harmless import alias rename to reject. Their
scripts/logs/results remain in the fresh review directory; final successful
attempts have distinct destinations. No candidate fix was made in response.

## Reproduction commands

From the already reconstructed checkout, NEVER reapply either source patch:

```sh
python3 -B /private/tmp/verus-redb-full-error-independent.M3mn67/audit_evidence.py
python3 -B /private/tmp/verus-redb-full-error-independent.M3mn67/gate_probes.py normal NEW_FRESH_TAG
python3 -B -O /private/tmp/verus-redb-full-error-independent.M3mn67/gate_probes.py optimized OTHER_FRESH_TAG
python3 -B -m unittest discover -s source/tools -p 'test_redb_slot_selection*.py'
python3 -B -O -m unittest discover -s source/tools -p 'test_redb_slot_selection*.py'
```

The normal runner command is the user-specified
`python3 -B source/tools/check_redb_slot_selection_opaque.py` with supplied
`--application`, `--verus`, `--solver`, a fresh outside-repo `--output`, and here
`--sysroot /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin`.
Repeat with `-O`. Exact executed proof commands and all hashes are preserved
in the fresh receipts. The final scratch inventory excludes only itself and
is recorded in `FINAL-INVENTORY.json`.
