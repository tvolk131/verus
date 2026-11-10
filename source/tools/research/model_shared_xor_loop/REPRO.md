# Replay

From this prepared checkout, with the named immutable completed dependencies
and pinned foundation still present, choose a fresh private output path:

```sh
python3 -B source/tools/research/model_shared_xor_loop/check.py \
  --freeze source/tools/research/model_shared_xor_loop/trial011-freeze.json \
  --output /tmp/model-shared-xor-loop-replay-fresh
```

The controller authenticates readiness AND the successful durable baseline
receipt; 523 foundation sources, 284 objects, runner/tools, loadpaths and GMP
manifest SHA256 `332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
It authenticates completed library receipt/source/object identities using
committed pins. It removes inherited Coq/OCaml/library overrides and writes
only to the fresh private output directory. No installation or shared build
is performed. Normal coqc compilation, per-declaration assumptions/proof terms,
and unrestricted recursive coqchk are mandatory. Every frozen input and
prerequisite stays guarded through final checking.

A changed candidate/controller requires a new pretrial freeze (never overwrite
an existing freeze), then a fresh trial directory. The accepted freeze is not
a shortcut around prerequisite authentication. `EVIDENCE.json` identifies all
retained trials; failures are development/setup failures, not semantic negatives.
