# A whole supported register segment

Normal compilation and recursive kernel checking passed in trial005. All
30 declarations, including constructors and generated eliminators, have closed
assumption reports. `results.json` binds the exact receipt and two objects.
Independent review is pending.

The requested source location is the frozen constructor's bb2:1–2, selected by
the already authenticated primitive reifier: unwrap `_3` into `_6`, then Offset
that produced pointer by `_2` into `_5`. `end_segment` explicitly models these
two assignments. It is an authored fragment program, not a general MIR decoder
or proof that a Rust entry supplies its descriptors.

`register_step` reads named descriptors from one register map and updates only
the destination, using the existing `source_step` fragment relation. The source
arguments are pure metadata, never physical authority. `end_segment_exec` works
for every register map with the two stated inputs and existing numeric Offset
guard. The intermediate map contains the pointer actually produced in `_6`;
the second operation consumes that same entry. The output register map has
exactly that pointer in `_6` and its computed end in `_5`. Every other register
and the physical state are preserved. Storage lifetime, raw reference creation,
reborrow, ownership and control/cleanup nodes remain outside this segment.

`end_code` is one existing-lrust function with a lexical intermediate binding.
`end_code_four_steps` proves four deterministic pure reductions: outer call,
thin-pointer projection, intermediate binding and Offset. Pure reduction
includes step availability, empty observations, identical physical state and
no spawned threads for every initial state. No heap operation, new language
primitive or extra physical permission is introduced. The imported model
Offset remains unbounded; the source fragment's explicit guard carries the
separate finite-address bounds.

`wp_end_code` preserves arbitrary separating resources. The combined theorem
`end_segment_refinement` uses the same symbolic input pointer and length in
the register execution and the complete target expression. It relates the
returned word to register `_5`, while retaining R once. This replaces independent
edge inputs with a proved data dependency, without inventing a connecting raw
reference-creation or storage step.

The source relation is the existing authored fragment semantics, not actual
Rust/MIR execution. Its assignment to native operations, incoming context,
provenance/ancestor loans, source phases and original C/F views remain unproved.
The source pin identifies the requested location; it supplies no semantic fact.
Native correspondences remain 0, all 44 minimal plus three phase/101 effects
open, full HMAC 1/1 and publication false. Original Rust bodies/domains and
the full goal are unchanged.

The controller authenticates the pinned foundation, GMP identity, all required
shared helpers and the completed primitive library's sources/objects/receipt.
Copied sources and products stay guarded through full recursive checking.
All outputs are private; the final freeze equals trial005's pretrial freeze.
Failed scripts are recorded in `development.json`, never semantic negatives.

```sh
python3 -B -O source/tools/research/native_register_segment/check.py \
  --freeze source/tools/research/native_register_segment/final-freeze.json \
  --output /tmp/fresh-native-register-segment
```
