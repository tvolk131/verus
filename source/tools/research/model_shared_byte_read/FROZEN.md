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
