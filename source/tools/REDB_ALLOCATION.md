# Bounded native allocation replay

`check_redb_allocation.py` is an offline qualification replay, not a proof
certificate, general acceptance framework or whole-redb verification.
It launches exactly eleven applications. No compiler build, dependency fetch,
network operation, vstd bootstrap or Rust test-harness build is part of replay.

```sh
python3 -B source/tools/check_redb_allocation.py \
  --output /private/tmp/new-allocation-replay \
  --native /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --host-runtime /private/tmp/verus-shift93-0GSYwzPc/runtime \
  --target32-runtime /private/tmp/verus-operational93-20261105/cross32/runtime \
  --host-sysroot /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin \
  --target32-sysroot /private/tmp/verus-operational93-20261105/cross32/sysroot
```

The output must not exist. `--plan-only` performs hash/budget preflight without
launching any verifier and exits 2, granting no theorem credit. An incomplete
run also exits 2 and retains every attempted invocation. A completed replay
exits 0 but explicitly remains pending independent human acceptance. Expected
failure cases must fail in their exact intended categories. The fixed scope,
body rows, runtime and sysroot hashes are pinned by the qualification digest;
the current output is never used to invent its own success oracle.

## What is established

The authored provider retains sixteen native bodies, including constructor,
clear and allocation, and adds three executable compositions with twenty-six
authored checked proof bodies. The constructor retains all u32 input pairs;
the operational invariant requires `pages <= 1_048_576` and
`pages <= 64 * ceil(capacity / 64)`, not `pages <= capacity`. The raw constructor
starts all-full: two allocations return `None, None`. Successful allocations
require native clear initialization. The authored clear-all lifecycle proves
zero-, one-, and multi-page least-free/distinct results, representation and exact
frames. Ten ordinary imported consumers check against freshly produced output.
These are conditional contracts, not proof that production callers establish
the conditions or that the complete allocator/header/recovery stack is correct.

Infer's measured gain is deliberately narrow: the reduced provider removes one
original checked local-shift bridge. Semantic contracts, snapshots, invariants,
most BV lemmas and sixteen extra checked frame-proof instantiations remain
authored. Native bodies and domains are not rewritten to make inference pass.
Queries are not counts of certified native bodies: the full report's complete
selected-body inventory is separately reconciled.

## Execution and trust boundaries

The launcher uses the exact hashed compiler and explicitly selected, fully
hashed sysroots; `VERUS_USE_RUSTUP=0` removes an unrelated launcher dependency,
not a verification/erasure/lifetime check. All writable homes and caches are
private. Host applications retain `--no-cheating`. The three existing explicit
layout fixtures follow the standard layout-test convention and instead validate
the real i686 target by full erasure compilation and ELF32/i386 object headers.
This is cross-compilation, not execution on physical 32-bit hardware. The layout
declaration is not counted as a checked executable body.

Negative reports must reconcile outcomes, exact source/body scope and intended
diagnostic locations. Command, environment and consumed byte bindings are
checked independently of report totals. Seeded failure must withdraw the VIR
while retaining an unchanged old rlib; that rlib is not a current certificate.
A later missing-interface import earns only import-gate credit, not a semantic
proof failure. Synthetic Python tests likewise earn no application proof credit.
Receipts are consistency evidence within this local execution trust boundary,
not cryptographic execution attestation.

The historical lifecycle research accidentally attempted `git write-tree`.
HEAD, index, branch and guarded files were preserved, but historical Git
object-store mutation remains **NOT_CERTIFIED**. No cleanup or rollback of the
object store is attempted. Old goals and production-caller establishment remain
open. Current exact tool/fixture provenance is in the qualification manifest;
historical source bindings are never silently relabeled after tooling changes.

Timings separate preflight, each application and total wall time. A previous
within-boundary hash-deduplication experiment remains diagnostic-only: no global
or stat-only hash cache is deployed. The most useful tooling improvement here
is one fixed preflighted command, exact planned launch demand, and preserved
attempt ledgers rather than another acceptance framework.
