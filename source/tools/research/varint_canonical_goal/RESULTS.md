# Full-domain canonical varint proof assistance

**Standalone canonical milestone accepted; the combined mutable-buffer goal is
not complete.** The exact original canonical theorem now verifies on every u64
and arbitrary initialized exclusive `[u8;10]`, preserving the original layout-free
executable macro, signature, checks and predicate.

| Result | Verified/errors | Required local checked bodies |
|---|---:|---|
| Frozen manual ordinary proof, candidate004 |9/0|exec `{u64}`, proof `{radix_positive, quotient_step, byte_step}`|
| Directed inline diagnostic, candidate006 |6/0|exec `{u64}`, proof `{}`; local mechanical blocks remain|
| Accepted application, [candidate008](candidate-008.rs), assisted cold/warm |6/0 each|exec `{u64}`, proof `{}`, inferred helpers `{}`|
| Separately named safety/length goal |6/0|exec `{safety_length}`, proof `{}`|
| Certified importing caller |1/0|exec `{caller}`, proof `{}`|

All application runs use normal full-crate `--no-cheating` verification, with
lifetime, trait, body, well-formedness and mode checks. Candidate008 has no local
mechanical proof blocks, theorem calls, helper proof functions or ghost snapshots.
The executable loop, indexing, early break, byte operations, counter and debug
check remain unchanged. No input-domain or layout restriction was added.

## Authored input and manual work

The original `radix_power` and `canonical` definitions are byte-for-byte unchanged.
Three visible inductive semantic inputs remain: remaining quotient, nonzero
quotient after a continued iteration (minimality), and the final-referent written
prefix with radix payload and continuation bits. Four visible checked exit
obligations state zero remaining quotient (also serving the debug check), upper
bound, minimality and the completed byte prefix with the terminal bit cleared.
These clauses are obligations, not assumed facts or inferred output summaries.

The frozen diagnostic helpers in candidate004 remain explicitly manual work:
positive-radix induction, shift/division composition and bounds, byte cast/mask
queries, iterator alignment/capacity/bounds, and initial division-by-one. They
are excluded from the accepted application. Candidate006 first demonstrated
that directed applications of existing vstd laws and bounded unfolding suffice;
the broad-broadcast candidate005 exhausted resources and is excluded.

The bounded pass now proposes the evidenced mechanical work: three checked
iterator/alignment/bound invariants, initial division law, conditional existing
arithmetic laws, two checked byte bit-vector queries, positivity assertions,
two checked computations, and scoped radix/power fuel. It introduces no new
library law, application assumption, effect allowlist or inferred encoder summary.
The original prophetic iterator model already expresses the required written
prefix; no new observation/frame law was necessary for canonical bytes.

## Source and trust gates

The profile intentionally covers only u64/step7/capacity10. It matches the resolved
canonical slice `iter_mut` specification identity (including its encoded proxy),
array unsizing/types, direct scalar/counter initialization, four source writes,
unit continuing increment and zero-break/mask shape. Unknown calls or loops in
the source loop reject; source traversal and radix unfolding have finite budgets.
Radix and arithmetic catalog signatures/modes/generics/proxies are checked before
constructing proof calls. Other shapes receive their unchanged ordinary obligations.

An independent validator removes exactly the recorded initial and step ghost
statements and generated invariants, then compares the **entire original VIR
function**, including source body, signature/contracts and termination metadata.
It checks placement/counts and rejects executable/contract mutation, changed
original invariant/decrease, duplicate or misplaced insertion. Every generated
expression receives a fresh noncolliding identity; scalar reads are Spec-only.
The three dedicated unit tests and all143 VIR unit tests pass. All8 focused
verifier tests pass, including zero/maximum callers and a generic-radix rejection.
Source-token and exact predicate/signature projections also pass; all historical
charter freeze hashes and earlier local candidate freezes remain unchanged.

The immutable private tool COPY was built from committed production changes
through3e3b4e4c. Its reported version is `0.2026.11.09.3e3b4e4.dirty` because
research/controller work was pending at build time; exact production source and
tool hashes are bound in [manifest.json](manifest.json). Matching vstd was fully
rebuilt and checked:2226/0. Its standard build retains the inherited vstd trust
boundary; forwarding `--no-cheating` into vstd itself rejected83 inherited
assume/admit uses and is retained as an excluded setup run. No new vstd law or
assumption was added. Every application/assistance trial retains `--no-cheating`.
Rust1.99.1, the copied accepted Z3, rustc/Verus/macros/SMT and the existing array,
iterator and range contracts remain trust boundaries. No native suffix or pointer
identity theorem is claimed. Policy67/cache70/schema38 were not changed by this
worker; integration/version reconciliation belongs to the parent.

