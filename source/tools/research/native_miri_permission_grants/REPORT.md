# Unchanged current-Miri permission predicate

`Permission::grants` verifies for every Permission/AccessKind input with no
precondition, local proof block, theorem call, snapshot or inferred helper.
The selected executable tokens and both enum variant lists match the pinned
Rust/Miri sources at58cf8f9c35b355199727627d07ae34a4e2846d67.

The authored contract is an explicit eight-row table: Unique and
SharedReadWrite permit both accesses, SharedReadOnly permits Read, and Disabled
permits neither. This checks a concrete dependency of current native grant
lookup. It establishes no stack/cache/provenance, runtime grant, argument-entry,
source-safety, physical byte or C/F correspondence theorem.

All six guarded outcomes passed under optimized Python:

* Enabled cold/warm and disabled:3 verified,0 errors each.
* Removing the Disabled guard, reversing Read to Write, or removing the
  SharedReadOnly guard:2 verified,1 postcondition error each. No mixed helper,
  precondition, arithmetic, invariant, frontend or resource rejection qualified.

The three checked bodies are the selected predicate and the two original enum
Clone derives. The scope gate also binds the authored specification table and
both Structural marker declarations by full Rust identity and exact status.
Other original derived Eq/PartialEq/Hash/Debug Rust items remain outside the
selected verification scope, explicitly not claimed checked. Normal compiler,
macro/equality translation, verifier/SMT and inherited vstd remain trust
boundaries. The source adapter adds Structural derives, a Verus namespace,
method visibility and result/contract annotations; it changes no representation
or selected executable expression.

Assistance is redundant for this component: ordinary verification succeeds
with goals disabled. This is not the missing causal shared-automation transfer.
Native execution was not run and is not claimed. Canonical6/0 and full HMAC1/1
remain inherited, not rerun. Native=0, all44+3/101 obligations remain open;
publication=false, goal_complete=false. Independent review `3a9885c0` accepted
the source and six actual outcomes, and found R1/R2 in the delivered gate.
Parent `3f7303c9` fixes both: support identities use an explicit receipt-label
mapping, and failed queries must be exactly the selected grants function.
The fresh six-case parent004 replay passes, and independent `a4d43ec9` resolves
both findings after all six own-path remapped outcomes pass. Mathematical source
and original receipts are unchanged.

[results.json](results.json) binds the successful receipt, frozen inputs and
full source identities. The first two trials verified the positive predicate
but their controller inventories were incomplete (Clone bodies, then
specification/Structural items); both are retained and excluded from acceptance.
No mathematical source change was made for those gate repairs.

```sh
python3 -B -O source/tools/research/native_miri_permission_grants/replay.py \
  --freeze source/tools/research/native_miri_permission_grants/freeze004.json \
  --output /tmp/native-miri-permission-grants-FRESH
```

The freeze describes this integration checkout. An isolated replay must rebind
repository paths to identical own-checkout bytes in a new explicit pretrial
freeze, retaining the exact completed source/verifier artifact hashes. Historical
checkout paths authorize no cross-checkout reads. All completed private baseline
and bundle artifacts remain read-only. No new assumptions, external-body stub,
domain restriction, production edit, engine/policy update, push or PR was used.
