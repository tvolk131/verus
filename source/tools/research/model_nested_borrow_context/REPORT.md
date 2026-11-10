# Contextual model nested borrowing

This is a formal MODEL implementation, not a Rust entry theorem. Publication
remains false. It adds no production facts, assumptions, validity predicate,
semantic rewrite, application permission, or change to the application domain.

## Checked interfaces and composition

`protocol.shorter_no_write` executes `Newlft;; ReBor;; Endlft;; Skip`.
An existing unique context element at path p becomes a shorter child and a
blocked original parent using `tctx_reborrow_uniq`. The child is resolved before
its fresh lifetime token is ended. The existing `unblock_tctx` rule recovers the
original-lifetime parent at the same path. The postcondition is
`vπ recovered = (vπ original_current, original_future)`. It does not claim that
the original parent's future already equals its current value.

The arbitrary iProp Frame is returned exactly. In particular it may contain an
arbitrary `tctx_interp tid T xl` and any ancestor restoration continuation.
Neither the parent nor Frame has to be a fresh top-level allocation. The
ordinary E/L contexts derive the child guard; no separate client permission
is introduced. Returned parent metadata can reflect changed depths or borrow
indices; the exposed identity is the original physical path/pointer and original
future prophecy observation, not a claim that all bookkeeping fields are equal.

`protocol.shorter_run` also executes a proved child computation. Its MODEL
internal callback preserves the child future and identifies its final current
value. Resolution and unblocking then transfer that current observation to the
recovered original parent, keeping its original future and Frame.

`writes.byte_child_write` executes `Skip;; Write ScOrd #p #new` on an arbitrary
nonempty vector old::tail. It opens `uniq_body_acc`, performs the physical SC
write and closes the same child body with new::tail, preserving its prophecy
identifier and loan index. The tail is arbitrary. `writes.shorter_write_changes`
composes that callback through child lifetime ending and parent recovery. For
old != new its postcondition proves that the recovered parent's current byte
observation differs from the original current observation. Thus there is no
blanket unchanged law for arbitrary child computations.

`instances.initialized_nested_no_write` consumes the existing initializer's
`model_reference`. It keeps the initializer's exact original prophecy binding
and restoration continuation across the shorter child, recovers the parent,
and only then applies the existing original-parent resolution rule. Its final
future/current equation mentions the initializer's original prophecy, not a
newly chosen child's prophecy. It preserves the arbitrary Frame.

`recovery.initialized_nested_payload` then ends the original parent and invokes
that retained continuation. It converts the existing equalizer using the empty
prophecy-variable inventory of byte vectors, relates the returned current
payload to the resolved original future, and returns the exact physical
`p ↦∗ map LitInt zs` together with Frame. This is the resource-return interface
for the parent's completed `ModelInputObservation.setup.raw_model_context_wp`
and `raw_model_context_observed`. That delivered setup was read at 6692623b;
this lane does not import or inspect the active observer implementation.

The public `raw_model_nested_read_adequacy` uses the existing initialized model
context, executes the nested protocol and original-parent end, then actually
reads p and returns the initial head byte. The ownership is constructed from
raw initialized heap cells inside the existing initializer, not supplied by the
caller. Generic n, explicit 10/128 instances, arbitrary model heap and strict interior p are supported. The checked physical resource postcondition preserves every byte and the exact arbitrary outside frame; the public read instance observes the restored original pointer.

## Operational and credit accounting

Newlft is a value, but its sequencing beta step is executed. ReBor is exactly
`Seq Skip Skip`, not a zero-step context inclusion: the proof invokes the existing
operational `tctx_reborrow_uniq`/`typed_instr_of_skip` rule. That rule obtains
`£(advance_credits d)`, a persistent receipt `⧖(S d)`, and two separate `⧗1`
tickets from the actual steps. `advance_credits d = 10*d*d+10*d+10`.

