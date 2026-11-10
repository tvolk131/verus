# Independent fix review

**Request changes for R1/P2 (incomplete F1 alias closure).** The original
direct-write repros are fixed; declaration/match-pattern aliases still select
incompatible snapshots and reject valid programs. See [FINDINGS.md](FINDINGS.md).
F2 is accepted: the exact regression passes with the four-profile inventory
and all prior disabled/skipped/determinism assertions retained.

Review base c9d072ad; production sources match engine ea897a9b. This is a new review of F1/F2 and composition, not whole-goal
or native-correspondence acceptance. No subdelegates or prior review children were used.

## Source inspection

The production delta adds `NativeMutableFor.iterator_init` and a pre-decoration
eligibility guard. Its constructor exemption is the exact recognized native
`iter_mut` expression with a closed coercion signature and direct input borrow.
The recognized loop exemption avoids treating its intended byte updates as
surrounding mutations. Ghost blocks remain subject to ordinary mode checking;
they cannot mutate executable inputs. All `PlaceX` projection variants are
handled by `may_reach`; temporary roots reject conservatively. DFS also visits
side-effect expressions within place/index/resolution wrappers and nested
branches. The preceding descriptor's 16,384-expression budget bounds the body
before the guard; multiple loops/constructors and unsupported native shapes skip.

Confirmed R1/P2: a declaration initializer and match
scrutinee are VIR places. Pattern moves and `ref mut` bindings are represented
through the pattern, rather than necessarily through the expression nodes
matched by this guard. No alias analysis connects such newly bound locals to the
input. Four privately copied, pretrial-frozen programs verify 2/0 off and fail
1/1 on, only at the generated entry invariant; the observer actually applies.

Whole-Function projection, independent scaffold/freshness validation, typed
catalog signatures/bounds, and the numeric-before-observer pipeline are unchanged.
The observer snapshots and invariants are ordinary checked proposals. Installed
bodies request full WF and mode checks, refresh resolution maps, and then undergo
normal body/lifetime/trait checking. A skipped observer returns the incoming
function, retaining earlier numeric proposals. No theorem, contract, model,
assumption, termination or publication gate was changed; policy/cache/schema
remain 68/71/38.

F2 changes only the exact public profile inventory to the four production entries
(`growth_loop`, `pointwise_transport`, `range_loop`,
`mutable_buffer_u64_radix128`). Disabled/preparation flags, nine per-function
skipped decisions, exact helper reporting, and cold/warm determinism assertions
remain intact. The exact reporting regression passed **1/1** (298 intentionally
filtered by `--exact`). The full `infer` target also passed **299/299**.

## Fresh evidence

- Own release compiler/macros and vstd: **2,226/0**. Rust **1.99.1**,
  Z3 **4.16.0**, producer **0.2026.11.10.c9d072a**; production bytes bound
  separately to ea897a9b. No borrowed compiler or vstd validation counts.
- VIR: **148 passed**, zero failed/ignored/filtered.
- Compiler targets: observer **13/13**, numeric **8/8**, full `infer` **299/299**:
  **320 passed**, zero failed/ignored/filtered across these three targets. The
  explicit F2-only run also passed **1/1** and is not double-counted in 320.
- Canonical: **26 expected outcomes**, replay and independent audit pass.
  Original frozen cold/warm **6/0**, exec `{u64}`, proof `{}`, helpers `{}`.
  Healthy `identity` helpers require soundness/maximality/exactness; intentional
  cached poison has required statuses `not_proved/proved/not_proved` and cannot
  publish. Import identity, invalid metadata and failed-export cleanup pass.
- Observer: **23 expected outcomes**, replay and additional independent audit
  pass with explicit body/helper identities. Prefix **2/0 on vs 1/1 off**;
  full HMAC **1/1** postcondition failure cold/warm. Manual canonical **9/0**
  and plain **2/0** remain diagnostic successes both ways.
