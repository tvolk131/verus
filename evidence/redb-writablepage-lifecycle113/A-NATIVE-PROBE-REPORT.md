# Lane A: native WritablePage lifecycle implementation/probe handoff

**Partial source implementation; native correspondence remains unchecked. All18 OPEN.**

The actual native layouts and nine original executable bodies were projected
into a private verification crate, and `mem_mut` received a three-clause checked
postcondition candidate without changing its executable statements. The final
existing-verifier application rejects the real `BuildHasherDefault` in the LRU
layout and the real standard `Mutex` in `WritablePage.buffer` during frontend
type admission. No original native executable body or authored proof body was
checked. No backend assertion, normal-drop ownership accounting, actual erased
EXEC comparison, header-to-cache relation, or native destructor effect was
established. This is an implementation/probe handoff, not an independent review.

## Exact candidate and source preservation

Final candidate: `native_lifecycle_nominal_annotated.rs` (4,037 bytes,
SHA256 `adba92c1db3cffe764e9103adb385a106eaaceb53732b73f596fe1c1c5b01609`).
The exact annotation delta is `annotation.patch`. It adds only:

```rust
pub(super) fn mem_mut(&mut self) -> (result: &mut [u8])
    ensures
        result@ == (*old(self).data)@,
        (*final(self).data)@ == final(result)@,
        final(self).offset == old(self).offset,
{
    Arc::get_mut(&mut self.data).unwrap()
}
```

These are attempted postconditions, not accepted facts. The returned byte
slice would need to expose the page's original bytes and propagate its final
bytes back to that same page, while retaining the offset. No pending-cache
premise, destructor postcondition, no-unwind promise, trusted producer,
external type/body declaration, or primitive law was added. The cache-return
effect is deliberately not inserted as an assumed postcondition.

`native_lifecycle_nominal.rs` and `native_lifecycle_nominal_restored.rs` are
byte-identical at 3,860 bytes, SHA256
`76fc1a1e898f87097a51b71c5e6708a64b4172846fc2ef76b357c7fb4b06c4ef`.
The final nominal restoration was compared as bytes, not run as a sixth
application. Application04 separately ran a byte-identical restoration of the
earlier baseline extraction. No positive result exists; neither restoration is
presented as the positive/semantic-negative/corrected control sequence required
for a genuine correspondence claim.

The final projection retains the exact layouts of `WritablePage`,
`LRUWriteCache`, `LRUCache<T>` and `FastHasher64`, the original `hash` submodule
and relevant type alias/re-export, and the native Default derives. It uses the
original `sync.rs` by absolute module path: the default non-`redb_no_std` branch
re-exports `std::sync::Mutex`, not a shadow lock. The nine byte-identical
executable blocks are `mem`, `mem_mut`, `Drop::drop`, `return_value`,
`take_value`, `LRUCache::get_mut`, and the three Hasher methods. The source
check hashes are in `SOURCE-PRESERVATION.json`. `native-snapshot/` contains five
complete native source files, each byte-identical to the recovered original;
`NATIVE-SNAPSHOT.json` authenticates both ends.

This is a dependency-cone projection, not the original complete redb crate:
its crate/DefId identity differs; unused native methods/types are omitted.
Application01-04 also flattened the private hasher module and omitted two
Default derives; application05 corrects those extraction limitations. All
earlier attempts and source files remain available. Even the corrected
projection is not an authenticated native producer or proof of actual glue
identity. The actual header and `PagedCachedFile::write` bodies are preserved
in the full snapshots and inspected below, but were not compiled in this probe.

## Original native path inspected before the annotation

Native source root:
`/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/recovered-native/redb/src`.

The recovered source's method at `page_manager.rs:1617` is actually in
`impl TransactionalMemory` (which starts at691). The task's
`PageManager::write_header` label identifies that file/method; no separate
nominal `PageManager` was invented. Its local branch still executes:

```rust
self.storage
    .write(0, DB_HEADER_SIZE, true)?
    .mem_mut()
    .copy_from_slice(&header.to_bytes(true));
```

Its storage field is `Arc<PagedCachedFile>`. In the original
`PagedCachedFile::write` at `cached_file.rs:881`, the actual stripe lock is
acquired, read-cache entries can be removed, an existing write-cache value
can be taken, eviction/writeback can return errors, and an existing or
freshly allocated `Arc<[u8]>` is selected. The method inserts/takes the slot
as needed and returns `WritablePage { buffer: stripe.clone(), offset, data }`.
Those branches and errors were not replaced with a success-only logical write.

