# Native-resource schema worksheet — November 7, 2026

**DRAFT / UNREVIEWED / UNINSTALLED. All 18 final obligations OPEN.**
This parent-authored, nonexecuting clarification reconciles
`NATIVE-RESOURCE-CONTRACT-REVIEW.md` findings 1–5 with
`NATIVE-RESOURCE-SOURCE-REVIEW.md` SRC-1–SRC-4. It does not amend the frozen
proposal, either independent report, or historical fixtures. The independent
reviewers have not reviewed this worksheet. No law, progress premise, runtime
assumption, engine implementation, or scope change is authorized or installed
by writing it. Retain S0 and all fail-closed Infer/import/export barriers.

## 1. Stable-boundary domain, not an assumption about all Arcs

Retain the amendment's immutable D, evolving heaps, typed places, allocation
generation, complete inventories, separate byte authority and caller-owned
interference rely. At an ordinary live block's stable boundary:

- S is the complete strong-contribution inventory; W is the complete explicit
  Weak inventory. Each identity has one accounted location/token.
- Raw strong count equals |S|; raw weak count equals |W| + 1. The additional
  one is a collective implicit weak while a strong owner remains, not one
  implicit weak per strong handle. Original h belongs to S throughout A3.
- A1's internal uniqueness lock temporarily replaces raw weak 1 with a
  sentinel. That is not a Weak identity or a stable inventory state. A1 may
  write control-block counters transiently, including on an extra-strong None
  path. Stable inventory/data-byte preservation is not a no-counter-write law.
- An immortal static contribution from an empty default Arc must be included
  in S or that origin excluded by a caller-discharged premise. An empty slice
  does not imply uniqueness. A dangling Weak::new is not a token for live C.
- Payload, allocator, reference/range/coercion and const substitutions must be
  the actual resolved ones. Global is a possible narrow allocator binding, not
  something inferred from an Arc spelling or silently substituted for an
  unknown allocator. Generated allocator Clone/drop glue remains to be bound.

These are proposed domain obligations, not newly proved constructor facts.
Raw/leaked/forgotten/FFI/untracked histories remain excluded or fully accounted;
observation of a count is not a complete-inventory or rely witness.

## 2. Four A3 transitions with explicit conservation

Each row below describes the requested normal result only after its actual
instance, lawful handle access, complete inventory, progress, allocator/glue
closure, counter bounds and resource premises are established. It supplies no
total-return theorem on its own. e/w are fresh identities for the new extra.

| Transition | Required inventory relation | Bounded history |
|---|---|---|
| Clone an extra strong from h | S' = S union {e}; W' = W; e not in S | strong: 1 -> 2; raw weak remains 1 |
| Downgrade h to an extra Weak | S' = S; W' = W union {w}; w not in W | explicit Weak: 0 -> 1; raw weak: 1 -> 2 |
| Retire that extra strong e | S' = S minus {e}; W' = W; e != h | strong: 2 -> 1; raw weak remains 1 |
| Retire that extra Weak w | S' = S; W' = W minus {w} | explicit Weak: 1 -> 0; raw weak: 2 -> 1 |

Freshness includes location/init generation and the appropriate strong/Weak
sort, not just a new display name. Retirement consumes the actual extra once;
it cannot consume h or a foreign token. At each normal stable boundary retain
the h -> C -> A association, allocation generation, data extent, data bytes,
implicit weak, and permission-indexed unrelated-resource frame. Counter updates
and the created/consumed handle are the specified changes, not framed memory.

Creating a handle creates no independent exclusive byte permission. Preserve
the byte-authority ledger except for separately checked compatible transport or
suspension. Retiring a count does not end a loan, consume an NLL end witness, or
restore suspended authority automatically. E1–E5 must check those operations.

Bounded counts avoid the known overflow branches, but actual inequalities,
atomic semantics and ordering remain binding obligations. Clone checks the old
count after fetch_add; downgrade checks before its CAS. Do not replace either
with a generic arithmetic schema. No invalid zero-count history gets a safe
outcome merely because the atomic operation wraps.

Nonfinal extra retirement excludes payload destruction/deallocation only under
its proved original-h/count premises. Bind mem::drop, moves, initialization/drop
flags, Arc/Weak bodies, allocator-field destruction and generated glue. At the
outer caller's scope exit, h itself may be dropped: prove ownership escapes or
account for that final edge. Nonfinal A3 does not close whole-body/page Drop.

## 3. Totality and panic runtime are explicit unresolved dependencies

SRC-1 finds a real specification gap: compare_exchange_weak may repeatedly fail
spuriously. Sequential execution and exclusion of interference do not by
themselves prove downgrade eventually returns. Record
`downgrade_atomic_progress = UNRESOLVED`. A future target/runtime progress law
would be a separate consequential trust candidate, not an implicit part of the
rely. Until proved or explicitly reviewed/adopted, leave normal-termination
acceptance OPEN; do not replace the required total claim by partial correctness.

