# Source-only next-step research: native fetch-image boundary

## Decision

No presently authenticated checked import discharges either missing native link.
Select **one engineering task: a fail-closed, source-preserving native fetch-image
binding capability probe**. Do not commission another supplied-children theorem.
The smallest useful candidate is an actual `PageImpl` buffer/view provenance
bridge, not a new separator-routing lemma. Its feasibility under the retained
engine is unestablished; a missing Arc/resource contract is a blocker, not a
license to add one as trust. No engine or trust adoption decision is made here.

This is research, not independent acceptance review of102, execution of103, or
proof evidence. All18 native redb cells remain OPEN; no new trust or proof credit.

## Provenance and exact inputs

Parent P = `/Users/tommyvolk/.t3/worktrees/verus/t3-60978068`.
L = `/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007`.
B = `/private/tmp/verus-crash-consistency94-20261105/native-branch-routing-023`.
N = `L/recovered-native/redb/src`. Full paths and SHA256 identities of every used
source/report are in `inputs-entry.sha256` and `inputs-exit.sha256`.

Native target pin: `c82ab865bb362bc4af067d332adc44978ff64813`.
Original native paths FAILED; Git continuity UNAVAILABLE, inherited unchanged.
These are locally recovered source-content identities, NOT native Git ancestry.
No Git commands, restoration, network access, runtime inspection or execution.
In particular N/tree_store/btree_base.rs is
`c3a1f5ca0cefdde0bc7d1b40536bc4f2d68a8bce852115433621a8b37fb9ec60`.
No concurrent103 output or scratch was read or modified.

Read the three requested102 reports, completion103 PLAN and checked-generic.rs.
103's fixture repairs leave the native fetch/range premises unresolved.

## What the native bodies actually connect

1. **Descent:** N/tree_store/btree.rs:1087 `get_helper` uses
   `descended.as_ref().unwrap_or(page)` at1089-1090, reads that page's tag and
   memory, calls native `BranchAccessor::child_for_key` at1103, then performs
   `descended = Some(self.mem.get_page(child_page, self.hint)?)` at1106. The next
   iteration reads that returned PageImpl. Native leaf handling at1094-1098
   searches its bytes and constructs an AccessGuard from that page and range;
   it does not search an independently supplied child array. Depth exhaustion
   returns Corrupted at1108. A two-level client cannot erase this error path.
2. **Forwarding:** N/tree_store/page_store/page_manager.rs:103 defines
   `PageResolver` holding `Arc<TransactionalMemory>`; :112 forwards get_page
   unchanged to `self.mem.get_page(page_number, hint)`.
3. **Actual fetch construction:** the same file:1922 checks page order, computes
   the address range, obtains `mem = self.storage.read(range.start, len, hint)?`,
   and returns `Ok(PageImpl { mem, page_number, ... })` at1947. The exact `mem`
   result and requested label are carried into the constructor. This is source
   dataflow evidence, not a checked storage/ownership theorem. :1266 rejects
   excessive order only; it does not prove file-fit, allocation or range validity.
4. **View:** N/tree_store/page_store/base.rs:180 stores `mem: Arc<[u8]>` and
   `page_number`. Its original `impl Page for PageImpl` at212 returns
   `self.mem.as_ref()` at214 and `self.page_number` at218. This exact native body
   is the first useful candidate for a checked immutable-view binding.
5. **Backend is not one read-from-file axiom:**
   N/tree_store/page_store/cached_file.rs:651 `PagedCachedFile::read` returns an
   Arc from the write buffer (None hint), read cache, committed write buffer
   (Clean hint), or a direct-read buffer (:713), with cache bookkeeping. Cache
   branches assert length only in debug. Equal page labels alone do not identify
   which bytes/snapshot were returned. Current inquiry stops at these read paths;
   it does not investigate write, growth, drop or mutation pipelines.

## Exact checked-interface gap, not new routing work

B/branch-003.rs:12 defines a projected `Page` trait with ghost `bytes()` and the
contract `memory().view == bytes()`. Its `get_page_number` has no image-binding
postcondition. B/clients-003.rs:9 implements that trait for **BytesPage**, a
borrowed slice plus an independently supplied page-number field. :41 route_u32
constructs a BytesPage with a dummy zero page number, then uses the existing
accessor; returned child numbers come from decoded branch bytes, not that dummy.
Neither this adapter nor B/branch-003.rs:114 new establishes any relation to the
native Arc-backed PageImpl or PageResolver. Branch023's existing routing,
partition/uniqueness and eleven immutable accessors receive NO new credit here.

L/consumer.rs:48 search_u32 requires `ordered_leaf(page@, widths)` and searches
the passed slice. Retained102 checked-generic.rs:25 `supplied` requires every
leaf's order and every key's adjacent-separator interval; :99 searches
`leaves[selected]`. Its page-number result does not bind that slice to the
PageImpl fetched by native get_helper. No inspected provider exports a native
PageImpl/Arc/resolver success-and-view contract. This is a scoped interface
finding, NOT an exhaustive assertion that Verus cannot support Arc.

## Why the second link is independently missing

For separator S[j], the needed native descendant-key invariant is:
all keys of child j are <= S[j], and all keys of child j+1 are > S[j], together
with outer ancestor bounds, under the same comparator and actual page images.
For more than one level these are descendant bounds, not just branch-local
separator sortedness. They must first be established on reachable native pages
and preserved by permitted tree transitions; read-only descent can only carry an
already established invariant. It cannot manufacture it from routing uniqueness.

