# Independent formal mathematics review

**Accept the exact MODEL prerequisite; no actionable findings.** Reviewed
`a49847d7` in prepared checkout `a13f0551ebd18cc2a7dd2b642c6b2f04547597c6`.
The entire `source/tools/research/model_input_observation/` package matches the
target commit. Fresh normal compilation and complete recursive checking passed;
both requested objects are byte-identical to accepted trial005. This accepts
the three stated MODEL theorems, not the full mutable-buffer goal.

In `candidates/setup.v:10–52`, `raw_model_context_wp` fixes `Outer : lrustGS Σ`.
The explicit compiled type print confirms that both callback and concluding WP,
their heap fragments, time context, invariant machinery and credits use
`lrustGS_irisGS Outer`. The typed reference uses `TG`, constrained by the actual
record equality `type_lrustGS TG = Outer` (line 17). `TG := TypeG Σ Outer ...`
(line 34) discharges it by reflexivity (line 48). No fresh physical heap is
allocated. Lifetime, prophecy and uniqueness ghost contexts are initialized
internally; `typePreG` supplies allocation capabilities, not a caller-owned
reference, permission or lifetime guard. The fresh lifetime supplies the local
context and nonempty proof. The exact interval payload and outside frame come
from `byte_interior_payload`, using only the stated range and heap-cell facts.

The setup pays lifetime allocation £1 out of £(1+k), keeps £k for the callback,
and executes the two actual beta reductions for `Skip;;e` (lines 39–47).
`wp_cumulative_time_receipt2` supplies two time receipts from execution;
one yields persistent time 1, the other supports the existing
`bytes_reference_construct`. These are not fabricated later credits. Masks
are discharged at the specified top mask, including the time-receipt mask
transition and the lifetime/prophecy/uniqueness namespace inclusion.
`Closed [] e` is explicit. The arbitrary resource-valued `Φ` is passed through
unchanged, with no coercion to a pure postcondition or extra callback premise.

In lines 58–81, `raw_model_context_observed` applies the authenticated
`initialized_interval_frame_adequacy` with £(1+k). Line 79 rewrites the actual
heap-record equality before applying the typed callback. The compiled callback
type uses `type_lrustGS TG` consistently for its WP and returned physical
resources. The observer's `observation.v:88–104` destructs ONE returned `vs`,
retains `length vs = n` and the SAME `P v vs`, and checks its full points-to
interval and exact original difference-map frame against the actual final
`heap_ctx σ`. This is physical agreement with the final state, not a selected
state or independent pure witness. Generic lengths include zero, 10 and 128;
there is no whole-object/base-pointer restriction. There is no final RSt0 or
termination premise; adequacy has its usual safety/partial-correctness meaning.
The public callback still must prove the entire physical postcondition.

In lines 86–110, `raw_model_read_outside_observed` fixes the standard `typeΣ`,
uses £3 = lifetime £1 + read/resolution £2, and substitutes the same heap record
at line 101. The imported `composition.v:33–61` accesses the existing unique
loan, executes `Read ScOrd`, restores the same loan's payload, and performs
prophecy resolution. The postcondition passed to the final observer is ONLY
the returned-head equality and exact outside-frame fragments. Lines 105–109
discard the remaining loan/prophecy resources and observe that frame using
the actual heap authority. The imported recovery continuation still requires
`[†κ]`; this proof does not end the lifetime or invoke recovery. Thus neither
the theorem nor its proof assumes that resolution recovers the physical input
interval. The head lookup correctly excludes an empty read; generic setup
remains valid for empty input.

`check.py:27–36,59` requires the exact mandatory frozen input set, including
both shared Python support files, parser, bindings and all dependency pins.
Lines 61–103 authenticate parsed JSON bytes against retained snapshots, the
pinned readiness/build/GMP records, and all source/object artifacts from four
successful kernel-checked prerequisite receipts. Lines 106–121 authenticate
the existing 523-source/284-object foundation, tool identities and loadpaths.
Lines 145–147 bind copied model source to the frozen identity; the generated
assumptions source and compiled products remain guarded through complete
recursive checking. Lines 175–179 set the proof flag only after all final
guards. Shared support uses explicit exceptions, so `-O` does not disable checks.
Source inspection and independent retained-byte authentication found no missing
identity check relevant to this replay. No filesystem mutation/race controls
were run.