Endlft is exactly `Seq Skip (Seq Skip Skip)`. The WP helper retains the existing
resolution delay `d*(d+1)` and unblocking delay `1+d0*(d0+1)`, discharged by the
existing persistent-time step rules with the same squared bounds as
`type_endlft`. It differs from that wrapper only by permitting the different
blocked/recovered context sorts that its underlying rules already permit and
by returning a general WP continuation.

The initialized wrapper executes an extra leading Skip to obtain its receipt
and an extra trailing Skip to resolve the recovered original parent. The
physical wrapper subsequently executes the original parent's Endlft. The write
callback executes its leading Skip, obtains `advance_credits d`, and explicitly
uses exactly two credits: one to open the delayed payload and one for SC write.
Unused generated credits are discarded in the affine model. Public setup uses
the existing initializer's one lifetime-allocation credit (k=0); there is no new
client credit/token premise. The final read uses the existing SC read rule.

## MODEL-relative internal premises

- Generic composition takes the existing unique tctx element, the ordinary
  lifetime-aliveness derivations for κ and the parent's full type lifetime,
  lctx/invariant contexts, and an arbitrary Frame.
- General child composition takes a separately proved WP callback. The no-write
  and concrete byte-write cases discharge it; it is not a published native law.
- The write case requires a valid new byte, an actual nonempty vector and its
  ordinary model cloc/pointer correspondence. The old byte and tail are
  initialized by the unique resource. Old != new is only the change witness.
- Initialized resource lemmas consume the existing `model_reference`, including
  its actual retained continuation. Public adequacy derives it internally from
  byte ranges and lookups in arbitrary raw h. There is no assumed final/current
  equation and no assumed restoration added by this lane.

## Acceptance and limits

The final replay compiles five modules and inventories all 24 candidate constants.
Six are closed; the remaining entries use only the inherited functional
extensionality and, in seven entries, Eqdep principles. No custom axiom is
introduced. Final receipt, sources/objects, declaration inventory and all
development trial receipts are recorded in results.json and development.json. Every attempted
script was frozen before compilation and retained in its private trial proofs
folder. Failed compilation is development evidence, not a semantic negative.
The checked changing-write theorem is the counterexample evidence.

The controller authenticates the ready Coq 8.20/VerusBelt115a618d baseline,
523 sources, 284 compiled modules, tools and pinned GMP manifest, plus the exact
heap, semantic bridge and typed input receipts and artifact inventories. It
uses retained snapshots and copied-source guards and never modifies dependency
objects. All outputs are private. New declarations include no Program definitions
or obligations; the controller rejects those constructs so no generated
obligation can escape its full declaration/Print Assumptions inventory.
Recursive coqchk checks the dependency closure and rejects unsafe summaries.

The remaining parent integration link is the actual final interval/frame
observer callback using initialized_nested_payload in the delivered exact-Outer
setup. It does not require an additional payload-recovery assumption. This lane
also supplies a direct public read adequacy instance without that observer.

Actual Rust loan/provenance, reservation/activation/retag, MIR phase simulation,
constructor/native pointer correspondence and original Verus C/F import remain
unproved. Native correspondences remain 0; all 44 operations in the six minimal
instances, three phases and 101 effects remain open. Full frozen HMAC remains
1/1 failing. No Rust source entry, full HMAC theorem, production fact,
publication or overall goal completion is claimed.

Final acceptance: trial 053, receipt `/tmp/model-nested-borrow-context-trial-053/receipt.json`, SHA256 `c144c56e1d38e5310219a90da8fb03ad0cca4036ab123eee3beb92fd773d8019`.

Replay from the repository root with `python3 -B -O source/tools/research/model_nested_borrow_context/check.py --freeze source/tools/research/model_nested_borrow_context/final-freeze.json --output <new-private-directory>`.

Load the checked objects with `-Q /tmp/model-nested-borrow-context-trial-053/proofs ModelNestedBorrowContext`, plus the pinned dependency mappings in check.py. The handoff theorem is `ModelNestedBorrowContext.recovery.initialized_nested_payload`.

All 53 trials are retained: two passed full replay (009 and 053); 51 failed during ordinary proof development. No filesystem concurrency/substitution experiments were run.
