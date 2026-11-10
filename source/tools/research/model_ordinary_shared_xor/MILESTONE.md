Typed source milestone (unvalidated development source)

The candidates state a Na1Ord indexed reader/session and generic start/count XOR
loop contract. The runtime loop is copied from frozen zip_transform with only
its two read orders and one write order changed. Cap is a separate input even
in the zero-count case. The prepared session requires no extra preparation
credit; the raw session invokes shared_key_prepare with g+1. Reads use the
existing guarded-cell Na primitive at 3*(g+1), and full-cell pad writes use
wp_write_na without an input store credit. These are statements and candidate
proofs, not checked evidence until a successful frozen receipt is recorded.

Scope remains MODEL partial correctness only: native0; 44+3/101 open;
publication false; goal incomplete. Canonical6/0 and HMAC1/1 are inherited,
not rerun. No initialized model input/provenance or Rust/C/F claim is made.

Subsequent validation: the exact final sources passed normal coqc and full
recursive coqchk in trial008. See results.json and HANDOFF.md for pins and limits.