The page at24 holds `Arc<Mutex<LRUWriteCache>>`, a `u64` offset, and
`Arc<[u8]>`. `mem_mut` at35 calls the real `Arc::get_mut(...).unwrap()`.
The original implicit destruction path at41 locks the same buffer and invokes
`return_value(self.offset, self.data.clone())`. Normal destruction also includes
the subsequent field glue for both Arc fields. There is no explicit-drop
rewrite in any probe.

`return_value` at91 evaluates
`get_mut(key).unwrap().replace(value).is_none()` inside an assertion.
The replacement happens before that assertion is tested; panic/error behavior
cannot be collapsed into a successful atomic logical return. `take_value`
at95 removes a Some payload, leaves the option empty, and unwraps the removed
value. The actual LRU `get_mut` at76 also stores the second-chance atomic flag
before returning the mutable value. `fast_hash.rs` selects the custom std
HashMap/BuildHasherDefault under the actual default cfg. No BTreeMap substitute
or default-hasher substitution was used. Later `flush_write_buffer` at490
locks each actual stripe, iterates `write_buffer.cache`, unwraps each payload,
and passes it to `self.file.write`; later iteration takes/drains the values.
The missing statement is still that the actual header write's bytes arrive
in those same reserved slots under this ownership and locking chain.

## Five charged applications, all retained

Every attempt directory includes exact argv/cwd/sanitized environment,
entry/preexecution/exit authentication, source snapshot, raw stdout/stderr,
resource samples, and result/output identities. The reservation in
`APPLICATIONS.json` precedes setup/launch. Setup failures count. All use one
worker, a90-second limit, the same frozen normal-drop runtime/library set,
`--no-cheating`, full crate scope, normal lifetime/erasure settings, and a
fresh absent `candidate.vir` export request. No internal-test mode, Infer,
verification filter, recovery-provider import, or gate bypass is used.

| # | Attempt | Observed result |
|---|---|---|
| 01 | Baseline original executable blocks | Setup/import failure: explicit vstd import duplicated automatic vstd loading; compiler assertion at `rust_to_vir_trait.rs:84`; exit1 |
| 02 | Same bytes, duplicate explicit import removed | Frontend refuses `BuildHasherDefault` and `std::sync::poison::mutex::Mutex`; exit1 |
| 03 | Unchanged executable blocks plus mem_mut clauses | Same two frontend refusals; exit1 |
| 04 | Byte-identical earlier baseline restoration | Same two frontend refusals; exit1 |
| 05 | Original private hasher namespace/derives plus clauses | Same two frontend refusals; exit1 |

All five fresh export paths remain absent; exact output directories were
inspected. All scopes report `complete_selected_scope_verified=false`,
`checked_exec_bodies=0`, `checked_proof_bodies=0`, and crate failure. The final
scope collects16 functions, selects11, and checks0. These collection counts
include generated methods and are not native-body verification counts.
`APPLICATION-SUMMARY.json` preserves each raw verification/scope result.

Original native executable bodies actually checked: **0**.
Authored executable bodies: **0**; authored proof bodies checked: **0**.
Authored contract clauses attempted: **3**, all on one original method.
Positive correspondence results: **0**. Backend semantic negatives: **0**.
No frontend error or setup panic is credited as a semantic-negative control.
All applications completed far below90seconds; none timed out. No additional
verifier application is needed to repeat this same unsupported-type boundary.

## Frozen engine boundary, separately from observed frontend errors

Actual invoked executable:
`/private/tmp/verus-crash-consistency94-20261105/drop-effects-parent-002/runtime/rust_verify`,
SHA256 `ce2a395d48a5d51f8ec38625344a615c48f2502030bbfc94874fb76e595141ba`.
Matched `libvstd.rlib` SHA256
`f75c880e31c8b13b5347960cacb5875d63341246ee3b3dbc83c390ffaf32e589`;
matched `vstd.vir` SHA256
`f0158271ffaf8f740b8a649db52ad71b956c551c01a593394584d54dc3b6bbcc`.
Sysroot: `/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin`.
All nine runtime files match that candidate's `runtime-binding.json`.
No Recovery021 binaries/metadata are combined with this different runtime.

The inspected, bound engine source is `staged/source/rust_verify/src/checked_normal_drop.rs`,
SHA256 `d9495f9d4b70f5a3feb86aebe43392f097294e8cf8f1a9d5cb98a79a79cb5e44`.
Static source inspection identifies the actual mechanism that would have to
follow type admission: Builder origin retention and elaborated MIR capture;
normalization and ownership accounting; exact resolved glue comparison; VIR
rewriting/call construction; and actual erased EXEC recapture/comparison.
The original interface is `--experimental-checked-normal-drop`; the
`verifier.rs:2914` integration retains the full-scope/lifetime/erasure/import
gates and rejects metadata export when checked effects are used.

