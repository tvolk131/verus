# Packed tag lookup interface (typed milestone; obligations pending)

Source: Rust 1.99.1 commit 58cf8f9c35b355199727627d07ae34a4e2846d67,
`borrow_tracker/mod.rs` BorTag and `stacked_borrows/item.rs` Item/Permission.
This interface is a target, NOT an accepted contract or a native certificate.

Carrier MUST remain `BorTag(std::num::NonZero<u64>)` and `Item(u64)`.
Proposed ghost observation: `tag_value(t: BorTag): u64 = t.0@` using the
standard NonZero view (availability in the accepted bundle must be checked).
`item_wf(i: Item): bool = (i.0 & (u64::MAX >> 3)) != 0`.

Lookup may eventually consume these INTERNAL results, only after body checking:

- `BorTag::new(i: u64) -> Option<BorTag>`: no precondition; None iff i == 0;
  Some(t) implies tag_value(t) == i. Domain is all u64, including MAX.
- `BorTag::get(&self) -> u64`: equals tag_value(*self), always nonzero.
- `BorTag::inner(&self) -> NonZero<u64>`: preserves the exact NonZero value.
- `BorTag::one() -> BorTag` and Default: value == 1.
- `BorTag::succ(self) -> Option<BorTag>`: None iff value == u64::MAX;
  otherwise Some value+1. No 61-bit restriction applies to BorTag.
- Equality/injectivity: a == b iff tag_value(a) == tag_value(b).
  Ordering: a < b iff tag_value(a) < tag_value(b), likewise <= and cmp.
  Original generated trait bodies and standard NonZero trust need explicit scope.
- `Item::new(tag, perm, protected)` returns with item_wf, tag == original,
  permission == original, protected == original, whenever tag_value <= TAG_MASK.
  The source asserts on larger valid BorTags; this is an internal normal-return
  contract for an EXISTING check, not an application input or tag budget.
- `Item::tag(self) -> BorTag`: requires item_wf(self), then returned value is
  self.0 & TAG_MASK. It is NOT defined for arbitrary packed words with zero low bits.
- `Item::set_permission(&mut self, perm)`: preserves low 61 bits and bit 63,
  hence item_wf, tag identity, and protected; returns the requested permission.
  This preservation applies to every packed u64, even ones not satisfying item_wf.

No current SourceBridge acceptance is asserted. A raw-u64 replacement of BorTag,
assumed native equivalence, new axioms, assume, and external_body are excluded.
The parent owns integration and independent acceptance. Global status remains:
native=0; 44 minimal + 3 phase / 101 writer effects OPEN; full HMAC UNPROVED;
publication=false; goal_complete=false.
