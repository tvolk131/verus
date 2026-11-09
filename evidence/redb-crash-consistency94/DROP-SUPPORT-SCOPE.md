# Authorized native destruction-support side quest

November 5, 2026. User authorization: "approved - side-quest it up, then get back
to the goal whenever that's done." No further application design input is needed
at this checkpoint. The original all-six, both-commit-mode and mixed-history redb
goal remains unchanged. No final property/history cell closes on this scope alone.

## Concrete purpose

Support a checked source-exact RawLeafBuilder producer route, including real
destruction and cleanup, without assuming completion or pretending Drop has no
effects. Reuse existing typed MIR, reference/place reasoning and proof machinery.
The independently assessed design in NATIVE-CLOSURE-DECISION.md is the starting
point, not an already implemented soundness theorem.

## First feasibility checkpoint

1. Identify actual typed destruction events, explicit core::mem::drop calls,
   cleanup edges, moves/drop flags and the source/IR correspondence needed for
   caller proof obligations. Determine which rustc query stage is authoritative;
   pre-elaboration syntactic Drop instructions are not complete generic glue.
2. Implement a diagnostic-only extractor in isolated compiler staging. It must
   not relax today's Drop rules, change proof results, export new trusted facts,
   or label an extracted graph as a checked destruction theorem.
3. Challenge the mapping with scope exit, early return, overwrite, temporary,
   moved/returned owners, conditional initialization, explicit drop, nested and
   generic owners, cleanup and non-drop escape. Unsupported or opaque mappings
   are explicit blockers, never omitted successful cases.
4. Select the narrowest concrete-type/native call-graph route that can connect
   every relevant event to a checked antecedent and effect. Independently review
   that route before it can change accepted proof behavior.

An extractor alone does not finish the side quest. The next implementation
checkpoint must discharge actual destruction-site obligations, prove the real
destructor under checked normal/unwind conditions, preserve original borrow
identity and page frames, and reject unhandled reachable glue/escapes. A new
assume_specification, application axiom, renamed/omitted Drop, forced no_std,
unconditional panicking value or blanket precondition relaxation is not a fix.

## Completion and return to the application goal

The side quest completes only after independently reviewed checked support is
usable in a source-exact native producer/Drop proof and fresh ordinary C0 readback
composition, with discriminating incomplete-builder/cleanup/borrow controls and
regressions. Then resume native persistence/KV composition. Pure diagnostics,
body-only sublemmas and passing finite tests remain supporting results only.

Work continues autonomously through routine compiler errors, proof failures,
localized repairs and measured checkpoint reallocations. Reassess with the user
only for a substantially broader architecture, new trust/format/API assumptions,
independently assessed unproductive direction or genuine external resource or
permission boundaries. Stage budgets are prospective checkpoints, not a global
automatic stop. No general arbitrary-Rust exception verifier is authorized by
default; fail closed when the bounded route cannot cover a reachable case.

## Isolation and evidence

Use a build-source copy, not a new Git checkout or branch, seeded from the reviewed
1091-path mask-repair source. The two-line mask fix is inherited and separately
identified; it does not supply destruction semantics. Preserve live compiler,
native pin, old seals/runtimes/toolchains, unrelated dirty work and owned Git
identity. No Git writes or networking. Bind new compiler inputs explicitly,
audit compiled local dep-info coverage, separate builds/vstd/application/test-only
processes and retain failures. Historical runtime005's nonexhaustive1084-map and
Git object-store noncertification qualifications remain unchanged.
