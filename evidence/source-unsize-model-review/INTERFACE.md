# Independent array-to-slice carrier review interface

Review base: 58ea8ca4; mathematical checkpoint f6ddbcd4; current controller
dfe1a04a (source-binding intermediates 41c3feb2/eb56be58). Own only this new
evidence directory. No implementation edits, subdelegation, push, or PR.

Typed MODEL interface under review:

```coq
carrier_store {H} (s : state H) (src dst n : nat) : option (state H)
```

Success requires distinct locals, initialized immediate `RefValue r` source,
uninitialized immediate destination and `n < 2^64`. It constructs a destination
`SliceValue r n` and preserves the original source carrier, physical cells,
anchors, history, private storage, trace and CONTROL. These conditions delimit a
partial MODEL; they are not additional application requirements. The source
selector authenticates only root bb0[4], move _4 mutable array128 to _3 mutable
slice. The model erases the reference's Rust type, validity, ownership and loan
information. Neither generic10/128 nor source identity proves native simulation.

Review acceptance requires the authenticated completed trial004 receipt, honest
trial001/003 equality and trial002 setup failure, a fresh README replay with
normal coqc, all33 individual assumptions (28 Carrier/5 Root), complete recursive
coqchk, unchanged parent Interface/Graph/Storage objects, and nine pure selector
controls in both Python modes. Inventory includes any generated declarations;
the inspection module must contain none. Frozen code/support/tool/dependencies,
readiness/durable-build/GMP pins and initial/final full TARGET guards are required.
No Iris logical import, exclusions, new axiom, admission or unsafe mode is allowed.

Accept/request-changes applies only to this representation MODEL. Entry,
validation/retag/write-hook, actual move, control/phase, source safety and original
C/F obligations stay open. `root_still_pending` cannot advance a source statement;
source-dead composition is not source reachability/loan resolution. Equal
addresses in `changed_tag` do not imply equal tags. No doubled permission follows.

The full every-u64 original varint/arbitrary initialized10 and exact independently
frozen HMAC pad-XOR/arbitrary initialized128/all key lengths (including0/>128),
prefix plus unchanged suffix, common checked automation, fidelity/normal checking
and combined independent acceptance requirements remain unchanged. Canonical6/0
and HMAC1/1 unproved are inherited. Native0; all44+3/101 open; publication=false;
full goal incomplete. No assumed defined execution/no_unwind/permission/source
safety/C-F bridge.
