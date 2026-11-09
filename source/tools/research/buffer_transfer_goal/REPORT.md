# Independent transfer remains blocked

The frozen second target is an exact source slice from bitcoin_hashes 0.14.0,
Fedimint's pinned dependency: the HMAC short-key `ipad.iter_mut().zip(key)` XOR
loop. Selection and source bytes were committed at `7fe03428` **before the first
proof trial**. The domain remains every initialized exclusive `[u8;128]` and
every immutable key slice, including keys longer than 128. No precondition was
added. The frozen full property specifies **all** resulting bytes: XOR below
`min(128, key.len())`, unchanged original pad above it. Rust exclusivity supplies
immutable-key preservation. This is neither a whole HMAC proof nor a security
claim. The original callsite uses a subset of this slice's generalized domain.

No new reusable assistance/law has been used by this target. The canonical
transfer completion criterion is **not** met. Safety and a manually annotated
written-prefix theorem are separately useful partial results.

## Proof outcomes

The immutable private accepted bundle identifies runtime `0c00b6ec`,
policy 67/cache 70/schema 38, Rust 1.99.1 and Z3 4.16.0. Compiler, macros and vstd
were copied byte-for-byte, not symlinked or rebuilt, and guarded throughout
accepted replays. `bundle-identity.json` records source/copy paths, SHA-256 and
separate inode identities. Normal `--no-cheating`, lifetime/trait checking and
whole-crate verification remain enabled.

- Frozen unannotated full goal: **1 verified / 1 postcondition error**, goals
  on/off and cold/warm. No target theorem was already automatic at baseline.
- Safety alone: **2/0**. Adding bytes instead of XOR reports only arithmetic
  overflow, a safety control, not a semantic result.
- `candidate-01.rs`: processed-reference XOR invariant is maintained; **1/1**
  at the full postcondition. No snapshot, proof block or theorem call.
- `candidate-02.rs`: mentioning `final(ipad)` inside the active mutable borrow
  is rejected by lifetime checking (**E0502**, zero SMT proof errors). This
  frontend/lifetime failure is excluded from semantic evidence.
- `partial-prefix.rs`: the same reference invariant without observation bridges
  does not prove even the written-prefix postcondition: **1/1**.
- `partial-prefix-manual-bounds.rs`: **2/0**, preserving the original domain
  and executable loop, with two sequence snapshots, min-length and observation
  bridge invariants. This property omits the suffix **only as an explicitly
  separate partial theorem**; it never replaces the frozen full goal.
- `full-manual-bounds.rs`: restoring the full frozen `pad_xor` contract with
  those same manual annotations still gives **1/1**, only the postcondition.
- `frame-repro.rs`: **0/2**, both failed postconditions. Merely creating and
  abandoning an IterMut does not prove an unchanged array; the proposed generic
  drop-frame interface is also unproved. Its body is an ordinary failed proof,
  never an axiom, assumed summary or imported certificate.

The unbounded-recommendation variants without explicit snapshot-length
invariants are retained unchanged. Their later bounded variants retain exactly
the same semantics and executable body, adding only the two length facts.
Every source/semantic-input candidate has its own pre-trial freeze; earlier
candidate records remain unchanged. All cases explicitly bind **zero** inferred
helpers before checking the shared named mandatory-obligation validator. There
is no helper cache hit or exact-inference claim: cold/warm refer to independent
processes with separate/retained private candidate-cache directories. Ordinary
semantic negatives retain the healthy empty helper set and exclude safety,
frontend, invariant, resource and mixed diagnostic failures. Intentional helper,
cache or metadata poisoning is not a relevant control for a nonexistent helper
candidate and is not claimed here.

## Missing interface and trust boundary

A useful exact typed failing dependency appears in `frame-repro.rs`:

```rust
pub proof fn iter_drop_frame<'a, T>(iter: IterMut<'a, T>)
    requires has_resolved(iter),
    ensures forall|j: int| #![auto] 0 <= j < iter.remaining().len() ==>
        *final(iter.remaining()[j]) == *iter.remaining()[j],
```

