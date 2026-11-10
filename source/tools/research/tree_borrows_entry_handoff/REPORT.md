# Actual checked Tree Borrows entry handoff

## Result

`Entry.v:entry_progress` derives fresh child creation, the required **READ after
creation**, `bor_step` via the actual `RetagIS`, final `state_wf`, and exact
outside-block frames in the published Simuliris Tree Borrows core at
e3abfcdc301f92250d4227fc03c00463bf57b982. It covers `FnEntry`, `MutRef`,
`TyFrz`, every positive natural range size (including 10 and 128), any represented
parent, and arbitrary represented ancestry and protectors satisfying the explicit
structural predicate. It changes the next tag from `snp` to `S snp`; heap, active
calls and next-call counter are unchanged in the result state.

`entry_existing_range_frame` additionally preserves every original node's
protector and default permission, and its lazy permission at **every offset
outside the interval within the same block**. Outside blocks are equal as entire
trees. These are model frames, not byte-array/HMAC suffix claims.

The scope/interface milestone is commit `5bacdd19`. The initial interface needed
notation/name-qualification repairs during elaboration; this is recorded as
development history, not semantic evidence. The delivered proof and its complete
compiled inventory are the final interface.

## Exact public entry premises

1. `state_wf s`.
2. `strs s !! blk = Some tr`.
3. `tree_contains parent tr`.
4. `cid ∈ scs s` (the call is already active).
5. `(0 < sz)%nat`.
6. `entry_eligible tr (scs s) parent (off, sz)`.

The last premise is an independently inspectable structural condition on the
**original** tree, not an existential successful execution:

* For every original node and every offset in the range, if the node is the
  parent or its ancestor (`rel_dec tr parent (itag it) = Child _`), its permission
  must not be `Disabled`.
* For every other node, if its protector is active and its permission initialized,
  its permission must be neither `Active` nor `Disabled`.

The ancestor case permits protected `Active` ancestors. The foreign case permits
inactive protectors, unprotected nodes, lazy nodes, and other permissions allowed
by the predicate. No fixed root parent, empty protector set, or fixed tree shape
is imposed. `state_wf` alone is not used to infer arbitrary-parent read progress.
The foreign-node exclusions match protector failure cases of the actual READ
transition; no assertion of minimality of the entire predicate is needed.

There is no public premise that `create_child = Some`, `memory_access = Some`,
the source is safe/defined, or a future operation succeeds. The internal
`entry_inserted_compatible` composition lemma accepts a creation equality only
to invoke upstream insertion/relationship lemmas. `entry_progress` derives that
equality by reducing actual `create_child` and constructs the read witness using
actual `all_access_compatible_implies_access_success`. It then applies the
upstream retag/access well-formedness lemmas. `entry_result` stores success
equalities as **conclusions**. The range-frame lemma consumes this established
result; its exact additional ordinary premises are in `INVENTORY.json`.

## Validation and complete evidence

* Normal Rocq 9.2 compilation of the final private copy succeeded.
* Full recursive `rocq check -o` succeeded, with no `-norec`, admissions, unsafe
  checking, assumed positivity or disabled checks: 458 libraries/loaded objects,
  51,983 constants. This is the complete dependency closure of this entry module,
  a subset of the delivered 507-library baseline plus this new module.
* All 17 own declarations are individually **Closed under the global context**:
  15 kernel constants (including all five generated field projections), the
  result record, and its constructor. `INVENTORY.json` contains the full compiled
  module, every type, every assumption result, exact declaration names, and the
  complete recursive checked-library inventory. The primitive record generates
  no separate eliminator constants in this compilation; the inventory is checked
  against both the kernel list and `Print Module`.
* `OBJECTS.json` lists every object actually loaded by this recursive check, with
  its SHA256. Each matched the delivered loaded-object map or the private normal
  compile receipt. No older Coq objects were added to the loadpath.
* `COMMANDS.json` records every supervised trial's exact argv, cwd, exit code,
  source/pretrial/product hashes and raw-log hashes. `TRIALS.json` distinguishes
  elaboration failures and setup failure from successful proof evidence.
* `AUTHENTICATION.json` binds the supplied ready-receipt SHA256 to the successful
  baseline build/kernel/inventory receipts and all source, installed-library,
  tool and object maps. The pre- and post-check authentication did not execute
  the historical driver or follow its worker checkout path. All completed
  baseline artifacts remained read-only. `SOURCES.json` binds the final local
  source/tool-driver bytes and immutable baseline manifests.

Raw logs, intermediate proof copies, query output and compiled products remain
private under `/tmp/tree-borrows-entry-handoff-20261110`. Reproduction commands
are in `COMMANDS.json`; use fresh trial names because the runner refuses to
overwrite trial directories. Only private source copies are compiled, using
`-Q /tmp/tree-borrows-checked-foundation-20261110/simuliris/theories simuliris`
and a separate `EntryHandoff` namespace. Inherited Coq/Rocq/OCaml path variables
are removed; temporary files/config/cache are private; jobs are at most two.

## Trust and coverage limits

All new concrete proofs are individually CLOSED. The full recursively checked
import context nevertheless reports functional extensionality, and the broader
delivered baseline includes classical excluded middle and constructive indefinite
description as well. This exceeds the existing production trust. The complete
upstream internal assumption/type maps remain in the existing checked-foundation
directory and are bound by the authenticated ready receipt; no new assumptions,
alternate semantics, source facts, imported native facts, engine or policy
changes were made. Printed opaque theorems appear as `Parameter` in `Print
Module`; their individual assumption queries establish that they are proofs,
not added axioms.

Coverage of `entry_eligible` by lawful Rust callers is **unproved**. The theorem
begins with an already represented parent, an already active call, and a model
`state_wf`; establishing FnEntry setup, current argument installation and native
source mapping is separate work. Empty/dangling references and current Miri
differences remain gaps. `Reserved` to `Active` is not borrowck two-phase
activation. Source identities or this conditional model proof establish no native
bridge. No original defined-execution, `SourceSafe`/`pool_safe`, no-unwind or
supplied-client-permission premise is assumed for the original Rust programs.

The original domains are unchanged: every u64/arbitrary initialized exclusive
`[u8;10]` for canonical unsigned varint; independently frozen bitcoin_hashes
0.14.0 exact HMAC XOR on arbitrary initialized `[u8;128]`, all valid key lengths
including 0 and >128, with both XOR prefix and untouched suffix. This worker
does not discharge these native-source goals or rerun their fixtures.

Inherited canonical status remains **6/0**; HMAC remains **1/1 UNPROVED**.
Native correspondences **ZERO**; **44 minimal + 3 phase simulations + 101 writer
effects OPEN**; `publication=false`; `goal_complete=false`. Fresh combined
independent acceptance and parent integration are still required. No push, PR,
production edit, installation, shared toolchain mutation or subdelegation occurred.
