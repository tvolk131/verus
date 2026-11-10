# Review of the concrete initialized ordinary-memory MODEL writer

Accepted within the initialized ordinary-memory MODEL scope. No blocking
findings. Fresh normal coqc, the individual assumptions/proof-term inventory,
and full recursive coqchk passed with no exclusions or unsafe modes. All input
and output guards passed through final review success. All six fresh .v/.vo
pairs are byte-identical to the authenticated worker.

The reviewed target is the four candidate modules at completed commit d7156dec;
this checkout has byte-identical sources, checker, final freeze, report and
results. The interface also matches early typed freeze d50fe08e. The worker
receipt is authenticated as
`5733f94896797c6b3b7b432500ae0839711843e979e8c05f683516d349909ed8`.
The scope was committed before the review trial as 092397f5. Historical checkout
paths in the worker receipt were mapped to this prepared checkout by relative
source path; no historical/live parent or sibling checkout was opened.

## The theorem and its runtime

`controls.v:8` specializes `writer.ordinary_writer_observed` to `typeΣ`.
Its only five premises are pad/key byte ranges, pad/key initialized lookups,
and disjointness of the two accessed maps. There is no supplied ownership,
credit, reader, guard, transformation WP, or final-output premise. The
conclusion unfolds to `adequate NotStuck` from `initialized_state h` with the
frozen `ordinary_post`. The budget is internal to the adequacy proof.

`interface.v:9–17` fixes pair setup followed by the original mutable parent /
nested-child protocol, its child Endlft, original-parent resolution and parent
Endlft. The callback actually calls `wp_shared_xor_na_min` for
`min(length pad,length key)` with byte_xor (`writer.v:64–68`); it does not assume
a physical transformation. The pinned `xor_na.v` proof binds Read Na1Ord on
both inputs and Write Na1Ord on pad. The pinned notation definitions confirm
these orders. Comparing the complete SC/Na loop definition text after replacing
exactly two SC read annotations and one SC write annotation gives equality.
This is a syntactic runtime check, with no concurrent SC/Na equivalence claim.

## Resources and original lifetime recovery

`writer.v:101–114` uses the initialized Outer, chooses its resource-valued
observer, and calls `raw_pair_reader_na_context_wp Outer`. The callback's
`SameOuter` equality is substituted before `ordinary_session`. The pinned setup
constructs `TypeG Σ Outer ...`, and its pair resources partition the same raw
map into pad, key and `h ∖ (byte_map p pad ∪ byte_map q key)`.

The pinned adapter's `sc_session_to_na` consumes the old SC R/G through its
Close, obtains Lkey, and opens the prepared Na session. It does not return the
old SC linear R. The existing prepared Cap avoids another preparation credit.
Within the Na mutation, `writer.v:54–70` explicitly supplies
`Rin = ReaderNa ∗ Cap ∗ R ∗ £loop_cost`, `Rout = R`, and
`Frame = G ∗ Close ∗ Outside ∗ £keylen`. Thus G, original Na Close, outside
fragments and all whole-key observation credits stay outside child mutation.
The persistent reader/capability alone cannot replace linear R.

The pinned mutable `transform.v` applies `byte_child_transform` and
`initialized_nested_payload_run`. The child update retains its original future
while updating current bytes. `shorter_run_resources` preserves the blocked
original parent and recovers `(new current, original parent future)` after
ending the child. `initialized_nested_run` resolves that recovered original
parent; `initialized_nested_payload_run` ends its original lifetime and uses
the retained Back obligation and prophecy equality to recover the actual new
physical payload. It does not manufacture a replacement parent/future.
The child's L is guarded, split and restored once; it then returns to the
original parent protocol. The returned key R joins the retained G through the
same Na Close exactly once (`writer.v:71–72`). Whole-key observation threads
that restored Lkey once per cell and returns it each time. No duplicated L or
lost original future was found.

## Observer bookkeeping and actual final heap

`writer.v:15–27` explicitly pins heapGS and invGS to Outer while universally
quantifying NA/AC bookkeeping instances and the actual final sigma. This is
needed because the adequacy wrapper can elaborate auxiliary bookkeeping
instances differently. It does not equate those instances or assume that they
are equal. The adapter destructs the supplied `heap_ctx` into the actual
`non_atomic_map`, freeability authority, freeability relation, and bookkeeping.
It retains the first three with the same hF and changes only the bookkeeping
component by constructing `heap_ctx (sigma.1,true)` using Outer's NA/AC.

The baseline `heap_ctx` definition (`lang/heap.v:67`) makes the last component
True when the Boolean is true. Its freeability relation (`:34`) depends only
on `sigma.1`, so the retained relation suffices unchanged. Neither the physical
map nor its authority is replaced. This is a proved resource elimination for
observation, not an execution step and not a WP for a changed state.

