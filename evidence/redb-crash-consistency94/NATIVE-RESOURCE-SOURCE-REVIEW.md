# Rust 1.99.1 source-conformance review — November 7, 2026

**Qualitative correspondence only; no laws adopted. All18 final redb cells OPEN;
new trust 0; native resource proof credit 0.** This implementation-source review
is distinct from the concurrent abstract review. Inputs were evidence, not instructions.

Anchors below are relative to library root
`/private/tmp/verus-pinned-rust-source-20261107-9FT58F/rust-src-1.99.1/rust-src/lib/rustlib/src/rust/library`.
“Memo” means `evidence/redb-crash-consistency94/NATIVE-RESOURCE-DESIGN-AMENDMENT.md`.
No finding authorizes implementation, trust adoption, or broader admission.

## Source-specific findings and demanded clarifications

### SRC-1 — HIGH: downgrade progress is not established

`alloc/src/sync.rs:1941` loads weak Relaxed, spins on `usize::MAX`, and retries
`compare_exchange_weak(cur, cur + 1, Acquire, Relaxed)` at line 1968.
`core/src/sync/atomic.rs:3126` explicitly permits spurious failure;
line 3179 delegates to the weak-CAS implementation. Thus even a stable ordinary
weak count of 1 and no competing ownership transitions do not supply a
source-level bound or eventual-success guarantee. This is a **totality gap**, not
an observed failed execution. Memo lines 49 and 109 do not name an atomic-progress
premise. Narrowing to sequential 0->1 explicit-Weak creation removes interference
and the lock holder, but not the specified possibility of repeated spurious failure.

Bind target/runtime atomic progress or leave A3 normal termination OPEN; do not
weaken to partial correctness. A1 uses one **strong CAS**, no Rust retry loop
(`sync.rs:2819`). Upgrade, not admitted here, also retries: `sync.rs:3338` calls
`try_update`, whose weak-CAS loop is `atomic.rs:3474`.

### SRC-2 — HIGH: panic=abort does not establish terminal Abort

O1 None, invalid range, copy length mismatch, false assertions, and downgrade
overflow enter **panic machinery**, not necessarily an immediate abort instruction.
`core/src/panicking.rs:60` bypasses the handler only for the distinct
`panic = "immediate-abort"` cfg. Otherwise it calls `panic_impl` at line 80.
`std/src/panicking.rs:612` is the handler; line 677 selects static-string versus
formatted payload, then `panic_with_hook` (`:777`) obtains the hook lock and runs
the default or custom hook (`:806`, `:822`) **before** `rust_panic` (`:840`).
`rust_panic` at line 886 reaches `__rust_start_panic`; the abort runtime body is
`panic_abort/src/lib.rs:33`, then `std/src/rt.rs:32` calls process abort.

A hook can perform effects, diverge, or terminate differently; default-hook,
formatting, lock and output progress are also unproved. The rely does not exclude
hook nontermination. `!` forbids an ordinary returned value, not divergence.
Bind this runtime closure and progress, or prove failure edges unreachable;
otherwise Abort stays OPEN. Do not substitute immediate-abort for panic=abort.
No Abort frame/destructor completion is claimed. Arc clone overflow instead
directly calls intrinsic abort (`sync.rs:2434`, `intrinsics/mod.rs:390`).

### SRC-3 — MEDIUM: Weak accounting needs transients/origin restrictions

For an ordinary live block at quiescent boundaries, the correspondence is
`strong = |S|`, `weak = |W_explicit| + 1`: **one collective implicit weak**, not
one per strong. `sync.rs:437` initializes both counters to 1; `:2138` constructs
the collective weak guard when the last strong destroys the payload.
`Arc::weak_count` (`:2002`) subtracts 1, and reports 0 for the uniqueness-lock
sentinel. It is not an interchangeable observation of the raw counter.

`is_unique` (`:2811`) CASes weak 1->`usize::MAX` with Acquire/Relaxed, reads strong
with Acquire, and stores weak=1 Release even when strong>1. A1 therefore writes
control-block state transiently on both the Some path and an extra-strong None
path. Interpret the memo's conservation/frame as data bytes, stable inventories,
and identity—not “no control-block memory write.” The sentinel is not an
explicit Weak token and must not satisfy a stable inventory equation.

