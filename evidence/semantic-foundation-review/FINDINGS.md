# Independent semantic foundation review

Review target: `1a9d4f22`, accepted source `8935534e`, handoff `05581f4b`.
Recommendation: **accept this exact frozen, model-only research integration**
on the independent validation below. No semantic proof defect or undisclosed
axiom was found in v12. Do not treat the controllers as general automatic
acceptance certificates until the findings below are addressed. None authorizes
native publication. Early assumption-gate repro: commit `52cb3bc5`.

## Prioritized findings and objections

**P2 — the automatic extra-assumption gate misses single-line axioms.**
`source/tools/research/native_semantic_bridge/check.py:93-101` collects only lines
matching `^(\S+)\s*:\s*$`, and never validates the axiom/unsafe summary emitted
by `coqchk -o`. `assumption_probe.v` imports the existing standard-library
`classic`, adds no axiom declaration or unchecked proof, and prints its actual
assumptions. Output is `classic : forall P : Prop, P \/ ~ P`. Normal compilation
and recursive kernel checking succeed, while the controller returns
`success=true`, `model_proofs_kernel_checked=true`, `logical_axioms=[]`.
The kernel summary explicitly lists `Coq.Logic.Classical_Prop.classic`, outside
the controller allowlist. Publication and all native-bridge flags remain false.

Reproduce from repository root (fresh private path required):

```sh
python3 -B -O evidence/semantic-foundation-review/reproduce.py \
  --output /tmp/semantic-foundation-review-extra-assumption
```

The probe deliberately supplies a new review freeze; it does not bypass or alter
the parent freeze. That also demonstrates that a controller receipt does not
enforce the advertised four-module/18+6 inventory: the controller accepts any
nonempty distinct module list. Existing frozen v12 sources must be reviewed
and their complete assumption inventory validated separately. This is a future
acceptance-gate defect, not evidence of a hidden axiom in those v12 sources.
Before relying on this controller as an automatic acceptance gate, require
complete named-constant coverage, robust assumption parsing and explicit full
kernel-summary policy checks. No implementation change is made in this lane.

**P2 — source eligibility does not completely bind the claimed borrowck operand
chain.** `source/tools/research/native_semantic_bridge/source_map.py:32` delegates
eligibility to `native_heap_lowering/lower.py`; that validates loan metadata and
the optimized native trace, but does not compare the borrowck reborrow/unsize/
call statements to that chain. Independently changing the borrowck unsize source
from local 4 to local 1, or its constructor call argument from local 3 to local 1,
still yields a successful 44-operation map. `source_map.py:90-93` checks only
the extraction receipt's `success` flag, without authenticating the supplied
JSON outputs against the delivered extraction evidence. A stable digest measured
at audit entry is not a provenance connection to an earlier extraction.

The original delivered JSON has the correct chain, which I inspected directly;
the original receipt and both JSON identities match the handed-off evidence.
All operations remain `unproved_rejected`, so this is a source-completeness audit
gap, not a false native proof. Before making completeness claims from arbitrary
replays, bind the actual extraction outputs and check the borrowck chain, not
just its separate metadata fields. Repro:

```sh
python3 -B -O evidence/semantic-foundation-review/boundary_probes.py \
  --output /tmp/semantic-foundation-review-boundary-probes
```

**P3 — readiness checks trust self-described stage labels and permit an empty
copied-source inventory.** `check.py:35-47` accepts a private synthetic receipt
containing `success=true`, the three required labels with `returncode=0`, and
`copied_sources={}`. The same boundary repro reaches and successfully runs the
real prover-version command. A deliberately empty module list then stops it
before proof compilation; the final status is failure and no proof status is
acquired. The test changes only private readiness/receipt files and links the
real read-only wrapper/manifest. It never alters or rebuilds the shared baseline.

The actual authorized baseline is healthy and independently authenticated below.
This is a provenance gap for arbitrary ready files, not evidence that the current
baseline is wrong. Bind readiness to the expected final receipt identity and
complete source inventory before using these setup gates as unattended trust
checks. Existing three refusal controls cover simple missing/failed states,
not this self-described-success case.

## Independent validation

All raw output is private at `/tmp/semantic-foundation-review-d8ef3a29`.
`validation.json` binds receipt/log hashes; `assumption-inventory.json` records
each named theorem and each generated obligation separately. These are measured
checks, not conclusions inferred from a process exit code alone.

