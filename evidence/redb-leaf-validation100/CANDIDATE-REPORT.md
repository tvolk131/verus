# Cycle100: bounded supplemental leaf-lookup proof repair

Date: November 7, 2026 America/Chicago (November 8 UTC).
Disposition: **minimum supplemental release target achieved, conditional on the
unchanged sealed import/runtime assumptions; independent review PENDING**.
This is a separate fresh authorized cycle, not an unchanged replay claim, an
allowance reset, a trust adoption, or a retry of provider-refused work.
All 18 final redb cells remain **OPEN**. Stop at this handoff.

## Results and original failure accounting

Five NEW direct verifier applications, serially, with one verifier worker and
60-second process-group deadlines; no additional direct invocations or builds.
Each selected the entire crate: all three original functions, plus the control
proof function in the two paired-control applications. No filtered positives.

1. **attempt-001, unchanged original:** exit 1, 2 verified / 1 failed function,
   exactly two BACKEND assertion diagnostics at original lines 14 and 17 in
   `generic_partition_bytes`. Complete-scope credit remains ZERO, checked
   exec/proof counts 0/0. Input SHA256 is exactly the sealed supplemental013
   input. This is the ONLY unchanged reproduction in this cycle. Historical
   attempt013 and its original failure classification are unchanged.
2. **attempt-002, proof-only repaired release:** exit 0, 3 verified / 0 errors,
   complete selected scope true, 3 checked executable bodies / 0 proof methods.
   BOTH original partition assertions remain verbatim and are established.
3. **attempt-003, correct mathematical control:** exit 0, 4 verified / 0 errors,
   all three original functions plus one checked proof method. The additional
   assertion is `number + 1 > number` for arbitrary mathematical `int` with
   no preconditions. An ordinary `--export control.vir` was requested and
   produced the .vir artifact (SHA recorded in the receipt and manifest).
   There is no proof-library compile or application/native execution.
4. **attempt-004, incorrect paired mathematical control:** exit 1, 3 verified /
   1 failed function, EXACTLY one intended BACKEND assertion diagnostic at
   input.rs:89:12, `assert(number + 1 < number)`. The failed-function list contains
   ONLY `review_generic::mathematical_assertion_control`. This source differs
   from the correct control by the single `>`/`<` operator, with identical
   logical flags, imports and environment modulo fresh output paths. Export
   was explicitly requested in BOTH controls. No .vir, .rlib, .rmeta or interface
   was emitted on failure; `interface_written` is false. Complete-scope credit
   is ZERO, checked exec/proof counts 0/0, despite three verified body results.
5. **attempt-005, restored repaired release:** byte-identical to attempt002,
   exit 0, 3 verified / 0 errors, complete selected scope true, 3 checked
   executable bodies. The paired extra control is removed by restoring the
   exact repaired input, not by filtering any original method.

There were zero unexpected verification failures, spawn failures or timeouts.
The two intended failing applications count toward the five. Runtime total
for the five verifier processes is recorded in fresh-summary.json (about 3.5s).
The application ledger is NEW; none of the previous 15-review/23-parent ledger
entries is removed, reclassified, or reset. No debug profile was attempted.

## Exact proof-only delta

`proof-only.patch` inserts precisely ONE 14-line ghost `proof` block immediately
after the existing index bound assertion in `generic_partition_bytes`.
For an arbitrary before/after index satisfying each original bound, it explicitly
instantiates the imported strict partition on
`<u32 as Value>::meaning(accessor.key_bytes_at(index))`, then proves the quantified
restatement on `leaf::encoded_key(page@, key_width, value_width, index)`.
The existing open accessor specification relates these representations.
The original two quantified assertion statements still follow the added block.

This checked pointwise instantiation is sufficient to bridge the two expressions
in the authenticated ordinary interface. It neither assumes a missing theorem
nor changes a provider contract. The successful full-scope runs establish the
repair; no unverified claim that the earlier failure was *merely* a trigger
issue or a target-source counterexample is required.

Deleting the exact added block recovers the original source BYTE FOR BYTE.
Therefore all original signatures, requires clauses/input domain, executable
statements, original assertions and the other two methods are preserved.
There are no deleted lines, added premises, narrowed domains, new specs, axioms,
assumptions, external bodies, admits, lifetime/proof skips, unsafe additions,
inference generation/admission changes or application/native body changes.
No auxiliary proof lemmas were needed (0 of 4).

Original source SHA256:
036505f3ec263621a2b5b1b81380625ad57e7170c4717750e027bd35f8c29477

Repaired/restored source SHA256:
714e854e227d04c300ac8be1dfcb860f53d1ef0098cc6c6bb048482ecf078363

Retained identities: generic_partition_bytes, midpoint_general,
raw_singleton_spare_tail in crate review_generic. One additional proof identity
is mathematical_assertion_control, preserved in two body variants; conservatively
charging both variants separately is 2/8 beyond the retained three. Additional
exec methods: zero. New spec identities: zero. Auxiliary lemmas: zero.
See method-ledger.json and source-preservation-audit.json.

