# Frozen probe outcomes

All 71 sources were frozen before their first invocation. Counts are verified/errors. Every invocation retains `--no-cheating`; full commands, names, verification scopes, artifact/log hashes and the original frozen hypotheses are in `manifest.json`. Frontend failures are excluded from semantic evidence.

The original controller completed all 142 invocations, then exited 1 on the stale `{ key }` expectation. `probe-adjudication.json` in the manifest preserves the original receipt and the exact two objections; it independently rechecks every input guard and outcome. No source, hypothesis, result or compiler invocation was replaced.

| Frozen case | Off: class, count | On: class, count | Observer selected on | Executable / proof bodies |
| --- | --- | --- | --- | --- |
| prior-prior-after-write | success 2/0 | success 2/0 | none | after_write / none |
| prior-prior-alias-after | success 3/0 | success 3/0 | none | write, client / none |
| prior-prior-before-write | success 2/0 | success 2/0 | none | before_write / none |
| prior-prior-escape | success 2/0 | success 2/0 | none | client / none |
| prior-prior-helper-before | success 3/0 | success 3/0 | none | write, client / none |
| prior-prior-match-move-after | success 2/0 | success 2/0 | none | client / none |
| prior-prior-move-let-after | success 2/0 | success 2/0 | none | client / none |
| prior-prior-nested-before | success 2/0 | success 2/0 | none | client / none |
| prior-prior-pattern-ref-after | success 2/0 | success 2/0 | none | client / none |
| prior-prior-pattern-ref-before | success 2/0 | success 2/0 | none | client / none |
| prior-prior-projection-helper | success 3/0 | success 3/0 | none | write, client / none |
| prior-prior-returned-helper | success 3/0 | success 3/0 | none | id, client / none |
| prior-prior-simple | success 2/0 | success 2/0 | client | client / none |
| prior-tuple-move-alias | success 2/0 | success 2/0 | none | client / none |
| prior-match-reborrow-alias | success 2/0 | success 2/0 | none | client / none |
| prior-tuple-ref-mut-alias | success 2/0 | success 2/0 | client | client / none |
| prior-target-scalar-before | success 2/0 | success 2/0 | client | client / none |
| prior-target-array-copy-after | success 2/0 | success 2/0 | client | client / none |
| prior-index-local-before | success 2/0 | success 2/0 | none | client / none |
| prior-index-target-before | success 2/0 | success 2/0 | none | client / none |
| prior-index-local-after | success 2/0 | success 2/0 | none | client / none |
| prior-temporary-copy-before | success 2/0 | success 2/0 | none | client / none |
| prior-temporary-copy-after | success 2/0 | success 2/0 | none | client / none |
| prior-index-helper-before | success 3/0 | success 3/0 | none | write, client / none |
| prior-constructor-effect-before | success 2/0 | success 2/0 | none | client / none |
| prior-constructor-effect-nested | success 2/0 | success 2/0 | none | client / none |
| prior-constructor-mismatch | success 3/0 | success 3/0 | none | make, client / none |
| prior-constructor-mismatch-nested | success 3/0 | success 3/0 | none | make, client / none |
| prior-constructor-mismatch-conditional | success 3/0 | success 3/0 | none | make, client / none |
| prior-constructor-mismatch-zip-unused | success 3/0 | success 3/0 | none | make, client / none |
| prior-constructor-mismatch-shadow | success 3/0 | success 3/0 | none | make, client / none |
| prior-multiple-buffers-loops | success 3/0 | success 3/0 | none | client / none |
| prior-multiple-buffers-unused | success 2/0 | success 2/0 | none | client / none |
| prior-binder-shadow | success 2/0 | success 2/0 | client | client / none |
| prior-prefix-readonly | postcondition 1/1 | success 2/0 | xor_key | xor_key / none |
| prior-prefix-key-temporary | postcondition 1/1 | postcondition 1/1 | none | xor_key / none |
| prior-prefix-unrelated-before-after | postcondition 1/1 | success 2/0 | xor_key | xor_key / none |
| prior-zip-helper-key | success 3/0 | success 3/0 | none | make_key, client / none |
| prior-zip-key-local-reassigned | success 2/0 | success 2/0 | none | client / none |
| prior-zip-key-parameter-reassigned | frontend 0/0 | frontend 0/0 | none | client / none |
| prior-zip-key-conditional | success 2/0 | success 2/0 | none | client / none |
| prior-numeric-prefix-composition | invariant 3/3 | success 8/0 | u64, xor_key | u64, xor_key / none |
| prior-invalid-overlap | frontend 0/0 | frontend 0/0 | none | client / none |
| prior-overflow-after-skip | safety 1/1 | safety 1/1 | none | client / none |
| prior-zip-key-parameter-same-lifetime | success 2/0 | success 2/0 | none | client / none |
| immutable-local-alias | success 2/0 | success 2/0 | none | client / none |
| same-name-immutable-shadow | success 2/0 | success 2/0 | none | client / none |
| same-name-mutable-shadow | success 2/0 | success 2/0 | none | client / none |
| shadow-after-constructor | success 2/0 | success 2/0 | client | client / none |
| nested-shadow-expression | success 2/0 | success 2/0 | none | client / none |
| shared-reference-projection | success 2/0 | success 2/0 | none | client / none |
| tuple-parameter-projection | success 2/0 | success 2/0 | none | client / none |
| struct-parameter-projection | success 2/0 | success 2/0 | none | client / none |
| block-local-capture | success 2/0 | success 2/0 | none | client / none |
| match-key-expression | success 2/0 | success 2/0 | none | client / none |
| mut-parameter-unassigned | success 2/0 | success 2/0 | none | client / none |
| mut-parameter-preassigned | success 2/0 | success 2/0 | none | client / none |
| mut-parameter-alias-reassign | success 2/0 | success 2/0 | none | client / none |
| mut-ref-parameter-reassign | success 2/0 | success 2/0 | none | client / none |
| block-counter-side-effect | success 2/0 | success 2/0 | none | client / none |
| helper-counter-side-effect | success 3/0 | success 3/0 | none | make_key, client / none |
| qualified-key-lookalike | success 3/0 | success 3/0 | none | zip, client / none |
| qualified-native-zip | success 2/0 | success 2/0 | client | client / none |
| qualified-native-iter | success 2/0 | success 2/0 | client | client / none |
| mut-key-with-proof-body | success 3/0 | success 3/0 | none | client / checked_identity |
| mutable-skipped-false-assert | assertion 1/1 | assertion 1/1 | none | client / none |
| helper-skipped-false-post | postcondition 2/1 | postcondition 2/1 | none | make_key, client / none |
| immutable-param-mutation-invalid | frontend 0/0 | frontend 0/0 | client | client / none |
| key-reborrow-lifetime-invalid | frontend 0/0 | frontend 0/0 | none | client / none |
| prefix-immutable-local-skip | postcondition 1/1 | postcondition 1/1 | none | xor_key / none |
| prefix-mutable-param-skip | postcondition 1/1 | postcondition 1/1 | none | xor_key / none |
