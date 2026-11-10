# Reproduce this evidence-only recheck

Run from this repository checkout. The authorized private raw extraction and
read-only Coq/VerusBelt installation must still exist at the paths in the
reviewed bindings. Use an unused private output root. No network, dependency
installation, baseline copy/build, server, subagent or implementation edit is
needed. The controller mirror copies only the explicitly frozen controller,
support and candidate inputs, not the baseline.

```sh
review_out=/tmp/semantic-guard-fix-review-independent
bridge=source/tools/research/native_semantic_bridge
review=evidence/semantic-guard-fix-review
ready=/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json
raw=/tmp/native-semantic-bridge-e4a8dcab/source-v1
python3 -B -O "$review/validate.py" identities --output "$review_out/identities"
python3 -B -O "$review/validate.py" suite --output "$review_out/suite"
python3 -B -O "$review/probes.py" --output "$review_out/probes"
python3 -B -O "$review/recheck.py" omission --output "$review_out/omission"
python3 -B -O "$review/validate.py" history --output "$review_out/history"
python3 -B -O "$bridge/check.py" --ready "$ready" \
  --freeze "$bridge/review-fix-freeze.json" --output "$review_out/proofs"
python3 -B -O "$review/recheck.py" mirror --output "$review_out/control-mirror"
mirror="$review_out/control-mirror/private-controller-tree/$bridge"
python3 -B -O "$mirror/guard_controls.py" --ready "$ready" \
  --freeze "$mirror/review-fix-freeze.json" --output "$review_out/guard-controls"
python3 -B -O "$review/validate.py" finish --work "$review_out" \
  --output "$review_out/final"
```

`finish` separately compiles/inspects all 25 statement/assumption outputs, checks
fresh/original/parent objects, and rechecks the initial baseline snapshot.
If the controls are still running concurrently, it preserves the completed
independent inventory and asks to rerun `finish` once their receipt exists;
that resume does not recompile the inventory. The commands above run sequentially.

The separate omission repro intentionally expects **two source-controller
successes on incompletely frozen synthetic provenance** and two full-freeze
refusals. These are the remaining finding, not accepted source/native facts.
The other controls expect setup, eligibility or proof-acceptance refusals;
failed commands never count as semantic counterexamples.

`collect.py --work "$review_out" --output <existing-new-evidence-directory>`
can package the small JSON/text artifacts after successful final validation.
Large raw command logs, proof objects, source copies and the baseline stability
snapshot remain private and are individually hashed by `validation.json`.
No claim is made that these private files will survive cleanup of `/tmp`.
