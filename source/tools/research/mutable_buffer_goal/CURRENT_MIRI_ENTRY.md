# Current Miri entry: remaining algorithm connection

The published entry handoffs now provide 25 individually closed historical
Stacked Borrows declarations and17 closed Tree Borrows declarations. They are
conditional MODEL results accepted at that scope by independent review
`e03ce8c2`. The fresh optimized-Python parent replay is recorded in
[parent-entry-replay.json](parent-entry-replay.json). The actual compiler
type/layout checks are in [parent-native-entry-type-binding.json](parent-native-entry-type-binding.json).
Neither result establishes incoming runtime permissions or source C/F.

The next comparison must use the current compiler-matched source in
[native_miri_policy_pin](../native_miri_policy_pin/pins.json), Rust commit
58cf8f9c35b355199727627d07ae34a4e2846d67. The following are source-inspection
facts, not a verified translation or a claim about all lawful Rust callers.

* Current `Stack` stores a bottom-to-top vector and an optional unknown bottom.
  `set_unknown_bottom` clears that vector. In `find_granting`, a wildcard can
  match the unknown bottom; a concrete tag smaller than the unknown bound can
  also match it. Both return `Ok(None)` when no known item grants access.
  Reversing only the known vector into an old-model stack therefore loses an
  entire represented permission case. An empty known vector with an unknown
  bottom is not an old empty stack. Coverage needs a richer relation or a
  justified source unreachability theorem, not an extra client restriction.
* `grant` with an actual access first performs `access` and pushes the new item
  at the resulting top. Its no-access SRW branch instead inserts at the first
  write-incompatible position. Wildcard/unknown no-access cases forget the
  known stack. These branches cannot be identified with one blanket stack-pop
  rule or with the conservative authored Reserved-child model.
* Current `pop_items_after` pops an item **before** invoking its fallible visitor.
  A visitor failure can leave a partially changed stack. Historical pure option
  failure supplies no corresponding final stack. An all-outcomes relation must
  address this distinction; Miri interpreter errors also must not be mislabeled
  as Rust unwinding or silently discarded.
* `Item` is the actual packed `u64`: low61 bits are the tag, two bits are the
  permission, and bit63 is the possible-protector flag. `Item::new` checks the
  tag against `u64::MAX >> 3`. Thus packed-item construction has a61-bit check,
  separate from the wider BorTag counter. Instrumentation failure at either
  boundary needs an explicit classification; no tag-budget precondition may be
  added to the original buffer functions.

A concrete next proof component is the unchanged current
`find_first_write_incompatible`, together with its real packed permission
accessors and vector lookup. Its first-incompatible/bounds contract can be
checked ordinarily over arbitrary stacks, then compared to the published
top-to-bottom list function. This advances the algorithm connection without
inventing another unrelated retag semantics. It does not cover wildcard/unknown
granting, fallible visitors, source argument installation, or caller validity.
Any adapter, authored precondition and unsupported source construct must remain
explicit; successful helper verification is not a full native certificate.

Full canonical remains6/0; the frozen full HMAC XOR-prefix-and-suffix property
remains1/1 unproved. Both are inherited, not rerun here. Native=0, all44+3/101
obligations remain open, publication=false, goal_complete=false. Same actual
checked automation, normal checks and fresh combined acceptance remain required.
