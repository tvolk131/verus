# Concrete ordinary-memory MODEL writer

`controls.raw_ordinary_full_writer_observed` proves the committed typed target
`interface.ordinary_writer_claim` from byte ranges, initialized cell lookups,
and disjointness of the accessed intervals. There is no external ownership,
reader, credit, guard, physical-WP, or final-state premise. The general theorem
is `writer.ordinary_writer_observed`; the concrete public instance uses `typeΣ`.

The frozen expression runs pair setup followed by the existing mutable
original-parent/nested-child protocol around `zip_transform_na byte_xor` for
`min(padlen,keylen)` iterations. The MODEL post observes the actual changed pad,
its exact XOR prefix and untouched suffix, the entire unchanged original key
with arbitrary `RSt r`, and the exact pair outside frame. All finite lengths
are supported. Corollaries cover 128-byte pads without a key-length bound,
empty key with `q=p`, empty pad, and same-block disjoint interiors.

## Resource composition

The same `Outer` is carried through `raw_pair_reader_na_context_wp` and the
constructed `TypeG`. The prepared-cap adapter closes the old SC session and
opens the checked Na session without extra preparation credits. The concrete
physical callback invokes `wp_shared_xor_na_min`, with **ReaderNa, Cap, R, and
loop credits explicitly in Rin**. Its Rout is the returned R. G, Close, the
outside fragments, and all key-observation credits remain in the mutation
frame. `initialized_nested_transform` changes the child and recovers/releases
the original pad parent. Then the original R/G close yields the key context.

The resource-valued post is an observer closure, built from the recovered pad,
the key context/capability, the outside fragments, and reserved credits. It
accepts the actual final `heap_ctx sigma`; it does not predict a final state.
To avoid coupling auxiliary bookkeeping class instances in adequacy wrappers,
`heap_ctx_for_observation` retains the supplied heap map/freeability authority
and discards atomic bookkeeping. It reconstructs a context over `(sigma.1,true)`
with **the identical physical heap map**. `whole_key_heap_observe`,
`interval_observe`, and `fragments_observe` establish observations of that map,
hence of the original sigma. This is a proved resource elimination, not a
runtime step, assumed state equation, supplied final observation, or new
primitive. The closure is universal over the bookkeeping class instances.

Sufficient internal credits are setup `2+k`, opening `1`, Na loop
`3*min(padlen,keylen)` at guard depth zero, and whole-key observation `keylen`.
`ordinary_budget_split` connects these to `3+3*min(padlen,keylen)+keylen`.
No minimality claim is made; the empty-key total is three.

## Representation and validation boundary

The early committed `candidates/interface.v` fixes runtime, post, budget, and
claim before trials. `semantic-freeze.json` pins that source unchanged. No root
vectors or writer source/object is imported. The local helpers `bytes_cast`,
`byte_list_cast`, `final_bytes`, `final_bytes_exact`, and `final_bytes_range`
transport the checked `xor_result_length` equality to the original pad's type.
Their declarations and assumptions are included in the complete inventory.

`check.py` freezes each trial; authenticates named completed dependencies and
transitive source/object/receipt pins; requires both baseline readiness and its
successful durable receipt; checks all 523 foundation sources and 284 objects;
and guards source/tool/GMP identities through final success. Toolchain library
contents also match the authenticated completed ordinary-reader receipt. All
compilation uses normal `coqc` in the private trial directory. The checker
inventories every local declaration and existing-instance registration,
rejects untracked constructor/obligation declarations, prints assumptions and
proof terms, and runs recursive `coqchk` with no exclusions or unsafe flags.
Raw logs and objects stay private. See `results.json`, `development.json`, and
`final-freeze.json` for accepted evidence and replay pins.

## Limits

MODEL only. No SC/Na concurrent equivalence, native simulation, Rust execution,
provenance, retag, C/F, total termination, or publication claim. Native
correspondences remain 0; minimal simulations 44 plus 3 phase obligations and
101 writer effects remain open. HMAC 1/1 is inherited, not rerun. Publication is
false and the overall goal remains incomplete. No subdelegation, shared build,
shared helper edit, root live output inspection, push, or PR was performed.

## Accepted result

Private trial `/tmp/model-ordinary-full-writer-trial-012` passed normal
compilation of all four modules, the complete 25-declaration assumptions and
proof-term inventory, and the full recursive kernel check (125 local
constants including 100 closed inventory helpers). There are no new
constructors or program obligations. The inherited logical principles are
dependent functional extensionality and `Eq_rect_eq`; no new axioms are added.
The successful durable receipt SHA-256 is
`5733f94896797c6b3b7b432500ae0839711843e979e8c05f683516d349909ed8`.
Trials 001–010 are retained proof-development failures, not semantic negatives.
Trial 011 also passed; trial 012 validates the final checker snapshot after
removing two trailing spaces. The Coq proof sources are identical in both.

Replay from this checkout, with the pinned completed artifacts present:

```sh
python3 -B source/tools/research/model_ordinary_full_writer/check.py \
  --freeze source/tools/research/model_ordinary_full_writer/final-freeze.json \
  --output /tmp/model-ordinary-full-writer-replay-UNUSED
```

The output directory must be unused. Logs, objects, generated inventory sources,
and the durable receipt remain in that private directory.
