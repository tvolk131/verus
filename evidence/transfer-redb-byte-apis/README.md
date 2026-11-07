# Native u64 little-endian extension and redb replay

The native `u64::to_le_bytes() -> [u8; 8]` and
`u64::from_le_bytes([u8; 8]) -> u64` calls now have ordinary Verus specifications
and a closed, typed Infer model. Arbitrary u64/eight-byte-array helpers and their
composition pass soundness, safe-domain maximality and output exactness. The
frozen redb bodies pass the former byte-API frontend boundary, then stop at the
original decoder's `u32::try_from(...).unwrap()` dependency. **This is a checked
bounded engine extension, not a verified redb codec or an accepted full-crate
transfer.** The original false formal codec property remains unassessed.

## Production change and trust

`source/vstd/bytes.rs` adds exactly two trusted primitive specifications, both
using its existing mathematical definitions, with unrestricted fixed-width
input domains and `no_unwind`. The definitions become open; the existing
inverse proofs are retained and reverified. The old open encoding entry point
is a compatibility alias. No new roundtrip axiom or external helper body is
introduced. Rust core/std implementation bodies are not formally verified.
The [trust inventory](TRUST.md) lists both assumptions, their Rust semantics,
the proved glue, existing dependencies and fingerprint/version decisions.

`source/vir/src/infer/byte_models.rs` recognizes the canonical core numeric
identity, matching vstd proxy and exact executable signatures. It generates
the existing array/index/cast/finite-bitwise vocabulary. Source-call types are
checked before restoring the primitive type of mathematical scalar summaries,
including literals and arithmetic results. It preserves operand safety; for
example `(x + 1).to_le_bytes()` retains `x <= u64::MAX - 1`.

All required checks and publication barriers remain active. Policy 55 and cache
61 distinguish this expanded boundary; schema 38 is unchanged because no serialized
representation changes. Existing declaration, complete-vstd-artifact, cache and
transitive interface fingerprints bind the model. Other widths, byte orders,
slice/Vec helpers and application lookalikes acquire no library law.

## Actual validation

- vstd: **2,219 verified, 0 errors**, including the existing byte inverse lemmas.
- VIR unit suite: **119 passed**; includes independent typed-model rejection and
  generated-value/native-Rust differential tests at every byte position.
- Dedicated `infer_byte_apis` integration suite: **6 passed**. It checks ordinary
  native API calls, both arbitrary-input roundtrips, every encoding byte and
  exact length, cold/warm/provider/client behavior, metadata/model binding,
  semantic negatives, poisoned caches and removal of stale exports.
- Adjacent suites: **31 ordinary array tests, 32 Infer array tests, 6 Infer
  integer-operation tests passed**.
- The broader `cargo test -p vir` command also ran **18 failing doctests** in
  unchanged `ast.rs`, `ast_to_sst.rs`, `modes.rs` and `resolution_inference.rs`.
  Their illustrative fragments contain omitted variables, ellipses or incomplete
  syntax. The 119 unit tests passed in that same invocation. These documentation
  failures are retained in the logs and were not fixed or counted as successes.

The persistent focused replay records six exact provider helpers and two exact
imported-client helpers, each cold and warm. Every required proof is fresh and
proved; warm runs report candidate-cache hits. The primitive and roundtrip
helpers have precondition `true`; the increment helper retains its actual overflow
restriction. Provider metadata contains both native byte models and existing
array equality dependencies. Verified native rlibs and interfaces are published.

Separate reversed-byte, shifted-byte, replaced-byte and wrong-caller-property
fixtures generate checked exact helper contracts and fail their intended caller
postcondition; no interface is published. They do not fail from unsupported
source, malformed Rust or resource exhaustion. Mutated policy, incomplete proofs,
corrupt summary fingerprints and stale trusted-model fingerprints fail the
corresponding import checks. The integration suite also changes the actual vstd
artifact and checks stale-model rejection, and poisons cached byte order/domain
to obtain fresh soundness/maximality failures with stale-export removal.

The native boundary harness executes the verified provider's actual primitive
calls against an independent shift-based encoding oracle for **6,144** cases,
covering every byte value at every position on three backgrounds, plus constant
and maximum increment boundaries. This empirical host check does not replace
the arbitrary-input formal checks or verify Rust's implementation universally.

## Frozen original versus updated application replay

The replay imports the original extraction/native harness read-only from the
frozen `evidence/transfer-redb` directory. It clones only the application into a
new persistent output directory; no engine checkout is created. Every selected
positive executable body, authored domain and native semantic mutation is
unchanged. Extraction and generated-source hashes are recorded and compared
with the frozen run. The original clone and raw run remain unmodified.

| Frozen probe | Original engine | Byte extension |
| --- | --- | --- |
| Canonical codec, cold/repeated | Native byte APIs unsupported by frontend; no candidates | Byte APIs accepted; Infer dependency walk rejects canonical `core::result::...::unwrap` in `u32::try_from(...).unwrap()`; no candidates |
| Deliberately false codec postcondition | Same frontend boundary; not semantic bug detection | Same unwrap boundary; still not semantic bug detection |
| Page-size equation/bound | Unsupported record binary operation at variable left shift | Same independently evidenced blocker |
| Address safety, including debug assertions disabled at codegen | Unsupported `core::panicking::panic` from `debug_assert!` | Same independently evidenced blocker |

