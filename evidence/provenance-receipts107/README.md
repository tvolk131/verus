# Selected-input provenance receipts107 — reviewed defect repaired and integrated

The scratch-only [candidate patch](SOURCE.patch) adds an opt-in standard-library
Python capture helper, documentation and 38 hermetic tests. It does not replace
the existing native-resource runner or characterize its explicitly selected
input scope as defective. A parent-corrected version of these three new opt-in
files is now integrated; no existing runner or compiler semantics are changed.

The helper records selected inputs, command shape, executable/compiler identities,
fresh output identities and sanitized execution evidence. Missing/seeded/stale
outputs, input drift, failed commands/timeouts, scan/log failures and malformed
plans refuse success. Only a fixed allowlisted child environment is used.
These are bounded checks, not a process sandbox or cryptographic source-to-binary
attestation; caller-declared compiler roles and command labels are not authority.
Vstd-only invocation is explicitly distinct from an engine-build command shape.

The [candidate report](CANDIDATE-REPORT.md) discloses all development failures,
the final 38/38 passing run and a separate byte-exact patch reconstruction with
38/38 passing. Hash snapshots follow discovery and are not atomic or complete
checkout authentication. The original candidate patch remains unchanged evidence.

`CANDIDATE-HANDOFF.tar.gz` retains every authenticated manifest payload plus
the manifest and seal, excluding no declared failure record. All archive members
were compared byte-for-byte and original hashes matched before/after retention.
[RESULTS.json](RESULTS.json) binds archive/member identities separately from
acceptance. No compiler/runtime/source correspondence, native proof, trust
adoption or admission/export change is established. All eighteen redb cells
remain **OPEN**.

## Independent review and parent cancellation repair

The unchanged [independent review](INDEPENDENT-REVIEW.md) blocked the original
candidate: `KeyboardInterrupt` could write `success=true` with no recorded exit
and leave its trusted child running. Its separate log-wording finding is
nonblocking: final-path hashing does not detect replacement by another regular
file. The parent reproduced the real-SIGINT failure before editing; the failed
receipt, live-child observation and reviewer-fixture cleanup are retained.

The parent repair records cancellation, kills/reaps the launched process group
before final observations, propagates `KeyboardInterrupt`/`SystemExit`, and
requires a completed zero exit with no timeout, interruption or errors for
success. Two new regression tests cover both exception types, and log wording
and the symlink-test name now match the actual endpoint guarantee.

Validation against the repaired scratch source passes 40 unit tests and all
13 reused reviewer checks. The byte-identical integrated parent files also
pass all 40 unit tests. This is a parent repair and validation, **not** a new
independent review; the reviewer explicitly recommended this bounded response.
The original review and red reproduction are not relabelled as passing results.

`INDEPENDENT-REVIEW-HANDOFF.tar.gz` retains the original sealed review bundle.
`CANCELLATION-REPAIR-HANDOFF.tar.gz` retains the exact repair source/delta,
red/green observations, commands, sanitized environments and raw test logs;
every repair archive member was compared byte-for-byte. The
[repair delta](CANCELLATION-REPAIR.patch) is against the unchanged original
candidate. `RESULTS.json` binds both versions and all integrated source hashes.

The helper remains explicitly opt-in. No diagnostic build, runtime adoption,
native producer or compiler/source attestation follows from its integration.
