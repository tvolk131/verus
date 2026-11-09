# Canonical ordinary-proof milestone

`candidate-004.rs` proves the full frozen canonical predicate on the original
layout-free executable body: **9 verified, zero errors**, full-crate normal
`--no-cheating` verification, Rust lifetime/trait checking enabled. Runtime is a
private immutable byte-for-byte copy of accepted `0c00b6ec` (policy67/cache70/schema38).
Every u64 and arbitrary initialized exclusive `[u8;10]` remain in scope. No
runtime statement, loop/indexing, or canonical definition changed; no snapshot,
new assumption, external summary, layout policy or verification bypass was added.

This is a **manual diagnostic ordinary proof**, not completion of automation or
independent transfer. Existing prophetic `IterMut` final-referent observations
suffice for this property. Untouched unyielded suffix is not part of canonical.

## Authored semantic input inventory

- Inherited recursive `radix_power` and exact `canonical` predicate.
- `n == number / radix_power(i)` remaining quotient.
- `i > 0 ==> n > 0` before a continued iteration, supporting minimality.
- Already-written final referents contain the appropriate radix payload plus128.
- Checked exit clauses state zero remaining quotient, upper/minimal bound and
  complete prefix with the last continuation bit cleared.

## Explicit manual mechanical work

- Iterator alignment `i == it.index()`, bound `i <=9`, capacity equality10.
- Initial imported division-by-one proof call.
- Per-iteration calls to `quotient_step` and `byte_step`.
- Checked `radix_positive` induction; checked shift/division composition with
  existing vstd laws; power unfolding; radix10 evaluation; division order bounds.
- Checked byte cast/OR/mask bit-vector assertions.

These remain visible diagnostic helpers; none are described as semantic inputs.
Full ordinary checked bodies are exactly `u64`, `radix_positive`, `quotient_step`,
`byte_step`, with zero inferred helpers. 9 solver queries include termination and
bit-vector obligations; query count is not a count of semantic properties.

## Retained failures

-001: invalid trigger on dereference, frontend failure, gate rejected.
-002:5 failed obligations: opaque pow evaluation, missing division order laws and
initial quotient relation. This is a mixed diagnostic failure, not a semantic negative.
-003: misspelled existing lemma name, Rust frontend failure. The first lightweight
trial gate accepted non-VIR frontend failure as generic failure; this is explicitly
excluded as proof evidence. Strengthened local gates/replays will bind named bodies
and classify frontend/semantic/auxiliary failures separately.
-004:9/0, all four named bodies checked. Successful inputs were hashed before trial.
-005: a separate reduction experiment; no success is claimed in this milestone.

`ordinary-milestone.json` records copy hashes, commands and raw receipts. All failed
candidates/freezes remain immutable. `trial.py` snapshots/hashes each input before
its compiler command and uses shared command receipts and ArtifactGuard. Replay:

```sh
python3 -B -O source/tools/research/varint_canonical_goal/trial.py \
  source/tools/research/varint_canonical_goal/candidate-004.rs \
  --bundle /private/immutable/accepted-bundle --output /private/trials --expect success
```

Trust: ordinary Verus/rustc/macros/SMT and existing vstd contracts including prophetic
mutable iterators, slice range and array unsizing. No broader application proof,
suffix/pointer theorem or complete goal achievement is claimed.
