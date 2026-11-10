# Independent formal review: conditional MODEL buffer transformation

Verdict: **accept the exact conditional MODEL law only; no actionable findings**.
Reviewed commit: `2b7092b930db4f4a779bd2a71e2eb659734bf928`, in isolated
checkout `39409553e674ebfe065c640454b6818586efa3f1`. All 14 library files present
at the reviewed commit match that commit byte for byte. The companion
`REPORT.md` present in this checkout was also read and hashed separately.

The accepted receipt authenticates to the committed `STATUS.json` hash
`687c9f1ab9a09456cf54cecfb8a440386b48e5a425ad7d7223cc92c7f7762aa7`.
A fresh normal Coq build and recursive kernel check succeeded. Both generated
objects, both proof sources, and all eight closed assumption inventories match
trial007. This is model progress, not completion of the active goal.

## Mathematical statement and proof

Line references in this section are to
`source/tools/research/model_buffer_transform/candidates/transform.v`.

- **Exact domain and postcondition (44–51).** `n : nat` is arbitrary subject to
  `n <= length xs` and `n <= length ys`. Both original lists satisfy
  `Forall byte_value`, where `byte_value z` means `0 <= z < 256` (5).
  There is no fixed pad size, empty/oversized-key exclusion, zero-initialization
  premise, allocation-base premise, XOR premise, or application-level requires
  change. The WP returns *all* of `p ↦∗ zvalues (transform_prefix n f xs ys)`
  and *all* of the unchanged `q ↦∗ zvalues ys`, not merely observations of the
  processed prefix. Other resources can be retained with the WP frame rule.
- **Pure laws (7–11, 22–36).** The recursive list function replaces exactly the
  first `n` positions under the stated bounds; its fallback otherwise is not
  an assertion that any out-of-bounds execution is safe. The two proved laws
  preserve the entire pad length and establish
  `drop n (transform_prefix n f xs ys) = drop n xs`. At each successor, the
  bounds force both lists to be nonempty, so induction descends on both tails.
- **Explicit checked operation obligation (40–42, 47, 69–71).**
  `binary_operation_spec f op` is a persistent, universally quantified WP for
  the actual `App op [a;b]`, conditional on both byte-range facts. It is a
  premise that a client must prove for its operation, not a new axiom or an
  unchecked XOR summary. The two literals come from the actual pad/key reads,
  in the order `a` then `b`; the postcondition identifies the returned literal
  with `f a b`. Persistence permits reuse on every iteration without
  duplicating the local physical resources.
- **Head/tail separation (56–68).** The length bounds exclude absent heads.
  The two `Forall_cons` eliminations retain the head byte facts and tail
  range facts separately. Rewriting `heap_mapsto_vec_cons` yields the pad
  head at `p`, tail at `p +ₗ 1`, key head at `q`, and tail at `q +ₗ 1`.
  Each actual SC read uses its matching head token and returns that token.
  No points-to resource is converted to a persistent hypothesis or copied.
- **Callback frame (69–73).** Only the pure head facts are passed to `Op`.
  `wp_wand` keeps the pad/key heads, both tails, and the credit budget in its
  continuation. The standard WP rule supports this linear frame. Returning
  only a pure equality from the callback loses none of those resources: they
  were never supplied to it. The subsequent store consumes the original
  pad-head resource at exactly the read address, then returns the updated
  token. The key head remains framed throughout.
- **Credits and progress (59, 72–79).** The initial `£(S n)` splits as one
  `Credit` and `£n` in `Credits`. Exactly the former is supplied with the
  pad-head token to the existing `wp_write_sc`; the latter is supplied to
  the recursive call. The loop expression has one explicit pad store per
  completed iteration. Arithmetic reduces `Z.of_nat (S n) - 1` to
  `Z.of_nat n`, and the actual recursive arguments advance both addresses by
  one. The induction hypothesis receives the two original tails, their byte
  facts and decremented bounds. Reassembly returns both full vectors at their
  original bases. There is no resource shortfall hidden behind a pure suffix
  observation.
- **Base case and minimum length (53–55).** At `n=0`, the integer comparison
  selects poison immediately; neither head is fetched and the complete
  original vectors are returned. If the key is empty, the length premise
  forces exactly this case. Taking `n=min(length xs,length ys)` satisfies
  both inequalities, including empty, shorter, equal and oversized keys.
  This is a specialization of the general theorem, not a restriction to a
  new input domain. No separate minimum-length corollary is claimed as one
  of the eight declarations.

The reviewed foundation defines locations as `(block,Z)` and `+ₗ` as offset
addition (`lang.v:9–10,187,245–246`). Vector ownership is an indexed separating
conjunction (`heap.v:48–52`); its cons equivalence preserves the same physical
locations (`heap.v:299–302`). The cell-map exclusivity result
`cmap_locs_ne` (`util/cellmap.v:2663–2684`) excludes duplicate physical ownership
across nonempty overlapping intervals. Nothing requires distinct *blocks*:
disjoint subregions of one allocation are allowed when the separating resource
premise can be supplied. This internal model premise establishes no arbitrary
interpretation of the original Rust source permission.

The SC lifting rules return each read token (`lifting.v:440–451`) and require
the old pad cell plus `£1` for the actual store (`lifting.v:549–560`). The
operational rules read the heap and update that same location
(`lang.v:261–263,276–281`). The callback composition uses the existing WP wand
and frame rules (`iris/program_logic/weakestpre.v:338–341,399–403`). These are
foundation rules, not added semantic primitives.

## Precise limits of the acceptance

The operation's postcondition promises an integer result equal to `f a b` on
return. It promises neither output byte range nor termination. This suffices:
the next iteration reads the untouched original tail, so it needs no range
fact about a previously written result. The result vector is an integer
vector; a byte-valued result requires a further property of the chosen `f`.