N/types.rs:237 specifies `left <= separator < right`; :251's default returns
left with a debug less-than check. Neither body proves that its arguments are
actual child extrema, that serialized child slots refer to those children, or
that all transitions preserve the relation. No producer proof is supplied here.
N/tree_store/btree.rs:950 verify_checksum_helper checks depth/cycles/checksums
and traverses decoded children. It carries no comparator/child-range bounds;
checksum equality is not a separator-range theorem. This source body cannot
discharge102's `supplied` interval premise either. No partition100/admission101
replay, arbitrary tree-invariant axiom, or renamed routing theorem repairs this.

## The ONE selected task: native fetch-image capability probe

**New property/domain.** Target successful original native fetches, using the
exact source-defined cfg and actual Arc-backed PageImpl, not a new BytesPage or
caller-provided child table. Check whether existing, authenticated engine
interfaces can express and prove: (a) returned page_number is the requested
number; (b) the returned memory slice is the immutable byte view of the **same
Arc result of that invocation's storage.read**, across resolver forwarding and
the native PageImpl::memory body. This is return-buffer provenance, not proof
that storage.read supplies the correct on-disk snapshot. Preserve Result errors,
actual address computation and native field/control flow. Native debug code is
selected by cfg, not deleted to avoid obligations.

The smallest candidate is PageImpl::memory's view contract for an actual native
page. A child view must be constructed FROM that fetched object, not assumed
equal to its bytes.
Even this candidate is gated on pre-existing justified Arc unsized-slice/view
semantics and a checked native-to-projected PageNumber representation relation.
Do not silently identify nominal types or treat field labels as image identities.

**Precise dependencies/stop gate.** Establish support for native Arc<[u8]>
borrowing/as_ref and immutable-view lifetime, native Result/? forwarding, and
representation binding to the imported codec. Full get_page additionally needs
checked address arithmetic/layout and an operational read-result binding, with
cache/snapshot identity and borrow stability if claiming actual page-store
correspondence. These are not exported by the inspected seven-crate composition
interface. Unsupported operations must yield a minimal source-anchored gap
reproducer; no external_body shim, assume/admit, ghost store axiom, abstract
fetch oracle, replacement native body, or new runtime/trust contract. Consequently
there is currently NO justified unconditional native fetch theorem to execute.

**Positive and localized controls, only if the gate passes.** Positive covers
arbitrary returned bytes and successful native page labels, not six fixed
fixtures; absence/error outcomes remain unchanged. Keep native read/route calls.
One negative substitutes a distinct buffer with the SAME label and must fail
only the provenance assertion. A second substitutes another label while retaining
the correct buffer and must fail only the requested-number assertion. Derive
distinct buffers/labels in checked callers; do not add desired-result premises.
Restore the byte-identical positive. Frontend/borrow/precondition/resource errors
are blockers, not semantic negatives. No such controls were run here.

**Capped plan for a separately authorized worker.** One worker, <=15min total,
stop optional work at12min; <=128MiB fresh scratch, >16GiB free disk. First
authenticate the native anchors above and inspect only the relevant existing
Arc/view/codec contracts. Record original-source-to-annotated-body byte mapping.
If a checked resource/representation binding is absent, STOP with a minimal gap
artifact and exact missing obligation, before writing a closure theorem. Only
if support is already justified: <=6 charged verifier applications, <=60s each,
one verifier/solver worker, no builds/exports/native execution; full-local-scope
positive, two localized negatives, positive restoration, with at most two
corrections. Keep all logs and auth receipts; stop on exhaustion or mismatches.

**Operational bindings/inherited boundaries.** Existing retained release
VIR/rlib pairs only: B attempts001 leaf_lookup023,002 pagecodec023,019
branch_routing023,020 branch_clients023; L attempt009 leaf_decoder with leaf-final
VIR,010 builder_step with builder-final VIR,011 lookup_client with client-final
VIR. Retain102's midpoint runtime, Rust1.99.1 sysroot, exact argv/environment,
no-cheating, single worker and release cfg, reauthenticating selected artifacts
before execution. Those binary/runtime/sysroot files were NOT opened or audited
by this research. Keep inherited midpoint/LE/std/slice/array/StructuralEq,
comparison/loop/adapter/compiler/solver qualifications; no new native ownership,
allocator, I/O, panic/glue or snapshot claims. Range establishment/preservation
remains a separate unsolved dependency, not a second task authorized here.

## Research accounting and authentication

Conservative start: 2026-11-08 01:39:37 UTC, i.e. Nov7 19:39:37 CST.
Optional-analysis cutoff 01:43:37 UTC; absolute deadline 01:45:37 UTC.
Only bounded source reads/searches, hashes, disk accounting and report writing.
Zero verifier/solver/compiler/build/test/native execution; no subdelegation,
parent edits, Git writes, network, sysroot audit or main103 interference.
No refused growth049/zero-growth050/mutation030/padding95/drop work investigated.
Initial disk free 56,314,308 KiB (>16GiB). Final time/disk/scratch accounting is
in ACCOUNTING.txt. Report must remain <=12KiB; entire scratch <=8MiB.

Entry hashes precede substantive source inspection; one branch-003.rs anchor
discovery scan preceded its recorded entry hash. This ordering limitation is
disclosed rather than claiming perfect before-first-read authentication. All
used inputs receive a recorded entry and exit hash; equality results are in
AUTHENTICATION.txt. No contents were repaired on mismatch. Handoff hashes cover
report, both input manifests, authentication receipt and accounting. Finalization
includes anchor/size checks at01:44:09 UTC; no further novelty search followed.