## Input/runtime authentication and preservation

Entry and exit independently authenticate the externally pinned review SEAL:
286358f51477c3f14193f5230a047759f26e0f17c7655fc63427971809129e53

And parent SEAL:
39cd35936577c41b43949776012c984a874892729b4da5fa15760fc71721b04c

Both MANIFEST hashes, exact payload membership (203 review / 1,676 parent), all
file sizes and SHA256s, regular/non-symlink/read-only payload status match at both
endpoints. No historical runner, auditor, finalizer, or sealed script was run
in place, and no historical artifact was modified. Runtime binaries were neither
copied nor rebuilt. The existing nine-file private midpoint runtime authenticates
exactly, including verus, rust_verify, z3, vstd.vir, libvstd.rlib and macro dylibs.

All six ordinary import files are reused read-only from the authenticated review:
provider attempt001, builder attempt003 importing001, client attempt004
importing001/003. Their hashes and source hashes are recorded in each invocation
and the entry/exit authentication files. NO operational91/current-parent driver
or parent final rlib was substituted. The logical argv, release setting,
`--no-cheating`, `--num-threads 1`, resource flags and sealed env are unchanged,
except scratch-local writable paths and the explicitly disclosed paired-control
export request. The sysroot is exactly the specified 1.99.1-aarch64-apple-darwin.

All 3,340 historical sysroot inventory files match size/content/symlink targets
at entry and exit, with exact current membership checked at exit. This full
inventory is historical baseline corroboration, NOT itself a newly externally
pinned seal. Additionally all 61 operational compiler/loader/target-library hashes
have direct pins in the authenticated review's preservation-before.json and match
both endpoint inventories. This includes rustc, rustc_driver, LLVM, core/std and
target libraries; the exact DYLD_LIBRARY_PATH is preserved in every raw env.
sysroot-pin-corroboration.json makes the 61/3340 distinction explicit.
No whole toolchain or historical package was repeatedly hashed at minor proof
iterations; full endpoint authentication is separate from source-only iterations.

The parent PLAN hash also matches entry/exit. No parent/frozen-input/runtime,
native source or Git writes were issued. Endpoint equality is not a transient
write, actor, unrelated live-worktree, shared-ref or objectstore attestation.

## Evidence and small metadata corrections

Each fresh runs/attempt-NNN preserves actual argv, complete env, cwd, matched
runtime and import pins, source/runner snapshots and hashes, raw stdout/stderr,
parsed backend JSON, receipt/exit/timeout/export outcomes. The append-only
application-ledger.jsonl reserves each attempt before process spawn. All source
drafts, proof-only patch and every runner version used are retained.

Attempt001's first harness scope extractor used the wrong JSON key, leaving its
scope.json/receipt scope as {}. The RAW stdout already contains the complete
three-function FAILED scope. The runner/receipt are preserved unchanged. The
correct key is verus-infer.verification_scope; derived-scope-001.json records the
raw failed scope, and derived scope files normalize all five applications without
rerunning the verifier. This was a metadata defect, not a scope-filtered run.

An exit-audit metadata draft also incorrectly expected all 3,340 historical
inventory files in the review's sealed 61-file operational subset. That draft
raised KeyError for bin/cargo; the following audit then lacked exit JSON. Both
drafts are preserved, the distinction is repaired/disclosed, and final endpoint
authentication plus audit PASS. Those two Python metadata-process failures did
NOT invoke the verifier or earn any proof credit. metadata-repair-log.json keeps
them explicit; no extra verifier application is hidden by these corrections.

All hard NEW limits are met: 5/16 direct applications, one worker, each <60s,
<15min total from first task command, scratch <128MiB, global free >16GiB at
every application boundary and final seal. Exact final budget is in SEAL.json.
Compiler/vstd/Cargo/harness builds, network, subdelegation, Git writes, native
application executions and runtime/toolchain copies are all zero.

## Unchanged acceptance boundary / handoff

The private midpoint and six LE std bindings remain EXPLICIT INHERITED
CONDITIONAL ASSUMPTIONS. They are not newly accepted Rust-core proofs, new trust,
or live adoption. Original native source path preservation FAILED and original
native Git continuity remains UNAVAILABLE; the recovered content matching redb
pin c82ab865bb362bc4af067d332adc44978ff64813 remains qualified input only.
Authenticating the parent package does NOT repair that historical loss.

No native builder/Drop/borrow/panic/full-trait ABI, producer lifecycle, database,
whole-crate-native or final-cell acceptance follows. No native builder-debug
coverage is inferred from release-only client imports. Excluded growth049,
zero-growth050, mutation030, padding95 and refused drop-pipeline work are untouched.
No T3 adoption, PR, commit or shared-ref assertion is made. Parent must inspect
the exact delta and commission fresh independent review before relying on closure.
All 18 final redb cells remain OPEN. No further work is performed after handoff.