All ten original-source verifier invocations remain unsupported and publish no
interface. Their second invocations have empty candidate caches: they are repeated
attempts, **not cache hits**. They differ from the successful focused warm runs.
The precise next observed codec blocker is the unmodeled `Result::unwrap` call;
this result makes no claim about proof success after that call is supported.
Record packing/shift handling and assertion/panic boundaries remain separate work.

The canonical intended property still quantifies over **all**
`region <= 0x000fffff`, `order <= 20`, and
`index <= (0x000fffff >> order)`, including maximum region, order 20 and index zero.
The encoder's reserved bits are cleared. Ignored/reserved bits and raw orders
21..31 retain the frozen separate native audit; they are not silently added to
the canonical proof domain. No order-zero/small-index restriction is introduced.

The original size probe still covers every u32 page size with order at most 20.
The address probe keeps positive u32 page size and the mathematical containment
conditions `H + (index + 1)*S <= R` and `D + (region + 1)*R <= u64::MAX`, where
`S = (1u64 << order)*page_size`. No address/size precondition is inferred from
byte roundtrip correctness. The original native regression passes **2,097,467
identifier checks and 12,222 address checks**, with three overflow layouts excluded
by that stated domain. This sampling enumerates legal indices over all orders
with varied/boundary regions; it is not the full cross-product of all regions.
The unchanged deliberate decoder shift `20 -> 21` mutation compiles and fails
the intended native `region roundtrip` assertion (exit 101).

## Pins, artifacts and reconstruction

The clean assigned base was verified before editing: HEAD, `origin/verus-infer`
and the shared git directory's `FETCH_HEAD` all name
`55bd67ad980e90e92c77efe4453a927f097c3386` (tree
`57340006528dc5a6e8cbc85591f007ffaf7a5366`). This worktree has no per-worktree
`FETCH_HEAD`, so the shared fetched record was read explicitly. No branch switch,
commit or PR was made. Application pin:
`c82ab865bb362bc4af067d332adc44978ff64813`.

The original SHA alone does not identify this dirty extension. `manifest.json`
records the final source-tree digest, changed-source hashes, exact dirty patch
hash, binary/macro/vstd/solver/toolchain hashes, validation commands and final
persistent replay paths. Each raw run has full `engine-source.json` (1,547 input
file hashes), `engine.patch`, generated sources, command argv/environment/logs,
structured reports, published focused artifacts and a receipt. Artifact guards
hash source and tool inputs around each replay command. Source guards also cover
the final compiler build, vstd regeneration and focused rechecks.

Final authoritative replay:
`/private/tmp/verus-redb-byte-extension-v4_rq8wp/run-8ihte2te`.
Its engine input-tree SHA-256 is
`c42a427dee17539bbf66c82ed2c18d83c8cb1fe083fe8abc28486084fb6c6139`;
the exact production/test/guide patch SHA-256 is
`30f4dba50c242ce6d7b9c42b3389b215c61605760f822b03636af1b129e3e589`.
Final build/check logs are under `/private/tmp/verus-redb-bytes-build/repo-format-final`, especially
`source-pinned-build-validation.json` and `replay-source-pinned.json`.
The earlier extension runs `run-nab5txft`, `run-jz_hpf_c` and exploratory build/test logs are
retained with their own source/artifact identities; they are not substituted for
these final source-pinned results.

Only this worktree's ignored `source/target-verus` and `source/z3` are mutable.
The prior worktree's artifacts were read/copied as build seeds, then compiler,
macros and vstd were rebuilt locally. Final regeneration cleans only this own
target's vstd package artifacts before rebuilding/verifying it. Compiler builds use
`--locked --offline -j 2`; vstd uses `--offline -j 2` with its recorded lockfile.
No shared mutable target/cache is used. Rust is 1.99.1,
Z3 is 4.16.0, macOS aarch64. Concurrent wall times are recorded observations,
not benchmark comparisons.

From the assigned engine worktree, rebuild as documented in `BUILD.md`, then:

```sh
python3 -B evidence/transfer-redb-byte-apis/run.py \
  --baseline /Users/tommyvolk/.t3/worktrees/verus/t3-19207161/evidence/transfer-redb \
  --redb-source /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --output-root /private/tmp --timeout 180
```

The runner creates a fresh `run-*` directory and separately clones the pinned
application there. `--baseline` may point to an exact read-only copy of the frozen
evidence; `--redb-source` may point to another clean clone at the application pin.
Individual replay commands have finite timeouts; build commands are bounded at
1,200 seconds in the recorded controller. The existing `infer_experiment.py` and
`infer_eval_support.py` enforce structured required checks, publication agreement,
artifact stability and process cleanup. Controller `success: true` means all
expected outcomes were validated, not that redb verification succeeded.

No redb crate code was edited; its `just test` is not needed for this engine
extension/extraction experiment. The inherited audit/adoption/build guidance and
redb's read-only AGENTS instructions were read. No application-specific exception,
new unchecked imported contract or publication-policy weakening is used.

Integration overlap: no `records.rs` or nearby early-return implementation is
edited. Shared registration is confined to `source/vir/src/infer.rs`; cache/policy
constants and the guide version paragraph may overlap another engine extension.
Combine version changes consistently when integrating; do not undo the typed
model gates or fingerprint/publication checks.
