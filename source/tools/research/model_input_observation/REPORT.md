# Resource-valued typed setup and final-state observation

Three MODEL composition theorems passed normal compilation and recursive
kernel checking in `input-observation-trial005`. Each inherits only the existing
dependent functional extensionality; complete object, receipt and freeze hashes
are in `results.json`. No Program obligations or new assumptions were added.
Independent review is pending.

`raw_model_context_wp` keeps the existing outer `lrustGS` fixed. From arbitrary
initial model fragments, time context and £(1+k), it derives the existing
typed-reference/lifetime/prophecy context and exact outside frame, executes the
explicit `Skip;;e` setup, and preserves an arbitrary resource postcondition
`Phi : val -> iProp Sigma`. The callback's typeG must wrap that SAME outer heap
instance; its field equality is a callback interface condition discharged by
the construction, not a permission supplied by the application. Reference
construction reuses the accepted ModelInputContext lemma. The two setup beta
steps and credits are unchanged from that library.

`raw_model_context_observed` composes this setup with the independently checked
final-state observer. A genuine callback WP returning a full interval, its
explicit length and semantic property, plus the exact original outside frame,
establishes those same values in actual final model-memory lookups. No second
heap, chosen final state, pure-result substitution, final RSt0 or termination
assumption is used. Inputs include arbitrary finite heaps, ranged byte lists,
lengths including 0/10/128 and strict interiors. The callback must still prove
its physical-resource postcondition; no buffer recovery follows from the
generic interface alone.

`raw_model_read_outside_observed` exercises resource preservation with the
existing executed borrowed read/restore/resolve protocol. It derives both the
actual returned head byte and actual final lookups for the entire original
outside frame. Initial £3 splits into lifetime setup £1 and read/resolution £2.
The read requires a nonempty head lookup; the generic setup has no nonempty
restriction. This theorem observes the OUTSIDE frame. It does not recover or
claim unchanged final contents of the still-borrowed input interval. That
requires the separately pending lifetime-end physical-resource recovery.

The controller authenticates the pinned foundation, ready and successful build
receipts, 523 sources/284 objects, tools/loadpaths, GMP manifest and all four
completed prerequisite libraries against their committed receipts. Source and
object copies remain guarded through recursive checking; proof-success flags
follow all final guards. Outputs are private. The final freeze is byte-identical
to trial005's pretrial freeze. Replay with a new private output:

```sh
python3 -B -O source/tools/research/model_input_observation/check.py \
  --freeze source/tools/research/model_input_observation/final-freeze.json \
  --output /tmp/fresh-model-input-observation
```

`development.json` retains the two failed full-controller trials and three
isolated read-script failures, their original sources, freezes and receipts.
They are elaboration/tactic failures, never semantic negatives. Earlier checked
one-/two-theorem milestones remain historical. No filesystem substitution or
concurrency experiment, toolchain install, baseline edit or production change
was performed.

This closes the MODEL interface gap between typed setup and resource-sensitive
final observation. It does not identify the original Rust input loan, preserve
its incoming ancestors/provenance, prove native MIR/phase simulation or import
the original executable reference's C/F observations. Native correspondences
remain 0; all 44 minimal plus three phase/101 writer effects remain open. Full
canonical remains 6/0; full frozen HMAC prefix AND untouched suffix remains
1/1. Original bodies/domains are intact, publication and goal completion false.
