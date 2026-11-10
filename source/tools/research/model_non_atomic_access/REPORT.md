# Ordinary model memory phases and same-cell XOR

Trial006 passed normal compilation, all nine individual assumption reports and
full recursive kernel checking. Every new declaration is closed. `results.json`
pins the completed receipt and objects; `development.json` retains the five
failed proof-development attempts separately from semantic evidence.

The primitive results use the actual existing lrust `prim_step` relation, with
empty observations and no forks. SC read/write each take one step. An isolated
ordinary read takes two: it increments the reader count, then restores that
count and the original complete state. Its starting count is arbitrary. An
ordinary write takes two: it requires RSt 0, passes through WSt and returns the
same final updated state as the SC primitive under their common premise.
Freeability and all other cells are preserved. These are isolated finite
executions, not unrestricted concurrent SC/non-atomic program equivalence.

The checked counterlemmas show that the first ordinary read phase changes the
state and that an ordinary write cannot start with active readers. No ordinary
operation or complete assignment is claimed pure. Those mathematical facts are
not Rust verifier mutation/rejection controls.

`cell_assignment_na` is an explicit MODEL ordering variant of the existing
read/XOR/same-cell-write expression. It performs genuine ordinary accesses and
calls the unchanged checked byte_xor operation. Its WP requires existing cell
ownership, byte-range facts and arbitrary Frame, and returns the updated same
cell, range and unchanged Frame. It does not request a store credit: the existing
two-phase ordinary store rule supplies its internal phase accounting. This is a
sufficient conditional WP, not a minimal-budget or termination claim.

Ownership remains an internal proof premise. The initialized model setup must
derive it before using this law; it is not a new Rust application requirement.
No native reference, register 17/19, provenance, retag, source phase or original
C/F correspondence is established. The original frozen Rust body/domain and
production code are untouched. Native correspondences remain zero, all44+3/101
remain open, full HMAC remains inherited1/1, publication and goal completion
remain false. The new model ordering variant does not replace the frozen body.

The controller authenticates readiness and successful baseline receipt, all523
sources/284 objects, tools, GMP manifest and complete source/object/receipt
identities of the three checked imported libraries. Sources, retained freezes
and products stay guarded through the final recursive check. Full dependency
context contains only inherited FE/Eq_rect_eq; no new axiom, primitive, unsafe
mode, constructor or obligation was added. Only compact artifacts are committed.

Replay into a new private output directory:

```sh
python3 -B source/tools/research/model_non_atomic_access/check.py \
  --freeze source/tools/research/model_non_atomic_access/final-freeze.json \
  --output /tmp/model-non-atomic-access-NEW
```

Independent review remains pending.
