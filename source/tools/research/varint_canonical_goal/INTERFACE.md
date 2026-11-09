# Proof-assistance interface, initial milestone

The exact frozen layout-free canonical predicate and u64/arbitrary initialized
exclusive ten-byte buffer domain remain unchanged. No executable statement,
loop/index form, or runtime check is changed. candidate-001.rs is an untested
manual diagnostic candidate, frozen before any proof trial.

Semantic inputs: recursive radix and exact canonical predicate inherited verbatim;
remaining-quotient invariant; nonzero quotient after a continued iteration (minimality);
final-referent written-prefix payload/continuation invariant. The loop ensures clauses
spell out the corresponding exit obligations; they are checked, not assumptions.

Mechanical work: iterator-index equality and capacity/bounds invariants; positive
radix induction; shift/division composition; byte casts/masks; generic division laws;
quantifier instantiation and final-referent observation/frame transport. Local calls
in this diagnostic are manual proof work and disqualify it as the accepted automated
fixture. No ghost snapshot initialization has been added.

Initial proposed observation interface (existing model, not a request for an axiom):
`Seq<&mut T>` final referents must retain each already written prefix value while
another element is borrowed, and after early break each final referent must connect
to the corresponding final array element. Existing `it.seq()` plus `final` may already
suffice. A minimal repro and specific failed obligation will follow actual trials.

Ordinary proofs explicitly expect zero inferred helpers. All named executable and
proof bodies must pass full --no-cheating verification with lifetime/trait checks.
No bounded engine recognizer is implemented or claimed before the proof is understood.
