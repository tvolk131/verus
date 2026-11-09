# Independent corrected-contract review — November 7, 2026

**Disposition: conditionally coherent outline; canonical bindings NOT READY; native engine/gate NOT ESTABLISHED. NOT ADOPTED.** New independent T2 review, S0 retained; no implementation authority or resumed task. I did not author the amendment. Assumptions adopted, new proof credit and new trust: zero. All 18 obligations **OPEN**.

## Findings first

1. **Admission blocker, not abstract contradiction: bindings remain unresolved.** D:613/D:636 have null canonical bindings. All 60 inspected CI rows have null schemas and unchecked joins (CI:73/CI:96); CI:94 retains cleanup. They cover mem, mem_mut and the older caller, not prospective A3. M:120 acknowledges this. **Minimum:** mark every missing typed instance/schema/profile/glue edge UNRESOLVED; names, DefPathHashes and metadata hashes cannot certify it or the missing matching library implementation.

2. **A3 conservation remains schematic.** M:109 repairs scope/counters/glue; M:58/M:139 require but do not instantiate its frame. **Minimum:** clone S'=S+{e}, W'=W; downgrade S'=S, W'=W+{w}; extra-strong retirement S'=S-{e}, W'=W; extra-Weak retirement S'=S, W'=W-{w}. Require fresh e/w and legal actual-handle access; retain h->C->A, bytes/extent/generation, implicit weak and unrelated resources. Retire only the extra, never h. Conserve byte authority with checked suspension/eligibility: clone grants no exclusive token; count retirement alone closes no loan/restores no authority. This is schema precision, not a claim the outline endorses corruption.

3. **Total outcomes are corrected; terminal Abort still needs its runtime domain.** M:58 excludes always-aborting/diverging success models. For O1(None), invalid ranges/lengths and false assertions, entry into panic or no normal successor is not yet finite terminal completion. M:108/M:136 retain formatting/cleanup. **Minimum:** cover all pre-terminal runtime state/edges, including any reachable hook/callback capable of divergence/reentry, or state a dischargeable exclusion premise; otherwise terminal completion stays OPEN. This is a conditional challenge, not a discovered pinned-library bug. `panic=abort` alone proves no abort frame, destructor completion or recovery theorem.

4. **Fixture closure is repaired in prose, not canonical sidecars.** M:139/M:149/M:153 supply both sources, actual owner roles, BASE inheritance and no live-alias normal suffix. Historical F:297/F:390/F:818 still unconditionally label a returned read LIVE, even for abort/borrow rejection; C:13 retains singular `source` for twice. **Minimum:** a fresh successor schema must outcome-index returns, bind BOTH sources and use capture-avoiding typed owner/frame maps. Preserve old artifacts. Short original `requires` lists are additional concerns, not closure.

5. **Debug labels do not fix arithmetic semantics.** M:146 requires nonwrapping arithmetic; F:30/C:20 name debug variants, and K:171 records `debug_assertions`, not overflow settings there. **Minimum:** bind actual overflow-check/codegen settings and original arithmetic/assert CFG, proving `start <= usize::MAX-8`, `start+8 <= n` and absolute composition in each profile. Do not infer checked overflow/wrapping from the debug label. No actual flag mismatch/run is alleged.

Missing engine implementation alone is **not** an abstract-law defect. Conversely, promises to check E1–E6 are not completed checks.

## Exact anchor legend and review extent

Anchors are `alias:line`, expanding these exact path prefixes (not shell variables):

