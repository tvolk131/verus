#170: implement the now-authorized native interpretation, not more ports

## Exact baseline and changed authority

Use authenticated sealed166, not a retry of its conservative60/64 task:
/private/tmp/redb-native-implementation166-20261109-015200.
HANDOFF166.json f625a93c7c638f9f8ce961c40abfca9bb1d411eb5d3713d44d177419343a5cd1
HANDOFF166.tar.gz 3cf3f6f5ec4f5219da9d1434c5d33fe373caabcf17a7a5ee8d52328ba611f662
MANIFEST166.json 8a92bf50f98965a182bc98ca01a22e9213b3d70f9fe9cb76f5b53eb9d099df7e
SEAL166.json db9495144431fbd8e95e5faee063cc3148b4a9027556029a5ab6b7df58d0b9d1

Authority169 resolves166's boundary question: implement a small reviewed compiler
interpretation as NEW explicit TCB, not a claimed formally proved native
simulation. The actual selected typed MIR event and its returned full pointer
may be interpreted by this component; no existing authoritative native-state
interface need be discovered first. This removes that design ambiguity, not
the obligation to implement the interpretation correctly or pass review.

## Required delivered kernel

1. Introduce an explicit native semantic context containing the supported typed
   control/activation state, native pointer/provenance/allocation generations and
   incoming shadow-resource frontier. Thread it through actual supported native
   statements, calls and successors. Define the semantics of these fields and
   operations; do not just rename State164 or add unconsumed descriptors.
2. Interpret the selected original allocation normal return as the SAME full
   pointer in canonical proof semantics. The call's output binding is the output
   of that native semantic operation, not an independent proof allocation or a
   scalar address later equated to it. Dynamic instruction evaluation and
   activation distinguish repeated calls; a static site is not the event.
3. Implement schema159's once-only genuine raw/Dealloc initialization for that
   normal-success operation and canonical loaded resource types. Preserve the
   source/ABI/runtime/layout/Alignment applicability gates. No success resources
   for null/error/unwind/abort/divergence or unsupported allocator selection.
4. Connect resource introduction/transfers/consumption to an integrated native
   affine checker, including alias/loan/destroy rules and branch-exclusive state.
   Wire actual generated canonical values into this enforcement; simply checking
   the Rust non-Clone status of a host-side descriptor is not enough. Do not
   rely on source-only Verus lifetime checks that never see generated resources.
5. Implement the actual SST-to-AIR candidate semantics using the above context,
   pointer and schema, instead of always returning "unimplemented". Keep a
   separate default-off pre-review admission gate. Candidate structural tests
   may inspect the emitted conditions without operational native credit.
6. Bind at least the reviewed primitive step to the actual original
   zero_filled_arc/Arc/from_iter_exact allocation route selected by160/156;
   distinguish this exact static source linkage from checked whole caller flow.
   Do not substitute an unrelated allocator fixture. Native application scope,
   actual byte initialization/store and unknown incoming resources stay OPEN
   where the supported kernel cannot establish them.

No frame law is added. A restricted empty-frontier kernel is acceptable only
with explicit incoming-frontier conditions, rejection of unknown/nonempty
frontiers and no original applicability credit until callers establish them.
Saying "no local bundle" does not establish that condition. Conditional kernel
functionality must be distinguished from accepted original allocations/bodies.

## Review gate and concrete controls

Do not activate the issuer for original verification or run solver/native proof
applications before a NEW independent exact-code review and parent disposition.
Build/WF/modes/structural and candidate AIR-shape tests are allowed, as is
compiler-only original source inspection. No original application/tests execute.
Retain default-off, all-fields quarantine and conservative transmute repairs.

Deliver real positive instruction-to-AIR/context checks plus negative controls
for duplicate/replayed issuance, substituted native return/provenance/size/
alignment/outcome, wrong source/callee/runtime, unknown/nonempty frontier,
stale/overwritten/moved/borrowed holders, dropped/duplicated resources, mutually
exclusive branches and unsupported cleanup. State exactly which ran and which
did not. A missing loaded-vstd test environment is a setup dependency to address,
not a successful nominal-type or issuer test.

If a bounded subset remains incomplete, identify actual code and precise open
operations. Do not spend the round merely reproducing166's boundary question:
169 selected the reviewed-TCB route. Propose no extra mathematical/frame trust
without separate exact authority. Do not claim kernel completion from scalar
controls, counters, file hashes, uninitialized ports or a further refusal opcode.

## Custody, budget and handoff

Fresh scratch, no parent/shared/sealed/native edits or old target writes. Own
one heavy slot; offline locked jobs1, incremental0/reduced debug. <=90min INCLUDING
seal,64 charged commands including setup/failure/refusal attempts, build<=600s,
diagnostic/test<=180s. Expected fresh logical/allocated<=1.5GiB, HARD<=2GiB,
sampled global free strictly>16GiB with full-build margin before every launch.
No cleanup authority, network, Git writes, source integration or subdelegation.

Instrument launches from the first execution. Persist actual child wait status
before optional post-reap group audit; PermissionError is an audit qualification,
not reason to lose the status. Tolerate vanished transient compiler files in
resource sampling, not resource violations. Record raw streams, exact argv,
allowlisted env/no secrets, clocks, source/tool bindings and failures. Reap owned
groups, preserve unique final binary, authenticate baseline inputs entry/exit,
seal source/semantic-rule delta, regular unique archive byte readback and honest
coverage/trust map. Existing166 receipt gaps remain historical, not repaired.

At handoff explicitly release the slot and stop; no follow-up/self-review.
Parent must obtain NEW independent exact-code review before any operative use.
All18 remain OPEN and current native acceptance0 until independently justified.