- Ran the unchanged `check.py` under Python `-B -O`, with the authorized real
  ready marker and `parent-replay-freeze.json`, in a fresh private output directory.
  Four ordinary `coqc -q` compilations and recursive `coqchk -silent -o` passed;
  kernel checking took 336.0 seconds. No unchecked flags or proof shortcuts.
- Matched all four compiled source hashes and all four fresh `.vo` hashes to the
  original successful receipt and delivered proof objects. They are byte-identical.
  Selected source/controller hashes match both `8935534e` and `05581f4b`.
- Independently enumerated exactly 18 named theorems (3 generic, 8 byte, 3 raw,
  4 controls) and six `byte_ty_obligation_1` through `_6` constants. A normally
  compiled `Fail Check ...byte_ty_obligation_7` confirms no seventh. Printed
  assumptions for all 24, plus `byte_ty`, with explicit per-constant markers.
  Eleven named theorems are closed; seven byte-specialized lemmas use only
  `FunctionalExtensionality.functional_extensionality_dep`. All six obligations,
  `byte_ty`, and `byte_list_length` are closed. “Closed” concerns global axioms;
  it does not discharge the explicit semantic premises of these statements.
- Reviewed the entire recursive module summary independently of the defective
  parser. Exactly `Coq.Logic.FunctionalExtensionality.functional_extensionality_dep`
  and `Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq` occur. Set is predicative; rewrite
  rules are disallowed; type-in-type, unsafe (co)fixpoints and assumed positivity
  lists are all empty. Summary bytes/hash equal the original disclosed summary.
- Verified the authorized baseline-ready and durable-build-2 receipt hashes and
  successful final receipt. Checked all 523 copied-source hashes against the
  baseline build and original clean dependency checkout at
  `115a618dd35ff8337112cf2081a7f5391d3973d5`. Trial guards cover 284 baseline
  objects, sources, tools, wrapper, manifests, ready marker and receipt; no
  guarded input changed. No baseline copy/build/install occurred in this review.
- Revalidated all five original successful receipt hashes and success flags,
  plus all seven retained failed-trial receipts/stderr hashes and failure states.
  No proof-development failure is classified as a Rust counterexample.
- Replayed mapping, eight original source eligibility controls and three original
  readiness controls independently. All passed with publication false; the map
  contains six instances, 44 unproved-rejected operations and three additional
  phase/exception obligations. It exactly matches committed `source-obligations.json`
  after expanding the latter's integer-indexed instance dictionary.
  These replay supplied extraction outputs; no Rust extraction/compiler regression
  or native-drop/HMAC verifier trial was rerun here.
- Recovered all 41 archived scripts with `git show 05581f4b:<path>`, verified
  every `HISTORY.json` hash and the 2,733-line total. The historical v12 freeze
  mismatches only `foundation-pin.json`. Its repository/commit/file identities
  are unchanged; the differences are readiness/purpose metadata. The parent
  replay freeze matches the reconciled tree. This cleanup loses no evidence.

The independent assumption inventory is reproducible after the normal replay:

```sh
python3 -B -O source/tools/research/native_semantic_bridge/check.py \
  --ready /tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json \
  --freeze source/tools/research/native_semantic_bridge/parent-replay-freeze.json \
  --output /tmp/semantic-foundation-review-fresh-proofs
python3 -B -O evidence/semantic-foundation-review/inventory.py \
  --proofs /tmp/semantic-foundation-review-fresh-proofs \
  --output /tmp/semantic-foundation-review-fresh-inventory
```

## Exact semantic and application boundary

`candidates/loan-v12/borrow_handoff.v:11-31` opens the dependency's defined
`uniq_body`, not an abstract source predicate. Its resource is a prophetic
indexed borrow of the model heap payload plus a time ticket. The input's
physical value forces `p = l.1`. Its restoration wand returns the exact same
`(l,x,ξi,di,gi,idx)`, depth/guard indices, physical reference and lifetime guard;
it requires the same later payload and ticket. The original loan is consumed
while open and is not simultaneously retained alongside an independent borrow.
The new accessor is intentionally no-write: unlike the underlying more general
`uniq_body_acc`, its public wand does not accept a replacement current value.

