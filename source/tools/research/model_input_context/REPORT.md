# Raw MODEL heap to typed input context

Implemented a supporting model construction, not Rust source entry or a native
correspondence. All changes are confined to this directory. No production hook,
policy, cache, schema, dependency, shared toolchain, push or PR change was made.

`INTERFACE.md` is the root contract. `FROZEN.md` was committed before trials at
4bdda9c5; it records the statement/resource/step inventory and the two subsequent
expression/mask clarifications before their trials. Implementation checkpoint:
5931d601. Final validation identities are in `results.json` and `final-freeze.json`.

## Checked interfaces

`vector.bytes_of_list` constructively maps every list zs to
`~~(bytes_sort(length zs))`. Its flattening is exactly zs; `byte_list_injective`
and `bytes_of_list_unique` prove uniqueness. `bytes_observation_independent`
proves the current byte observation does not depend on the prophecy assignment.
All five vector constants are closed under the global context.

`context.raw_model_context_adequacy` accepts arbitrary finite h, exact p, arbitrary
zs, pure Forall byte range and per-index raw-map lookup facts. Its continuation
e must be closed because this model encodes sequencing by lambda application.
A universally quantified WP callback is the ordinary proof obligation for e.
It receives the DERIVED context, not a new application permission premise:

- Fresh typeG wrapping the SAME lrustGS/heap allocated by the existing initializer;
  ordinary lifetime, prophecy, uniqueness, time and invariant contexts.
- A fresh nonempty alive model lifetime kappa, empty external and singleton local
  lifetime context `[kappa <=local []]`. No separate client lifetime guard.
- `model_reference zs p tid kappa`, a transparent existential package containing
  actual `ty_own (&uniq{kappa} (bytes_ty(length zs)))` at `[FVal #p]`, with
  current `bytes_of_list zs`, payload indices d=0,g=0, reference d=1,g=0,
  freshly allocated prophetic indices/index, the ordinary location/future
  prophecy observation, and the existing lifetime-end restoration continuation.
- Exact separating outside frame `initial_fragments(h minus interval_map p #zs)`
  and k retained later credits. The callback resumes at mask top.

The theorem derives physical fragments from h internally via
`initialized_heap_adequacy` and `byte_interior_payload`. It then initializes
llft/proph/uniq contexts, wraps the existing heap in TypeG, allocates invctx and a
fresh lifetime, and calls the existing `alloc_uniq_body`. The private resource
constructor is `bytes_reference_construct`. There is no caller ty_own, owner,
PointsTo, freeability, allocation-base or zero-byte premise. Bytes have empty
outlives lists; inclusion in their static lifetime is proved. Repeated empty
cloc paths have trivial guarded metadata, proved using the existing empty-cloc
law and `guards_true`; no metadata authority is assumed.

The public expression is explicitly **Skip;;e**, not an unchanged Rust body.
It initializes with **£(1+k)**: £1 for llft_alloc and £k for the callback. The
Skip subexpression supplies cumulative ticket 2 using the existing operational
time rule; one ticket becomes persistent receipt 1 from receipt 0, and one goes
into alloc_uniq_body. There are exactly two prefix beta steps: Skip itself and
the sequencing binder. Construction happens after those steps. Ghost updates
are not alleged runtime steps. The standard time initialization/state machinery
is inherited unchanged from the checked heap initializer.

Zero length is permitted by the generic theorem, including p absent from h.
The payload is empty, metadata is empty, and the allocated prophecy remains the
ordinary model prophecy (its value sort is unit). No nonconstant observation is
claimed for the zero-length unit sort. Every length, including 10 and 128, and
any strict interior location with arbitrary exact surrounding frame is covered.

## Actual operational composition

`composition.initialized_reference_read_resolve` uses the EXISTING
`source_context_access` to open the constructed reference, reads its actual first
initialized cell at EXACT p with wp_read_sc, recombines all cells and restores the
SAME ty_own through the supplied close continuation, then consumes that restored
reference with EXISTING `source_loan_release`. Guards come from llctx. Its WP
postcondition retains the exact frame, ordinary local context, future=current
prophecy observation, original prophecy relation and lifetime-end restoration
continuation. There is no second loan and no fabricated subobject freeability.
The lemma spends £2: one credit to expose the later payload and one for the
resolution's d*(g+1)=1 later. The read is an actual model heap operation.

