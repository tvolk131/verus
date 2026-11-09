# Independent review of the combined word/foreign/checking boundary

Review input: `db26cfc7dbd680d8cf58c08d6ef70496558f8b85`, compared with
`d0d12c6f3297439352c1f837a31c8759642a2bb2`. The production deltas reviewed are
`ddad04c5` (word goal selection and checked library), `66350499` (nested
readonly declared foreign leaves), and `db26cfc7` (mandatory checking phases
and conservative library provenance). Historical child reports and earlier
reviews are context, not fresh combined validation. The concurrent tuple
implementation is absent from this snapshot and is outside this review.

## Finding and recommendation

**Fix the single test diagnostic mismatch below, then accept for the stated
scope. No engine correctness or publication blocker was reproduced.**

**P2 — new provenance regression is unconditionally red.**
`source/rust_verify_test/tests/infer_cases/interfaces.rs:260` checks
`"trusted standard-model fingerprint"`, while the actual importer diagnostic
is `"stale or missing trusted-model fingerprint"` (emitted by
`source/rust_verify/src/import_export.rs`). A valid provider and consumer pass;
the poisoned provider correctly rejects. The test then panics at line 259
because the expected substring is absent. Replace the test's expected substring
with `"trusted-model fingerprint"`; no importer relaxation is needed. Production
and regression sources were left untouched for the parent to fix.

Reproduced both in the seven-test interface group and in an isolated run:

```sh
# From source/, using the private matching bundle/build directories below:
cargo test --offline --locked --release \
  --config 'profile.release.package.rust_verify_test.opt-level=0' \
  -p rust_verify_test --test infer \
  interface_cases::ordinary_word_goal_exports_bind_standard_library_models \
  -- --exact --test-threads=1
```

The isolated result is 0 passed/1 failed, exit 101. The full-test CI job includes
this test, so the mismatch should be corrected before integration. Independent
metadata and actual-artifact poison probes use the real diagnostic and establish
that the enforced provenance rejection itself works.

## Fresh validation

- Own compiler/macros and full matching vstd build: **2,226 verified, zero
  errors**. All **139 VIR library tests** pass, zero failures or ignored tests,
  including the three selector tests, two closed foreign-descriptor/receiver
  tests and the existing source/contract/projection/evidence controls.
- Focused integration: **32/32 pass** (13 counting, 12 goals, four foreign
  opaque, three word goals). Interfaces: **6/7 pass**, with only the diagnostic
  assertion finding above failing. Its isolated rerun also fails. The other
  292 tests in the `infer` integration binary were not run.
- Independent `probe.py`: **80 verifier invocations**, 44 successes and 36
  expected rejections, all final expectations satisfied. Every rejected run
  removes/withholds its physical export, including repeated valid seeds.
  These counts include seeds and are not 80 application theorems. Ordinary-only
  zero-candidate providers reject every body-selection/checking bypass; the
  generic tracked duplicate rejects normally and with `--no-lifetime`.
  Certified consumers reject disabled lifetime/trait/body checking with and
  without local `--infer-goals`. Policies 63, 64 and 65 reject independently of
  producer/schema changes. Mutated model fingerprints and a changed vstd artifact
  in a disposable bundle reject, even with an otherwise intact provider.
  Legacy no-flag/no-verify ordinary export succeeds without an inference trailer,
  confirming the explicitly excluded legacy boundary.
- Independent word scope controls: homonymous user symbols select no word laws;
  an assisted executable does not leak its broadcast into a later proof body.
  Nested monomorphic foreign queries verify cold/warm and across imports, with
  fresh required soundness/maximality/exactness. Mutable, by-value, sound-only,
  root, generic and destructor cases reject eligibility; wrong declared identity
  rejects translation. False opaque equality fails an ordinary assertion. A
  single provider combining the word-assisted ordinary function and nested
  foreign inference verifies, its consumer composes both, and an added failing
  ordinary assertion prevents the combined export.