This is ordinary partial-correctness WP, not total WP. The structural loop
count decreases after each returning callback, but an arbitrary checked
callback need not return. Likewise, the one-store/one-credit statement concerns
the loop's explicit pad store and supplied budget. It is not a proved global
execution-trace bound on all effects, allocations, forks or stores inside an
arbitrary callback. Its contract contains no general no-effects or no-write
postcondition. Framed pad/key resources are still preserved by the checked WP.

There is **no XOR implementation in this reviewed library**. The separate XOR
worker's directory was not inspected. No actual native Zip/source entry,
provenance, ancestor, phase, loop, C/F, constructor-loan recovery,
cold/warm Infer automation, or full HMAC theorem follows. The recursive
`zip_transform` expression is a model program, not evidence that the unchanged
Rust body implements it. No new axiom, trusted contract, semantic primitive,
unsafe checking mode or application requires clause was introduced here.

The user-supplied broader state is retained without re-certifying it: native
correspondence 0; 44 minimal plus 3 phase / 101 effects open; full frozen HMAC
prefix **AND** suffix 1/1; canonical 6/0; publication false; goal active. This
review does not promote those markers into a native or full-HMAC proof.

## Replay and evidence authentication

Fresh private output:
`/tmp/model-buffer-transform-review-bb24scei/replay/`.

```sh
python3 -B -O source/tools/research/model_buffer_transform/check.py \
  --freeze source/tools/research/model_buffer_transform/checked-loop-freeze.json \
  --output /tmp/model-buffer-transform-review-bb24scei/replay
```

That output path now exists and must not be reused for another replay. Select
a new private path for future runs. All generated `.vo`, `.glob`, `.aux`,
`.vos`, `.vok` and lia/cache products remained in the private output; none is
committed. No source-changing, substitution, concurrency or artifact-control
experiment was run. No baseline was rebuilt or written, dependency installed,
Rust source changed, sibling/live checkout read, subagent started, or push/PR
performed.

The fresh receipt SHA-256 is
`9d04ca14e065d0566e3e01fc705576fbfc18641f894dad2b198c7bce145aa435`.
The two byte-identical accepted/fresh objects are:

| Object | SHA-256 |
| --- | --- |
| `transform.vo` | `f52e837a74813389929f93be16dafef642c2475d1b2c62ae46122ac4ee264d29` |
| `Assumptions.vo` | `e29f373b66c65c4cf1f4a3f6dca390ebd81065c4a58dc1803e8a41762a1aba4c` |

`audit.py` independently authenticates the committed STATUS receipt, all seven
trial receipts, their copied sources and exact retained pretrial freezes;
then both successful replays' sources, objects, raw assumptions, kernel
summaries and exact command arguments. It uses historical repository paths
only as receipt metadata and maps their relative names to this isolated
checkout. It never follows those paths into the parent/live checkout.

The checker and supporting code were read: `check.py`, `infer_eval_support.py`,
`infer_experiment.py`, `proof_checks.py`, and `acceptance_inputs.py`. The ten
mandatory frozen inputs include the interface, candidate, checker, both shared
supports, parser, environment helper, foundation pin/baseline and tool/object
bindings. Parsed readiness, durable receipt, pin, bindings and baseline bytes,
plus the exact parsed freeze bytes, are matched to their retained snapshots
before use (`check.py:23–27,47–80`). Readiness **and** a successful durable build
are required, with successful version/build/kernel stages. All 523 copied
foundation files, 284 objects, tools and `_CoqProject` loadpaths are bound to
the pinned foundation (`check.py:81–96`). GMP manifest SHA-256 is exactly
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`;
Coq is 8.20.0 and VerusBelt is
`115a618dd35ff8337112cf2081a7f5391d3973d5`.

Copied candidate and generated inventory sources are guarded. Compilation uses
normal `coqc -q`; recursive checking uses `coqchk -silent -o` on both complete
modules, with no admission/nonrecursive flag. Product and input guards run
after checking and before the final checked flag is set
(`check.py:113–153`). The shared parser allows the foundation's two standard
axioms generally; this review additionally checked that **all eight particular
declarations** printed exactly `Closed under the global context`. The recursive
kernel context lists functional extensionality and
`Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq` from dependencies, with no unsafe fixpoints,
type-in-type, assumed positivity or rewrite rules. Those two foundation
axioms are not assumptions of the eight reviewed declarations. No new Program
definition or obligation appears in the candidate.

The read-only auditor can be rerun against the retained output:

```sh
python3 -B evidence/model-buffer-transform-review/audit.py \
  --replay /tmp/model-buffer-transform-review-bb24scei/replay/receipt.json \
  --summary evidence/model-buffer-transform-review/audit.json
```

## Retained failed development

All six failures have matching committed receipt hashes, copied sources,
`before_first_trial` freezes, recorded freeze identities and private frozen
byte copies. Each is an ordinary `coqc` exit 1 before assumptions/kernel
checking. They are **not semantic countertheorems** and were not rerun.

| Trial | Failing script phase | Copied-source line |
| --- | --- | --- |
| 001 | Expression-list/vector notation elaboration | 42 |
| 002 | Expression-list/vector notation elaboration | 42 |
| 003 | WP triple introduction / missing introduction structure | 54 |
| 004 | Boolean comparison/branch tactic matching | 60 |
| 005 | Read-pattern matching (`LitV p` versus literal location syntax) | 63 |
| 006 | Final postcondition reassembly / redundant vector-cons rewrite | 79 |

`audit.json` contains compact hashes and outcomes for these records. Raw replay
logs, proof objects and the historical copied sources remain private at their
indexed `/tmp` locations. Acceptance is limited to the exact checked
conditional MODEL law described above; no broader gate is discharged.