- W = `/Users/tommyvolk/.t3/worktrees/verus/t3-60978068`
- Q = `/private/tmp/verus-native-slice-primitive-proposal043-0ywdS3`
- U = `/private/tmp/verus-native-proposal043-semantic047-YqMgfR0y`
- T = `/private/tmp/verus-native-slice-testprep042-I01FH1HU`
- G = W/`evidence/redb-crash-consistency94/GOAL.md`; M = W/`evidence/redb-crash-consistency94/NATIVE-RESOURCE-DESIGN-AMENDMENT.md`
- P = Q/`PROPOSAL.md`; CI = Q/`CALL-INVENTORY.json`; R = U/`REPORT.md`; H = U/`HANDOFF.md`
- RC = T/`RESOURCE-CONTRACT.md`; F = T/`expected-phase-matrix.json`; C = T/`PROPOSED-CLAUSES.json`; D = T/`dependency-manifest.json`
- B = T/`callers.inc.rs`; BC = T/`borrow-check-only.inc.rs`
- K = `/private/tmp/verus-native-slice-implementation042-spATdMar/handoff/compiler/source/rust_verify/src/native_resource_effects.rs`
- V = W/`source/rust_verify/src/verifier.rs`; IE = W/`source/rust_verify/src/import_export.rs`
- I = W/`source/vir/src/infer.rs`; RI = W/`source/vir/src/resolution_inference.rs`

Read-only scope: named inputs, bodies, original obligation lists, selected binding evidence, minimum gate and targeted sources. Prior audits are not recertified; K is historical, not current-parent proof.

## Independent per-law assessment

| Law and anchors | Challenge and disposition |
|---|---|
| A1 — M:70; P:64; RC:101 | Some iff S={h}, W={} requires complete stable inventories, exclusive handle access, success-capable byte authority and rely. Extra strong/explicit Weak normally yields None without requiring exclusive bytes. Missing premises/bindings are refusal, not None. Preserve h/C/A and bytes; lend, never duplicate. Conditional outline coherent; library truth unresolved. |
| A2 — M:79; P:87 | Actual shared permission, not mapping/value deref alone; same A/extent/generation read loan transported through mem and LIVE beyond callee return. Instantiate nonmutation/frame. |
| O1 — M:83; P:113 | Some normally transports its unique contained lens once, no owner creation. None needs terminal completion (finding 3), not vacuity. A helper success-only precondition cannot turn alias controls into frontend refusals. |
| S1 — M:88; P:127 | Relative 0<=s<=e<=length plus representable in-allocation absolute composition; suspend ancestors. Empty/end lenses retain origin/generation/reference obligations, not fresh ownership/pointer inequality. Invalid ranges fail via actual path; len/unsizing allocate nothing. |
| S2 — M:96; P:136 | Initialized source pre-call bytes -> exact exclusive destination; frame complement/resources. Zero length is an empty update, not validity/permission exemption; nonoverlap is not unequal addresses. Temporary borrow ends differ from underlying authority. Gate sources are disjoint from WHOLE target A (M:141); same-allocation split-copy excluded. |
| S3 — M:104; P:147 | Length/byte equality through live reads, never owner equality; bind actual shared/mutable-reference wrappers. True assertions normally continue, false require finding 3's closure. Sidecar negatives are not runtime asserts. S3 only where actually used. |
| A3 — M:109; P:95; RC:110 | Four bounded extra-handle cases; checked access/counts, fresh identities, one-shot exact nonfinal retirement with h live and implicit weak intact. Include mem::drop, actual destructors/flags/recursive glue or exact reviewed encapsulation. Finding 2's frame remains schematic. No generic drop, final-h/payload/deallocation or page Drop law. |

No matching core/alloc implementation theorem was checked. Under `/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/lib/rustlib/src/rust/library/`, `alloc/src/sync.rs` and `core/src/slice/mod.rs` were independently found missing; D:638/R:212 report missing sources too. No substitute was consulted. Metadata certifies neither A1 iff, A3 glue nor termination.

## Inventory, authority, identity and premise ownership

M:26/M:33/M:42/M:49 repair P:29/P:46:

- Immutable D contains evolving H0/H1/H2. Byte writes preserve allocation/control-block/generation identities; equal bytes/reused addresses identify nothing. Places include body, activation, projections and init generation. Moves transport identities, not clone/reinitialize them.
- S/W cover ALL contributions with identities/locations/tokens, not accessible locals only. Exclude or account raw, forgotten/leaked, escaped/untracked and FFI histories. Exclusion is caller-owned, not inferred from Arc. Implicit weak remains separate while h lives.
- Accounting is not byte authority: None can account extras without exclusive bytes. Shared/exclusive loans and suspended ancestors remain independent; handle retirement alone discharges no loan.
- Rely excludes unauthorized concurrent/reentrant clone/downgrade/upgrade/retirement, raw/incompatible access for each loan's duration. Sequential authorized A3 remains allowed. Its absent capability/environment witness cannot circularly assume get_mut success.
- Page validity, source read/liveness/disjointness, frame authority, bounds and opaque fields are caller premises. Instantiate BOTH twice sources (compatible sharing between them, not with A). Owner_pair needs both inventories, source disjoint from both pages, right/B update, left/A frame, both buffer/offsets and F:179's unswapped discriminator.

