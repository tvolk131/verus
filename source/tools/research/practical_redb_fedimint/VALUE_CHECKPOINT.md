# Value checkpoint 1: inspect existing mechanisms before adding a layer

This is source inspection on the accepted baseline, not a solver outcome, new feature, or proof. R1/F1 remain frozen and unproved; the independent workers must measure the actual frontend/ordinary/inference boundary. No engine/vstd edit is approved at this checkpoint.

| Mechanism | Existing accepted source | Meaning for selected cases |
|---|---|---|
| Checked and saturating unsigned arithmetic | `source/vir/src/infer/integer_models.rs`, `source/vstd/std_specs/num.rs` | Existing explicit models cover checked arithmetic and saturating add/sub/mul. Do not claim reusing these is a new automation improvement. |
| Potential API boundary | Same files | No accepted `div_ceil` specification was found for R1 `required_words`, nor `saturating_div` for F1 `absolute_fee`; saturating inference deliberately accepts only add/sub/mul. Fresh baseline must confirm actual behavior. Do not substitute `/` or a ceiling formula into original executable code. |
| Native-range loop assistance | `source/vir/src/infer/range_goals.rs` | Already preserves source loop/frontend scaffolding while proposing checked prefix invariants. R1 should first establish whether it applies before proposing another loop recognizer. No need for native IterMut/Rev semantics in its direct-index loop. |
| Checked bitvector interval hints | `source/vir/src/infer/bitvector_hints.rs` | Proposes independently checked pure scalar facts; excludes arbitrary calls, sequence access, and non-pre snapshots from BV reinterpretation. It does not itself establish state frames or supply an unknown library contract. |
| Packing interpretation | `source/vir/src/infer/packing_goals.rs` | Bounded shift/or equations with ordinary BV checks. Existing packed-field results do not establish saturation monotonicity or arbitrary bitmap resize. |
| Word-view assistance | `source/vir/src/infer/word_goals.rs` | Closed existing OR-member/OR-view/full-view laws. R1 clear/interval updates may need different mechanical work; this alone does not show a shared benefit with F1. |
| Ordinary checked arithmetic lemmas | `source/vstd/arithmetic/mul.rs`, `source/vstd/arithmetic/div_mod.rs` | Existing `lemma_mul_inequality`, `lemma_div_is_ordered`, fundamental quotient/remainder and remainder-bound laws are candidates for explicit ordinary diagnostic proofs. Calls/case splits remain mechanical glue and must be counted. |

## Decision

First prioritize an exact accepted-runtime eligibility/contract baseline for the native calls in R1 and F1. If an absent call contract blocks the original property, record the unsupported boundary and stop that case within45 active minutes; do not widen into new std assumptions, native/core-MIR certification, semantics or representation machinery. Any replacement needs a fresh source/property registration before trials and preserves the unsuccessful case in coverage.

Only after a full-domain ordinary proof and baseline evidence identify a concrete common need in both actual components should the integrator authorize a generic checked saturation/range/observation layer. The value question is which application property gets closer and which literal local proof calls/invariants/blocks disappear. A shared layer that helps only a synthetic formula, an already automatic helper, or one project does not satisfy the goal.

The original HMAC/native objective remains deferred and unachieved.
