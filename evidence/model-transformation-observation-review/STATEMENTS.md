# Exact seven-declaration statement inventory
Extracted from the unchanged sources at `d639f378`. Internal laws have
`Context `{!typeG Σ}`; the public theorem has `{Σ : gFunctors}` and
`{!typePreG Σ}`. All claims are MODEL-relative. Full sources are authenticated
by `audit-results.json`; proof objects and logs remain private.

## resource_protocol.shorter_run_resources

```coq
  Lemma shorter_run_resources {𝔄} E L I tid iκs p (ty : type 𝔄) κ m (x_final : ~~𝔄) e (Rin Rout Frame : iProp Σ) :
    lctx_lft_alive E L κ → lctx_lft_alive E L (ty_lft (&uniq{κ} ty)%T) →
    Closed [] e →
    (∀ α c, uniq_bor_loc c = uniq_bor_loc m → uniq_bor_current c = uniq_bor_current m →
      llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
      llctx_interp (α ⊑ₗ [κ] :: L) -∗ invctx_interp tid ⊤ iκs I -∗
      tctx_elt_interp tid (p ◁ &uniq{α} ty) c -∗ Rin -∗
      WP e {{ _, Rout ∗ ∃ c', llctx_interp (α ⊑ₗ [κ] :: L) ∗ invctx_interp tid ⊤ iκs I ∗
        tctx_elt_interp tid (p ◁ &uniq{α} ty) c' ∗
        ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
             vπ (uniq_bor_current c') π = vπ x_final π⟩ }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
    tctx_elt_interp tid (p ◁ &uniq{κ} ty) m -∗ Rin -∗ Frame -∗
    WP (Newlft;; ReBor;; e;; Endlft;; Skip) {{ _,
      Frame ∗ Rout ∗ llctx_interp L ∗ invctx_interp tid ⊤ iκs I ∗
      ∃ m', tctx_elt_interp tid (p ◁ &uniq{κ} ty) m' ∗
      ⟨π, vπ m' π = (vπ x_final π, uniq_bor_future m π)⟩ }}.
```

## child.bytes_vector_mapsto

```coq
  Lemma bytes_vector_mapsto n (x : ~~(bytes_sort n)) tid p :
    (p,repeat [] n) #↦!∗ ty_phys (bytes_ty n) x tid ⊣⊢
    p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list n x).
```

## child.byte_child_transform

```coq
  Lemma byte_child_transform n E L α p (x0 x1 : ~~(bytes_sort n)) c tid e
      (Rin Rout : iProp Σ) :
    lctx_lft_alive E L α →
    uniq_bor_loc c = (p,repeat [] n) → uniq_bor_current c = x0 →
    Forall (λ z, 0 ≤ z < 256) (byte_list n x1) →
    (Forall (λ z, 0 ≤ z < 256) (byte_list n x0) →
      p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list n x0) -∗ Rin -∗
      WP e {{ _, p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list n x1) ∗ Rout }}) →
    llft_ctx -∗ time_ctx -∗ uniq_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty n)) c -∗ £1 -∗ Rin -∗
    WP e {{ _, Rout ∗ ∃ c', llctx_interp L ∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty n)) c' ∗
      ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
           vπ (uniq_bor_current c') π = vπ x1 π⟩ }}.
```

## initialized.initialized_nested_run

```coq
  Lemma initialized_nested_run zs p tid κ (x1 : ~~(bytes_sort (length zs)))
      e (Rin Rout Frame : iProp Σ) :
    Closed [] e →
    (∀ α c,
      uniq_bor_loc c = (p,repeat [] (length zs)) →
      uniq_bor_current c = bytes_of_list zs →
      llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
      llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c -∗ Rin -∗
      WP e {{ _, Rout ∗ ∃ c', llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) ∗
        invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) ∗
        tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c' ∗
        ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
             vπ (uniq_bor_current c') π = vπ x1 π⟩ }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ Rin -∗ Frame -∗
    WP (Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip) {{ _,
      Frame ∗ Rout ∗ llctx_interp [κ ⊑ₗ []] ∗
      ∃ ξi ζi idx,
      let x := bytes_of_list zs in
      let l := (p, repeat [] (length zs)) in
      let m := (l,x,ξi,0%nat,0%nat,idx) in
      let ξ := uniq_prvar x ξi in
      let ζ := PrVar (at_locₛ (bytes_sort (length zs)) ↾
                      prval_to_inh (@vπ (at_locₛ (bytes_sort (length zs))) (p,x))) ζi in
      ⟨π, uniq_bor_future m π = vπ x1 π⟩ ∗
      ⟨π, π ζ = (p,(π ξ).1)⟩ ∗
      ([†κ] ={↑Nllft}=∗ ▷ ∃ x' (d' g' : nat),
        ((λ π, (π ξ).1) :== vπ x') ∗ ⧖(S d' `max` g') ∗
        ty_gho (bytes_ty (length zs)) x' d' g' tid ∗
        l #↦!∗ ty_phys (bytes_ty (length zs)) x' tid) }}.
```

## initialized.initialized_nested_payload_run

```coq
  Lemma initialized_nested_payload_run zs p tid κ
      (x1 : ~~(bytes_sort (length zs))) e (Rin Rout Frame : iProp Σ) :
    Closed [] e →
    (∀ α c,
      uniq_bor_loc c = (p,repeat [] (length zs)) →
      uniq_bor_current c = bytes_of_list zs →
      llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
      llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c -∗ Rin -∗
      WP e {{ _, Rout ∗ ∃ c', llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) ∗
        invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) ∗
        tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c' ∗
        ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
             vπ (uniq_bor_current c') π = vπ x1 π⟩ }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ Rin -∗ Frame -∗
    WP ((Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip);; Endlft) {{ _,
      Frame ∗ Rout ∗ p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) }}.
```

## transform.initialized_nested_transform

```coq
  Lemma initialized_nested_transform zs p tid κ
      (x1 : ~~(bytes_sort (length zs))) e (Rin Rout Frame : iProp Σ) :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) (byte_list (length zs) x1) →
    (Forall (λ z, 0 ≤ z < 256) zs →
      p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> zs) -∗ Rin -∗
      WP e {{ _, p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) ∗ Rout }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ £1 -∗ Rin -∗ Frame -∗
    WP ((Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip);; Endlft) {{ _,
      Frame ∗ Rout ∗ p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) }}.
```

## observed.raw_model_transform_observed

```coq
  Theorem raw_model_transform_observed h p zs
      (x1 : ~~(bytes_sort (length zs))) e k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) zs →
    Forall (λ z, 0 ≤ z < 256) (byte_list (length zs) x1) →
    (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (TG : typeG Σ),
      Forall (λ z, 0 ≤ z < 256) zs →
      p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> zs) -∗ £k -∗
      WP e {{ _, p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) }}) →
    adequate NotStuck
      (Skip;; ((Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip);; Endlft))%E
      (initialized_state h)
      (λ _ σ, ∃ vs, length vs = length zs ∧
        vs = ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) ∧
        interval_observed σ p vs ∧
        frame_observed σ (h ∖ interval_map p ((λ z : Z, LitV (LitInt z)) <$> zs))).
```
