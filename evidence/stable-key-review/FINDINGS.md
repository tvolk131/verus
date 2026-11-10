# Scoped findings

No new source, logic, trust or publication blocker was reproduced. Recommend
**accepting a93b774a's stable-key restriction** and closing R1/R2 from
`evidence/observer-origin-review/FINDINGS.md` at
`8b484ab33a6aef3986ce998d29da232992c5fe11`.

## R1 closed: mutable key bindings skip assistance

The exact previously frozen local-reassignment and explicit-same-lifetime
parameter sources each verify **2/0 off and 2/0 on**, with no observer selected.
Executable inventory is exactly `{client}`, proof and inferred-helper inventories
are empty. The actual iterator's original key value is no longer confused with
later evaluations of a mutable binding. New same-name mutable shadows, mutable
parameter alias reassignment, and reassignment through `&mut &slice` also verify
with assistance skipped.

## R2 closed: executable key producers retain ordinary verification

The exact previously frozen `make_key` repro verifies **3/0 off and 3/0 on**,
with no observer selected and no Infer annotation-purity abort. Exactly
`{make_key, client}` executable bodies are checked; proof and inferred-helper
inventories are empty. New side-effectful and qualified key producers also skip
and verify. A deliberately false helper postcondition still fails in both modes.

## Intentional conservative boundary, not a requested change

Immutable local aliases, conditionals, matches, projections and braced key
expressions skip. The old prefix `.zip({ key })` probe therefore fails its authored
postcondition **1/1 in both modes**. The original frozen review hypothesis wrongly
retained its old success/selection expectation; all 142 invocations completed,
then the original controller exited 1 on exactly those two expectation mismatches.
The untouched receipt, original hypotheses and explicit independent adjudication
are retained. No compiler/source rerun or outcome substitution was made.

The bare immutable-parameter readonly prefix still proves **2/0 on versus 1/1
off**, with its arbitrary buffer and all valid key lengths unchanged. Unsupported
shape skipping does not require narrowing that application domain.

See [REVIEW.md](REVIEW.md), [PROBES.md](PROBES.md) and the hash-bound
[manifest.json](manifest.json) for full validation, exact commands and limits.
The full native/HMAC goal remains incomplete.
