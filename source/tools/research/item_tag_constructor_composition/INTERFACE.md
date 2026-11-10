# Frozen internal source milestone

Rust 1.99.1 / 58cf8f9c35b355199727627d07ae34a4e2846d67.
Parent engine 3aa7f49a; acceptance context 89deb6a7. Only this new directory is owned.

Carrier: original private `BorTag(NonZero<u64>)`, private `Item(u64)`.
Observation: public closed `BorTag::value(&self): u64 = self.0@` and
`Item::word(self): u64 = self.0`. Internal predicate:
`item_word_wf(w) = (w & (u64::MAX >> 3)) != 0`.
Target original `Item::tag`: requires internal word validity, returns genuine
BorTag with value equal to low 61 bits. Arbitrary high three bits allowed.
Original new/get retain all-u64 domain and original constructor FnDef spelling.
No claim that native transitions establish the internal predicate.

Conditional second milestone: original complete Item::new under the existing
assertion's normal-return bound `tag.value() <= TAG_MASK`, preserving assertion,
all debug assertions, `u64::from(bool)` and runtime comparisons. Above-bound
assertion outcomes are not discarded from the original-source claim: only a
bounded normal-return helper is proposed. This file is a target, not acceptance.

No source safety, physical permission, native/reference C/F certificate,
publication or whole-goal completion follows. Native=0; publication=false;
goal_complete=false; canonical fresh acceptance and exact full HMAC remain pending.