Origin caveat: `Arc<[T]>::default` (`:3866`) for alignment <=16, including u8,
clones an immortal static strong (`:3822`, `:3879`). One accessible empty Arc
then has strong>=2. Exclude this origin from 1->2->1 or account for its permanent
static strong; emptiness proves no uniqueness. `Weak::new`'s dangling sentinel
likewise contributes nothing to a live C (`:353`, `:3390`).

### SRC-4 — HIGH: allocator effects and generated destruction remain unbound

`Arc::clone` (`sync.rs:2405`) does a Relaxed strong fetch_add and then
`self.alloc.clone()` (`:2438`); downgrade also clones the allocator (`:1972`).
The exact bound is `A: AllocatorClone`, whose safety documentation forbids
unwinding on clone (`core/src/alloc/mod.rs:518`), **not arbitrary effects,
divergence, or abort**. Arc and Weak each store `alloc: A` (`sync.rs:274`, `:353`);
its field destructor remains part of generated glue even when the user-written
counter-drop body returns early. An arbitrary allocator can defeat the proposed
normal-return or unrelated-resource frame law.

Default `Global` is a unit struct with derived Clone and no custom Drop here
(`alloc/src/alloc.rs:55`), narrowing the closure. Bind it and generated Clone/
field glue, or review another allocator's effects/progress. `mem::drop<T>`'s
empty body (`core/src/mem/mod.rs:1045`) still drops its argument. Compiler-replaced
drop glue (`core/src/ptr/mod.rs:845`), typed flags, moves, allocator fields and
cleanup/recursive glue remain in A3 or its canonical encapsulated closure.

## Primitive/body correspondence (conditional support, not certificates)

### A1 / A2 — Arc uniqueness and read projection

- `get_mut` (`sync.rs:2673`) branches on `is_unique`: true returns
  `Some(get_mut_unchecked(this))`; false returns None. The unchecked projection
  (`:2748`) creates `&mut (*ptr).data`, deliberately **not** a reference covering
  the count fields. No allocation, byte initialization, or data copy occurs.
- Under complete ordinary-block inventories, stable/synchronized boundaries,
  exclusive access to h, the proposed interference rely, and valid permissions,
  this supports Some iff S={h}, W_explicit={}. An extra strong with weak=1 gives
  strong>1 after locking; any explicit Weak makes weak>=2 and the CAS fail.
  Neither failure supplies a writable lens; observed counts cannot replace the rely.
- Upgrade is a relevant excluded interference, not “harmless because weak was
  later dropped”: `Weak::upgrade` (`:3313`) refuses strong=0, otherwise attempts
  an Acquire/Relaxed increment and clones the allocator. An upgraded Arc can
  outlive the last explicit Weak, leaving strong>1. Sequential retirement must
  account for every such contribution, or forbid that history as proposed.
  Source atomics do not discharge inventory, happens-before or lifetime premises.
- `Deref for Arc<T,A>` (`:2449`, body `:2453`) returns `&self.inner().data`;
  `inner` (`:2127`) uses `NonNull::as_ref` (`core/src/ptr/non_null.rs:450`).
  This supports unchanged data/metadata, no allocation/count increment and a
  shared result—not invented permissions, loan-end delimitation or write authority.
  A2 yields `&T`, and A1 yields `Option<&mut T>`: the exact `[u8]`/array/container
  substitution and any extra deref chain must be bound, not guessed.
- `is_unique` and `get_mut` are not resource-interchangeable just because their
  booleans agree: the former's pointer-invalidation distinction is explicitly
  documented at `sync.rs:2783`. Lending/suspending/restoring authority remains a
  prospective resource interpretation, not a property proved by these bodies.

### O1 — unwrap transports the payload, without a new owner

