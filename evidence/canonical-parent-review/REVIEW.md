# Independent canonical integration review

Review base: `395c82994539a311c5e76d5a1a8eb5e91ca712ab`, including parent
`112a33f036c0c54a9277e30d61d9448e2f5bd232`. No production changes, delegation,
push, PR, global settings or version changes were made by this review.

## Scope and decision

**Recommend acceptance of this scoped canonical integration.** No blocking
correctness/trust finding remains after independent static review and matching
validation: fresh compiler/macros, verified vstd 2226/0, all 143 VIR tests,
all eight focused compiler tests, the optimized-Python 26-case replay/audit,
seven additional multi-function/policy probes and five metadata-binding probes
pass. All eight optimized-Python harness tests also pass.
This review covers only full-domain canonical u64/radix128/ten-byte assistance
and its parent integration. It cannot accept the entire mutable-buffer goal.
The original HMAC pad-XOR prefix **and untouched suffix** property and actual
shared proof-producing transfer remain unproved; observation/native-certificate
work is outside this checkout and this recommendation.

## Correctness and trust assessment

No blocking correctness defect found in the reviewed implementation.

- `source/vir/src/infer/buffer_goals.rs:61`: arithmetic catalog lookup checks
  the resolved vstd identity, ordinary function attributes, static item kind,
  parameter types and Spec modes, return type/function mode, no proxy,
  no generics/bounds, and nonempty lemma ensures. The local radix function
  must be a nonprophetic, nonexternal ordinary monomorphic Spec definition.
  The iterator-index/sequence lookup at lines 537 and 572 is less general:
  it recognizes the existing vstd iterator wrapper methods by path and resolved
  types, rather than using the arithmetic catalog signature checker. It does
  not justify supporting arbitrary catalogs or iterator implementations.
- `buffer_goals.rs:185`: eligibility restricts the scalar width, step, capacity,
  native slice proxy/unsizing/borrow origin, scalar/counter initialization and
  four writes. The 8192-node traversal budget bounds source recognition.
  Recognition proposes proof work; it is not a proof of control-flow semantics.
  Every generated invariant, precondition, BV query, positivity assertion and
  bounded radix computation must subsequently verify on the retained body.
- `buffer_goals.rs:628`: independent decoration validation removes precisely
  the root and loop-body prefix statements and appended invariants, checks their
  counts and placement, and compares the whole retained function data. Original
  contracts, resolution wrappers, invariants, isolation and termination data
  are retained. Expression-ID collision checking at line 779 rejects overlap
  with existing body expressions or another new expression. Freshness is scoped
  to each function; repeated generated IDs in different functions are safe
  because mode/resolution maps and insertion traversals are per function.
- `buffer_goals.rs:856` and `source/rust_verify/src/query.rs:629`: successful
  decoration registers the function for a full ordinary well-formedness check,
  covering the body rather than only generated contracts. At `query.rs:750`,
  after trait merge/demotion/default-body inheritance, full mode checking
  replaces the stale resolution map and installs the checked function. At
  `query.rs:783`, the function participates in assisted-spec constructor-map
  completion before ordinary resolution inference.
- `source/vir/src/infer/assistance.rs:34`: step reads are Spec reads; direct
  variable terms concern copyable u64/usize scalars or a ghost iterator wrapper.
  The added lemma calls reside in `ExprX::Ghost` blocks. Original prophetic
  final-referent prefix clauses remain untouched and pass the refreshed normal
  mode analysis. No assumed current/future equality is installed. Generated VIR
  proof statements have no source HIR executable counterpart; initial source
  erasure modes remain the modes used for source erasure. The original HIR still
  goes through Rust borrow checking (`verifier.rs:3575`).
- `source/rust_verify/src/verifier.rs:2931,2991,3597`: full-crate/lifetime/trait
  checks and successful completion of all ordinary/auxiliary queries gate
  inference-aware publication. `source/vir/src/infer/registry.rs:8` changes cache
  identity to v71; it stores candidate formulas, never successful proofs.
  `source/vir/src/infer_proof.rs:48` and its policy aggregate bind mandatory
  obligations to registered candidates and reject missing/duplicate/stale
  evidence. A warm cache does not waive these checks.
- `source/rust_verify/src/import_export.rs:154,163,197`: imports bind schema38,
  policy68, producer identity, exact required proof inventory, full exported
  summary fingerprint, dependency fingerprints and available trusted-model
  artifacts. These are consistency checks on trusted compiler artifacts, not
  cryptographic proof validation of a maliciously rewritten `.vir` file.
  Existing models/library contracts remain an explicit trust boundary.

## Historical evidence and fidelity

Candidate008 retains exactly the original predicate, recursive radix, signature,
input domain and executable macro projection. It has three authored semantic
invariants and four checked exit clauses, no local mechanical proof blocks,
theorem calls or snapshots. Candidate004 is separately inventoried manual work.
There is no zero-buffer precondition, numeric restriction, layout assumption,
new vstd law, mutable-call allowlist or inferred encoder summary. `git diff
cac21695..395c8299 -- source/vstd` is empty.

All nine earlier charter/local candidate freezes match. The standalone
`acceptance-freeze.json` still binds its historical fixture and evidence:
only `copy_bundle.py` and `replay.py` differ, from the intentional source-inventory
extension in `395c8299`. The historical manifest remains policy67/cache70 and
must not be cited as matching policy68 validation. This review records separate
matching-runtime tool/source hashes; it preserves all historical evidence.

## Validation and evidence

Final commands, outcomes, source/tool hashes and raw receipt locations appear
in `manifest.json`. Large logs, intermediate sources and binaries remain under
`/tmp/canonical-parent-review-395c8299/`. The retained setup failure is excluded
from semantic evidence. All 1,724 tracked source/dependency inputs in the private
before/after inventory are unchanged, as are all guarded tools and trial inputs.

The matching 26-case replay and audit pass, with exact native body and helper
inventories (including zero helpers) and every mandatory identity-helper proof
bound. Canonical cold/warm are 6/0, manual diagnostic 9/0, provider cold/warm
9/0, and the certified importing caller 1/0. Healthy-helper semantic negatives
reject only their authored postcondition; seeded failed export is removed.
Unsupported shapes/disabled assistance and the wrong-range diagnostic remain
separately classified. Typed cached-helper poison is consumed on a cache hit,
fails its required proofs and suppresses export; metadata policy poison rejects.

Seven independent additional probes pass: two decorated functions with no
helpers (10/0); the same with one fully checked exact identity helper (13/0);
a false caller of the second function (13/1, only postcondition failure); and
four intentional policy rejections for disabled lifetime checks, disabled trait
checks, disabled verification and filtered scope. Both decorated functions are
explicitly required in the assistance and native-body inventories. Five further
intentional metadata controls reject producer, trusted-model, dependency,
required-proof-inventory and exported-summary mismatches. Their compiler
rejections are not counted as semantic evidence.

The first additional-probe run used `--verify-function` without the required
`--verify-root`/module selector. Its CLI/setup rejection is retained and excluded.
The corrected controller was rerun in new private directories; the final run also
enforces explicit empty helper coverage on all four policy rejections. No panic,
resource failure or frontend failure is counted as a semantic negative.

Remaining trust includes Rust1.99.1, the compiler and macros, Z3 4.16.0, the
ordinary verifier/SMT encoding, existing prophetic iterator/slice/array/range
contracts, and inherited verified-vstd admissions. No claim is made for native
untouched-suffix framing, pointer identity, HMAC security, arbitrary widths,
capacities or shifts, or general mutable-effect inference.