- Fresh word replay: all **18 compiler invocations** meet their expected
  outcomes. Original baseline and forwarding wrapper remain 3 verified/1
  failed function; direct alias is 4/0, permission 2/0, inferred mask 18/0,
  and separate representation equivalence 1/0. Opt-out and XOR/padding/return/
  count controls reject ordinary postconditions, without frontend/resource
  errors. The unchanged tuple helper is still unsupported in this snapshot.
  Shift inference verifies 16/0. Inferred providers verify 3/0 cold and warm,
  with the expected miss/hit and fresh checks; imported consumer verifies 1/0.
  Failing ordinary assertion after a valid inferred helper removes a seeded
  export. Query counts are not source-body counts.
- Fresh full-request Fedimint replay: all **nine verification invocations**
  meet expectations. Opaque translation alone is 3/0 and a legacy ordinary
  export, not an application proof. Complete inferred proof is 10/0 cold and
  warm, checking both original helpers and all three authored functions;
  imported consumers are 3/0 twice. Bare and wrong SHA type declarations reject
  at translation. Guard deletion and false policy each give 9/1 ordinary
  authored-policy failures, retain all required exact helper checks, and remove
  seeded exports. Across the four local inferred runs, 24 mandatory helper
  obligations are freshly proved. No resource/frontend failure is called a
  semantic counterexample.
- Native compilation retains real Bitcoin 0.32.8/bitcoin_hashes 0.14.0 and all
  seven fields with Debug/Serialize/Deserialize/Clone derives. All 21 selected
  registry package versions/checksums match the application's lockfile. The
  independent-source/erased native oracle passes **8,448** boundary/payload
  comparisons and the fixed-0/mutant-10 and `u32::MAX` coverage witnesses.
  This finite test checks correspondence; it is not a crypto or whole-flow proof.
- Fourteen Python guard test executions pass (seven tests each normally and
  under `-O`). The clean redb pin is freshly checked: the struct and all four
  helper/method items match upstream bytes and locations. The full-request
  runner independently checks Fedimint's clean pin, source hashes and exact
  extracted struct/helper items, then checks cleanliness again.

Two review-fixture setup mistakes were corrected before the complete final
probe run: a nested user module needed its own prelude import, and a generic
opaque declaration needed an explicit recursive-type policy. Those trial
receipts/logs and the short tail preflight remain private and hashed in the
manifest. Neither is an engine finding or semantic counterexample. The final
generic negative reaches Infer eligibility, rather than stopping on the missing
declaration annotation. The original matching bundle is unchanged after all
proofs/tests; the poisoned artifact was only in a separate disposable copy.

## Source audit

- `source/vir/src/infer/word_goals.rs:11` matches the exact `CrateId::Vstd`
  `word` paths, static function identity, modes, safety/termination attributes,
  monomorphic parameters, and proof signatures. All three catalog entries must
  exist before selection. `demanded` at line 46 follows supplied contracts and
  assertions through non-vstd spec definitions. Its shared 4,096-node budget
  includes the original body scan, and its 32-distinct-spec limit counts
  wrappers; exhaustion discards discoveries. This can conservatively miss
  assistance; it cannot authorize a theorem. The selector deliberately checks
  identity/signature, not a syntactic copy of the laws' specifications: the
  checked library and bound library artifact supply those meanings.
- `word_goals.rs:112` excludes library/proof/spec/inferred/unsafe/async/external
  bodies and termination suppression. Lines 129–144 insert exactly three
  `Fuel(...,1,true)` statements, clone all original statements and the tail,
  retain the body's source identity, and change no other function field. There
  is no assumption, application update summary, contract replacement or
  executable rewrite. `pipeline.rs:43` places this before sequence-trigger
  assistance and subsequent ordinary verification. Reporting selection is not
  reporting proof acceptance.
- `source/vstd/word.rs` adds exactly five proof bodies: two private BV facts
  and three public broadcast bridges. The OR bridge requires a valid bit; the
  view/count bridge requires `index < size <= 64`; the full-word bridge
  requires every padding bit allocated. The sequence broadcast is internal
  to this module; client word broadcasts are scoped. There are no new axioms,
  assumed specs or unchecked proof bodies in this delta. Existing Seq/count,
  bit-vector backend and solver trust remain.
- The accepted stateful fixture changes representation vocabulary explicitly:
  it directly aliases generic `member` and defines the sequence view with
  generic `view`. `representation_equivalence.rs` independently proves equality
  to the historical definitions, without being called by the accepted fixture.
  The frozen source guard checks those exact adaptations and original executable
  bodies/contracts. The literal old baseline and forwarding wrapper are still
  distinct unsuccessful cases. This is not arbitrary abstraction discovery,
  scalar tuple inference, a multiword allocator or a whole-database proof.
