# Compose typed input construction with actual final-memory observation

The accepted typed-input construction and final-state observer currently have
different public interfaces: the former exposes only a pure adequacy result,
while the latter needs a WP that returns physical resources. This lane adds a
resource-valued WP setup interface in one fixed outer heap instance.

For arbitrary initialized finite model heap h, subobject p and ranged byte list
zs, derive the same existing model reference, ordinary lifetime/type/prophecy
contexts and exact outside frame from initial_fragments h, time_ctx and
£(1+k). Execute the existing explicit Skip;;e setup and preserve any callback
postcondition Φ : val -> iProp Σ. The newly constructed typeG must wrap the
EXACT outer lrustGS; do not initialize a second physical heap. No caller-supplied
owner, permission or lifetime guard. Keep closure, setup steps and credits
explicit, and reuse the already checked reference construction.

Compose this interface with initialized_interval_frame_adequacy so that a
callback proving returned interval resources yields observations of the actual
final interval and exact original outside frame. Length, semantic property
and actual final lookups must refer to the same returned values. Generic n,
including 0/10/128, arbitrary bytes and strict interiors remain covered.
The callback's WP remains a proof obligation; this composition cannot promise
to recover physical resources merely from an owned-reference resolution.

Freeze each candidate before trials, preserve failures as development evidence,
compile normally and recursively kernel-check complete assumption inventories.
No source runtime rewrite, new axioms or trusted contracts, operational changes,
application premises or native facts. Model setup is not the incoming Rust
borrow, and actual Rust provenance/ancestor/phase/C-F correspondence plus the
full frozen HMAC prefix-and-suffix theorem remain open. Publication stays false.
