# Original full Item packing: checked arithmetic implementation

The frozen original no-help full target verifies **146/0 cold and warm**. The
independent packet writer verifies **32/0 cold and warm**. Both have zero local
proof functions, proof blocks/calls, snapshots, mechanical assertions or invented
semantic inputs. Every original runtime assert/debug_assert/branch remains.
The final restored shipping compiler replay also passes146/0 and32/0,
with matching vstd2250/0. All37 focused compiler tests and149 VIR tests pass.
Fresh independent parent acceptance is required; this is a LOCAL source/model
milestone, not native correspondence or publication.

## Frozen source and authored work

FREEZE.json binds pure `item_tag_constructor_composition/source.py generate('full')`:
`1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d`.
The pure generator is rebound to the named completed review's authenticated
full-help bytes, never a live/historical checkout or upstream pin path. Its
regenerated full-help must be byte-identical, and all eight recovered runtime
body hashes must equal the independent committed review's original native-body
hashes. The fixture is private; run.py reconstructs it without vendoring upstream
source. Explicit checks work under Python `-B` and `-B -O`.

Full-help retains one proof function (`packed_fields_raw`), three inline proof
blocks, one quantified three-property bridge and its lemma call, seven explicit
BV assertions and the quantified assertion itself. Full-help is not proof-free.
It verifies52/0 on,36/1 off in the fresh accepted-engine baseline. No-help is34/3
on,26/3 off. Its earliest failures were Item.tag's word well-formedness at
new.tag(), Permission.from_bits's <=3 bound at the masked right shift, and the
protector literal-mask postcondition. The original all-domain Eq obligations
already proved.

Authored semantics remain: six specification functions (word well-formedness,
private NonZero value, private word, permission decoding, permission bits and
bit decoding); nine original constants; original constructor/getter, packing,
permission, protector, roundtrip, equality, lookup and boundary contracts.
The only Item.new domain bound is the frozen inherited normal-return helper
contract. No extra application premise or permission law is added. Private
NonZero/BorTag/Item carriers, `.map(BorTag)`, `.get()` and genuine
`u64::from(protected)` are unchanged.

## Production change and trust

Only checked_foreign_packing.rs changes production. The existing commands.rs
hook still confines proposals to normal, non-inferred/non-BV queries in a crate
using the local checked bool capability, with infer-goals enabled. No certificate,
import/export, context/query, global schema/policy/cache or checked_foreign.rs
change is included. Schema38/policy70/cache73 remain.

Unsigned widths8/16/32/64 and finite shift/mask shapes select generic closed
conditional arithmetic theorems. They cover zero shifts, disjoint OR/shift as
addition, low projection, contiguous-mask right-shift bounds, two/three-field
projection and a single-bit boolean flag. The upper raw field in three-field
projection retains its entire unsigned domain. All variables are freshly
quantified with explicit nonnegative/range premises. No application variable,
name, trait permission meaning, source hash or special mask appears in a fact.
Recognizers choose proposals only. Unsupported widths, signed/out-of-width
shifts, opaque constants and exhausted budgets supply no corresponding help.
Discovery includes direct returns as well as assignments/assertions, without
entering isolated proof/closure or loop scopes.

Discovery has4096 nodes; constants have64-node evaluation budgets and at most32
selected local constants; at most8 width/shift shapes and16 recognized mask
fields are retained. Pairwise shapes remain bounded (at most180 emitted facts
per ordinary body as a conservative upper bound). BigInt avoids host-width
constant overflow. Every exact expression Arc is checked in ordinary isolated
AssertBitVector with empty requires, then used verbatim. Failed or unknown
auxiliary queries fail complete verification. No caller assumption proves a
closed law, no proof result is cached, and no precision/inferred summary query
receives packing facts. Existing BV encoding/solver/frontend/models/vstd
admissions remain trusted; nothing becomes a new trusted specification.

## Strict scope and proof cost