For None/invalid-range/length-mismatch/false-assert paths, distinguish entry into
panic from finite terminal Abort. `panic=abort` invokes the panic handler and
hooks; formatting, locks, I/O, callbacks, reentry and progress must be covered
by the actual runtime domain. Record `panic_runtime_progress = UNRESOLVED` and
`panic_runtime_effect_closure = UNRESOLVED`. Immediate-abort is a different cfg,
not a substitute. Direct intrinsic abort, such as clone overflow, has its own
binding and cannot justify the panic paths. Invalid-reference/permission UB is
not a promised Abort outcome just because an optional debug check may fire.

For panic-dependent total-outcome claims, establish the exact runtime/progress
closure or prove the relevant panic edge unreachable for the actual caller.
Downgrade's normal-path progress gap remains a separate requirement either way.
A panic-entry marker, a diverging model, or an unproved
exclusion premise proves neither. Abort carries no normal heap/frame, returned
reference, destructor-completion or recovery claim. No current runtime domain
or eventual-success premise is adopted here.

## 4. Successor-sidecar draft shape, not replacement fixture evidence

Any future successor sidecar must identify its frozen predecessor and bind each
actual source occurrence separately. For a twice caller use first_source and
second_source with their own typed place, read capability, liveness interval,
whole-target disjointness, length and frame obligations. Compatible source
sharing does not permit either source to overlap the target allocation.

Owner/frame maps are typed, capture-avoiding substitutions over D, instantiated
body, activation, place/init generation, allocation/control block, interval and
loan tree. For an owner pair bind right/B as target and left/A as framed owner,
including both inventories and offset/buffer fields. Equal bytes, matching
names, hashes or numeric intervals cannot establish owner identity.

| Outcome/check status | Returned read and successor requirements |
|---|---|
| Checked normal return | Bind only actually returned reads; they remain LIVE until their checked original-NLL end. Transport the exact loan tree and H0 -> H1 -> H2. |
| Checked None followed by finite Abort | No normal returned read and no later read/drop normal suffix. Some/normal-success negative remains distinguished from proving Abort. |
| Panic entered, terminal closure unproved | UNRESOLVED outcome; no normal poststate/read credit and no terminal-Abort credit. |
| Rust borrow rejection | No resource return/outcome theorem; BC_CONFLICT gets borrow-check credit only. |
| Intended backend semantic rejection | No accepted poststate; identify the rejected owner/frame/end-witness VC and retain a matched positive control. |

An unexecuted sidecar uses UNRUN/UNRESOLVED rather than marking any outcome
checked. Keep all 17 fixture expectations, BASE closure and both debug profiles;
four alias cases add A3 rather than replacing BASE. This worksheet is not a
new fixture manifest, a capture run or a result for any of those 34 cells.

## 5. Binding slots and next authorization boundary

The following slots remain UNRESOLVED for every actual profile/occurrence:

- Canonical typed Instance/substitutions/owner environment; borrow regions and
  evaluated-place/argument/result/successor/erasure correspondence; schema id
  and version. Display paths and equal DefPathHashes are not these bindings.
- Instantiated core/alloc/std/build identity; target/ABI/pointer width/atomics,
  cfg/features/sanitization, actual overflow-check/codegen settings, panic
  handler/hooks/runtime and compiler/runtime identities. Debug labels alone
  determine none of the missing arithmetic or termination semantics.
- Checked original arithmetic/CFG: start <= usize::MAX - 8, start + 8 <= n,
  nonwrapping absolute lens composition, and applicable reference/isize bounds.
  Retain actual assertion/cleanup successors rather than editing their effects.
- Atomic/intrinsic and generated-code closure: allocator Clone/drop, moves and
  flags/glue, metadata/unsizing, comparison specializations and assertion CFG;
  backend copy/compare/raw_eq where actually selected. Exact u8 copy is not an
  element-Clone loop. Source inspection does not prove these bindings or E1–E6.

The exact source archive is now available and independently inspected; the
earlier missing-source observation remains historical, not the current blocker.
Typed correspondence, progress/runtime closure and checked engine obligations
remain open. Do not fill slots from the old 60 capture rows or invent bindings
for prospective fixtures. All E1–E6 stay checked engineering requirements.

The next consequential step needs explicit implementation/build allowances
and independent review; trust/progress-law adoption and any S1 policy require
their own decisions. A candidate may not admit anything while these obligations
are missing. No old allowance is reset, no refused diagnostic is resumed, and
none of this dependency work discharges a final database cell.
