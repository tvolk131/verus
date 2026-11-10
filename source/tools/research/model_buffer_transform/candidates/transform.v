From lrust.lang Require Import proofmode notation lifting heap.
Set Default Proof Using "Type".
Open Scope Z_scope.

Definition byte_value (z : Z) : Prop := 0 ≤ z < 256.
Definition zvalues (zs : list Z) : list val := (λ z : Z, LitV (LitInt z)) <$> zs.
Fixpoint transform_prefix (n : nat) (f : Z → Z → Z) (xs ys : list Z) : list Z :=
  match n,xs,ys with
  | S m,x::xs,y::ys => f x y :: transform_prefix m f xs ys
  | _,_,_ => xs
  end.

Definition zip_transform : val :=
  rec: "loop" ["op"; "n"; "pad"; "key"] :=
    if: "n" ≤ #0 then #☠ else
      let: "a" := !ˢᶜ"pad" in
      let: "b" := !ˢᶜ"key" in
      let: "c" := "op" ["a"; "b"] in
      "pad" <-ˢᶜ "c";;
      "loop" ["op"; "n" - #1; "pad" +ₗ #1; "key" +ₗ #1].

Lemma transform_prefix_length n f xs ys :
  (n ≤ length xs)%nat → (n ≤ length ys)%nat →
  length (transform_prefix n f xs ys) = length xs.
Proof.
  revert xs ys. induction n; intros [|x xs] [|y ys]; simpl; intros; try lia.
  rewrite IHn; auto with lia.
Qed.

Lemma transform_prefix_suffix n f xs ys :
  (n ≤ length xs)%nat → (n ≤ length ys)%nat →
  drop n (transform_prefix n f xs ys) = drop n xs.
Proof.
  revert xs ys. induction n; intros [|x xs] [|y ys]; simpl; intros; try done; try lia.
  apply IHn; lia.
Qed.

Section operational.
  Context `{!lrustGS Σ}.
  Definition binary_operation_spec (f : Z → Z → Z) (op : val) : iProp Σ :=
    □ (∀ a b, ⌜byte_value a⌝ -∗ ⌜byte_value b⌝ -∗
      WP (App (of_val op) (Lit (LitInt a) :: Lit (LitInt b) :: nil)) {{ w, ⌜w = #(f a b)⌝ }}).

  Lemma wp_zip_transform n f op p q xs ys :
    (n ≤ length xs)%nat → (n ≤ length ys)%nat →
    Forall byte_value xs → Forall byte_value ys →
    binary_operation_spec f op -∗
    {{{ p ↦∗ zvalues xs ∗ q ↦∗ zvalues ys ∗ £n }}}
      (App (of_val zip_transform) (of_val op :: Lit (LitInt (Z.of_nat n)) :: Lit (LitLoc p) :: Lit (LitLoc q) :: nil))
    {{{ RET LitV LitPoison;
      p ↦∗ zvalues (transform_prefix n f xs ys) ∗ q ↦∗ zvalues ys }}}.
  Proof.
    revert p q xs ys. induction n as [|n IH]; intros p q xs ys BoundX BoundY RangeX RangeY.
    - iIntros "#Op" (Φ). iModIntro. iIntros "[Pad [Key Credit]] Post".
      wp_rec. wp_op. wp_if. iApply "Post". by iFrame.
    - destruct xs as [|a xs],ys as [|b ys]; simpl in BoundX,BoundY; try lia.
      apply Forall_cons in RangeX as [RangeA RangeX].
      apply Forall_cons in RangeY as [RangeB RangeY].
      iIntros "#Op" (Φ). iModIntro. iIntros "[Pad [Key [Credit Credits]]] Post".
      wp_rec. wp_op. case_bool_decide; wp_if; first lia.
      rewrite /zvalues /= !heap_mapsto_vec_cons.
      iDestruct "Pad" as "[PadHead Pad]". iDestruct "Key" as "[KeyHead Key]".
      wp_bind (Read ScOrd #p).
      iApply (wp_read_sc with "PadHead"); first done.
      iNext. iIntros "PadHead". wp_let.
      wp_bind (Read ScOrd #q).
      iApply (wp_read_sc with "KeyHead"); first done.
      iNext. iIntros "KeyHead". wp_let.
      wp_bind (App (of_val op) (Lit (LitInt a) :: Lit (LitInt b) :: nil)).
      iPoseProof ("Op" $! a b with "[] []") as "Run"; [done|done|].
      iApply (wp_wand with "Run"). iIntros (w) "%Eq". subst w.
      wp_let. wp_bind (Write ScOrd #p _).
      iApply (wp_write_sc ⊤ p _ _ _ with "[PadHead Credit]"); [done|by iFrame|].
      iNext. iIntros "PadHead". wp_seq. do 3 wp_op.
      replace (Z.of_nat (S n) - 1) with (Z.of_nat n) by lia.
      iApply (IH (p +ₗ 1) (q +ₗ 1) xs ys with "Op [Pad Key Credits]");
        [lia|lia|done|done|by iFrame|].
      iNext. iIntros "[Pad Key]". iApply "Post".
      rewrite /zvalues /= !heap_mapsto_vec_cons. by iFrame.
  Qed.
End operational.
