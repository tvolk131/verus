# Native validation v4 acceptance

The finite milestone is complete. The original native path from unchecked
signed unit through signature checking and outer validation now proves output
identity, session/round/membership checks, existing parent policy and ordered
errors. The proof remains conditional on an explicit lawful verification
interface. [Assumptions and scope](assumptions.md) state its limits.

Engine implementation: `5929d1db8f39548bcca78b19012b273ea6b613d7`.
Native implementation: `fa88a4f0956e7a888bbe3d47bc8e14286f2af953`.
Native baseline: `a2de660d5467ce3611584768285b9227a1e2832b`.
The exact full engine SHA is authoritative in [the source receipt](engine-source-receipt.json).

The verifier now infers complete shared field identity through an owner with
unrelated opaque storage. Its bounded ordinary `map_err` assistance composes
constructor fields and existing callee postconditions, including clone
relations, while verifying the unchanged callback and surrounding body.
Metadata policy 50 / candidate-cache version 56 requires providers and
dependents to be rebuilt together.

| Acceptance evidence | Result |
| --- | --- |
| Fresh restored proof | 204 verified results, zero errors (101 crypto + 103 consensus) |
| Exact inferred helpers | 39; all 117 mandatory certificates proved |
| Compiled checked bodies | 125 executable bodies + one existing generated proof body |
| V3 coverage | All 35 helpers and 84 previously checked items retained |
| Native tests | 192 unit tests + one doctest pass |
| Verus inference regressions | All 285 pass |
| Vstd | 2,215 verified, zero errors |
| Semantic mutations | All seven fail ordinary application verification; helper certificates remain healthy |
| Publication controls | No-verify rejected by the compiler; filtered invocation rejected by Cargo |
| Source inventory | 179 comparable original bodies unchanged; no new exclusions |
| V3 preservation | Original fixture's 161 tracked files and fixed engine's nine pinned artifacts unchanged |

`validation_order_goal` covers all native branches for arbitrary input under the
checked reference provider, including simultaneous failures. The constructed
positive-round goal builds the real parent map, ControlHash, PreUnit, FullUnit
and unchecked wrapper, then calls the original validator and proves successful
output identity. Signature success and rejection also have unrelated checked
Packet examples. V3's initial/positive metadata policies and domains remain.

The mutations bypass signature/session/round/member/parent checks, alter an
error payload, and reverse session/round priority. These are deliberate changes,
not newly discovered defects in Aleph. Development logs also retain a rejected
stale-policy import and two controller-assertion mistakes; neither is counted
as a semantic mutation success. The accepted controller run exits successfully
and finishes with a fresh full restored replay.

This acceptance was performed by the implementing agent. No additional
independent-agent review is claimed. Both repositories were committed locally;
nothing was pushed.

## Evidence and reproduction

- [Machine-readable closure](closure-summary.json) and [control outcomes](controls-summary.json).
- [Source audit](source-audit.json), with original-body hashes and all six new typed-only assumed signatures.
- [Native patch](native-source.patch) and [complete accepted native source](native-source.tar.gz), preserving its license and prior evidence.
- [Raw evidence](raw-evidence.tar.gz) and [per-member hashes](raw-evidence-index.json): 315 verified members, including exact command/environment receipts, full stdout/stderr, original/accepted syntax inventories and source snapshots, recorded runners, proof/native outputs and compiler test outputs.
- [Engine source provenance](engine-source-receipt.json). The proof binary was built from this exact source content before the local implementation commit; its version banner retains the baseline SHA and dirty marker. Artifact hashes bind every accepted proof to the recorded compiler and vstd. The full regression run uses that same compiler hash.

The recorded runner uses `/private/tmp/verus-validation-v4` as its work root,
with the Aleph checkout at `aleph/`, and the shared Verus workspace as its engine.
The accepted run is `runs/accepted-restored-full`; it uses a fresh target directory
and records hashes of every proof source before and after execution. The
`accepted-*` mutations and controls are the acceptance series. `runs/native-tests`
and `engine-validation/infer-final.*` / `vstd-final.*` record the other checks.

For another location, build the engine commit and matching macros/vstd using the
repository build instructions, extract the accepted native archive, and run the
recorded Cargo command with updated paths. The proof command is:

```sh
cargo verus verify --manifest-path /path/to/aleph/Cargo.toml \
  --config 'patch.crates-io.vstd.path="/path/to/verus/source/vstd"' \
  --offline --locked -p aleph-bft-crypto -p aleph-bft-types -p aleph-bft \
  --fwd-verus-args-to roots -- --infer-goals --no-cheating --output-json
```

Use Rust 1.99.1, the matching release binaries on PATH, a fresh CARGO_TARGET_DIR
and VERUS_INFER_CACHE, and VERUS_Z3_PATH pointing to `source/z3`. Clear inherited
Rust flags/wrappers as the archived runner does. Native tests use Rust 1.85.1,
a separate target directory and ordinary `cargo test` for those three packages.
The offline commands require the recorded Cargo.lock dependencies to be cached.
`source/tools/adoption_inventory` produces the parsed syntax inventory; the
archived closure script compares retained helper/body names, certificate states,
source hashes and the preserved v3 workspace/artifacts.