These are coherent restrictions, not constructor/checkout or outer-caller proofs. Premises must be visible and satisfiable, not contradictory/vacuous.

## Review047 F1–F7 disposition matrix

“Resolved in text” is outline-level correction, NOT an admitted contract/check. Open design/binding issues and implementation evidence are separate columns.

| Prior finding / exact original anchor | Resolved in text — amendment anchors | Still open | Implementation evidence needed |
|---|---|---|---|
| F1, R:13; P:64/P:113 | M:58/M:70/M:83/M:88/M:96/M:104: normal termination/failure partition; M:63: no abort frame. | Exact outcome/resource schemas; terminal runtime domain. | Checked normal/None/abort outcomes, not ensures vacuity. |
| F2, R:39; P:46 | M:33/M:42/M:49: complete inventories, implicit weak, separate authority, rely and sequential A3. | Formal predicates/noncircular discharge. | Each caller, both pages/sources; no inferred crate invariant. |
| F3, R:76; P:156 | M:132/M:133/M:134/M:135: engineering, joins, actual NLL, not HasResolved. | Checked representation/correspondence absent, not new abstract contradiction. | E1–E4, same-D/H1 freshness, affine rejection, live read. RI:6 supports value/lifetime distinction. |
| F4, R:112; P:127/P:136 | M:90/M:94/M:99/M:139/M:146: empty/zero, bounds, frames, both sources, actual roles. | Typed substitutions and actual arithmetic/profile schema. | Bounds/frame VCs; empty/end/reversed schema cases beyond fixed-eight fixtures. |
| F5, R:155; P:95 | M:109: bounded A3/counters/exact glue; no final/page destruction. | Explicit conservation (finding 2); canonical/glue keys. | Four alias controls + BASE, both profiles; no live-case normal suffix. |
| F6, R:186; P:186; H:17 | M:149/M:176/M:137/M:185: BASE, **S0**, barriers; S1 unchosen. | All-native closure/invalidation, not selected export permission. | E5/E6 and regressions. Isolated work remains internal/unadmitted under S0. |
| F7, R:207; P:194/P:236 | M:10/M:120/M:190: provenance != semantics, no repaired history. | Matching sources, canonical schemas, source/runtime correspondence. | Correspondence evidence where claimed, or separate explicit TCB decision; neither follows from hashes. |

## E1–E6 are all OPEN checked engineering

| Item / amendment anchor | Required checked evidence, not an external assumption |
|---|---|
| E1 / M:132 | Affine VIR/SST/AIR channel through all transformations; freshness/generations, overlap/conservation/frames. No finite oracle or domain reset. |
| E2 / M:133 | Original body/CFG/profile -> GHOST/EXEC/erasure occurrences; evaluated projections/arguments/results/successors once, preserving reservation/activation and Rust borrow checking. |
| E3 / M:134 | Checked original methods/helpers/callers; exact returned subtree across get_mut->unwrap->mem_mut->caller. Fresh activation 2 in D consumes H1. No application axiom. |
| E4 / M:135 | Actual NLL/CFG ends and descendant closure on relevant successors. Atomically reject stale/consumed/foreign-domain/wrong generation/owner/body/place/interval witnesses. Permit checked cross-body mapping; returned mem read stays live. |
| E5 / M:136 | Joins/partial init, normal/None/abort/cleanup/glue/drop flags/arithmetic and indirect/transitive effects; panic flag erases no captured edge. Unknown reachable effects refuse. |
| E6 / M:137 | S0, checked-body or reviewed-bound-primitive dependencies; capture is neither. Preserve default-OFF, labels and full-scope/import/export; no S1 adoption. |

