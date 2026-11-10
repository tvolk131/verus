# Independent fixed-reader and shared-loop review

Review targets are exactly `ModelPairReader16` at `30c9c17c` and
`ModelSharedXorLoop12` at `88a56443`. This directory contains review evidence
only. No implementation, baseline, compiler, vstd, shared helper, Rust body or
application domain is changed. The early independent typed milestone is
`79594b1f` (all six source modules normally compiled, exact worker object hashes;
kernel checks were still pending at that commit).

## Decision and authentication

Accepted for the stated scope, with the nonblocking generated-helper inventory
qualification below. Fresh receipt identities are recorded in `results.json`.
Acceptance is restricted to the two MODEL libraries. Neither is a concrete
combined public writer or a native/termination theorem.

The completed worker receipts are authenticated byte for byte:

| Library | Completed artifact | Receipt SHA256 |
| --- | --- | --- |
| Pair reader | `/tmp/model-pair-reader-trial-012` | `534374d1dc414dc7bc98368b4f1acb892796238e6ae03e10286a0421d1e80bde` |
| Shared loop | `/tmp/model-shared-xor-loop-trial011` | `41b447e1daee6caa6a24e80589f353ca71d07f5293893ff452b40f5ebaf2d881` |

Each target directory is compared with its exact git revision. All receipt
inputs, frozen sources, generated sources, objects and individual library
assumptions are authenticated against those completed receipts. Historical
worktree names in receipts are labels: repository paths are resolved in this
prepared checkout, never in a worker/parent/sibling checkout. No root live
`parent-pair-reader-30c9c17c` or `parent-shared-xor-loop-88a56443` replay is read.

The original controllers and freezes are used unchanged:
`model_pair_reader/final-freeze.json` and
`model_shared_xor_loop/trial011-freeze.json`, respectively. Their exact
filenames/hashes agree with their own results/EVIDENCE and reproduction text.
The independent review freeze predates both fresh trials. Normal `coqc`, all
individual assumptions, full recursive `coqchk -o` and final artifact guards
are required. `-silent` on the pair checker suppresses progress only; there
are no exclusions, admissions, quick builds or unsafe flags.

Both readiness **and** successful durable baseline receipt are required, with
foundation `115a618dd35ff8337112cf2081a7f5391d3973d5`, build
`verus-belt-build-115a618d`, pinned `run-rocq`, 523 source files, 284 objects,
and GMP manifest SHA256
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
The loop freeze additionally authenticates the external Coq/OCaml/GMP inventory;
the independent guard covers the union for both trials. No inherited Coq,
OCaml or dynamic-library overrides were present for the pair trial. The loop
controller explicitly removes them. No installation/shared build is performed.

## Pair reader: constructive handoff

`ModelPairInput.setup.shared_key_view key q tid κ` is the original tctx predicate.
`ModelSharedByteRead.shared_read.shared_key_view n κ q x d g tid` is raw
`ty_own (&shr{κ} (bytes_ty n))`. These names must remain qualified: they are
different predicates with different indices. `handoff.pair_view_raw_exact`
converts the tctx to an existential synchronized raw `d/d` plus `⧖d`; it proves
no bound on that old existential.

`fixed.model_key_share_fixed` instead retains the constructed unique reference's
payload indices. It calls actual `resolve_uniq_body` with **0/0**, combines the
returned real lifetime guard with the base guard, and builds raw shared
**d=1/g=0**. Its persistent byte information follows from the original initialized
key range. It consumes the unique body and executes the actual existing
`Share` expression (`Skip;; Skip`) through ordinary WP steps. It introduces no
replacement instruction, postulated typed rule, assumed bound or exclusive
owner recovered from an alias. `fixed.fixed_view_pair_context` separately
packages the exact original pair tctx at synchronized **1/1**, preserving the
persistent raw 1/0 view.

`setup.raw_pair_reader_context_wp` allocates one TypeG over its given Outer,
initializes both nonempty lifetimes, uses the exact raw pair payload and keeps
the original `Skip;; Skip;; Share;; e`. Its two initial Skip steps supply the
cumulative tickets and persistent time receipt used by reference construction.
`resolve_uniq_body` generates its timed credits from that constructed unique
body. The internal budget **2+k** splits into lifetime-context allocation (1),
fixed reader preparation (1) and exactly **k** for the callback.
`raw_pair_reader_observed` supplies this budget inside initialized observation
adequacy. It keeps caller-selected resource-valued Φ in the **same Outer**;
no owner, guard, points-to, depth or budget premise is added to the raw-input
application interface.

The callback receives the pad reference/Lpad, exact outside fragments, exact
pair tctx, fixed Cap, persistent Reader, spatial Timeless R and G, G's lifetime
guard, and the R/G closure to ordinary Lkey. `pair_reader_first_observed`
executes an actual SC key read from initialized raw input, keeps G outside it,
then closes the returned R with G. Persistent shared aliases are duplicated,
not exclusive ownership or linear R/G.

