# F1 baseline: unsupported; full component unproved

**Scope checkpoint: DEFER F1.** Accepted f4415999 rejects the original `u64::saturating_div` call at pinned gateway_api.rs:277 before backend proof checking. This is a measured frontend boundary, not a failed arithmetic theorem, safety counterexample or solver resource failure. Ordinary and `--infer-goals` agree. No common engineering layer is proposed. F0 and F1 remain two attempted/inspected Fedimint cases, zero successful whole-component proofs; F0 stays unsupported for the previously recorded map APIs. No replacement selected.

## Property, source, domain and fidelity

The frozen selected property is the full PaymentFee closure: exact componentwise limits, exact checked component sums/overflow rejection, original nested saturated fee calculation, dominance under componentwise limits, saturated send/receive accounting and exact nonsaturating equations. All u64 base/ppm/payment inputs remain admitted, including 0/MAX. Only the conditional dominance theorem assumes within(a,b). The source-connected composed property is in component.rs; it has **not been verified**.

Original source pin b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236, every source hash/byte count in parent SELECTION.json checked, all eight accepted runtime hashes and receipt checked, tools copied to private directories. Eight selected method signatures/bodies retained byte for byte, including saturating_div. Genuine named Amount and original relevant runtime derives retained. Source caller connections: lnv2-client lib.rs:717 send_fee.add_to(amount), :1115 receive_fee.subtract_from(amount.msats); gateway lib.rs:2627 checked_add before :2635/:2643 limit guards. Async/database/crypto/whole payment state is outside scope. Freeze/adaptation/omitted-trait details: FREEZE.md and fidelity.json.

Ordered commits before trials: e28d6715 froze executable projection, full-domain specifications and intended theorem; b573df7e preregistered diagnostics before their runs. No semantic policy, body or input-domain change followed the freeze. The full_plain_derive diagnostic changes only one verifier annotation; the absolute_fee_only diagnostic selects unchanged methods to isolate the failure, never a replacement success case.

## Ordinary versus infer measurement

| Frozen/diagnostic input | Ordinary | --infer-goals | Body/precision/publication result |
|---|---|---|---|
| Frozen full component, Amount infer_derive | Frontend macro rejection, 0 verified / 0 semantic errors | Same | Native collection false; 0 checked bodies, helper identities [], no interface |
| Full component, ordinary derives | Unsupported original saturating_div | Same | Native collection true, 0 checked exec / 0 checked proof bodies; complete scope false; helper identities []; no interface |
| Original absolute_fee + Amount dependencies only | Unsupported original saturating_div | Same | Same; no backend verification or AIR work |

Full/plain and absolute-only runs omit numeric verified/errors fields in their frontend JSON; these must not be invented as a “0 failed proof” result. The reported checked body counts are explicitly zero. All application commands use --no-cheating, edition2024, Rust1.99.1, --compile, and no lifetime/trait/WF/mode/erasure suppression. Compilation does not finish because the frontend rejects the call. No claim that subsequent ordinary checks passed.

Measured baseline process wall times: frozen ordinary/infer 0.052/0.048 seconds; full ordinary-derive 0.747/0.717 seconds; absolute-only 0.735/0.743 seconds. These are single observations, not cold/warm performance claims. The final optimized-Python focused replay took approximately 0.8 seconds per mode. Setup/hash-copy time is outside those command timings. No compiler or vstd build, native functional comparison, solver trial on a substitute algorithm or large test matrix was run. Eligibility investigation stopped well before45 active minutes, with no build waits or prerequisite work.

Raw logs, privately copied sources/binaries and receipts: /tmp/fedimint-f1-baseline-1fwb3g_v/{trials2,plain,absolute,validated-final}/. BASELINE.json binds the outcomes and log hashes. First `trials/` invocation failed to locate rustup on PATH and is excluded from proof evidence. An intermediate validator replay incorrectly expected native collection false for all frontend failures; it failed explicitly (success=false). Corrected validation distinguishes completed native collection from zero checked bodies. The final replay passed under Python -B -O using explicit require exceptions (then an additional frozen whole-fixture SHA gate was checked offline; no compiler command or proof input changed); prior six baseline results were also validated offline under -B -O with the same validator. Receipt success means the expected unsupported outcome was reproduced, never application-proof success.

