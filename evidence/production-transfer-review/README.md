# Independent production transfer integration review

Reviewed November 9, 2026: `55bd67ad980e90e92c77efe4453a927f097c3386`
through `3b05c1912b9d7314ad00c832c4065f4362562e02`, including the final
`infer-proof-composition` report and subsequent CI/embedded-vstd changes.
Historical reports were read as claims, not counted as fresh validation.

**Recommendation: accept for the stated extracted-helper transfer scope.**
No acceptance-blocking correctness or trust-boundary defect was found.
No production code, upstream application source, or parent artifact was edited.
This review does not establish completion of later stateful verification work.

## Findings and objections

No false contract, weakened precision check, unchecked auxiliary fact, or failed
publication barrier was reproduced. The principal trust boundaries remain:

- Canonical libcore API correspondence is trusted frontend code. The two Option
  bridge bodies are checked, but this is not a universal equivalence proof for
  arbitrary native effects and destructors.
- The two primitive little-endian byte specifications, including `no_unwind`,
  trust Rust library behavior. Their implementation bodies are not verified.
- Existing vstd specifications, array extensional equality, Rust/Verus lowering,
  imported artifacts and the SMT pipeline remain trusted.

Two synthesis limits were reproduced without changing their failing fixtures:
`Option::map_or` returning the literal empty array is unsupported, and a
single-scalar-field codec with a mutable accumulator is projected into the
scalar profile and rejected. A separate two-field codec retains the nested
update and verifies with checked caller assistance. These are rejected inputs,
not accepted partial array summaries or evidence against the original redb
codec. The bounded native-byte-callback result does not imply arbitrary array
literal callback support.

A static concern about resolving saved calls through later assignments was
**not reproduced**. The equation-preserving return probe passes with and without
`--infer-goals`; its false-property counterpart fails both ways. The initial
`result == 0` formulation fails ordinary solver composition too, so it cannot
support a claim that assistance introduced a regression. All original logs and
fixtures remain in the private raw bundle.

No broad input-domain reduction, first-party assumed implementation, disabled
verification, or substitution of native tests for proofs was needed.

## Source review anchors at the reviewed HEAD

| Concern | Reviewed implementation and conclusion |
| --- | --- |
| Native Option correspondence | `source/rust_verify/src/fn_call_to_vir.rs:3260`; `source/vstd/native_option.rs:1`: core identity/self-type/signature checks select the matching vstd diagnostic item; ordinary generic bridge bodies remain checked. |
| Exact callbacks | `source/vir/src/infer/records.rs:1221`; `source/vir/src/ast_to_sst.rs:4752`; `source/vir/src/infer_checks.rs:252`: eager ordinary values/defaults, Some-guarded callback requirements, original-body callback checking, and independent precision packet transformation remain separate. |
| Complete array outputs | `source/vir/src/infer_checks.rs:18`; `source/vir/src/infer_checks/state.rs:531`; `source/vir/src/infer/arrays.rs:132`: actual by-value arrays use whole-array shallow extensional observations; reference observations keep identity; callback-array equality is an explicit model dependency. |
| Integer conversion/unwrap | `source/vir/src/infer/integer_conversions.rs:42`; `source/vir/src/sst_to_air.rs:1929`: the closed fixed-integer conversion/unwrap route retains representability and operand safety; maximality uses the original lowered call requirements, not a guessed successful cast. |
| Local/branch effects | `source/vir/src/infer/records.rs:29`; `source/vir/src/infer/records.rs:478`; `source/vir/src/infer/records.rs:938`: the array-codec profile is straight-line; block-local bindings are discarded while outer updates survive; readonly operands cannot hide those updates. Early-return requirements remain path guarded. |
| Ordinary caller proposals | `source/vir/src/infer/goals.rs:12`; `source/vir/src/infer/goals.rs:411`; `source/vir/src/infer/bitvector_hints.rs:294`; `source/rust_verify/src/commands.rs:293`: bounded proposals decorate ordinary queries only; each theorem has empty BV requires and the identical checked/used expression. Original obligations remain. |
| Imports/cache | `source/rust_verify/src/import_export.rs:17`; `source/rust_verify/src/import_export.rs:72`; `source/vir/src/infer/registry.rs:8`: policy 63/cache 69/schema 38, entire-library model fingerprints, fresh mandatory checks, and export invalidation remain in force. Regenerated record summaries are not cache hits. |
| Transfer acceptance | `source/tools/check_infer_transfers.py:61`; scenario extraction hashes and pinned clean upstream inputs: full-crate positive proofs and separate semantic/safety/auxiliary negatives are required; frontend/resource failures cannot replace them. |

