# Checked bounded word-view goal automation

This is an engine extension with an explicit generic representation vocabulary,
not automatic discovery of an application's semantic model. `--infer-goals`
now selects three ordinary checked vstd broadcasts for demanded u64 membership
and word-sequence observations. With that vocabulary, the original stateful
executable bodies and original semantic contracts verify without application
snapshots, proof blocks, representation assertions or theorem calls.

The literal historical baseline still fails five postconditions. A forwarding
membership spec wrapper still leaves two postconditions unproved. The accepted
reduction uses a direct generic membership alias and generic word-view definition;
these changes are explicit, and their equivalence to the original definitions
is separately proved. No smaller domain or executable rewrite was substituted.

## Frozen inputs and evidence

Target/delta frozen at `d3bbd34e`; initial fixtures and their hashes at `4ea46ae1`.
The historical experiment at `4c501fd19f51ae8cfa1f5033c5316769b622a413`
(earlier freezes `6a8f807b`, `72898742`) is retained byte for byte in
`../redb_single_word_set`, including its report, receipts and baseline. It was
not revised to claim automation. Initial compiler/library source is the prepared
f97ce98f / production 3b05c191 state. The clean pinned upstream redb checkout
was read only. The new target retains one physical word, 1..=64 logical pages,
allocated padding, arbitrary membership, bit < len and repeated allocation.

An own release compiler and matching verified vstd were built with Rust 1.99.1,
`PATH=/home/dev/.cargo/bin:$PATH`, `CARGO_BUILD_JOBS=2`. Standalone Z3 was copied,
not linked. Before connecting the new pass, the unchanged build verified 2,221
vstd bodies and reproduced exactly the original five postcondition diagnostics:
3 verified, 1 failed function, full crate, no frontend/safety/resource failure.
Baseline receipt:
`/tmp/infer-word-automation-20261109/single-word-set-vwms4mym/receipt.json`.

Fresh final receipt:
`/tmp/infer-word-automation-20261109/word-goals-rxf1dufs/receipt.json`.
`RESULTS.json` records fixture/tool/log hashes, outcomes, selected laws, candidate
identities, fresh precision results and imported dependency fingerprints. All
proof runs use full checking and `--no-cheating`; no lifetime or verification
bypass is used. Raw logs, trial sources and the stable baseline binary bundle
are under private `/tmp/infer-word-automation-20261109`.

## Mechanism and measured reduction

`vstd::word` supplies a u64 membership function and finite sequence view. Five
proof bodies check the OR-bit and full-word BV facts, the integer-indexed
membership bridge, view update/count using the existing count-prefix update
lemma, and full-view equivalence under explicitly allocated padding. The matching
full vstd build verifies **2,226 bodies, zero errors**. The new module explicitly
enables the existing sequence theory internally; its laws are not globally
broadcast by default to clients.

`vir::infer::word_goals` follows supplied contracts/assertions through at most
32 non-vstd spec definitions and 4,096 expression nodes. It selects the complete
fixed three-lemma catalog at exact vstd identities and expected safe proof
signatures. Exhaustion discards all discoveries. It adds only scoped broadcast
fuel, preserving original statements, tail, body identity and all contracts.
Library proofs, inferred candidates, external/unsafe/async bodies and suppressed
termination are excluded. It performs no application or body-pattern recognition.
JSON strategy `word_view_broadcast` reports selection, never proof acceptance.

The accepted `automatic_alias.rs` replaces only the original membership definition
with `pub use vstd::word::member` and the `allocated` implementation with
`vstd::word::view`. The erased/executable bodies and every application contract
remain unchanged. The separate `representation_equivalence.rs` audit proves both
representation definitions equal for all words and lengths, and is never called
by accepted fixtures. Replay checks the exact permitted textual adaptation and
the historical extracted body bytes.

Compared with historical assistance, accepted application bodies remove both
ghost snapshots, all three mechanical theorem calls and the entire local proof
block (index, membership, invariant, extensionality and padding bridges). Remaining
authored work: the semantic invariant, view/count policy, input bounds and seven
set postcondition clauses; representation imports/definitions; and helper contracts
unless separately inferred below. No semantic policy is inferred from bitwise OR.

## Fresh outcomes

Counts below are verifier query results, not necessarily source-body counts;
precision subqueries contribute to the inferred-mask and helper totals.