K:173/K:174 report unimplemented lowering/capture-only erasure join. Current guards were inspected at V:2923/V:2957 (full-scope), IE:135 (import), IE:275/IE:296 (export), I:249 (publication). Presence is not all-path/regression or resource-engine proof.

## Complete 17-fixture closure in BOTH profiles

JSON inspection: exactly 17 IDs x debug-OFF/debug-ON = **34 prospective cells**, not proofs: per profile 14 backend-VC, 2 resource-VC-or-checked-abort, 1 borrow-only. Historical status is UNRUN/null VC counts. Later reported typecheck/borrow evidence is not erased/replayed (M:160).

Every resource row inherits outcome-relevant **BASE**: checked original methods/helpers/callers, A1/A2/O1/S1/S2, S3 where actually used, E1–E6. Four alias cases add A3. Only checked outcome/reachability excludes a suffix; BC_CONFLICT earns no resource-negative credit.

| Fixture / exact F anchor | Body / source anchor and required claim |
|---|---|
| P_SINGLE / F:8 | B:12/B:2: normal exact A[8,16) write; complement/B/source/opaque frame, original h; returned mem read LIVE. |
| P_TWICE / F:57 | B:20: calls at 8 then 16; n>=24, BOTH source premises, H1->H2, first update retained, fresh activation and current witness. |
| P_OWNER / F:107 | B:30: right/B is updated; left/A framed despite equal entry bytes; both actual inventories and two live returned observations. |
| N_OWNER / F:157 | B:30: reject B lens rebound to A; F:179 discriminates wrong bytes. Owner mismatch must fail independently of byte coincidence. |
| P_STRONG_RETIRED / F:207 | B:49: clone exact e, drop e BEFORE helper; prove 1->2->1 and full BASE normal/frame result. |
| N_STRONG_LIVE / F:254 | B:39: e live at helper; fail requested normal/Some VC or prove None->terminal abort. B:45/B:46 are not normal successors. |
| P_WEAK_RETIRED / F:300 | B:69: downgrade w, exact Weak retirement BEFORE helper; explicit 0->1->0, implicit obligation retained, BASE normal/frame result. |
| N_WEAK_LIVE / F:347 | B:59: explicit w live; fail requested normal/Some or prove None/abort, with no normal B:65/B:66 suffix. |
| P_CLOSED / F:393 | B:79: close exclusive root/child subtree only after B:86's second copy at genuine NLL end; B:87's shared result remains LIVE. |
| N_LIVE_CLOSURE / F:441 | Same B:79: request closure between B:85 and B:86; intended backend loan-end claim must fail, not an unrelated borrow/frontend check. |
| N_STALE_ACTIVATION / F:487 | B:20/B:2: SECOND_END atomically rejects consumed activation-1 witness for activation 2; current-witness positive succeeds. |
| N_CROSS_STATE / F:535 | Twice: atomically reject foreign D despite matching numbers; legitimate same-D H1/H2 are not foreign. |
| N_CROSS_BODY / F:583 | Twice: atomically reject unmapped foreign body/instance/place; permit checked callee-return transport. |
| P_FRAME / F:631 | B:12: A complement, separately authorized equal-valued B, source, h/C, extent/generation, page.buffer/offset and opaque resources. |
| N_NEIGHBOR_FRAME / F:679 | Same B:12: reject H1[A][7]=source[0] under source[0]!=H0[A][7]; intended interval frame failure. |
| N_OTHER_FRAME / F:727 | Same B:12: reject H1[B][8]=source[0] under source[0]!=H0[B][8] and B!=A; intended allocation frame failure. |
| BC_CONFLICT / F:775 | BC:8/BC:9: real conflicting mem before later child use; Rust rejection BEFORE resource VCs, no semantic-negative credit. |

F:821 retains matched source/runtime/sysroot/cfg/profile/closure, satisfiable normal premises, nonvacuous abort and no whole-native label. F:828 retains default-OFF/ON pass/fail partners; effects/import/export/unhandled-primitive and source/cfg/metadata mismatch refusals; original borrow/erasure checks; nonempty per-body resource inventories AND checked results in both profiles. M:149 removes none. None is certified satisfied here.