`core/src/option.rs:1011` matches self by value: Some(val)->val;
None->`unwrap_failed` (`:2247`), which calls core panic with a static message.
For `Option<&mut [u8]>` this supports one moved contained reference, not cloning,
copying its bytes, allocating, or running a payload destructor on Some. The
payload's referent and transported loan must remain identical. The successful
branch is straight-line; the None outcome has SRC-2's open runtime closure.
Option's niche/layout and MIR move/reborrow correspondence are compiler bindings,
not additional consequences of this source match.

### S1 — length, indexing, coercions, and empty references

`[T]::len` (`core/src/slice/mod.rs:116`) calls `ptr::metadata`
(`core/src/ptr/metadata.rs:100`), an intrinsic boundary. Slice Index/IndexMut
wrappers (`slice/index.rs:18`, `:31`) delegate to the resolved SliceIndex.
For **ops::Range<usize>**, `:405` / `:419` use checked_sub(end,start), then
end<=len; success produces a pointer offset by start and metadata end-start
through helpers `:83` / `:95`. Failure calls `slice_index_fail` (`:39`): it
reports start>len first, then end>len, then start>end. This supports the memo's
0<=s<=e<=length domain, but does not prove caller arithmetic before construction
of the range or absolute allocation offsets. Different range types need their
own binding: RangeTo delegates at `:502`; RangeFrom checks start and subtracts
at `:543` / `:556`; RangeFull returns its input at `:635` / `:641`.

`[k,k)`, including `[len,len)`, passes these checks and preserves derived origin.
Raw slice validity (`slice/raw.rs:14`, `:150`) retains non-null/alignment even at
zero length, single-allocation/initialization where applicable, lifetime access
constraints, and isize::MAX/nonwrapping extent bounds. Empty pointers may be
dangling-but-valid-for-zero or one-past: require neither a readable first byte
nor a fresh nonempty allocation. Zero length repairs no invalid reference.
An Arc lens retains live C/A origin; it cannot mint other empty ownership.

Unsizing is **not an executable library implementation body**:
`marker.rs:199` states Unsize impls are compiler-provided; `:235` is the empty
lang-item trait; `ops/unsize.rs:42`, `:48`, `:58` declare lifetime-constrained
reference coercions. Allocationless same-origin `[T;N]`->`[T]` with length N and
proper shared/mutable reborrowing requires a bound compiler cast/metadata/borrow
interpretation; library source does not close this gap.

### S2 — exact u8 copy is a raw-copy helper, not an element-Clone loop

`slice/mod.rs:4339` requires T:Copy and invokes `copy_from_slice_impl` (`:5582`).
That helper checks lengths **before** the copy and calls len_mismatch_fail
(`:5588`) if unequal. It then calls
`ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), dest.len())` (`:5605`).
The helper's T:Clone signature calls no user Clone; public T:Copy and exact u8
matter. It does not dispatch to an element loop; clone_from_slice's separate
specialization is not this primitive.

`core/src/ptr/mod.rs:552` optionally checks unsafe preconditions then delegates
to intrinsic copy_nonoverlapping (`:573`; declaration `intrinsics/mod.rs:3253`).
For initialized u8, valid exclusive destination/shared source, equal bounded
lengths and compatible nonoverlap, this supports the source-prestate byte update,
unchanged source bytes, and frame outside destination. It does not prove the
intrinsic/backend or permission model. The memo's whole-target source separation
is stronger than the helper's required separation of copied regions; keep that
restriction rather than crediting same-allocation split-copy by implication.

At length 0 there is no byte update. The intrinsic wrapper's zero-size check
can relax non-null checks, but the **incoming references** cannot be null.
`ub_checks.rs:159` computes nonoverlap by distance>=size; size 0 does not demand
different addresses. This agrees with the memo. The optional checks explicitly
cannot be relied upon for safety (`ub_checks.rs:68`); invalid permissions/UB do
not become a specified Abort domain merely because debug checks sometimes fire.
Temporary call borrows must be mapped/ended separately from underlying loans.

### S3 — u8 equality specializes; reference wrappers and assertion expansion matter

