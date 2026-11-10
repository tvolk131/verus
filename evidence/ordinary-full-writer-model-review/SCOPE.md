# Independent review scope (before fresh trial)

Review the exact completed MODEL implementation d7156dec, with the typed
interface fixed at d50fe08e, integrated in the prepared checkout. Own only this
new evidence directory. Do the review directly; no delegates, push or PR.

Authenticate worker receipt /tmp/model-ordinary-full-writer-trial-012/receipt.json
SHA-256 5733f94896797c6b3b7b432500ae0839711843e979e8c05f683516d349909ed8.
Freeze the review replay before running it. Use normal coqc in a fresh private
trial, inventory each of the 25 source declarations and 100 generated helpers
(including assumptions, constructors, obligations and existing instances), and
full recursive coqchk without exclusions. Compare all six fresh .v/.vo pairs
with the authenticated worker; retain input/output guards through success.

Inspect raw initialized premises, actual Na1 runtime, same heap map/Outer,
original parent/child recovery and original R/G closure, exact pad/key/outside
post, all finite lengths and internal sufficient budget. In particular review
the bookkeeping-only observer adapter and resource framing. Public fresh-world
adequacy does not establish entry from an arbitrary Rust caller; the generic
caller MODEL library is a different result.

Only this checkout/history and exact named completed artifacts are inputs.
Receipt checkout paths are historical labels, never live source directories.
Baseline 115a618d is read-only: readiness AND durable successful receipt,
523 sources/284 objects, pinned tools and GMP manifest
332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37.
Use python3 -B; no installs, shared builds/edits, broad metadata searches,
race/substitution tests, root model_full_writer outputs, Rust or HMAC builds.
Raw logs and objects remain private; commit compact evidence and exact replay.

MODEL only: native0, 44+3 minimal/phase and 101 writer effects open. HMAC1/1
inherited; canonical6/0 inherited; publication=false; overall goal incomplete.
No native/provenance/retag/source-simulation/C/F, unrestricted SC/Na concurrency
equivalence, termination or publication claim.
