(* Review-only controller probe. No native theorem or production consumer. *)
From Coq Require Import Classical_Prop.
Theorem review_excluded_middle (P : Prop) : P \/ ~ P.
Proof. exact (classic P). Qed.
Print Assumptions review_excluded_middle.