The complete new named-constant inventory is:

| Constant in `ModelInputObservation.setup` | `Print Assumptions` |
| --- | --- |
| `raw_model_context_wp` | dependent functional extensionality only |
| `raw_model_context_observed` | dependent functional extensionality only |
| `raw_model_read_outside_observed` | dependent functional extensionality only |

There are no new definitions or Program obligations. The exact axiom name is
`Coq.Logic.FunctionalExtensionality.functional_extensionality_dep`.
The complete recursive dependency summary additionally lists the existing
`Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq`; it reports predicative Set, no rewrite
rules, no type-in-type, no unsafe (co)fixpoints and no assumed positivity.

All eight historical receipt hashes authenticate. Source-only classification
agrees with the retained sources/diagnostics: trial001 is numeric-scope
elaboration, trial003 is tactic goal-count mismatch, auxiliary001 lacks
`typeΣ`'s import, and auxiliary002/003 fail `iApply` during the heap-substitution
proof. These are development failures, never semantic negatives. Trials002/004
are historical one-/two-theorem successes; trial005 is the reviewed three-theorem
package. All five trial source copies match their respective freezes, and the
three auxiliary sources match their retained source hashes.

Reproduction commands, from this checkout:

```sh
python3 -B -O source/tools/research/model_input_observation/check.py \
  --freeze source/tools/research/model_input_observation/final-freeze.json \
  --output /tmp/model-input-observation-review-a13f0551-20261110
python3 -B -O evidence/model-input-observation-review/audit.py \
  /tmp/model-input-observation-review-a13f0551-20261110 \
  evidence/model-input-observation-review/audit.json
```

The first command requires a new, nonexistent private output. Before the second,
the supplementary private `inspection/Inspect.v` imports `setup`, enables
`Set Printing All`, and checks each theorem with `Check @...`. Its exact source
hash and normal `coqc -q` command are in `audit.json.type_inspection` (no new
mathematical declarations). `audit.json` records exact argv/cwd/exit status for
all three replay stages and that inspection, every dependency source/object pin,
all package/frozen input hashes, raw-log paths/hashes and historical diagnostics.
It is a compact derived record, not a copied raw replay directory.

Accepted receipt SHA256:
`3ad9535801ca112a729115ce69eb3c7f51f8307c8852510db0b4ba288b001635`.
Fresh private receipt SHA256:
`5cb29cf8b50b0b7250203f22db6ca3c70258ed33ff29140a548a31c8c51e355c`.
Final freeze, trial005 freeze and both replay freeze copies are byte-identical:
`8c70c69533cd2344698d1fcf5212a86d697d2b849695773e56dd32c0358f9f5a`.
Both runs yield `setup.vo`
`a0b1362457f25ecf7dac43ab00cc6ad4db05700ed63d88ae2745b9fbd949a9af`
and `Assumptions.vo`
`3d88feb0ef7ee52347ed343c2e84418b6896d419f5b26ffcf26176d64eeff037`.
The GMP manifest is the required
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.

Only this new evidence directory is changed. All compiled/auxiliary products,
caches, copied sources and raw replay outputs remain under the private `/tmp`
path above. No installs, implementation changes, delegation, pushes or PR work.
No live parent job or sibling source was inspected. The unverified
`model_buffer_transform` prototype was neither inspected nor validated.

Original Rust entry, provenance/ancestor preservation, native phase simulation
and original executable C/F observations remain open. Native correspondences
remain 0, with 44 minimal + 3 phase obligations and 101 effects open. The supplied
canonical 6/0 and frozen HMAC prefix-AND-untouched-suffix 1/1 status is retained,
not rerun or upgraded by this scoped review. Publication and goal completion
remain false. No source fix or mathematical weakening is requested.
