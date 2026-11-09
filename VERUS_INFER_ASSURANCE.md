# Verus Infer assurance

This is the current assurance map for the experimental implementation. It does
not claim formal verification of the inference machinery or the Rust standard
library. The [feature reference](source/docs/guide/src/infer.md) defines supported
behavior; the [direction document](VERUS_INFER_DIRECTION.md) sets priorities.

## Acceptance and executable preservation

- Candidate generation is untrusted. Ordinary verification checks soundness
  against the original executable. Exact mode additionally requires safe-domain
  maximality and output exactness within independently selected observations.
- Installation preserves executable bodies and signatures. Supported loop and
  callback paths add checked proof annotations; profile-specific projection
  checks reject changed executable content and invalid decorations.
- Precision queries are separately classified and cannot replace ordinary
  safety checks. Maximality uses the original verifier's successful-execution
  conditions; it must not assume the candidate precondition it is checking.
- The production acceptance policy binds outcomes to the candidate and
  obligation. Missing, failed, incomplete, skipped, duplicate or mismatched
  required evidence cannot authorize acceptance. A later success cannot repair
  an earlier failed subquery.
- Candidate caches store formulas, not proof results. Eligibility is checked
  again and every required proof is fresh, including on cache hits.
- Inferred dependents wait for required checks. Full selected-crate verification
  is required for publication. Failure invalidates inference-aware stale
  interfaces; older unmarked artifacts have the detection limits described in
  the feature reference.
- Imports validate guarantees and contract, dependency and model fingerprints.
  Imported bodies are not reverified. Metadata is enforcement and provenance,
  not a standalone independently checkable proof certificate.

The small acceptance component is
[source/vir/src/infer_proof/policy.rs](source/vir/src/infer_proof/policy.rs), with
its compiler adapter in [infer_proof.rs](source/vir/src/infer_proof.rs).
Precision transformations are in
[infer_checks.rs](source/vir/src/infer_checks.rs) and its state submodule.

## Proof assistance

Assistance prepares proposals; ordinary original-body verification remains the
acceptance boundary. A generated contract, invariant, trigger, callback equation
or `applied` reporting row is not itself a verified outcome.

Demand-driven scalar assistance checks its conditional theorem through the
existing bit-vector backend before using the identical fact in the ordinary
query. Unsupported inputs and exhausted generation budgets install no guessed
facts. A false auxiliary theorem fails verification. Generation budgets and
solver limits are different failure categories.

Keep synthesis and independent observations/checks separate when consolidating
shared traversal, substitution and annotation utilities. Preserve executable
safety, authored goals, termination and publication controls in every extension.

## Trusted interfaces and model inventory

Rust type/borrow checking, the Verus frontend and lowering, SMT encoding, solver
results and imported library specifications remain trusted boundaries. Verifying
vstd does not prove the native implementations of its assumed specifications.

- Primitive arithmetic, casts, checked/saturating methods, array helpers and
  collection/iterator APIs rely on their stated vstd models. Historical audits
  account for added primitive specifications and `no_unwind` declarations.
- Native u64 byte conversion uses explicit assumed primitive specifications in
  [bytes.rs](source/vstd/bytes.rs). The transfer
  [byte-model inventory](evidence/transfer-redb-byte-apis/TRUST.md) states their
  assumptions; native Rust core implementation bodies are not proved here.
- Native Option bridge bodies in
  [native_option.rs](source/vstd/native_option.rs) are checked. The frontend's
  correspondence from canonical core APIs to those bodies is still trusted
  implementation code. The [Option inventory](evidence/native-option-unification/TRUST.md)
  states supported callback/effect limits.
- Native byte spare-capacity assistance uses the checked
  [byte buffer API](source/vstd/byte_vec.rs) and existing tracked pointer
  machinery. The [hex acceptance](evidence/verus-infer-hex-v3/README.md) records
  the narrow u8/Global profile and excluded unsafe operations.
- Application views, semantic invariants and dependency laws remain explicit
  inputs. Cryptography, codec and locking assumptions must be stated for each
  application slice; inference does not invent stronger laws for them.

A new or strengthened trusted library model needs an explicit trust inventory,
matching dependency fingerprints and an account of native semantics. A checked
first-party implementation must not be replaced by an assumed summary merely
to remove a proof blocker.

## Regression and replay controls

Run the Python harness regressions from the repository root:

```sh
python3 -B -m unittest discover -s source/tools -p 'test_infer*.py'
python3 -B -m unittest discover -s source/tools -p 'test_redb*.py'
python3 -B -m unittest discover -s source/tools -p 'test_fedimint*.py'
```

The production acceptance-policy and full source-mutation controls are:

```sh
python3 -B source/tools/test_infer_assurance_mutations.py --policy-only
python3 -B source/tools/test_infer_assurance_mutations.py
```

Use the pinned Rust toolchain. Full mutations also need cached Cargo
dependencies and matching Z3. Baselines must pass; each mutant must compile and
produce a completed nonempty failing test result. A crash, compilation error,
empty test filter or timeout does not count as mutation detection.

The Rust `infer*.rs` integration suites exercise supported profiles, independent
precision checks, source preservation, imports, caches, callbacks and reporting.
[Evaluation instructions](source/tools/infer_eval/README.md) describe cold/warm
and incremental checks. [Transfer replay](source/tools/INFER_TRANSFERS.md) checks
application pins, preserved source, fresh proof reports and publication.

Keep semantic postcondition failures, executable safety failures, failed
auxiliary theorems, unsupported syntax, import rejection and resource exhaustion
separate. Controller completion and native tests are not application proof.
Preserve failure receipts, semantic mutations, seeded stale-interface controls,
and a final restored-source replay.

## Later: formal verification targets

1. **Production acceptance component.** Prove that acceptance/publication needs
   every required successful outcome for the same candidate and dependency set.
   State assumptions about query outcomes, fingerprints, serialization and I/O.
2. **Precision transformations.** Give semantics to the supported IR and prove
   that maximality/exactness queries establish their claims, including calls,
   branch reachability, loops and independent complete-output observations.
3. **Independent replay.** Investigate explicit original-body/contract/precision
   obligations checked through another verifier path, with justified translation
   and build correspondence.

These are separately scoped assurance projects. Formalizing every precision
transformation is not a prerequisite for each bounded automation improvement.
None of these targets proves the entire compiler, standard library or solver.

## Historical audits and milestone evidence

Detailed implementation/model audits and checkpoint-by-checkpoint test history
are preserved in [the immutable assurance journal](https://github.com/tvolk131/verus/blob/3b05c1912b9d7314ad00c832c4065f4362562e02/VERUS_INFER_ASSURANCE.md).
Its sections retain the assumptions and version identities of their own
milestones, not the current build. Read it selectively when changing a relevant
model or proof boundary. An offline copy is available with:

```sh
git show 3b05c1912b9d7314ad00c832c4065f4362562e02:VERUS_INFER_ASSURANCE.md
```

Frozen acceptance documents and evidence directories are unchanged. Current
application scope is linked from the [project entry point](VERUS_INFER.md).