Slice PartialEq (`slice/cmp.rs:19`) first compares lengths, returning false on
mismatch. For equal lengths the BytewiseEq specialization (`:146`, body `:151`)
does unchecked_mul(len,element_size) then `compare_bytes(...) == 0`. u8's
BytewiseEq impl is `core/src/cmp/bytewise.rs:32`; size is 1 and valid slice
extent discharges multiplication overflow. `intrinsics/mod.rs:2523` declares
compare_bytes; its safety contract requires readability of the **whole** range,
not just a prefix before a differing byte. This supports value equality, not
allocation/owner equality, and permits compatible shared-source aliasing.

`core/src/cmp.rs:2345`, `:2420`, `:2493`, `:2509` supply shared/shared,
mutable/mutable and mixed-reference wrappers delegating through reborrows.
Array/slice comparisons are not necessarily this same final instance:
`array/equality.rs:21`, `:43` call `as_array<N>` (`slice/mod.rs:853`) and then
array equality; its BytewiseEq specialization uses **raw_eq**
(`array/equality.rs:152`, `intrinsics/mod.rs:2504`). Bind actual coercions,
const N and selected specialization; do not infer slice equality from syntax.

`assert_eq!` (`core/src/macros/mod.rs:43`) evaluates/borrows both operands once,
compares them, and calls `assert_failed` only on failure. That function
(`core/src/panicking.rs:397`, `:434`) introduces Debug trait objects and panic
formatting. With exact equality and true result these failure dependencies can
be checked unreachable; otherwise SRC-2 applies. `assert!` is compiler-built-in
(`macros/mod.rs:1738`); `debug_assert!` additionally depends on debug_assertions
cfg (`:290`). Library text alone does not prove their generated CFG or permit
dropping failed-assertion formatting/cleanup edges by name.

### A3 — bounded transitions and the precise excluded final branches

- Strong 1->2: `Arc::clone` uses fetch_add(1,Relaxed) (`sync.rs:2417`), keeping
  ptr/data identity and copying no payload bytes. It checks **old > MAX**, not
  old>=MAX, after increment. MAX is isize::MAX (`:56`). old=MAX is not the
  abort branch; wrapping atomics are not a generic bounded-counter theorem
  (`atomic.rs:3192`). Checked 1->2 is far below this boundary.
- Strong 2->1: `Drop for Arc` (`sync.rs:2864`) does fetch_sub(1,Release), returns
  immediately when old!=1. It does not call payload Drop, acquire fence or
  deallocate on this branch. Invalid zero-count histories are not given a safe
  outcome by the wrapping subtract. Exact consuming drop/glue remains SRC-4.
- Explicit Weak 0->1 means raw weak **1->2**, not 0->1. Downgrade checks
  cur>MAX **before** the CAS and panics on overflow; the bounded branch is
  safe from arithmetic overflow, subject to SRC-1 progress and allocator closure.
  Its debug assertion excludes dangling (`sync.rs:1971`).
- Explicit Weak 1->0 with original h live means raw weak **2->1**.
  `Weak::drop` (`:3533`) decrements Release, and only old==1 executes acquire
  plus deallocation (`:3544`). Thus the bounded explicit retirement does not
  free C/A or destroy T, and the collective implicit weak persists.
- Last-strong Drop instead executes acquire, a static-origin debug assertion,
  and `drop_slow` (`:2900`, `:2911`), which drops T in place under the weak guard
  (`:2138`). Last raw-weak Drop computes layout and calls allocator deallocate
  (`:3555`). These are **excluded branches**, not proved bodies. Original h
  staying live is the decisive nonfinal premise; no final-h, payload, page Drop
  or deallocation support is obtained from the nonfinal argument.

## Remaining binding / engineering obligations

1. Bind actual typed call/coercion/assertion/drop occurrences to canonical
   instances, substitutions (T, allocator, range, const N), owner environment,
   borrow regions, schema version and resource/outcome parameters. This review
   supplies no such mapping/non-null schema and adopts no earlier capture manifest.
