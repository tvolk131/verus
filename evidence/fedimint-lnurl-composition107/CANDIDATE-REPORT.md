# Fedimint LNURL invoice-accounting implementation

November 7, 2026, America/Chicago. **IMPLEMENTED AND CHECKED; NOT ACCEPTED OR ADOPTED.**

## Delivered property

`composition.rs` calls the unchanged checked native fee/debit helpers and scalar
expiration encoder/decoder through two genuinely distinct providers. For every
u64 invoice amount, base fee and ppm, its debit and recovered fee reconstruct
max(invoice amount, fee), without addition overflow. There is no fee<=amount
caller premise. `accounting-sum-corollary.rs` explicitly exports that sum equality
and the exact iff condition for reconstruction of the original invoice.

Independent receiver inputs remain independent: `checked_event_projection`
proves Some iff contract<=expiration, with exact mathematical sum; otherwise None.
It is an authored checked-add observer, not a rewrite of native unchecked Add.
The native-policy adapter uses the original amount/component guards and proves
exact invoice reconstruction and contract>=49,500 msat. Some means numeric policy
acceptance only, not successful invoice/receive execution.

## Executed results

11 conservatively charged direct verifier applications; one worker; 60-second
hard limit each; no timeouts. Full raw argv, allowlisted environment, cwd,
source/import hashes, stdout/stderr, JSON scopes and statuses accompany each
attempt in `attempts.jsonl` and `<label>.*`. The ledger is never reset.

- Distinct native fee provider: 55 verification conditions, zero errors.
- Distinct expiration provider: 11 verification conditions, zero errors.
- Imported composition: 10 verified; combined local composition: 76 verified.
- Explicit full-domain sum corollary: 11 verified.
- Guarded mutant encodes net contract instead of fee: exactly two authored
  accounting postconditions fail; no overflow/frontend/resource failure.
- Independent mutant witness positively proves policy-accepted 100,000 msat
  reports 99,000 (contract=49,500; actual fee=50,500; encoded value=49,500).
  The corrected source positively proves reporting 100,000.
- Restoration is byte-identical to corrected composition and verifies again.
- False unconditional conservation at amount=0, base=1, ppm=0 is semantically
  rejected. False receiver totality at contract=u64::MAX, expiration=0 is rejected.
- One initial frontend failure called an executable helper inside assert;
  it is retained with its exact source, then corrected without provider changes.

`audit.py` checks complete inherited native body inventory, required fresh
soundness/maximality/exactness work, real certificate identities and dual-import
fingerprints, failure localization, source hashes and byte-identical restoration.
`audit.json` records the result. Preparation failures are separately retained.
Verification counts are verification conditions, not counts of new native proofs.
Authored composition/witness checks are not additional inferred native bodies.

## Authentication and limits

Entry/per-attempt/exit authentication hashes actual used source, provider tools,
runtime, compiler/sysroot libraries, native files and historical evidence.
The frozen runtime matches all nine runtime-binding hashes. Seven recovered
native files match both existing SHA256 guards and recovery-ledger Git-blob IDs;
all original selected method/declaration hashes and provider calls are preserved.
Actual provider crate identities differ, rather than relabeling existing artifacts.

**Recovered native is SOURCE-CONTENT-ONLY, not clean native continuity.** The
recovery ledger explicitly records failed original-path preservation. Twelve
current-parent source entries differ from the frozen historical source map
(including missing entries); these differences are retained in authentication
receipts. Current consumed input hashes are frozen separately. Runtime
independent_review remains OPEN. Compiler/source correspondence is NOT repaired.

Inherited Verus/SMT/compiler and vstd saturation/division/Option/checked-number
correspondences remain assumptions. No new assume/admit/axiom/external-body,
feature, trust model, engine/runtime/vstd build, native application execution,
network access, Git write or parent edit is introduced. The providers' existing
constant-division and constructor preconditions remain unchanged; the total
composition adds no caller requirements.

Producer-origin transport equality and the receiver's non-manual fallback branch
remain conditional. No crypto, serialization, async, database, consensus, crash,
funds-safety, liveness, clean-checkout or full-native caller theorem is claimed.
All 18 redb cells remain OPEN. Excluded lanes are not resumed. No acceptance/adoption.

See `exit-comparison.json`, `SEAL.json`, and `SHA256SUMS` for final time/storage
accounting and immutable input comparison. The content-addressed archive includes
the exact implementation patch, sources, every verifier failure and all receipts.
