# Typed early milestone (not an accepted runtime bridge)

Base: bc894bebb619260b6723105875294837c45e845f. Pinned Rust source commit:
58cf8f9c35b355199727627d07ae34a4e2846d67. BorTag's declaration is extracted
byte-for-byte from the pinned borrow_tracker/mod.rs; all seven derives and the
private NonZero<u64> field remain. No constructor is needed: comparisons borrow
two arbitrary genuine tags. There is no 61-bit precondition.

Target: actual compiler-generated PartialEq::eq returns exactly
`a.0@ == b.0@` for every genuine tag pair. The existing standard NonZero view and
PartialEqSpecImpl suffice for direct NonZero equality (1/0). The existing inverse
view axiom proves ghost wrapper injectivity (2/0 including original Clone), but
that proof alone says nothing about an ignored runtime comparison body.

A private typed observation and a closed PartialEqSpecImpl candidate avoid
changing field visibility. The __verus_infer_derived_eq marker selects the real
compiler-generated eq body, then the baseline rejects the request at its explicit
single-scalar-field policy gate. This is an inference-profile barrier, not a
semantic counterexample or an accepted derived method. No helper obligations
have passed at this point. Proposed next interface: a bounded candidate-only
NonZero<u64> view projection, with original-body soundness, domain maximality and
output exactness required normally, including import/export validation.

Ord is a separate milestone. The baseline's standard nonzero module provides no
OrdSpecImpl (its comment cites the Destruct bound). Merely delegating a candidate
cmp_spec to that absent model does not establish a standard-view ordering law.
PartialOrd with an actual Ord derive delegates to Ord, so it cannot be claimed
from PartialOrdSpecImpl alone. No vstd change is authorized here.

All commands keep --no-cheating and full checking. Launcher/PATH and annotation
WF failures are retained separately from ordinary proof failures. No production
source change has been made at this milestone. All raw generated sources, logs,
VIR/solver output and receipts are private /tmp artifacts. trial.py authenticates
canonical success and all eight bundle hashes; historical support keys rebind to
identical files in this checkout. Python gates survive -O.

Global scope: native=0; 44 minimal + 3 phase / 101 writer effects OPEN; exact
canonical application and full HMAC remain UNPROVED; publication=false;
goal_complete=false. Parent owns independent acceptance and version reconciliation.