## Authored semantics and literal mechanical burden

Authored semantic inputs remain six spec functions, eight method postconditions, one dominance theorem contract and twelve composed-property assertions. Saturation ordering is explicit: product clamp, integer division by one million, then base-add clamp. No numeric authority substituted for runtime Amount ordering.

Before and after are identical: one local proof function with an empty body, one proof block and one call to policy_dominance; zero implemented arithmetic-helper proof steps, zero saturation case splits, zero mechanical assertions inside original methods, zero loop invariants, zero snapshots. There are eight semantic contract annotations, one attempted infer_derive annotation and three plain verus_verify annotations. **Inventory correction:** FREEZE.md's “0 manual lemma calls” and “four verus_verify annotations” were inaccurate; count the existing single theorem call as manual glue and count three plain annotations plus the derive annotation. No proof burden reduction occurred. The empty proof lemma is an outstanding obligation, not an admitted/implemented manual proof. There is no feasible explicit ordinary proof of the selected original closure using this accepted runtime under the authorized boundaries.

Already automatic: accepted num.rs has specifications for u64 saturating_add/sub/mul and checked_add. Availability is source-inspection evidence; no standalone subset was rerun or counted as a new success. Actual inferred helper identities: **[]**, exactness/maximality/soundness helper checks: **none reached**. No new automation contribution.

The freeze initially incorrectly claimed infer_derive checked this named Amount. Measured macro rejection shows it supports only the allowed tuple shape. In ordinary-derive diagnostic inventories Amount::partial_cmp/cmp/eq are outside selected scope; they supply no checked numeric observation. Division masks any subsequent runtime comparison contract failure. There is no claim that is_within is proved or that derived comparisons are healthy. The preregistered policy_subset was not run after the stop decision and is not exposed as a continuation in the final replay.

## Value checkpoint and stopping decision

The immediate typed eligibility gap is the actual call `u64::saturating_div(receiver: u64, rhs: u64) -> u64`, where receiver is the result of `payment.saturating_mul(ppm)` and rhs is the literal1_000_000. Intended observation is `result == receiver / 1_000_000` on every u64 receiver, with the original API/body retained. The compiler explicitly reports this callee unsupported. Adding the suggested assume_specification would violate this assignment; it was not done. No new foreign-MIR, Rust semantic or representation project started.

If the original dependencies were supported, the proposed mechanical demand would be normalization of u64 saturation and int min/max bounds plus monotonicity `cap(p*a)/1_000_000 <= cap(p*b)/1_000_000` for nonnegative p and 0<=a<=b, lifted through genuine Amount calls. Existing lemma_mul_inequality and lemma_div_is_ordered could be ordinary manual steps, still charged as mechanical calls. Shared min/max/range normalization might help REDB R1, but this run never reaches those obligations; no cross-project causal need or usefulness is established. The named-Amount derive adapter is a separate restricted-profile issue, not evidence for a common REDB fix.

Stopping condition met: original executable dependency unsupported under accepted models, so F1 is deferred with its entire selected property unachieved. Parent must decide any replacement before trials. No production edit is justified from this lane. No property-directed negative controls were attempted because there is no healthy positive composed proof; frontend rejection is not a semantic negative control. Ordinary caller/import/cold-warm checks are deferred for the same reason. No checks were disabled to manufacture a positive.

HMAC/native/Miri/Coq obligations remain **DEFERRED AND UNACHIEVED**, outside the prerequisites. No subagents, network, push/PR, engine edits, vstd edits, original application edits or sibling checkout/process/cache inspection occurred.

Reproduce the bounded failure (fresh private output automatically allocated):

```sh
python3 -B -O source/tools/research/practical_redb_fedimint/fedimint/baseline.py --scenario absolute_fee_only
```

`--scenario frozen` and `--scenario full_plain_derive` reproduce the two other preregistered inputs. Replay depends only on the authorized pinned source/runtime paths, Rust toolchain and this checkout's existing infer_experiment/infer_eval_support utilities.