- Additional frozen probes: **18 sources / 36 off/on runs**, including four
  valid-program rejection regressions (R1). Direct before/after writes pass
  **2/0 both ways**, explicit helper/alias writes **3/0**, mutable escape **2/0**,
  nested target mutation **2/0**, projected/returned-helper mutation **3/0**;
  all correctly skip the observer. Ghost-spec read remains **2/0** with observer.
  Unrelated-array prefix remains **2/0 on vs 1/1 off**. Combined numeric/prefix
  code verifies **8/0**, exact exec `{u64,xor_key}`, proof/helpers `{}`, and both
  observations selected. Its **3/3 off** result is diagnostic, not a strict
  semantic negative. Skipped-observer overflow still rejects **1/1** and illegal
  aliases reject Rust **E0499** in both modes; frontend failures are separate.

The actual exported provider, rlib, metadata trailer, inventory of 359 trusted-model
bindings,
`identity` proof statuses and importing caller's matching provider fingerprint
were checked. These are artifact consistency checks, not cryptographic validation
of malicious metadata. All generated conditions remain checked proposals.

## Reproduction

Build from `source/`, with `/home/dev/.cargo/bin` first in PATH,
`CARGO_BUILD_JOBS=2` and explicit `VERUS_Z3_PATH`:

```sh
cargo build --release --locked -p rust_verify -p verus -p cargo-verus -p vstd_build
cargo generate-lockfile --offline --manifest-path vstd/Cargo.toml
# Prepend this checkout's target-verus/release to PATH:
target-verus/release/cargo-verus verus build --release --locked --offline --manifest-path vstd/Cargo.toml -- --num-threads 2
# From the checkout root:
python3 -B -O evidence/shared-buffer-fix-review/probe.py --raw /tmp/shared-buffer-fix-review-c9d072ad --freeze-only
bash evidence/shared-buffer-fix-review/validate.sh
bash evidence/shared-buffer-fix-review/replay.sh
```

Private raw evidence: `/tmp/shared-buffer-fix-review-c9d072ad/`.
`probe.py --freeze-only` writes and hashes all independent repros before trials;
`probe.py` checks named bodies/helpers and explicit observer selections. Final
probe sources are independent private byte copies, guarded before/after every
trial, and compared against the committed repro bytes. The initial same-byte
checkout-source trials are preserved separately and do not supply final counts.
The recorded run used the default raw directory. For another run, select a new
`REVIEW_RAW` directory (create it before freezing), pass the same directory to
`probe.py --raw`, and reuse the built compiler. Existing bundles deliberately
reject overwrite. `replay.sh` creates an independent immutable bundle from this checkout's fresh
build, then runs probes, canonical replay/audit, observation replay, and the
additional independent `audit.py`. Every new application invocation has
`--no-cheating`; no whole-body/lifetime/trait filters or bypasses are used.
`validate.sh` runs the full VIR unit suite, both focused compiler targets and
full `infer` target; supplying target names runs only those targets. A process
interruption (exit 143) during dependency compilation produced no focused test
outcome; its log is retained and the compiler targets were restarted separately.
A second exit-143 interruption affected a partial full-`infer` run. Its passing
rows are not counted as a completed suite; the full target was retried with a
terminal session and completed 299/299. Causes were not established. Normal compiler regressions
preserve intentional policy negative options. Normal vstd retains its inherited 83 admissions.

Two derived frozen fixtures retain a final blank line from fixture composition;
`git diff --check` reports those two EOF whitespace warnings. Their bytes are
kept identical to the pretrial freeze and private compiled copies.

## Scope limits

Domains remain every u64/arbitrary initialized exclusive `[u8;10]` and arbitrary
initialized `[u8;128]`/every valid immutable key length, including empty and keys
longer than 128. No zero-buffer, layout or narrower-domain assumption was added.
Trust still includes Verus/rustc/macros/SMT and inherited iterator/prophetic-reference
specifications; no new native entry, pointer, helper/drop or frame law was proved.

Original frozen canonical remains required; full frozen HMAC still requires both
XOR-prefix and untouched-suffix. Prefix-only proofs do not discharge the full
property. Native authored-model 115/0 is conditional research (not rebuilt here), not an established
input/final-array adequacy or native primitive theorem. Actual native
correspondences remain zero and native publication remains false. A failed true
native-drop is an unproved positive, not a native counterexample.

Parent's remaining compiler targets and historical Python results are not
substituted for fresh evidence in this review. No parent polling/waiting. No
production/application/parent/sibling files, policy/cache/schema or global
settings were edited; the ignored solver is a permitted private byte copy.
