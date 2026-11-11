; Qualified finite observational fragment, NOT a full-vstd/native countermodel.
; A slice on an empty borrowed Vec may have hidden decrease Some(1): no
; constructor metric=length equation exists. Use the SAME slice inside e and j.
(set-logic ALL)
(declare-datatypes () ((Slice s) (Enum e j) (Rev r)))
(declare-sort Ref 0)
(declare-datatypes () ((Pair (pair (index Int) (value Ref)))))
(define-fun W () Int 18446744073709551615)
(define-fun slice_remaining ((x Slice)) (Seq Ref) (as seq.empty (Seq Ref)))
(define-fun slice_decrease ((x Slice)) Int 1) ; Some payload; all metrics Some
(define-fun slice_lawful ((x Slice)) Bool true)
(define-fun slice_count_supported ((x Slice)) Bool true)
(define-fun slice_none ((x Slice)) Bool true)
(define-fun enum_inner ((x Enum)) Slice s)
(define-fun enum_index ((x Enum)) Int (ite (= x e) 0 W))
; Seq::new(0, indexed tuples) is empty for either index, including W.
(define-fun enum_remaining ((x Enum)) (Seq Pair) (as seq.empty (Seq Pair)))
(define-fun enum_decrease ((x Enum)) Int (slice_decrease (enum_inner x)))
(define-fun enum_none ((x Enum)) Bool (slice_none (enum_inner x)))
(define-fun enum_count_supported ((x Enum)) Bool (slice_count_supported (enum_inner x)))
(define-fun enum_lawful ((x Enum)) Bool
  (and (slice_lawful (enum_inner x)) (slice_count_supported (enum_inner x))
       (<= (+ (enum_index x) (enum_decrease x)) W)))
(declare-fun rev_inner (Rev) Enum)
(define-fun rev_remaining ((x Rev)) (Seq Pair) (enum_remaining (rev_inner x))) ; reverse(empty)=empty
(define-fun rev_none ((x Rev)) Bool (enum_none (rev_inner x)))
(define-fun rev_decrease ((x Rev)) Int (enum_decrease (rev_inner x)))
(define-fun rev_lawful ((x Rev)) Bool (enum_lawful (rev_inner x)))
; Slice count/metric bounds, constructor remaining, and enumerate post instance.
(assert (<= 0 (slice_decrease s)))
(assert (<= (slice_decrease s) W))
(assert (<= (seq.len (slice_remaining s)) W))
(assert (= (enum_inner e) s))
(assert (= (enum_index e) 0))
(assert (enum_lawful e))
; Actual rev post observations; stronger equal payload, not only Some agreement.
(assert (= (rev_remaining r) (enum_remaining e)))
(assert (= (rev_none r) (enum_none e)))
(assert (= (rev_decrease r) (enum_decrease e)))
(push 1)
(assert (not (rev_lawful r)))
(check-sat)
(get-value ((rev_inner r) (enum_lawful e) (enum_lawful j) (rev_decrease r)))
(pop 1)
; The missing binding would suffice in this fragment. It is NOT installed.
(push 1)
(assert (= (rev_inner r) e))
(assert (not (rev_lawful r)))
(check-sat)
(pop 1)
