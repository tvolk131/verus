# Frozen no-help Item packing milestone

The target is pure `source.py generate('full')`, SHA-256 in FREEZE.json.
Only named completed fixture bytes are read: the pure generator is rebound to
runtime bodies recovered from its authenticated full-help output. Regenerating
full-help must match that completed output byte-for-byte before freezing full.
No historical checkout or upstream artifact path is followed. The frozen full
source has no local proof functions, blocks/calls, snapshots, or assertions
other than its original runtime assert/debug_assert macros.

The original private NonZero/BorTag/Item carriers, .map(BorTag), .get(),
u64::from(protected), normal-return tag bound, semantics/views and all runtime
branches remain. The full-help manual cost is one proof function, three inline
proof blocks, one quantified bridge with one lemma call, and seven explicit BV
assertions (four in the helper, two in perm, one in protected). The quantified
assertion itself is an eighth authored assertion.

Authenticated accepted engine35bc7b12, schema38/policy70/cache73:
full on34/3, off26/3; full-help on52/0. Full-help off recorded in receipts.
First mechanical failures: Item.tag's word well-formedness at new.tag(),
Permission.from_bits's u64 <=3 precondition at the masked right shift, and
protected's literal-mask postcondition. Original Eq soundness/maximality/exactness
remain freshly checked. Four initial trials failed before proof due to runner
VERUS_ROOT pointing at the not-yet-built private bundle; these setup failures
are retained, corrected trials use baseline2 names.

Only checked_foreign_packing.rs may change production. Proposed generic finite
unsigned arithmetic facts must remain closed, quantified with ranges, checked
by isolated ordinary BV with no caller requires, and used as the exact same Arc.
No name/permission semantics, domain reduction, trusted additions, global
versions, publication changes or precision/inferred-query changes are allowed.
Existing commands.rs hook remains. A natural independent flag writer is required.

Private raw evidence root is in PRIVATE_ROOT. All results are conditional MODEL
checks; native=0, publication=false, goal_complete=false. HMAC/canonical are
unchanged. Fresh independent parent acceptance is required.