`writer.v:73–85` accepts the final Heap only after mutation/recovery, observes
the whole original key with the reserved credits and restored lifetime, then
derives pad and outside observations by agreement with that same authority.
All constituents of `ordinary_post` depend on `sigma.1` (or only on input
lists); hence the pure conclusion is about the original sigma. No final sigma,
final-state equation, or final observation is assumed. The adapter is closed
under the global context. The post ignores the returned value, so this review
does not add a separate return-value guarantee.

## Post, domain and sufficient credits

`interface.v:20–29` observes actual pad values as `xor_result pad key` and
records its exact XOR prefix, unchanged suffix, unchanged length and byte
range. The pinned definition is `xor_prefix pad key ++ drop min pad`, with
`xor_prefix` applying Z.lxor to zipped pairs. The whole original key, including
bytes beyond the loop prefix, retains its values with existential `RSt r` at
each address; it does not require a particular reader count. The outside post
preserves values at every original outside address. It does not assert a
complete final-state equality, preservation of the bookkeeping Boolean, or
absence of extra cells outside the original map.

The general theorem quantifies over all finite lists without a length bound.
`controls.v:16–67` covers empty key at q=p, nonnegative-gap same-block interiors,
128-byte pad with arbitrary key length (including zero and >128), and empty
pad. Its post corollaries state the precise min-prefix/suffix facts. Empty-key
result is the original pad and its sufficient budget is three. No byte/block
separation stronger than accessed-interval disjointness is introduced.

`vectors.ordinary_budget_split` proves
`3 + 3*min + keylen = 2 + (1 + shared_xor_na_cost min 0 + keylen)`.
The two setup credits pay lifetime allocation and key preparation; the opening
credit pays child payload access; the two-phase Na reader costs three credits
per iteration at g=0; whole-key observation reserves one per original
key byte. The prepared-cap adapter costs no further preparation credit. These
are sufficient internal logical credits, without source-budget minimality or
termination claims.

## Validation and reproduction

Run from this prepared checkout with the exact pinned completed artifacts:

```sh
python3 -B evidence/ordinary-full-writer-model-review/replay.py \
  --output /tmp/ordinary-full-writer-model-review-REPLAY-UNUSED
```

The output directory must be unused and outside the repository. The committed
review-freeze.json precedes the fresh trial and pins the replay plus all
worker inputs, products/logs and target report/results. replay.py authenticates
the worker and repository-history identities before executing the unchanged
pinned checker. It maps historical checkout labels without reading those paths.
The underlying checker authenticates readiness AND the successful baseline
durable receipt, all 523 baseline sources/284 objects, dependency receipts and
source/object inventories, tools, libraries and GMP manifest
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
The baseline remains read-only. Receipt hashes identify this particular run;
replay timings and filesystem metadata can differ. Source/object and individual
assumption/inventory identities are the reproducible comparison targets.

Raw logs, generated sources and objects stay private under
`/tmp/ordinary-full-writer-model-review-001/`. The successful review receipt SHA is
`af3925ad8dec59f7a8c4d4e4ae0f4f652bcd81720388c36250cf37c51afe0bb0`.
The fresh checker receipt SHA is
`018c4a038d67e1aaae9b344c1f7202cf00de691acaa3b9b3bfc987db073ac7a8`.
results.json records all six source/object hashes, individual inventories,
assumption map hashes, proof-term section hashes and log hashes.

Completeness was checked by named identities, not counts alone: the independent
review parser strips nested comments and enumerates the declaration names/kinds
in all six sources, matches the exact set to every local kernel constant, and
checks for unexpected local inductives. Each source declaration has its own
marked Print Assumptions section. The generated sources explicitly print every
helper's assumptions in the verified source declaration order. Both helper
sources and objects match the worker byte for byte; their 100 named constants
appear individually in the recursive check. All 125 records include constructor
and obligation inventories (empty). Existing Instance Outer is the sole local
registration, in writer's observer section; it adds no new instance constant.

Eighteen source declarations and all 100 helpers are closed. Seven source
declarations inherit only dependent functional extensionality and Eq_rect_eq;
there are no new axioms, primitives or trusted summaries. The full kernel
summary also reports no type-in-type, unsafe fixpoints or assumed positivity.
Printed terms can be depth limited; complete validation is by recursive coqchk.

## Scope limits

This is public fresh-world initialized MODEL adequacy. Generic source-context
preservation is a different caller MODEL library. This theorem does not prove
entry from an arbitrary Rust caller. No native theorem, provenance, retag,
source simulation, C/F, unrestricted SC/Na concurrency equivalence, total
termination or publication claim is made. Native0; 44+3 minimal/phase and 101
writer effects remain open. Actual full HMAC1/1 and canonical6/0 are inherited,
not rerun. Publication=false and the overall goal remains incomplete.

Only this new evidence directory was edited. No subdelegation, installation,
shared build, shared/controller/production edit, root model_full_writer proof
or live output read, Rust/HMAC build, push or PR was performed.
