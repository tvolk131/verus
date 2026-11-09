# Lifecycle113 revision2: parent repair of source-review findings

## Status

Partial, unbuilt, unintegrated source candidate. No native correspondence or
final redb cell is proved. The independent review of revision1 remains
**CHANGES REQUIRED** and its exact17-file candidate is preserved unchanged.
These are parent-authored small repairs, not a new independent approval.

## Finding dispositions

- **P1: remove eager glue queries before refusal.** The new raw capture helper
  no longer accepts TyCtxt or explicitly resolves drop glue, ADT destructors,
  type projections or instantiated bodies. It records the original root type,
  full place/projection diagnostics and explicit NOT_QUERIED identity statuses.
  Actual glue, shim kind and destructor execution are unestablished. Inherited
  drop_impl remains unchanged and is not invoked by this raw refusal path.
  This removes the specific new codegen-query dependency; it is not a theorem
  of total compiler/formatting/allocation behavior or a native Drop proof.
- **P2 identity and labels:** no partial resolved-instance pair is advertised.
  Glue/shim identity is explicitly not queried. Drop.drop is labelled
  async_cancellation_cleanup_target, the unwind cleanup target is separate,
  and replace is labelled replace_assignment_origin, not an execution branch.
- **P2 missing origins:** both origin-map Options remain available to the raw
  scanner. Status distinguishes missing, incomplete, completed-empty,
  completed-without-local-match and completed-with-matches. Builder completion
  explicitly seals an empty temporary capture; it no longer looks missing.
  An absent binder map does not establish an anonymous owner. All unestablished
  sites still produce Err before normalize or replacement installation.

The actual Scope and TempLifetime inputs are now retained as typed observations.
They do not prove destruction-lifetime identity, local stability through MIR
passes, occurrence-specific ownership, AstId correspondence or resource effects.
Multiple observations at one local remain multiple observations, not a unique
provenance join. Phase reads/resets are still separate observations, not an
atomic/session-epoch proof. Named-root and inherited strict subset rules are
not broadened; extra refusals on cleanup/ghost/unreachable sites remain possible.

## Actual validation and unexecuted controls

Three direct rustfmt syntax parses return0, without installing formatted output.
Nine inherited function bodies remain byte-identical to the historical source.
The source validator checks status/refusal shape and absence of explicit query
calls in the new raw helper. These are textual/formatter checks, not compiler
typechecks, Rust AST equivalence, tests of runtime behavior or native proof.

Five Rust tests are authored for the pure status-classification helper. **None
was executed**: test compilation/typechecking remains outside current authority.
No backend semantic control was run. Compiler builds/typechecks0, native bodies
checked0, verifier/solver applications0 in this parent source-repair task.
Round113's only five verifier applications are lane A's retained frontend/setup
failures, never positives or semantic negatives. No new Infer behavior is added.

One source-audit application failed because the string tcx also matched a Rust
lifetime. VALIDATION-FAILURES.md preserves that documentary failure and the
shell-wrapper status qualification; validation-02.stdout/stderr retain the
corrected direct application. No failed source audit is hidden or counted as
a native control. VALIDATION.json records the successful syntax/source checks.

All1483 parent baseline records still reconcile before final document updates:
1481 exact matches and exactly two previously recorded GOAL/STATUS changes.
No parent/native source, old candidate/review, frozen runtime or sealed evidence
was edited. Only three source files were copied. All historical native-path
FAILED, Git UNAVAILABLE, compiler/source UNESTABLISHED and conditional/unadopted
TCB qualifications persist. All18 cells remain OPEN.

## Remaining prerequisite

A separately authorized isolated pinned-source compiler typecheck/build is
needed to execute/validate this new hook and its regressions. It would validate
capture/refusal plumbing only, not establish the missing native Arc/Mutex/LRU
ownership, uniqueness, poison, reservation and destructor effects. Those remain
explicit proof/TCB boundaries; no new trusted laws, native producer, build,
source integration or acceptance-gate change has been authorized here.
No automatic additional review/probe/build chain is started.
