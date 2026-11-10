# Reproduction

Run from the repository containing `d639f378`. Set `review_private` to a new
private directory (must not exist). Do not use any parent/sibling checkout or
follow historical repository roots embedded in receipts. The only external
reads are immutable foundation/toolchain and the completed pinned libraries
and receipts named by the unchanged controller and committed metadata.

```sh
review_private=/tmp/model-transformation-observation-review-NEW
mkdir "$review_private"
git archive d639f378db2f701ffea26cef3c52950bada392bb source/tools > "$review_private/source.tar"
mkdir "$review_private/checkout"
tar -xf "$review_private/source.tar" -C "$review_private/checkout"
python3 -B evidence/model-transformation-observation-review/review.py \
  --checkout "$review_private/checkout" \
  --output "$review_private/authentication-before.json"
python3 -B "$review_private/checkout/source/tools/research/model_mutable_writer/check.py" \
  --freeze "$review_private/checkout/source/tools/research/model_mutable_writer/trial016-freeze.json" \
  --output "$review_private/replay"
python3 -B evidence/model-transformation-observation-review/review.py \
  --checkout "$review_private/checkout" --fresh "$review_private/replay" \
  --output "$review_private/audit-results.json"
```

Require every command to succeed before advancing. `review.py` first validates
the mandatory readiness **and successful durable receipt**, full baseline source
and object identities, tools/GMP, all frozen inputs, completed prerequisite
artifacts, and the authenticated seven/six-law receipts. The unchanged controller
checks its own full guard and pinned prerequisites again before compiling. Its
output directory must be new. The final command reads the fresh output only
after completion and compares all twelve V/VO artifacts byte-for-byte, including
Assumptions, with authenticated trial016; it reparses assumption/kernel logs.

For the six permitted parser-only unit tests, run in the prepared checkout's
`source/tools/research/native_semantic_bridge` directory:

```sh
python3 -B -m unittest test_proof_checks.Assumptions
```

Do not invoke the `SourceBindings` suite, which is outside this review scope.
No race/substitution or historical experiment reruns are needed or authorized.

The actual review prepared its archive directly through Python's tar reader
rather than retaining a tar file. Completed private output:
`/tmp/verus-infer-buffer-cycle-20261109/model-transformation-observation-review-d639f378/replay`.
The prepared checkout resides alongside that directory. The exact completed
receipt hash and command exit states are committed in `audit-results.json`.

`STATEMENTS.md` is extracted from the five authenticated candidate sources,
from each Lemma/Theorem header through the line before Proof. It inventories
all seven declarations, with the section contexts stated at the top. Logs and
compiled proof objects must remain private.
