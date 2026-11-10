# Independent formal mathematics review

Scoped acceptance of the stated **MODEL prerequisite** at a0d77659 (lane
c6116d49, 54480a6e, 0ca503ea, a0d77659). No blocking mathematical finding in
that scope. Prepared checkout: a13f0551ebd18cc2a7dd2b642c6b2f04547597c6;
the reviewed lane has no diff from a0d77659. This is not acceptance of an
actual Rust/native correspondence, full native translation, or either original
writer goal.

## Mathematical checks

* `fragment.v:16` defines explicit numeric MODEL premises: positive address,
  nonnegative extent bounded by 2^63−1, base+extent at most 2^64−1. Pointers
  retain their window and cursor; locations are exactly model base+cursor,
  numeric addresses are native base+cursor. The numeric and model bases are
  independently supplied. This convention does not establish Rust provenance,
  allocation existence/liveness, or a physical owner. These are MODEL
  conditions, not new application requires.
* `source_step` has exactly eight typed constructors: metadata, wide wrap and
  unwrap, thinning, thin wrap and unwrap, bounded nonnegative byte Offset,
  and CTFE-false branch. The heap/state is unchanged. Unsupported operations
  have no constructor and translate to `None`. The translator accepts register
  **words**; equal word counts alone do not establish descriptor typing or
  bounds. Simulation requires the typed source-step premise.
* `simulation.v:4,26,40` proves actual imported lrust `head_step`, using BetaS,
  BinOpS/BinOpOffset and CaseS. Inspection of the pinned `lang/lang.v:245,250,256,357`
  confirms those are the existing semantics. Every output word reduces in one
  head step, preserving the entire state (heap, locks and global flag), with
  no event or fork. The reverse clause determines every possible head
  reduction of that represented expression. It is conditional target
  determinacy, not a target-to-Rust execution converse. Wide outputs comprise
  two independent word expressions, not a single atomic two-word program.
* `offset_result_bounds` preserves the block and proves numeric bounds;
  `interior128_offset` quantifies over every window and positive strict interior
  start. `array_dereferenceable` is the half-open array interval. Its one-past
  lemma excludes start+128 even with a containing-allocation suffix. This is
  an interval-domain fact, not a theorem that arbitrary descriptors permit
  physical dereference. The imported OffsetOp is unbounded; the source guards
  supply the bounds. No zero-byte/base-pointer restriction or array-subobject
  freeability premise appears.
* `physical.v:10,20` preserves arbitrary separating R per translated output
  word. Metadata carries no `ty_own` or mapsto authority. The nonempty-payload
  countertheorem establishes exclusivity; copying descriptors cannot duplicate
  the physical payload.
* `physical.v:37` uses the SAME `model_reference zs p tid κ`, reads original p,
  and retains the exact arbitrary Frame, `llctx_interp [κ ⊑ₗ []]`, ordinary
  future-resolution observation, original pointer observation, and lifetime-end
  restoration Back. Its postcondition matches the pinned imported
  `ModelInputContext.composition.initialized_reference_read_resolve`; the proof
  adds only Offset and its sequencing binder. It still requires £2. Here n
  is arbitrary model arithmetic; this composition itself asserts no native
  bounds and never dereferences the computed end.
* `physical.v:72` publicly proves MODEL adequacy from arbitrary raw h and
  arbitrary ranged byte list zs of length 128 with exact cell lookups. The
  accepted initializer internally creates one reference, splitting the exact
  outside heap frame. Prefix, suffix and unrelated cells are unrestricted.
  The displayed program is `Skip;; let end = original+128 in Read ScOrd original`:
  two setup beta steps, Offset, binder, read (five steps); initialization uses
  £1+£2=£3. The result predicate is only `v = #z` and ignores final state. The
  proof deliberately discards the richer resource postcondition. It proves
  neither the actual final input-buffer frame nor a Rust source-body theorem.

## Source boundary and replay authentication

The frozen source binding authenticates the successful extraction receipt and
both native and borrowck JSON payloads. Their source hash matches the local
unchanged `drop.rs` fixture. I inspected the actual structured constructor body,
caller-resolved instance identity, original locals, operand/projection enums,
type/layout/NonNull transparent nonnull field representation, and u8 IS_ZST
CTFE-false target bb2. `reify.py:select` checks the seven exact typed statement
payloads and branch/layout conditions rather than using debug trace strings.
The producer's assignment of fragment meaning to MIR remains untrusted; exact
hashes do not turn it into a Rust semantics theorem. The constructor selector
is a frozen-graph selector, not a general verified MIR frontend.