`instances.raw_model_read_adequacy` discharges the public callback, yielding
adequacy of **Skip;;Read ScOrd #p** from pure raw-map/range/head facts, with initial
£3 accounted internally (1 lifetime + 2 composition). It executes the two setup
beta steps and one read step; resolution is a checked ghost update using the
explicit credit, not another hidden model instruction. The postcondition is the
actual read value #z. `raw_model_read10` and `raw_model_read128` expose the original
sizes without restricting any bytes or location offsets.
`strict_interior_model_read` instantiates a nonempty prefix and suffix, arbitrary
byte middle and unrelated disjoint finite frame; the pointer is base+prefix length.
The resource-level composition retains the exact frame even though ordinary
adequacy concludes a pure value proposition.

## Countertheorems and assumptions

Ordinary proved controls refute a wrong typed physical pointer, wrong current
bytes, duplication of a nonempty full payload, and duplication of an occupied
outside frame. `nonempty_future_not_constant` constructs two mathematical
assignments with different future values for every nonempty byte list.
`nonempty_future_not_assumed_current` refutes a universal future=current equation.
These statements do not assert that both assignments satisfy a particular
prophecy observation: resolution restricts observations in the usual model.
`assignment_at` is only a pure counterexample function, not a prophecy allocator.
Failed scripts are development failures only and are never semantic negatives.

Every definition, fixpoint, lemma and corollary is individually inventoried by
Print Assumptions; there are no Program obligations or implicit helper constants
introduced by new Program definitions. Pure vector laws are closed. The three
assignment/future control lemmas inherit Eq_rect_eq through simpl_eq. The typing
construction/composition/adequacy statements inherit dependent functional
extensionality from existing type representations. Remaining closed constants
and exact per-constant assumptions are enumerated in results.json. Recursive
checking allows only the existing FE and Eq_rect_eq dependency axioms; no added
axiom, admission, unsafe recursion/positivity, type-in-type or rewrite rules.

## Replay and limits

Run from this checkout, choosing a fresh output directory:

```sh
python3 -B -O source/tools/research/model_input_context/check.py \
  --freeze source/tools/research/model_input_context/final-freeze.json \
  --output /tmp/fresh-model-input-context
```

Final optimized trial032 successfully compiled all five modules plus the
assumption inventory and recursively checked all six objects. All 26 constants
have complete assumption sections; zero Program obligations. The recursive
kernel run took 186.3 seconds. Receipt and object/log hashes are in results.json;
final-freeze.json is byte-identical to that successful run's pretrial freeze.
The earlier successful kernel milestone was trial024 (12 constants). Thirty
failed development trials retain their source copies, freezes, logs and receipts
in the private paths indexed by development.json; none is semantic evidence.
No filesystem race or substitution tests were run.

The compact controller authenticates the ready marker AND successful durable
baseline receipt, all 523 source/284 object pins, tools/loadpaths, shared support
identities and both immutable prerequisite libraries. In particular it compares
the retained GMP manifest SHA to the authenticated existing baseline GMP pin,
as well as stability-guarding it. All required freeze names are checked exactly;
loaded JSON bytes match retained snapshots and pretrial hashes. Copies of all
proof sources are identity-guarded through compilation and recursive kernel
checking. All final source/product/input guards run before the proof success
flag. Prover cwd and generated auxiliary files are private to each output.
No filesystem concurrency or substitution experiment was performed.

AllRSt0/global-lock=false/empty-freeability describes this new MODEL heap, not
every original Rust history. A fresh ghost context does not identify an incoming
runtime loan, suspended ancestors/provenance or original source future. The
extracted loan0 is the local _4 reborrow, not incoming _1; ghost MutRef unpacking
supplies no physical pointer. Actual Rust source typing/context/primitive/phase
simulation and Verus C/F import remain UNPROVED. No unrelated replacement
program is used as a native theorem. Canonical 6/0, full frozen HMAC 1/1, native 0,
44 minimal simulations plus 3 phases /101 effects open; publication=false.
Original every-u64/arbitrary-[10] and arbitrary-[128]/all-valid-key (including
0 and >128) prefix-AND-untouched-suffix goals are unchanged. This prerequisite
does not complete either goal. Parent integration and NEW independent review
are required; this lane did not review or modify either prerequisite library.
