# Approval169: explicit native compiler-semantics TCB route

User decision following boundary168 and the two explanatory turns:

> Got it. Okay, sounds good. I approve what you need to do here.

This approves the proposed practical route: develop a narrowly scoped native
MIR/heap/full-pointer interpreter and integrated affine-flow checker as an
explicit new trusted compiler component, subject to independent exact-code
review and explicit disposition BEFORE activation. It is not a claim that
source review or tests mechanically prove refinement to native Rust execution.
Future proofs using an adopted component must name its exact semantic/compiler
TCB dependency. No new native correctness, issuer application or cell closure
is conferred by this approval alone.

## Boundary now authorized

Within the supported, source-bound partial native MIR subset, the reviewed
compiler implementation may be the authoritative interpretation of native
activation/control flow, heap/pointer provenance and corresponding shadow
resource operations. We no longer require that a future exact reviewed
component first prove a machine-checked simulation against an externally pinned
native operational model. That would be a stronger, distinct assurance route.

The implementation must actually interpret the selected native call and SAME
normal-return pointer and maintain consumptive resource flow. This is permission
for an explicit compiler-semantic interpretation, not permission to grant
arbitrary pointers authority, replace the original allocation with a wrapper,
or disguise a static ID/non-Clone descriptor as an operational ownership model.
Canonical loaded resource types, actual initialized shadow resources, branch/
activation-sensitive ownership, destructive transfers, provenance/generations,
and unsupported-outcome refusal remain requirements on reviewed code.

The mathematical allocator schema remains exactly disposition159/LAW158. No
second allocator axiom, arbitrary supplied relation, blanket owned-memory or
ambient frame, initialized-byte guarantee, Arc/caller/cache/constructor summary,
assumed caller validity, totality/no-unwind or rollback law is added. All actual
layout, initialization, allocation-to-cell/store, higher-level ownership and
application obligations stay checked. Incoming global ownership must be explicit
and justified; an empty local map is not proof of an empty incoming frontier.

## Activation and unchanged restrictions

Before ANY operative issuer use, require a NEW independent review of exact
implementation/semantic rules/source bindings, an explicit pinned parent
disposition, original compiler-only applicability/negative checks and a trust
ledger. Development builds, WF/mode tests, structural/AIR-shape tests and source
analysis remain permitted. Unreviewed original permission issuance, solver/native
acceptance and native export remain forbidden. Approval of an envelope is not
self-approval of as-yet unwritten code.

The target is still the original full two-phase acknowledged-durability cell,
with all18 original cells required. No local instruction, generated example,
header-only theorem or conditional model closes that cell. Native page/cache/
buffer/flush/public-ack/tree/history/crash/recovery premises must compose.

All existing restrictions continue: no network/Git writes, parent/native source
integration, original application/test execution, general destructive cleanup,
weakened Infer/S0/full-scope/import/export admission, unspecified laws, or reset
of sealed/exhausted/interrupted task histories. Cleanup162 is complete and grants
no future deletion authority. One heavy slot, offline locked jobs1, strict free
disk>16GiB and per-task/command/scratch bounds remain mandatory.

Current accounting: approved native-semantics development/TCB envelope1;
conditionally adopted mathematical allocator schemas1; operative new compiler
components0, operative laws0, actual issuance0, accepted original allocations/
stores/bodies0/0/0; all18/G1/G2 OPEN; native export REFUSED. Goal service last
observed BLOCKED, untouched; no fabricated resume/reset/ACTIVE state.
