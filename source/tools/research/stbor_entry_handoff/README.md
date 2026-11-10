# Published Stacked Borrows entry handoff: unvalidated candidate

`candidate.v` is an ordinary-proof candidate against the original pinned
Stacked Borrows artifact, not the separate conservative byte-stack model.
It has **not run**. Compilation must wait for the matching baseline's successful
normal build and complete recursive kernel-check receipts. No theorem or native
fact is accepted from this candidate.

The structural premise describes a current Unique/SharedReadWrite grant at the
head of each position's stack; arbitrary ancestor tails and protectors remain.
It derives the published `valid_block` premise and proposes progress, actual
head-step execution, wellformedness, whole-heap preservation, outside-stack
frame and active-protector preservation. It does not define lawful Rust inputs
by this predicate. Proving coverage of every lawful Rust input remains open.

`FixedSize n` is the artifact's model size. Mapping actual Rust byte extent and
addresses to those locations is a separate obligation. Generic10/128 and the
empty-size model case establish no common native automation. All original
domains, exact bodies and full properties remain required.

This artifact uses old raw Untagged semantics and has explicit bounds TODOs.
The candidate uses only mutable-reference retag operations and makes no raw
pointer correspondence claim. Neither it nor the baseline proves that pinned
rustc1.99.1 executes this model, discharges source safety, or interprets the
original/pending Verus C/F observations. Exception/cleanup correspondence is
also open; the model retag expression has no native unwinding semantics.

Native0,44+3/101 open, publication=false, whole goal incomplete. The two
foundation builds are isolated research candidates; the existing Coq8.20 and
Infer production environments are unchanged.

The current candidate004 adds a proposed FnEntry normalization lemma from the
published `valid_block` premise to a new top grant. It does not require the
caller tag already at the top. This lemma is also uncompiled; obtaining that
block premise from the actual argument copy is the
[earlier source obligation](../../../../evidence/published-entry-source-review/MISSING-LEMMA.md).
The source-only review covered the earlier12-declaration candidate, not this
new13-declaration version. No compiled proof has been accepted for either.
