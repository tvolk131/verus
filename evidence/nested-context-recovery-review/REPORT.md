# Independent formal mathematics review of d6dc448d

Review scope: `source/tools/research/model_nested_borrow_context/` at
`d6dc448d`, reviewed from isolated HEAD
`3ce5025c35ae3cab8983c5c793d007bbc5f6769d`. All files in that lane match the
reviewed commit. No proof-source changes are proposed.

## Verdict

Accept the exact MODEL laws described below. No blocking mathematical defect
was found. Fresh ordinary compilation, the complete assumption inventory,
recursive kernel checking, and final source/product/input guards all passed.

## Mathematical conditions checked

Paths below are relative to `source/tools/research/model_nested_borrow_context/`.

1. **Existing contextual reborrow, not replacement ownership.**
   `candidates/protocol.v:76-120` starts with an arbitrary existing unique
   context element. Lines 87-101 allocate the shorter local lifetime and invoke
   the existing operational `tctx_reborrow_uniq`. Its output contains the child
   and the blocked original parent at the same path. The original resource is
   consumed by that rule, and only the blocked continuation is held while the
   child is active. Lines 102-119 resolve the child, end its lifetime, and call
   `unblock_tctx_cons_unblock` to recover the original-lifetime context element.
   This is actual resource recovery through the existing rule.

   The equation is `vπ m' π = (vπ (uniq_bor_current m) π,
   uniq_bor_future m π)`, not a claim that original future already equals
   current. Reborrowing keeps the original cloc in the inherited rule; both
   returned contexts use the same evaluable path. Returned borrow indices and
   depths may change. Neither this review nor the theorem requires structural
   equality of all metadata.

2. **Arbitrary context and ancestor continuation.**
   `candidates/protocol.v:80-84,107,115-119` frames and returns an arbitrary
   `iProp` exactly. The existing parent need not be a top-level allocation;
   `Frame` may hold an ancestor's blocked context or restoration continuation
   and any disjoint typing context. The ancestor resources are preserved, not
   regenerated. This is a frame-preservation/composition law, not a new theorem
   that automatically ends every ancestor lifetime.

3. **Ordinary lifetime guard and internal premises.**
   `candidates/protocol.v:15-27,77-80,110-112` derives child inclusion/aliveness
   from the ordinary local/external contexts and the parent's aliveness.
   Parent type-lifetime aliveness supplies the unblocking rule's premise.
   There is no added client lifetime token or permission. The generic
   unique context element, lifetime derivations, global invariant contexts,
   and the callback in `shorter_run` are explicitly MODEL-internal premises.
   They are not new application `requires`.

4. **Proved child callback and actual changed-byte case.**
   `candidates/protocol.v:121-184` executes the callback with only the child
   resource, retaining the blocked parent outside it. The callback must prove
   its WP, preserve the child's future observation, and identify final current;
   lines 164-184 then transfer that current through original-parent unblocking.
   `candidates/writes.v:31-82` opens the existing unique body, obtains the actual
   head points-to, calls `wp_write_sc` at line 65, and closes the body with
   `new::tail`. The construction at line 75 keeps the original child prophecy
   identifier, loan index, depths, and cloc. Future preservation uses the
   inherited `uniq_prvar_app_irrel`, not equality of old and new bytes.
   `candidates/writes.v:88-124` composes that write and derives a contradiction
   from unchanged current when `old <> new`. The arbitrary tail is preserved.
   The valid-new-byte and nonempty-vector conditions are explicit; old-byte
   validity is obtained from the existing payload. This refutes the blanket
   unchanged-current law within the model.

5. **Initializer's original prophecy and continuation survive.**
   `candidates/instances.v:37` opens `model_reference` once into its original
   witnesses `ξi, ζi, idx`, ownership, observation, and `Back`. Only ownership
   enters the nested protocol (lines 54-61). After recovery, lines 65-76 apply
   `type_resolve_instr` / `source_loan_release` to the recovered parent. The
   recovered equation transports the result to the original tuple's future.
   Line 78 returns the same witnesses, original observation, and retained
   `Back`. It does not resolve a newly chosen child future and label it the
   original parent's result.

6. **Physical recovery comes from retained Back.**
   `candidates/recovery.v:21-58` ends the original MODEL parent using the local
   lifetime ownership (lines 35-41), then invokes the retained `Back` at line
   42. The physical payload comes from that continuation. Resolution alone
   is never used as an ownership-producing rule.

   `bytes_vars_nil` (lines 9-10) proves the prophecy-variable inventory empty
   for every byte-vector value. Thus the equalizer may be instantiated at
   `⊤, [], 1%Qp` (lines 46-48): the required prophecy-token collection is empty
   and `syn_indep` supplies the dependency condition. Combining the returned
   equalizer observation and the ORIGINAL resolved future in one
   `proph_obs_sat` (lines 49-53) gives a single assignment witnessing
   `vπ x' = vπ (bytes_of_list zs)`. `bytes_observed_injective` then gives actual
   byte-vector equality. Lines 55-58 return the full physical interval
   `p ↦∗ map LitInt zs` and the exact arbitrary `Frame`.

   The original parent here is the fresh MODEL lifetime supplied by the
   accepted initializer. Ending it is not a proof of ending the actual Rust
   incoming `'a`, whose interpretation/correspondence remains open.

