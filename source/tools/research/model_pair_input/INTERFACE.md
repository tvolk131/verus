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

## Precise implementation handoff (pretrial)

Namespace `ModelPairInput.setup`; definitions `byte_values`, `byte_map`,
`pair_outside`, `shared_key_view`, `pair_setup`.

`pair_setup e := (Skip;; Skip;; Share;; e)` (right associated).
`byte_values zs := (fun z : Z => LitV (LitInt z)) <$> zs`.
`byte_map p zs := interval_map p (byte_values zs)`.
`pair_outside h p pad q key := h ∖ (byte_map p pad ∪ byte_map q key)`.
`shared_key_view key q tid κ := tctx_interp tid
  +[#q ◁ &shr{κ} (bytes_ty (length key))]
  -[((q, repeat [] (length key)), bytes_of_list key)]`.

`raw_pair_model_context_wp Outer e h p pad q key Φ k` requires closed `e`,
byte ranges and exact raw lookups for both lists, and
`byte_map p pad ##ₘ byte_map q key`. Its continuation is:

```coq
forall (TG : typeG Σ) tid κpad κkey,
  @type_lrustGS Σ TG = Outer ->
  κpad <> llft_empty -> κkey <> llft_empty ->
  llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗
  elctx_interp [] -∗
  llctx_interp [κpad ⊑ₗ []] -∗ llctx_interp [κkey ⊑ₗ []] -∗
  invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
  model_reference pad p tid κpad -∗
  shared_key_view key q tid κkey -∗
  initial_fragments (pair_outside h p pad q key) -∗
  £k -∗ WP e @ ⊤ {{ Φ }}
```

The conclusion is `initial_fragments h -∗ £(1+k) -∗ time_ctx -∗
WP (pair_setup e) @ ⊤ {{ Φ }}`, with Φ and all heap resources in the fixed
Outer. Two independently allocated lifetime handles are passed exactly once;
there is no distinct-allocation-block premise. Empty key at q=p satisfies the
disjointness premise automatically. `shared_key_view` is an actual ordinary
shared-reference tctx, usable through `tctx_hasty_val` as an existential depth,
time receipt and shared `ty_own`. Its physical key ownership stays under the
shared lifetime guard; clients receive no exclusive key payload.

Public `raw_pair_model_observed` will initialize the raw heap, setup credits
and time context internally, preserve an arbitrary family `Φ Outer`, and use a
resource observer `heap_ctx σ -∗ Φ Outer v ={⊤,∅}=∗ ⌜φ v σ⌝` in that same
Outer. This does not supply or assume a final-memory fact. Root remains
responsible for pad mutation/recovery and the final observer; the shared-read
consumer may derive a read independently from this exact view.
