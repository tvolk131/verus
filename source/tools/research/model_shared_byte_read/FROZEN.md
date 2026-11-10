# Pretrial typed handoff

Namespace: `ModelSharedByteRead.shared_read`. All parameters below are generic:
`n i d g : nat`, `q : loc`, `x : ~~(bytes_sort n)`, `tid`, `κ`, `E`, `L`.
`view := ty_own (&shr{κ} (bytes_ty n)) ((q,repeat [] n),x)
  d g tid [FVal #q]` is the exact existing type (not a new ownership primitive).
It is persistent. A usable view necessarily has successor depth; no fixed
reference depth or `d=g` restriction is intended. In particular d=1,g=0 works.
No exclusive key vector, supplied guard, uniqueness context or time receipt is
required by the read law. `lctx_lft_alive E L κ` is the only lifetime side condition.

Intended precise laws, before the first proof trial:

* Preparation: `llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗ view -∗
  £(g+1) ={⊤}=∗ llctx_interp L ∗ capability`.
  `capability` persistently packages that SAME view and
  `⌜Forall (fun z => 0 <= z < 256) (byte_list n x)⌝`.
  Ghost guard opening uses delay g+1, restores L and has zero program steps.
* Reusable read: `i<n -> llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
  capability -∗ Frame -∗ £(g+1) -∗
  WP Read ScOrd #(q +ₗ i) @ ⊤ {{ v, ∃ z,
    ⌜byte_list n x !! i = Some z /\ v = #z /\ 0 <= z < 256⌝ ∗
    llctx_interp L ∗ capability ∗ Frame }}`.
  One existing SC program step; physical guard delay g+1, exactly g+1
  explicit input credits. No context allocation or caller guard. Arbitrary
  separating Frame may include more copies of the capability or other shared
  aliases. E and llft_ctx are persistent and remain available.
* Direct view read composes the above: 2*(g+1) credits, one SC step,
  identical view and ordinary L returned. At g=0: preparation 1, each reusable
  read 1, direct read 2. This is a sufficient explicit budget, not a lower bound.

Existing `read_shr` is a whole-object copy accessor with d=g and d+1 credits;
its consumer `type_deref` uses a non-atomic read. The indexed SC implementation
will instead specialize the SAME physical guard and use
`wp_read_sc_guarded_cells`, preserving the whole shared view. The byte range
comes from the byte ghost guard and `bytes_initialized`, not an app premise.
The physical guard starts at NllftG and is weakened to naN as in baseline code.

Sharing input remains exactly baseline `type_share_instr`:
`lctx_lft_alive E L κ -> typed_instr E L I +[p ◁ &uniq{κ}ty] Share
 (const +[p ◁ &shr{κ}ty])
 (fun post '-[m] => fun mask π =>
   (vπ m π).1 = (vπ m π).2 ->
   post -[(uniq_bor_loc m,uniq_bor_current m)] mask π)`.
This law creates physical/ghost guards of delay d+1 in the typed-instruction
convention (output ghost depth d); it does not require a caller guard.
The read interface does not change arbitrary mutable callbacks Rin -> Rout.

Every trial freezes sources, controllers and support identities first, compiles
normally in a fresh private /tmp directory, inventories every declaration and
Program obligation, and recursively coqchks all modules without exclusions.
Failures are development evidence. No new axioms, admissions or primitives.
Only inherited baseline logical axioms may occur and will be listed exactly.
Optional same-heap observation or model composition will be added only with
separate frozen statements; no claim of exclusive key extraction.

Actual native correspondences ZERO; 44 minimal + 3 phase / 101 writer effects
remain open. Canonical 6/0 and frozen full HMAC prefix+suffix 1/1 inherited,
not evidence supplied by this work. Publication and full-goal completion false.

## Additional scope, frozen before its trials

`shared_key_heap_observe` takes the genuine `heap_ctx σ`, ordinary E/L,
capability, arbitrary Frame and g+1 credits. For each i<n it returns all of
these (same σ) and `∃ z r, byte_list n x !! i = Some z /\
σ.1 !! (q +ₗ i) = Some (RSt r,#z) /\ 0 <= z < 256`.
It executes zero program steps and uses baseline `atomic_read` to inspect the
same authoritative non-atomic map. The read state r is existential, not forced
to zero. This is observation with a restored heap, never exclusive extraction.

Checked specializations will cover empty-index impossibility, depth-zero
impossibility, capability duplication, arbitrary-list input through the pinned
ModelInputContext.vector representation, explicit same-key alias framing and a
129-byte/index-128 read. They do not change the generic law or native scope.

## Scoped callback reader, frozen before its trials

`key_reader n κ q x d g tid R` is a persistent universally quantified indexed
SC read law: for any i<n and separating Frame, it takes the linear token R,
Frame and g+1 credits; it returns R, Frame, the same capability and ranged byte
lookup. It needs no E/L or supplied guard at read time.

`shared_key_reader_session` starts with ordinary E/L, the same shared view and
g+1 preparation credits. It internally allocates two fractional context halves
and returns `∃ R G, ⌜Timeless R /\ Timeless G⌝ ∗ R ∗ G ∗ capability ∗
(G &&{NllftG}&&> llctx_interp L) ∗ key_reader ... R ∗
(R -∗ G ={top}=∗ llctx_interp L)`.
The G side supports existing guarded mutable-pad access; R stays available
through its arbitrary Rin->Rout callback. Each key read restores R. Closing pad
access restores G, then joining R/G restores ordinary L. No root callback edits,
exclusive key payload, lifetime extension or new caller guard. The persistent
reader requires its internally generated linear R, so it cannot outlive closure.
Setup executes zero program steps; each read executes one SC step. No credits
beyond preparation g+1 and g+1 per read. Generic n, depth, g and arbitrary Frame.
