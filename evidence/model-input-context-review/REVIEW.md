# Independent formal mathematics review

**Decision: ACCEPT the exact raw MODEL heap-to-typed-reference prerequisite.**
No mathematical or controller finding requiring a change. This accepts neither
Rust source entry nor the full canonical/HMAC goals.

Reviewed `94b87742`, in prepared isolated checkout
`1f465e1d9fd236135d6aec910c8a1b82bebc7942`: `INTERFACE.md`, `FROZEN.md`,
`REPORT.md`, results/development records, all five candidate modules and
`check.py`. `git diff 94b87742 HEAD -- source/tools/research/model_input_context`
shows only the added `parent-freeze.json`; mathematical/controller bytes are
unchanged. Only this new evidence directory is owned by the review.

## Statement and premise review

`raw_model_context_adequacy` is conditional adequacy for the explicit expression
`Skip;;e`, with `Closed [] e`. Its heap domain is every finite `gmap loc val` h,
every p and every list zs, assuming only `Forall (0<=z<256) zs` and
`zs!!i=Some z -> h!!(p+i)=Some #z` about those inputs. The additional premise is
the universally quantified WP proof for e under the **derived** context, not an
application-supplied permission. The ambient `typePreG` is the ordinary ghost
signature requirement; the read instance supplies `typeΣ`. There is no caller
owner, PointsTo, existing `ty_own`, allocation-base, zero-initialization,
freeability or separate lifetime-guard premise.

The callback receives ordinary llft/proph/uniq/time contexts, empty external
context, singleton local lifetime context, invctx for its fresh tid,
`model_reference zs p tid κ`, exactly
`initial_fragments (h ∖ interval_map p (# <$> zs))`, and £k.
Here `# <$> zs` abbreviates the literal-value list in the sources. Its typeG,
tid and nonempty κ are universally quantified; the implementation instantiates
them with its own fresh allocations. Prophecy/loan indices are existentially
packaged inside `model_reference`, never chosen by the caller.

The global initializer allocates authority and fragments for the same h in one
heap instance. TypeG wraps that existing lrustGS; it does not initialize another
physical namespace. `byte_interior_payload` extracts the exact payload and map
difference frame. The initial state explicitly uses all RSt0, global-lock=false
and empty freeability authority. This is a MODEL convention, not a premise
about original Rust histories. No subobject freeability is derived or used.

`llft_alloc`, `proph_init`, `uniq_init` and `invctx_alloc` follow the existing
soundness initialization. `llftl_begin'` allocates a fresh alive singleton κ;
`fresh_local_alive` derives the required guard from ordinary llctx. No separate
legacy lifetimeG is requested. Bytes' lifetime list is empty, so their outlives
obligation reduces to inclusion in static. Empty cloc paths reduce the metadata
guard to True, discharged by `guards_true`; no metadata authority is assumed.

`model_reference` is a transparent existential definition containing the actual
`ty_own (&uniq{κ} (bytes_ty (length zs)))` at `[FVal #p]`, its location/future
prophecy relation and lifetime-end restoration continuation. The private
`bytes_reference_construct` consumes physical fragments derived internally by
the public theorem, plus ordinary contexts/time receipts. It invokes the
existing `alloc_uniq_body`, whose prophecy/loan introduction allocates fresh
indices. It does not assume an owner or future=current equation. Payload
depths are d=0,g=0 and reference depths d=1,g=0, satisfying the bounds exactly.

The generic theorem has no nonempty premise. For zs=[], p may be absent from h:
the physical payload and metadata are empty, the outside frame is all h, and the
value sort of the future is unit. This is a valid empty MODEL construction;
nonconstant empty futures and an empty first-cell read are not claimed.
Lengths 10/128 are genuine specializations of the generic theorem. The strict
interior instance uses arbitrary nonempty prefix/suffix, arbitrary ranged
middle and a disjoint unrelated finite frame, at `base+length prefix`.
There is no allocation-origin or offset restriction on the generic theorem.

## Constructive representation and operational composition

The vector module constructs the nested-product value from zs by recursion,
proves exact flattening, injectivity and uniqueness, and proves that its current
observation is independent of the assignment. These proofs work for arbitrary
integer lists, with byte range needed only for the byte type's ghost predicate.
`bytes_current_physical` and `bytes_current_ghost` connect that value to exactly
the initialized cells/range. There is no existentially guessed current vector.

The prefix is exactly two beta reductions: Skip's lambda application, then the
enclosing sequencing binder. Closure is needed for the existing `pure_rec`
rule. `wp_bind Skip` confines the reduced mask of
`wp_cumulative_time_receipt2` to the Skip subexpression; the callback resumes at
top. Of its two cumulative tickets, one combines with persistent receipt 0 to
produce persistent receipt 1, and one goes to `alloc_uniq_body`. Initial £(1+k)
splits into £1 consumed by lifetime-context allocation and £k for the callback.
Construction follows the two prefix steps. Ghost allocation/resolution is not
an extra runtime rewrite, and this setup is not alleged to be the native body.

`initialized_reference_read_resolve` opens existing `source_context_access` on
this very reference. The returned payload is unpacked at its original p;
`wp_read_sc` reads the actual first fragment, returning #z from the supplied
head lookup. Tail fragments remain separate. The read returns its fragment,
all cells are recombined, and the accessor close continuation restores the
**same** reference, with the same indices and pointer, and llctx. Its ghost
component then goes to existing `source_loan_release`. Guards are obtained
from llctx and llctx is restored after resolution. No second loan is allocated.

