# Stage1 local NonZero Ord implementation preregistration

Base fc0479a6; source design ebbd3cc1, accepted conditions fc0479a6.
Before implementation trials: inherited read-only audit passed with Python -B
and -O -B, 168 artifacts / 7 MIR / 11 baseline cases, zero new proofs.
Frozen baseline source is run.py::source('carrier_ord') and
source('carrier_partial') in ../nonzero_ordering_goal, authenticated by
../../../../evidence/nonzero-ord-factoring-design/INPUTS.json at base778c5e6a.
The actual source remains `a.cmp(b)` with arbitrary genuine borrowed NonZero<u64>.
Baseline Ord0/1, PartialOrd5/0. Full domain includes MAX and shared aliases.

Typed interfaces live separately in checked_nonzero_ord.rs:
ConcreteOrderingInstances binds actual trait/core/type/get/primitive identities;
CheckedExpansion has a private constructor and injective typed formal/result map;
OrderingModelBindings must originate in ordinary imported definitions;
check_ordering_function takes the actual complete Function, not its descriptor.
No acceptance token, assumed spec or new representation/view equality is allowed.
No activation until resolver, whole instantiated MIR, exact get expansion,
actual full Function/signature/contract correspondence and normal helper queries
are ALL active. Missing typed model registration is an explicit blocker.

Whole carrier inventory: nine locals, two blocks; 14 ordered entry statements,
primitive cmp with Move(_3),Move(_6), return _0, normal bb1/unwind Continue;
four ordered storage deaths then Return. Two single Transmute assignments at
indices4/11 map get formal1/result0 to (5,4)/(8,7). Original NZ copies at3/10
remain; every storage/borrow/read is accounted for. Getter: two locals, one
Transmute Copy formal1 to result0, Return. Primitive: five locals, seven
statements, one block with Cmp; bound dependency ONLY, never a new Cmp encoding.
Metadata is_polymorphic=true is not a type proof; recurse through concrete
normalized types and bound regions independently. Bool recognizer unchanged.

Actual Function inventory must bind two ordered distinct params, distinct return
and fresh local identities; exact types, attrs, contracts, get/get/primitive
call targets/trait dependencies, reads/borrows/effects/order/exits. Requires empty;
ensures exact unsigned numeric Ordering of original input Views. No local app
proof/snapshot/theorem/range restriction. Native source retains original std call;
synthetic helper is logical VIR only. Existing get View/spec_get and u64 Ord
models are inherited trust; new checker code is compiler TCB, not a kernel theorem.

Export barrier: any selected explicit/Cargo output for ordering-using crate must
be invalidated/refused before private pruning, even mixed bool/order, success or
failure. No ordering certificate/import kind. Keep schema39/policy72/cache75.
No vstd/core/std/model/historical fixture edits. No wrapper profile extension.

Validation targets: own Rust1.99.1 offline locked jobs2 rustup-first build,
matching vstd2250, immutable eight-tool bundle before proofs, direct cold/warm,
goals-off0/1, swapped/false postconditions with healthy helper, aliases/full
symbolic domain and boundary observations; complete structural map/body/effect
negatives; all export/disabled checks controls; bool2/0 and goals-off clients1/0,
false0/1; VIR149, existing rust_verify lib6 + new units, focused40. Failures and
setup runs retained separately. No proofs claimed before all checks. If blocked,
report exact remaining typed obligation, with no unsafe substitute.

All raw products private. Compact evidence here only. All commits local, no
push/PR/publication commands or permission request. No agents. Native0;
HMAC1/1 UNPROVED;44+3/101 OPEN; whole-goal publication=false;goal_complete=false.
