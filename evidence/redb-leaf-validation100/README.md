# Redb leaf validation100 — independently reviewed conditional proof repair

The supplemental leaf007 partition gap has an independently reviewed proof-only repair.
All three original functions pass the release verifier and a restored repeat.
The two original partition assertions, signatures, requires clauses and input
domain remain unchanged. The original failed013 record remains failed.

## Proof delta and controls

The exact [proof-only.patch](proof-only.patch) adds one 14-line ghost block. It
explicitly instantiates the imported strict partition on accessor key bytes and
uses the existing open specification to relate them to encoded-page keys.
There are no new lemmas, assumptions or provider-contract changes. The parent
independently reproduced the patch and insertion-only source comparison.

Five of 16 new verifier applications were used: unchanged failure reproduction,
repaired release positive, correct mathematical control, intended incorrect
mathematical control, restored positive. The incorrect control has one localized
backend assertion failure and no requested export. The paired arithmetic control
checks backend/nonpublication behavior; it is not a partition-specific mutation.
Fresh independent review inspected the raw evidence and added that stronger
control in new scratch without changing this frozen candidate.

Two disclosed metadata-processing corrections are reviewed separately. The
original exception tracebacks are absent: the frozen candidate retains disclosure,
draft code and raw verifier logs, not contemporaneous exception logs. The reviewer
independently reproduced the metadata errors and explicitly labels those logs as
reproductions. They do not authenticate the original exception occurrence or add
verifier calls or proof credit. The unchanged [candidate report](CANDIDATE-REPORT.md)
must be read with this evidence qualification.

## Independent partition-specific control

The [independent review](INDEPENDENT-REVIEW.md) confirms exactly one 14-line ghost
insertion, preserving all three original functions, the original domain and
native statements, and both final partition assertions. It finds no blocking
objection to this conditional supplemental release proof repair.

Three of four fresh review applications were used. The exact repaired positive
checks all three selected executable bodies and writes its requested ordinary
export. Changing only the original final before-partition comparator from `<`
to `>=` gives one localized backend assertion failure and no requested export
or interface. Byte-identical restoration again checks all three bodies and
writes its requested ordinary export. The Infer `interface_written` flag stays
false even on these positives; an ordinary `.vir` export is not an Infer
interface or native acceptance. The report also explains a nonvacuous admitted
singleton input mathematically, without native execution or a claimed application
counterexample.

The parent authenticated all 123 review payloads plus the manifest/seal,
independently checked raw control outcomes and the one-comparator delta, and
compared all 125 review-archive members with their source bytes. Exact hashes,
counts and remaining limits are in [REVIEW-RESULTS.json](REVIEW-RESULTS.json).
`INDEPENDENT-REVIEW-HANDOFF.tar.gz` preserves the complete sealed handoff.

## Retention and boundary

`CANDIDATE-HANDOFF.tar.gz` contains all 76 authenticated payloads plus their
manifest and seal. Seal SHA256:
`0d370a2c977c9db13f42ba8bfdb9bedb9dcd3b5f6323d9f391e7b4196e3c0bce`.
Every archive member was compared with its source bytes. `RESULTS.json` binds
the archive/member hashes; `candidate.rs` is an exact source copy for inspection,
not integrated application code. No runtime binary or build cache is copied.

The matched private midpoint runtime/imports and midpoint/six LE std bindings
remain inherited **conditional assumptions**, not new Rust-core proofs or live
adoption. Original native source-path preservation remains FAILED and native Git
continuity UNAVAILABLE; recovered pinned content is not a restored checkout.
This release client result establishes no native builder-debug, Drop/borrow,
panic, full trait ABI, producer lifecycle or whole-database theorem.

All 18 final redb cells remain OPEN. No new trust, admission policy, runtime,
final theorem scope or whole-goal acceptance is adopted. The shared redb status
was reconciled after admission101 finished using its original pinned context.
