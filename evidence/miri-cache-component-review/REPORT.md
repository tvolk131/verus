Recommendation: **accept the bounded cache initialization/add/consistency milestone only**.
No blocking finding in that scope. No new native acceptance, lookup theorem,
runtime permission, original control-flow theorem, or application automation result.
Review base: `bc3317f327f99b2f8b18d1268c8e1a8e0c844445`.
Early typed scope was committed as `bf83044e` before the review proof epochs.

The fresh accepted epoch is
`/tmp/native-miri-cache-component-review-20261110-proof003`.
Its execution receipt SHA256 is
`293bafc3d5ffdcc58817283366331fd36aabd2f7fa264cdf7a57f8ca64c9018b`;
pretrial freeze SHA256 is
`4c1e3bebd9caf8d007412da4bc61bf29c54be408ffcaa28b6544728031573226`;
acceptance SHA256 is
`6e31f83069294d9cfcfb4b59a7268a092b993c5534db55d355904e9d2fcd7da1`.
Final acceptance exited 0 under Python `-B -O`. It checked all 99 frozen input
identities, all 22 products, the complete body inventory and final guards.
Exact verifier argv, environments and terminal exits are in `RECEIPT.json`;
private controller terminal/log identities are in `PRODUCTS.json`.

Fresh outcomes reproduce the delivery:

| Cases | Result | Classification and bound |
| --- | --- | --- |
| Cache cold / warm / assistance off | each 17/0, exit 0 | Entire selected crate; helpers empty; no causal assistance gain |
| Wrong head index / shortened index shift | each 16/1, exit 1 | Exactly `StackCache::add`, postcondition only |
| Bad initialization index | 16/1, exit 1 | Exactly `Stack::new`, postcondition only |
| False preservation property | 16/1, exit 1 | Exactly authored consistency lemma, postcondition only |
| Literal enabled lookup, assistance on/off | frontend failure, exit 1 | Same four unsupported operations; no branch proved |
| Reverse lawfulness / empty reverse loop | 0/1 and 1/1, exit 1 | Support proof failures; **not semantic controls** |

For each semantic control, all other queries succeeded and the failed-function
inventory names exactly the intended target. No frontend, panic, resource,
bounds, precondition, or mixed failure was accepted as a semantic rejection.
The three executable mutations fail exact fidelity at their intended body;
the false-property mutation preserves the executable bodies. Positive candidate
bytes were unchanged in all three own epochs and match worker `cf61f45f` /
`7789ce2f`, integrated `bf1a18d2` / `0e00fe65`, and the parent base.
The earlier worker/integrated interface commits precede the component's addition.