One of £2 exposes the later payload, the other discharges resolution's
reference d*(g+1)=1 later. The read instance therefore initializes £3 internally
(one lifetime credit plus two composition credits), executes the two setup beta
steps and the actual read, and concludes the returned value #z. The WP
composition preserves arbitrary Frame once, llctx, the resolved future=current
observation, the original location/future observation and the lifetime-end
restoration continuation. Persistent observations/time facts may be reused;
physical payloads and frames are not duplicated.

The adequacy conclusions use `(λ w _, w=#z)` and ignore the final heap. The
resource-level frame preservation above is real, but it is not a full writer
frame/final-state observation theorem. The upcoming state-observation lane is
outside this review.

## Controls and complete assumption coverage

The wrong-pointer control contradicts the actual reference's physical singleton
pointer equation. The wrong-current control is a pure representation inequality,
backed by the inverse/uniqueness laws. Payload duplication is refused for a
nonempty full fragment vector; frame duplication is refused when the frame
contains an occupied cell. Neither control falsely forbids duplicating emptiness.

`assignment_at` is a pure dependent function, not a prophecy allocator.
`nonempty_future_not_constant` exhibits two mathematical assignments giving
different head values; `nonempty_future_not_assumed_current` refutes a universal
equation over all assignments. **Those assignments need not both be compatible
with any given prophecy observation.** `REPORT.md` states this qualification
explicitly. There is no stronger compatibility/future-independence claim in the
reviewed statements, report or result record. Only the *current* observation is
proved assignment-independent. Resolution legitimately restricts observations.

All 26 definitions/fixpoints/lemmas/theorems/corollaries occur in the freeze,
generated assumption inventory and replay result, with zero new Program
definitions/obligations. The complete assumption partition is retained by name
in `checks.json`:

| Assumptions | Constants covered |
| --- | --- |
| Closed (11) | all 5 vector constants; `fresh_local_alive`; `strict_interior_byte_lookup`; `assignment_at`; `wrong_current_bytes`; both duplication controls |
| Inherited dependent functional extensionality (12) | the other 6 context constants; composition lemma; 4 adequacy/interior read constants; wrong-pointer control |
| Inherited Eq_rect_eq (3) | `assignment_at_lookup` and both nonempty future controls |

Recursive checking reports only the two allowed inherited axioms, predicative
Set, no rewrite rules, no type-in-type, no unsafe (co)fixpoints and no assumed
positivity. No axiom/admission/unchecked computation is introduced.

## Controller and evidence

Source inspection confirms exact mandatory freeze names, authentication of
loaded JSON bytes against retained snapshots, ready marker and durable-build-2
receipt pins, 523 foundation sources, 284 foundation objects, tool/loadpath and
shared-support identities, and both prerequisite libraries' source/object pins.
The retained GMP manifest hash is **compared to** the existing authenticated
baseline pin, as well as stability guarded; the earlier heap C1 issue does not
remain here. Copied candidate sources stay identity guarded through normal
compilation and recursive checking. Assumption markers and allowed axioms are
checked. All final source/product/input guards precede the proof success flag;
the enclosing receipt records failures. Prover cwd and auxiliary writes are in
private output. This is a review of this controller, not a new review of the
prerequisite libraries or an adversarial filesystem experiment.

The worker trial032 COMPLETE receipt, its retained freeze, five source copies,
six objects and kernel log authenticate against `results.json`. Its normal
coqc/recursive coqchk stages succeeded (kernel 186.286 s); these are **read
receipts**, not executions performed by this reviewer. The retained development
manifest and all 32 historical freeze/receipt pairs authenticate. Trial001 is a
setup failure; the other 29 failures are compilation failures. Successes are
024 and 032. None is impossibility or negative semantics evidence.

`parent-freeze.json` differs from `final-freeze.json` only in hashes for the two
updated parent metadata records. Proofs, controller, contracts, shared support
and baseline bindings remain identical. The parent's active replay and live
checkout were not inspected or polled.

Fresh commands executed by this reviewer, from the review checkout:

```sh
python3 -B -O source/tools/research/model_input_context/check.py \
  --freeze source/tools/research/model_input_context/parent-freeze.json \
  --output /tmp/model-input-context-review-1f465e1d-normal
python3 -B evidence/model-input-context-review/audit.py \
  --replay /tmp/model-input-context-review-1f465e1d-normal \
  --output evidence/model-input-context-review/checks.json
```

Fresh outcomes, exact receipt/log/source/object hashes, elapsed stage times and
complete assumption partition are in `checks.json`; the authenticated replay
receipt retains exact prover argv/loadpaths/cwd. All seven fresh stages passed:
normal coqc compiled all five modules and the assumption inventory, then
recursive coqchk checked all six objects in 216.795 s. All six object hashes
match trial032 exactly; all 26 assumption sections match the advertised groups.
The fresh receipt SHA256 is
`bbec38f8d582dffa453e4a5ecc6daea17336b0ccba917f9b6bcc5f92383ca85c`.
No baseline rebuild/copy/install/write, source-changing experiment,
filesystem concurrency experiment, delegate, push or PR is part of this review.

## Preserved limits

Allocating a fresh MODEL context does not identify an incoming Rust loan,
suspended ancestor, provenance, original future or C/F interpretation, nor
convert a live Rust reference. Explicit setup is not the original native body;
extracted loan0 is local `_4`, not incoming `_1`. Actual shared checked transfer
remains unconfirmed. Native correspondences remain 0; 44 minimal simulations,
3 phases and 101 effects remain open; publication=false. Canonical 6/0 and full
HMAC 1/1 are inherited status, not rerun here. Every-u64/arbitrary-[10] and the
full frozen HMAC prefix **and** untouched suffix over arbitrary [128] and all
valid keys (including lengths 0 and >128) remain required. There is no
application-local mechanical proof/call/snapshot discharge from this review.
No contract, domain, premise or production fact was changed.
