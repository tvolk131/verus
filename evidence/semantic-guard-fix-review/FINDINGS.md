# Independent recheck of the semantic controller fixes

Review base: `e84eed6c79d9c104cefada569de0c99221dc2b51`; fix: `1efbd004`;
prior findings: `ed9fd420:evidence/semantic-foundation-review/FINDINGS.md`.
This is a fresh review of the automatic guards and their positive compatibility.
The mathematical v12 model integration was already accepted. This review makes
no native correctness or publication claim.

## Prioritized finding

**P2 — source controllers still accept an incomplete freeze that omits the
extraction binding, permitting post-freeze replacement of its provenance pin.**
`source_map.py:137-144` and `audit.py:22-27` validate only the supplied entries
of `freeze['files']`; neither requires `source-bindings.json` to be present.
Their artifact guards also omit that binding when the freeze omits it. This
contrasts with `check.py:39-45`, which requires its trust inputs explicitly.

Exact independent repro, from this checkout, using a fresh output path:

```sh
python3 -B -O evidence/semantic-guard-fix-review/recheck.py omission \
  --output /tmp/semantic-guard-fix-review-omission-repro
```

The repro copies the exact controller/support bytes and selected source files
to a private tree; it edits no implementation. It first writes a freeze identical
to `review-fix-freeze.json` except for removing the one source-binding entry.
**After writing that freeze**, it changes the private omitted binding to accept
a synthetic extraction receipt containing only `{"success": true}`. The two
actual output JSON files remain byte-identical to the reviewed extraction.
Both source-map and all 12 source controls then exit 0 with `success=true`.
Neither receipt records the changed binding as a guarded input. With the complete
freeze, the same synthetic input and binding change are correctly refused with
a source-binding freeze mismatch.

This is not an authorized reviewed pretrial rebind: the altered binding is absent
from the pretrial freeze, and no extraction execution supports the synthetic
receipt. A measured current hash cannot replace that earlier provenance check.
Require a complete mandatory source-audit trust-input set, including the binding,
and always snapshot that set. Reject incomplete freezes before using the binding.
The minimum regression should remove the binding entry and require refusal from
both entry points. The same completeness policy should cover imported checker
code and the source fixture rather than relying on a caller to list them.

This finding does **not** invalidate the full delivered freeze, the real reviewed
extraction, the four mathematical objects, or any actual native theorem. The
accepted map still rejects all native operations; publication stays false even
in this adversarial run. Raw repro results are in
`/tmp/semantic-guard-fix-review-e84eed6c/omission/omission-results.json`, also
copied into this evidence directory. The serious result was reported with the
exact repro before broader validation continued.

For this run the successful synthetic source-map receipt hashes to
`0ad815c22ea8cbecc86d6b44ac7190d19210bc7a9d6bf129c0b91a14549329b1`;
the successful synthetic 12-control audit receipt hashes to
`35c10559fd2ae994fb3585fca60e2ec5f7af74aaa966ac99ab5c00ad9d0c2a45`.

## Recommendation

Accept the **exact fully frozen model-only replay** and the fixes to the three
original counterexamples. Require the small completeness fix above before
advertising source-map/audit as unattended provenance gates for caller-supplied
freezes. No remaining false acceptance was demonstrated for the exact delivered
`review-fix-freeze.json`, the pinned baseline or the pinned raw extraction.
There is no finding against the unchanged mathematical proof sources.

## Independent validation

All raw output is under `/tmp/semantic-guard-fix-review-e84eed6c/`.
`validation.json` binds receipts and command logs; `identities.json` records
source/tool identities, all copied-source hashes and the canonical object map.
The scripts in this directory are review-only and reproducible. No parent
assertion of success was used as a substitute for these fresh runs.

Fresh positive receipt SHA-256:
`ced460a84ff22d3fb0830aae6e16193103fa3d1e9a6f9735792d161f8a6e87b2`.
Fresh three-acceptance-control receipt SHA-256:
`cca743f7d9157325b30fdf2484e0e5e10d41400d620ef97f5ace1da621f4849b`.
Packaged validation SHA-256:
`d41b6f1a956e8411490863bd2bb707fff10986be2a8248c53483ca3537b8c19e`.
Its manifest binds 143 private receipt/log/snapshot identities. The final
independent stability check covers 816 baseline/tool/readiness files.