## Fresh validation and reproduction

The matching compiler, macro set and verified vstd were built in this isolated
checkout using Rust 1.99.1 and `CARGO_BUILD_JOBS=2`. Cargo ran from `source/`.
The standalone solver was copied, not symlinked, from the authorized parent
location: Z3 4.16.0, SHA-256
`e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765`.
The vstd build reports **2,221 verified, zero errors**. **134 VIR library tests**
and **65 focused integration tests across 10 binaries** pass, with no ignored
tests. The latter include embedded-vstd setup and all nine new root integration
test binaries in this delta. Python discovery passes **160 tests** (136 infer, 14 redb, 10 Fedimint),
plus **12 optimized front-door test executions**. These counts are disjoint
within normal discovery; optimized executions repeat tests.

The fresh common frontdoor reports `success: true` and
`application_proofs_proved: true` for **all four slices**. It independently
accepts 61 declared main controls and 315 repeated mandatory inference checks:
Fedimint 5/30, codec 16/84, page size 11/33, address range 29/168 (cases/checks).
Those check counts are not distinct application theorems. The combined receipt
is `transfers/transfer-suite-m1qmju9f/receipt.json` under the raw bundle.
The native finite oracles are supporting evidence only; the accepted verifier
reports, exact checks, negative diagnoses and publication results are separate.

The independent probe runner made **56 compiler invocations across 28 fixtures**:
16 successful proofs, 40 rejections, no timeouts, and no unexpected acceptance
of a false-property fixture. Every rejected invocation left no export. All
144 emitted mandatory helper checks proved; checked probe fixtures report
full-crate verification. Rejections include unsupported synthesis and ordinary
solver limitations, so this is not a claim that every exploratory positive
expectation passed. The original expected outcomes remain in the runner and
raw receipts for inspection.

Build, focused integration, Python, probe and four-scenario replay commands,
completed results, source/tool identities and log hashes are in
[`manifest.json`](manifest.json). Raw logs/receipts/fixtures are retained under
`/tmp/verus-transfer-review-20261109` (mode 0700). No binaries or large raw
receipts are committed. To repeat the independent probes after building:

```sh
export PATH=/home/dev/.cargo/bin:$PATH
python3 -B evidence/production-transfer-review/probe.py \
  --verus source/target-verus/release/verus --solver source/z3 \
  --output /path/to/new/private/probe-directory
```

Use `source/tools/INFER_TRANSFERS.md` for the common application frontdoor.
The actual pinned clean inputs here were under
`/tmp/verus-infer-cycle-20261109/applications/{fedimint,redb}`. Their exact pins,
body hashes and frozen codec domain were rechecked. The redb domain remains
`region <= 0xfffff`, `order <= 20`, `index <= (0xfffff >> order)`.
Address-range results use the inferred checked-arithmetic domain and the actual
debug-on/off configuration. They do not assert native release no-panic behavior
outside that domain. The original debug guard alone does not imply end
containment; the replay preserves the `4090..8186` witness and rejects the false
universal containment property.

This is an extracted deadline/codec/arithmetic-helper review. It is not a
whole-Fedimint, whole-redb, cryptographic, concurrency, crash-recovery, or native
Rust standard-library correctness proof. The 18 historical VIR doctest failures
were not rerun or counted as passing; this is not a full-workspace green claim.
