From ModelCallerWriter Require Export caller.
From ModelOrdinarySharedXor Require Export xor_na.
Set Default Proof Using "Type".
Open Scope Z_scope.

Definition caller_xor_na_body n k (p q : loc) : expr :=
  App (of_val zip_transform_na)
    [of_val byte_xor; Lit (LitInt (Z.of_nat (Nat.min n k)));
     Lit (LitLoc p); Lit (LitLoc q)].
Definition caller_xor_na_procedure n k p q : expr :=
  (Newlft;; ReBor;; caller_xor_na_body n k p q;; Endlft;; Skip)%E.
Definition caller_xor_na_credit n k g : nat :=
  ((g+1) + (1 + shared_xor_na_cost (Nat.min n k) g))%nat.

Section interface.
  Context `{!typeG Σ}.
  (* Exact milestone. Existing typeG, E/L/I, and parent remain in scope.
     κ need not have future=current, and Frame may hold ancestor restorations. *)
  Definition caller_xor_na_contract n k E L I tid iκs κ κkey (p q : loc)
      (x : ~~(bytes_sort n)) (key : ~~(bytes_sort k)) m d g
      (Frame : iProp Σ) : iProp Σ :=
    (llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
     llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
     tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty n)) m -∗
     shared_key_view k κkey q key d g tid -∗
     £(caller_xor_na_credit n k g) -∗ Frame -∗
     WP caller_xor_na_procedure n k p q {{ _,
       Frame ∗ shared_key_view k κkey q key d g tid ∗
       elctx_interp E ∗ llctx_interp L ∗ invctx_interp tid ⊤ iκs I ∗
       ∃ m', tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty n)) m' ∗
         ⟨π, vπ m' π =
           (vπ (xor_bytes n x (byte_list k key)) π, uniq_bor_future m π)⟩ ∗
         ⌜byte_list n (xor_bytes n x (byte_list k key)) =
             xor_result (byte_list n x) (byte_list k key) ∧
           length (xor_result (byte_list n x) (byte_list k key)) = n ∧
           take (Nat.min n k) (xor_result (byte_list n x) (byte_list k key)) =
             xor_prefix (byte_list n x) (byte_list k key) ∧
           drop (Nat.min n k) (xor_result (byte_list n x) (byte_list k key)) =
             drop (Nat.min n k) (byte_list n x) ∧
           Forall byte_value (xor_result (byte_list n x) (byte_list k key))⌝ }})%I.
End interface.
