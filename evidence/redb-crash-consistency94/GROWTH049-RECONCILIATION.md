# growth049 evidence reconciliation — November 7, 2026

**Decision: retain the bounded replay evidence; approval remains WITHHELD.**
This is a read-only reconciliation, not another proof run or an integrity
reauthentication. Let R denote
`/private/tmp/verus-redb-growth049-independent-3VahHb` and C denote
`/private/tmp/verus-redb-fixedheight-growth045-sGMSLz`.

## What the existing replay establishes

- R's recorded applications001..018 reproduced C's final014..031 outcomes:
  **eight positive passes and ten intended isolated semantic rejections**, not
  eighteen positive proofs. Both OFF/opt3 and ON/opt0 used fresh, successful,
  same-profile provider/client imports. The five negative distinctions per
  profile have passing counterparts. These are verifier results, not native
  executions. Basis: `R/REPORT.md:104`, `C/REPORT.md:141`.
- The checked mathematical dependency is fixed EXISTING height1..4,
  `full=true`, initial `rep` AND `all_spare` at EVERY level,
  `0 <= old <= new <= 1048576`, with new<=64/4096/262144 for heights1/2/3.
  Height4 uses the global cap. Zero and equality are included; no level is
  inserted. Within that domain the method exports complete after-`rep`, spare
  preservation, all retained true/false flags, newly logical SET/reserved bits,
  exact word lengths `max(old_words,ceil(new_layer_length/64))`, unchanged old
  physical words, MAX appended words, and canonical summaries at every level.
  Physical means initialized Vec elements, NOT allocator capacity/ownership.
  Basis: `R/REPORT.md:66`, `R/inputs/growth-006.rs:1641` and `:1740`.
- This remains conditional on inherited models/toolchain/SMT assumptions and
  selected scope. Provider107 items are 25 native +3 authored exec,42 proof,
  34 spec,3 bodyless proxies; not107 native bodies. Consumer21 constructor
  cases per profile are checked calls, not runtime executions. No broader
  all-callers/native-identity certificate follows. Replay authentication and
  complete per-run audit are prior reported findings, not redone here.

## What reviewer019 actually shows

`native_lifecycle(64,4097,4097)` selects height3: logical lengths change
`[1,1,64] -> [2,65,4097]`; physical word counts stay `[1,2,65]`.
The attempted source matches `runs/attempt-019/input.rs` byte-for-byte.
Its OFF invocation imports own001/002. Stderr reports the sole assertion
failure at line25, `bit(witness.grown@[0].words(),1)`; receipt records exit1,
verified0/errors1, no certified selected body, and no output artifacts.
Earlier assertions have no reported diagnostic, but are NOT a separately
successful partial witness. Basis: `R/boundary-witness.rs:8`,
`R/runs/attempt-019/stderr.txt:1`, `receipt.json:28`, `invocation.json:22`.

This is failure to discharge a caller assertion, not proof of its negation,
not a native counterexample, not an isolated negative control, and not a
contradiction of the provider theorem. It earns ZERO semantic credit.
Planned020 false-claim OFF and021/022 ON were never launched; the required
additional positive/false-claim pair in BOTH profiles remains incomplete.
Basis: `R/STOP-REASON.json:2`, `R/MATHEMATICAL-REVIEW.md:107`.

## Premise versus bridge versus contradiction

**No missing semantic premise is established for this case.** Its arguments
satisfy `client_domain` and `2 <= old < new`. The imported lifecycle contract
exports `profile(before,64,{1})`, `profile(grown,4097,{1})`, exact storage sizes,
and every physical-word frame; `profile` includes `all_spare` and `rep`.
Basis: `R/inputs/client-005.rs:6`, `:92`, `:110`.

A short mathematical entailment uses only those exported facts:
old root length=1 and one physical word imply index1 is old root spare;
instantiate `all_spare(before)` at height0 and `spare_full(before[0])` at
query1 to obtain the old root bit. Instantiate the exported word frame at
height0/word0; unfolding `bit` transports that bit to the grown root.
Thus the failed assertion follows without a new premise or provider theorem.
The root word need NOT be MAX: its bit0 is false along the retained-free path.
`spare_word_full` cannot justify that mixed root word; the existing witness's
leaf-word64 calculation alone is not the explicit root bridge.
Basis: `R/inputs/growth-006.rs:9`, `:1587`, `:1637`,
`R/inputs/client-005.rs:131`, `R/boundary-witness.rs:19`.

**Supported:** an adequate root-bit/frame bridge exists mathematically and is
not explicitly supplied by this witness. **Hypothesis only:** SMT quantifier
instantiation/unfolding is the precise failure cause. No verifier-tested
diagnosis, missing provider lemma, or actual contradiction is established.
Separately, the analytical example without `all_spare` shows why that ALREADY
PRESENT premise cannot simply be dropped; it attacks an unclaimed extension,
not this theorem, and was not native execution (`R/MATHEMATICAL-REVIEW.md:63`).

## Minimum sound next step and unchanged limits

Now: transfer this distinction into the durable handoff and STOP. A future
authorized proof-engineering task should first expose the above tiny,
transparent caller-local spare-bit/word-frame bridge, rather than weaken the
claim, assume the answer, broaden trust, or replay the whole proof. Its verifier
success is NOT established here. No edit, retry, new control, or follow-on is
authorized by this proposal; standing stop/no-retry rules remain operative.

Review049 stays19/40 applications and2/12 source versions, including019 and
the authored but unlaunched false source; candidate045 stays31/64 and41/48.
Nine candidate failed drafts, ten credited controls, four recorded auxiliary
failures, and all inherited ledgers/qualifications remain distinct and intact.
No allowance is reset, pooled, transferred, or implied by this memo.

**Explicit limits:** NO approval/adoption/trust or code integration;
NO native counterexample established; NO full-Buddy proof (mutable-Vec
IntoIterator gap remains UNPROVED); NO final redb acceptance (all18 obligations
remain OPEN; ownership-engine trust/scope approval remains pending).
Only this new memo/output directory was written. No proof/source/input edits,
verifier/solver/compiler/native-target/build runs, network, Git writes,
subdelegation, prior-task resumption, or separate flagged zero-growth050 access.
