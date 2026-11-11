# Reproduction and retained inputs

Use the prepared checkout at base131ffc35. No shell worktree creation, parent
build, live target/cache or historical build-receipt tool path is required.
The only external proof runtime input is the named immutable accepted bundle.
Its receipt SHA256 is
43b29d75d63b897675449ac1bfd56024c4245ff6475d88ef85563349b495f679.
The controller authenticates all eight files before the first provider and
rebinds relevant engine source to identical own/Git 5d79f711 bytes. It reads
only the exact pinned upstream borrow_tracker/mod.rs from pins.json.

Fresh proof baseline (prints a new private root):

```
python3 -O -B source/tools/research/nonzero_ordering_goal/run.py
```

Fresh diagnostic MIR inventory, substituting the printed root:

```
python3 -O -B source/tools/research/nonzero_ordering_goal/mir_inventory.py /tmp/NEW_BASELINE_ROOT
```

To audit retained evidence without rerunning proofs:

```
python3 -O -B source/tools/research/nonzero_ordering_goal/finish.py /tmp/nonzero-ordering-baseline-qoptrgkf /tmp/nonzero-ordering-mir-x1sbsnfo
```

The same finish command passed with normal `python3 -B` as well. It authenticates
all frozen input/output receipts and every original native-body hash, verifies
the actual rustc -vV commit matches Miri's source pin, validates the MIR closure,
and recreates MIR.json and SEAL.json. No assertions are acceptance gates.

For a fresh baseline, first run audit.py on its root to write its BASELINE.json;
then use finish.py on the fresh baseline and diagnostic roots. Evidence path
labels and span-bearing debug hashes will differ; compare semantic identities,
proof statuses, body inventory and typed fingerprints, not wall time. No speedup
is claimed. A fresh run must remain blocked at the recorded ordinary carrier
Ord query and profile gates until a separately reviewed implementation exists.

Final raw directories are owner-only /tmp artifacts. SEAL.json records their
file hashes. Do not commit their source fixtures, debug MIR, AIR, binary outputs,
logs or cache entries. The sidecar is a diagnostic compiler client; it does not
serve as a replacement verifier and its output must never be imported as a
foreign certificate. No changed compiler, macros or vstd were built.