`whole_key.whole_key_heap_observe` requires ordinary Lkey, Cap, arbitrary Frame,
the actual `heap_ctx σ`, and explicitly reserved `(g+1)*keylen` credits. It
returns that **same σ**, Lkey, Cap and Frame. Every original `key !! i = Some z`
implies `σ.1 !! (q+i) = Some (RSt r,#z)` for some unrestricted r, with byte
range. Neither zero counters nor an exclusive key vector is assumed. This is
a fancy-update observation; it does not execute a program Read. At fixed g=0
the sufficient reserved budget is keylen. Join R/G before applying it.

Controls include exact indexed values, persistent Cap aliases, empty key at
the same address as the pad, and disjoint interiors in one allocation
(`p=base+1`, `q=base+1+padlen+gap`, gap≥0). The explicit empty observer has no
cell access and needs no credits. These controls establish the stated MODEL
cases; they do not assert Rust provenance or turn overlapping nonempty regions
into valid pair inputs.

## Shared loop: unchanged operation and exact result

The expression is the imported **`ModelBufferTransform.transform.zip_transform`**
with actual existing **`ModelByteXor.byte_xor.byte_xor`**. No executable loop is
defined here. `shared_xor_result`, `shared_xor_cost` and `shared_xor_contract`
are proof data. The internal op-parametric helper is discharged with checked
`wp_byte_xor`; no op-law premise survives in the concrete XOR interface.

The invariant fixes original key q/x while advancing start. The actual shared
read at `q+start` returns `byte_list n x !! start = Some z` and its byte range.
That exact lookup identifies the head of `drop start`; recursive pointer
correspondence is `(q+start)+1=q+(S start)`. For arbitrary start/count satisfying
`count≤length pad` and `start+count≤n`, the postcondition returns the physical
pad with `transform_prefix count Z.lxor pad (drop start (byte_list n x))`, same
spatial R, Cap and Frame. `shared_xor_verified` also proves exact length,
unchanged suffix after count and byte range.

Every concrete theorem explicitly requires the **separate persistent Cap even
when count=0**. Reader is persistent; R is spatial and passed once through
each read and recursive call, then returned. Frame is retained outside the
reader (whose internal Frame is True), and returned. No exclusive key vector
is supplied or extracted. Preparation stays outside this loop law. Each step
uses **g+1** shared-read credits and **1** store credit; `shared_xor_cost_step`
splits **count*(g+2)** accordingly. This is a sufficient-budget statement,
without a minimality claim.

`wp_shared_xor_min` proves count=min(padlen,n), exact pointwise XOR prefix and
original suffix. `wp_shared_xor_128` constrains only pad length and initialized
byte range. It covers arbitrary initialized pads and contents for every finite
key length, including 0 and >128. It is not a finite fixture argument.

## Inventory qualification and remaining limits

There are 16 pair source declarations and 12 loop source declarations. No new
library instances, constructors or obligations occur; the pair's contextual
`Existing Instance Outer` is the only existing instance registration. The
pair controller also generates **32** closed `Goal True` marker theorems in
`Assumptions`; these are not included in its advertised source count 16. The
loop records its **48** closed marker helpers explicitly. `inventory-results.json`
enumerates all **108** compiled declarations, including those 80 helpers, and
independently queries every helper's assumptions. This is an evidence-inventory
qualification for root, not an unsound source theorem or missing recursive
check: the pair controller already includes its full Assumptions module in
`coqchk`. New query files contain no declarations or proof rules.

Individual source assumptions are either closed or inherited dependent
functional extensionality, exactly matching the workers. The recursive closure
also uses inherited `Eq_rect_eq`; kernel summaries reject unsafe fixpoints,
type-in-type, assumed positivity and rewrite rules. No new axiom, primitive,
trusted summary, unsafe mode or application premise is introduced.

Root still has to compose pad-loan mutation, reader setup, unchanged shared
loop, original-parent recovery and same-final-heap pad/key/frame observation.
This review accepts no concrete combined public writer. Fresh initialized MODEL
resources do not interpret real Rust input, provenance or ancestor-loan history,
and establish no original C/F or native Zip correspondence, termination,
crypto/security theorem or production inference fact.

Frozen Rust source/domain hashes are checked for preservation only. Canonical
6/0 and full HMAC 1/1 remain inherited and are not rerun. Native correspondences
remain **0**, with **44+3** simulations and **101** writer effects open;
publication=false and the overall Infer goal remains incomplete.

All raw outputs/objects remain private in the paths recorded by `results.json`;
only compact reports, hashes, freezes and reproduction controllers are committed.
No setup/development failure is treated as semantic evidence. This session had
an exploratory receipt-field `KeyError` before the trials and an initial git
identity configuration failure before the milestone commit; neither ran a
negative proof or altered an implementation. The actual fresh trial outcomes
and any failed trial classification are retained by the controllers.