`supported_type` at103 permits only the concrete u64/reference subset;
`drop_impl` at237 rejects nested field glue; `prepare` at771 requires a local
unconditional checked Drop body, mask-none and no-unwind. WritablePage's Arc
fields, Mutex/guard calls, returned mutable slice, branching cache operations,
and field glue are outside that subset. The source's generated
reference-retirement assumption at696 is the inherited, independently reviewed
conditional/unadopted rule, not authority to generalize to these native fields.
The matched vstd smart-pointer source provides Arc construction/default
contracts but no native Mutex ownership/poisoning and Arc::get_mut lifecycle
bridge for this path. These are source observations, not backend failures from
this probe: all applications stop before the normal-drop preparation mechanism.

No sound source-only repair of that first native ownership mechanism can be
checked by this existing executable under the no-build/no-new-trust limits.
Adding the suggested external type declarations would not prove ownership,
poisoning, reservation identity, alias uniqueness or field glue. A fake supported
primitive, widened whitelist, unconditional destructor effect or hypothetical
affine event producer would assume the missing edge. No such change was made.
The deliverable is therefore the refused source-preserving annotation candidate,
not an uncompiled engine patch represented as an implemented lifecycle bridge.
The live affine `native_resource.rs` remains hypothetical and supplies no
producer or E2 evidence here.

## Authentication, limits and preservation

First real clock: **2026-11-08T14:30:38Z**. Own absolute deadline:
**14:50:38Z**, earlier than the parent's15:17:06Z handoff deadline.
First charged application reserved at14:33:09.727992Z. Final observation before
report writing:14:38:41Z, with3,319,571 scratch bytes and68,722,753,536 free
bytes. Final seal records the later exact time/resources. The512MiB limit and
strictly greater than16GiB free-disk floor hold in every observed sample.
These are boundary samples, not continuous global filesystem monitoring.

The first own primary-input manifest contains44 records. Later manifests grow
to47 and51 as additional bound engine/vstd files and libLLVM are inspected.
For each application, every recorded primary path authenticates at entry,
preexecution and exit; common records also match final authentication with zero
drift. The actually invoked rust_verify, Python, macro executable dylibs, all
nine frozen runtime files, and rustc driver are in the initial authenticated
set. libLLVM and newly inspected source files were added later; their first
application-wide entry coverage is not retroactively claimed for application01.
The LLVM shared-library entry omission in the first setup-failed application
is an authentication limitation, not proof of a different runtime.
System shared-cache libraries are not exhaustively authenticated. Z3 is pinned
as the configured solver, but no backend/solver execution is claimed from
frontend refusals. The Mach-O inspection in `dependencies-*` gives load names,
not process tracing. The original tool transcript also contains the exploratory
commands; no exhaustive transitive OS/harness execution attestation is claimed.

Runner versions are retained byte-identically to the identities in the attempt
receipts: `runner-v1.py`, `runner-v2.py`, and `runner.py`. The first observation
set is retained in `observation-01/` before the fifth application's summary.
The original two missing guessed fixture/source paths and transient runner
snapshot newline mismatch are documented in `EXPLORATORY-FAILURES.md`; none
launched an uncharged verifier or supplied proof credit. Raw compiler panic
and all type-refusal diagnostics remain in their original attempt directories.

Both parent audits individually hash all1,483 baseline records against the
documented two expected GOAL/STATUS updates. There are **zero reconciled
mismatches and zero entry-to-exit parent changes**. This is content integrity,
not Git object-store or historical native-path restoration. No parent/native/
sealed111/112/old evidence file was edited. No runtime/toolchain/cache was
copied. All authored files were created/changed with apply_patch, and generated
logs/snapshots were written only in this fresh scratch directory.

Builds, Cargo/library/application compilation, `--compile`, native application
execution, network, Git writes, parent integration, new trust and subdelegation:
**0**. The user overrides the recovered native AGENTS.md build/test recipe;
no `just test` was run. No goal service API was used: last observed status stays
BLOCKED and was not agent-resumed. Historical NO-GO112 is superseded only for
the precisely reopened local implementation/proof attempt. S0/Infer/full-scope/
import/export rules, excluded pipelines, conditional/unadopted TCB, native-path
FAILED, Git UNAVAILABLE, compiler/source UNESTABLISHED and all18 OPEN remain.
Hashes are integrity evidence, not build or native correctness proofs.

## One concrete next action

Parent should reconcile application05's exact two nominal-type refusals and
the original lock/Arc/field-glue chain above with Lane B's source-only findings,
and record the checked native Mutex/guard ownership-and-poisoning mechanism
(with custom-hasher admission as an additional frontend prerequisite) as the
next implementation dependency. No further native proof, review acceptance,
source integration or cell closure follows from this handoff.