`reified.v:9` lists eight **independent** edges. It does not connect SliceDesc
to RawWide through the unsupported bb0:5 raw-reference creation. The coverage
retains all 44 operations: eight selected, two bb1 CTFE-unreachable, 34
unsupported. The three empty drop-glue query instances are retained as
unsupported graph observations, not executed destructors. Unsupported source
is not erased into a complete translation. Retag, reborrow, unsize,
reservation/activation, FakeRead, storage, call/return, aggregate ownership,
phases and cleanup/unwind/drop remain unproved.

Normal replay authenticated ready+successful Coq **8.20.0**, VerusBelt
**115a618dd35ff8337112cf2081a7f5391d3973d5**, all **523 source files / 284
objects**, the loadpaths, four tools, three completed dependency libraries,
and GMP manifest
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
The worker receipt is exactly
`e98d875a3062053a87e0030d9ee9ec71536a58a942f5ddf182a1cbd2162f3998`.
The final freeze equals trial018's pretrial freeze. Inspection of
`check.py:155–190` and the pinned shared helpers confirms retained candidate
copies, assumption source, proof objects and authenticated inputs are checked
through recursive coqchk; all final guards precede success. Normal `require`
checks remain active under Python `-O`.

Fresh normal compilation of four modules plus Assumptions and complete recursive
`coqchk -silent -o` passed. All **five proof-object hashes match trial018**;
all compile/assumption/kernel logs and deterministic source/control/freeze
outputs also match byte-for-byte. Kernel checking took 230.04 s in this replay.
`audit.json` records exact argv/cwd for all six commands in each run, private
paths, 871 authenticated inputs per run, source/object/log/receipt hashes,
foundation pins and all 18 development receipts. `assumptions.json` contains
the full **121-declaration** inventory, including Inductives, constructors and
**24 generated eliminators**: **119 closed**, and only
`physical.offset_then_original_read_resolve` and
`physical.interior128_offset_read_adequacy` use existing dependent functional
extensionality. No new axioms, Program definitions/obligations or unchecked
proof constructs. Recursive dependencies report only existing FE and
Eq_rect_eq, with predicative Set, no rewrite rules, type-in-type, unsafe
(co)fixpoints or assumed positivity.

Seven normal replay controls reject in-memory typed-payload changes for retag,
original operand, arithmetic, cleanup, CTFE=true, layout, and NonNull validity.
No filesystem mutation/concurrency/substitution control was performed. All
16 failed development trials retain their receipt, freeze and candidate-source
hashes; none is semantic negative evidence.

## Commands, retained private output, and integration limits

Run from this checkout's repository root:

```sh
git diff a0d77659 HEAD -- source/tools/research/native_mir_lrust_refinement
python3 -B -O source/tools/research/native_mir_lrust_refinement/check.py \
  --freeze source/tools/research/native_mir_lrust_refinement/final-freeze.json \
  --output /tmp/native-mir-lrust-review-a13f0551-replay
python3 -B evidence/native-mir-lrust-review/authenticate.py
```

The output path must be new for compilation; `authenticate.py` reads the
completed output. It writes only compact review JSON in this evidence directory
and imports no artifact-control driver. Worker repo-input hashes are checked
against this checkout, without opening the original worker checkout. Raw
receipt/logs, copied replay sources, .vo/.glob/.aux/.vos/.vok and cache products
remain under `/tmp/native-mir-lrust-review-a13f0551-replay` and the completed
trial directories. Fresh receipt SHA256:
`05f4b712446ca375b0fb84aad370ba7017992e248d323c3cd70093498572327c`.

Action for integration: retain the conditional MODEL label and all unsupported
boundaries. Do not increment native correspondences by eight, promote the pure
adequacy result to a final physical frame, or infer source permission from
borrowck data. An actual source-context/execution/provenance interpretation and
the original C/F observations are still required before native progress can be
claimed. **Native correspondences 0; 44 minimal + 3 phase / 101 effects open;
publication=false; goal_complete=false.** Canonical 6/0 and full frozen HMAC
prefix-AND-untouched-suffix 1/1 are inherited and were not rerun.

No implementation, shared-baseline, parent or sibling edits; no installs,
push/PR, subdelegation, live parent-job inspection, or inspection/validation of
the excluded model_buffer_transform prototype. Only this new evidence directory
is changed.