The authenticated upstream source is Rust/Miri commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`, using the supplied immutable pins.
Source inspection and strict lexical fidelity confirm exact executable tokens
for `StackCache::add` (52), `Stack::new` (66), `Item::perm` (18) and
`Permission::from_bits` (58), after removing only the disclosed ghost block and
selecting the original stack-cache cfg. The representations are genuine
`Item(u64)`, `BorTag(NonZero<u64>)`, `Vec<Item>`, two 32-element arrays,
`Option<BorTag>` unknown bottom and `Range<usize>` unique range.

The checked add effect covers arbitrary packed items, arbitrary `usize` indices
and arbitrary initial arrays: the supplied head pair and all 31 shifted pairs
are correct in **both** arrays. The original overlapping `copy_within` calls
remain. Exact initialization establishes the singleton vector, None bottom,
all 32 matching cache pairs, and unique_range 0..1 exactly for decoded Unique,
otherwise 0..0. It does not prove item-tag validity. The preservation lemma
requires full prior pair consistency, a matching added pair and the proved add
effect; it does not prove recovery from invalid short-stack entries.

The 17 successful queries correspond to **7 executable bodies, 1 proof body,
4 spec items and 8 auxiliary items: 20 selected items**, not 17 executable bodies.
The executable inventory includes the three checked Clone derives. Twelve
derived Eq/PartialEq/Ord/PartialOrd/Hash/Debug items are explicitly outside
selection. Original Stack/StackCache Clone+Debug and custom Item Debug are omitted
from the projection. Their native bodies are not proved by this result.
`INVENTORY.json` lists every selected and excluded item and the source adapters.

Manual work consists of one `Item::perm` proof block containing two bit-vector
assertions and one authored quantified consistency-preservation proof function.
There are no inferred helpers, new axioms, assumes, external-body stubs, unsafe
blocks or disabled checking. Standard Vec/vec!, array/slice, overlapping
copy_within, Range, Option, NonZero Clone and arithmetic specifications remain
inherited trust, as do the ordinary verifier/SMT backend and compiler/runtime
environment. In particular, the native copy_within implementation and native
no-unwind are not proved here. The reported 359-model inventory is hashed and
referenced privately; it is not a minimal dependency certificate. Inspecting own
vstd source does not independently prove its correspondence to bundled binaries.

Authentication independently covered the canonical receipt's success and all
eight accepted bundle hashes, the delivered final008 receipt/freeze identities,
all 45 frozen inputs with **12 explicit identical-byte own-path rebindings**,
nine completed helper snapshots, 22 products, and all 45 logged runs across eight
retained delivered epochs. Historical checkout paths were labels and were not
opened during authentication/replay. The delivered acceptance code was inspected
and reused through an explicit SHA-reader mapping; the own controller adds fresh
execution, stronger freezing and independent selected/excluded inventory and
positive/negative status checks. No candidate annotations were changed.

Two own acceptance-controller setup errors are retained separately. In proof001,
the historical-path guard also rejected the own checkout. In proof002, an added
check incorrectly required `no_inference` for negative controls, whose correct
status is `caller_proof_failed` with zero requested/generated helpers. The delivered
gate had passed in proof002 before this added check failed. Both epochs completed
all eleven proof runs with final guards; neither is final accepted evidence.
Exact prior controller bytes are retained and authenticated against their freezes.
Each correction preceded a new freeze and complete eleven-case rerun. Proof003
is the sole accepted own epoch. No proof/frontend/resource failure was relabeled
to hide a controller error. `ATTEMPTS.json` retains all 33 own verifier runs and
the original setup/annotation/cfg/proof classification corrections, including the
delivered final007 selected-count gate error.

Normal sandbox launch failed with bwrap RTM_NEWADDR; commands used the exact
automatically reviewed `require_escalated` mechanism without host/network changes.
An initial instruction-discovery search mistakenly extended to `/home/dev/.t3`;
it was interrupted with exit 130, returned no paths, and no results were consumed.
This was a procedural deviation from the requested own-checkout-only search rule,
not review evidence. No live sibling source, build or process data was imported.
No subdelegation, dependency installation, shared configuration/cache edits,
baseline builder, broad regression run, security/race/substitution experiment,
push, PR or publication occurred. All committed changes are new review evidence.

Full lookup remains open at the literal NonZero constructor-as-function-value,
actual FxBuildHasher, Option::is_some_and and slice swap frontend blockers, plus
iterator lawfulness/decrease and tag/hasher interfaces. No Read/Write,
Concrete/Wildcard, known/unknown-bottom, success/None/error branch is established.
Whole-native preservation across insert, retain, truncate, clear/refill,
permission updates and other transitions remains open, as does the write caller
precondition. These are internal proof needs, not extra application restrictions.

Native correspondences remain **0**. Prior canonical arbitrary-u64 / arbitrary
initialized exclusive `[u8;10]` acceptance remains **6/0**; only its receipt and
bundle were authenticated here. Exact bitcoin_hashes 0.14 HMAC pad-XOR on arbitrary
initialized `[u8;128]`, all lawful key lengths including 0 and >128, and both XOR
prefix and unchanged suffix remains **1/1 UNPROVED**. Required actual automation
and fresh combined independent goal acceptance remain outstanding.
**44 minimal + 3 phase / 101 writer effects OPEN; publication=false;
goal_complete=false.**