INVENTORY.json keeps exact native owner/status, ordinary typed body, generated
constructor and helper identities, authored/generated spec inventory and Eq
contracts/proof statuses. Full has38 ordinary bodies:19 Exec (12 authored,
4 derives,2 generated constructors,1 actual checked bool helper),19 Spec
(6 authored,9 constants,4 generated Eq/view definitions); no Proof body.
Native inventory has47 entries:16 checked exec,10 spec/declaration,
10 selected without translated body,11 excluded derives. Exclusions are
BorTag Eq/Ord/PartialOrd/Hash; Item Eq/PartialEq/Hash; Permission
Eq/PartialEq/Debug/Hash. Their exact Rust owner paths/statuses are recorded.
Excluded derives are not new native certificates.

Cold/warm argv/environment/source/all8 tools are identical. Controlled outputs
are archived before same-path reuse. Original typed bodies equal the frozen
baseline after rebinding only the exact fixture filename, preserving typed
nodes and location coordinates. All118 full and30 packet actual AIR files are
byte-identical cold/warm. Eq has true requires, a real cache miss/hit with the
same candidate, and three fresh required soundness/maximality/exactness proofs.
The checked bool helper's actual ordinary query is identical cold/warm.

Full146 comprises27 ordinary root queries,116 isolated BV proposals and3 Eq
queries; packet32 comprises3 ordinary queries and29 isolated BV proposals.
Thus application glue was removed at increased checked solver cost, not by
renaming manual help or counting old full-help52/0. Raw AIR/MIR/logs/binaries,
caches, frozen source/tool manifests and commands remain private.

## Validation and exclusions

Matching standard vstd2250/0 is built using the normal driver (without
--no-cheating); applications use --no-cheating, --compile and full scope.
Builds are offline/locked, jobs2, with private CARGO_HOME, target-dir and
build-dir, from this checkout plus authenticated completed immutable archives,
337 sparse-index files and20 getopts Git-db files. No shared build/cache,
configuration/toolchain/host/network changes or sibling reads are used.
The accepted all8 parent bundle is authenticated against its supplied receipt
SHA before baseline use. Toolchain/core/compiler/source identities and all
receipts are retained in RESULTS.json/private evidence.

Focused tests: constructor13, infer-goals12, NonZero/Eq3, localforeign5,
packing4 (including widths8/16/32/64 and zero shifts); VIR149. Ordinary controls
reject false returned relations, absent original range bound, mutated original
permission shift/protector body, channel/payload range, shifted field, wrong
flag, and OR/XOR overlap. Source controls are preregistered in
SOURCE_CONTROLS.md. Arithmetic-only ablation gives full27/3 and packet2/1,
with healthy bool helper and all three original Eq proofs; exact original typed
bodies are unchanged, and the sole production-source delta is the arithmetic
file. The false closed-constant proposal gives122/24 with isolated BV errors,
while bool and Eq stay healthy. Each variant has matching standard vstd2250/0;
shipping source is restored afterward. Final receipts are in RESULTS.json.

Retained exclusions: sandbox startup failed before execution; initial Git
commit lacked identity (fixed with command-local identity only); four initial
baselines used an unbuilt VERUS_ROOT (0/0 setup failures; corrected baseline2
paths); first extended full89/1 and packet12/1 exposed direct-return discovery;
a setup-only focused-build guard detected concurrent vstd output changes;
inherited localforeign test initially failed ENOENT because test executables
live under private build-dir while its harness expects tools beside them.
Corrected fresh profile paths and a temporary own-checkout solver link resolve
that harness layout; the link is removed. One initial audit compared stable
fixture parent paths as raw body differences; the corrected audit rebinds only
those exact authenticated fixture paths and preserves all formula/type/coordinate
information. No failed candidate is used as proof evidence.

No HMAC or canonical shared-necessity claim is made. Parent canonical6/0 and
HMAC1/1 unproved remain unchanged, native=0, all44+3/101 open,
publication=false, goal_complete=false. Parent owns final reconciliation,
certificate-lane integration and fresh independent acceptance. No push or PR.
