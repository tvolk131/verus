# Ordinary two-phase byte access in the checked model

The existing model XOR cell/loop uses SC accesses. Native-source correspondence
must justify the original ordinary memory accesses; it cannot silently identify
SC and non-atomic steps. Establish the precise existing-lrust non-atomic
primitive executions and a read/XOR/same-cell-write WP with unchanged frame.

Use actual Na1Ord/Na2Ord head/primitive rules, with explicit lookup/ownership
premises. Reads permit arbitrary RSt reader counts; their isolated two-phase
execution restores the original state. Writes require RSt 0 and pass through
WSt before the final value. Track the different number of steps and temporary
state changes, without claiming purity or unrestricted concurrent equivalence.
Compare final effects with the existing SC rules only under their shared
premises. Reuse the existing checked byte_xor and independent scalar_xor8 law.

Internal WP ownership stays explicit and must later be derived by public model
setup. Do not append it to the original application's requirements. Preserve
the original frozen Rust body/domain. These laws remain MODEL facts: actual
MIR memory/provenance, phases, input-context and C/F simulation are unproved.
Native correspondences/effect counters remain zero/open; publication false.

Own this new directory only. Read-only pinned foundation/dependencies; normal
compilation, pretrial freezes, complete declaration/assumption inventory and
recursive kernel checking. No new axioms, primitives, unsafe shortcuts, baseline
changes or sibling checkout reads. Keep raw logs/objects private.