- Fresh `python3 -B -O check.py` against the active review-fix freeze ran ordinary
  `coqc -q` for all four modules, generated/compiled the separately delimited
  inventory, and recursively checked the four modules **and**
  `NativeSemanticBridge.AssumptionInventory` with `coqchk -silent -o`.
  All seven commands exited 0; recursive checking took 268.53 seconds.
- Exactly 18 named statements and six generated obligations are present.
  A separate normally compiled audit prints all 25 constant types and their
  assumptions, including `byte_ty`, and confirms obligation 7 is absent.
  There are exactly seven FE-dependent specialized byte statements; the other
  18 inventoried constants are closed. Generic/raw/control statements,
  `byte_list_length`, `byte_ty` and all six obligations have closed contexts.
  No native axiom appears. Global closure does not remove explicit model premises.
- Independently inspected the entire real kernel summary. Exactly
  `Coq.Logic.FunctionalExtensionality.functional_extensionality_dep` and
  `Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq` appear. Set is predicative; rewrite
  rules are disallowed; type-in-type, unsafe (co)fixpoints and assumed positivity
  inventories are empty. The complete summary is byte-identical to the original
  disclosed summary (SHA-256
  `960298c45af69cab79feb24986db3c1625fa949b59775c1fd8c11bf410bdb526`).
- All four source files are byte-identical at `8935534e`, `05581f4b`, `1efbd004`
  and the current base. All four fresh `.vo` files match both the original and
  parent proof receipts and actual original objects. Their hashes are recorded
  below and in the machine-readable validation.
- All eight unit tests pass under both normal and optimized Python. Fresh source
  audit passes its 12 controls; readiness passes its four controls, including
  fake-success/empty-copied-source metadata. All are eligibility/setup refusals,
  not semantic negative proofs.
- Fresh `guard_controls.py`, executed from a byte-identical private controller
  mirror so transient candidate copies stay out of this worktree, passes all
  three acceptance refusals. Reduced inventory fails before any prover command.
  The inline imported `classic` variant normally compiles all four modules and
  the inventory, then fails on `unexpected logical axiom: Classical_Prop.classic`.
  The unused hidden classical definition normally compiles all four and its
  closed/FE-only inventory; recursive coqchk exits 0, then the kernel-summary
  gate refuses `Coq.Logic.Classical_Prop.classic`. No compiler/tactic/resource
  failure supplies one of these refusal results.
- Additional probes cover 67 parser/source cases and 13 full proof-controller
  refusals. These include inline/qualified/indented extra axioms; missing,
  duplicate, reordered or mislabeled boundaries; wrong per-constant assumptions;
  unsafe/missing/duplicate kernel summaries; omitted mandatory proof-controller
  inputs; empty/reordered/duplicate module lists; and 16 altered borrowck/native
  operand or inventory cases. Old unsize-local-1 and call-local-1 mutations now
  refuse. Missing or byte-changed receipt/native/borrowck extraction files refuse;
  an exact private copy of the real extraction remains compatible.
- Independently authenticated the reviewed ready marker and successful
  durable-build-2 final receipt. Recomputed the 523-file copied-source inventory
  against both the build and the clean official VerusBelt checkout at
  `115a618dd35ff8337112cf2081a7f5391d3973d5`. Recomputed all 284 baseline `.vo`
  hashes and canonical map hash
  `92585333704cfeea68bed68a061df64939d6b5c2c72d1c3398f0680a9cd394c4`.
  Coqc/coqchk/wrapper/opam hashes match `baseline-bindings.json`.
  Actual tool output identifies Coq 8.20.0 and OCaml 4.14.2.
  The baseline snapshot remained unchanged; no baseline install/copy/build/edit
  occurred. All proof compilation wrote only to private output paths.
