# Native reference88

This is an authored reference proof, not an Infer feature or an inference
certification. `reference.rs` contains the exact pinned redb struct and nine
executable bodies, with contracts, ghost snapshots, checked proof blocks and
inclusive-loop invariants. `audit.py` compares all executable tokens after its
narrow, enumerated erasure. Native comments are retained; no native computation
or statement moves into a ghost context.

The only new trust is the `u32::div_ceil` correspondence in
`vstd/std_specs/num.rs`: all `u32` numerators and nonzero `u32` divisors, using
unbounded mathematical addition before division. This is not verification of
Rust core. The numeric macro is unchanged. Existing matched-vstd specifications
for Vec allocation/resize/index/mutable borrow, Clone, saturating subtraction,
minimum and inclusive range iteration remain library trust, not new app trust.

`resized` describes every stored bit, not just logical bits. It specifies exact
logical length and representation, maximum old/required word count, growth
initialization, preserved old padding/spares, fresh-word defaults including
padding, and unchanged storage on shrink. Preconditions impose only storage
validity; accesses additionally need their genuine index/shift safety premise.
Neither minimal storage nor old/new ordering bounds the resize theorem.

Four authored clients execute native methods: resize/read, set/read/clear/read,
shrink/regrow/read, and resize/set/read/clear/read. `import.rs` calls their real
exported ordinary contracts from a fresh crate. No dummy inference export is
used and ordinary imports are not represented as certified inference artifacts.

`expected.json` fixes the body inventory, trust and expected counts. The runtime
producer is derived from the actual compiler/vstd source map, not HEAD. `run.py`
uses private offline copied Cargo caches, homes and targets, pins the absolute
local toolchain, counts each verifier launch before spawn, retains timeouts and
raw output, and withholds failed providers. Budgets are one compiler build, one
matched-vstd run and 64 implementation application launches; tests are separate.
Do not invoke `build` again after its single slot is consumed. A staging-only
`vstd` resume is allowed only after the successful recorded compiler build and
exact equality with its original compiler source map.

The implementation output is
`/private/tmp/verus-native-reference88.w3Ws2g/implementation`. Source fixtures,
commands, inputs, logs, runtime hashes, controls and original guard observations
are retained there. A failed frontend/infra control is not a semantic kill.
The first `wrong_method` exploratory expectation was incorrect: `checked_div`
already had a spec and verified. Its result is retained with zero kill credit;
`wrong_method_ceiling` instead checks that its existing floor semantics cannot
be confused with the new ceiling correspondence.

The historical runner's `required_words_blocker` expectation remains historical:
its old unsupported diagnostic is no longer true on this runtime. The permanent
source-built command is now:

```sh
python3 -B source/tools/check_redb_stateful.py \
  --native-checkout /path/to/clean/pinned/redb \
  --output /path/outside/the/repository/new-reference-replay \
  --cargo-cache /path/to/existing/offline/cargo-home \
  --resize-provider reference88
```

It builds a fresh source-bound runtime and matched vstd, preserves the existing
transition/access negative gates, checks the bridge, audits all nine native
bodies and struct, and verifies a fresh ordinary imported-contract consumer.
Without the explicit complete provider, the default track still exits 2 for an
incomplete resize goal. The implementation tools here are not independent
review or an inference certificate. New independent review remains required;
no overall goal completion or automation success is claimed by this package.