7. **Public raw-cell construction and original-pointer observation.**
   `candidates/recovery.v:62-82` uses the pinned
   `ModelInputContext.context.raw_model_context_adequacy` with `k=0`.
   Inspection of that immutable copied dependency confirms it constructs
   `model_reference` internally from raw initialized cells via
   `bytes_reference_construct`; public callers supply only byte ranges,
   lookups, and the head condition. No caller-supplied `ty_own`, restoration
   assumption, or final/current equation has been added.
   The final SC read is at `#p`, after full payload recovery. It reads the head,
   not `p + length zs`. Arbitrary lengths and heaps are supported, with explicit
   10/128 corollaries at lines 85-101. The strict-interior result at lines
   103-116 reads `base + length prefix`, with nonempty prefix/suffix, arbitrary
   disjoint outside heap, and no extra offset/block restriction.
   The public adequacy postcondition observes the returned value; the exact
   interval/frame resource postcondition belongs to the internal recovery law.

## Steps, masks, credits, and trust

`candidates/protocol.v:31-71` is a WP continuation form of the pinned
`type_endlft`. Comparison with that immutable source confirms the same guarded
local-context splitting/restoration, lifetime-ending update, mask changes,
resolution delay `d*(d+1)`, and unblocking delay `1+d0*(d0+1)`. The squared
receipt bounds are discharged by the same step-fupd rules. The generalization
permits distinct context sorts and a WP continuation; it changes no semantics.

The exact public read expression is:

```text
Skip;; (((Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip);; Endlft);; Read ScOrd #p)
```

The outer setup `Skip` belongs to the existing initializer. `Newlft` is a value
whose sequencing step executes; `ReBor` is `Seq Skip Skip`; each `Endlft` is
`Seq Skip (Seq Skip Skip)`. The initialized wrapper's leading `Skip` obtains its
receipt, and its trailing `Skip` performs original-parent resolution. The
payload wrapper executes the original parent's `Endlft` before the physical
read. These are explicit model expressions, not zero-step context inclusions.

The inherited `typed_instr_of_skip` generates `£(advance_credits d)`,
`⧖(S d)`, and two `⧗1` tickets from operational steps, with
`advance_credits d = 10*d*d + 10*d + 10`. The byte writer's leading `Skip`
generates credits, and `candidates/writes.v:48-58,65` explicitly uses two: one
for the payload later and one for the SC store. Excess credits are affine.
Public setup uses the initializer's single lifetime-allocation credit with
`k=0`; this lane adds no client credit premise. Namespace/mask side conditions
are proved at the existing update and access calls; no unsafe checking flag,
semantic primitive, axiom, trusted contract, application source body, or domain
change occurs in these five modules.

`check.py:58-123` authenticates retained JSON/freeze bytes, both ready and
successful durable-build receipts, 523 copied sources, 284 baseline objects,
tools, loadpaths, GMP, and exact prerequisite source/object inventories. It
requires successful durable build/version/kernel stages, not readiness alone.
`check.py:147-176` guards copied proof sources, inventories all 24 declarations,
forbids Program/obligation and incomplete/unchecked constructs, and runs
ordinary compilation plus recursive `coqchk -silent -o`. Lines 177-181 check
copied sources, products, and original inputs before setting proof success.
This review used normal replay only; no mutation or substitution experiments
or artifact-control driver was created.

## Evidence and boundary

The worker's trial-053 receipt authenticates to
`c144c56e1d38e5310219a90da8fb03ad0cca4036ab123eee3beb92fd773d8019`.
Its six copied sources (including generated `Assumptions.v`), six objects,
every recorded log digest, and full assumption map match `results.json`.
All 53 retained trial freezes, copied candidate scripts, and durable receipts
were authenticated against `development.json`. Only 009 and 053 passed;
the other 51 are failed compilations, not semantic negatives. The checked
`shorter_write_changes` theorem supplies the changing-write evidence.

Fresh replay output is private:
`/tmp/nested-context-recovery-review-20261110-001`. The command was:

```sh
python3 -B -O source/tools/research/model_nested_borrow_context/check.py --freeze source/tools/research/model_nested_borrow_context/final-freeze.json --output /tmp/nested-context-recovery-review-20261110-001
```

The fresh durable receipt SHA256 is
`7a5971448f1a43a15318ae4aa99abbc43b9849763494d7958af73bbd6ecc9b01`.
All five ordinary module compilations, assumption compilation, and full
recursive kernel check returned 0. All six objects are byte-identical to
`results.json`; all six copied sources also match. The complete 24-entry
assumption map matches exactly: six closed, eleven FE-only, seven FE/Eqdep.
There are no Program declarations or generated Program obligations. The kernel
summary contains only inherited functional extensionality and Eqdep, with
predicative Set, no rewrite rules, and no type-in-type, unsafe (co)fixpoint, or
assumed-positivity dependencies. After successful replay, the review also
rehashed all 866 input snapshots and all source/object products.

`verification.json` retains the complete assumption map, source/object hashes,
recursive checker command, baseline identities, and receipt/log digests.
No raw replay directory or compiled artifact is placed in this evidence folder.

Acceptance is limited to MODEL contextual composition, the concrete changed-byte
child theorem, and exact NoWrite physical recovery/read adequacy. Full mutable
writer/observer composition is pending and was not reviewed. The parent live
replay, root-30 composition, and full XOR-26 work were not inspected. This
review does not establish actual Rust loan/provenance, incoming-lifetime
ending, MIR reservation/activation/retag, native pointer/constructor
correspondence, phase simulation, or original Verus C/F import.

The unchanged boundary is native correspondences 0; 44 minimal operations,
3 phases, and 101 effects open; canonical 6/0; full frozen HMAC prefix AND
suffix 1/1 still failing/open; publication false; overall goal incomplete.
Those outside-scope goal results were not rerun.

Procedure exception, disclosed during the review: one read-only audit lookup
followed an absolute path in the worker receipt to the frozen
`foundation-baseline.json` in sibling checkout `t3-4112111d`. That exceeded the
no-sibling-inspection instruction. No other content there was inspected and no
file there was changed. Subsequent checks use this isolated checkout and the
named immutable `/tmp` artifacts only. The live parent replay was not accessed.
