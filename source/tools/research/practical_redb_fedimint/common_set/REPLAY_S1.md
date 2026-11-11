# S1 local replay

Implementation commits: scope `9d4d7dc0`, fixture freeze `e51548fa`, implementation
`b06ec2e9`, frontend-shape/typed-discovery correction `5b733f1e`. These are local
proposals for parent review and combined acceptance, not goal completion.

Private output root: `/tmp/practical-set-s1-3b931894`. `final-bundle/receipt.json`
pins all eight runtime files and the exact production source hashes. Use
`final-bundle`, not the Cargo target: that target also built the deliberately
disabled S1 ablation. `accepted75` is the privately copied, SHA-verified eight-tool
baseline and original receipt from SELECTION_R2_F2. The registered solver SHA is
unchanged. `source-pins.json` authenticates all five selected project source files.

For an independent local replay with the materialized immutable runtime:

```sh
. /tmp/practical-set-s1-3b931894/env.sh
python3 source/tools/research/practical_redb_fedimint/common_set/replay_s1.py \
  --bundle /tmp/practical-set-s1-3b931894/final-bundle \
  --output /tmp/s1-independent-new-output
```

The output path must be new. The controller uses existing `LoggedCommands`,
`replay_receipt`, `ArtifactGuard`, inference exactness/coverage and publication
checks. It retains raw stdout/stderr, commands, tool/source hashes, failure names,
full local body inventories, timing and pair observations. Apps always use
`--no-cheating`; no application defines or calls an authored pair helper.
`--baseline` expects the two reduced fixtures to fail with goals off and on,
and omits the later positive/control suite. It works with accepted75 or the
private S1-disabled ablation bundle.

For a fresh build, use an isolated Cargo home/target and exactly the authenticated
archives/index/Git objects in the two approved manifests. The private copies and
original manifests are retained under the output root. The omitted empty Git refs
directory was recreated privately and `refs/remotes/origin/parse-partial` was set
to authenticated object `1ba7b831cfa9ebf9dea584ba43c71ea49613267f`; no fetch occurred.
Use the pinned Rust 1.99.1 toolchain, `RUSTC_BOOTSTRAP=1`, jobs2, offline Cargo and
the four cfg flags from `tools/vargo/src/commands/mod.rs`:

```sh
export RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt'
cargo build --offline --manifest-path source/Cargo.toml \
  -p rust_verify -p verus -p verus_builtin -p verus_builtin_macros \
  -p verus_state_machines_macros -p vstd_build
cargo test --offline --manifest-path source/Cargo.toml -p vir -p air --lib
```

Copy the five newly built executable/macro/builtin files into a new private bundle,
add the authenticated solver and `verus-root`, then from `source/` run the newly
built `vstd_build <absolute-new-bundle> --verbose` with `VERUS_IN_VARGO=1` and
`VERUS_Z3_PATH` set. This performs NORMAL full vstd verification and compilation;
no `--no-verify`, lifetime/trait suppression, or `--no-cheating` forwarded into
existing vstd admissions. Only use the resulting library after the build succeeds.
The standalone exact C0 helper proof is also retained as `helper-no-cheating.rs`
with its successful no-cheating output.

`ablation-build.py/json` retains the one-line private pipeline ablation, exact
before/disabled/restored hashes and build command. Only S1's call to `prepare` was
disabled. All other goal profiles and the checked library stayed enabled. The
original pipeline bytes were restored in `finally`; this is not a production flag.

For later composition, S1 runs after preceding preparation passes. One allocator
reserves every serialized integer in ALL current local and imported function ASTs
and retains allocations across ALL decorated functions. Its independent audit
checks Expr, Place, read and statement IDs against that accumulated reservation.
A later S2 pass must seed from the current S1-decorated output (and all earlier
outputs) in the same way; S1 cannot authorize a later pass to reuse an old seed.
There is no claimed hardcoded disjoint range or within-function-only guarantee.
