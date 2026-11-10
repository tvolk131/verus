# Implementation scope and interface milestone

Parent base: `04b51409`. Owned edits: this directory only. No delegation,
checkout creation, push, PR, production changes, installs, or shared toolchain
writes. The completed `/tmp/stbor-checked-foundation-20261110` is read-only.
Historical checkout paths in delivered receipts are labels only.

Before any proof trial, retain candidate004 and its status verbatim in Git and
in a private per-trial source freeze. Its 13 declarations have zero accepted
proofs at this milestone. Correct syntax/proofs against the matching normally
checked Coq8.9.1 foundation, then inventory every own compiled declaration and
helper with Check / Print Assumptions and recursive coqchk without exclusions.
Every attempt gets exact argv, clean environment, exit, raw-log hash, source,
tool and object receipts. Failed attempts are setup/proof failures, never
semantic controls. Private outputs/cache/TMPDIR; serial compilation, jobs <=2.

Target interface: positive generic n, incoming published MODEL valid_block +
WF + live call -> published FnEntry normalization -> published TwoPhase
reborrow, actual head_step witnesses, returned-tag relation, heap/call/WF,
outside-span and active-protector/ancestor frames; specialize 10 and 128.
Keep zero-size/dangling historical behavior separate. No new axioms,
admissions, unsafe modes, hand-authored operational systems, caller-top,
pool_safe, retag-success or future-defined-execution entry assumptions.

The source-only review covers candidate003's 12 declarations only. Neither its
suggested sequent nor the current candidate is proof evidence before checking.
MODEL success does not bind actual caller_arg_copy -> callee Local(1), remaining
entry hooks, StorageLive2/3/4, or original C/F observations. All-lawful native
coverage, permissions, typing, bounds and unwind remain open. Native=0;
44 minimal +3 phase simulations /101 writer effects OPEN; publication=false;
goal_complete=false. Varint inherited6/0; full HMAC inherited1/1 UNPROVED.