- Authenticated the original extraction receipt and both actual JSON files
  against the frozen source binding. Fresh mapping has six instances, 44
  `unproved_rejected` operations, and three additional source-phase/exception
  obligations. Expanding the committed map's instance dictionary yields exactly
  the fresh map. This replays reviewed raw evidence; it does not rerun extraction.
  A fresh extraction requires explicit review and pretrial rebinding.
- Independently verified the parent COMPLETE replay receipt and its seven command
  log hashes, original/parent positive proof receipts, all five original successful
  receipts, and seven retained failed-trial receipt/stderr pairs. Recovered all
  41 archived scripts directly with `git show 05581f4b:<path>`, matched every
  `HISTORY.json` hash and the 2,733-line total. No lost evidence was found.

| Module | Fresh/original proof-object SHA-256 |
|---|---|
| borrow_handoff | `20e45423a7bdeb96380f865907bab68e454dc2e9491b202cc684091856532286` |
| byte_array | `1f09a29760dd4457743bf24f33ab7bf33b4c77ee586fcf7645f8bec1bc2098a4` |
| raw_view | `14447220c43800c42a2e09c9892c0260a65abeaf4b6978ec59aa03f9d3c9996e` |
| model_limits | `e1defe41a5e3746f55eb3d5471b37d66218085b88231124d83c9e28f0fbed3f6` |

## Trust and semantic limits

The output parsers are specific to the pinned normal Rocq emitter, not arbitrary
hostile text validators. Three deliberately synthetic format probes are recorded
honestly as tolerances: an indented split unknown axiom name after an allowed
axiom, non-heading text following a `<none>` summary line, and text outside the
inventory delimiters. They are accepted by the corresponding parser in isolation.
No real-emitter/full-controller false acceptance was demonstrated from these
strings; the genuine hidden-dependency control is refused by the controller's
real recursive-kernel-summary gate. These observations do not warrant claiming a native or logical
counterexample. The scripts verify fixed inputs before/after trials, rather than
providing protection against a hostile OS, Python runtime, dynamic library,
standard-library replacement or a malicious concurrent writer that restores
bytes between snapshots. The independently authenticated, read-only shared
baseline and tool installation remain trusted inputs.

The exact semantic boundary is unchanged. `source_context_access` assumes the
legitimate model `ty_own` for the original semantic reference plus ordinary
lifetime/typing contexts; its public form needs no extra client `G`/PointsTo.
It opens the actual unique prophetic payload and ticket, consumes that original
loan while open, and restores the same location/current/prophecy/index and local
context on closure. It does not establish that an actual incoming Rust reference
inhabits that model interpretation. Byte statements cover arbitrary model values
0..255 and locations, without zero-initialization or allocation-base premises;
this is not a checked native layout/provenance relation. `ptr_ty` is copyable
with ghost interpretation True and grants no heap access. Offset is mathematical
unbounded displacement; empty-heap Offset adequacy is a model-limit control.

No actual Rust input embedding, primitive or source-phase simulation, original
array exit adequacy, or Verus C/F import proof is established. All 44 minimal
simulations and all 101 full-writer effects remain open. Publication is false.
Native-drop 0/1 and full frozen HMAC 1/1 are inherited outcomes, not rerun or
newly proved. No source-entry implementation, Rust key fix, or full Rust suite
was duplicated or read from another live worktree.

Full global acceptance still requires canonical 6/0 for every u64 and arbitrary
initialized `[10]`, with the same actually checked assistance for the complete
HMAC XOR prefix and unchanged suffix over arbitrary initialized `[128]`. All
valid key lengths, including 0 and >128, dangling empty keys, and disjoint key/pad
regions in one allocation remain required. No new application permission,
input restriction, assumed bridge, supplied raw authority, or native axiom is
introduced by this review.

Only this new evidence directory is committed. No subagents, push, PR,
implementation edit, publication hook or shared-baseline mutation. An initial
`find .. -name AGENTS.md` was interrupted before output; it traversed parent
directory entries but opened no sibling file contents. All subsequent file reads
used this checkout, explicit ancestor instruction paths, or the authorized private
raw evidence/baseline paths. Historical freezes retain their original controllers;
the active positive run uses `review-fix-freeze.json` exclusively.
