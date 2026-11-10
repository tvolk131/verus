# Independent review checkpoint (before fresh verification)

Base: 6a278283494eba7aafc86a51a10522fc7739da76. Review worker commits
d70148e0fe10e7ba3cc2a8d2a4208233705c2a77 and
c00341dabbd7c8b9922246b78135bd53953735bd, integrated b4abdb63/ffc6ddd9.
Only new evidence/miri-tag-component-review/ is owned. No delegation or delivery
source mutation. Private review outputs will be new mode-0700 /tmp directories.

Typed target: BorTag(std::num::NonZero<u64>) retains every nonzero u64, including
MAX; no application tag budget. Item(u64) updates accept arbitrary initial words.
Item.tag alone requires low 61 bits nonzero. Item.new has an existing packing
assertion at TAG_MASK; its complete body remains unproved.

Acceptance candidates are separate: mathematical bitfields 26/0 on/off;
original runtime-body subset 45/0 on/off (10 exec + 5 proof); explicitly changed
constructor-closure adapter 49/0 on/off (14 exec + 5 proof). Adapter acceptance
cannot establish the exact constructor/lookup bridge. Reproduce eight false
semantic controls, each one named failed query and identical healthy helper
prefix. All six positive runs require complete ordinary selected-body checking,
no cheating, no inferred helper, no assistance payload, no interface publication.

Manual inputs: 5 authored proof bodies, 2 inline proof blocks, 21 bit-vector
assertions, 1 local lemma call, 2 inherited NonZero inverse-axiom calls; all
contracts/spec definitions authored; adapter has an authored closure postcondition.
Runtime generated Eq/Ord/Hash bodies are excluded; three Clone bodies checked.
Inherited opaque NonZero/Option/trait/panic specs and verifier/solver/compiler are
trusted. On/off equality cannot establish causal inference gain.

Retained frontend/setup/resource/proof/expectation failures stay distinct. True
bool conversion and runtime Eq/Lt failures are standard-spec gaps, never semantic
negatives. No new native finite test is needed if retained support authenticates.
No full compiler/vstd rebuild, broad regression, security experiment, or production,
engine, vstd, policy, shared cache/config change is authorized.

Whole goal is unchanged: canonical 6/0 inherited; exact bitcoin_hashes 0.14 HMAC
pad XOR both prefix and suffix, every valid keylen including 0 and >128, remains
1/1 UNPROVED. No fresh combined acceptance here. Native=0; 44 minimal + 3 phase /
101 writer effects OPEN; publication=false; goal_complete=false. No Coq, native,
application, control-flow, physical-permission, or safety theorem follows.