`source_context_access` at line 46 obtains the guard from `llctx_interp L`,
`elctx_interp E` and the explicit `lctx_lft_alive E L κ` judgment. It returns
the local context on closure; the external context is persistent by the actual
dependency definition. There is no extra client `G`, PointsTo, freeable or owned
allocation premise. It still requires `typeG`, logical lifetime/unique contexts,
the alive judgment and, crucially, the semantic `ty_own` of the original reference.
Those are legitimate model-typing premises, not an established interpretation
of the original Rust input. Renaming them application preconditions would change
the task and is not authorized.

`byte_array.v` instantiates nested products of range-restricted mathematical
integer cells: size 128, list length 128, physical `FVal (LitInt z)` cells and
`Forall (0 ≤ z < 256)`. No zero-content premise or allocation-base equality
appears. Any model `cloc`/contents satisfying that ownership interpretation are
covered, including subobjects. Its fancy heap predicate is the dependency's
actual heap-resource interpretation, including possible cell indirection;
the new theorem does not itself establish a Rust byte layout, provenance relation
or executable raw-read/write rule. The later modality and ticket are real
restrictions; this is not an immediate unrestricted heap capability.

`raw_view.v:10-13,25-28` returns ONE original `&uniq` and a `ptr_ty` observation
of the original location or its mathematical offset. It does not return a
second exclusive reference or a memory permission for the offset pointer.
The dependency's `pptr.v:13-27` defines `ptr_ty` with ghost interpretation True
and a Copy instance. `typed_source_loan_release` consumes the model loan using
`resolve`; future=current is a prophecy observation under the model's guarded
resolution semantics, not a pure unconditional equation or a Rust destructor
execution theorem. `typed_instr` also expands to ordinary lifetime, prophecy,
time, invariant, type-context and WP premises.

`model_limits.v` proves actual operational controls: arbitrary model pointer
values have raw-pointer type; an empty heap cannot take the stated raw-read
head step; and Offset permits every mathematical integer displacement.
`empty_heap_offset_adequate` really invokes `lrust_adequacy`, but for this
empty-heap Offset expression only. No closed/source-typed handoff program,
native execution, in-allocation offset or actual Rust reference adequacy follows.
The report/interface/comparison correctly disclose these limits. Reading
“source-context” or “subobject” as a Rust embedding would overstate the statements.

## Precise next missing native judgment

Establish a checked input-interpretation judgment for the **actual original
Rust `&mut [u8;128]` at the extracted entry**, relating its physical pointer,
initialized byte subobject, provenance and original loan to the dependency's
`ty_own (&uniq{κ} byte_array128_ty)` for the same current bytes and C/F observation.
Derive it from the original Rust input/typing semantics for arbitrary initialized
bytes and valid containing-allocation subobjects, without a new application
permission, base-equality restriction, supplied raw authority or assumed bridge.
Another conditional ghost-heap accessor does not establish this judgment.

Carry that relation through the actual reborrow/unsize constructor operand and
all native operations: RawPtr, metadata/thinning, NonNull validity/transmute,
finite-width in-allocation/isize Offset including one-past, move/copy/retag,
aggregate/call/return/drop and source-phase/exception behavior. Then establish
original-array exit adequacy and a checked Verus C/F import identifying the
restored model future with `final(original Rust array)`. Identity hashes,
layout sizes, the 115/0 model trace and model-only type soundness cannot supply
any of these correspondences.

**Zero actual Rust native correspondences are proved. All 44 minimal native
simulations and all 101 full-writer effects remain unproved; publication=false.**
The inherited native-drop 0/1 and full frozen HMAC 1/1 outcomes were not rerun.
RefinedRust e39cd5f1 remains inspection-only, with no build/import in this review.
Full acceptance still requires the canonical 6/0 theorem for every u64 and
arbitrary initialized [10], with no application-local mechanical proofs/calls/
snapshots, AND the same checked assistance for the full independently frozen
HMAC XOR prefix plus unchanged suffix on arbitrary initialized [128]. All valid
key lengths, including 0 and >128, dangling empty keys, and disjoint key/pad
regions within one allocation remain required. No narrowing or assumed bridge.

Review edits/commits are confined to this new evidence directory. No subagents,
push/PR, production/research implementation edits, native facts or publication
hooks. Operational deviation: an initial overbroad `find .. -name AGENTS.md`
was interrupted before output; it traversed parent directory entries, but no
sibling file contents were opened or used. Subsequent discovery stayed within
this checkout and explicit ancestor instruction paths.
