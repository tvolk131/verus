# Independent Fedimint LNURL accounting107 review

November 7, 2026, America/Chicago. Review started 2026-11-08T02:56:43Z.

## Bounded verdict

No new arithmetic, guard, provider-identity, certificate-import, or mutant-localization
defect was found in the checked numeric slice. Seven fresh direct verifier
applications produced the expected six successful results and one intended
backend-only rejection. This is not acceptance/adoption or a native application
or caller theorem. A provenance objection below remains OPEN.

## Finding: incomplete transitive-loader authentication

P2, evidence completeness: candidate `run.py` authenticates the runtime files,
compiler/sysroot libraries, source, and helpers, but omits the executable
`/Users/tommyvolk/.cargo/bin/rustup`. Its recorded PATH selects that executable;
the source-bound Verus launcher invokes `rustup toolchain list` and `rustup run`
by default (`source/verus/src/main.rs:129` and
`source/verus/src/main.rs:192`). This is separate from the twelve already
disclosed source-map differences and from compiler/source correspondence.

The fresh review reused precisely the recorded runtime/environment and initially
authenticated the same declared immutable bindings. Consequently its seven
applications also lack entry/preexecution authentication of this transitive
loader. The gap was found after the seventh application; the budget was not
expanded and no retrospective authentication is asserted. The late observed
SHA256 is `f7c7e11eb2e07f006e532a8aaac77b32f4527d305ac386c0c633ca6133d35ca9`.
It is checked again at exit. The existing nine runtime hashes and pinned
compiler/sysroot hashes all match. This gap does not demonstrate a wrong
arithmetic result, but prevents claiming complete actual-tool coverage.

## Fresh applications

All use one worker, `--num-threads 1`, `--no-cheating`, sanitized recorded
environments, independent scratch cache, and a hard 60-second application limit.
Only provider applications compile their own declared small rlib/interface.

1. Native fee provider: 55 verification conditions, zero errors.
2. Distinct expiration provider: 11 verification conditions, zero errors.
3. Imported corrected composition: 10 verification conditions, zero errors.
4. Explicit full-domain sum including composition: 11 verification conditions,
   zero errors.
5. Wrong-net-value mutant: 8 verified, exactly two accounting postcondition
   failures at source lines 80 and 96; no VIR/frontend/overflow/resource failure.
   No `--export` is requested and no interface is produced.
6. Separate policy-accepted 100,000-msat witness: 10 verified, explicitly reports
   99,000 from contract 49,500 plus decoded wrong fee 49,500; actual fee 50,500.
7. Restoration: byte-identical to corrected composition; 10 verified, zero errors.

All seven finished without timeout. The ledger is seven conservatively charged
direct applications, not seven plus uncounted provider compilation. No eighth
application, compiler/vstd/runtime build, Cargo invocation, native application
execution, network access, Git write, subdelegation, or excluded-lane work occurs.
Counts are verification conditions, not additional inferred native bodies.

## Independent checks

- Authenticated all 120 manifest payloads and supplied manifest/report/seal hashes.
- Reauthenticated 1,273 declared immutable input bindings and nine runtime files
  at entry, before/after every application, and exit. The loader gap above is
  explicitly outside that declared set and is not concealed by these counts.
- Byte-compared all 122 original archive file members using the correct
  `fedimint-lnurl/` prefix. Original archive SHA256:
  `1525b3cc5b57345d4f868ae3c8ba388d475e6c705c4e0f99419553d9d4d7fdab`.
  The parent's initial omitted-prefix metadata error is not a candidate error.
- Read all eleven historical attempts, stdout/stderr, scopes, commands,
  authentication records, controls, preparation failures, audit and patch.
  Historical provider counts 55/11, imported composition 10, combined local 76,
  and explicit sum 11 match. The executable call inside assert is retained as a
  frontend error followed by correction; three historical semantic negatives
  reject with 2/1/1 backend postconditions respectively.
- Replayed original auditor with only its two output destinations redirected
  into this scratch. Its audit JSON and implementation patch exactly match the
  frozen originals; the candidate is never written.
- Regenerated both exact provider sources from authenticated recovered native
  content and checked all original native item hashes and required scope fields.
- Independently parsed fresh provider certificates: distinct real crate IDs,
  schema 38, policy candidate-bound-evidence-v76, required soundness/maximality/
  exactness proofs proved, cold caches, positive solver timings. Every fresh
  composition imports the exact two regenerated certificate fingerprints and
  corresponding freshly compiled rlibs; trust/model inventories are unchanged.
- Mutation changes exactly two guarded probe/witness encoder arguments; the
  unrestricted core and independent receiver remain unchanged. The separate
  witness weakens only its general probe goal and proves the exact 99,000 result.
  Thus this is localized accounting sensitivity, not an end-to-end transport test.

## Mathematical and native boundaries

The fee law is `cap(cap(amount * ppm) / 1_000_000 + base)`: multiplication
saturates before division, followed by saturating addition. It is not an
unbounded-product percentage formula. Division is by a fixed nonzero constant;
the inherited arithmetic/constructor premises and vstd correspondences remain.
For every u64 amount/base/ppm, debit `max(amount-fee, 0)` plus recovered fee
equals `max(amount, fee)`, a representable u64. Original-amount reconstruction
holds exactly iff `fee <= amount`; no such caller premise is added.

The authored checked-add observer returns Some exactly iff independent
`contract <= expiration`; it is not native unchecked `Amount::Add` verification.
Overflow boundary examples and saturation witnesses are included in the positive
crate. The native numeric amount/component guards suffice for exact invoice
reconstruction and contract >=49,500 msat. Some is only numeric guard acceptance,
not successful invoice creation, payment, discovery, receiving, or event emission.
Producer-to-receiver equality and taking the non-manual fallback remain conditional.
Historical amount=0/base=1/ppm=0 conservation and max-contract/zero-expiration
totality controls were audited, not rerun under the exhausted seven-call budget.

## Still open

Recovered native remains SOURCE-CONTENT-ONLY, with failed original-path
preservation. All twelve current-parent/frozen-source-map differences remain
recorded, including missing entries. Compiler/source correspondence is NOT
repaired; runtime independent_review remains OPEN. No whole-Fedimint, clean
native continuity, funds/crash/crypto/liveness, native caller, acceptance, or
adoption claim is made. All eighteen redb cells remain OPEN.

Recovered-native scoped instructions were read, as were available codebase,
development, and Rust-style skills. The referenced linked-specs and multipart-review
skills are absent from the recovered tree and known local skill roots. No source
change or subdelegation was used to compensate; this procedural limit is disclosed.

Raw evidence, exact commands/environments, per-application status/scopes, declared
input authentication and the actual seven-call ledger accompany this report.
The seal records final elapsed/storage figures and the exit comparison. No
automatic continuation or research chain is requested.
