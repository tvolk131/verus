# One heap and typing context for a mutable pad and shared key

Construct a resource-valued MODEL setup in one fixed Outer lrustGS and one
internally allocated TypeG, starting from genuine initial raw-map fragments.
Inputs are initialized byte lists pad/key at p/q, with disjoint nonempty
intervals and exact raw lookups. Support arbitrary pad/key lengths, including
10/128-byte pads and keys 0, below, equal to and above 128. Disjoint regions may
be interior subobjects of the same allocation. Empty keys impose no physical
cell access or blanket nonaliasing requirement (q=p is permitted when empty).

Derive both references internally with existing bytes_reference_construct,
and use the existing checked `Share` / type_share_instr rule to turn the key
into the actual semantic shared-reference type. Handoff should provide the pad
model_reference, shared key view `((q, repeat [] (length key)), bytes_of_list key)`,
ordinary contexts for their two fresh model lifetimes and an exact outside
frame. Prefer separately usable Lpad/Lkey components so opening the pad guard
does not hide the key context needed for reading. It is fine to provide a
proved recombination law. Preserve arbitrary resource-valued continuation Φ
in the SAME Outer heap, following ModelInputObservation.raw_model_context_wp.
Make all setup steps, tickets and credits explicit, internally supplied by
the public initialized-heap adequacy wrapper. No client permissions/guard input.

This is fresh MODEL initialization, not interpretation of an actual incoming
Rust lifetime, provenance or ancestor history. Do not claim otherwise. An exact
two-region final observer adapter is useful if it follows from genuine returned
resources; it must not assume final-memory observations. Root separately handles
the changing pad's child loan and original-parent physical recovery.

Only this new directory is owned. Baseline/vstd/compiler/shared tools and sibling
directories stay read-only. Freeze each candidate before trial; normal compile,
complete declaration/obligation assumptions inventory, recursive kernel checking,
compact source/object/receipt hashes. No axioms, new primitives, trusted summaries,
unsafe settings, fixture/domain weakening or publication. Native correspondence
and full HMAC completion remain unproved.
