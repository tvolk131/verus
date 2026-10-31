# Native hex public vector decoder acceptance

Accepted: the original `decode_to_vec` composes the native length-checking string
constructor, ChunksExact-backed forward iterator, accepted pair theorem, actual
runtime len/size_hint, unsafe pointer-writing drain and error conversions. No
production algorithm was replaced. Public input is arbitrary Rust str, measured
in UTF-8 **bytes**, with no ASCII precondition.

* Success has exactly input_byte_length/2 bytes, each equal to 16*high+low under
  the independent numeric ASCII digit policy; mixed case and empty input work.
* Odd byte length wins before character errors. Otherwise the error contains the
  first invalid original byte and its original byte position, including native
  high-nibble priority. Existing next/nth skipping/state/saturation goals remain.
* Real allocation capacity is at least the request. Exclusive tracked ownership,
  same-allocation provenance/bounds, initialized byte prefix and receipt transfer
  justify the original writes and set_len. One-past advancement is allowed;
  one-past writes and exposing uninitialized bytes fail. Early Err retains native
  Vec length zero and relinquishes byte permissions before allocation cleanup.

Engine implementation: `6d80948ce8c981bd650ae851cd99c64cf7625f5d`.
Native implementation: `54b6672c00c2cbeac8bcd7c6761cf1b0bec1a6a2`.
Application proof target: aarch64-apple-darwin, 64-bit usize.
Baseline engine `5b8a980e9`; native `ae20db0`; upstream `129db6a` (1.3.0).
Accepted v1/v2 sources/archives, the prior unaccepted probe, and Aleph v3/v4
inputs/evidence/fixed engine pins remain unchanged.

The [charter](charter.md) and [trust inventory](assumptions.md) bound the capability.
The four minimal trusted Vec boundaries are allocation, exclusive acquisition,
bounded pointer advance and receipt-checked set_len. Byte writes/splitting,
initialization preservation, sealing, abandonment and prefix assistance are
ordinary checked library bodies using existing tracked raw-pointer machinery.
No first-party table/iterator/drain equation is assumed. Proof-only lowering
validates resolved core/alloc primitive identities and u8/Global instantiation,
retains normal lifetime/body checking, and erases to the untouched native body.

Acceptance results:

* **37 results, zero errors**, including 32 named selected executable bodies and
  the retained table/initializer proof. Four added application obligations are
  runtime length, reachable drain domain, original drain and public endpoint.
  Nine semantic goal functions (seven retained, two added) are separate from the
  two new native endpoints and from auxiliary/tooling counts.
* The pair remains the single exact inferred helper, with all **three mandatory
  soundness/maximality/exactness certificates**. The new implementation contracts
  and required semantics pass ordinary body verification; they are not reported
  as additional inferred exact helpers.
* **24 injected native mutations reject**, including table entries, nibbles,
  invalid acceptance/priority/payload, UTF-8/odd lengths, chunking/skipping/state,
  positions/saturation, runtime hints, pointer stride, capacity, initialization,
  output length and public error conversion. These are controls, not upstream
  defects. Without-assistance, no-verify, filtered and no-lifetime publication
  controls reject. Restored sources replay freshly with matching hashes.
* **294 inference regressions pass**. The three new test groups cover an unrelated
  native buffer filler, cold/warm cross-crate values and first-error composition,
  linked erased execution, actual runtime hints, no-vacuity, ignored overrides,
  no-flag publication, permission duplication, stale receipt/wrong allocation,
  drop/reallocation, uninitialized exposure and unsupported alias/adapter cases.
* Const/static **57 pass** (two existing ignored), external traits **43**, unwind
  **17**, type invariants **90**. One stale external-trait fixture now uses last
  rather than redeclaring the pre-existing vstd count model; its intended split
  specification and forbidden-call coverage are retained.
* Vstd **2219 verified**, tooling only (2215 baseline plus four checked bodies).
  Native hex default/all/no-default/alloc-only tests pass on **1.99.1**, and MSRV
  **1.74.0** passes. The independent filler passes native/MSRV tests. A proof-
  compiled hex rlib passes a linked native UTF-8/output/error smoke test.
* Parser audit preserves all **218 upstream signatures and executable bodies**.
  No native assumed specification, external body, proof block, assertion, loop
  invariant, manual trigger or blanket implementation exclusion is added.
  Existing public required semantics and v1 digit/pair policy bodies are pinned
  independently. Cargo configuration, lockfiles and feature defaults are intact.

The internal drain domain is cursor + remaining_pairs <= usize::MAX/2, proved for
every reachable forward string state; it adds no public precondition. Synthetic
cursor saturation remains a separate iterator theorem. Fixed arrays (including
2*N overflow), backward/mixed iteration, the public forwarding wrapper, slice
unsafe drains, arbitrary T/destructors, aliases/reallocation/raw reads/escapes,
and general unsafe Rust remain outside this finite acceptance. Unsupported
profile rejection is distinguished from ordinary failed safety proofs.

Replay from the engine root with matching compiler/macros/vstd, metadata schema
38, policy **54**, cache **60**, and Rust 1.99.1:

```sh
python3 source/tools/check_hex_vec_adoption.py \
  --native-root /Users/tommyvolk/Documents/GitHub/hex-conservative \
  --output /private/tmp/hex-vector-replay-new
```

The controller needs native write permission, restores each mutation in finally,
uses fresh per-proof cache/export paths and --no-cheating, then runs offline
locked native feature/MSRV tests. `acceptance-committed` is the accepting series
against the rebuilt 6d80948 engine; earlier runs are development evidence. An
unsound development attempt at trait defaults was exposed by a len control and
removed before acceptance. Policy 54 rejects those prototype exports. The final
binary banner names 6d80948; source/artifact hashes bind the complete build.

See [closure](closure-summary.json), [controls](controls-summary.json),
[source audit](source-audit.json), [required policy pins](requirements-preservation.json),
[engine receipt](engine-source-receipt.json), [preservation](preservation.json), and
[raw evidence index](raw-evidence-index.json). The native archive includes the
committed checkout and its ignored matching Cargo.lock. All commits are local.
