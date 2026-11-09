# One conditional native insert-key link

This is an implementation handoff, **not independent acceptance**. `80a` is an
evidence label, not an engine version. The only runtime is the sealed accepted
**cache78 / policy72 / schema38** runtime. No engine, byte-coercion support,
shared reader, controller, guide, CI, original application or historical evidence
is changed relative to the accepted 2,145-file baseline.

## The bounded advance

The selected operation is the actual
`PrefixDatabaseTransaction<Inner>::raw_insert_bytes` implementation of
`IDatabaseTransactionOpsCore`, at `fedimint-core/src/db/mod.rs:823` of pin
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`:

```rust
let key = self.get_full_key(key);
self.inner.raw_insert_bytes(&key, value).await
```

Two different checks deliberately remain separate:

- A Verus executable preparation adapter calls the already-supported inferred
  native key helper and proves a **conditional operation-footprint contract**.
  This connects the argument preparation to an explicit key-level backend
  relation. Local and paired VIR/rlib imported callers check the result.
- A direct-rustc harness compiles and executes the **entire original async
  insert method item**, not a synchronous rewrite. It checks argument, value,
  single-dispatch and result forwarding against a demonstrator backend, plus
  cancellation/error/overlap/extra-write witnesses. These are finite Rust
  executions of extracted code, **not a proof of production async refinement**.

The exact verifier repro retaining the async trait and method is rejected with
`Verus does not yet support Opaque types in trait def`. This is an observed
blocker of this extraction, not a theorem that every alternative asynchronous
encoding is impossible. The implementation does not erase async/traits and
advertise the resulting model as a checked native operation.

## Exact proof and execution counts

| Check | Exact scope |
| --- | --- |
| Provider | 12 verified obligations, 0 errors; 3 executable bodies, 5 proof bodies, 6 spec definitions |
| Provider inference | 2 exact contracts: 1 original private generic native helper and 1 auxiliary public wrapper; all 6 required soundness/maximality/exactness checks pass |
| Imported consumer | 2 verified obligations, 0 errors; 1 executable body and 1 proof body; **0 newly inferred functions** |
| Native async proofs | **0**; dispatch, backend implementation, cancellation semantics and full application remain unproved |
| New inferred capabilities | **0**; native byte append was already accepted in cycle79 |
| New assumed application contracts / external bodies | **0** |
| Native harness | 6 Rust tests; 120 normal-success matrix cases plus 7 cancellation/error/overlap/backend/unpolled cases, 127 scenarios total |
| Compiler replay | 13 cases per mode: cold provider, warm provider, consumer, 4 Verus negatives, async blocker, native harness and 4 native semantic mutants |

`proof-scope.json` binds every provider/consumer selected definition, full item
and body hash, native extraction and trust category. Spec definitions are not
counted as checked proof bodies. Repeating a provider in the warm case is fresh
required proof work, not another native function or a cache-hit claim. Owned
candidate cache reports remain `miss` / `regenerated_state`.

## Reproduction

Inputs are the restored accepted baseline and the three read-only paths below.
No Cargo command, dependency download, application build, compiler rebuild,
Cargo-home write, commit, push, ref update, PR or subdelegation is required.

```text
runtime: /private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/runtime78
native:  /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint
sysroot: /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin
```

Run from the restored checkout; each output directory must not already exist:

```sh
python3 -B -m unittest discover -s source/tools -p test_fedimint_insert_link80a.py -v
python3 -B -O -m unittest discover -s source/tools -p test_fedimint_insert_link80a.py -v
python3 -B source/tools/check_fedimint_insert_link80a.py --output /private/tmp/insert-link80a-normal/replay --rust-opt 0
python3 -B -O source/tools/check_fedimint_insert_link80a.py --output /private/tmp/insert-link80a-optimized/replay --rust-opt 2
```

Create the parent output directory first, or choose an unused output directly
under `/private/tmp`. The runner removes inherited `GIT_*`, `CARGO_*`, `RUST*`,
`VERUS_*` and `VARGO*` context; sets `GIT_OPTIONAL_LOCKS=0`; uses explicit sealed
solver/sysroot paths; and confines caches, generated source and compiler outputs
to the new output directory. It verifies all 2,145 baseline paths and the 10,782
accepted guarded original inputs before and after, plus its own inputs and the
nine runtime artifacts. Native HEAD and clean status are checked with sanitized
Git and optional locks disabled.

The frozen handoff supplies the exact invocations, stdout/stderr, generated
source, paired exports, binaries, initial failed attempts, checksums,
reconstruction inventory and baseline-relative `apply_patch` delta. The delta
adds only this evidence directory and the two uniquely named tools. Apply the
accepted combined patch **once to clean checkpoint `3b05c191`**, verify its 2,145
hashes, then apply the new delta once. Do not apply the accepted patch to an
already restored tree. The handoff's reconstruction check uses a plain temporary
copy of those baseline files, not another Git worktree or ref.

## Fail-closed receipt checks

The new reader uses type-sensitive exact comparisons against frozen expected
reports: complete selected-body scopes, publication/completion flags, exact
counts, domains, generated contracts, proof requirements, inference kinds,
dependency identities and the existing 380-entry standard-model inventory.
There is no application model in that inventory. Function timings must be fresh
positive integers; top-level total work must be positive. Inference candidate
IDs are source-location-dependent: syntax/uniqueness and the exact dependency
link are checked, while the frozen shape identifies them by full function name.
Source bytes and extraction hashes are bound separately.

Both **separate** `--import` and `--extern` arguments must name the paired
provider exports. The whole argv is checked, rejecting joined flags, changed
import kinds, extra flags, duplicates and scope filters. Every positive export
requires a regular VIR file, regular nonempty rlib archive and the exact typed
certificate role/producer/policy/proof/model inventory. Source-dependent summary
digests must be 64 hex characters; the consumer report and export must bind the
**fresh actual provider certificate fingerprint**, not a fixture fingerprint.
Provider artifact hashes are rechecked after import.

All five negative compiler cases start with a genuine successful provider VIR
and must remove it. This checks stale proof-export nonacceptance, not stale rlib
cleanup or an adversarial compiler/cache-decoder theorem. Successful receipt
fixtures originate in actual successful sealed-runtime runs; their expected
contracts and complete scopes are frozen, not learned during acceptance.
Unit tests mutate every locked report/certificate/source/label field and cover
dynamic digest/work checks, malformed JSON, wrong artifact kinds, absent or
modified artifacts, symlinks, import kinds, joined flags, boolean-as-integer
counts/exits, false completion flags, failure categories and failure locations.
Synthetic artifact unit fixtures are explicitly not real VIR/rlib proofs.

Compiler negative controls separately reject an unprefixed prepared key, missing
backend premise, missing namespace premise and out-of-bounds indexing. Native
Rust mutants skip the prefix, change the value, dispatch twice or discard the
returned previous value; all compile and fail the forwarding test. Frontend
unsupported async and safety failures are not mislabeled as semantic proofs.

See `SOURCE_REFINEMENT.md` for the remaining obligations and reachability limits.
