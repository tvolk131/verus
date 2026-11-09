# Checked source-connected pure prefix boundary (cycle78)

November 2, 2026. This bounded bridge follows the reviewed CONDITIONAL MODEL-ONLY
transaction/replay result without upgrading that model into a native transaction
theorem. Three original pure method bodies are now checked against explicitly
AUTHOR-OWNED contracts, with byte-identical executable bodies:

- `PrefixDatabase::get_full_key`, native `fedimint-core/src/db/mod.rs:712`.
- `PrefixDatabaseTransaction::get_full_key`, native `db/mod.rs:767`.
- `PrefixDatabaseTransaction::get_full_range`, native `db/mod.rs:773`.

The two key bodies retain `Vec<u8>`, `&[u8]`, `self.prefix.clone()`,
`extend_from_slice(key)` and the original final expression. The range body retains
`std::ops::Range`, its `start`/`end` fields and both original method calls. No body
is projected to a sequence model or replaced with a hand-written executable.
`native-items.json`, `contracts.json` and the per-replay correspondence ledger
make the scaffold changes and body identity auditable.

## What the checked boundary says

The returned vector view equals `prefix + suffix`, for every input, with no
authored precondition. Checked lemmas prove prefix membership, exact same-prefix
suffix cancellation/injectivity and disjoint keys for prefix-INCOMPARABLE byte
namespaces. Authored callers connect these lemmas to the original methods.
The range theorem proves exact endpoint transformation, not backend range-scan
correctness, ordering or deletion confinement. A checked comparable-prefix
overlap witness and a failing unconditional-disjointness control expose the
necessary condition. Actual encoded module prefixes are NOT proved incomparable.

Provider scope: 3 native executable bodies, 2 authored executable callers,
5 authored proof bodies, 3 authored specification definitions; 13 identities,
10 verified queries, zero errors. Each imported consumer has 3 authored checked
executable bodies, 3 identities, 3 verified queries, zero errors. These are
ordinary checked contracts and ordinary Verus imports, NOT inferred summaries.

## Infer result and trust

The original three bodies are probed with Infer before and after replay. Both
probes reject `Vec::extend_from_slice` at the source call as frontend/VIR
`unsupported`, requesting 3 functions and generating ZERO. This is not inference
progress. `INFERENCE_BLOCKER.md` freezes the exact diagnostic and bounded proposal.
The final probe seeds an ordinary successful artifact and requires Infer's
failure path to delete it. Ordinary failed exports are separately disclosed:
they may retain/replace `.vir` despite `interface_written=false`; none is used
as an import or success certificate. Only freshly successful guarded providers
are imported. No warm-cache or inferred-interface certification claim is made.

Ordinary Verus/compiler translation, Z3 and reused sealed vstd are trusted.
In particular, the existing vstd specifications for Vec clone/extend and u8 clone
are library trust, not fresh verification of their standard-library implementations.
There is no new application axiom, `assume`, `admit` or `external_body` helper.
No fresh full-vstd result, backend, transaction, consensus or funds result follows.

## Portable replay

From this repository, select an existing clean application at pin
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`, the supplied sealed verifier/solver,
and a NONEXISTENT output directory:

```sh
python3 -B -m unittest discover -s source/tools -p test_fedimint_prefix_boundary.py -v
python3 -B -O -m unittest discover -s source/tools -p test_fedimint_prefix_boundary.py -v
python3 -B source/tools/check_fedimint_prefix_boundary.py \
  --application /path/to/pinned-fedimint --verus /path/to/sealed-tools77/verus \
  --solver /path/to/sealed-tools77/z3 --output /path/to/new-output
```

Repeat the last command under `python3 -B -O` with another fresh output. Optional
`--sysroot /path/to/existing/1.99.1-toolchain` overrides read-only discovery.
The runner never invokes Cargo/rustup, downloads dependencies or rebuilds tools.
It pins all nine audited runtime byte identities, uses private homes/caches/tmp,
strips inherited Git overrides, disables optional locks and checks clean native
pin/source fidelity and the current `ArtifactGuard` before/after invocations.
No previous scratch or worktree path is required for reproduction.

Each replay has 13 cases: 2 Infer blockers, 4 positive local/imported compiles
(debug assertions on/off; default erasure; `--no-cheating`), 6 intended semantic
proof failures and 1 intended arithmetic-safety failure. Exact counts, source
spans, failed-function identities, artifact hashes and raw commands are bound in
each receipt. Failed frontend, panic or resource cases never count as semantic
evidence. See `REPORT.md`, `SOURCE_REFINEMENT.md` and `artifacts.json` for the
fresh/audited distinction and persistent raw artifact locations/hashes.