- `source/vir/src/infer/records/foreign_opaque.rs:5` permits only nested exact
  shared-reference leaves; `shape.rs:361` adds precisely that predicate to the
  existing walker. A matching datatype identity and frontend-checked cross-crate
  proxy are required, with no type arguments/parameters/bounds, conditional
  sizedness, exposed fields, destructor, invariant or extensional equality.
  Foreign root, mutable/by-value and sound-only profiles receive no new route.
  Existing first-party opacity is a separate older route and is not tightened
  by this change. The frontend's external-type correspondence is trusted;
  this predicate supplies no representation, constructor, method or crypto law.
- The complete Fedimint extraction keeps the actual SHA-256 type, all seven
  fields, constant and both original helper bodies. Only the proof extraction
  omits derives; separate native compilation includes them. Hash/amount/short
  channel fields are present but unread by these deadline helpers. Authored
  deadline policy, opaque correspondence, native finite testing and universal
  selected-domain proof are different claims. Funding, settlement, authenticity,
  crypto and callsite wiring remain outside scope.
- `source/rust_verify/src/config.rs:139` strengthens the shared certification
  predicate with normal lifetime and trait-consistency checking. Both the main
  guard (`verifier.rs:2927`) and independent exporter guard
  (`import_export.rs:327`) use it. The first covers all goal-assisted providers,
  including empty candidate/profile sets; imported consumers also use it.
  Normal tracked lifetime checks and trait checks run before `finish_verus`;
  its publication barrier (`verifier.rs:3599`) still requires zero driver,
  frontend and proof errors. Full body checking includes all existing filters.
- `verifier.rs:3016` conservatively binds available standard models for every
  `--infer-goals` provider, not just recognized special interfaces.
  `import_export.rs:72,119` fingerprints the model together with the entire
  imported artifact. Thus a vstd model entry binds the same artifact containing
  the checked word laws, even though the word laws are not themselves entries
  in the executable-model registry. Imports validate those fingerprints before
  consuming contracts, and carry them transitively. This is intentionally
  conservative invalidation, not fine-grained dependency minimization.
- Policy 66 rejects earlier inference-aware providers before accepting an empty
  ordinary-only proof list. Schema 38 still describes the same representation.
  Cache 69 stores candidate proposals; mandatory proof outcomes are fresh, so
  no candidate-formula bump is required by these eligibility/publication changes.
  Plain ordinary no-flag early exports intentionally remain outside this gate;
  their physical presence is not a proof receipt. Metadata is enforcement and
  provenance, not a standalone independently checkable proof certificate.

## Replay

No production code, application input, other checkout or parent runtime artifact
was changed. No delegation, push or PR was used. The matching compiler, macros
and fully checked vstd are built independently under
`/tmp/infer-combined-review-db26-20261109`; that directory is mode 0700.
Standalone Z3 is a regular copy at this checkout's ignored `source/z3`.
Cargo commands run from `source/`, with `/home/dev/.cargo/bin` first on PATH,
Rust 1.99.1, two jobs, and private target/build directories. No rebuilding parent
bundle is an input. Raw logs, generated probes and binaries remain private in
`/tmp`; the manifest commits compact hashes and outcomes only.

After building with `evidence/transfer-fedimint-full-request/build.py`, replay:

```sh
python3 -B evidence/combined-db26-review/probe.py \
  --verus /path/to/private/target-verus/release/verus \
  --solver source/z3 --output /path/to/new/private/probes
python3 -B source/tools/research/word_goal_automation/replay.py \
  --verus /path/to/private/target-verus/release/verus --output /path/to/private/replays
python3 -B evidence/transfer-fedimint-full-request/run.py \
  --application /path/to/clean/pinned/fedimint \
  --verus /path/to/private/target-verus/release/verus \
  --solver source/z3 --output /path/to/private/replays
```

This review does not reaccept earlier transfers, prove the inference machinery,
or cover future tuple changes. Explicit external declarations, imported specs,
compiler/backend and SMT solver remain trust inputs. Unsupported/frontend
controls are classified separately from ordinary semantic proof failures.
