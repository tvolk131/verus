# Independent mathematical review of finite model initialization

**Accept the exact MODEL proof prerequisite. One P2 controller identity check
needs correction (C1 below). No mathematical defect found in the frozen proofs.**
This review starts at `9f91fd58` and reviews implementation `3d799b38`.
The model directory and shared assumption parser are unchanged from that head.
The original interface is byte-identical to `da164976`; first development
sources/controller were committed at `f9cc7d74`. Only this new evidence directory
is changed. No subagents, installs, baseline builds/writes, source substitutions,
filesystem concurrency experiments, application edits, pushes or PRs were used.

## C1 — P2: the standalone controller does not authenticate a consumed environment manifest

[`check.py:65`](../../source/tools/research/native_heap_initialization/check.py#L65)
adds `rocq-toolchain/gmp-private/manifest.json` to `ArtifactGuard`, but the
subsequent trusted-identity comparisons (lines 74–93) never compare its captured
digest with `trusted['toolchain']['GMP_packages']['sha256']`. That digest already
exists in the authenticated `foundation-baseline.json`.

The pinned `run-rocq` wrapper reads this manifest's `env_prefixes` on every
invocation and applies them to the prover environment, including library search
paths. Thus the standalone controller checks that this input stays unchanged
after its initial snapshot, without establishing that the snapshot has the
trusted identity. A stable preexisting replacement is not rejected by an
expected-digest comparison. This conclusion follows from reading the controller,
wrapper and pins; **no replacement or concurrency experiment was executed**.

Requested correction: compare the retained manifest hash to its existing
authenticated pin before any prover command. This is a bookkeeping correction;
it requires no proof changes, new mathematical premise or user permission.
The six retained refusal cases cover source/controller identity and omitted
frozen inputs; they do not cover this manifest.

This does **not** invalidate the actual proof receipts reviewed here. Independently
checking the current manifest and the recorded input snapshot of the fresh,
parent and both worker replays gives the pinned digest
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`
in every case. Acceptance of those concrete model proofs is supported; blanket
acceptance of the standalone controller's input authentication is withheld.

## Mathematical conclusions and premise inventory

- **Exact initialization (`initialization.v:8–38`).** `h` is an arbitrary finite
  `gmap loc val`, not an existentially chosen heap or a supplied owner. Fmap
  preserves exactly its domain and values, adding `RSt 0`. Map induction allocates
  the empty map once and extends the same `ι` at each fresh key, accumulating
  exactly one full linear fragment per key. No preexisting points-to resource,
  Rust validity, allocation base, byte restriction, loan or permission is required.
  The quantified `invGS`/`na_logicG` are Iris infrastructure. One global model map
  handle is used throughout; the inherited implementation internally packs three
  backing ghost names (cell map/read bank/write bank), rather than allocating a
  new world for each cell.
- **Operational adequacy (`initialization.v:40–79`).** For each given `e,h,φ,k`,
  the WP obligation is universal over `lrustGS Σ`. Its resources are supplied
  internally by the same freshly allocated heap instance, not assumed of a
  caller. The proof preserves the inherited strong adequacy construction:
  `heap_ctx σ ∗ time_interp stepcnt`, time initialization at step 1, initial
  time credits plus the requested `£k`, non-atomic pool and atomic counter at
  zero. It neither changes the operational semantics nor sets `MaybeStuck`.
  `lrustΣ` provides a concrete infrastructure instance. The conclusion is
  safety/partial correctness over erased executions, not termination, and its
  value postcondition ignores the final physical state. The WP obligation is
  essential. The example really executes `Read ScOrd #l`, using the fragment
  selected by `h !! l = Some v`, and returns this exact `v`; it does not choose
  a convenient initial value or substitute a pure pointer computation.
- **Lock/freeability (`initialization.v:37–61`).** Empty freeability authority
  satisfies `heap_freeable_rel` because that relation constrains only its present
  entries. This legitimate empty initialization grants no deallocation fragment.
  The authoritative non-atomic map includes the inherited bank/count invariants;
  fresh insertion establishes `RSt 0`. Global lock `false` is accompanied by the
  required pool/counter resources. These choices describe this model convention,
  not restrictions proved or imposed on actual valid Rust input histories.
- **Exact partitions (`partition.v:8–105`, `controls.v:55–64`).** Union requires
  disjoint domains, so left-biased union cannot silently overwrite an overlapping
  value. Submap inclusion includes value agreement. Extraction gives precisely
  `h ∖ a`, with both directions of the separating equivalence available for
  recombination. For intervals, lookup and its converse witness rule exclude
  hidden/extra keys. Locations are `(block,Z)` and shifting preserves the block;
  distinct natural indices cannot alias through machine overflow. Freshness and
  append arithmetic are proved, including the prefix-plus-middle suffix offset.
  Prefix and suffix are unrestricted lists and may both be nonempty; no base
  offset equality or freeability assumption appears. Arbitrary disjoint unrelated
  maps are retained exactly.
- **Interior bytes (`partition.v:74–138`).** The only additional pure premises
  are exact per-index lookups, and, for bytes, `Forall (0 ≤ z < 256)`. Every list
  length is allowed, hence 10 and 128, with arbitrary permitted byte values.
  The cloc equivalence uses one empty cell chain per location and `FVal` of the
  same supplied values. This is the existing physical payload shape, not a
  proof of `ty_own`, prophecy or source validity. Interior adequacy receives both
  the payload and exact remainder from global initialization. These are resource
  extraction/recombination laws; they do not prove that an arbitrary executing
  writer leaves its suffix unchanged. Such a program still needs its WP proof.

## The five countertheorems are genuine resource incompatibilities

| Checked theorem | Premises and decisive contradiction |
| --- | --- |
| `wrong_initial_value` | Exact authority, one wrong fragment, present `v`, and `v ≠ w`; authoritative-map agreement forces `v = w`. No second fragment is assumed. |
| `wrong_initial_location` | Exact authority and one fragment at a key whose lookup is `None`; agreement requires the key to be present. |
| `wrong_initial_frame` | Exact authority, a proposed frame containing `w`, and actual value `v ≠ w`; selects that single frame fragment and uses agreement. It does not assume a second correct fragment. |
| `overlapping_frame_refused` | Both resource maps contain the same key; selects their two full fragments in the same heap namespace and uses cell-map exclusivity. Equality of their values does not rescue overlap. |
| `duplicate_same_namespace` | Two full fragments at the same key and `ι`; `cmap_locs_ne` forces the key to differ from itself. This deliberately rejects duplication regardless of values. |

The pure premises are consistent: present-key singleton maps and distinct model
integer values witness the wrong-value/frame cases; an empty map witnesses the
absent-key case; singleton maps sharing a key witness overlap. Correct authority
and matching fragments are constructible together by `finite_map_initialize`.
The contradiction is the intended incompatible resource combination, not a
contradiction already hidden in pure premises. Neither wrong-value theorem is
merely a duplicate-permission test. These are proved propositions ending in
`Qed`, not failed scripts, universal program-error claims, or Rust negatives.

## Independent checking and trust

The fresh command was:

```sh
python3 -B source/tools/research/native_heap_initialization/check.py \
  --freeze source/tools/research/native_heap_initialization/final-freeze.json \
  --output /tmp/model-heap-initialization-review-9f91fd58-normal
```

All three normal `coqc -q` compilations, the assumption inspection and recursive
`coqchk -silent -o` passed (kernel: 65.75 seconds). No admission, new axiom,
unsafe positivity/fixpoint mode, type-in-type, or rewrite rule was introduced.
All four object hashes match both worker runs and the independent parent run.
[receipt.json](receipt.json) binds exact commands in the raw receipt, logs,
object hashes and all completed historical receipts; it avoids repeating their
large input manifests. The foundation's 523 sources, 284 objects, loadpaths,
readiness, successful durable-build-2 receipt and executable identities were
checked by the frozen replay; C1's missing manifest comparison was independently
supplied by this review's authentication.

[DECLARATIONS.txt](DECLARATIONS.txt) is fresh `Print Module` output from the
compiled objects, including expanded types and section parameters, independently
matched against every source declaration and the assumption inventory: four
definitions (including the structural `Fixpoint`) and 25 theorems; zero new
Program obligations. `Parameter` in this module display denotes opaque theorem
signatures, not new source axioms. [ASSUMPTIONS.txt](ASSUMPTIONS.txt) gives all
29 individual closed outputs. [KERNEL.txt](KERNEL.txt) is the complete recursive
summary: only inherited dependent functional extensionality and `Eq_rect_eq`
occur in dependencies. Closure of individual new constants and this larger
dependency summary are distinct checks.

`audit.py` authenticates the finished worker and parent receipts and retained
products/logs without visiting their source checkouts. The first successful
worker run used the retained v2 controller; its sole frozen input difference
from the final replay is `check.py` (the later private working-directory change).
Proof inputs and all four proof objects agree. The six provenance refusals and
15 failed development trials were authenticated, not rerun. Their recorded
sources, freezes, receipts and logs remain retained. No failed trial is semantic
evidence or an impossibility result; provenance refusals are separate from the
five kernel-checked countertheorems.

Trust includes the supplied Rocq 8.20.0/OCaml 4.14.2 runtime and ordinary host
libraries, pinned official VerusBelt `115a618dd35ff8337112cf2081a7f5391d3973d5`,
Iris/guarding/model definitions, and the two inherited dependency axioms. Hashes
authenticate artifacts; they do not establish a Rust interpretation.

## Limits that remain mandatory

No actual incoming Rust configuration, typing/context derivation, provenance,
original C/F observation, ancestor loan/protector or native execution is mapped
to this heap. Allocating a model world cannot convert a live Rust reference or
recover its existing loan. No different model program, fresh namespace or chosen
initial bytes may replace those missing source semantics. The original source
ownership/context interpretation remains unproved; actual native correspondences
remain zero, with all 44 minimal simulations, three phases and 101 effects open.

The every-u64/arbitrary-[10] canonical target and full frozen HMAC prefix+suffix
over arbitrary [128] and all valid key lengths (including 0 and >128) remain
unchanged. HMAC 1/1 and native-drop 0/1 are inherited and were not rerun. Shared
observer causal reuse across the full properties is unconfirmed. No native
fact, publication, new trusted contract, new input restriction or full-goal
completion is supported by this acceptance.
