# Exact typed review milestone (before review compilation)

Scope: read-only independent review of `model_pair_input` at `27c0422f`
(18 declarations) and `model_shared_byte_read` at `bef470ad` (19 declarations).
`pretrial.json` authenticates both requested completed receipts, every returned
source/object (including generated Assumptions), both committed final freezes,
all frozen controller/support/pin inputs, and the exact implementation revisions.
`statement-inventory.json` independently enumerates every definition, theorem and
instance, its exact statement, and individual frozen assumptions. Neither source
contains constructors, projections, Program definitions or obligations.

The pair handoff uses one `Outer : lrustGS Σ` and internally constructs one
`TG : typeG Σ` with `type_lrustGS TG = Outer`. Separate ordinary contexts are
`llctx_interp [κpad ⊑ₗ []]` and `llctx_interp [κkey ⊑ₗ []]`. It supplies
`model_reference pad p tid κpad` and
`ModelPairInput.setup.shared_key_view key q tid κkey`, the latter equivalent to
`∃ d, ⧖d ∗ ty_own (&shr{κkey} (bytes_ty (length key)))
((q, repeat [] (length key)), bytes_of_list key) d d tid [FVal #q]`.
There is no fixed `d`, no fixed output `g`, and no normalized combined budget.

The distinct `ModelSharedByteRead.shared_read.shared_key_view n κ q x d g tid`
is the existing `ty_own` at arbitrary `d,g`. Preparation costs `g+1`; reusable
SC read costs `g+1`; direct read costs `2*(g+1)`. Session setup supplies an
existential linear token `R` and guard token `G`; the persistent reader still
requires `R` per read, and closure joins both to restore ordinary `L`.

Review will check real checked Share, exact physical read and range, arbitrary
separating Frame/aliases, exact outside frame, arbitrary lengths and same-block
interiors, empty key `q=p`, no public physical-permission input, and actual
same-heap observation at existential `RSt r`. Namespace qualification is
required for integration. These are separate MODEL interfaces, with no new
combined-loop/public/native theorem in this review.

Next: use the unmodified frozen controllers with mandatory supported pins,
fresh normal `coqc`, complete individual assumptions and full recursive
`coqchk -silent -o` for each library in private outputs. Accept only successful
durable receipts and authenticate all fresh bytes against the completed results.
No source edits, substitutions, excluded experiments, Rust rebuilds or subagents.
