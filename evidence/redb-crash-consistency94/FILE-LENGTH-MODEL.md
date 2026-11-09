# Candidate length/error persistence model

This is a checked mathematical dependency, NOT accepted native-backend
correspondence or a new storage axiom. All eighteen final cells remain open.
The environment decisions identified in
`/private/tmp/campaign94-hash-io-design-20261105-9hT6Fr/REPORT.md` remain explicit.

`FileImage` separates visible and durable length, pending length choices, and
the existing byte-image pending-value sets. Its byte capacity is symbolic ghost
padding, not a fixed application-size or finite-test bound. Only bytes below
the selected file length are readable file data. The native abstraction and
finite-history capacity-witness construction have not yet been proved.

A successful issued resize changes visible length. Growth zero-fills the newly
visible range. It does NOT make the new length or zero bytes durable. Pending
length choices include EVERY intermediate length between the old visible and
requested length, plus earlier pending choices: no atomic old/new-length
assumption is introduced. This is a proposed environmental relation, not a
claim that Rust's trait establishes its crash or failed-resize semantics.
Failed `set_len`, position-write EOF extension and native stale-write
cancellation are still unproved dependencies.

The in-bounds write subrelation requires the actual target range to be below
visible EOF. Native callers must prove that premise; it is not a new exclusion
from the frozen campaign. Interrupted writes allow arbitrary subsets of the
requested bytes to become visible, including every prefix. A failed sync may
persist any previously permitted bytes/length, without clearing pending
choices or acknowledging completion. Both no-effect and fully-applied error
outcomes are admitted. Process restart leaves this backend state intact;
power restart discards volatile state, not a substitute for successful sync.

Successful sync publishes visible bytes AND length under its explicit model
relation. Proving that the chosen real platform's barrier satisfies this
relation, especially required file-length metadata, remains a separate reviewed
environment boundary. No new trusted I/O binding has been added.

Live byte protection requires both compatible pending byte values AND that
every pending length includes the selected byte. Checked lemmas preserve this
through compatible writes, safe shrink/growth, partial writes, persistence,
sync, failures and restarts, with induction over arbitrary finite model traces.
The resize safety threshold and compatible-write conditions remain caller
premises; native allocation, reclamation, buffers and recoverable-root
footprints have not been shown to establish them.

This model deliberately admits non-atomic intermediate crash lengths and
errored-but-durable effects. Neither an Err-is-rollback theorem nor corruption
reporting forever is credited as recovery availability. Model controls cannot
substitute for native crash atomicity, native error correctness or recovery.
