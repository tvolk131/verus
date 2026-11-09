# Insert argument, backend footprint and remaining native obligations

## Accepted starting point

The accepted `transfer-prefix-integration79` infers all three original native
key/range helper contracts. The old cycle78 ZERO-INFERENCE language and the
earlier atomic-replay ZERO-native-correspondence language describe those
historical milestones; they are not the current inference baseline. All baseline
historical bytes are retained. The atomic-replay model's attempt/recovery,
marker-lifetime and reachability conditions are not discharged here.

`source-ledger.json` records five whole pinned source/lock hashes and five exact
extracted items. The transaction record keeps all three fields, their order and
types, including generic `Inner`, `Option<GlobalDBTxAccessToken>` and the token's
native `u32`. The native key helper's full original method item is unchanged,
private and generic. Only an external Infer attribute is added. Its inferred
precondition is `true` and its exact result is `prefix + key` over byte views.
The native async insert method and its trait declaration are byte-identical in
the generated Rust harness and async verifier repro. There are no inserted ghost
statements or changed executable statements in either extracted native method.

## What the new theorem says

The author-owned executable `prepare_insert` calls auxiliary `key_contents`,
which instantiates the original generic native key helper with `Inner = u8`.
The provider checks that generic native helper over its original domain; `u8`
instantiation is only import plumbing. The adapter returns the exact native
helper-produced target and proves this implication for arbitrary byte prefixes,
keys, values and map states:

```text
target = prefix ++ key
backend_insert(before, after, target, value)
    := after == before.insert(target, value)
therefore:
    outside prefix, lookup(before, other) == lookup(after, other)
    at target, lookup(after, target) == Some(value)
    at every unequal key, lookup(before, other) == lookup(after, other)
```

`lookup` distinguishes absence from present values, so the frame includes the
domain, not merely values at pre-existing keys. Another checked proof derives
an unchanged **peer namespace** from supplied prefix-incomparability. That
condition is not inferred and is not established for actual native encodings.
The executable adapter's quantified conditional frame and peer lemma are also
checked through certified paired VIR/rlib import. The adapter is **authored
preparation code**, not the async method with `await` removed.

The backend relation is a supplied key-level contract for a successful event in
a transaction-local byte-key/value map view. It is independent of namespaces;
it does not assume `outside_frame`, peer separation or the application goal.
It is stronger than an unconstrained trait signature and includes a single-key
map footprint. **No theorem asserts this relation for RocksDB**, and no external
body or assumed specification installs it. These are ordinary explicit theorem
premises. Thus this is a checked source-connected operation-preparation/encoding
lemma with a conditional backend footprint, not a verified backend footprint.

The relation does not constrain the returned previous value. Native RocksDB
uses `snapshot().get(key)` followed by `put(key, value)` in `block_in_place`;
the exact meaning of that snapshot/transaction/FFI composition must be proved
before assigning a production old-value contract. The demonstrator backend's
return semantics are used only in finite forwarding tests. Errors, panics,
cancellation, notifications, locks, logs, allocators and arbitrary external
effects are outside the map relation. No atomic commit, rollback, durable image,
recovery, consensus, funds or full-transaction invariant is claimed.

## Source-connected but unproved native steps

| Native source | Observed connection | Still required |
| --- | --- | --- |
| `db/mod.rs:823` | Full original async insert item calls the checked generic key helper, then forwards `&key` and unchanged `value` to the inner operation | A verifier-supported async/trait translation, polling/borrowing/aliasing semantics, exact dispatch and returned-result simulation |
| `db/mod.rs:450`, `:1691` | `with_prefix` / `to_ref_with_prefix` put supplied arbitrary prefixes into wrappers | Constructor/ownership/aliasing correspondence; arbitrary supplied prefixes are not guaranteed disjoint |
| `db/mod.rs:687`, `:464`, `:1714` | Module-id constructors call `module_instance_id_to_byte_prefix`; it appends `u16::consensus_encode_to_vec()` after `0xff` | Correct encoding and prefix-incomparability for distinct actual module IDs, then constructor-to-wrapper correspondence |
| `encoding/mod.rs:521`, `:543`; `encoding/btc.rs:324` | u16 uses BigSize through the native encoding adapter and its writer | Variable-length BigSize correctness/prefix-freeness; it must not be replaced by invented fixed-width bytes |
| `fedimint-rocksdb/src/lib.rs:333` | Backend insert reads `snapshot().get(key)` and invokes `put(key,value)` | An abstraction relation from transaction/backend state to the byte map, supplied-key write footprint and successful-result semantics; Rust/C++/FFI and physical/external effects |
| `db/mod.rs:802` | Matching global access token intentionally returns the unprefixed inner transaction | Confinement applies only to the selected prefix-mediated path, not global-token use or arbitrary module behavior |
| `db/mod.rs:832`, `:837`, `:860`, `:872` | Read/remove/range/prefix-delete have their own forwarding, stream and backend contracts | Not proved or executed by this bounded insert link; no range ordering or prefix-stripping theorem is added |