## Accepted controls and retained failures

The committed controller completes **26 cases**. The independent [audit](audit.py)
rechecks exact case/input identities, all named bodies and all mandatory helper
proof statuses, source fidelity, certified import identity and raw artifact/log
hashes. In ordinary-only cases the helper set is explicitly empty. Healthy-helper
cases bind `identity` and required soundness/maximality/exactness proofs before
checking the original caller obligation.

- Canonical cold/warm6/0, separate safety/length6/0, healthy exact helper9/0,
  provider cold/warm9/0 and certified import1/0 pass.
- Four false-property callers and seeded failed publication reject only their
  authored postcondition, with healthy exact helpers; the seeded export disappears.
- Disabled assistance, wrong shift/break/counter/payload/continuation, step8,
  capacity9, width32 and generic-radix cases reject through unchanged ordinary
  obligations with no numeric proposal. They are diagnostic/unsupported-shape
  controls, not strict semantic evidence or claims of wider coverage.
- Wrong returned range rejects its authored postcondition8/1 with healthy helpers
  and checked loop assistance. The inherited truncating-cast recommendation note
  excludes it from the strict semantic-negative gate; its sole error kind/count
  and exact selected-body set are checked separately.
- Intentional typed cached-helper poison is separate: required soundness and
  exactness are `not_proved`, required maximality is `proved`, the cache is hit
  and export is suppressed. Invalid metadata independently rejects import.

Initial invalid triggers, missing/misspelled lemmas, mixed diagnostic failures,
resource exhaustion, two skipped prototype trials, a duplicate-AST-ID panic,
a solver-environment omission, fixture attribute placement and controller cache
shape mistakes are retained and excluded. Fresh fixed runs are new receipts.
The manifest records every failed receipt and build/test log hash; raw logs and
binaries stay private under `/tmp/varint-canonical-proof-assistance/`.

## Replay and integration boundary

From `source/`, build own matching compiler/macros/vstd per BUILD.md with
`PATH=/home/dev/.cargo/bin:$PATH`, `CARGO_BUILD_JOBS=2` and an explicit solver.
The build commands are `cargo build --release -p rust_verify -p verus -p cargo-verus`
and `./target-verus/release/cargo-verus build --manifest-path vstd/Cargo.toml --release`.
Then, from the checkout root:

```sh
python3 -B -O source/tools/research/varint_canonical_goal/copy_bundle.py \
  --from "$PWD/source/target-verus/release" --solver /matching/z3 --output /private/new-bundle
python3 -B -O source/tools/research/varint_canonical_goal/replay.py \
  --bundle /private/new-bundle --output /private/replays
python3 -B -O source/tools/research/varint_canonical_goal/audit.py \
  /private/replays/canonical-XXXX/receipt.json --output /private/audit.json
```

The accepted worker receipt is
`/tmp/varint-canonical-proof-assistance/replays/canonical-96zf4he4/receipt.json`;
its independent audit is `/tmp/varint-canonical-proof-assistance/accepted-audit.json`.
The parent independently reproduced cold/warm6/0 and healthy-caller publication
rejection in its own private COPY, reported in parent commit e4011e16. Those
parent-owned receipts are supporting evidence, not modified by this worker.

**First remaining global blocker:** the independently frozen HMAC prefix plus
untouched-suffix theorem and reuse of an actual checked mechanical step across
both writers. A shared parser alone does not demonstrate transfer. This numeric
profile does not match HMAC Zip. Combined integration, fresh review and replay
remain parent work; no full-goal completion is claimed.

Composition must retain full function well-formedness/mode checking after all
insertions and refresh resolution information. The numeric pass rejects calls
inside its source loop body, so order it before added body proof calls or jointly
validate their recorded ghost-only decorations. Root ghost annotations and loop
invariants can compose; identity collisions fail closed. Shared observation/length
steps may be factored by the parent without changing this standalone evidence.
