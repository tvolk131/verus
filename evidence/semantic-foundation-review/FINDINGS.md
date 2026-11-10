# Independent semantic foundation review — preliminary controller finding

Review target: `1a9d4f22`, accepted source `8935534e`, handoff `05581f4b`.
This early committed repro is evidence only. Full v12 replay/review is in progress.

**P2 — the automatic extra-assumption gate misses single-line axioms.**
`source/tools/research/native_semantic_bridge/check.py:120` collects only lines
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
