# Exact typed MODEL milestone

`candidates/shared_xor.v` defines `shared_xor_contract`: the unchanged imported
`zip_transform` applied to imported `byte_xor`, at the actual key address
`q + start`, with literal `byte_list n x` result, explicit linear R, separate
persistent Cap, arbitrary Frame, and count*(g+2) credits. This is a typed
assertion to prove, not an axiom or assumed operation summary. Its intended
side conditions are count<=length pad, start+count<=n and initialized byte pad.
The checked session supplies both Reader and Cap; preparation is outside.

The first checkpoint also proves pure cost splitting, byte range, unchanged
length and unchanged suffix. No implementation theorem or Rust correspondence
is claimed at this milestone. Replay is frozen before each trial; failed
proof/setup trials are retained and excluded from semantic evidence.

Checkpoint trial004 passed normal `coqc`; its recursive kernel check was still
running when this milestone was committed. Trials001–003 were development/setup
failures, not semantic negatives. Final evidence will record all four receipts.