The backend must relate a completed event's supplied key to its map effect,
including any necessary concurrency/linearization discipline. Ownership and
exclusive borrows in a source excerpt do not supply a production map abstraction
or prohibit unrelated effects. This handoff does not create a general-purpose
unverified backend summary and count it as a new inferred capability.

## Exact Rust extraction boundary

The demonstrator retains native async syntax, `.await`, full record fields,
generic bound and full method items. Derive and async-trait attribute expansion
are not compiled. The local trait contains only the selected exact insert
declaration; the other six core operations are not projected into fake no-ops.
`DatabaseResult`, marker trait and backend are harness definitions. `MaybeSend`
is represented by a local Send marker; static Rust async-trait dispatch is not
Fedimint's boxed/dynamic async-trait macro expansion. The native application is
not built, linked or modified. This restriction is why Rust test success is not
called a universal native async correspondence.

The normal matrix covers five supplied prefixes (including empty and
variable-length-shaped examples), four keys, three values and two prior-value
states. It checks exact call count/key/value, result passthrough and the
demonstrator's complete map update. Literal prefix examples are not an execution
or proof of native module encoding. Additional tests preserve actual polling and
dropping of the extracted async body, rather than replacing it with a
synchronous transaction model.

## Counterexamples and reachability

- Checked Verus witnesses show comparable prefixes `[1]` and `[1,2]` can address
  the same byte key, and an unconstrained after-map can change an outside key.
  Neither witness satisfies the missing assumption in the corresponding false
  stronger claim. These are not discoveries of Fedimint bugs.
- `with_prefix` accepts arbitrary bytes: overlapping user-chosen namespaces are
  not forbidden by that API. This does **not** show that actual distinct
  module-id BigSize encodings overlap; no native module-id separation theorem or
  counterexample is claimed.
- Pending-after-write, error-after-write and extra-write backends are reachable
  in this demonstrator trait substitution. They show why a signature and correct
  prefix arithmetic alone do not imply rollback or a backend frame. They do
  **not** establish such behavior as reachable in native RocksDB or production
  Fedimint. Dropping the demonstrator future preserves its deliberately staged
  effect; no production cancellation/rollback conclusion follows.
- The global-token escape is intentional source behavior, not a vulnerability
  established here. A full module-confined-operation claim must exclude or
  separately model that path and all other effects.

Actual Fedimint bugs established by this handoff: **zero**. Unproved native
obligations and disproved stronger model claims must remain distinguished.

## Trust inventory

Author-owned: the six map/prefix definitions, three footprint/separation lemmas,
two positive counterexample proofs, executable preparation adapter, imported
caller goals, extraction/receipt tools, harness backend and all negative tests.
Inferred: only the original native get-full-key helper and public auxiliary
wrapper, using the already accepted byte-append capability. Backend relation and
peer incomparability remain explicit premises, never inferred or silently
asserted native facts.

Trusted: sealed runtime78 and its nine artifacts, solver, Rust toolchain,
Verus translation and ordinary proof backend, reused vstd Seq/Map and existing
Vec clone/extend/u8 specifications, and local OS/process/hash mechanisms. The
380 reported standard-model entries are the sealed runtime's inventory, not 380
new application axioms or capabilities. Neither std/vstd implementation nor the
toolchain/solver is freshly rebuilt or reverified here. There is no new trusted
external application contract. All native async/backend refinement remains open.
