# Parallel advancement107 — implementation lanes and decision gates

The [bounded five-lane plan](PLAN.md) is active. Each lane runs in separate
private scratch; inherited parent source and failure ledgers are preserved.
Results are reviewed before semantic integration, not automatically adopted.

- **Fedimint accounting:** eleven applications completed; full-domain accounting,
  independent receiver overflow and numeric-policy reconstruction are checked.
  [Authenticated candidate retention](../fedimint-lnurl-composition107/README.md)
  is complete; all seven independent numeric rechecks behave as expected, with
  an open transitive-loader provenance objection. Native continuity is not repaired.
- **Hex native composition:** three applications completed; two full positives
  each report 54 verified obligations, and the nibble-swap control rejects at its
  intended assertion. [Authenticated candidate retention](../hex-native-composition107/README.md)
  is complete; fresh independent review/rechecks pass without a new
  theorem-specific blocker. No
  `decode_to_array::<1>` or persisted warm-cache claim is made.
- **Provenance receipts:** helper, documentation and 38 hermetic tests delivered.
  [Authenticated retention](../provenance-receipts107/README.md) is complete;
  independent review found a cancellation blocker. The parent reproduced and
  repaired it, passed 40 unit tests plus 13 reviewer checks, and integrated only
  the corrected three opt-in tooling files. Receipts are not attestations.
- **Restricted owned codec:** actual VIR typed conversion and nine tests delivered
  in a private test-only module. [Authenticated retention](../owned-candidate-codec107/README.md)
  is complete; independent review requested three changes. The parent repaired
  them in fresh scratch with five fail-before regressions and 174 passing final
  inference tests, including 19 codec tests. The attempted complete VIR run hit
  unavailable Z3 and remains exit 128. Neither the codec nor persistence is
  integrated or accepted. The original denied-metrics wrapper result is preserved.
- **Native diagnostic proposal:** independent source-only review completed.
  Execution is **NOT AUTHORIZED**. Other lanes do not wait for this decision.

## Separate native diagnostic decision

The [unchanged independent worksheet](NATIVE-DIAGNOSTIC-REVIEW.md) specifies
exactly one possible `diagnostic107-D1` authorization: 45 minutes, at most 8 GiB
scratch, one engine build, one nonverifying companion-vstd build, six frontend
cases and two exact-call repeats. Mandatory offline dependency/support and
Git-dependent build-metadata preflight gates may prevent any build. No solver,
native application execution, primitive law, native producer, semantic adapter,
admission relaxation or runtime adoption is included. No retries are authorized
by that proposed scope. It remains a proposal until explicitly approved.

Successful future diagnostics would describe a **new compiler candidate** and
its observed call binding, not retrospectively authenticate the old frozen
compiler or establish native buffer authority/nominal codec correspondence.
The existing named receipt records a vstd-only build with an existing compiler;
it does not establish a compiler build, nor prove an Arc call was rejected.

`NATIVE-DIAGNOSTIC-REVIEW.tar.gz` retains all five original review files; every
archive member was compared byte-for-byte. [Review retention results](NATIVE-DIAGNOSTIC-REVIEW-RESULTS.json)
bind report/manifest/archive identities and the separately scoped 32-input
recheck. The reviewer explicitly discloses that not all hashes preceded first
read and does not claim an atomic snapshot or all-235 historical authentication.

All eighteen final redb cells remain **OPEN**. S0, trust, native-resource,
ordinary Infer, import/export, source-continuity and final acceptance gates
remain unchanged. Do not resume or reroute excluded tasks.
