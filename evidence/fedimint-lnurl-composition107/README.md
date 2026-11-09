# Fedimint LNURL accounting107 — qualified review, loader provenance open

This fresh bounded implementation composes separately identified checked native
fee/debit and expiration providers. For arbitrary u64 amount and fee components,
it establishes `debit + recovered_fee = max(amount, fee)` with a representable
sum, without a `fee <= amount` caller premise. A separate corollary characterizes
exactly when that reconstructed value equals the original amount.

An independently supplied receiver retains its own inputs and exact overflow
boundary. Its checked-add projection is an authored observer, not a rewrite or
totality claim for native unchecked addition. The source-guarded adapter proves
numeric-policy reconstruction and contract amount at least 49,500 msat; `Some`
does not mean successful invoice/receive execution.

The [candidate report](CANDIDATE-REPORT.md) and [attempt ledger](ATTEMPTS.jsonl)
retain all eleven applications, including an initial frontend failure. Checked
providers report 55 and 11 verification conditions; imported composition reports
10, combined composition 76, and explicit sum corollary 11. These counts are not
new native-body proof counts. The wrong-value mutant fails two accounting
postconditions only, while a separate accepted 100,000-msat witness proves the
incorrect reported value 99,000. Corrected and byte-identically restored sources
pass. False unconditional conservation and receiver totality are also rejected.

`CANDIDATE-HANDOFF.tar.gz` retains all 120 authenticated payloads plus the
manifest and seal. Every archive member was compared byte-for-byte, and all
payloads matched before and after retention. Generated candidate-cache records
inside this evidence archive are retained as original receipts, not installed
or reused as runtime cache state. [RESULTS.json](RESULTS.json) binds all identities;
[SOURCE.patch](SOURCE.patch) is an unintegrated candidate patch.

Parent retention initially expected archive names without the actual
`fedimint-lnurl/` prefix. That metadata comparison failed before copying any
payload; the corrected prefix-aware comparison authenticated all 122 original
members. Neither source nor archive was changed, and no verifier was rerun.

Fresh independent review confirms the numeric slice with a loader-provenance
objection still open. Recovered native bytes are source-content
evidence only: original-path preservation failed, and compiler/source continuity
is not repaired. Twelve current-parent versus frozen-source-map differences
remain explicitly disclosed. Existing compiler, vstd and arithmetic/library
assumptions remain conditional; no new trust, runtime adoption or parent semantic
integration is claimed. All eighteen final redb cells remain **OPEN**.

## Independent review and transitive-loader qualification

The unchanged [independent review](INDEPENDENT-REVIEW.md) finds no new numeric
proof, provider/certificate correspondence or mutation-localization defect.
All seven fresh applications behave as expected, including provider and
composition positives, the localized wrong-value rejection, the 99,000-msat
witness and byte-identical restoration. All eleven historical attempts and
failures remain separately audited, not erased or counted as fresh rechecks.

The review identifies P2: the transitive `rustup` executable was not hashed at
entry or before execution in either the candidate or review runs. Late hashing
cannot establish its earlier identity. This prevents a complete actual-tool
coverage claim; it is not a demonstrated arithmetic counterexample. No further
applications are launched to retroactively repair historical provenance.

`INDEPENDENT-REVIEW-HANDOFF.tar.gz` retains all 74 authenticated review payloads
plus manifest and seal; every original archive member was compared byte-for-byte.
[REVIEW-RESULTS.json](REVIEW-RESULTS.json) binds review identities separately from
the unchanged original candidate. No adoption or source/runtime acceptance follows.