The existing standard-library IterMut contract links initial/final referents to
the enclosing slice, but does not constrain final values of unyielded referents.
The existing Zip specification truncates the modeled returned sequence; it
provides no resolution/frame projection for its unreturned IterMut tail. A
checked generic observation law can help the written prefix, but cannot simply
assert the missing unconsumed frame. This is a model-boundary investigation,
not evidence the real Rust program violates the suffix property. No trusted
standard-library contracts were strengthened, no new assumptions were added,
and no worker engine/library code was copied or inspected before selection.

The first remaining blocker is a justified unchanged-tail model/law compatible
with the existing trust constraints. It must actually connect the original
source's iterator drop/zip behavior to the final array. This workstream owns no
engine or vstd changes and does not duplicate the effects worker's module.
Parent was sent the exact frozen zip client, typed interface and partial prefix
proof; independent native/source work continued while that interface remained
unsettled.

## Authored and manual work

The authored semantic inputs are the full `pad_xor` definition/postcondition and
the processed-prefix XOR relation. The source-slice boundary, arbitrary pad/key
domain and original call-context relation are also authored choices. In the
ordinary successful **partial** proof, manual mechanical work consists of:

1. Ghost `before = ipad@` and prophetic `future = final(ipad)@` snapshots.
2. Both snapshot lengths retained as 128 in the loop.
3. Zip length equal to `min(128, key.len())`.
4. Three pointwise bridges: initial pad byte, immutable key byte, eventual pad
   byte versus the corresponding pair referents.

There are no local proof blocks or theorem calls in that partial proof, but its
snapshots and bridges violate the intended final annotation boundary. There
is no claimed mechanical proof reduction or shared-automation transfer. The
snapshot-free frozen candidate is still blocked. Compiler/macros, SMT, existing
vstd axioms/standard-library contracts, and Rust's implementation/model
correspondence remain trust boundaries; native testing does not discharge them.

## Native/source correspondence

`native_replay.py` validates the crate archive against Fedimint's exact lock
checksum, extracts privately, and checks the selected source against the
committed upstream file. It compiles the whole real bitcoin_hashes dependency,
instantiates `HmacEngine<sha256::Hash>::new`, and compares the exact upstream
source loop with the **actual Verus macro-erased** baseline and manual fixtures.
Each debug/release run checks 2,816 cases across arbitrary initialized bytes and
key lengths 0, 1, 2, 63, 64, 65, 127, 128, 129, 255 and 1024, including every
untouched suffix byte and immutable key. An OR mutation produces observable
differences. These finite checks are supporting evidence, never proof.

The committed native Cargo.lock freezes the actual independently resolved build
closure; it is not relabeled as Fedimint's entire dependency resolution. Package
and lock hashes are recorded. Both Cargo target and build directories are
private `/tmp`, commands run from `source/`, `CARGO_BUILD_JOBS=2`. An initial
native build accidentally followed `source/.cargo/config.toml`'s separate
`build-dir` despite `CARGO_TARGET_DIR`; the solely generated local ignored
artifacts were moved to private `/tmp` and the corrected replay overrides both
paths. No parent/application checkout or artifact was changed.

## Replay

```sh
python3 -B -O source/tools/research/buffer_transfer_goal/replay.py \
  --verus /matching/verus --solver /matching/z3 --output /private/transfer
python3 -B -O source/tools/research/buffer_transfer_goal/native_replay.py \
  --archive /cache/bitcoin_hashes-0.14.0.crate \
  --verus /matching/verus --solver /matching/z3 --output /private/transfer-native
```

The verifier replay uses shared `infer_experiment.py` and
`infer_eval_support.py`, explicit `require` gates (also under optimized Python),
immutable artifact guards and command/failure receipts. It validates expected
failures, not completion of the full goal. It includes a healthy partial export
seed followed by failed full-goal publication/removal. `results.json` records
compact accepted outcomes, hashes, raw receipt paths and excluded development
failures. During controller development a script edit overlapped an active replay;
the artifact guard correctly rejected that entire batch. It is excluded, and
the final controller was committed before the accepted rerun. Raw JSON/logs/
binaries stay under private `/tmp`; nothing was pushed.