2. Bind core/alloc/std identity and exact instantiated build: target/ABI,
   pointer width and atomics, cfg/target features, sanitization (`sync.rs:66`
   has a fence versus ThreadSanitizer load choice), debug/overflow profile,
   panic handler/runtime, library codegen settings and compiler/runtime identity.
   Bind intrinsics/atomic semantics, generated Global Clone, unsizing, assertions,
   moves/reborrows, drop flags/glue and any backend memcpy/memcmp/raw_eq lowering
   or encapsulate their exact reviewed closure. This source package contains no
   proof of compiler-generated code or source-to-installed-binary equivalence.
3. Alias cases additionally need allocator/glue closure (SRC-4), atomic progress
   (SRC-1), complete accounting including static/sentinel origins (SRC-3), and
   byte/loan/frame premises. Control predicates need checked bodies/bindings:
   Option::is_some/is_none
   (`option.rs:633`, `:682`) and any Arc count observations are not O1 unwrap.
   Weak::clone (`sync.rs:3465`), upgrade, Weak::new, Arc construction, and final
   drops are not added to bounded A3 by association; review/bind them only if
   actual reachability requires them, otherwise prove exclusion. At normal
   caller scope exit, a still-owned local h may itself incur final Drop: prove
   ownership escapes or account for that edge; naming only the earlier extra
   retirement is not whole-body closure.
4. E1-E6 remain **unproved required checked engineering**, not trusted axioms:
   affine IR transport, original CFG correspondence, native/helper/caller bodies,
   NLL ends, joins/cleanup/arithmetic and closed admission/export. No full-matrix
   resource results/caller discharge. Retain **S0 whole-native admission**,
   default-OFF/fail-closed Infer and certified import/export barriers. No admission,
   export or scope-S1 selection occurs here.

## Independent evidence chain and used-source accounting

- Retained `LIBRARY-SOURCE-BINDING.json` is byte-identical to intake
  `SOURCE-BINDING.json`; independently hashed SHA256:
  `69765c446f5044ae605ea418556011fefb2436d172c7fff9580d7ee4e06c5e70` (4,560 B).
- Archive independently hashed:
  `e485a88c45db546b73571df7bebfcb4454ba4fb552c492e25fc5b00eeb5bf22d`
  (5,863,216 B). Inventory independently hashed:
  `c99b07945ae7eee666e3a4abd1681832a141297ccdf34a30a899e39263c2d80f`
  (754,770 B). Both match the summary. Release/package metadata gives
  `58cf8f9c35b355199727627d07ae34a4e2846d67`, Rust 1.99.1.
- At entry (2026-11-07T18:49:19Z), rehashed **all 3,701 extracted regular files /
  73,147,458 B** against inventory and compared all archive file contents
  (948 directory entries; no extraction). All matched; no extra extracted files.
  Integrity is not semantic review of all files. Actual inspection/search use:
  **28 library + 2 package metadata files = 30 files / 1,710,116 B**.
- `USED-SOURCE-HASHES.tsv` records these 30 files. Each listed digest is the
  identical independently verified entry, exit, and inventory digest, with size
  checked as well. Scope `library` uses the root above; scope `package` uses
  `/private/tmp/verus-pinned-rust-source-20261107-9FT58F/rust-src-1.99.1`.
  Exit checks are repeated after writing the fresh artifacts; no source edits.
- Installed manifest and retained version-query output were rehashed against
  summary hashes/sizes. **No compiler invocation here.** No detached signature,
  full toolchain reauthentication or reproducible-build proof. Selected-runtime
  comparison remains attributed intake evidence; no prior toolchain artifact
  was opened/re-certified. The three supplied evidence files were checked
  unchanged at exit.

## Operational accounting and handoff

Only this report/hash table were written via apply_patch in the fresh directory.
Free space >126 GiB at entry, >129 GiB before writing; exit enforces >=16 GiB.
Parent/source/old artifacts unchanged. Builds (compiler/vstd/Cargo/harness),
compiler invocations, verifier/solver runs, native/semantic executions, network,
installs, Git writes, subdelegations, abstract-review polls/replacements and
old-task retries: **all 0** (one read-only git status query). Forbidden task
material was not opened/resumed/rerouted. No permission adoption, core/alloc
formal verification, native resource theorem or database acceptance.

**STOP AT HANDOFF. All18 OPEN / new trust0 / native resource proof credit0.**