## Three separate verdicts and trust classification

1. **Abstract design:** no counterexample found to the corrected conditional outline with its full premises. Major textual objections are substantially repaired; findings 2–5 remain schema precision. Neither formal consistency nor a completed contract check is certified.
2. **Canonical bindings: NOT READY.** Need typed resolved Instance/substitutions/owner environment, retained lifetime/borrow correspondence, exact library/build/schema versions, target/ABI/cfg/debug/overflow/panic/compiler/runtime identities and occurrence/glue closure. D:636 remains null; old rows bind no prospective A3. Matching library bodies remain unresolved. No external-law adoption occurs.
3. **Actual engine/gate: NOT ESTABLISHED; zero new proof credit.** E1–E6, per-profile results and all-native closure remain OPEN. **S0** and full-scope/import/export barriers stay. Internal evidence is not an admitted native theorem, Infer certificate or database acceptance.

**External candidates:** exact outcome-complete A1/A2/O1/S1–S3/A3 library/runtime/glue laws, including any encapsulated terminal closure: proposed TCB, not proved core/alloc. **Caller premises:** valid incoming page/handle/allocation, complete histories/inventories, exclusion/rely, byte/loan/frame capabilities, BOTH actual sources/bounds, any explicit runtime-state restriction. Entry assertions discharge no outer caller. **Still-required theorems/checks:** E1–E6, actual methods/helpers/caller outcomes/frames, constructor/checkout premise establishment, whole-native closure and database properties. Engineering/application summaries cannot become external axioms. **No assumptions are adopted.**

## Smallest next increment and decision

Smallest useful **future, separately authorized** increment: one fresh, nonexecuting/uninstalled A3 worksheet with four bounded transitions, finding 2's conservation, outcome-indexed runtime domain, BASE references and separate profile/overflow/glue-key slots. Missing typed/source bindings stay UNRESOLVED, never synthesized from names/hashes. This draft needs no build/run/trust installation or old-evidence edit and earns no theorem credit.

**Next decision:** authorize only that bounded drafting/review, retaining S0? TCB adoption, source/binding acquisition beyond this read-only authority, implementation/build/version allowances and alternative scope policy each require separate decisions. No follow-on task or allowance reset starts here.

## All 18 final obligations explicitly OPEN

Pinned native redb source: `c82ab865bb362bc4af067d332adc44978ff64813` (G:10). The six properties cross the three history modes; no helper review discharges a cell.

| Property | One-phase | Two-phase | Mixed-mode |
|---|---|---|---|
| Crash atomicity | OPEN | OPEN | OPEN |
| Acknowledged durability | OPEN | OPEN | OPEN |
| Preservation of recoverable state | OPEN | OPEN | OPEN |
| Recovery correctness, including interrupted recovery | OPEN | OPEN | OPEN |
| Error correctness | OPEN | OPEN | OPEN |
| Logical key/value refinement | OPEN | OPEN | OPEN |

Native correspondence, trusted-boundary review and independent final acceptance also remain undischarged. Page construction/Drop, cache/locks, storage/publication/commit/recovery are not proved by this gate outline (M:190; G:16).

## Preservation and accounting

Only this fresh-directory report is written. Parent/old artifacts/proof source/Git state are unchanged by this lane. All nine primary input hashes matched on exit; no prior full-inventory audit was renewed. Inherited missing-native/provenance/object-store qualifications persist (P:236; R:437).

Builds (compiler/vstd/Cargo/harness), verifier/solver runs, native executions, proof/Rust edits, network, Git operations/writes, retries, subdelegation and resets: **all 0**. No old helper executed, toolchain copied, or refused diagnostic/task inspected/resumed/rerouted. JSON/hash inspection is not proof.

Free-space samples: 139,287,020 KiB entry, 138,252,112 prewrite, 136,413,952 near exit: all >16 GiB. Only this <=20 KiB report is retained; no global filesystem/ref certification.

**STOP — review handoff only. All 18 OPEN. Assumptions adopted 0. Trust installed 0.**
