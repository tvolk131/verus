# Trust and fidelity inventory

## Existing trusted boundary, not new assumptions

- Ordinary Verus lowering, proof obligations, SMT encoding and the solver remain
  the backend boundary. No application-specific identity or unchecked summary is
  added to inference.
- `vstd::std_specs::convert` already specifies primitive TryFrom success exactly
  by representability, with the successful cast and an unspecified error payload.
  The new synthesis handles only a contiguous conversion/unwrap observation; it
  does not invent or export an error value.
- `vstd::std_specs::result` already requires `Result::Ok` for unwrap and ensures
  the success payload. Soundness checks that requirement normally. Only
  successful-execution maximality may use that success requirement, at the
  closed canonical integer/TryFromIntError instantiation.
- The integrated `vstd::bytes` specifications and proved inverse lemmas are
  unchanged. They already trust native u64 byte conversion behavior and absence
  of unwinding. Rust core implementation bodies are not newly verified here.
- The entire vstd artifact and recognized declarations remain fingerprinted.
  Policy 56/cache 62 invalidate the previous semantics; schema 38 is unchanged.

## Checked, untrusted construction

Candidate formulas use existing casts, integer ranges, arrays, finite bit
operators and complete record values. Source-call signatures and canonical
dispatch are checked before composing them. Operand safety remains in the
candidate's domain. The application receives no assumed unwrap success.
Soundness, safe-domain maximality and output exactness all run on the unchanged
executable methods, including ordinary library preconditions and panic checks.
Publication and same-crate availability remain gated by all required checks.

The new record-to-array profile is bounded and straight-line. Its mutable locals
are scalar values, never reference aliases. Readonly subexpressions cannot mutate
their state. Branches, matches, returns, loops, logical short circuits and
variable shifts in this profile are rejected rather than unsoundly merged.
Existing scalar/owned shift checks are reused, not suppressed.

The packing bridge in the authored caller is ordinary ghost code: its finite
bitvector claims explicitly provide the word definition and the original three
domain hypotheses to the bitvector solver. The byte inverse follows from the
already proved vstd lemma. It adds no axiom, assumption, unchecked executable
contract or input restriction. Injectivity is proved through two actual encoded
arrays and the inferred actual decoder relation.

## Evidence safeguards and exclusions

The integrated snapshot's 22 hashes were verified before extending it. Existing
failed-baseline and parent integration evidence, original record tests and vstd
byte source are preserved. New replay hashes cover pinned source and generated
fixtures; immutable artifact guards cover engine sources, verifier, vstd,
solver, rustc/core/std and native compilation inputs throughout each command.

The public-interface fixture changes visibility only. It is not a claim that
redb publishes PageNumber as a public API. Source fidelity refers to the audited
extracted method bodies and frozen domain, not a proof of the surrounding crate.
Native checks are empirical and do not replace formal coverage. Reserved-bit
audits from historical evidence were not rerun or claimed by this codec harness.

The original unaided caller still lacks automatic bitwise bridge reasoning.
Its failed caller obligation is explicitly retained, not called a semantic
counterexample or silently omitted. The positive proof is the separately
authored, fully checked caller over the same domain and unchanged methods.

The faithful mutant must retain successfully proved inferred relations and fail
the authored semantic goal. A malformed goal, unsupported frontend source,
incomplete proof or resource exhaustion cannot satisfy that negative control.
Metadata failures are independently classified as import rejection. Stale export
controls use a real earlier verified interface for early frontend/import errors;
opaque sentinel bytes are used only after local inference is known.

Discovery/build setup failures and earlier replay-controller attempts remain in
the persistent scratch root. They are not counted as successful proof runs.
The final authoritative replay is the `bp47idhd` directory named in README.md.
