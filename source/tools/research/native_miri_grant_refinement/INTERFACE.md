# Frozen implementation interface (before trials)

The selected.json strings are the exact compiler-pinned executable functions:
Stack::find_first_write_incompatible, Stack::get, Item::perm, Permission::from_bits,
and optional reusable to_bits/set_permission/protected dependencies. literal.rs
retains these executable bodies unchanged. Whole Miri is not selected or proved.
FREEZE.json authenticates the three full upstream files and all eight verifier
bundle artifacts against the delivered receipt. Raw upstream bytes stay private.

Intended internal contract: for arbitrary Vec<Item>, every Item an arbitrary u64,
and granting < borrows.len() whose decoded permission is Unique or SRW, return r
with granting+1 <= r <= len. Unique: r = granting+1. SRW: all granting+1 <= j < r
are SRW and r = len or r is not SRW. No prefix/tag/protector/stack-shape constraint.
This is an authored internal contract; real caller validity remains unproved and
no original application input domain changes. usize arithmetic must be checked.

Frozen carrier adapter projects private Stack to its borrows field, discarding
unknown_bottom and cache metadata: selected bodies read only borrows and do not
mutate Stack. This is a source projection argument, not a formally verified
memory/layout or whole-package equivalence. The unused impl lifetime is removed.
The diagnostic bug! macro maps to panic! with the exact message tokens, since
rustc's diagnostic implementation is not part of this local crate. The checked
precondition must make both diagnostic branches unreachable. Derives initially
match Item/Permission source; all later frontend-driven changes will be recorded.
New specs, contracts, invariants, checked lemmas, manual calls and standard
library contracts will be separately inventoried. No new assumptions or external
bodies are permitted. Any frontend blocker will retain the literal failed case.

Only after source-helper success: compare independent semantic characterizations
to the historical top-to-bottom list function. Its answer is removed prefix size,
not vector retained index: removed = len - r, granting_list = len - 1 - granting.
This mathematical comparison cannot establish Rust semantics, caller obligations,
Coq import of Rust verification, or current-Miri runtime correspondence.

Native=0; inherited canonical6/0 and HMAC1/1 unproved are not rerun here. All44+3
phase/101 writer obligations remain open; publication=false; goal_complete=false.
