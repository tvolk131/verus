# Consolidated native bitmap proof campaign

Status: FROZEN FOR IMPLEMENTATION, not proof or acceptance evidence.
Authorized November 4, 2026. This resumes the existing incomplete bitmap goal;
it does not turn earlier PARTIAL / NO-GO verdicts into successes.

## Baseline and ownership

Start with the accepted publication86 source: 2,217 nonignored paths, authenticated
before editing. The historical source-map file has SHA-256
`02cad9ff88c33d31284dd07643b5272f487c677972dba274ae1f8bde4c679521`.
The accepted compiler has SHA-256
`a89fd51fd6a22239ddce5957a2b01e8400214a06836631fffbae8cde5b7637d9`.
These identify the starting point, not a manually selected producer for new builds.
Runtime producer identities must be derived from the actual build source manifest.

The native redb pin is `c82ab865bb362bc4af067d332adc44978ff64813`.
Its `src/tree_store/page_store/bitmap.rs` has SHA-256
`d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff`.
The native Fedimint pin remains `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.
Preserve both native checkouts, the 1,401 original immutable inputs, unrelated
dirty source, and sealed historical packages. No commits, manual branch/ref
writes, pushes, network, or shared Cargo/toolchain writes are authorized.

## Combined deliverable

1. A permanent, documented, offline command builds a source-bound runtime,
   verifies matched vstd, audits the pinned native source, and runs permanent
   source-connected proof fixtures and discriminating controls. It must not
   depend on historical temporary packages or hand-selected producer strings.
2. A complete checked proof of unchanged native `U64GroupedBitmap::resize`,
   with the actual `bits_in_range`, `required_words`, `data_index_of`,
   `select_mask`, `len`, `get`, `set`, and `clear` bodies and native struct.
3. If a checked reference exposes one clear reusable mechanical improvement,
   implement and independently review at most one narrow Infer change.
   A complete explicit proof is otherwise acceptable; automation credit is
   separate and requires a discriminating before/after demonstration.

## Proof and domain

Keep the original algorithm, native helper calls, inclusive loop, mutation,
arithmetic, and assertions. Erased annotation auditing must establish source
correspondence; a rewritten model or loop is not the native proof.

Allow every `u32` logical length, every word value, and arbitrary valid spare
storage: `len <= 64 * data.len()`. Do not require minimal storage, zero padding,
one word, a bounded number of words, or convenient old/new length ordering.
Operation premises retain the actual safe index/shift domains. Prove:

- The new logical length is exactly `new_len`, and representation stays valid.
- The stored word count is `max(old_word_count, ceil(new_len / 64))`.
- Growth initializes exactly the exposed logical interval to `full`.
- All other previously stored bits, including padding and spare words, survive.
- Newly allocated words have exactly the native zero/all-one initialization,
  including their padding; shrink does not truncate or clear stored words.
- Actual reads/updates compose with the result, including shrink then regrow.

Use mathematical integers for specification arithmetic; do not add artificial
nonoverflow assumptions that exclude safe native inputs. Query totals alone
cannot establish complete executable/proof body coverage or a usable export.

## Explicit author allowance

Unlike the historical strict83 zero-lemma experiment, this renewed campaign
allows authored semantic contracts, invariants, ghost snapshots, checked mask
lemmas, and checked local proof blocks. This relaxation is explicit: the user
approved the combined proposal allowing a checked reference proof. Inventory
all such work; do not relabel it inference. Keep native executable bytes and
assertions unchanged. Existing checked entry-forall assistance may be used.

No `assume`, `admit`, application axioms, new trusted specification family,
external-body application summary, replaced native helper, or weakened theorem.
Existing vstd/native correspondences retain their documented trust boundary;
vstd's intentional library assumptions are not application proof credit.
Ordinary imports are not silently upgraded to certified inference imports.

## Budget and stop rule

At most two implementation/review cycles, 160 direct application verifier
launches across implementers, parent, and reviewers, four compiler builds, and
two matched-vstd verifications. Count launcher/setup failures and disclose
timeouts; test-suite invocations are reported separately, not added as native
proof credit. Initially allocate 24 direct launches to reference diagnosis,
24 to consolidation, and retain the remainder for any justified change,
composition, and independent review. No automatic third cycle or scope growth.

Stop with an honest incomplete goal if the complete native proof is blocked
within this budget. Deliver useful consolidation separately, but never replace
resize completion with transition/components, a reviewed blocker, or a budget
limit. Historical strict83's 20 open rows remain historical unless separately
discharged; this renewed scope does not retrospectively close them.

## Acceptance rows

| ID | Required result |
| --- | --- |
| C1 | Source-only, documented offline build/replay; source/runtime/native binding |
| C2 | Permanent native-connected fixtures and integrated malformed-evidence tests |
| P1 | Complete nine-body native proof; no failed/omitted executable scope |
| P2 | All frozen domains and exact stored-bit/padding/frame properties |
| P3 | Actual get/set/clear and shrink/regrow composition; honest import trust |
| N1 | Semantic, safety, frame, snapshot, and publication controls reject |
| A1 | Author burden accounted; any new automation independently checked |
| R1 | Existing relevant Infer, redb, Fedimint, and publication regressions |
| R2 | New independent review, repaired findings, exact source preservation |

All rows start OPEN. A passing transition fixture closes neither P1 nor P2.
