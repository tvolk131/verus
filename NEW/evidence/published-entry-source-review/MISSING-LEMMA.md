# Small missing sequents (statements only, not Coq-checked)

The following uses the actual published `state`, `loc`, `tag`, `ptr_id`,
`call_id`, `call_id_stack`, `stacks`, `retag`, `valid_block`, and candidate
`mutable_top_block`. It is a proposed ordinary model lemma, not an axiom to
import. No `.v` file, stub Rust semantics or certificate is introduced.

```coq
forall (sigma : state) (l : loc) (old : tag) (n : nat)
       (c : call_id) (cs : call_id_stack),
  (0 < n)%nat -> Wf sigma -> sigma.(scs) = c :: cs ->
  valid_block sigma.(sst) sigma.(scs) l old
              (RefPtr Mutable) (FixedSize n) ->
  exists (p : ptr_id) (alpha' : stacks) (next' : ptr_id),
    retag sigma.(sst) sigma.(snp) sigma.(scs) c l old FnEntry
          (RefPtr Mutable) (FixedSize n) = Some (Tagged p, alpha', next') /\
    mutable_top_block alpha' l p n.
```

This derives the successful **past model FnEntry**, rather than assuming it or
assuming top-head. The source suggests a short proof using
`retag_is_freeze_is_Some`, `retag_change_ref_NZST`, and
`tag_on_top_retag_ref_uniq` (steps_progress.v:763; steps_retag.v:805,696).
The latter unfolds to `UniqueRef false` for `FnEntry`; `tag_on_top` is
a head-lookup equality with the permission/tag and an existential protector.
For n=0 top is vacuous, but the returned tag can be old/Untagged; do not claim
the displayed Tagged-result conclusion for n=0. This lemma is NOT RUN, and
would only be conditional MODEL algebra even if checked.

The earlier, blocking source obligation is narrower than a whole source-safety
theorem but is not supplied by that lemma. Its typed interface can be stated
without inventing an operational Rust instance:

```coq
(* Cfg : Type; EntryArg1 : Cfg -> Prop;
   BeforeArg1Copy : Cfg -> state -> loc -> tag -> Prop.
   These are NOT defined/assumed here: they must be independently specified
   from the original entry domain and the exact source event below. *)
forall (S : Cfg), EntryArg1 S ->
  exists (sigma : state) (l : loc) (old : tag)
         (c : call_id) (cs : call_id_stack),
    BeforeArg1Copy S sigma l old /\ Wf sigma /\
    sigma.(scs) = c :: cs /\
    valid_block sigma.(sst) sigma.(scs) l old
                (RefPtr Mutable) (FixedSize 128).
```

`BeforeArg1Copy` must bind the concrete **caller_arg_copy** read by
`pass_argument`, destination callee `Place::from(Local(1))`, type
`&mut [u8;128]`, its pointee `(alloc_id, base_offset, old provenance)`, and the
same arbitrary original initialized 128 bytes. `EntryArg1` must follow the
original Rust contract/entry semantics; it cannot mean top-head, a grant
success, a desired post, or future defined execution. `valid_block` here is
an obligation to DERIVE from independent permissions/history, not a new client
require. An exact current-Miri-to-historical representation/refinement must
justify using this block predicate, including wildcard/unknown cases; otherwise
this displayed source interface needs a richer target, not a restricted domain.

The source event requiring that block is call.rs:394's
`copy_op_allow_transmute(&caller_arg_copy, &callee_arg)` inside
`with_retag_mode(FnEntry)` (531): its validation reaches
`M::retag_ptr_value(ptr, ty)` (validity.rs:625), then
`sb_reborrow(place, size, Unique/write/strong, new_tag, FnEntry)` and
per-byte `Stack::access(Write, orig_tag, ...)`. No top-head hypothesis is
needed on the **caller** tag if these actual grants and protector checks can
be justified. Bounds, alignment, carrier reads/writes and ABI errors also need
separate operational coverage.

Next required relation: the copy/validation must install the resulting pointer
in actual callee `_1`, preserving allocation/subobject/bytes; all later argument
and return-carrier protection and frame-init hooks must finish; then the three
actual root `StorageLive(2/3/4)` events must reach bb0[3] preserving its referent
head, active frame/protectors, and `_1` binding. Only then may a related
`sigma3` satisfy `mutable_top_block sigma3.(sst) l p 128` for evaluating
`_4 = &mut (*_1)`. This source-prefix relation is unproved, not a premise
silently filled by a hash or a past event from another call/body/phase.

At bb0[3] itself one still needs an all-outcomes source-operation relation:
`eval_place(*_1, true)`, `force_allocation`, `to_ref`, TwoPhase validation/retag,
carrier write to `_4`, local-write hooks, and successful PC advance, plus every
error/stop/unwind outcome. A published `Retag` heap-frame theorem supplies
only the retag fragment of that relation. There is no source `Cfg` instance,
coverage proof, source C/F binding, or checked sequent in this review.
