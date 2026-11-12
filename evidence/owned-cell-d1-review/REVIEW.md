# Independent review of inactive D1

Prepared root `5c503b74955b2594e91d7f0b2809541936f53d5b`; D1 worker source
`2e7b8b319`, integrated by `3cd726a25/9fa778d3d/83588e221/8fea9b45b`.
Early scope/authentication commit: `14d761345`. This review owns only this
new evidence directory. Final rerun receipts/costs are recorded in RESULTS.json.

**Finding D1-R1 (P2, before relying on backend type-use enforcement):**
`source/vir/src/internal_repr.rs::validate_krate_uses` visits only Functions
and datatypes. It omits `trait_impls`, `assoc_type_impls`, `traits` and
`opaque_types`. Independent synthetic VIR fixtures place the same otherwise
rejected `UnsafeCell<u32>` in a trait implementation argument, an associated
type implementation RHS, and the opaque-type inventory. All three pass both
`validate_krate_uses` and actual `Ctx::new` entry with a baseline GlobalCtx.
The direct type-use check and datatype-field container reject that type.
See boundary.rs, test `document_backend_container_scope_before_any_activation`.
The three omissions are runtime-reproduced; omission of trait declarations is
source-inspected only. The backend therefore cannot independently enforce the
closed instantiation rule over all its type-bearing metadata.

Suggested correction: visit/check these remaining inventories, or refuse them
explicitly whenever this owned descriptor is present. Include negative tests
at backend entry, even if frontend admission separately excludes these forms.
Using the current general `visit_krate` alone still leaves `opaque_types`
unvisited; handle that inventory explicitly. This is an internal boundary
hardening finding, **not a demonstrated admitted-source bypass or unsound
native theorem**. The descriptor is uninstalled and F1 refuses these source
forms; no current source authority is obtained. The fixture creates synthetic
metadata and prebuilds GlobalCtx from the descriptor-only baseline: it does
not claim the malformed crate passed the ordinary full frontend/WF pipeline.

The dormant proposal API has no production caller. It is private under the
existing module and invoked only by the loaded-native unit. Factory inputs are
fresh loaded lang-item metadata, native stable identities, actual private
field/generic/?Sized/!Sync checks, resolved original method signatures with
same returned/input bound region, and original instantiated optimized MIR.
It constructs the same nominal native datatype and real LOCAL helper scope;
there is no forged core owner. Descriptor encoding and source access stay
separate; wire serialization/deserialization refuses Internal descriptors.

The new WF hook checks actual instantiations; the backend rechecks descriptor
shape and Function/datatype uses, subject to D1-R1. Rev's factory/validator
remains a separate family, its argument behavior unchanged, and the backend
retains its Rev-only fast path. No default, schema39/policy78/cache81, wire
format, trusted registry, native source or vstd source changes were introduced.
No public source installer, new field authority, helper installation, call
rewrite or exported fact was found. Default-off behavior stays on the ordinary
pipeline; explicit requests end before imports/pruning/cache reuse.

Actual emitted computations were inspected: new constructs the native
aggregate from `copy _1`; into_inner moves `(_1.0:u64)`; get_mut exclusively
reborrows `((*_1).0:u64)`. The checker reads the actual Function argument/return
and body types, nominal field/variant, local origins, read mode and dereference
place against MIR. It does not merely compare an operation descriptor.
Wrong origin, target, field, width, copy/move and mutable reborrow fixtures
are meaningful computation rejection controls. Its exact shape can refuse
otherwise equivalent MIR, which is fail-closed on this pinned toolchain.

**This is intentionally partial correspondence.** Ensures and their
current/future relations are not independently validated. Nor are admitted
client input/store/reload/writeback effects. No complete certificate or
semantic proof follows. In particular the three normal full Function WF
checks all refuse `disallowed: field expression for an opaque datatype`.
Those expected failures and wrong-width/admission refusals are negative
controls, not passed native body proofs. No mode/lifetime/SMT/body/state credit
is awarded to the proposals.

Before authority, freshly bind native constructor aggregate, consumed private
field move and exclusive field reborrow to the *complete* emitted Function,
contract and original native DefId/signature/MIR. Recheck authority at normal
WF/backend boundaries without source attributes or a forged owner. Then pass
ordinary full body/contract checks. Current/future correspondence and distinct
F2 same-client final input-field state query (arbitrary entry), false-state and
store/writeback deletion controls remain necessary. F3 must inspect original
native erasure calls and original-name mode entries under real compilation.
No scalar result proof or supplied constructor history substitutes for these.

Fresh validation: loaded-native unit **1/1**, complete VIR **171/171** (including
all eight Rev tests), separate Rev rerun **8/8**, and combined focused suites
**10/10**, covering **166 verifier invocations** by exact test-source enumeration
(32 preflight +134 admission) plus one native callback rustc build. Every
verifier invocation carries --no-cheating. Both ordinary and Infer are represented;
malformed parse cases are refusal controls. Independent synthetic boundary
fixture **2/2**, after an earlier reduced **2/2** run. Its three accepted
malformed placements witness D1-R1; those passes are not acceptance evidence.
The loaded-native unit reruns 54 MIR +4 signature/region, eight descriptor,
six type-argument and45 emitted-Function corruptions, both wire refusals,
two wrong-width hooks and three normal private-field WF refusals.

Combined compiler SHA256:
`a3aa9a1cb805996b6a00045d81d73549159268ae6b0e489567c7341cdf67369d`.
Seven supporting files are inherited unchanged from the authenticated frozen
D1 snapshot; vstd2257/0 is inherited, not rerun. All eight snapshot files and
twelve prepared-source hashes were reauthenticated after testing. Eleven
logged commands took451.177s, including failed harness attempts. Validation
froze at2026-11-12T17:25:17.693742Z; conservatively counting the entire interval
from17:05 snapshot availability takes1217.694s (20.295 minutes), including
all builds/waits. No build subtraction is needed for the45-minute limit.
CPU and monetary meters were unavailable. Setup/guard/fixture failures are
explicitly retained in RESULTS.json and private receipts; none is a suppressed
production failure. The first compile took180.080s and succeeded, but its
wrapper guard failed on this review's driver mtime; the stable-driver build
rerun passed before any combined runtime use.

The prepared root's F1 skip fix is preserved byte-for-byte in its two files.
The old D1 binary intentionally lacks it; combined runtime claims use the
fresh compiler built from this checkout. F1 acceptance belongs exclusively
to its separate recheck. All worker receipts referenced inside the mutable
implementation output were merely inspected as committed manifests; that
live directory and D2 source/build outputs were never read. No comparator
snapshot was needed. Raw compiler logs, repeated receipts and binaries stay
private under `/tmp/owned-cell-d1-review-973ca52d`, with hashes in RESULTS.json.

Conclusion: actionable D1-R1 hardening finding; otherwise the bounded dormant
scaffold and its honest refusal boundaries are reproducible. No full stage,
F1 or publication acceptance. Field authority/full contracts/current-future/
F2/F3 remain open; native body/state proofs **0**. CP1/CP2/source coupling false;
Mutex/guard/cache/REDB theorem unproved; HMAC/native DEFERRED AND UNACHIEVED.