| Fixture | Verified / failed | Outcome |
| --- | --- | --- |
| historical baseline, flag enabled | 3 / 1 | five original postconditions unproved |
| representation equivalence audit | 1 / 0 | original and generic definitions equal |
| generic view with forwarding member wrapper | 3 / 1 | invariant and return still unproved |
| direct generic alias, original stateful bodies/goals | 4 / 0 | no local proof assistance |
| same direct alias, flag disabled | 3 / 1 | five postconditions unproved |
| unrelated permission grant + idempotence/count | 2 / 0 | no local proof assistance |
| stateful fixture with inferred mask helper | 18 / 0 | all three exact checks proved |
| permission OR -> XOR | 0 / 2 | four postcondition diagnostics |
| stateful OR -> XOR | 3 / 1 | five postcondition diagnostics |
| padding requirement removed (negative only) | 3 / 1 | return postcondition rejects broadened domain |
| return equality -> inequality | 3 / 1 | return postcondition rejects |
| wrong count delta | 1 / 1 | count postcondition rejects |
| isolated tuple helper inference | frontend rejection | unsupported tuple return signature |
| isolated shift helper inference | 16 / 0 | soundness, maximality, exactness proved |
| cold / warm inferred provider | 3 / 0 each | cache miss / hit; three fresh checks each |
| separate-crate consumer | 1 / 0 | imported identity/fingerprint retained; word law selection transfers |
| valid inference + failing ordinary assertion | 3 / 1 | prior exported interface removed; no new publication |

Semantic negatives have only postcondition failures, no frontend, precondition,
assertion or resource failure. Additional mechanically unproved XOR properties
are not all claimed semantically false. The publication control deliberately
fails an ordinary assertion after a valid inferred candidate and checks a
previously populated export path is absent afterwards.

Existing inference **can replace `select_mask`'s authored requires/ensures**
without rewriting its original executable body: the inferred precondition is
`(bit as u64) <= 63` and the result equation is the original shift. Both an
isolated caller and the complete stateful fixture prove with fresh soundness,
maximality and exactness checks. `automatic_inferred_mask.rs` retains only the
`data_index_of` helper's authored tuple-component equations. Existing inference
rejects that helper's tuple result before candidate generation; it was not
rewritten to two functions or a scalar wrapper.

The three new selector unit tests pass. Existing projection safeguards (5),
registry/cache tests (7), and proof-evidence tests (9) also pass. The replay guard
tests (2) and historical guards (5) pass. All 28 integration tests pass: infer_counting (13), infer_goals (12), and
infer_word_goals (3). Test-only optimization was reduced for build time; verifier
binaries remained the own release build. Initial test-fixture ownership and
negative error-location marker mistakes were corrected; failed trial logs remain.

## Limits, trust and integration

There is no new assumed spec, axiom, unchecked summary or first-party assumption.
The engine delta is untrusted selection machinery whose proposals
remain checked through ordinary Verus verification. New library bodies are
proved with the existing backend/Seq/count theory. Existing compiler/solver,
vstd infrastructure and Vec contracts retain their usual trust status.
Inferred candidate generation, projection gates, dependency identity,
fingerprints, fresh required precision checks and publication code are unchanged.

Arbitrary wrapper-trigger bridge synthesis remains a concrete blocker: the
forwarding membership wrapper exposes the view for selection but cannot connect
both padding quantifiers automatically. Canonical representation symbols are
currently required for the full stateful reduction. The plain historical baseline
and arbitrary authored views are not automatically solved. No multiword bitmap,
allocator search, clear/free, transaction, durability or concurrency result is
claimed. No fresh native exhaustive/erasure test is claimed; exact executable
fidelity is guarded, and historical native evidence remains historical.

Policy/cache constants remain v63/v69 in this isolated base: candidate formulas
and acceptance policy were not changed. Parent's reserved v64 full-scope
publication guard must be integrated separately; the reported ordinary-only
`--infer-goals --no-verify` publication gap is pre-existing and this work does not
fix or use that bypass. Rebuild compiler/macros/vstd and providers after integration.
Shared-file touches are one module declaration, one pipeline hook, one report
field and one reporting chain, plus vstd's module declaration and documentation.
No independent review is claimed; the requested integration review remains separate.

Replay after building the matching bundle:

```sh
python3 -B source/tools/research/word_goal_automation/test_replay.py
python3 -B source/tools/research/word_goal_automation/replay.py --output /private/existing/directory
```

Failed exploratory trials were retained: the initial missing broadcast trigger,
the forwarding-wrapper bridge failure, a rejected exploratory conditional padding
proof (discarded), and internal vstd attempts before the explicit sequence import
was actually loaded. None is presented as accepted proof evidence.
